use deadpool_redis::redis::AsyncCommands;

use crate::{config::Database, error::AppResult};

#[derive(Clone)]
pub struct F123Repository {
    #[allow(unused)]
    database: Database,
}

impl F123Repository {
    pub fn new(db_conn: &Database) -> Self {
        Self {
            database: db_conn.clone(),
        }
    }

    // Todo: implement mini cache in memory for last data cached (Interval 3 seconds)
    pub async fn get_cache_data(&self, id: &i32) -> AppResult<Option<Vec<u8>>> {
        let mut conn = self.database.redis.get().await?;
        let data: Option<Vec<u8>> = conn.get(&format!("{REDIS_F123_PREFIX}:{id}:cache")).await?;

        Ok(data)
    }

    #[allow(unused)]
    pub async fn events_data(&self, id: i64) -> AppResult<()> {
        todo!()
    }
}
