use actix_web_opentelemetry::RequestMetricsBuilder;
use authn_service::configuration::{get_configuration, DatabaseSettings};
use authn_service::startup::run_app;
use opentelemetry::global;
use serde_json::json;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use tracing_log::log::error;
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    // We retrieve the port assigned to us by the OS
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let meter = global::meter("actix_web");
    // Request metrics middleware
    let request_metrics = RequestMetricsBuilder::new().build(meter);

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database(&configuration.database).await;

    let server = run_app(listener, connection_pool.clone(), request_metrics)
        .expect("Failed to bind address");
    let _ = tokio::spawn(server);
    TestApp {
        address,
        db_pool: connection_pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(&*format!(r#"CREATE DATABASE "{}";"#, config.database_name))
        .await
        .expect("Failed to create database.");

    // Migrate database
    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}

#[tokio::test]
async fn new_key_returns_a_200_for_valid_key_data() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    #[derive(serde::Serialize, serde::Deserialize)]
    struct MyRequestBody {
        environment_id: String,
        cloud_provider_region: String,
        cloud_provider: String,
        name: String,
    }

    let body = json!(
    {
        "environment_id": "my-env-id",
        "cloud_provider_region": "us-east-2",
        "cloud_provider": "aws",
        "name": "My test environment - auto test only"
    });

    let request_body: MyRequestBody =
        serde_json::from_value(body).expect("Failed to serialize body");

    // Act
    let response = client
        .post(&format!(
            "{}/environment/{}/keys",
            &app.address, request_body.environment_id
        ))
        .json(&request_body)
        .send()
        .await
        .unwrap();

    #[derive(serde::Deserialize)]
    struct ResponseBody {
        id: String,
        api_key: String,
    }

    // Assert
    assert_eq!(&200, &response.status().as_u16());

    let res_body: ResponseBody = response
        .json()
        .await
        .expect("Expected response body as json");

    let saved = sqlx::query!(
        "SELECT id, name FROM query_api_access_key WHERE id = $1",
        Uuid::parse_str(&res_body.id).unwrap()
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.name, "My test environment - auto test only");
}
