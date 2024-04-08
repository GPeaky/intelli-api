use deadpool_redis::redis;
use tokio::time::Instant;
use tracing::info;

use crate::{
    cache::{EVENTS, MOTION, PARTICIPANTS, SESSION, SESSION_HISTORY},
    config::{constants::*, Database},
    error::AppResult,
    structs::{F123CachedData, F123GeneralCachedData},
};

#[derive(Clone)]
pub struct F123Repository {
    db: &'static Database,
}

impl F123Repository {
    pub fn new(db: &'static Database) -> Self {
        Self { db }
    }

    // Todo - finish this integration and try to optimize it :) 2ms is too much
    // Todo - implement mini cache in memory for last data cached (Interval 3 seconds)
    pub async fn get_cache_data(&self, id: i32) -> AppResult<F123CachedData> {
        let time = Instant::now();
        let general_data = self.general_data(id).await?;

        let (events, session_history) = tokio::try_join!(
            self.events_data(general_data.event_keys),
            self.session_history_data(general_data.session_history_keys)
        )?;

        let cached_data = F123CachedData {
            motion: general_data.motion,
            session: general_data.session,
            participants: general_data.participants,
            session_history,
            events,
        };

        let time = time.elapsed();
        info!("Cached data: {:?}", cached_data);
        info!("Time elapsed: {:?}", time);

        Ok(cached_data)
    }

    #[inline(always)]
    async fn general_data(&self, id: i32) -> AppResult<F123GeneralCachedData> {
        let mut pipe = redis::pipe();
        let mut conn = self.db.redis.get().await?;

        let (motion, session, participants, event_keys, session_history_keys): (
            Option<Vec<u8>>,
            Option<Vec<u8>>,
            Option<Vec<u8>>,
            Option<Vec<String>>,
            Option<Vec<String>>,
        ) = pipe
            .get(&format!("{REDIS_F123_PREFIX}:{id}:{MOTION}"))
            .get(&format!("{REDIS_F123_PREFIX}:{id}:{SESSION}"))
            .get(&format!("{REDIS_F123_PREFIX}:{id}:{PARTICIPANTS}"))
            .keys(&format!("{REDIS_F123_PREFIX}:{id}:{EVENTS}:*"))
            .keys(&format!("{REDIS_F123_PREFIX}:{id}:{SESSION_HISTORY}:*"))
            .query_async(&mut conn)
            .await?;

        Ok(F123GeneralCachedData {
            motion,
            session,
            participants,
            event_keys,
            session_history_keys,
        })
    }

    #[inline(always)]
    async fn events_data(&self, keys: Option<Vec<String>>) -> AppResult<Option<Vec<Vec<Vec<u8>>>>> {
        match keys {
            None => Ok(None),
            Some(keys) => {
                let mut pipe = redis::pipe();

                for key in &keys {
                    pipe.lrange(key, 0, -1);
                }

                let mut conn = self.db.redis.get().await?;
                Ok(pipe.query_async(&mut conn).await?)
            }
        }
    }

    #[inline(always)]
    async fn session_history_data(
        &self,
        keys: Option<Vec<String>>,
    ) -> AppResult<Option<Vec<Vec<u8>>>> {
        match keys {
            None => Ok(None),
            Some(keys) => {
                let mut pipe = redis::pipe();

                for key in &keys {
                    pipe.get(key);
                }

                let mut conn = self.db.redis.get().await?;
                Ok(pipe.query_async(&mut conn).await?)
            }
        }
    }
}
