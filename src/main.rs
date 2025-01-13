use std::future::IntoFuture;

use eyre::{Context, Result};

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use rustls::ClientConfig;
use secrecy::ExposeSecret;
use tokio::net::TcpListener;
use tokio_postgres::Config;
use tokio_postgres_rustls::MakeRustlsConnect;

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
    let mut config = Config::new();
    let rust_tls_config = ClientConfig::builder()
        .with_root_certificates(rustls::RootCertStore::empty())
        .with_no_client_auth();
    let tls = MakeRustlsConnect::new(rust_tls_config);
    config
        .host(&configuration.database.host)
        .port(configuration.database.port)
        .user(&configuration.database.user)
        .password(configuration.database.password.expose_secret())
        .dbname(&configuration.database.database);
    let manager = PostgresConnectionManager::new(config, tls);
    let pool = Pool::builder()
        .build(manager)
        .await
        .context("Creating Database pool")?;
    pool.get()
        .await
        .context("Grabbing connection for testing")?
        .query_one("SELECT 1", &[])
        .await
        .context("Querying the DB")?;
    tracing::info!("Binding to: {}", address);
    let listener = TcpListener::bind(address)
        .await
        .context("Binding listener")?;
    let server = run(listener, pool).await.context("Getting server")?;
    server.into_future().await.context("Serving traffic")?;
    Ok(())
}
