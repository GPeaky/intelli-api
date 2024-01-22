use std::{ops::DerefMut, str::FromStr};

use deadpool_postgres::tokio_postgres::{Config, NoTls};
use deadpool_redis::{Config as RedisConfig, PoolConfig, Runtime};
use dotenvy::var;
use tracing::info;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("migrations");
}

#[derive(Clone)]
pub struct Database {
    pub redis: deadpool_redis::Pool,
    pub pg: deadpool_postgres::Pool,
}

impl Database {
    pub async fn default() -> Self {
        info!("Connecting Databases...");

        // Postgres connection
        let pg = {
            let config =
                Config::from_str(&var("DATABASE_URL").expect("Environment DATABASE_URL not found"))
                    .unwrap();

            let manager = deadpool_postgres::Manager::new(config, NoTls);

            deadpool_postgres::Pool::builder(manager)
                .max_size(100)
                .build()
                .unwrap()
        };

        // Run migrations
        {
            let mut conn = pg.get().await.unwrap();
            let client = conn.deref_mut().deref_mut();

            embedded::migrations::runner()
                .run_async(client)
                .await
                .unwrap();
        }

        let redis = {
            let mut config =
                RedisConfig::from_url(var("REDIS_URL").expect("Environment REDIS_URL not found"));
            config.pool = Some(PoolConfig::new(300)); // Set the pool size

            config
                .create_pool(Some(Runtime::Tokio1))
                .expect("Failed to create Redis pool")
        };

        // Redis connection

        Self { redis, pg }
    }
}
