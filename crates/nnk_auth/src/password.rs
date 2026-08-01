use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use rand::rngs::OsRng;

use crate::AuthError;

pub trait PasswordService: Send + Sync {
    fn hash(&self, password: &str) -> Result<String, AuthError>;
    fn verify(&self, password: &str, hash: &str) -> Result<(), AuthError>;
}

#[derive(Debug, Default)]
pub struct Argon2PasswordService;

impl PasswordService for Argon2PasswordService {
    fn hash(&self, password: &str) -> Result<String, AuthError> {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|_| AuthError::Internal)
    }

    fn verify(&self, password: &str, hash: &str) -> Result<(), AuthError> {
        let parsed = PasswordHash::new(hash).map_err(|_| AuthError::Internal)?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .map_err(|_| AuthError::InvalidCredentials)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_and_verify() {
        let svc = Argon2PasswordService;
        let hash = svc.hash("zonezone1").unwrap();
        assert!(svc.verify("zonezone1", &hash).is_ok());
        assert_eq!(
            svc.verify("wrongpass", &hash).unwrap_err(),
            AuthError::InvalidCredentials
        );
    }
}
