use crate::state::AppState;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Path, Query, State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use futures_util::{SinkExt, StreamExt};
use nnk_domain::MemberRole;
use nnk_protocol::{ClientMsg, LobbyView, ServerMsg};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct WsQuery {
    pub token: String,
}

/// Must-auth WebSocket: `/ws/rooms/{code}?token=<jwt>`
pub async fn ws_handler(
    State(state): State<AppState>,
    Path(code): Path<String>,
    Query(query): Query<WsQuery>,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let (user_id, username) = state.auth.verify_token(&query.token).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "invalid token" })),
        )
    })?;
    let code = code.to_uppercase();
    let role = state.rooms.member_role(&code, user_id).await.map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "not a room member — join via REST first" })),
        )
    })?;
    let live = state.live.get(&code).map(|r| r.clone()).ok_or((
        StatusCode::BAD_REQUEST,
        Json(json!({ "error": "room not in memory" })),
    ))?;

    let rooms = state.rooms.clone();
    Ok(ws.on_upgrade(move |socket| async move {
        handle_socket(socket, live, rooms, user_id, username, role, code).await;
    }))
}

async fn handle_socket(
    socket: WebSocket,
    live: std::sync::Arc<crate::state::LiveRoom>,
    rooms: crate::state::SharedRooms,
    user_id: nnk_domain::UserId,
    _username: String,
    role: MemberRole,
    code: String,
) {
    let (mut sink, mut stream) = socket.split();
    let mut rx = live.tx.subscribe();

    let welcome = ServerMsg::Welcome { user_id, role };
    let room = live.state.read().await.clone();
    let lobby = ServerMsg::Lobby(LobbyView::from_state_for(&room, Some(user_id), role));
    let snapshot = ServerMsg::RoomState(room);
    if sink
        .send(Message::Text(
            serde_json::to_string(&welcome).unwrap().into(),
        ))
        .await
        .is_err()
    {
        return;
    }
    let _ = sink
        .send(Message::Text(serde_json::to_string(&lobby).unwrap().into()))
        .await;
    let _ = sink
        .send(Message::Text(
            serde_json::to_string(&snapshot).unwrap().into(),
        ))
        .await;

    let (out_tx, mut out_rx) = tokio::sync::mpsc::unbounded_channel::<ServerMsg>();
    let out_tx_broadcast = out_tx.clone();

    let send_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                biased;
                maybe = out_rx.recv() => {
                    let Some(msg) = maybe else { break };
                    let text = serde_json::to_string(&msg).unwrap();
                    if sink.send(Message::Text(text.into())).await.is_err() {
                        break;
                    }
                }
                recv = rx.recv() => {
                    let Ok(msg) = recv else { break };
                    match msg {
                        // Bare Lobby broadcasts have empty legal_actions — ignore.
                        ServerMsg::Lobby(_) => {}
                        ServerMsg::RoomState(ref room) => {
                            let lobby = ServerMsg::Lobby(LobbyView::from_state_for(
                                room,
                                Some(user_id),
                                role,
                            ));
                            for m in [lobby, msg] {
                                let text = serde_json::to_string(&m).unwrap();
                                if sink.send(Message::Text(text.into())).await.is_err() {
                                    return;
                                }
                            }
                        }
                        other => {
                            let text = serde_json::to_string(&other).unwrap();
                            if sink.send(Message::Text(text.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                }
            }
        }
    });

    while let Some(Ok(msg)) = stream.next().await {
        let Message::Text(text) = msg else { continue };
        let Ok(client_msg) = serde_json::from_str::<ClientMsg>(&text) else {
            let _ = out_tx_broadcast.send(ServerMsg::Error {
                code: "bad_json".into(),
                message: "cannot parse ClientMsg".into(),
            });
            continue;
        };
        let Some(action) = client_msg.into_action() else {
            continue;
        };
        let current = live.state.read().await.clone();
        match rooms.apply(current, user_id, role, action) {
            Ok(out) => {
                *live.state.write().await = out.state.clone();
                let _ = rooms.persist_snapshot(&code, &out.state).await;
                // Peers rebuild personalized Lobby from RoomState in send_task.
                let _ = live.tx.send(ServerMsg::RoomState(out.state));
            }
            Err(e) => {
                let _ = out_tx_broadcast.send(ServerMsg::Error {
                    code: "reject".into(),
                    message: e.to_string(),
                });
            }
        }
    }

    drop(out_tx_broadcast);
    send_task.abort();
}
