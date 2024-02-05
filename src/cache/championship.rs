use async_trait::async_trait;
use deadpool_redis::redis::{self, AsyncCommands};
use rkyv::{Deserialize, Infallible};
use tracing::error;

use crate::{
    config::{constants::*, Database},
    entity::Championship,
    error::{AppResult, CacheError},
};

use super::EntityCache;

const ID: &str = "id";
const NAME: &str = "name";
const USER_ID: &str = "user_id";

/// `ChampionshipCache` is a caching structure for storing and retrieving championship data using Redis.
/// It provides methods to interact with a Redis cache to retrieve championships by user ID or by name,
/// as well as methods to set, delete, and manage championship data in the cache.
///
#[derive(Clone)]
pub struct ChampionshipCache {
    db: Database,
}

impl ChampionshipCache {
    /// Creates a new `ChampionshipCache` instance with the provided `Database` reference.
    ///
    /// # Arguments
    ///
    /// * `db` - A reference to the database used for caching.
    ///
    /// # Returns
    ///
    /// A new `ChampionshipCache` instance.
    pub fn new(db: &Database) -> Self {
        Self { db: db.clone() }
    }

    /// Retrieves all championships associated with a user by their user ID from the Redis cache.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID of the user whose championships you want to retrieve.
    ///
    /// # Returns
    ///
    /// An `AppResult` containing an `Option<Vec<Championship>>`. If championships for the user are
    /// found in the cache, it returns `Some(championships)`. If not found, it returns `None`.
    ///
    /// # Errors
    ///
    /// If there is an error while interacting with the Redis cache or deserializing the championship
    /// data, this function returns an `AppError` indicating the issue.
    ///
    #[allow(unused)]
    pub async fn get_all(&self, user_id: i32) -> AppResult<Option<Vec<Championship>>> {
        let entities: Option<Vec<u8>> = {
            let mut conn = self.db.redis.get().await?;

            conn.get(&format!(
                "{REDIS_CHAMPIONSHIP_PREFIX}:{USER_ID}:{}",
                user_id
            ))
            .await?
        };

        if let Some(entities) = entities {
            let archived = unsafe { rkyv::archived_root::<Vec<Championship>>(&entities) };

            let Ok(entities) = archived.deserialize(&mut Infallible) else {
                error!("Failed to deserialize championships from cache");
                Err(CacheError::Deserialize)?
            };

            return Ok(Some(entities));
        }

        Ok(None)
    }

    /// Retrieves a championship by its name from the Redis cache.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the championship you want to retrieve.
    ///
    /// # Returns
    ///
    /// An `AppResult` containing an `Option<Championship>`. If a championship with the provided name
    /// exists in the cache, it returns `Some(championship)`. If not found, it returns `None`.
    ///
    /// # Errors
    ///
    /// If there is an error while interacting with the Redis cache or deserializing the championship
    /// data, this function returns an `AppError` indicating the issue.
    ///
    #[allow(unused)]
    pub async fn get_by_name(&self, name: &str) -> AppResult<Option<Championship>> {
        let bytes: Option<Vec<u8>> = {
            let mut conn = self.db.redis.get().await?;
            conn.get(&format!("{REDIS_CHAMPIONSHIP_PREFIX}:{NAME}:{}", name))
                .await?
        };

        if let Some(bytes) = bytes {
            let archived = unsafe { rkyv::archived_root::<Championship>(&bytes) };

            let Ok(entity): Result<Championship, std::convert::Infallible> =
                archived.deserialize(&mut Infallible)
            else {
                error!("Failed to deserialize championship from cache");
                Err(CacheError::Deserialize)?
            };

            return Ok(Some(entity));
        }

        Ok(None)
    }

    /// Sets all championships associated with a user in the Redis cache.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID of the user for whom championships are being set.
    /// * `championships` - A reference to a vector of `Championship` instances to be stored in the cache.
    ///
    /// # Returns
    ///
    /// An `AppResult` indicating the success or failure of setting championships in the cache.
    ///
    /// # Errors
    ///
    /// If there is an error while serializing or interacting with the Redis cache, this function
    /// returns an `AppError` indicating the issue.
    ///
    #[allow(unused)]
    pub async fn set_all(&self, user_id: i32, championships: &Vec<Championship>) -> AppResult<()> {
        let Ok(bytes) = rkyv::to_bytes::<_, 256>(championships) else {
            error!("Failed to serialize championships to cache");
            Err(CacheError::Serialize)?
        };

        let mut conn = self.db.redis.get().await?;

        conn.set_ex(
            &format!("{REDIS_CHAMPIONSHIP_PREFIX}:{USER_ID}:{}", user_id),
            &bytes[..],
            Self::EXPIRATION,
        )
        .await?;

        Ok(())
    }

