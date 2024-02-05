use std::{ops::DerefMut, str::FromStr};

use deadpool_postgres::{
    tokio_postgres::{Config, NoTls},
    Pool,
};
use deadpool_redis::{Config as RedisConfig, PoolConfig, Runtime};
use dotenvy::var;
use refinery::embed_migrations;
use tracing::info;

embed_migrations!("migrations");

/// Represents the application's database connections, encapsulating both Redis and Postgres pools.
#[derive(Clone)]
pub struct Database {
    /// The Redis connection pool for caching or other Redis operations.
    pub redis: deadpool_redis::Pool,
    /// The PostgreSQL connection pool for database operations.
    pub pg: deadpool_postgres::Pool,
}

impl Database {
    /// Initializes and returns a new `Database` instance by setting up connections to both Redis and Postgres databases.
    ///
    /// It reads database URLs from the environment, creates connection pools for both databases,
    /// and performs database migrations for Postgres.
    ///
    /// # Panics
    /// This function will panic if:
    /// - The environment variables `DATABASE_URL` or `REDIS_URL` are not set.
    /// - It fails to create the connection pools.
    /// - It fails to run database migrations.
    ///
    /// # Examples
    /// ```
    /// let database = Database::default().await;
    /// ```
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

        Self::migrations(&pg).await;

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

    /// Executes database migrations for the Postgres database.
    ///
    /// This is an internal function called during the initialization of the `Database` struct.
    ///
    /// # Panics
    /// This function will panic if it fails to acquire a connection from the pool or to run the migrations.
    #[inline]
    async fn migrations(pg: &Pool) {
        let mut conn = pg.get().await.unwrap();
        let client = conn.deref_mut().deref_mut();
        migrations::runner().run_async(client).await.unwrap();
    }
}
