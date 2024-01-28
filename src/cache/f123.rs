use deadpool_redis::{redis::AsyncCommands, Connection};

use crate::{config::constants::*, error::AppResult};

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

    #[inline(always)]
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

    #[inline(always)]
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

    #[inline(always)]
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

    #[inline(always)]
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

    // pub async fn prune(&mut self) -> AppResult<()> {
    //     self.redis
    //         .del(&format!(
    //             "{REDIS_F123_PREFIX}:{}:cache",
    //             &self.championship_id
    //         ))
    //         .await?;

    //     Ok(())
    // }
}
