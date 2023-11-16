use crate::error::AppResult;
use bb8_redis::redis::{aio::Connection, AsyncCommands};

const DATA_PERSISTENCE: usize = 15 * 60;
const BASE_REDIS_KEY: &str = "f123_service:championships";

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
                &format!("{BASE_REDIS_KEY}:{}:motion", &self.championship_id),
                data,
                DATA_PERSISTENCE,
            )
            .await?;

        Ok(())
    }

    #[inline(always)]
    pub async fn set_session_data(&mut self, data: &[u8]) -> AppResult<()> {
        self.redis
            .set_ex::<&str, &[u8], ()>(
                &format!("{BASE_REDIS_KEY}:{}:session", &self.championship_id),
                data,
                DATA_PERSISTENCE,
            )
            .await?;

        Ok(())
    }

    #[inline(always)]
    pub async fn set_participants_data(&mut self, data: &[u8]) -> AppResult<()> {
        self.redis
            .set_ex::<&str, &[u8], ()>(
                &format!("{BASE_REDIS_KEY}:{}:participants", &self.championship_id),
                data,
                DATA_PERSISTENCE,
            )
            .await?;

        Ok(())
    }

    #[inline(always)]
    pub async fn push_event_data(&mut self, data: &[u8], string_code: &str) -> AppResult<()> {
        self.redis
            .rpush::<&str, &[u8], ()>(
                &format!(
                    "{BASE_REDIS_KEY}:{}:events:{string_code}",
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
                    "{BASE_REDIS_KEY}:{}:history:{car_idx}",
                    &self.championship_id
                ),
                data,
                DATA_PERSISTENCE,
            )
            .await?;

        Ok(())
    }
}
