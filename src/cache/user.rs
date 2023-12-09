use super::EntityCache;
use crate::{
    config::{constants::*, Database},
    entity::User,
    error::{AppResult, CacheError},
};
use async_trait::async_trait;
use deadpool_redis::redis::{self, AsyncCommands};
use log::error;
use rkyv::{Deserialize, Infallible};
use std::sync::Arc;

const ID: &str = "id";
const EMAIL: &str = "email";

pub struct UserCache {
    db: Arc<Database>,
}

impl UserCache {
    pub fn new(db: &Arc<Database>) -> Self {
        Self { db: db.clone() }
    }

    #[inline(always)]
    pub async fn get_by_email(&self, email: &str) -> AppResult<Option<User>> {
        let bytes: Option<Vec<u8>> = {
            let mut conn = self.db.redis.get().await?;
            conn.get(&format!("{REDIS_USER_PREFIX}:{EMAIL}:{}", email))
                .await?
        };

        if let Some(bytes) = bytes {
            let archived = unsafe { rkyv::archived_root::<User>(&bytes) };

            let Ok(user) = archived.deserialize(&mut Infallible) else {
                error!("Failed to deserialize user from cache");
                Err(CacheError::Deserialize)?
            };

            return Ok(Some(user));
        }

        Ok(None)
    }
}

#[async_trait]
impl EntityCache for UserCache {
    type Entity = User;
    const EXPIRATION: usize = REDIS_CACHE_EXPIRATION;

    #[inline(always)]
    async fn get(&self, id: &i32) -> AppResult<Option<Self::Entity>> {
        let bytes: Option<Vec<u8>> = {
            let mut conn = self.db.redis.get().await?;
            conn.get(&format!("{REDIS_USER_PREFIX}:{ID}:{}", id))
                .await?
        };

        if let Some(bytes) = bytes {
            let archived = unsafe { rkyv::archived_root::<Self::Entity>(&bytes) };

            let Ok(user) = archived.deserialize(&mut Infallible) else {
                error!("Failed to deserialize user from cache");
                Err(CacheError::Deserialize)?
            };

            return Ok(Some(user));
        }

        Ok(None)
    }

    #[inline(always)]
    async fn set(&self, entity: &Self::Entity) -> AppResult<()> {
        let Ok(bytes) = rkyv::to_bytes::<_, 128>(entity) else {
            error!("Failed to serialize user to cache");
            Err(CacheError::Serialize)?
        };

        let mut conn = self.db.redis.get().await?;

        redis::pipe()
            .atomic()
            .set_ex(
                &format!("{REDIS_USER_PREFIX}:{ID}:{}", entity.id),
                &bytes[..],
                Self::EXPIRATION,
            )
            .set_ex(
                &format!("{REDIS_USER_PREFIX}:{EMAIL}:{}", entity.email),
                &bytes[..],
                Self::EXPIRATION,
            )
            .query_async(&mut *conn)
            .await?;

        Ok(())
    }

    #[inline(always)]
    async fn delete(&self, id: &i32) -> AppResult<()> {
        let mut conn = self.db.redis.get().await?;

        let bytes: Option<Vec<u8>> = conn
            .get_del(&format!("{REDIS_USER_PREFIX}:{ID}:{}", id))
            .await?;

        if let Some(bytes) = bytes {
            let archived = unsafe { rkyv::archived_root::<User>(&bytes) };

            let Ok(user): Result<User, std::convert::Infallible> =
                archived.deserialize(&mut Infallible)
            else {
                error!("Failed to deserialize user from cache");
                Err(CacheError::Deserialize)?
            };

            conn.del(&format!("{REDIS_USER_PREFIX}:{EMAIL}:{}", user.email))
                .await?;
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::error::AppResult;

    #[ntex::test]
    async fn test_get_by_email() -> AppResult<()> {
        Ok(())
    }

    #[ntex::test]
    async fn test_get() -> AppResult<()> {
        Ok(())
    }

    #[ntex::test]
    async fn test_set() -> AppResult<()> {
        Ok(())
    }

    #[ntex::test]
    async fn test_delete() -> AppResult<()> {
        Ok(())
    }
}
