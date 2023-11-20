use bb8_postgres::{tokio_postgres::NoTls, PostgresConnectionManager};
use bb8_redis::{
    bb8::{self, Pool},
    RedisConnectionManager,
};
use dotenvy::var;
use tracing::info;

pub struct Database {
    pub redis: Pool<RedisConnectionManager>,
    pub pg: Pool<PostgresConnectionManager<NoTls>>,
}

impl Database {
    pub async fn default() -> Self {
        info!("Connecting Databases...");
        let pg_manager = PostgresConnectionManager::new_from_stringlike(
            var("DATABASE_URL").expect("Database url not found"),
            NoTls,
        )
        .unwrap();

        let pg = bb8::Pool::builder()
            .min_idle(Some(1))
            .max_size(1000) // Test if 100 is a good number
            .build(pg_manager)
            .await
            .unwrap();

        let manager =
            RedisConnectionManager::new(var("REDIS_URL").expect("Environment REDIS_URL not found"))
                .unwrap();

        let redis = bb8::Pool::builder()
            .min_idle(Some(1))
            .max_size(1000) // Test if 100 is a good number
            .build(manager)
            .await
            .unwrap();

        Self { redis, pg }
    }

    pub fn active_pools(&self) -> (u32, u32) {
        let redis = self.redis.state();
        let pg = self.pg.state();

        (redis.connections, pg.connections)
    }
}
