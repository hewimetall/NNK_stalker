use crate::state::HealthState;
use bevy::prelude::*;
use nnk_protocol::LobbyView;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Mutex;

#[derive(Resource, Clone)]
pub struct NetTx(pub Sender<NetEvent>);

#[derive(Resource)]
pub struct NetRx(pub Mutex<Receiver<NetEvent>>);

#[derive(Debug)]
pub enum AuthMode {
    Login,
    Register,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum NetEvent {
    Health {
        state: HealthState,
        message: String,
    },
    Authenticated {
        token: String,
        user_id: String,
        username: String,
    },
    Lobby {
        lobby: LobbyView,
        connect_ws: bool,
    },
    WsStatus {
        generation: u64,
        message: String,
    },
    WsError {
        generation: u64,
        message: String,
    },
    Error {
        message: String,
    },
}

pub struct NetPlugin;

impl Plugin for NetPlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = mpsc::channel();
        app.insert_resource(NetTx(tx))
            .insert_resource(NetRx(Mutex::new(rx)));
    }
}

#[cfg(target_arch = "wasm32")]
mod imp {
    use super::*;
    use futures_util::StreamExt;
    use gloo_net::http::Request;
    use gloo_net::websocket::{futures::WebSocket, Message};
    use nnk_protocol::{AuthRequest, AuthResponse, CreateRoomResponse, ServerMsg};
    use serde::{Deserialize, Serialize};
    use serde_json::Value;
    use wasm_bindgen_futures::spawn_local;

    const TOKEN_KEY: &str = "nnk_token";

    #[derive(Debug, Deserialize)]
    struct MeResponse {
        user_id: String,
        username: String,
    }

    #[derive(Debug, Serialize)]
    struct ReadyBody {
        ready: bool,
    }

    pub fn stored_token() -> Option<String> {
        storage()
            .and_then(|storage| storage.get_item(TOKEN_KEY).ok().flatten())
            .filter(|token| !token.is_empty())
    }

    pub fn store_token(token: &str) {
        if let Some(storage) = storage() {
            let _ = storage.set_item(TOKEN_KEY, token);
        }
    }

    pub fn clear_stored_token() {
        if let Some(storage) = storage() {
            let _ = storage.remove_item(TOKEN_KEY);
        }
    }

    fn storage() -> Option<web_sys::Storage> {
        web_sys::window().and_then(|window| window.local_storage().ok().flatten())
    }

    pub fn spawn_health(tx: Sender<NetEvent>) {
        spawn_local(async move {
            let event = match Request::get("/health").send().await {
                Ok(response) if response.ok() => NetEvent::Health {
                    state: HealthState::Online,
                    message: "сервер онлайн".into(),
                },
                Ok(response) => NetEvent::Health {
                    state: HealthState::Offline,
                    message: format!("сервер ответил HTTP {}", response.status()),
                },
                Err(error) => NetEvent::Health {
                    state: HealthState::Offline,
                    message: format!("сервер недоступен: {error}"),
                },
            };
            let _ = tx.send(event);
        });
    }

    pub fn spawn_validate_token(token: String, tx: Sender<NetEvent>) {
        spawn_local(async move {
            match get_json::<MeResponse>("/api/me", &token).await {
                Ok(me) => {
                    let _ = tx.send(NetEvent::Authenticated {
                        token,
                        user_id: me.user_id,
                        username: me.username,
                    });
                }
                Err(error) => {
                    clear_stored_token();
                    let _ = tx.send(NetEvent::Error { message: error });
                }
            }
        });
    }

    pub fn spawn_auth(mode: AuthMode, username: String, password: String, tx: Sender<NetEvent>) {
        spawn_local(async move {
            let path = match mode {
                AuthMode::Login => "/api/auth/login",
                AuthMode::Register => "/api/auth/register",
            };
            let body = AuthRequest { username, password };
            match post_json::<AuthRequest, AuthResponse>(path, None, &body).await {
                Ok(auth) => {
                    store_token(&auth.access_token);
                    match get_json::<MeResponse>("/api/me", &auth.access_token).await {
                        Ok(me) => {
                            let _ = tx.send(NetEvent::Authenticated {
                                token: auth.access_token,
                                user_id: me.user_id,
                                username: me.username,
                            });
                        }
                        Err(error) => {
                            let _ = tx.send(NetEvent::Error { message: error });
                        }
                    }
                }
                Err(error) => {
                    let _ = tx.send(NetEvent::Error { message: error });
                }
            }
        });
    }

