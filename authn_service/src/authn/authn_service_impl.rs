use super::ports::*;
use async_trait::async_trait;
use tracing::debug;
use tracing::info;
use tracing::log::error;
use super::postgres_token_repo::*;
use super::redis_token_repo::*;

pub struct AuthnServiceImpl<A: TokenCache, B: TokenDatabase> {
    pub token_cache: A,
    pub token_database: B,
}

#[async_trait]
impl <A, B> AuthnService for AuthnServiceImpl<A, B>
    where A: TokenCache + Sync + Send, B: TokenDatabase + Sync + Send {

    // Issue a token for an environment regardless of whether one already exists
    // Persist this to the database, and update the cache
    async fn issue(self: &Self, issuance: &Issuance) -> Option<Token> {

        let token = self.token_database.generate_token().await;

        if !self.token_database.store_token(&issuance, &token).await {
            error!("Unable to store token");
            return None; // This represents an issue storing the token
        }
        if !self.token_cache.cache_token(&token, &issuance.environment).await {
            error!("Unable to cache token");
            return None; // This represents an issue caching the token
        }

        Some(token)
    }

    // Authenticate a token by checking the cache first
    // If not existing in the cache, we will check the database
    async fn authenticate(self: &Self, token: &Token) -> Option<bool> {
        match self.token_cache.is_token_cached(token).await {
            Ok(true) => {
                debug!("Token exists in cache");
                Some(true)
            },
            Ok(false) => {
                debug!("Token does not exists in cache");
                match self.token_database.environment_from_token(token).await {
                    Some(environment) => {
                        debug!("Token exists in db, but not cache. Updating cache");
                        let cached = self.token_cache.cache_token(token, &environment).await;
                        if !cached {error!("Failure during cache update on valiation")}
                        Some(true)
                    },
                    None => Some(false)
                }
            },
            Err(e) => {
                error!("Error with TokenCache, {:#?}", e);
                None
            }
        }
    }

    fn build(token_cache: RedisTokenCacheImpl, token_database: PostgresTokenDatabaseImpl) -> AuthnServiceImpl<RedisTokenCacheImpl,PostgresTokenDatabaseImpl> {
        AuthnServiceImpl { token_cache, token_database }
    }
}

impl <A,B> Clone for AuthnServiceImpl<A, B> 
    where A: TokenCache + Clone + Sync + Send, B: TokenDatabase + Clone + Sync + Send {
    fn clone(&self) -> Self {
        return AuthnServiceImpl { token_cache: self.token_cache.clone(), token_database: self.token_database.clone() }
    }
}