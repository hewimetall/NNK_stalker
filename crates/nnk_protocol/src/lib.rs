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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMsg {
    JoinRoom { code: String },
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

    #[test]
    fn client_msg_roundtrip() {
        let msg = ClientMsg::MoveToken {
            to: HexCoord::new(2, -1),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let back: ClientMsg = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, back);
    }

    #[test]
    fn into_action_maps() {
        assert!(matches!(
            ClientMsg::RollD6Move.into_action(),
            Some(Action::RollD6Move)
        ));
        assert!(ClientMsg::JoinRoom {
            code: "X".into()
        }
        .into_action()
        .is_none());
    }

    #[test]
    fn server_msg_tagged() {
        let msg = ServerMsg::Error {
            code: "x".into(),
            message: "y".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"error\""));
    }

    #[test]
    fn auth_shapes() {
        let a = AuthRequest {
            username: "a".into(),
            password: "b".into(),
        };
        let _ = CreateRoomResponse {
            room_id: Uuid::nil(),
            code: "A".into(),
        };
        let json = serde_json::to_string(&a).unwrap();
        assert!(json.contains("username"));
    }
}
