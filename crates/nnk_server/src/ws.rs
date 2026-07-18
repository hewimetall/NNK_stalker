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
    let lobby = ServerMsg::Lobby(LobbyView::from_state(&room));
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

    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let text = serde_json::to_string(&msg).unwrap();
            if sink.send(Message::Text(text.into())).await.is_err() {
                break;
            }
        }
    });

    while let Some(Ok(msg)) = stream.next().await {
        let Message::Text(text) = msg else { continue };
        let Ok(client_msg) = serde_json::from_str::<ClientMsg>(&text) else {
            let _ = live.tx.send(ServerMsg::Error {
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
                let _ = live
                    .tx
                    .send(ServerMsg::Lobby(LobbyView::from_state(&out.state)));
                let _ = live.tx.send(ServerMsg::RoomState(out.state));
            }
            Err(e) => {
                let _ = live.tx.send(ServerMsg::Error {
                    code: "reject".into(),
                    message: e.to_string(),
                });
            }
        }
    }

    send_task.abort();
}
