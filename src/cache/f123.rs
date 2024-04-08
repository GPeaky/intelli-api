use deadpool_redis::{
    redis::{self, AsyncCommands},
    Connection,
};
use tracing::info;

use crate::{
    config::constants::*, error::AppResult, protos::packet_header::PacketType,
    structs::OptionalMessage,
};

pub const EVENTS: &str = "events";
pub const MOTION: &str = "motion";
pub const SESSION: &str = "session";
pub const PARTICIPANTS: &str = "participants";
pub const SESSION_HISTORY: &str = "session_history";

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

    // Remove inline if this is called more than once
    #[inline]
    pub async fn save(
        &mut self,
        packet_type: PacketType,
        data: &[u8],
        second_param: Option<OptionalMessage<'_>>,
    ) -> AppResult<()> {
        match packet_type {
            PacketType::CarMotion => self.set_motion_data(data).await,
            PacketType::SessionData => self.set_session_data(data).await,
            PacketType::Participants => self.set_participants_data(data).await,

            PacketType::SessionHistoryData => {
                let car_id = match second_param.unwrap() {
                    OptionalMessage::Number(car_id) => car_id,
                    _ => unreachable!(),
                };

                self.set_session_history(data, car_id).await
            }

            PacketType::EventData => {
                let string_code = match second_param.unwrap() {
                    OptionalMessage::Text(string_code) => string_code,
                    _ => unreachable!(),
                };

                self.push_event_data(data, string_code).await
            }

            PacketType::FinalClassificationData => {
                info!("Final classification data");
                self.prune().await

                // if let Err(e) = self
                //     .cache
                //     .set_final_classification_data(&data)
                //     .await
                // {
                //     warn!("F123 cache: {}", e);
                // }
            }
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
    #[inline(always)]
    async fn set_motion_data(&mut self, data: &[u8]) -> AppResult<()> {
        self.redis
            .set_ex(
                &format!("{REDIS_F123_PREFIX}:{}:{MOTION}", self.championship_id),
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
    #[inline(always)]
    async fn set_session_data(&mut self, data: &[u8]) -> AppResult<()> {
        self.redis
            .set_ex(
                &format!("{REDIS_F123_PREFIX}:{}:{SESSION}", self.championship_id),
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
    #[inline(always)]
    async fn set_participants_data(&mut self, data: &[u8]) -> AppResult<()> {
        self.redis
            .set_ex(
                &format!(
                    "{REDIS_F123_PREFIX}:{}:{PARTICIPANTS}",
                    self.championship_id
                ),
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
    #[inline(always)]
    async fn push_event_data(&mut self, data: &[u8], string_code: &str) -> AppResult<()> {
        self.redis
            .rpush(
                &format!(
                    "{REDIS_F123_PREFIX}:{}:{EVENTS}:{}",
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
    #[inline(always)]
    async fn set_session_history(&mut self, data: &[u8], car_idx: u8) -> AppResult<()> {
        self.redis
            .set_ex(
                &format!(
                    "{REDIS_F123_PREFIX}:{}:{SESSION_HISTORY}:{}",
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
    #[inline(always)]
    async fn prune(&mut self) -> AppResult<()> {
        info!("Called prune");

        let mut pipe = redis::pipe();

        pipe.del(&format!(
            "{REDIS_F123_PREFIX}:{}:{MOTION}",
            &self.championship_id
        ))
        .del(&format!(
            "{REDIS_F123_PREFIX}:{}:{SESSION}",
            &self.championship_id
        ))
        .del(&format!(
            "{REDIS_F123_PREFIX}:{}:{PARTICIPANTS}",
            &self.championship_id
        ));

        pipe.query_async(&mut self.redis).await?;

        let patters = vec![
            format!("{REDIS_F123_PREFIX}:{}:{EVENTS}:*", &self.championship_id),
            format!(
                "{REDIS_F123_PREFIX}:{}:{SESSION_HISTORY}:*",
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
