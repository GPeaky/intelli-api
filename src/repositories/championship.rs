use crate::{
    config::Database,
    dtos::ChampionshipCacheData,
    entity::Championship,
    error::{AppResult, ChampionshipError},
};
use bb8_redis::redis::{self, AsyncCommands};
use rkyv::{Deserialize, Infallible};
use std::sync::Arc;
use tracing::{error, info};

pub struct ChampionshipRepository {
    database: Arc<Database>,
}

impl ChampionshipRepository {
    pub async fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            database: db_conn.clone(),
        }
    }

    pub async fn ports_in_use(&self) -> AppResult<Vec<(i32,)>> {
        let ports_in_use = sqlx::query_as::<_, (i32,)>(
            r#"
                SELECT port FROM championship
            "#,
        )
        .fetch_all(&self.database.pg)
        .await?;

        Ok(ports_in_use)
    }

    pub async fn find(&self, id: &i32) -> AppResult<Option<Championship>> {
        if let Some(championship) = self.get_from_cache(id).await? {
            return Ok(Some(championship));
        };

        let championship = sqlx::query_as::<_, Championship>(
            r#"
                SELECT * FROM championship
                WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.database.pg)
        .await?;

        // TODO: Check if handle it in a better way
        if let Some(ref championship) = championship {
            self.set_to_cache(championship).await?;
        }

        Ok(championship)
    }

    // TODO: Check if this is the best way to do this
    pub async fn session_data(&self, id: &i32) -> AppResult<ChampionshipCacheData> {
        let Some(championship) = self.find(id).await? else {
            Err(ChampionshipError::NotFound)?
        };

        let mut redis = self.database.redis.get().await.unwrap();
        let (session_data, motion_data, participants_data, session_history_key): (
            Vec<u8>,
            Vec<u8>,
            Vec<u8>,
            Vec<String>,
        ) = redis::pipe()
            .atomic()
            .get(&format!(
                "f123_service:championships:{}:session_data",
                championship.id
            ))
            .get(&format!(
                "f123_service:championships:{}:motion_data",
                championship.id
            ))
            .get(&format!(
                "f123_service:championships:{}:participants_data",
                championship.id
            ))
            .keys(&format!(
                "f123_service:championships:{}:session_history:*",
                championship.id
            ))
            .query_async(&mut *redis)
            .await
            .unwrap();

        let history_data: Vec<Vec<u8>> = redis.mget(&session_history_key).await.unwrap_or_default();

        Ok(ChampionshipCacheData {
            session_data,
            motion_data,
            participants_data,
            history_data,
        })
    }

    // TODO: Add cache for this function
    pub async fn find_all(&self, user_id: &i32) -> AppResult<Vec<Championship>> {
        let championships = sqlx::query_as::<_, Championship>(
            r#"
                SELECT
                    c.*
                FROM
                    championship c
                JOIN
                    user_championships uc ON c.id = uc.championship_id
                WHERE
                    uc.user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.database.pg)
        .await?;

        Ok(championships)
    }

    // TODO: Add cache for this function
    pub async fn user_champions_len(&self, user_id: &i32) -> AppResult<usize> {
        let championships = sqlx::query_as::<_, (i32,)>(
            r#"
                SELECT
                    c.id
                FROM
                    championship c
                JOIN
                    user_championships uc ON c.id = uc.championship_id
                WHERE
                    uc.user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.database.pg)
        .await?;

        Ok(championships.len())
    }

    // TODO: Move this to a trait and implement it for all repositories
    async fn get_from_cache(&self, id: &i32) -> AppResult<Option<Championship>> {
        let mut conn = self.database.redis.get().await.unwrap();
        let championship = conn
            .get::<_, Vec<u8>>(&format!("championship:{}", id))
            .await;

        match championship {
            Ok(championship) if !championship.is_empty() => {
                let archived = unsafe { rkyv::archived_root::<Championship>(&championship) };

                let Ok(championship) = archived.deserialize(&mut Infallible) else {
                    error!("Failed to deserialize championship from cache");
                    return Ok(None);
                };

                Ok(Some(championship))
            }

            Ok(_) => Ok(None),
            Err(_) => Ok(None),
        }
    }

    async fn set_to_cache(&self, championship: &Championship) -> AppResult<()> {
        let mut conn = self.database.redis.get().await.unwrap();
        let bytes = rkyv::to_bytes::<_, 72>(championship).unwrap();

        conn.set_ex::<&str, &[u8], ()>(
            &format!("championship:{}", championship.id),
            &bytes[..],
            60 * 60,
        )
        .await
        .unwrap();

        Ok(())
    }
}
