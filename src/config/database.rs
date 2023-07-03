use dotenvy::var;
use redis::{Client, Connection};
use tracing::info;

pub struct Database {
    pub redis: Client,
}

impl Database {
    pub fn default() -> Self {
        info!("Connecting Databases...");

        Self {
            redis: Client::open(var("REDIS_URL").unwrap()).unwrap(),
        }
    }

    pub fn get_redis(&self) -> Connection {
        self.redis.get_connection().unwrap()
    }
}
