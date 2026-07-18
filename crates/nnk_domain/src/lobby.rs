//! Lobby constraints: 4–5 authenticated players (including GM).

/// Minimum seats to leave lobby and start the campaign.
pub const LOBBY_MIN_PLAYERS: u8 = 4;
/// Hard cap for a room (friends table).
pub const LOBBY_MAX_PLAYERS: u8 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RoomPhase {
    #[default]
    Lobby,
    Playing,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capacity_bounds() {
        assert_eq!(LOBBY_MIN_PLAYERS, 4);
        assert_eq!(LOBBY_MAX_PLAYERS, 5);
        assert!(LOBBY_MIN_PLAYERS <= LOBBY_MAX_PLAYERS);
    }
}
