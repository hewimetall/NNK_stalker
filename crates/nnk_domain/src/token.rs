use serde::{Deserialize, Serialize};

use crate::{HexCoord, LocationId, UserId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TurnStage {
    NeedLocation,
    NeedSector,
    NeedHex,
    NeedD6,
    NeedMove,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerToken {
    pub user_id: UserId,
    pub display_name: String,
    pub ready: bool,
    pub location: Option<LocationId>,
    pub sector: Option<u8>,
    #[serde(default)]
    pub ccc_tens: Option<u8>,
    #[serde(default)]
    pub ccc_units: Option<u8>,
    pub hex: HexCoord,
    #[serde(default)]
    pub target_hex: Option<HexCoord>,
    pub move_points: u8,
    #[serde(default)]
    pub travel_stage: TurnStage,
}

impl PlayerToken {
    pub fn new(user_id: UserId, display_name: impl Into<String>) -> Self {
        Self {
            user_id,
            display_name: display_name.into(),
            ready: false,
            location: None,
            sector: None,
            ccc_tens: None,
            ccc_units: None,
            hex: HexCoord::ZERO,
            target_hex: None,
            move_points: 0,
            travel_stage: TurnStage::NeedLocation,
        }
    }
}

impl Default for TurnStage {
    fn default() -> Self {
        Self::NeedLocation
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
        assert!(t.target_hex.is_none());
        assert_eq!(t.move_points, 0);
        assert_eq!(t.travel_stage, TurnStage::NeedLocation);
        assert!(t.location.is_none());
        assert!(t.ccc_tens.is_none());
        assert!(t.ccc_units.is_none());
    }
}
