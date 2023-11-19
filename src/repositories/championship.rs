use crate::{
    cache::{EntityCache, RedisCache},
    config::Database,
    dtos::ChampionshipCacheData,
    entity::Championship,
    error::{AppResult, ChampionshipError},
};
use bb8_redis::redis::{self, AsyncCommands};
use std::sync::Arc;

pub struct ChampionshipRepository {
    database: Arc<Database>,
    cache: Arc<RedisCache>,
}

impl ChampionshipRepository {
    pub async fn new(db_conn: &Arc<Database>, cache: &Arc<RedisCache>) -> Self {
        Self {
            database: db_conn.clone(),
            cache: cache.clone(),
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
        if let Some(championship) = self.cache.championship.get(id).await? {
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

        if let Some(ref championship) = championship {
            self.cache.championship.set(championship).await?;
        }

        Ok(championship)
    }

    // TODO: Add cache for this function
    pub async fn exist_by_name(&self, name: &str) -> AppResult<()> {
        let championship = sqlx::query_as::<_, (i32,)>(
            r#"
                SELECT id FROM championship
                WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.database.pg)
        .await?;

        if championship.is_some() {
            Err(ChampionshipError::AlreadyExists)?;
        }

        Ok(())
    }

    // TODO: Check if this is the best way to do this
    pub async fn session_data(&self, id: &i32) -> AppResult<ChampionshipCacheData> {
        let Some(_) = self.find(id).await? else {
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
            .get(&format!("f123:championships:{}:session", id))
            .get(&format!("f123:championships:{}:motion", id))
            .get(&format!("f123:championships:{}:participants", id))
            .keys(&format!("f123:championships:{}:history:*", id))
            .query_async(&mut *redis)
            .await
            .unwrap();

        let history_data: Vec<Vec<u8>> = redis.mget(&session_history_key).await.unwrap_or_default();

        Ok(ChampionshipCacheData {
            session_data,
            motion_data,
            participants_data,
            history_data,
            events_data: None,
        })
    }

    pub async fn find_all(&self, user_id: &i32) -> AppResult<Vec<Championship>> {
        if let Some(championships) = self.cache.championship.get_all(user_id).await? {
            return Ok(championships);
        };

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

        self.cache
            .championship
            .set_all(user_id, &championships)
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
}
