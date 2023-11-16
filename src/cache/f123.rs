use crate::{config::constants::*, error::AppResult};
use bb8_redis::redis::{aio::Connection, AsyncCommands};

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
            .set_ex::<&str, &[u8], ()>(
                &format!("{REDIS_F123_PREFIX}:{}:motion", &self.championship_id),
                data,
                REDIS_F123_PERSISTANCE,
            )
            .await?;

        Ok(())
    }

    #[inline(always)]
    pub async fn set_session_data(&mut self, data: &[u8]) -> AppResult<()> {
        self.redis
            .set_ex::<&str, &[u8], ()>(
                &format!("{REDIS_F123_PREFIX}:{}:session", &self.championship_id),
                data,
                REDIS_F123_PERSISTANCE,
            )
            .await?;

        Ok(())
    }

    #[inline(always)]
    pub async fn set_participants_data(&mut self, data: &[u8]) -> AppResult<()> {
        self.redis
            .set_ex::<&str, &[u8], ()>(
                &format!("{REDIS_F123_PREFIX}:{}:participants", &self.championship_id),
                data,
                REDIS_F123_PERSISTANCE,
            )
            .await?;

        Ok(())
    }

    #[inline(always)]
    pub async fn push_event_data(&mut self, data: &[u8], string_code: &str) -> AppResult<()> {
        self.redis
            .rpush::<&str, &[u8], ()>(
                &format!(
                    "{REDIS_F123_PREFIX}:{}:events:{string_code}",
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
            .set_ex::<&str, &[u8], ()>(
                &format!(
                    "{REDIS_F123_PREFIX}:{}:history:{car_idx}",
                    &self.championship_id
                ),
                data,
                REDIS_F123_PERSISTANCE,
            )
            .await?;

        Ok(())
    }
}
