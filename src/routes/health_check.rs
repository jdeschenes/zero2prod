use axum::{extract::State, http::StatusCode, response::IntoResponse};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use eyre::Context;
use tokio_postgres_rustls::MakeRustlsConnect;
use uuid::Uuid;

type ConnectionPool = Pool<PostgresConnectionManager<MakeRustlsConnect>>;

pub struct HealthCheckError(eyre::Report);

impl IntoResponse for HealthCheckError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("Internal server error: {}", self.0);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error".to_string(),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for HealthCheckError
where
    E: Into<eyre::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

#[tracing::instrument(
    name = "Health Check"
    fields(
        request_id = %Uuid::new_v4(),
    )
)]
pub async fn health_check(State(pool): State<ConnectionPool>) -> Result<(), HealthCheckError> {
    let conn = pool.get().await.context("Getting a connection pool")?;
    conn.query_one("SELECT 1", &[])
        .await
        .context("Perform databse query")?;
    tracing::info!("Checking health check");
    Ok(())
}
