use crate::config::constants::*;
use deadpool_redis::redis::{self, AsyncCommands};
// use tokio::time::Instant;
use tracing::info;

use crate::{config::Database, error::AppResult, structs::F123CachedData};

#[derive(Clone)]
pub struct F123Repository {
    #[allow(unused)]
    db: &'static Database,
}

impl F123Repository {
    pub fn new(db: &'static Database) -> Self {
        Self { db }
    }

    // Todo: finish this integration and try to optimize it :) 2ms is too much
    // Todo: implement mini cache in memory for last data cached (Interval 3 seconds)
    #[allow(unused)]
    pub async fn get_cache_data(&self, id: i32) -> AppResult<F123CachedData> {
        // let time = Instant::now();
        let mut conn = self.db.redis.get().await?;

        // Todo: create a struct for this data
        #[allow(clippy::type_complexity)]
        let (motion, session, participants, events_keys, session_history_key): (
            Vec<u8>,
            Vec<u8>,
            Vec<u8>,
            Vec<String>,
            Vec<String>,
        ) = redis::pipe()
            .atomic()
            .get(&format!("{REDIS_F123_PREFIX}:{id}:motion"))
            .get(&format!("{REDIS_F123_PREFIX}:{id}:session"))
            .get(&format!("{REDIS_F123_PREFIX}:{id}:participants"))
            .keys(&format!("{REDIS_F123_PREFIX}:{id}:events:*"))
            .keys(&format!("{REDIS_F123_PREFIX}:{id}:session_history:*"))
            .query_async(&mut conn)
            .await
            .unwrap();

        let session_history: Vec<Vec<u8>> = conn.get(&session_history_key).await?;

        let mut pipe = redis::pipe();

        for key in events_keys {
            pipe.lrange(&key, 0, -1);
        }

        let events: Vec<Vec<Vec<u8>>> = pipe.query_async(&mut conn).await.unwrap();

        let cached_data = F123CachedData {
            motion,
            session,
            participants,
            events,
            session_history,
        };

        // info!(
        //     "F123Repository::get_cache_data took {}ms",
        //     time.elapsed().as_millis()
        // );

        info!("F123Repository::get_cache_data {:?}", cached_data);

        Ok(cached_data)
    }

    #[allow(unused)]
    pub async fn events_data(&self, id: i64) -> AppResult<()> {
        todo!()
    }
}
