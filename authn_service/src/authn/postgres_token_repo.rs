use super::ports::*;
use config::builder;
use sqlx::PgPool;
use async_trait::async_trait;
use sqlx::query;
use uuid::Uuid;
use std::{sync::Arc};
use tracing::{info, error, warn};
use autometrics::autometrics;

pub struct PostgresTokenDatabaseImpl {
    pub pg_pool: Arc<PgPool>
}

#[async_trait]
impl TokenDatabase for PostgresTokenDatabaseImpl {

    async fn generate_token(self: &Self) -> Token {
        Uuid::new_v4().to_string()
    }

    #[autometrics]
    async fn store_token(self: &Self, issuance: &Issuance, token: &String) -> bool {
        let query_result = 
            query!("insert into api_keys (environment, token) values ($1, crypt($2, gen_salt('bf')))", &issuance.environment, &token)
            .execute(&*self.pg_pool)
            .await;
            
        match query_result {
            Ok(v) => {
                match v.rows_affected() {
                    0 => {
                        info!("Token record not stored");
                        false
                    },
                    1 => {
                        info!("Token record stored");
                        true
                    },
                    _ => {
                        warn!("Unexpected number of tokens stored");
                        false
                    }
                }
            },
            Err(e) => {
                error!("Error when storing token: {:?}", e);
                false
            }
        }
    }

    #[autometrics]
    async fn environment_from_token(self: &Self, token: &String) -> Option<String> {
        let query_result = 
            query!(r#"select environment as "environment" from api_keys where token = crypt($1, token)"#, &token)
            .fetch_one(&*self.pg_pool)
            .await;
        match query_result {
            Ok(row) => {
                Some(row.environment)
            },
            Err(_e) => {
                None
            }
        }
    }

    fn build(pg_pool: Arc<PgPool>) -> PostgresTokenDatabaseImpl {
        PostgresTokenDatabaseImpl { pg_pool: pg_pool }
    }
}

impl Clone for PostgresTokenDatabaseImpl {
    fn clone(&self) -> Self {
        return PostgresTokenDatabaseImpl { pg_pool: self.pg_pool.clone() }
    }
}