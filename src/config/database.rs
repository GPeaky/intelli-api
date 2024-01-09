use std::str::FromStr;

use deadpool_postgres::tokio_postgres::{Config, NoTls};
use deadpool_redis::{Config as RedisConfig, PoolConfig, Runtime};
use dotenvy::var;
use tracing::info;

#[derive(Clone)]
pub struct Database {
    pub redis: deadpool_redis::Pool,
    pub pg: deadpool_postgres::Pool,
}

impl Database {
    pub async fn default() -> Self {
        info!("Connecting Databases...");

        // Postgres connection
        let config =
            Config::from_str(&var("DATABASE_URL").expect("Environment DATABASE_URL not found"))
                .unwrap();

        let manager = deadpool_postgres::Manager::new(config, NoTls);
        let pg = deadpool_postgres::Pool::builder(manager)
            .max_size(100)
            .build()
            .unwrap();

        // Redis connection
        let mut config =
            RedisConfig::from_url(var("REDIS_URL").expect("Environment REDIS_URL not found"));
        config.pool = Some(PoolConfig::new(300)); // Set the pool size

        let redis = config
            .create_pool(Some(Runtime::Tokio1))
            .expect("Failed to create Redis pool");

        Self { redis, pg }
    }
}
