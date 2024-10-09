use std::{ops::DerefMut, str::FromStr};

use cache::ServiceCache;
use deadpool_postgres::{
    tokio_postgres::{Config, NoTls},
    Manager, Pool,
};
use dotenvy::var;
use refinery::embed_migrations;
use tracing::info;

pub use cache::EntityCache;

mod cache;

embed_migrations!("migrations");

/// Encapsulates Postgres database connections.
pub struct Database {
    pub pg: Pool,
    pub cache: ServiceCache,
}

impl Database {
    /// Initializes a new `Database` instance.
    ///
    /// # Panics
    /// If DATABASE_URL is not set, pool creation fails, or migrations fail.
    pub async fn new() -> Self {
        info!("Connecting Databases...");

        // Postgres connection
        let pg = {
            let config =
                Config::from_str(&var("DATABASE_URL").expect("Environment DATABASE_URL not found"))
                    .unwrap();

            let manager = Manager::new(config, NoTls);
            Pool::builder(manager).max_size(200).build().unwrap()
        };

        Self::run_migrations(&pg).await;

        let cache = ServiceCache::new();

        Self { pg, cache }
    }

    /// Runs database migrations.
    #[inline]
    async fn run_migrations(pg: &Pool) {
        let mut conn = pg.get().await.unwrap();
        let client = conn.deref_mut().deref_mut();
        migrations::runner().run_async(client).await.unwrap();
    }
}
