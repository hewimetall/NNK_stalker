use nnk_domain::{
    CccDraw, DiceKind, HexCoord, LocationId, MemberRole, RoomState, TurnStage, UserId,
};
use nnk_rules::{legal_actions, Action};
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
    pub mission_id: u8,
    pub active_user_id: Option<Uuid>,
    pub active_display_name: Option<String>,
    pub players: Vec<LobbyPlayerView>,
    pub history: Vec<String>,
    /// Snake_case action ids for the requesting user (empty if unknown).
    pub legal_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LobbyPlayerView {
    pub user_id: Uuid,
    pub display_name: String,
    pub ready: bool,
    pub is_gm: bool,
    pub location: Option<String>,
    pub sector: Option<u8>,
    pub hex: HexCoord,
    pub move_points: u8,
    pub travel_stage: String,
}

impl LobbyView {
    pub fn from_state(state: &RoomState) -> Self {
        Self::from_state_for(state, None, MemberRole::Player)
    }

    pub fn from_state_for(state: &RoomState, viewer: Option<UserId>, role: MemberRole) -> Self {
        let legal = viewer
            .map(|uid| {
                legal_actions(state, uid, role)
                    .into_iter()
                    .filter_map(action_wire_id)
                    .collect()
            })
            .unwrap_or_default();
        let hist_len = state.history.len();
        let history = state
            .history
            .iter()
            .skip(hist_len.saturating_sub(16))
            .cloned()
            .collect();
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
            mission_id: state.mission_id,
            active_user_id: state.active_user_id.map(|id| id.0),
            active_display_name: state
                .active_user_id
                .and_then(|id| state.find_token(id))
                .map(|t| t.display_name.clone()),
            players: state
                .tokens
                .iter()
                .map(|t| LobbyPlayerView {
                    user_id: t.user_id.0,
                    display_name: t.display_name.clone(),
                    ready: t.ready,
                    is_gm: t.user_id == state.gm_user_id,
                    location: t.location.map(|l| l.name_ru().to_string()),
                    sector: t.sector,
                    hex: t.hex,
                    move_points: t.move_points,
                    travel_stage: travel_stage_wire_id(t.travel_stage).into(),
                })
                .collect(),
            history,
            legal_actions: legal,
        }
    }
}

fn travel_stage_wire_id(stage: TurnStage) -> &'static str {
    match stage {
        TurnStage::NeedLocation => "need_location",
        TurnStage::NeedSector => "need_sector",
        TurnStage::NeedHex => "need_hex",
        TurnStage::NeedD6 => "need_d6",
        TurnStage::NeedMove => "need_move",
    }
}

fn action_wire_id(action: Action) -> Option<String> {
    Some(match action {
        Action::SetReady { ready } => format!("set_ready:{ready}"),
        Action::StartGame => "start_game".into(),
        Action::StartMission { mission_id } => format!("start_mission:{mission_id}"),
        Action::RollD20Location => "roll_d20_location".into(),
        Action::DrawCcc => "draw_ccc".into(),
        Action::RollD20Hex => "roll_d20_hex".into(),
        Action::RollD6Move => "roll_d6_move".into(),
        Action::MoveToken { to } => format!("move_token:{},{}", to.q, to.r),
        Action::Chat { .. } => return None,
    })
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
        assert_eq!(view.mission_id, 1);
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