    pub fn spawn_create_room(token: String, tx: Sender<NetEvent>) {
        spawn_local(async move {
            let result = async {
                let created = post_empty::<CreateRoomResponse>("/api/rooms", &token).await?;
                let path = format!("/api/rooms/{}/join", created.code);
                post_empty::<LobbyView>(&path, &token).await
            }
            .await;
            send_lobby_result(result, true, tx);
        });
    }

    pub fn spawn_join_room(code: String, token: String, tx: Sender<NetEvent>) {
        spawn_local(async move {
            let path = format!("/api/rooms/{}/join", code.trim().to_uppercase());
            let result = post_empty::<LobbyView>(&path, &token).await;
            send_lobby_result(result, true, tx);
        });
    }

    pub fn spawn_ready(code: String, ready: bool, token: String, tx: Sender<NetEvent>) {
        spawn_local(async move {
            let path = format!("/api/rooms/{}/ready", code.trim().to_uppercase());
            let body = ReadyBody { ready };
            let result = post_json::<ReadyBody, LobbyView>(&path, Some(&token), &body).await;
            send_lobby_result(result, false, tx);
        });
    }

    pub fn spawn_start(code: String, token: String, tx: Sender<NetEvent>) {
        spawn_local(async move {
            let path = format!("/api/rooms/{}/start", code.trim().to_uppercase());
            let result = post_empty::<LobbyView>(&path, &token).await;
            send_lobby_result(result, false, tx);
        });
    }

    pub fn spawn_refresh_lobby(code: String, token: String, tx: Sender<NetEvent>) {
        spawn_local(async move {
            let path = format!("/api/rooms/{}", code.trim().to_uppercase());
            let result = get_json::<LobbyView>(&path, &token).await;
            send_lobby_result(result, false, tx);
        });
    }

    pub fn spawn_ws(code: String, token: String, generation: u64, tx: Sender<NetEvent>) {
        spawn_local(async move {
            let Some(url) = ws_url(&code, &token) else {
                let _ = tx.send(NetEvent::WsError {
                    generation,
                    message: "не удалось собрать WebSocket URL".into(),
                });
                return;
            };
            let mut ws = match WebSocket::open(&url) {
                Ok(ws) => ws,
                Err(error) => {
                    let _ = tx.send(NetEvent::WsError {
                        generation,
                        message: format!("WS не подключен: {error}"),
                    });
                    return;
                }
            };
            let _ = tx.send(NetEvent::WsStatus {
                generation,
                message: "WS подключается...".into(),
            });
            while let Some(message) = ws.next().await {
                match message {
                    Ok(Message::Text(text)) => handle_ws_text(&tx, generation, &text),
                    Ok(Message::Bytes(_)) => {}
                    Err(error) => {
                        let _ = tx.send(NetEvent::WsError {
                            generation,
                            message: format!("WS ошибка: {error}"),
                        });
                        break;
                    }
                }
            }
            let _ = tx.send(NetEvent::WsStatus {
                generation,
                message: "WS отключен".into(),
            });
        });
    }

    fn handle_ws_text(tx: &Sender<NetEvent>, generation: u64, text: &str) {
        match serde_json::from_str::<ServerMsg>(text) {
            Ok(ServerMsg::Lobby(lobby)) => {
                let _ = tx.send(NetEvent::Lobby {
                    lobby,
                    connect_ws: false,
                });
            }
            Ok(ServerMsg::Error { message, .. }) => {
                let _ = tx.send(NetEvent::WsError {
                    generation,
                    message,
                });
            }
            Ok(_) => {}
            Err(error) => {
                let _ = tx.send(NetEvent::WsError {
                    generation,
                    message: format!("WS JSON: {error}"),
                });
            }
        }
    }

    fn ws_url(code: &str, token: &str) -> Option<String> {
        let location = web_sys::window()?.location();
        let protocol = if location.protocol().ok()?.starts_with("https") {
            "wss"
        } else {
            "ws"
        };
        let host = location.host().ok()?;
        Some(format!(
            "{protocol}://{host}/ws/rooms/{}?token={}",
            code.trim().to_uppercase(),
            token
        ))
    }

