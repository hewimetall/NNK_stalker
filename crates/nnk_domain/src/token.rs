use serde::{Deserialize, Serialize};

use crate::{HexCoord, LocationId, UserId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TurnStage {
    /// Exploring the zone / between mission-targeting steps (Пилигрим фаза II).
    Idle,
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
    /// Hit points (Пилигрим НР); default leather-jacket starter.
    #[serde(default = "default_hp")]
    pub hp: u16,
    #[serde(default)]
    pub rubles: u32,
    #[serde(default)]
    pub artifacts: u8,
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
            travel_stage: TurnStage::Idle,
            hp: default_hp(),
            rubles: 0,
            artifacts: 0,
        }
    }
}

impl Default for TurnStage {
    fn default() -> Self {
        Self::Idle
    }
}

fn default_hp() -> u16 {
    100
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
        assert_eq!(t.travel_stage, TurnStage::Idle);
        assert_eq!(t.hp, 100);
        assert!(t.location.is_none());
        assert!(t.ccc_tens.is_none());
        assert!(t.ccc_units.is_none());
    }
}
