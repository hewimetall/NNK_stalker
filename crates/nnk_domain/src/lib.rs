//! Domain layer — entities & value objects only (SOLID: SRP, no infra).

mod error;
mod hex;
mod ids;
mod location;
mod room;
mod token;

pub use error::*;
pub use hex::*;
pub use ids::*;
pub use location::*;
pub use room::*;
pub use token::*;
