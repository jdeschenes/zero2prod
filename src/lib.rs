use axum::{routing::get, serve::Serve, Router};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use eyre::Result;

use routes::health_check;
use tokio_postgres_rustls::MakeRustlsConnect;

pub mod configuration;
mod routes;
pub mod telemetry;

pub async fn run(
    listener: tokio::net::TcpListener,
    pg_pool: Pool<PostgresConnectionManager<MakeRustlsConnect>>,
) -> Result<Serve<tokio::net::TcpListener, Router, Router>> {
    let app = Router::new()
        .route("/health_check", get(health_check))
        .with_state(pg_pool);
    Ok(axum::serve(listener, app))
}
