use crate::{config::constants::*, error::AppResult};
use deadpool_redis::{redis::AsyncCommands, Connection};

const MOTION: &str = "motion";
const SESSION: &str = "session";
const PARTICIPANTS: &str = "participants";
const EVENTS: &str = "events";
const HISTORY: &str = "history";

pub struct F123InsiderCache {
    redis: Connection,
    championship_id: i32,
}

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
                &format!("{REDIS_F123_PREFIX}:{}:{MOTION}", &self.championship_id),
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
                &format!("{REDIS_F123_PREFIX}:{}:{SESSION}", &self.championship_id),
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
                &format!(
                    "{REDIS_F123_PREFIX}:{}:{PARTICIPANTS}",
                    &self.championship_id
                ),
                data,
                REDIS_F123_PERSISTENCE,
            )
            .await?;

        Ok(())
    }

    #[inline(always)]
    pub async fn push_event_data(&mut self, data: &[u8], string_code: &str) -> AppResult<()> {
        self.redis
            .rpush(
                &format!(
                    "{REDIS_F123_PREFIX}:{}:{EVENTS}:{string_code}",
                    &self.championship_id
                ),
                data,
            )
            .await?;

        Ok(())
    }

    #[inline(always)]
    pub async fn set_session_history(&mut self, data: &[u8], car_idx: &u8) -> AppResult<()> {
        self.redis
            .set_ex(
                &format!(
                    "{REDIS_F123_PREFIX}:{}:{HISTORY}:{car_idx}",
                    &self.championship_id
                ),
                data,
                REDIS_F123_PERSISTENCE,
            )
            .await?;

        Ok(())
    }
}
