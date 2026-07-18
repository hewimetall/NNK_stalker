mod http;
mod state;
mod ws;

use axum::Router;
use axum::http::{HeaderValue, header};
use axum::routing::{get, post};
use nnk_app::{AuthService, RoomService};
use nnk_auth::{Argon2PasswordService, JwtTokenService};
use nnk_persistence::SqliteStore;
use state::AppState;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("nnk_server=info".parse()?))
        .init();

    let database_url =
        std::env::var("NNK_DATABASE_URL").unwrap_or_else(|_| "sqlite:nnk.db?mode=rwc".into());
    let jwt_secret = std::env::var("NNK_JWT_SECRET").unwrap_or_else(|_| {
        tracing::warn!("NNK_JWT_SECRET not set — using insecure dev default");
        "dev-only-change-me".into()
    });
    let bind = std::env::var("NNK_BIND").unwrap_or_else(|_| "0.0.0.0:8080".into());
    let static_dir = std::env::var("NNK_STATIC_DIR").unwrap_or_else(|_| "web".into());

    let store = Arc::new(SqliteStore::connect(&database_url).await?);
    let auth = Arc::new(AuthService::new(
        store.clone(),
        Arc::new(Argon2PasswordService),
        Arc::new(JwtTokenService::new(jwt_secret, 12)),
    ));
    let rooms = Arc::new(RoomService::new(store));
    let state = AppState::new(auth, rooms);

    let index = PathBuf::from(&static_dir).join("index.html");
    let static_files = ServeDir::new(&static_dir).not_found_service(ServeFile::new(index));

    let api = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/api/auth/register", post(http::register))
        .route("/api/auth/login", post(http::login))
        .route("/api/me", get(http::me))
        .route("/api/rooms", post(http::create_room))
        .route("/api/rooms/{code}", get(http::get_lobby))
        .route("/api/rooms/{code}/join", post(http::join_room))
        .route("/api/rooms/{code}/ready", post(http::set_ready))
        .route("/api/rooms/{code}/start", post(http::start_game))
        .route("/ws/rooms/{code}", get(ws::ws_handler))
        .with_state(state);

    let app = Router::new()
        .merge(api)
        .fallback_service(static_files)
        .layer(SetResponseHeaderLayer::overriding(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-store, max-age=0"),
        ))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = bind.parse()?;
    tracing::info!("nnk_server listening on http://{addr} (lobby UI from {static_dir}/)");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
