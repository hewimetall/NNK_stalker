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
}
