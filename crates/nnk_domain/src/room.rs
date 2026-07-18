use serde::{Deserialize, Serialize};

use crate::{PlayerToken, RoomId, UserId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoomState {
    pub room_id: RoomId,
    pub code: String,
    pub gm_user_id: UserId,
    pub mission_id: u8,
    pub tokens: Vec<PlayerToken>,
    pub active_user_id: Option<UserId>,
    pub history: Vec<String>,
    /// Deterministic RNG seed for replay (turn-based engine contract).
    pub rng_seed: u64,
}

impl RoomState {
    pub fn new(room_id: RoomId, code: String, gm: PlayerToken, rng_seed: u64) -> Self {
        let gm_user_id = gm.user_id;
        Self {
            room_id,
            code,
            gm_user_id,
            mission_id: 1,
            tokens: vec![gm],
            active_user_id: Some(gm_user_id),
            history: Vec::new(),
            rng_seed,
        }
    }

    pub fn find_token(&self, user_id: UserId) -> Option<&PlayerToken> {
        self.tokens.iter().find(|t| t.user_id == user_id)
    }

    pub fn find_token_mut(&mut self, user_id: UserId) -> Option<&mut PlayerToken> {
        self.tokens.iter_mut().find(|t| t.user_id == user_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PlayerToken;

    #[test]
    fn room_holds_gm() {
        let gm_id = UserId::new();
        let room = RoomState::new(
            RoomId::new(),
            "ABCD12".into(),
            PlayerToken::new(gm_id, "GM"),
            7,
        );
        assert_eq!(room.gm_user_id, gm_id);
        assert_eq!(room.find_token(gm_id).unwrap().display_name, "GM");
        assert_eq!(room.rng_seed, 7);
    }
}
