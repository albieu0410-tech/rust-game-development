use std::sync::Arc;
use std::time::Instant;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use deduced_core::{Comparison, Round, RoundConfig, RoundStatus, score_round};

use crate::state::{AppState, RoundSession};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/categories", get(list_categories))
        .route("/api/categories/{id}/answers", get(list_answers))
        .route("/api/round", get(current_round).post(start_round))
        .route("/api/round/reset", post(reset_round))
        .route("/api/guess", post(submit_guess))
}

#[derive(Serialize)]
struct CategoryDto {
    id: String,
    name: String,
    attempts: usize,
    answer_count: usize,
}

#[derive(Serialize)]
struct AnswerDto {
    id: String,
    name: String,
}

#[derive(Serialize)]
struct ComparisonDto {
    key: String,
    label: String,
    guessed_value: String,
    comparison: &'static str,
}

#[derive(Serialize)]
struct GuessOutcomeDto {
    guessed_name: String,
    comparisons: Vec<ComparisonDto>,
}

#[derive(Serialize)]
struct RoundStateDto {
    category_id: String,
    category_name: String,
    attempts_used: usize,
    max_attempts: usize,
    status: &'static str,
    last_guess: Option<GuessOutcomeDto>,
    answer_name: Option<String>,
    score: Option<u32>,
    elapsed_seconds: Option<u64>,
}

#[derive(Serialize)]
struct ErrorDto {
    error: String,
}

fn error_response(status: StatusCode, message: impl Into<String>) -> Response {
    (
        status,
        Json(ErrorDto {
            error: message.into(),
        }),
    )
        .into_response()
}

fn comparison_tag(comparison: Comparison) -> &'static str {
    match comparison {
        Comparison::Match => "match",
        Comparison::Higher => "higher",
        Comparison::Lower => "lower",
        Comparison::Different => "different",
        Comparison::Partial => "partial",
    }
}

fn round_state_dto(
    state: &AppState,
    session: &RoundSession,
    last_guess: Option<GuessOutcomeDto>,
) -> RoundStateDto {
    let round = &session.round;
    let category = state
        .content
        .category(&round.answer.category)
        .expect("round category always exists in loaded content");

    let finished = round.status != RoundStatus::Playing;

    RoundStateDto {
        category_id: category.id.clone(),
        category_name: category.name.clone(),
        attempts_used: round.attempts_used(),
        max_attempts: round.max_attempts,
        status: match round.status {
            RoundStatus::Playing => "playing",
            RoundStatus::Won => "won",
            RoundStatus::Lost => "lost",
        },
        last_guess,
        answer_name: finished.then(|| round.answer.name.clone()),
        score: finished.then(|| score_round(round).points),
        elapsed_seconds: finished.then(|| session.started_at.elapsed().as_secs()),
    }
}

async fn list_categories(State(state): State<Arc<AppState>>) -> Json<Vec<CategoryDto>> {
    let categories = state
        .content
        .categories
        .iter()
        .map(|category| CategoryDto {
            id: category.id.clone(),
            name: category.name.clone(),
            attempts: category.attempts,
            answer_count: state.content.answers_for_category(&category.id).count(),
        })
        .collect();

    Json(categories)
}

async fn list_answers(
    State(state): State<Arc<AppState>>,
    Path(category_id): Path<String>,
) -> Json<Vec<AnswerDto>> {
    let answers = state
        .content
        .answers_for_category(&category_id)
        .map(|answer| AnswerDto {
            id: answer.id.clone(),
            name: answer.name.clone(),
        })
        .collect();

    Json(answers)
}

#[derive(Deserialize)]
struct StartRoundRequest {
    category_id: String,
}

async fn start_round(
    State(state): State<Arc<AppState>>,
    Json(request): Json<StartRoundRequest>,
) -> Response {
    let Some(category) = state.content.category(&request.category_id) else {
        return error_response(StatusCode::NOT_FOUND, "unknown category");
    };

    let seed: u64 = rand::random();
    let round = match Round::new(
        &state.content.answers,
        RoundConfig {
            category: category.id.clone(),
            seed,
            max_attempts: category.attempts,
        },
    ) {
        Ok(round) => round,
        Err(err) => return error_response(StatusCode::BAD_REQUEST, err.to_string()),
    };

    let session = RoundSession {
        round,
        started_at: Instant::now(),
    };
    let dto = round_state_dto(&state, &session, None);

    *state.session.lock().expect("session mutex poisoned") = Some(session);

    Json(dto).into_response()
}

async fn current_round(State(state): State<Arc<AppState>>) -> Response {
    let guard = state.session.lock().expect("session mutex poisoned");
    match guard.as_ref() {
        Some(session) => Json(round_state_dto(&state, session, None)).into_response(),
        None => error_response(StatusCode::NOT_FOUND, "no active round"),
    }
}

async fn reset_round(State(state): State<Arc<AppState>>) -> StatusCode {
    *state.session.lock().expect("session mutex poisoned") = None;
    StatusCode::NO_CONTENT
}

#[derive(Deserialize)]
struct GuessRequest {
    name: String,
}

async fn submit_guess(
    State(state): State<Arc<AppState>>,
    Json(request): Json<GuessRequest>,
) -> Response {
    let mut guard = state.session.lock().expect("session mutex poisoned");
    let Some(session) = guard.as_mut() else {
        return error_response(StatusCode::NOT_FOUND, "no active round");
    };

    let category_id = session.round.answer.category.clone();
    let Some(category) = state.content.category(&category_id) else {
        return error_response(StatusCode::NOT_FOUND, "unknown category");
    };

    let Some(guess) = state.content.find_answer(&category_id, &request.name) else {
        return error_response(StatusCode::BAD_REQUEST, "no answer matches that name");
    };
    let guess = guess.clone();

    let result = match session.round.submit_guess(category, &guess) {
        Ok(result) => result.clone(),
        Err(err) => return error_response(StatusCode::CONFLICT, err.to_string()),
    };

    let last_guess = GuessOutcomeDto {
        guessed_name: result.answer_name,
        comparisons: result
            .comparisons
            .into_iter()
            .map(|comparison| ComparisonDto {
                key: comparison.key,
                label: comparison.label,
                guessed_value: comparison.guessed_value.display_value(),
                comparison: comparison_tag(comparison.comparison),
            })
            .collect(),
    };

    Json(round_state_dto(&state, session, Some(last_guess))).into_response()
}
