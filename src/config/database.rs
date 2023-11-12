use dotenvy::var;
use redis::{aio::Connection, Client};
use redis_pool::RedisPool;
use sqlx::MySqlPool;
use tracing::info;

pub struct Database {
    pub redis: RedisPool<Client, Connection>,
    pub mysql: MySqlPool,
}

impl Database {
    pub async fn default() -> Self {
        info!("Connecting Databases...");
        let mysql = MySqlPool::connect(&var("DATABASE_URL").unwrap())
            .await
            .unwrap();

        let client = Client::open(var("REDIS_URL").unwrap()).unwrap();
        let redis = RedisPool::new(client, 16, Some(200));

        Self { redis, mysql }
    }
}
