use super::ports::*;
use async_trait::async_trait;
use redis::{AsyncCommands, RedisError};
use tracing::{info, debug, log::error};
use std::{sync::Arc, error::Error};

pub struct RedisTokenCacheImpl {
    pub redis_client: Arc<redis::Client>
}

#[async_trait]
impl TokenCache for RedisTokenCacheImpl {

    async fn cache_token(self: &Self, token: &Token, environment: &String) -> bool {
        let redis_client = &*self.redis_client;
        if let Ok(mut conn) = redis_client.get_async_connection().await {
            let key = format!("token:{}", token);
            conn.set_ex(key, environment, 60)
                .await
                .map(|_: String| true)
                .unwrap_or(false)
        } else {
            false
        }
    }

    async fn is_token_cached(self: &Self, token: &Token) -> Result<bool, RedisError> {
        let mut redis_client = &*self.redis_client;

        match redis_client.get_async_connection().await {
            Ok(mut conn) => {

                let key = format!("token:{}", &token);
                match conn.exists(key).await {
                    Ok(mybool) => match mybool{
                        true => {
                            debug!("Token in cache");
                            Ok(true)
                        },
                        false => {
                            debug!("Token not in cache");
                            Ok(false)
                        }
                    },
                    Err(e) => {
                        error!("Error checking redis cache, {:#?}", e);
                        Err(e)
                    }
                }
            }, 
            Err(e) => {
                error!("Error establishing connection to redis cache, {:#?}", e);
                Err(e)
            }
        }
    }

    fn build(client: Arc<redis::Client>) -> RedisTokenCacheImpl {
        RedisTokenCacheImpl { redis_client: client }
    }
}

impl Clone for RedisTokenCacheImpl {
    fn clone(&self) -> Self {
        return RedisTokenCacheImpl { redis_client: self.redis_client.clone() }
    }
}
