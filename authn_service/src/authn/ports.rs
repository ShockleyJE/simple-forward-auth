use std::{sync::Arc, error::Error};

use async_trait::async_trait;
use redis::RedisError;
use serde::{Serialize, Deserialize};
use sqlx::PgPool;
use super::{authn_service_impl::AuthnServiceImpl, redis_token_repo::RedisTokenCacheImpl, postgres_token_repo::PostgresTokenDatabaseImpl};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct Issuance {
    pub environment: String,
}

pub type Token = String;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AuthnService {
    async fn issue(&self, issuance: &Issuance) -> Option<Token>;
    async fn authenticate(&self, token: &Token) -> Option<bool>;
    fn build(token_cache: RedisTokenCacheImpl, token_database: PostgresTokenDatabaseImpl) -> AuthnServiceImpl<RedisTokenCacheImpl,PostgresTokenDatabaseImpl>;
}

#[async_trait]
pub trait TokenDatabase {
    async fn store_token(&self, issuance: &Issuance, token: &String) -> bool;
    async fn generate_token(&self) -> Token;
    async fn environment_from_token(&self, token: &String) -> Option<String>;
    fn build(pg_pool: Arc<PgPool>) -> PostgresTokenDatabaseImpl;
}

#[async_trait]
pub trait TokenCache {
    async fn cache_token(&self, token: &Token, environment: &String) -> bool;
    async fn is_token_cached(&self, token: &Token) -> Result<bool, RedisError>;
    fn build(client: Arc<redis::Client>) -> RedisTokenCacheImpl;
}