use nnk_domain::HexCoord;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    StartMission { mission_id: u8 },
    RollD20Location,
    DrawCcc,
    RollD20Hex,
    RollD6Move,
    MoveToken { to: HexCoord },
    Chat { text: String },
}
