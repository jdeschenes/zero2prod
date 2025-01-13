use std::future::IntoFuture;
use std::sync::LazyLock;

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use rustls::ClientConfig;
use secrecy::ExposeSecret;
use tokio_postgres::Config;
use tokio_postgres_rustls::MakeRustlsConnect;

use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

struct TestApp {
    address: String,
}

static TRACING: LazyLock<()> = LazyLock::new(|| {
    let default_filter_level = "debug".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber).expect("failed to initialize subscriber");
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber).expect("failed to initialize subscriber");
    };
});

#[tokio::test]
async fn health_check_works() {
    let test_app = spawn_app().await;
    let address = test_app.address;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn health_check_works2() {
    let test_app = spawn_app().await;
    let address = test_app.address;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

pub async fn configure_database(
    configuration: &DatabaseSettings,
) -> Pool<PostgresConnectionManager<MakeRustlsConnect>> {
    let mut config = Config::new();
    let rust_tls_config = ClientConfig::builder()
        .with_root_certificates(rustls::RootCertStore::empty())
        .with_no_client_auth();
    let tls = MakeRustlsConnect::new(rust_tls_config);
    // Connect to the database.
    let (client, connection) = tokio_postgres::connect(
        "host=127.0.0.1 user=postgres password=password",
        tls.clone(),
    )
    .await
    .expect("Error connecting to DB");

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    client
        .execute(
            &format!(r#"CREATE DATABASE "{}";"#, configuration.database),
            &[],
        )
        .await
        .expect("Unable to create database");
    // Open a Connection
    // Create Database
    // Migrate

    config
        .host(&configuration.host)
        .port(configuration.port)
        .user(&configuration.user)
        .password(configuration.password.expose_secret())
        .dbname(&configuration.database);
    let manager = PostgresConnectionManager::new(config, tls);
    let pool = Pool::builder()
        .build(manager)
        .await
        .expect("Could not create DB Pool");
    pool
}

async fn spawn_app() -> TestApp {
    LazyLock::force(&TRACING);
    let mut configuration = get_configuration().expect("Unable to get configuration");
    configuration.database.database = uuid::Uuid::new_v4().to_string();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Binding listener");
    let port = listener.local_addr().unwrap().port();
    let pool = configure_database(&configuration.database).await;

    let server = run(listener, pool.clone())
        .await
        .expect("Failed to bind address");

    let _ = tokio::spawn(server.into_future());
    TestApp {
        address: format!("http://127.0.0.1:{}", port),
    }
}
