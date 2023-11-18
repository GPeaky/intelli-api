use super::EntityCache;
use crate::{
    config::{constants::*, Database},
    entity::Championship,
    error::{AppResult, CacheError},
};
use axum::async_trait;
use bb8_redis::redis::AsyncCommands;
use rkyv::{Deserialize, Infallible};
use std::sync::Arc;
use tracing::error;

const ID: &str = "id";
const USER_ID: &str = "user_id";

pub struct ChampionshipCache {
    db: Arc<Database>,
}

impl ChampionshipCache {
    pub fn new(db: &Arc<Database>) -> Self {
        Self { db: db.clone() }
    }

    #[allow(unused)]
    pub async fn get_all(&self, user_id: &i32) -> AppResult<Option<Vec<Championship>>> {
        let entities;

        // Drop the connection as soon as possible
        {
            let mut conn = self.db.redis.get().await?;

            entities = conn
                .get::<_, Vec<u8>>(&format!(
                    "{REDIS_CHAMPIONSHIP_PREFIX}:{USER_ID}:{}",
                    user_id
                ))
                .await?;
        }

        if entities.is_empty() {
            return Ok(None);
        }

        let archived = unsafe { rkyv::archived_root::<Vec<Championship>>(&entities) };
        let Ok(entities) = archived.deserialize(&mut Infallible) else {
            error!("Error deserializing championships from cache");
            return Err(CacheError::Deserialize)?;
        };

        Ok(Some(entities))
    }

    #[allow(unused)]
    pub async fn set_all(&self, user_id: &i32, championships: &Vec<Championship>) -> AppResult<()> {
        let Ok(bytes) = rkyv::to_bytes::<_, 256>(championships) else {
            error!("Failed to serialize championships to cache");
            Err(CacheError::Serialize)?
        };

        let mut conn = self.db.redis.get().await?;

        conn.set_ex::<&str, &[u8], ()>(
            &format!("{REDIS_CHAMPIONSHIP_PREFIX}:{USER_ID}:{}", user_id),
            &bytes[..],
            Self::EXPIRATION,
        )
        .await?;

        Ok(())
    }

    #[inline(always)]
    pub async fn delete_by_user_id(&self, user_id: &i32) -> AppResult<()> {
        let mut conn = self.db.redis.get().await?;

        conn.del::<&str, ()>(&format!(
            "{REDIS_CHAMPIONSHIP_PREFIX}:{USER_ID}:{}",
            user_id
        ))
        .await?;

        Ok(())
    }
}

#[async_trait]
impl EntityCache for ChampionshipCache {
    type Entity = Championship;

    #[inline(always)]
    async fn get(&self, id: &i32) -> AppResult<Option<Self::Entity>> {
        let entity;

        // Drop the connection as soon as possible
        {
            let mut conn = self.db.redis.get().await?;

            entity = conn
                .get::<_, Vec<u8>>(&format!("{REDIS_CHAMPIONSHIP_PREFIX}:{ID}:{id}"))
                .await?;
        }

        if entity.is_empty() {
            return Ok(None);
        }

        let archived = unsafe { rkyv::archived_root::<Self::Entity>(&entity) };

        let Ok(entity) = archived.deserialize(&mut Infallible) else {
            error!("Error deserializing championship from cache");
            return Err(CacheError::Deserialize)?;
        };

        Ok(Some(entity))
    }

    #[inline(always)]
    async fn set(&self, entity: &Self::Entity) -> AppResult<()> {
        let Ok(bytes) = rkyv::to_bytes::<_, 72>(entity) else {
            error!("Failed to serialize championship to cache");
            Err(CacheError::Serialize)?
        };

        let mut conn = self.db.redis.get().await?;

        conn.set_ex::<&str, &[u8], ()>(
            &format!("{REDIS_CHAMPIONSHIP_PREFIX}:{ID}:{}", entity.id),
            &bytes[..],
            Self::EXPIRATION,
        )
        .await?;

        Ok(())
    }

    async fn delete(&self, id: &i32) -> AppResult<()> {
        let mut conn = self.db.redis.get().await?;

        conn.del::<&str, ()>(&format!("{REDIS_CHAMPIONSHIP_PREFIX}:{ID}:{}", id))
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::error::AppResult;

    #[tokio::test]
    async fn test_get() -> AppResult<()> {
        Ok(())
    }

    #[tokio::test]
    async fn test_set() -> AppResult<()> {
        Ok(())
    }

    #[tokio::test]
    async fn test_get_all() -> AppResult<()> {
        Ok(())
    }

    #[tokio::test]
    async fn test_set_all() -> AppResult<()> {
        Ok(())
    }

    #[tokio::test]
    async fn test_delete() -> AppResult<()> {
        Ok(())
    }
}
