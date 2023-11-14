use bb8_redis::{
    bb8::{self, Pool},
    RedisConnectionManager,
};
use dotenvy::var;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::info;

pub struct Database {
    pub redis: Pool<RedisConnectionManager>,
    pub pg: PgPool,
}

impl Database {
    pub async fn default() -> Self {
        info!("Connecting Databases...");
        let pg = PgPoolOptions::new()
            .min_connections(1)
            .max_connections(100)
            .connect(&var("DATABASE_URL").expect("Environment DATABASE_URL not found"))
            .await
            .unwrap();

        let manager =
            RedisConnectionManager::new(var("REDIS_URL").expect("Environment REDIS_URL not found"))
                .unwrap();

        let redis = bb8::Pool::builder()
            .min_idle(Some(1))
            .max_size(300) // Test if 100 is a good number
            .build(manager)
            .await
            .unwrap();

        Self { redis, pg }
    }

    pub fn active_pools(&self) -> (u32, u32) {
        let redis = self.redis.state();
        let pg = self.pg.size();

        (redis.connections, pg)
    }
}
