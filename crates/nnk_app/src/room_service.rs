use nnk_domain::{MemberRole, PlayerToken, RoomId, RoomState, UserId};
use nnk_ports::RoomRepository;
use nnk_protocol::CreateRoomResponse;
use nnk_rules::{Action, StepOutcome, add_player, init, step};
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
        self.rooms
            .add_member(room_id, user_id, MemberRole::Player, username)
            .await?;

        let state = match live {
            Some(s) => add_player(s, PlayerToken::new(user_id, username)),
            None => init(
                room_id,
                code,
                PlayerToken::new(user_id, username),
                rand::thread_rng().gen(),
            ),
        };
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
    async fn create_join_apply() {
        let db = Arc::new(SqliteStore::connect_memory().await.unwrap());
        let gm = UserId::new();
        db.create_user(gm, "gm", "h").await.unwrap();
        let svc = RoomService::new(db.clone());
        let (created, state) = svc.create(gm, "gm").await.unwrap();
        assert_eq!(created.code.len(), 6);

        let player = UserId::new();
        db.create_user(player, "p", "h").await.unwrap();
        let state = svc
            .join(&created.code, player, "p", Some(state))
            .await
            .unwrap();
        assert_eq!(state.tokens.len(), 2);

        let role = svc.member_role(&created.code, gm).await.unwrap();
        assert_eq!(role, MemberRole::Gm);

        let out = svc
            .apply(state, gm, MemberRole::Gm, Action::StartMission { mission_id: 2 })
            .unwrap();
        assert_eq!(out.state.mission_id, 2);
        svc.persist_snapshot(&created.code, &out.state)
            .await
            .unwrap();
    }
}
