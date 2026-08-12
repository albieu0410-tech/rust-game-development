use axum::Json;

use deduced_protocol::HealthResponse;

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse::default())
}
