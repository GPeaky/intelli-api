use super::EntityCache;
use crate::{
    config::Database,
    entity::Championship,
    error::{AppResult, CacheError},
};
use axum::async_trait;
use bb8_redis::redis::AsyncCommands;
use rkyv::{Deserialize, Infallible};
use std::sync::Arc;
use tracing::{error, info};

const CHAMPIONSHIP_PREFIX: &str = "championship";
const CHAMPIONSHIPS_PREFIX: &str = "championships";

pub struct ChampionshipCache {
    db: Arc<Database>,
}

impl ChampionshipCache {
    pub fn new(db: &Arc<Database>) -> Self {
        Self { db: db.clone() }
    }

    #[allow(unused)]
    pub async fn get_all(&self, user_id: &i32) -> AppResult<Option<Vec<Championship>>> {
        info!("Trying to get championships from cache");
        let entities;

        // Drop the connection as soon as possible
        {
            let mut conn = self.db.redis.get().await?;

            entities = conn
                .get::<_, Vec<u8>>(&format!("{CHAMPIONSHIPS_PREFIX}:user_id:{user_id}"))
                .await?;
        }

        info!("Found {} championships in cache", entities.len());

        if entities.is_empty() {
            info!("Loading Data bytes {:?}", entities);

            info!("No championships found in cache");
            return Ok(None);
        }

        let archived = unsafe { rkyv::archived_root::<Vec<Championship>>(&entities) };

        let Ok(entities) = archived.deserialize(&mut Infallible) else {
            error!("Error deserializing championships from cache");
            return Err(CacheError::Deserialize)?;
        };

        info!("Used championships from cache");

        Ok(Some(entities))
    }

    #[allow(unused)]
    pub async fn set_all(&self, user_id: &i32, championships: &Vec<Championship>) -> AppResult<()> {
        info!("Received Set All Championship: {:?}", championships);

        let Ok(bytes) = rkyv::to_bytes::<_, 256>(championships) else {
            error!("Failed to serialize championships to cache");
            Err(CacheError::Serialize)?
        };

        info!("Saving Data bytes {:?}", bytes);

        let mut conn = self.db.redis.get().await?;

        conn.set_ex::<&str, &[u8], ()>(
            &format!("{CHAMPIONSHIPS_PREFIX}:user_id:{}", user_id),
            &bytes[..],
            Self::EXPIRATION,
        )
        .await?;

        info!("Saved {} championships to cache", championships.len());

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
                .get::<_, Vec<u8>>(&format!("{CHAMPIONSHIP_PREFIX}:id:{id}"))
                .await?;
        }

        if entity.is_empty() {
            return Ok(None);
        }

        let archived = unsafe { rkyv::archived_root::<Self::Entity>(&entity) };

        // TODO: Check a better way ton handle this
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
            &format!("{CHAMPIONSHIP_PREFIX}:id:{}", entity.id),
            &bytes[..],
            Self::EXPIRATION,
        )
        .await?;

        Ok(())
    }
}
