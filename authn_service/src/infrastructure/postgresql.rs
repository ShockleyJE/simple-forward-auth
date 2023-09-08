use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;

pub async fn configure_with_db_url(db_url: &str) -> PgPool {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Unable to connect to Postgresql")
}