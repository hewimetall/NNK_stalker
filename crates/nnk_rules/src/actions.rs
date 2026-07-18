use nnk_domain::HexCoord;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    /// Toggle or set ready flag while in lobby.
    SetReady { ready: bool },
    /// GM starts the match when lobby has 4–5 ready players.
    StartGame,
    StartMission { mission_id: u8 },
    RollD20Location,
    DrawCcc,
    RollD20Hex,
    RollD6Move,
    MoveToken { to: HexCoord },
    Chat { text: String },
}
