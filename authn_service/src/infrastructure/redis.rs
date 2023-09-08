pub use redis::Client;

pub async fn configure_with_redis_url(redis_url: &str) -> redis::Client {
    redis::Client::open(redis_url).expect("Unable to connect to Redis")
}