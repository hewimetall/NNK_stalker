use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use nnk_domain::UserId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AuthError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub exp: i64,
    pub iat: i64,
}

pub trait TokenService: Send + Sync {
    fn issue(&self, user_id: UserId, username: &str) -> Result<String, AuthError>;
    fn verify(&self, token: &str) -> Result<(UserId, String), AuthError>;
}

#[derive(Debug, Clone)]
pub struct JwtTokenService {
    secret: String,
    ttl_hours: i64,
}

impl JwtTokenService {
    pub fn new(secret: impl Into<String>, ttl_hours: i64) -> Self {
        Self {
            secret: secret.into(),
            ttl_hours,
        }
    }
}

impl TokenService for JwtTokenService {
    fn issue(&self, user_id: UserId, username: &str) -> Result<String, AuthError> {
        let now = Utc::now();
        let claims = Claims {
            sub: user_id.0.to_string(),
            username: username.to_string(),
            iat: now.timestamp(),
            exp: (now + Duration::hours(self.ttl_hours)).timestamp(),
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|_| AuthError::Internal)
    }

    fn verify(&self, token: &str) -> Result<(UserId, String), AuthError> {
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AuthError::InvalidToken)?;
        let id = Uuid::parse_str(&data.claims.sub).map_err(|_| AuthError::InvalidToken)?;
        Ok((UserId(id), data.claims.username))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issue_verify_roundtrip() {
        let svc = JwtTokenService::new("secret", 1);
        let uid = UserId::new();
        let token = svc.issue(uid, "stalker").unwrap();
        let (back, name) = svc.verify(&token).unwrap();
        assert_eq!(back, uid);
        assert_eq!(name, "stalker");
    }

    #[test]
    fn bad_token() {
        let svc = JwtTokenService::new("secret", 1);
        assert_eq!(svc.verify("not.a.jwt").unwrap_err(), AuthError::InvalidToken);
    }

    #[test]
    fn wrong_secret() {
        let a = JwtTokenService::new("a", 1);
        let b = JwtTokenService::new("b", 1);
        let token = a.issue(UserId::new(), "x").unwrap();
        assert!(b.verify(&token).is_err());
    }
}