    /// Deletes all championships associated with a user from the Redis cache.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID of the user whose championships you want to delete.
    ///
    /// # Returns
    ///
    /// An `AppResult` indicating the success or failure of deleting championships from the cache.
    ///
    /// # Errors
    ///
    /// If there is an error while interacting with the Redis cache, this function returns an
    /// `AppError` indicating the issue.
    ///
    pub async fn delete_by_user_id(&self, user_id: i32) -> AppResult<()> {
        let mut conn = self.db.redis.get().await?;

        conn.del(&format!(
            "{REDIS_CHAMPIONSHIP_PREFIX}:{USER_ID}:{}",
            user_id
        ))
        .await?;

        Ok(())
    }

    /// Deletes all championships associated with a list of users from the Redis cache.
    ///
    /// # Arguments
    ///
    /// * `id` - The identifier for the set of championships (e.g., a group ID).
    /// * `users` - A vector of user IDs for whom championships are to be deleted from the cache.
    ///
    /// # Returns
    ///
    /// An `AppResult` indicating the success or failure of deleting championships from the cache.
    ///
    /// # Errors
    ///
    /// If there is an error while interacting with the Redis cache, this function returns an
    /// `AppError` indicating the issue.
    ///
    /// # Note
    ///
    /// This method requires testing to ensure its functionality.
    ///
    pub async fn delete_all(&self, id: i32, users: Vec<i32>) -> AppResult<()> {
        let mut pipe = redis::pipe();

        users.iter().for_each(|user_id| {
            pipe.del(&format!(
                "{REDIS_CHAMPIONSHIP_PREFIX}:{USER_ID}:{}",
                user_id
            ));
        });

        pipe.del(&format!("{REDIS_CHAMPIONSHIP_PREFIX}:{ID}:{}", id));

        let mut conn = self.db.redis.get().await?;
        pipe.query_async(&mut conn).await?;

        Ok(())
    }
}

#[async_trait]
impl EntityCache for ChampionshipCache {
    type Entity = Championship;
    const EXPIRATION: u64 = REDIS_CACHE_EXPIRATION;

    async fn get(&self, id: i32) -> AppResult<Option<Self::Entity>> {
        let bytes: Option<Vec<u8>> = {
            let mut conn = self.db.redis.get().await?;
            conn.get(&format!("{REDIS_CHAMPIONSHIP_PREFIX}:{ID}:{id}"))
                .await?
        };

        if let Some(bytes) = bytes {
            let archived = unsafe { rkyv::archived_root::<Self::Entity>(&bytes) };

            let Ok(entity) = archived.deserialize(&mut Infallible) else {
                error!("Error deserializing championship from cache");
                return Err(CacheError::Deserialize)?;
            };

            return Ok(Some(entity));
        }

        Ok(None)
    }

    async fn set(&self, entity: &Self::Entity) -> AppResult<()> {
        let Ok(bytes) = rkyv::to_bytes::<_, 72>(entity) else {
            error!("Failed to serialize championship to cache");
            Err(CacheError::Serialize)?
        };

        let mut conn = self.db.redis.get().await?;

        redis::pipe()
            .set_ex(
                &format!("{REDIS_CHAMPIONSHIP_PREFIX}:{ID}:{}", entity.id),
                &bytes[..],
                Self::EXPIRATION,
            )
            .set_ex(
                &format!("{REDIS_CHAMPIONSHIP_PREFIX}:{NAME}:{}", entity.name),
                &bytes[..],
                Self::EXPIRATION,
            )
            .query_async(&mut conn)
            .await?;

        Ok(())
    }

    async fn delete(&self, id: i32) -> AppResult<()> {
        let mut conn = self.db.redis.get().await?;

        let bytes: Option<Vec<u8>> = conn
            .get_del(&format!("{REDIS_CHAMPIONSHIP_PREFIX}:{ID}:{}", id))
            .await?;

        if let Some(bytes) = bytes {
            let archived = unsafe { rkyv::archived_root::<Self::Entity>(&bytes) };

            let Ok(entity): Result<Championship, std::convert::Infallible> =
                archived.deserialize(&mut Infallible)
            else {
                error!("Failed to deserialize championship from cache");
                Err(CacheError::Deserialize)?
            };

            conn.del(&format!(
                "{REDIS_CHAMPIONSHIP_PREFIX}:{NAME}:{}",
                entity.name
            ))
            .await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::error::AppResult;

    #[ntex::test]
    async fn test_get() -> AppResult<()> {
        Ok(())
    }

    #[ntex::test]
    async fn test_set() -> AppResult<()> {
        Ok(())
    }

    #[ntex::test]
    async fn test_get_all() -> AppResult<()> {
        Ok(())
    }

    #[ntex::test]
    async fn test_set_all() -> AppResult<()> {
        Ok(())
    }

    #[ntex::test]
    async fn test_delete() -> AppResult<()> {
        Ok(())
    }
}
