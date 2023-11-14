use super::EntityCache;
use crate::{config::Database, entity::Championship, error::AppResult};
use axum::async_trait;
use bb8_redis::redis::AsyncCommands;
use rkyv::{Deserialize, Infallible};
use std::sync::Arc;
use tracing::error;

const CHAMPIONSHIP_PREFIX: &str = "championship";

pub struct ChampionshipCache {
    db: Arc<Database>,
}

impl ChampionshipCache {
    pub fn new(db: &Arc<Database>) -> Self {
        Self { db: db.clone() }
    }
}

#[async_trait]
impl EntityCache for ChampionshipCache {
    type Entity = Championship;

    #[inline(always)]
    async fn get(&self, id: &i32) -> AppResult<Option<Self::Entity>> {
        let mut conn = self.db.redis.get().await.unwrap();

        let Ok(entity) = conn
            .get::<_, Vec<u8>>(&format!("{CHAMPIONSHIP_PREFIX}:{id}"))
            .await
        else {
            error!("Error getting championship from cache");
            return Ok(None);
        };

        if entity.is_empty() {
            return Ok(None);
        }

        let archived = unsafe { rkyv::archived_root::<Self::Entity>(&entity) };

        let Ok(entity) = archived.deserialize(&mut Infallible) else {
            error!("Error deserializing championship from cache");
            return Ok(None);
        };

        Ok(Some(entity))
    }

    #[inline(always)]
    async fn set(&self, entity: &Self::Entity) -> AppResult<()> {
        let mut conn = self.db.redis.get().await.unwrap();
        let bytes = rkyv::to_bytes::<_, 128>(entity).unwrap();

        conn.set_ex::<&str, &[u8], ()>(
            &format!("{CHAMPIONSHIP_PREFIX}:{}", entity.id),
            &bytes[..],
            Self::EXPIRATION,
        )
        .await
        .unwrap();

        Ok(())
    }
}
