use bevy::prelude::Resource;
use nnk_protocol::LobbyView;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppState {
    #[default]
    Auth,
    Lobby,
    Playing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusField {
    #[default]
    Username,
    Password,
    RoomCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum HealthState {
    Checking,
    Online,
    Offline,
}

impl Default for HealthState {
    fn default() -> Self {
        Self::Checking
    }
}

#[derive(Debug, Clone, Resource)]
pub struct ClientSession {
    pub app_state: AppState,
    pub token: Option<String>,
    pub user_id: Option<String>,
    pub display_name: Option<String>,
    pub username_input: String,
    pub password_input: String,
    pub room_code_input: String,
    pub focused: FocusField,
    pub health: HealthState,
    pub error: Option<String>,
    pub status: String,
    pub lobby: Option<LobbyView>,
    pub inflight: usize,
    pub ws_generation: u64,
    pub ui_version: u64,
}

impl Default for ClientSession {
    fn default() -> Self {
        Self {
            app_state: AppState::Auth,
            token: crate::net::stored_token(),
            user_id: None,
            display_name: None,
            username_input: String::new(),
            password_input: String::new(),
            room_code_input: String::new(),
            focused: FocusField::Username,
            health: HealthState::Checking,
            error: None,
            status: "проверка сервера...".into(),
            lobby: None,
            inflight: 0,
            ws_generation: 0,
            ui_version: 0,
        }
    }
}

impl ClientSession {
    pub fn mark_dirty(&mut self) {
        self.ui_version = self.ui_version.wrapping_add(1);
    }

    pub fn set_error(&mut self, message: impl Into<String>) {
        self.error = Some(message.into());
        self.inflight = self.inflight.saturating_sub(1);
        self.mark_dirty();
    }

    pub fn begin_request(&mut self, status: impl Into<String>) {
        self.inflight += 1;
        self.status = status.into();
        self.error = None;
        self.mark_dirty();
    }

    pub fn finish_request(&mut self, status: impl Into<String>) {
        self.inflight = self.inflight.saturating_sub(1);
        self.status = status.into();
        self.mark_dirty();
    }

    pub fn set_lobby(&mut self, lobby: LobbyView) {
        self.room_code_input = lobby.code.clone();
        self.app_state = if lobby.phase == "playing" {
            AppState::Playing
        } else {
            AppState::Lobby
        };
        self.lobby = Some(lobby);
        self.error = None;
        self.mark_dirty();
    }

    pub fn logout(&mut self) {
        crate::net::clear_stored_token();
        self.app_state = AppState::Auth;
        self.token = None;
        self.user_id = None;
        self.display_name = None;
        self.password_input.clear();
        self.lobby = None;
        self.ws_generation = self.ws_generation.wrapping_add(1);
        self.status = "сессия завершена".into();
        self.mark_dirty();
    }

    pub fn is_busy(&self) -> bool {
        self.inflight > 0
    }

    pub fn me_is_gm(&self) -> bool {
        let Some(lobby) = &self.lobby else {
            return false;
        };
        let Some(user_id) = &self.user_id else {
            return false;
        };
        lobby
            .players
            .iter()
            .any(|player| player.is_gm && same_user_id(&player.user_id.to_string(), user_id))
    }

    pub fn me_ready(&self) -> bool {
        let Some(lobby) = &self.lobby else {
            return false;
        };
        let Some(user_id) = &self.user_id else {
            return false;
        };
        lobby
            .players
            .iter()
            .find(|player| same_user_id(&player.user_id.to_string(), user_id))
            .map(|player| player.ready)
            .unwrap_or(false)
    }
}

pub fn same_user_id(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}
