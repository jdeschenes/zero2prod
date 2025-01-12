use axum::{routing::get, serve::Serve, Router};
use eyre::{Ok, Result};

use routes::health_check;

pub mod configuration;
mod routes;
pub mod telemetry;

pub async fn run(
    listener: tokio::net::TcpListener,
) -> Result<Serve<tokio::net::TcpListener, Router, Router>> {
    let app = Router::new().route("/health_check", get(health_check));
    Ok(axum::serve(listener, app))
}
