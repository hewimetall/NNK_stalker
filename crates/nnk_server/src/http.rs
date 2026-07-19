use crate::state::{AppState, LiveRoom};
use axum::extract::{FromRequestParts, Path, State};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::Json;
use nnk_app::AppError;
use nnk_auth::AuthError;
use nnk_domain::{MemberRole, UserId};
use nnk_ports::PortError;
use nnk_protocol::{AuthRequest, ClientMsg, LobbyView, ServerMsg};
use nnk_rules::Action;
use serde::Deserialize;
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
            err(StatusCode::UNAUTHORIZED, "неверный логин или пароль")
        }
        AppError::Auth(AuthError::WeakPassword) => {
            err(
                StatusCode::BAD_REQUEST,
                "пароль слишком короткий (минимум 8 символов)",
            )
        }
        AppError::Auth(AuthError::InvalidUsername) => err(
            StatusCode::BAD_REQUEST,
            "логин: 3–32 символа, буквы/цифры/_/- (можно кириллицу)",
        ),
        AppError::Auth(AuthError::UsernameTaken) => {
            err(StatusCode::CONFLICT, "такой логин уже занят — войдите или выберите другой")
        }
        AppError::Auth(AuthError::InvalidToken) => {
            err(StatusCode::UNAUTHORIZED, "сессия истекла — войдите снова")
        }
        AppError::Auth(AuthError::Internal) => {
            err(StatusCode::INTERNAL_SERVER_ERROR, "ошибка авторизации")
        }
        AppError::Port(PortError::NotFound) | AppError::Message("room not found") => {
            err(StatusCode::NOT_FOUND, "not found")
        }
        AppError::Domain(nnk_domain::DomainError::LobbyFull { max }) => err(
            StatusCode::CONFLICT,
            &format!("lobby full (max {max} players)"),
        ),
        AppError::Domain(nnk_domain::DomainError::LobbyNotReady) => {
            err(StatusCode::CONFLICT, "not all players are ready")
        }
        AppError::Domain(nnk_domain::DomainError::LobbySize { min, max, have }) => err(
            StatusCode::CONFLICT,
            &format!("need {min}..={max} players, have {have}"),
        ),
        AppError::Domain(nnk_domain::DomainError::AlreadyPlaying) => {
            err(StatusCode::CONFLICT, "game already started")
        }
        AppError::Domain(nnk_domain::DomainError::StillInLobby) => {
            err(StatusCode::CONFLICT, "still in lobby")
        }
        AppError::Domain(other) => err(StatusCode::BAD_REQUEST, &other.to_string()),
        AppError::Message(m) => err(StatusCode::BAD_REQUEST, m),
        other => {
            tracing::error!(error = %other, "app error");
            err(StatusCode::INTERNAL_SERVER_ERROR, "internal error")
        }
    }
}

async fn broadcast_lobby(live: &LiveRoom, state: &nnk_domain::RoomState) {
    // RoomState only — each WS subscriber rebuilds Lobby with its legal_actions.
    let _ = live.tx.send(ServerMsg::RoomState(state.clone()));
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
) -> Result<Json<LobbyView>, (StatusCode, Json<serde_json::Value>)> {
    let code = code.to_uppercase();
    let live = state.live.get(&code).map(|r| r.clone()).ok_or_else(|| {
        err(
            StatusCode::NOT_FOUND,
            "room not in memory — create a new lobby",
        )
    })?;
    let current = live.state.read().await.clone();
    let new_state = state
        .rooms
        .join(&code, user.user_id, &user.username, Some(current))
        .await
        .map_err(map_app)?;
    *live.state.write().await = new_state.clone();
    broadcast_lobby(&live, &new_state).await;
    let role = state
        .rooms
        .member_role(&code, user.user_id)
        .await
        .unwrap_or(MemberRole::Player);
    Ok(Json(LobbyView::from_state_for(
        &new_state,
        Some(user.user_id),
        role,
    )))
}

pub async fn get_lobby(
    State(state): State<AppState>,
    user: AuthUser,
    Path(code): Path<String>,
) -> Result<Json<LobbyView>, (StatusCode, Json<serde_json::Value>)> {
    let code = code.to_uppercase();
    let role = state
        .rooms
        .member_role(&code, user.user_id)
        .await
        .map_err(map_app)?;
    let live = state
        .live
        .get(&code)
        .ok_or_else(|| err(StatusCode::NOT_FOUND, "room not in memory"))?;
    let rs = live.state.read().await.clone();
    Ok(Json(LobbyView::from_state_for(
        &rs,
        Some(user.user_id),
        role,
    )))
}

#[derive(Deserialize)]
pub struct ReadyBody {
    pub ready: bool,
}

pub async fn set_ready(
    State(state): State<AppState>,
    user: AuthUser,
    Path(code): Path<String>,
    Json(body): Json<ReadyBody>,
) -> Result<Json<LobbyView>, (StatusCode, Json<serde_json::Value>)> {
    apply_lobby_action(
        &state,
        user,
        &code,
        Action::SetReady { ready: body.ready },
    )
    .await
}

pub async fn start_game(
    State(state): State<AppState>,
    user: AuthUser,
    Path(code): Path<String>,
) -> Result<Json<LobbyView>, (StatusCode, Json<serde_json::Value>)> {
    apply_lobby_action(&state, user, &code, Action::StartGame).await
}

async fn apply_lobby_action(
    state: &AppState,
    user: AuthUser,
    code: &str,
    action: Action,
) -> Result<Json<LobbyView>, (StatusCode, Json<serde_json::Value>)> {
    let code = code.to_uppercase();
    let role = state
        .rooms
        .member_role(&code, user.user_id)
        .await
        .map_err(map_app)?;
    // StartGame must be GM — rules also enforce; REST doubles check for clearer errors.
    if matches!(action, Action::StartGame) && role != MemberRole::Gm {
        return Err(err(StatusCode::FORBIDDEN, "only GM can start the game"));
    }
    let live = state
        .live
        .get(&code)
        .ok_or_else(|| err(StatusCode::NOT_FOUND, "room not in memory"))?
        .clone();
    let current = live.state.read().await.clone();
    let out = state
        .rooms
        .apply(current, user.user_id, role, action)
        .map_err(map_app)?;
    *live.state.write().await = out.state.clone();
    let _ = state.rooms.persist_snapshot(&code, &out.state).await;
    broadcast_lobby(&live, &out.state).await;
    Ok(Json(LobbyView::from_state_for(
        &out.state,
        Some(user.user_id),
        role,
    )))
}

pub async fn post_action(
    State(state): State<AppState>,
    user: AuthUser,
    Path(code): Path<String>,
    Json(msg): Json<ClientMsg>,
) -> Result<Json<LobbyView>, (StatusCode, Json<serde_json::Value>)> {
    let action = msg
        .into_action()
        .ok_or_else(|| err(StatusCode::BAD_REQUEST, "join is not an action"))?;
    apply_lobby_action(&state, user, &code, action).await
}
