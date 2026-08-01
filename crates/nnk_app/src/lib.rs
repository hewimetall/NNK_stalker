//! Application services — OCP/DIP: depend on ports + auth traits.

mod auth_service;
mod room_service;

pub use auth_service::*;
pub use room_service::*;

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AppError {
    #[error(transparent)]
    Auth(#[from] nnk_auth::AuthError),
    #[error(transparent)]
    Port(#[from] nnk_ports::PortError),
    #[error(transparent)]
    Domain(#[from] nnk_domain::DomainError),
    #[error("{0}")]
    Message(&'static str),
}
