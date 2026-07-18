use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DomainError {
    #[error("illegal move: distance {distance} exceeds move points {points}")]
    IllegalMove { distance: u32, points: u8 },
    #[error("unknown player")]
    UnknownPlayer,
    #[error("forbidden: {0}")]
    Forbidden(&'static str),
    #[error("invalid dice value: {0}")]
    InvalidDice(u8),
    #[error("invalid CCC combination")]
    InvalidCcc,
    #[error("lobby full (max {max} players)")]
    LobbyFull { max: u8 },
    #[error("need {min}..={max} players to start, have {have}")]
    LobbySize { min: u8, max: u8, have: u8 },
    #[error("not all players are ready")]
    LobbyNotReady,
    #[error("game already started")]
    AlreadyPlaying,
    #[error("still in lobby")]
    StillInLobby,
}
