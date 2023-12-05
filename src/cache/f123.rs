use crate::{config::constants::*, error::AppResult};
use deadpool_redis::{redis::AsyncCommands, Connection};

// const EVENTS: &str = "events";

pub struct F123InsiderCache {
    redis: Connection,
    championship_id: i32,
}

#[allow(unused)]
// TODO: Implement cache with new batching method || Save events data in database (Postgres)
impl F123InsiderCache {
    pub fn new(redis: Connection, championship_id: i32) -> Self {
        Self {
            redis,
            championship_id,
        }
    }

    pub async fn set(&mut self, data: &[u8]) -> AppResult<()> {
        self.redis
            .set_ex(
                &format!("{REDIS_F123_PREFIX}:{}:cache", &self.championship_id),
                data,
                REDIS_F123_PERSISTENCE,
            )
            .await?;

        Ok(())
    }

    pub async fn prune(&mut self) -> AppResult<()> {
        self.redis
            .del(&format!(
                "{REDIS_F123_PREFIX}:{}:cache",
                &self.championship_id
            ))
            .await?;

        Ok(())
    }

    // #[inline(always)]
    // pub async fn push_event_data(&mut self, data: &[u8], string_code: &str) -> AppResult<()> {
    //     self.redis
    //         .rpush(
    //             &format!(
    //                 "{REDIS_F123_PREFIX}:{}:{EVENTS}:{string_code}",
    //                 &self.championship_id
    //             ),
    //             data,
    //         )
    //         .await?;
    //     Ok(())
    // }
}
