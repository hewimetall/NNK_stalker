use serde::{Deserialize, Serialize};

use crate::{HexCoord, LocationId, UserId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerToken {
    pub user_id: UserId,
    pub display_name: String,
    pub ready: bool,
    pub location: Option<LocationId>,
    pub sector: Option<u8>,
    pub hex: HexCoord,
    pub move_points: u8,
}

impl PlayerToken {
    pub fn new(user_id: UserId, display_name: impl Into<String>) -> Self {
        Self {
            user_id,
            display_name: display_name.into(),
            ready: false,
            location: None,
            sector: None,
            hex: HexCoord::ZERO,
            move_points: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_token_defaults() {
        let id = UserId::new();
        let t = PlayerToken::new(id, "ГГ");
        assert_eq!(t.hex, HexCoord::ZERO);
        assert_eq!(t.move_points, 0);
        assert!(t.location.is_none());
    }
}
