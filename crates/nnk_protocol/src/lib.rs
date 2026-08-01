use nnk_domain::{
    CccDraw, DiceKind, HexCoord, LocationId, MemberRole, MissionDef, NpcKind, RoomState, TurnStage,
    UserId,
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
    pub round_phase: String,
    pub round_number: u32,
    pub min_players: u8,
    pub max_players: u8,
    pub player_count: u8,
    pub can_start: bool,
    pub mission_id: u8,
    pub mission_title: String,
    pub mission_objective: String,
    pub mission_briefing: String,
    pub game_day: u32,
    pub game_hour: u8,
    pub active_user_id: Option<Uuid>,
    pub active_display_name: Option<String>,
    pub players: Vec<LobbyPlayerView>,
    pub npcs: Vec<LobbyNpcView>,
    pub event_tokens: Vec<EventTokenView>,
    pub last_event: Option<String>,
    pub history: Vec<String>,
    /// Snake_case action ids for the requesting user (empty if unknown).
    pub legal_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventTokenView {
    pub id: u32,
    pub hex: HexCoord,
    pub resolved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LobbyPlayerView {
    pub user_id: Uuid,
    pub display_name: String,
    pub ready: bool,
    pub is_gm: bool,
    pub location: Option<String>,
    pub sector: Option<u8>,
    pub ccc_tens: Option<u8>,
    pub ccc_units: Option<u8>,
    pub hex: HexCoord,
    pub target_hex: Option<HexCoord>,
    pub move_points: u8,
    pub travel_stage: String,
    pub hp: u16,
    pub rubles: u32,
    pub artifacts: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LobbyNpcView {
    pub name: String,
    pub kind: String,
    pub hex: HexCoord,
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
        let mission = MissionDef::get(state.mission_id)
            .unwrap_or_else(|| MissionDef::get(1).expect("mission catalog must include mission 1"));
        Self {
            code: state.code.clone(),
            phase: match state.phase {
                nnk_domain::RoomPhase::Lobby => "lobby".into(),
                nnk_domain::RoomPhase::Playing => "playing".into(),
            },
            round_phase: if state.phase == nnk_domain::RoomPhase::Lobby {
                String::new()
            } else {
                state.round_phase.wire_id().into()
            },
            round_number: state.round_number,
            min_players: state.min_players,
            max_players: state.max_players,
            player_count: state.player_count(),
            can_start: state.can_start(),
            mission_id: state.mission_id,
            mission_title: mission.title.to_string(),
            mission_objective: mission.objective.to_string(),
            mission_briefing: mission.briefing.to_string(),
            game_day: state.game_day,
            game_hour: state.game_hour,
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
                    ccc_tens: t.ccc_tens,
                    ccc_units: t.ccc_units,
                    hex: t.hex,
                    target_hex: t.target_hex,
                    move_points: t.move_points,
                    travel_stage: travel_stage_wire_id(t.travel_stage).into(),
                    hp: t.hp,
                    rubles: t.rubles,
                    artifacts: t.artifacts,
                })
                .collect(),
            npcs: state
                .npcs
                .iter()
                .map(|npc| LobbyNpcView {
                    name: npc.name.clone(),
                    kind: npc_kind_wire_id(npc.kind).into(),
                    hex: npc.hex,
                })
                .collect(),
            event_tokens: state
                .event_tokens
                .iter()
                .map(|t| EventTokenView {
                    id: t.id,
                    hex: t.hex,
                    resolved: t.resolved,
                })
                .collect(),
            last_event: state.last_event.clone(),
            history,
            legal_actions: legal,
        }
    }
}

fn travel_stage_wire_id(stage: TurnStage) -> &'static str {
    match stage {
        TurnStage::Idle => "idle",
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
        Action::RollExitZone => "roll_exit_zone".into(),
        Action::RollExploreD6 => "roll_explore_d6".into(),
        Action::DrawEvent => "draw_event".into(),
        Action::ReturnToBase => "return_to_base".into(),
        Action::FinishBase => "finish_base".into(),
        Action::RollD20Location => "roll_d20_location".into(),
        Action::DrawCcc => "draw_ccc".into(),
        Action::RollD20Hex => "roll_d20_hex".into(),
        Action::RollD6Move => "roll_d6_move".into(),
        Action::MoveToken { to } => format!("move_token:{},{}", to.q, to.r),
        Action::SpawnNpc { .. } => "spawn_npc".into(),
        Action::Chat { .. } => return None,
    })
}