    fn send_lobby_result(
        result: Result<LobbyView, String>,
        connect_ws: bool,
        tx: Sender<NetEvent>,
    ) {
        match result {
            Ok(lobby) => {
                let _ = tx.send(NetEvent::Lobby { lobby, connect_ws });
            }
            Err(message) => {
                let _ = tx.send(NetEvent::Error { message });
            }
        }
    }

    async fn get_json<T>(path: &str, token: &str) -> Result<T, String>
    where
        T: serde::de::DeserializeOwned,
    {
        let response = Request::get(path)
            .header("Authorization", &format!("Bearer {token}"))
            .send()
            .await
            .map_err(|error| error.to_string())?;
        parse_response(response).await
    }

    async fn post_empty<T>(path: &str, token: &str) -> Result<T, String>
    where
        T: serde::de::DeserializeOwned,
    {
        let response = Request::post(path)
            .header("Authorization", &format!("Bearer {token}"))
            .send()
            .await
            .map_err(|error| error.to_string())?;
        parse_response(response).await
    }

    async fn post_json<B, T>(path: &str, token: Option<&str>, body: &B) -> Result<T, String>
    where
        B: serde::Serialize,
        T: serde::de::DeserializeOwned,
    {
        let mut request = Request::post(path);
        if let Some(token) = token {
            request = request.header("Authorization", &format!("Bearer {token}"));
        }
        let response = request
            .json(body)
            .map_err(|error| error.to_string())?
            .send()
            .await
            .map_err(|error| error.to_string())?;
        parse_response(response).await
    }

    async fn parse_response<T>(response: gloo_net::http::Response) -> Result<T, String>
    where
        T: serde::de::DeserializeOwned,
    {
        let status = response.status();
        let text = response.text().await.map_err(|error| error.to_string())?;
        if !(200..=299).contains(&status) {
            return Err(extract_error(status, &text));
        }
        serde_json::from_str::<T>(&text).map_err(|error| format!("JSON: {error}"))
    }

    fn extract_error(status: u16, text: &str) -> String {
        serde_json::from_str::<Value>(text)
            .ok()
            .and_then(|value| {
                value
                    .get("error")
                    .and_then(|error| error.as_str())
                    .map(str::to_owned)
            })
            .unwrap_or_else(|| {
                if text.trim().is_empty() {
                    format!("HTTP {status}")
                } else {
                    format!("HTTP {status}: {text}")
                }
            })
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use super::*;

    pub fn stored_token() -> Option<String> {
        None
    }

    pub fn clear_stored_token() {}

    pub fn spawn_health(tx: Sender<NetEvent>) {
        let _ = tx.send(NetEvent::Health {
            state: HealthState::Offline,
            message: "сетевой клиент доступен только в WASM".into(),
        });
    }

    pub fn spawn_validate_token(_token: String, tx: Sender<NetEvent>) {
        let _ = tx.send(NetEvent::Error {
            message: "WASM-only client".into(),
        });
    }

    pub fn spawn_auth(_mode: AuthMode, _username: String, _password: String, tx: Sender<NetEvent>) {
        let _ = tx.send(NetEvent::Error {
            message: "WASM-only client".into(),
        });
    }

    pub fn spawn_create_room(_token: String, tx: Sender<NetEvent>) {
        let _ = tx.send(NetEvent::Error {
            message: "WASM-only client".into(),
        });
    }

    pub fn spawn_join_room(_code: String, _token: String, tx: Sender<NetEvent>) {
        let _ = tx.send(NetEvent::Error {
            message: "WASM-only client".into(),
        });
    }

    pub fn spawn_ready(_code: String, _ready: bool, _token: String, tx: Sender<NetEvent>) {
        let _ = tx.send(NetEvent::Error {
            message: "WASM-only client".into(),
        });
    }

    pub fn spawn_start(_code: String, _token: String, tx: Sender<NetEvent>) {
        let _ = tx.send(NetEvent::Error {
            message: "WASM-only client".into(),
        });
    }

    pub fn spawn_refresh_lobby(_code: String, _token: String, tx: Sender<NetEvent>) {
        let _ = tx.send(NetEvent::Error {
            message: "WASM-only client".into(),
        });
    }

    pub fn spawn_ws(_code: String, _token: String, _generation: u64, _tx: Sender<NetEvent>) {}
}

pub use imp::*;
