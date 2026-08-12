use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;

use deduced_protocol::QueueResponse;

use crate::multiplayer::matchmaking;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct QueueRequest {
    player_id: String,
}

pub async fn queue(
    State(state): State<Arc<AppState>>,
    Json(request): Json<QueueRequest>,
) -> Json<QueueResponse> {
    Json(matchmaking::enqueue(&state, request.player_id).await)
}

#[derive(Deserialize)]
pub struct PlayerQuery {
    player_id: String,
}

pub async fn status(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PlayerQuery>,
) -> Json<QueueResponse> {
    Json(matchmaking::status(&state, &query.player_id))
}

pub async fn leave(
    State(state): State<Arc<AppState>>,
    Json(request): Json<QueueRequest>,
) -> axum::http::StatusCode {
    matchmaking::leave(&state, &request.player_id);
    axum::http::StatusCode::NO_CONTENT
}
