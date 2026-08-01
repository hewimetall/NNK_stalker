use dashmap::DashMap;
use nnk_app::{AuthService, RoomService};
use nnk_auth::{Argon2PasswordService, JwtTokenService};
use nnk_domain::RoomState;
use nnk_persistence::SqliteStore;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};

pub type SharedAuth = Arc<AuthService<SqliteStore, Argon2PasswordService, JwtTokenService>>;
pub type SharedRooms = Arc<RoomService<SqliteStore>>;

#[derive(Clone)]
pub struct AppState {
    pub auth: SharedAuth,
    pub rooms: SharedRooms,
    pub live: Arc<DashMap<String, Arc<LiveRoom>>>,
}

pub struct LiveRoom {
    pub state: RwLock<RoomState>,
    pub tx: broadcast::Sender<nnk_protocol::ServerMsg>,
}

impl AppState {
    pub fn new(auth: SharedAuth, rooms: SharedRooms) -> Self {
        Self {
            auth,
            rooms,
            live: Arc::new(DashMap::new()),
        }
    }
}
