use super::EntityCache;
use crate::{
    config::{constants::*, Database},
    entity::Championship,
    error::{AppResult, CacheError},
};
use async_trait::async_trait;
use deadpool_redis::redis::{self, AsyncCommands};
use log::error;
use rkyv::{Deserialize, Infallible};

const ID: &str = "id";
const NAME: &str = "name";
const USER_ID: &str = "user_id";

#[derive(Clone)]
pub struct ChampionshipCache {
    db: Database,
}

impl ChampionshipCache {
    pub fn new(db: &Database) -> Self {
        Self { db: db.clone() }
    }

    #[allow(unused)]
    pub async fn get_all(&self, user_id: &i32) -> AppResult<Option<Vec<Championship>>> {
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

    #[allow(unused)]
    pub async fn set_all(&self, user_id: &i32, championships: &Vec<Championship>) -> AppResult<()> {
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

    #[inline(always)]
    pub async fn delete_by_user_id(&self, user_id: &i32) -> AppResult<()> {
        let mut conn = self.db.redis.get().await?;

        conn.del(&format!(
            "{REDIS_CHAMPIONSHIP_PREFIX}:{USER_ID}:{}",
            user_id
        ))
        .await?;

        Ok(())
    }

    // Todo: Test the functionality of this method
    #[inline(always)]
    pub async fn delete_all(&self, id: &i32, users: Vec<i32>) -> AppResult<()> {
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

    #[inline(always)]
    async fn get(&self, id: &i32) -> AppResult<Option<Self::Entity>> {
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

    #[inline(always)]
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

    async fn delete(&self, id: &i32) -> AppResult<()> {
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
