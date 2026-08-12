use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use deduced_protocol::ProfileSyncRequest;

use crate::error::error_response;
use crate::services::profile as profile_service;
use crate::state::AppState;

pub async fn sync(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ProfileSyncRequest>,
) -> Response {
    match profile_service::sync_profile(&state.pool, &request).await {
        Ok(response) => Json(response).into_response(),
        Err(err) => {
            eprintln!("failed to sync profile: {err}");
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to sync profile")
        }
    }
}
