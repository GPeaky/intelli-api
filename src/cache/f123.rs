use deadpool_redis::{
    redis::{self, AsyncCommands},
    Connection,
};

use crate::{config::constants::*, error::AppResult};

// const EVENTS: &str = "events";

/// `F123InsiderCache` is a caching structure for storing Formula 123 (F123) championship data using Redis.
/// It is designed to manage caching for a specific championship, and it provides methods to set various
/// types of data related to that championship, such as motion data, session data, participants data, events data,
/// and session history. Additionally, it offers a method to prune cached data when it is no longer needed.
///
/// Each instance of `F123InsiderCache` is intended to be associated with a single championship, and multiple
/// instances can be created to manage caching for different championships.
///
pub struct F123InsiderCache {
    redis: Connection,
    championship_id: i32,
}

#[allow(unused)]
// Todo: Implement constants for redis keys to avoid hardcoding
// TODO: Implement cache with new batching method || Save events data in database (Postgres)
impl F123InsiderCache {
    /// Creates a new `F123InsiderCache` instance with the provided Redis connection and championship ID.
    ///
    /// # Arguments
    ///
    /// * `redis` - A Redis connection instance.
    /// * `championship_id` - The ID of the championship for which data will be cached.
    ///
    /// # Returns
    ///
    /// A new `F123InsiderCache` instance.
    pub fn new(redis: Connection, championship_id: i32) -> Self {
        Self {
            redis,
            championship_id,
        }
    }

    /// Sets motion data for the championship in the Redis cache.
    ///
    /// # Arguments
    ///
    /// * `data` - A reference to the motion data to be stored.
    ///
    /// # Returns
    ///
    /// An `AppResult` indicating the success or failure of setting motion data in the cache.
    ///
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

    /// Sets session data for the championship in the Redis cache.
    ///
    /// # Arguments
    ///
    /// * `data` - A reference to the session data to be stored.
    ///
    /// # Returns
    ///
    /// An `AppResult` indicating the success or failure of setting session data in the cache.
    ///
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

    /// Sets participants data for the championship in the Redis cache.
    ///
    /// # Arguments
    ///
    /// * `data` - A reference to the participants data to be stored.
    ///
    /// # Returns
    ///
    /// An `AppResult` indicating the success or failure of setting participants data in the cache.
    ///
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

    /// Pushes event data for the championship in the Redis cache.
    ///
    /// # Arguments
    ///
    /// * `data` - A reference to the event data to be pushed.
    /// * `string_code` - A string identifier for the type of event data (e.g., "race_start").
    ///
    /// # Returns
    ///
    /// An `AppResult` indicating the success or failure of pushing event data in the cache.
    ///
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

    /// Sets session history data for the championship in the Redis cache.
    ///
    /// # Arguments
    ///
    /// * `data` - A reference to the session history data to be stored.
    /// * `car_idx` - An identifier for the car associated with the session history data.
    ///
    /// # Returns
    ///
    /// An `AppResult` indicating the success or failure of setting session history data in the cache.
    ///
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

    /// Prunes cached data associated with the championship when it is no longer needed.
    ///
    /// # Returns
    ///
    /// An `AppResult` indicating the success or failure of pruning cached data.
    ///
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
