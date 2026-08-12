mod api;
mod config;
mod error;
mod multiplayer;
mod services;
mod state;

use std::path::Path;
use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use sqlx::postgres::PgPoolOptions;

use deduced_content::load_content_from_dir;

use state::AppState;

#[tokio::main]
async fn main() {
    let content = load_content_from_dir(Path::new("content"))
        .expect("game content must load from the content/ directory");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config::database_url())
        .await
        .expect(
            "failed to connect to Postgres — is it running? try `docker compose up -d postgres`",
        );

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run database migrations");

    let state = Arc::new(AppState {
        content,
        pool,
        multiplayer: multiplayer::MultiplayerState::default(),
    });

    let app = Router::new()
        .route("/health", get(api::health::health))
        .route("/daily/current", get(api::daily::current))
        .route("/daily/submit", post(api::daily::submit))
        .route("/daily/leaderboard", get(api::daily::leaderboard))
        .route("/profile/sync", post(api::profile::sync))
        .route("/matches", post(api::matches::create))
        .route("/matches/join", post(api::matches::join))
        .route("/matches/history", get(api::matches::history))
        .route(
            "/matches/{match_id}/ws",
            get(multiplayer::websocket::handler),
        )
        .route("/matchmaking/queue", post(api::matchmaking::queue))
        .route("/matchmaking/status", get(api::matchmaking::status))
        .route("/matchmaking/leave", post(api::matchmaking::leave))
        .with_state(state);

    let addr = config::bind_addr();
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind to address");

    println!("DEDUCED server running at http://{addr}");
    axum::serve(listener, app).await.expect("server error");
}
