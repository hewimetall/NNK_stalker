//! Auth primitives — ISP/DIP: small traits, swappable impls.

mod jwt;
mod password;

pub use jwt::*;
pub use password::*;

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AuthError {
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("invalid token")]
    InvalidToken,
    #[error("weak password")]
    WeakPassword,
    #[error("invalid username")]
    InvalidUsername,
    #[error("internal auth error")]
    Internal,
}

pub fn validate_username(username: &str) -> Result<(), AuthError> {
    let u = username.trim();
    if u.len() < 3 || u.len() > 32 {
        return Err(AuthError::InvalidUsername);
    }
    if !u
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(AuthError::InvalidUsername);
    }
    Ok(())
}

pub fn validate_password(password: &str) -> Result<(), AuthError> {
    if password.len() < 8 {
        Err(AuthError::WeakPassword)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn username_rules() {
        assert!(validate_username("ab").is_err());
        assert!(validate_username("stalker_1").is_ok());
        assert!(validate_username("bad name").is_err());
    }

    #[test]
    fn password_rules() {
        assert!(validate_password("short").is_err());
        assert!(validate_password("longenough").is_ok());
    }
}
