use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use deduced_protocol::{DailyChallenge, DailyLeaderboard, DailySubmissionRequest};

use crate::error::error_response;
use crate::services::daily as daily_service;
use crate::state::AppState;

pub async fn current(State(state): State<Arc<AppState>>) -> Json<DailyChallenge> {
    let day_index = daily_service::today_day_index();
    let category = daily_service::category_for_day(day_index, &state.content.categories);
    let seed = daily_service::seed_for_day(day_index);

    Json(DailyChallenge {
        challenge_id: daily_service::challenge_id_for(day_index, &category.id),
        category_id: category.id.clone(),
        seed,
        content_version: state.content.content_version.clone(),
    })
}

pub async fn submit(
    State(state): State<Arc<AppState>>,
    Json(request): Json<DailySubmissionRequest>,
) -> Response {
    let (day_index, category_id) = match daily_service::parse_challenge_id(&request.challenge_id) {
        Ok(parsed) => parsed,
        Err(err) => return error_response(StatusCode::BAD_REQUEST, err.to_string()),
    };
    let seed = daily_service::seed_for_day(day_index);

    let result = match daily_service::replay_submission(
        &state.content,
        &category_id,
        seed,
        &request.guesses,
    ) {
        Ok(result) => result,
        Err(err) => return error_response(StatusCode::BAD_REQUEST, err.to_string()),
    };

    match daily_service::record_submission(
        &state.pool,
        &request.challenge_id,
        &request.player_id,
        &result,
        request.elapsed_ms,
    )
    .await
    {
        Ok(true) => Json(result).into_response(),
        Ok(false) => error_response(
            StatusCode::CONFLICT,
            "this player has already submitted today's Daily challenge",
        ),
        Err(err) => {
            eprintln!("failed to store daily submission: {err}");
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to store submission",
            )
        }
    }
}

#[derive(Deserialize)]
pub struct LeaderboardQuery {
    challenge_id: String,
}

pub async fn leaderboard(
    State(state): State<Arc<AppState>>,
    Query(query): Query<LeaderboardQuery>,
) -> Response {
    match daily_service::leaderboard(&state.pool, &query.challenge_id, 20).await {
        Ok(entries) => Json(DailyLeaderboard {
            challenge_id: query.challenge_id,
            entries,
        })
        .into_response(),
        Err(err) => {
            eprintln!("failed to load daily leaderboard: {err}");
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to load leaderboard",
            )
        }
    }
}
