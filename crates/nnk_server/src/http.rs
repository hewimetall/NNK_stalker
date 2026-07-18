use crate::state::{AppState, LiveRoom};
use axum::extract::{FromRequestParts, Path, State};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::Json;
use nnk_app::AppError;
use nnk_auth::AuthError;
use nnk_domain::UserId;
use nnk_ports::PortError;
use nnk_protocol::AuthRequest;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct AuthUser {
    pub user_id: UserId,
    pub username: String,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "missing Authorization"))?;
        let token = auth
            .strip_prefix("Bearer ")
            .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "expected Bearer token"))?;
        let (user_id, username) = state
            .auth
            .verify_token(token)
            .map_err(|_| err(StatusCode::UNAUTHORIZED, "invalid token"))?;
        Ok(AuthUser { user_id, username })
    }
}

fn err(status: StatusCode, message: &str) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(json!({ "error": message })))
}

fn map_app(e: AppError) -> (StatusCode, Json<serde_json::Value>) {
    match e {
        AppError::Auth(AuthError::InvalidCredentials) => {
            err(StatusCode::UNAUTHORIZED, "invalid username or password")
        }
        AppError::Auth(AuthError::WeakPassword) => {
            err(StatusCode::BAD_REQUEST, "password must be at least 8 chars")
        }
        AppError::Auth(AuthError::InvalidUsername) => {
            err(StatusCode::BAD_REQUEST, "invalid or taken username")
        }
        AppError::Auth(AuthError::InvalidToken) => err(StatusCode::UNAUTHORIZED, "invalid token"),
        AppError::Port(PortError::NotFound) | AppError::Message("room not found") => {
            err(StatusCode::NOT_FOUND, "not found")
        }
        AppError::Message(m) => err(StatusCode::BAD_REQUEST, m),
        other => {
            tracing::error!(error = %other, "app error");
            err(StatusCode::INTERNAL_SERVER_ERROR, "internal error")
        }
    }
}

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<AuthRequest>,
) -> Result<Json<nnk_protocol::AuthResponse>, (StatusCode, Json<serde_json::Value>)> {
    state.auth.register(body).await.map(Json).map_err(map_app)
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<AuthRequest>,
) -> Result<Json<nnk_protocol::AuthResponse>, (StatusCode, Json<serde_json::Value>)> {
    state.auth.login(body).await.map(Json).map_err(map_app)
}

pub async fn me(user: AuthUser) -> Json<serde_json::Value> {
    Json(json!({
        "user_id": user.user_id.0,
        "username": user.username,
    }))
}

pub async fn create_room(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<nnk_protocol::CreateRoomResponse>, (StatusCode, Json<serde_json::Value>)> {
    let (resp, room_state) = state
        .rooms
        .create(user.user_id, &user.username)
        .await
        .map_err(map_app)?;
    let (tx, _) = broadcast::channel(64);
    state.live.insert(
        resp.code.clone(),
        Arc::new(LiveRoom {
            state: tokio::sync::RwLock::new(room_state),
            tx,
        }),
    );
    Ok(Json(resp))
}

pub async fn join_room(
    State(state): State<AppState>,
    user: AuthUser,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let code = code.to_uppercase();
    let live = state.live.get(&code).map(|r| r.clone());
    let current = match &live {
        Some(l) => Some(l.state.read().await.clone()),
        None => None,
    };
    let new_state = state
        .rooms
        .join(&code, user.user_id, &user.username, current)
        .await
        .map_err(map_app)?;
    if let Some(live) = live {
        *live.state.write().await = new_state.clone();
        let _ = live
            .tx
            .send(nnk_protocol::ServerMsg::RoomState(new_state));
    }
    Ok(Json(json!({ "ok": true, "code": code })))
}
