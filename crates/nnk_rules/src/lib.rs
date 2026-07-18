//! Pure rules engine adapted from verifying-turn-based-games skill.
//! Illegal actions return `Err` without mutating state (documented policy).

mod actions;
mod engine;
mod rng;

pub use actions::*;
pub use engine::*;
pub use rng::*;
