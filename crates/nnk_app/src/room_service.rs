use nnk_domain::{MemberRole, PlayerToken, RoomId, RoomState, UserId};
use nnk_ports::RoomRepository;
use nnk_protocol::CreateRoomResponse;
use nnk_rules::{Action, StepOutcome, init, step, try_add_player};
use rand::Rng;
use std::sync::Arc;

use crate::AppError;

pub struct RoomService<R> {
    rooms: Arc<R>,
}

impl<R: RoomRepository> RoomService<R> {
    pub fn new(rooms: Arc<R>) -> Self {
        Self { rooms }
    }

    pub async fn create(
        &self,
        gm_id: UserId,
        gm_name: &str,
    ) -> Result<(CreateRoomResponse, RoomState), AppError> {
        let room_id = RoomId::new();
        let code = generate_code();
        self.rooms
            .create_room(room_id, &code, gm_id, gm_name)
            .await?;
        let seed = rand::thread_rng().gen();
        let state = init(
            room_id,
            code.clone(),
            PlayerToken::new(gm_id, gm_name),
            seed,
        );
        Ok((
            CreateRoomResponse {
                room_id: room_id.0,
                code,
                min_players: state.min_players,
                max_players: state.max_players,
            },
            state,
        ))
    }

    pub async fn join(
        &self,
        code: &str,
        user_id: UserId,
        username: &str,
        live: Option<RoomState>,
    ) -> Result<RoomState, AppError> {
        let code = code.to_uppercase();
        let room_id = self
            .rooms
            .find_id_by_code(&code)
            .await?
            .ok_or(AppError::Message("room not found"))?;

        let live = live.ok_or(AppError::Message(
            "room not in memory — create a new lobby after server restart",
        ))?;
        let state = try_add_player(live, PlayerToken::new(user_id, username))?;

        // Persist membership after capacity check succeeds.
        let role = if state.gm_user_id == user_id {
            MemberRole::Gm
        } else {
            MemberRole::Player
        };
        self.rooms
            .add_member(room_id, user_id, role, username)
            .await?;
        Ok(state)
    }

    pub async fn member_role(
        &self,
        code: &str,
        user_id: UserId,
    ) -> Result<MemberRole, AppError> {
        let (_room, member) = self
            .rooms
            .get_member_by_code(&code.to_uppercase(), user_id)
            .await?
            .ok_or(AppError::Message("not a room member"))?;
        Ok(member.role)
    }

    pub fn apply(
        &self,
        state: RoomState,
        user_id: UserId,
        role: MemberRole,
        action: Action,
    ) -> Result<StepOutcome, AppError> {
        Ok(step(state, user_id, role, action)?)
    }

    pub async fn persist_snapshot(&self, code: &str, state: &RoomState) -> Result<(), AppError> {
        self.rooms.save_snapshot(code, state).await?;
        Ok(())
    }
}

fn generate_code() -> String {
    const ALPH: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut rng = rand::thread_rng();
    (0..6)
        .map(|_| ALPH[rng.gen_range(0..ALPH.len())] as char)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use nnk_persistence::SqliteStore;
    use nnk_ports::UserRepository;
    use nnk_rules::Action;

    #[tokio::test]
    async fn create_join_capacity_and_start() {
        let db = Arc::new(SqliteStore::connect_memory().await.unwrap());
        let gm = UserId::new();
        db.create_user(gm, "gm", "h").await.unwrap();
        let svc = RoomService::new(db.clone());
        let (created, mut state) = svc.create(gm, "gm").await.unwrap();
        assert_eq!(created.max_players, 5);
        assert_eq!(created.min_players, 4);

        for i in 0..4 {
            let player = UserId::new();
            db.create_user(player, &format!("p{i}"), "h").await.unwrap();
            state = svc
                .join(&created.code, player, &format!("p{i}"), Some(state))
                .await
                .unwrap();
        }
        assert_eq!(state.tokens.len(), 5);

        let overflow = UserId::new();
        db.create_user(overflow, "overflow", "h").await.unwrap();
        assert!(svc
            .join(&created.code, overflow, "overflow", Some(state.clone()))
            .await
            .is_err());

        for t in state.tokens.clone() {
            state = svc
                .apply(
                    state,
                    t.user_id,
                    MemberRole::Player,
                    Action::SetReady { ready: true },
                )
                .unwrap()
                .state;
        }
        let out = svc
            .apply(state, gm, MemberRole::Gm, Action::StartGame)
            .unwrap();
        assert_eq!(out.state.phase, nnk_domain::RoomPhase::Playing);
        svc.persist_snapshot(&created.code, &out.state)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn cold_join_without_live_rejected() {
        let db = Arc::new(SqliteStore::connect_memory().await.unwrap());
        let gm = UserId::new();
        db.create_user(gm, "gm2", "h").await.unwrap();
        let svc = RoomService::new(db.clone());
        let (created, _state) = svc.create(gm, "gm2").await.unwrap();
        let player = UserId::new();
        db.create_user(player, "solo", "h").await.unwrap();
        let err = svc
            .join(&created.code, player, "solo", None)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Message(_)));
    }

    #[tokio::test]
    async fn rejoin_same_player_idempotent() {
        let db = Arc::new(SqliteStore::connect_memory().await.unwrap());
        let gm = UserId::new();
        db.create_user(gm, "gm3", "h").await.unwrap();
        let svc = RoomService::new(db.clone());
        let (created, state) = svc.create(gm, "gm3").await.unwrap();
        let state = svc
            .join(&created.code, gm, "gm3", Some(state))
            .await
            .unwrap();
        assert_eq!(state.tokens.len(), 1);
        assert_eq!(svc.member_role(&created.code, gm).await.unwrap(), MemberRole::Gm);
    }
}
