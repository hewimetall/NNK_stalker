use nnk_auth::{
    AuthError, PasswordService, TokenService, validate_password, validate_username,
};
use nnk_domain::UserId;
use nnk_ports::UserRepository;
use nnk_protocol::{AuthRequest, AuthResponse};
use std::sync::Arc;

use crate::AppError;

pub struct AuthService<U, P, T> {
    users: Arc<U>,
    passwords: Arc<P>,
    tokens: Arc<T>,
}

impl<U, P, T> AuthService<U, P, T>
where
    U: UserRepository,
    P: PasswordService,
    T: TokenService,
{
    pub fn new(users: Arc<U>, passwords: Arc<P>, tokens: Arc<T>) -> Self {
        Self {
            users,
            passwords,
            tokens,
        }
    }

    pub async fn register(&self, req: AuthRequest) -> Result<AuthResponse, AppError> {
        validate_username(&req.username)?;
        validate_password(&req.password)?;
        let username = req.username.trim().to_string();
        let id = UserId::new();
        let hash = self.passwords.hash(&req.password)?;
        self.users
            .create_user(id, &username, &hash)
            .await
            .map_err(|e| match e {
                nnk_ports::PortError::Conflict(_) => AppError::Auth(AuthError::InvalidUsername),
                other => AppError::Port(other),
            })?;
        let access_token = self.tokens.issue(id, &username)?;
        Ok(AuthResponse {
            user_id: id.0,
            username,
            access_token,
            token_type: "Bearer".into(),
        })
    }

    pub async fn login(&self, req: AuthRequest) -> Result<AuthResponse, AppError> {
        validate_username(&req.username)?;
        validate_password(&req.password)?;
        let user = self
            .users
            .find_by_username(req.username.trim())
            .await?
            .ok_or(AuthError::InvalidCredentials)?;
        self.passwords
            .verify(&req.password, &user.password_hash)?;
        let access_token = self.tokens.issue(user.id, &user.username)?;
        Ok(AuthResponse {
            user_id: user.id.0,
            username: user.username,
            access_token,
            token_type: "Bearer".into(),
        })
    }

    pub fn verify_token(&self, token: &str) -> Result<(UserId, String), AppError> {
        Ok(self.tokens.verify(token)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nnk_auth::{Argon2PasswordService, JwtTokenService};
    use nnk_persistence::SqliteStore;

    async fn svc() -> AuthService<SqliteStore, Argon2PasswordService, JwtTokenService> {
        let db = Arc::new(SqliteStore::connect_memory().await.unwrap());
        AuthService::new(
            db,
            Arc::new(Argon2PasswordService),
            Arc::new(JwtTokenService::new("test-secret", 1)),
        )
    }

    #[tokio::test]
    async fn register_login_me() {
        let s = svc().await;
        let reg = s
            .register(AuthRequest {
                username: "stalker1".into(),
                password: "zonezone1".into(),
            })
            .await
            .unwrap();
        let (uid, name) = s.verify_token(&reg.access_token).unwrap();
        assert_eq!(uid.0, reg.user_id);
        assert_eq!(name, "stalker1");

        let login = s
            .login(AuthRequest {
                username: "STALKER1".into(),
                password: "zonezone1".into(),
            })
            .await
            .unwrap();
        assert_eq!(login.user_id, reg.user_id);
    }

    #[tokio::test]
    async fn reject_duplicate() {
        let s = svc().await;
        let req = AuthRequest {
            username: "dup".into(),
            password: "zonezone1".into(),
        };
        s.register(req.clone()).await.unwrap();
        assert!(s.register(req).await.is_err());
    }

    #[tokio::test]
    async fn bad_login() {
        let s = svc().await;
        s.register(AuthRequest {
            username: "xuser".into(),
            password: "zonezone1".into(),
        })
        .await
        .unwrap();
        let err = s
            .login(AuthRequest {
                username: "xuser".into(),
                password: "wrongpassword".into(),
            })
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Auth(AuthError::InvalidCredentials)));
    }
}
