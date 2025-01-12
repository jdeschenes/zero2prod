use axum::{http::StatusCode, response::IntoResponse};
use uuid::Uuid;

#[tracing::instrument(
    name = "Health Check"
    fields(
        request_id = %Uuid::new_v4(),
    )
)]
pub async fn health_check() -> impl IntoResponse {
    tracing::info!("Checking health check");
    StatusCode::OK
}
