use deadpool_redis::{
    redis::{self, AsyncCommands},
    Connection,
};

use crate::{config::constants::*, error::AppResult};

// const EVENTS: &str = "events";

pub struct F123InsiderCache {
    redis: Connection,
    championship_id: i32,
}

#[allow(unused)]
// Todo: Implement constants for redis keys to avoid hardcoding
// TODO: Implement cache with new batching method || Save events data in database (Postgres)
impl F123InsiderCache {
    pub fn new(redis: Connection, championship_id: i32) -> Self {
        Self {
            redis,
            championship_id,
        }
    }

    pub async fn set_motion_data(&mut self, data: &[u8]) -> AppResult<()> {
        self.redis
            .set_ex(
                &format!("{REDIS_F123_PREFIX}:{}:motion", self.championship_id),
                data,
                REDIS_F123_PERSISTENCE,
            )
            .await?;

        Ok(())
    }

    pub async fn set_session_data(&mut self, data: &[u8]) -> AppResult<()> {
        self.redis
            .set_ex(
                &format!("{REDIS_F123_PREFIX}:{}:session", self.championship_id),
                data,
                REDIS_F123_PERSISTENCE,
            )
            .await?;

        Ok(())
    }

    pub async fn set_participants_data(&mut self, data: &[u8]) -> AppResult<()> {
        self.redis
            .set_ex(
                &format!("{REDIS_F123_PREFIX}:{}:participants", self.championship_id),
                data,
                REDIS_F123_PERSISTENCE,
            )
            .await?;

        Ok(())
    }

    pub async fn push_event_data(&mut self, data: &[u8], string_code: &str) -> AppResult<()> {
        self.redis
            .rpush(
                &format!(
                    "{REDIS_F123_PREFIX}:{}:events:{}",
                    &self.championship_id, string_code
                ),
                data,
            )
            .await?;
        Ok(())
    }

    pub async fn set_session_history(&mut self, data: &[u8], car_idx: u8) -> AppResult<()> {
        self.redis
            .set_ex(
                &format!(
                    "{REDIS_F123_PREFIX}:{}:session_history:{}",
                    self.championship_id, car_idx
                ),
                data,
                REDIS_F123_PERSISTENCE,
            )
            .await?;

        Ok(())
    }

    pub async fn prune(&mut self) -> AppResult<()> {
        let mut pipe = redis::pipe();

        pipe.del(&format!(
            "{REDIS_F123_PREFIX}:{}:motion",
            &self.championship_id
        ))
        .del(&format!(
            "{REDIS_F123_PREFIX}:{}:session",
            &self.championship_id
        ))
        .del(&format!(
            "{REDIS_F123_PREFIX}:{}:participants",
            &self.championship_id
        ));

        pipe.query_async(&mut self.redis).await?;

        let patters = vec![
            format!("{REDIS_F123_PREFIX}:{}:events:*", &self.championship_id),
            format!(
                "{REDIS_F123_PREFIX}:{}:session_history:*",
                &self.championship_id
            ),
        ];

        for pattern in patters {
            let keys: Vec<String> = self.redis.keys(pattern).await?;

            if !keys.is_empty() {
                self.redis.del(keys).await?;
            }
        }

        Ok(())
    }
}
