use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

use crate::error::error_response;
use crate::multiplayer::lobby;
use crate::services::matches as match_history;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct CreateMatchRequest {
    player_id: String,
}

#[derive(Serialize)]
pub struct CreateMatchResponse {
    match_id: String,
    join_code: String,
}

pub async fn create(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateMatchRequest>,
) -> Response {
    let (match_id, join_code) = lobby::create_match(&state, request.player_id);
    Json(CreateMatchResponse {
        match_id,
        join_code,
    })
    .into_response()
}

#[derive(Deserialize)]
pub struct JoinMatchRequest {
    join_code: String,
    player_id: String,
}

#[derive(Serialize)]
pub struct JoinMatchResponse {
    match_id: String,
}

pub async fn join(
    State(state): State<Arc<AppState>>,
    Json(request): Json<JoinMatchRequest>,
) -> Response {
    match lobby::join_match(&state, &request.join_code, request.player_id).await {
        Ok(match_id) => Json(JoinMatchResponse { match_id }).into_response(),
        Err(message) => error_response(StatusCode::BAD_REQUEST, message),
    }
}

#[derive(Deserialize)]
pub struct HistoryQuery {
    player_id: String,
}

pub async fn history(
    State(state): State<Arc<AppState>>,
    Query(query): Query<HistoryQuery>,
) -> Response {
    match match_history::history_for_player(&state.pool, &query.player_id, 20).await {
        Ok(entries) => Json(entries).into_response(),
        Err(err) => {
            eprintln!("failed to load match history: {err}");
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to load match history",
            )
        }
    }
}
