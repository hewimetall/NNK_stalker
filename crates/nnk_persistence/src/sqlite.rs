use async_trait::async_trait;
use nnk_domain::{MemberRole, RoomId, RoomState, UserId};
use nnk_ports::{PortError, RoomMember, RoomRepository, UserRecord, UserRepository};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Clone)]
pub struct SqliteStore {
    pool: SqlitePool,
}

impl SqliteStore {
    pub async fn connect(database_url: &str) -> anyhow::Result<Self> {
        let options = SqliteConnectOptions::from_str(database_url)?.create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;
        let store = Self { pool };
        store.migrate().await?;
        Ok(store)
    }

    pub async fn connect_memory() -> anyhow::Result<Self> {
        Self::connect("sqlite::memory:").await
    }

    async fn migrate(&self) -> anyhow::Result<()> {
        sqlx::query(include_str!("../../../migrations/001_init.sql"))
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

fn map_db(err: sqlx::Error) -> PortError {
    if let sqlx::Error::Database(db) = &err {
        if db.message().contains("UNIQUE") {
            return PortError::Conflict("unique constraint");
        }
    }
    PortError::Storage(err.to_string())
}

#[async_trait]
impl UserRepository for SqliteStore {
    async fn create_user(
        &self,
        id: UserId,
        username: &str,
        password_hash: &str,
    ) -> Result<(), PortError> {
        sqlx::query("INSERT INTO users (id, username, password_hash) VALUES (?, ?, ?)")
            .bind(id.0.to_string())
            .bind(username)
            .bind(password_hash)
            .execute(&self.pool)
            .await
            .map_err(map_db)?;
        Ok(())
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<UserRecord>, PortError> {
        let row: Option<(String, String, String)> = sqlx::query_as(
            "SELECT id, username, password_hash FROM users WHERE username = ? COLLATE NOCASE",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db)?;

        Ok(row.map(|(id, username, password_hash)| UserRecord {
            id: UserId(Uuid::parse_str(&id).unwrap()),
            username,
            password_hash,
        }))
    }

    async fn find_by_id(&self, id: UserId) -> Result<Option<UserRecord>, PortError> {
        let row: Option<(String, String, String)> =
            sqlx::query_as("SELECT id, username, password_hash FROM users WHERE id = ?")
                .bind(id.0.to_string())
                .fetch_optional(&self.pool)
                .await
                .map_err(map_db)?;

        Ok(row.map(|(id, username, password_hash)| UserRecord {
            id: UserId(Uuid::parse_str(&id).unwrap()),
            username,
            password_hash,
        }))
    }
}

#[async_trait]
impl RoomRepository for SqliteStore {
    async fn create_room(
        &self,
        room_id: RoomId,
        code: &str,
        gm_user_id: UserId,
        gm_name: &str,
    ) -> Result<(), PortError> {
        sqlx::query(
            "INSERT INTO rooms (id, code, gm_user_id, status, mission_id) VALUES (?, ?, ?, 'lobby', 1)",
        )
        .bind(room_id.0.to_string())
        .bind(code)
        .bind(gm_user_id.0.to_string())
        .execute(&self.pool)
        .await
        .map_err(map_db)?;

        self.add_member(room_id, gm_user_id, MemberRole::Gm, gm_name)
            .await
    }

    async fn find_id_by_code(&self, code: &str) -> Result<Option<RoomId>, PortError> {
        let id: Option<String> = sqlx::query_scalar("SELECT id FROM rooms WHERE code = ?")
            .bind(code)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db)?;
        Ok(id.map(|id| RoomId(Uuid::parse_str(&id).unwrap())))
    }

    async fn add_member(
        &self,
        room_id: RoomId,
        user_id: UserId,
        role: MemberRole,
        display_name: &str,
    ) -> Result<(), PortError> {
        let role = match role {
            MemberRole::Gm => "gm",
            MemberRole::Player => "player",
        };
        sqlx::query(
            "INSERT OR IGNORE INTO room_members (room_id, user_id, role, display_name) VALUES (?, ?, ?, ?)",
        )
        .bind(room_id.0.to_string())
        .bind(user_id.0.to_string())
        .bind(role)
        .bind(display_name)
        .execute(&self.pool)
        .await
        .map_err(map_db)?;
        Ok(())
    }

    async fn get_member(
        &self,
        room_id: RoomId,
        user_id: UserId,
    ) -> Result<Option<RoomMember>, PortError> {
        let row: Option<(String, String)> = sqlx::query_as(
            "SELECT role, display_name FROM room_members WHERE room_id = ? AND user_id = ?",
        )
        .bind(room_id.0.to_string())
        .bind(user_id.0.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db)?;

        Ok(row.map(|(role, display_name)| RoomMember {
            user_id,
            role: if role == "gm" {
                MemberRole::Gm
            } else {
                MemberRole::Player
            },
            display_name,
        }))
    }

    async fn get_member_by_code(
        &self,
        code: &str,
        user_id: UserId,
    ) -> Result<Option<(RoomId, RoomMember)>, PortError> {
        let row: Option<(String, String, String)> = sqlx::query_as(
            "SELECT r.id, rm.role, rm.display_name FROM rooms r \
             JOIN room_members rm ON rm.room_id = r.id \
             WHERE r.code = ? AND rm.user_id = ?",
        )
        .bind(code)
        .bind(user_id.0.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db)?;

        Ok(row.map(|(id, role, display_name)| {
            (
                RoomId(Uuid::parse_str(&id).unwrap()),
                RoomMember {
                    user_id,
                    role: if role == "gm" {
                        MemberRole::Gm
                    } else {
                        MemberRole::Player
                    },
                    display_name,
                },
            )
        }))
    }

    async fn save_snapshot(&self, code: &str, state: &RoomState) -> Result<(), PortError> {
        let json = serde_json::to_string(state).map_err(|e| PortError::Storage(e.to_string()))?;
        sqlx::query(
            "UPDATE rooms SET snapshot_json = ?, mission_id = ?, updated_at = datetime('now') WHERE code = ?",
        )
        .bind(json)
        .bind(state.mission_id as i64)
        .bind(code)
        .execute(&self.pool)
        .await
        .map_err(map_db)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nnk_domain::PlayerToken;
    use nnk_ports::{RoomRepository, UserRepository};

    #[tokio::test]
    async fn user_crud() {
        let db = SqliteStore::connect_memory().await.unwrap();
        let id = UserId::new();
        db.create_user(id, "Stalker", "hash").await.unwrap();
        let found = db.find_by_username("stalker").await.unwrap().unwrap();
        assert_eq!(found.id, id);
        assert_eq!(found.username, "Stalker");
        assert!(db.create_user(UserId::new(), "stalker", "x").await.is_err());
    }

    #[tokio::test]
    async fn room_members_and_snapshot() {
        let db = SqliteStore::connect_memory().await.unwrap();
        let gm = UserId::new();
        db.create_user(gm, "gm1", "h").await.unwrap();
        let by_id = db.find_by_id(gm).await.unwrap().unwrap();
        assert_eq!(by_id.username, "gm1");
        assert!(db.find_by_id(UserId::new()).await.unwrap().is_none());

        let room = RoomId::new();
        db.create_room(room, "ABCD12", gm, "gm1").await.unwrap();
        assert_eq!(db.find_id_by_code("ABCD12").await.unwrap().unwrap(), room);
        assert!(db.find_id_by_code("ZZZZZZ").await.unwrap().is_none());

        let player = UserId::new();
        db.create_user(player, "p1", "h").await.unwrap();
        db.add_member(room, player, MemberRole::Player, "p1")
            .await
            .unwrap();
        let m = db.get_member(room, player).await.unwrap().unwrap();
        assert_eq!(m.role, MemberRole::Player);
        let m = db.get_member_by_code("ABCD12", player).await.unwrap().unwrap();
        assert_eq!(m.1.role, MemberRole::Player);
        let gm_m = db.get_member_by_code("ABCD12", gm).await.unwrap().unwrap();
        assert_eq!(gm_m.1.role, MemberRole::Gm);

        let state = RoomState::new(room, "ABCD12".into(), PlayerToken::new(gm, "gm1"), 1);
        db.save_snapshot("ABCD12", &state).await.unwrap();
    }
}
