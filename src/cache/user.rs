use super::EntityCache;
use crate::{
    config::{constants::*, Database},
    entity::User,
    error::{AppResult, CacheError},
};
use axum::async_trait;
use bb8_redis::redis::{self, AsyncCommands};
use rkyv::{Deserialize, Infallible};
use std::sync::Arc;
use tracing::error;

pub struct UserCache {
    db: Arc<Database>,
}

impl UserCache {
    pub fn new(db: &Arc<Database>) -> Self {
        Self { db: db.clone() }
    }

    #[inline(always)]
    pub async fn get_by_email(&self, email: &str) -> AppResult<Option<User>> {
        let user;

        // Drop the connection as soon as possible
        {
            let mut conn = self.db.redis.get().await?;
            user = conn
                .get::<_, Vec<u8>>(&format!("{REDIS_USER_PREFIX}:email:{}", email))
                .await?;
        }

        if user.is_empty() {
            return Ok(None);
        }

        let archived = unsafe { rkyv::archived_root::<User>(&user) };

        let Ok(user) = archived.deserialize(&mut Infallible) else {
            error!("Failed to deserialize user from cache");
            Err(CacheError::Deserialize)?
        };

        Ok(Some(user))
    }

    #[inline(always)]
    pub async fn delete(&self, id: &i32) -> AppResult<()> {
        let mut conn = self.db.redis.get().await?;

        let bytes = conn
            .get_del::<_, Vec<u8>>(&format!("{REDIS_USER_PREFIX}:id:{}", id))
            .await?;

        if bytes.is_empty() {
            let archived = unsafe { rkyv::archived_root::<User>(&bytes) };
            // TODO: Check a better way ton handle this
            let user: User = archived.deserialize(&mut Infallible).unwrap();

            conn.del(&format!("{REDIS_USER_PREFIX}:email:{}", user.email))
                .await?;
        }

        Ok(())
    }
}

#[async_trait]
impl EntityCache for UserCache {
    type Entity = User;

    #[inline(always)]
    async fn get(&self, id: &i32) -> AppResult<Option<Self::Entity>> {
        let user;

        // Drop the connection as soon as possible
        {
            let mut conn = self.db.redis.get().await?;
            user = conn
                .get::<_, Vec<u8>>(&format!("{REDIS_USER_PREFIX}:id:{}", id))
                .await?;
        }

        if user.is_empty() {
            return Ok(None);
        }

        let archived = unsafe { rkyv::archived_root::<Self::Entity>(&user) };

        // TODO: Check a better way ton handle this
        let Ok(user) = archived.deserialize(&mut Infallible) else {
            error!("Failed to deserialize user from cache");
            Err(CacheError::Deserialize)?
        };

        Ok(Some(user))
    }

    #[inline(always)]
    async fn set(&self, entity: &Self::Entity) -> AppResult<()> {
        // TODO: Check a better way ton handle this
        let Ok(bytes) = rkyv::to_bytes::<_, 128>(entity) else {
            error!("Failed to serialize user to cache");
            Err(CacheError::Serialize)?
        };

        let mut conn = self.db.redis.get().await?;

        let _ = redis::pipe()
            .atomic()
            .set_ex::<&str, &[u8]>(
                &format!("{REDIS_USER_PREFIX}:id:{}", entity.id),
                &bytes[..],
                Self::EXPIRATION,
            )
            .set_ex::<&str, &[u8]>(
                &format!("{REDIS_USER_PREFIX}:email:{}", entity.email),
                &bytes[..],
                Self::EXPIRATION,
            )
            .query_async(&mut *conn)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::error::AppResult;

    #[tokio::test]
    async fn test_get_by_email() -> AppResult<()> {
        Ok(())
    }

    #[tokio::test]
    async fn test_get() -> AppResult<()> {
        Ok(())
    }

    #[tokio::test]
    async fn test_set() -> AppResult<()> {
        Ok(())
    }

    #[tokio::test]
    async fn test_delete() -> AppResult<()> {
        Ok(())
    }
}
