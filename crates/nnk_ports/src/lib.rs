//! Ports — DIP: application depends on these traits, not SQLite.

use async_trait::async_trait;
use nnk_domain::{MemberRole, RoomId, RoomState, UserId};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PortError {
    #[error("not found")]
    NotFound,
    #[error("conflict: {0}")]
    Conflict(&'static str),
    #[error("storage error: {0}")]
    Storage(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserRecord {
    pub id: UserId,
    pub username: String,
    pub password_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoomMember {
    pub user_id: UserId,
    pub role: MemberRole,
    pub display_name: String,
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create_user(
        &self,
        id: UserId,
        username: &str,
        password_hash: &str,
    ) -> Result<(), PortError>;

    async fn find_by_username(&self, username: &str) -> Result<Option<UserRecord>, PortError>;

    async fn find_by_id(&self, id: UserId) -> Result<Option<UserRecord>, PortError>;
}

#[async_trait]
pub trait RoomRepository: Send + Sync {
    async fn create_room(
        &self,
        room_id: RoomId,
        code: &str,
        gm_user_id: UserId,
        gm_name: &str,
    ) -> Result<(), PortError>;

    async fn find_id_by_code(&self, code: &str) -> Result<Option<RoomId>, PortError>;

    async fn add_member(
        &self,
        room_id: RoomId,
        user_id: UserId,
        role: MemberRole,
        display_name: &str,
    ) -> Result<(), PortError>;

    async fn get_member(
        &self,
        room_id: RoomId,
        user_id: UserId,
    ) -> Result<Option<RoomMember>, PortError>;

    async fn get_member_by_code(
        &self,
        code: &str,
        user_id: UserId,
    ) -> Result<Option<(RoomId, RoomMember)>, PortError>;

    async fn save_snapshot(&self, code: &str, state: &RoomState) -> Result<(), PortError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Compile-time object-safety / trait surface smoke via dummy impl.
    struct Mem;

    #[async_trait]
    impl UserRepository for Mem {
        async fn create_user(
            &self,
            _id: UserId,
            _username: &str,
            _password_hash: &str,
        ) -> Result<(), PortError> {
            Ok(())
        }
        async fn find_by_username(&self, _: &str) -> Result<Option<UserRecord>, PortError> {
            Ok(None)
        }
        async fn find_by_id(&self, _: UserId) -> Result<Option<UserRecord>, PortError> {
            Ok(None)
        }
    }

    #[tokio::test]
    async fn trait_callable() {
        let m = Mem;
        assert!(m.find_by_username("x").await.unwrap().is_none());
    }
}
