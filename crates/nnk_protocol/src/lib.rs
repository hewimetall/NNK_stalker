use nnk_domain::{
    CccDraw, DiceKind, HexCoord, LocationId, MemberRole, RoomState, UserId,
};
use nnk_rules::Action;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthResponse {
    pub user_id: Uuid,
    pub username: String,
    pub access_token: String,
    pub token_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateRoomResponse {
    pub room_id: Uuid,
    pub code: String,
    pub min_players: u8,
    pub max_players: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LobbyView {
    pub code: String,
    pub phase: String,
    pub min_players: u8,
    pub max_players: u8,
    pub player_count: u8,
    pub can_start: bool,
    pub players: Vec<LobbyPlayerView>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LobbyPlayerView {
    pub user_id: Uuid,
    pub display_name: String,
    pub ready: bool,
    pub is_gm: bool,
}

impl LobbyView {
    pub fn from_state(state: &RoomState) -> Self {
        Self {
            code: state.code.clone(),
            phase: match state.phase {
                nnk_domain::RoomPhase::Lobby => "lobby".into(),
                nnk_domain::RoomPhase::Playing => "playing".into(),
            },
            min_players: state.min_players,
            max_players: state.max_players,
            player_count: state.player_count(),
            can_start: state.can_start(),
            players: state
                .tokens
                .iter()
                .map(|t| LobbyPlayerView {
                    user_id: t.user_id.0,
                    display_name: t.display_name.clone(),
                    ready: t.ready,
                    is_gm: t.user_id == state.gm_user_id,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMsg {
    JoinRoom { code: String },
    SetReady { ready: bool },
    StartGame,
    StartMission { mission_id: u8 },
    RollD20Location,
    DrawCcc,
    RollD20Hex,
    RollD6Move,
    MoveToken { to: HexCoord },
    Chat { text: String },
}

impl ClientMsg {
    pub fn into_action(self) -> Option<Action> {
        Some(match self {
            ClientMsg::JoinRoom { .. } => return None,
            ClientMsg::SetReady { ready } => Action::SetReady { ready },
            ClientMsg::StartGame => Action::StartGame,
            ClientMsg::StartMission { mission_id } => Action::StartMission { mission_id },
            ClientMsg::RollD20Location => Action::RollD20Location,
            ClientMsg::DrawCcc => Action::DrawCcc,
            ClientMsg::RollD20Hex => Action::RollD20Hex,
            ClientMsg::RollD6Move => Action::RollD6Move,
            ClientMsg::MoveToken { to } => Action::MoveToken { to },
            ClientMsg::Chat { text } => Action::Chat { text },
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMsg {
    Welcome {
        user_id: UserId,
        role: MemberRole,
    },
    RoomState(RoomState),
    Lobby(LobbyView),
    DiceResult {
        kind: DiceKind,
        value: u8,
        by: UserId,
    },
    LocationRolled {
        location: LocationId,
        by: UserId,
    },
    CccResult {
        draw: CccDraw,
        by: UserId,
    },
    TokenMoved {
        user_id: UserId,
        from: HexCoord,
        to: HexCoord,
    },
    Error {
        code: String,
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use nnk_domain::{PlayerToken, RoomId, RoomState};

    #[test]
    fn client_msg_roundtrip() {
        let msg = ClientMsg::SetReady { ready: true };
        let json = serde_json::to_string(&msg).unwrap();
        let back: ClientMsg = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, back);
    }

    #[test]
    fn into_action_maps() {
        assert!(matches!(
            ClientMsg::StartGame.into_action(),
            Some(Action::StartGame)
        ));
        assert!(ClientMsg::JoinRoom { code: "X".into() }
            .into_action()
            .is_none());
    }

    #[test]
    fn lobby_view_from_state() {
        let gm = UserId::new();
        let state = RoomState::new(RoomId::new(), "ABC".into(), PlayerToken::new(gm, "gm"), 1);
        let view = LobbyView::from_state(&state);
        assert_eq!(view.phase, "lobby");
        assert_eq!(view.max_players, 5);
        assert_eq!(view.players.len(), 1);
        assert!(view.players[0].is_gm);
    }

    #[test]
    fn auth_shapes() {
        let a = AuthRequest {
            username: "a".into(),
            password: "b".into(),
        };
        let json = serde_json::to_string(&a).unwrap();
        assert!(json.contains("username"));
    }

    #[test]
    fn server_lobby_tagged() {
        let gm = UserId::new();
        let state = RoomState::new(RoomId::new(), "ABC".into(), PlayerToken::new(gm, "gm"), 1);
        let msg = ServerMsg::Lobby(LobbyView::from_state(&state));
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"lobby\""));
        let back: ServerMsg = serde_json::from_str(&json).unwrap();
        assert!(matches!(back, ServerMsg::Lobby(_)));
    }

    #[test]
    fn create_room_response_capacity() {
        let r = CreateRoomResponse {
            room_id: Uuid::nil(),
            code: "X".into(),
            min_players: 4,
            max_players: 5,
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("max_players"));
    }
}
