use serde::{Deserialize, Serialize};

use crate::{HexCoord, LocationId, MissionDef, PlayerToken, TurnStage};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerTablet {
    pub callsign: String,
    pub display_name: String,
    pub mission_id: u8,
    pub mission_title: String,
    pub mission_objective: String,
    pub location: Option<LocationId>,
    pub location_name: Option<String>,
    pub sector: Option<u8>,
    pub ccc_tens: Option<u8>,
    pub ccc_units: Option<u8>,
    pub current_hex: HexCoord,
    pub target_hex: Option<HexCoord>,
    pub move_points: u8,
    pub travel_stage: TurnStage,
    pub notes: String,
}

impl PlayerTablet {
    pub fn from_token(token: &PlayerToken, mission: &MissionDef, notes: impl Into<String>) -> Self {
        let display_name = token.display_name.clone();
        Self {
            callsign: display_name.clone(),
            display_name,
            mission_id: mission.id,
            mission_title: mission.title.to_string(),
            mission_objective: mission.objective.to_string(),
            location: token.location,
            location_name: token
                .location
                .map(|location| location.name_ru().to_string()),
            sector: token.sector,
            ccc_tens: token.ccc_tens,
            ccc_units: token.ccc_units,
            current_hex: token.hex,
            target_hex: token.target_hex,
            move_points: token.move_points,
            travel_stage: token.travel_stage,
            notes: notes.into().chars().take(240).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LocationId, MissionDef, UserId};

    #[test]
    fn tablet_aggregates_token_and_mission_state() {
        let mut token = PlayerToken::new(UserId::new(), "ГГ");
        token.location = Some(LocationId::Cordon);
        token.sector = Some(36);
        token.ccc_tens = Some(30);
        token.ccc_units = Some(6);
        token.hex = HexCoord::new(0, -2);
        token.target_hex = Some(HexCoord::ZERO);
        token.move_points = 3;

        let tablet = PlayerTablet::from_token(&token, MissionDef::get(1).unwrap(), "short note");

        assert_eq!(tablet.callsign, "ГГ");
        assert_eq!(tablet.display_name, "ГГ");
        assert_eq!(tablet.mission_id, 1);
        assert_eq!(tablet.location_name.as_deref(), Some("Кордон"));
        assert_eq!(tablet.current_hex, HexCoord::new(0, -2));
        assert_eq!(tablet.target_hex, Some(HexCoord::ZERO));
        assert_eq!(tablet.notes, "short note");
    }
}