fn npc_kind_wire_id(kind: NpcKind) -> &'static str {
    match kind {
        NpcKind::Stalker => "stalker",
        NpcKind::Mutant => "mutant",
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMsg {
    JoinRoom { code: String },
    SetReady { ready: bool },
    StartGame,
    StartMission { mission_id: u8 },
    RollExitZone,
    RollExploreD6,
    DrawEvent,
    ReturnToBase,
    FinishBase,
    RollD20Location,
    DrawCcc,
    RollD20Hex,
    RollD6Move,
    MoveToken { to: HexCoord },
    SpawnNpc { name: String, kind: NpcKind },
    Chat { text: String },
}

impl ClientMsg {
    pub fn into_action(self) -> Option<Action> {
        Some(match self {
            ClientMsg::JoinRoom { .. } => return None,
            ClientMsg::SetReady { ready } => Action::SetReady { ready },
            ClientMsg::StartGame => Action::StartGame,
            ClientMsg::StartMission { mission_id } => Action::StartMission { mission_id },
            ClientMsg::RollExitZone => Action::RollExitZone,
            ClientMsg::RollExploreD6 => Action::RollExploreD6,
            ClientMsg::DrawEvent => Action::DrawEvent,
            ClientMsg::ReturnToBase => Action::ReturnToBase,
            ClientMsg::FinishBase => Action::FinishBase,
            ClientMsg::RollD20Location => Action::RollD20Location,
            ClientMsg::DrawCcc => Action::DrawCcc,
            ClientMsg::RollD20Hex => Action::RollD20Hex,
            ClientMsg::RollD6Move => Action::RollD6Move,
            ClientMsg::MoveToken { to } => Action::MoveToken { to },
            ClientMsg::SpawnNpc { name, kind } => Action::SpawnNpc { name, kind },
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
        assert!(matches!(
            ClientMsg::SpawnNpc {
                name: "Guide".into(),
                kind: NpcKind::Stalker,
            }
            .into_action(),
            Some(Action::SpawnNpc { .. })
        ));
        assert!(ClientMsg::JoinRoom { code: "X".into() }
            .into_action()
            .is_none());
    }

    #[test]
    fn lobby_view_from_state() {
        let gm = UserId::new();
        let mut state = RoomState::new(RoomId::new(), "ABC".into(), PlayerToken::new(gm, "gm"), 1);
        state.mission_id = 3;
        state.game_day = 2;
        state.game_hour = 14;
        state.tokens[0].location = Some(LocationId::Cordon);
        state.tokens[0].sector = Some(36);
        state.tokens[0].ccc_tens = Some(30);
        state.tokens[0].ccc_units = Some(6);
        state.tokens[0].target_hex = Some(HexCoord::new(1, 0));
        state.npcs.push(nnk_domain::Npc::new(
            "Guide",
            NpcKind::Stalker,
            gm,
            HexCoord::ZERO,
        ));
        let view = LobbyView::from_state(&state);
        assert_eq!(view.phase, "lobby");
        assert_eq!(view.max_players, 5);
        assert_eq!(view.players.len(), 1);
        assert!(view.players[0].is_gm);
        assert_eq!(view.mission_id, 3);
        assert_eq!(view.mission_title, MissionDef::get(3).unwrap().title);
        assert_eq!(
            view.mission_objective,
            MissionDef::get(3).unwrap().objective
        );
        assert_eq!(view.mission_briefing, MissionDef::get(3).unwrap().briefing);
        assert_eq!(view.game_day, 2);
        assert_eq!(view.game_hour, 14);
        assert_eq!(view.players[0].location.as_deref(), Some("Кордон"));
        assert_eq!(view.players[0].sector, Some(36));
        assert_eq!(view.players[0].ccc_tens, Some(30));
        assert_eq!(view.players[0].ccc_units, Some(6));
        assert_eq!(view.players[0].target_hex, Some(HexCoord::new(1, 0)));
        assert_eq!(view.npcs.len(), 1);
        assert_eq!(view.npcs[0].name, "Guide");
        assert_eq!(view.npcs[0].kind, "stalker");
        assert_eq!(view.npcs[0].hex, HexCoord::ZERO);
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
