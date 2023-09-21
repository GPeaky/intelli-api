use dotenvy::var;
use redis::{aio::Connection, Client};
use sqlx::MySqlPool;
use tracing::info;
pub struct Database {
    redis: Client,
    pub mysql: MySqlPool,
}

impl Database {
    pub async fn default() -> Self {
        info!("Connecting Databases...");
        let mysql = MySqlPool::connect(&var("DATABASE_URL").unwrap())
            .await
            .unwrap();

        let redis = Client::open(var("REDIS_URL").unwrap()).unwrap();

        info!("Prepared Statements Saved!, Returning Database Instance");
        Self { redis, mysql }
    }

    pub async fn get_redis_async(&self) -> Connection {
        self.redis.get_async_connection().await.unwrap()
    }
}
