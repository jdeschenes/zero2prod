use std::future::IntoFuture;

use eyre::{Context, Result};

use tokio::net::TcpListener;
use zero2prod::telemetry::{get_subscriber, init_subscriber};
use zero2prod::{configuration::get_configuration, run};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber)?;
    let configuration = get_configuration().context("Getting configuration")?;
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    tracing::info!("Address to bind: {}", address);
    let listener = TcpListener::bind(address)
        .await
        .context("Binding listener")?;
    let server = run(listener).await?;
    server.into_future().await?;
    Ok(())
}
