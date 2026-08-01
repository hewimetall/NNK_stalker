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
    #[error("username taken")]
    UsernameTaken,
    #[error("internal auth error")]
    Internal,
}

/// Username: 3..=32 chars, letters (incl. Cyrillic), digits, `_` `-`.
pub fn validate_username(username: &str) -> Result<(), AuthError> {
    let u = username.trim();
    let len = u.chars().count();
    if !(3..=32).contains(&len) {
        return Err(AuthError::InvalidUsername);
    }
    if !u.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(AuthError::InvalidUsername);
    }
    // Reject whitespace-only / control
    if u.chars().any(|c| c.is_control()) {
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
        assert!(validate_username("Сталкер01").is_ok());
        assert!(validate_username("Илья").is_ok());
    }

    #[test]
    fn password_rules() {
        assert!(validate_password("short").is_err());
        assert!(validate_password("longenough").is_ok());
    }
}
