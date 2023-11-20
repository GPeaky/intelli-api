use crate::{
    cache::{EntityCache, RedisCache},
    config::Database,
    dtos::ChampionshipCacheData,
    entity::{Championship, FromRow},
    error::{AppError, AppResult, ChampionshipError},
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

    pub async fn ports_in_use(&self) -> AppResult<Vec<i32>> {
        let rows = {
            let conn = self.database.pg.get().await?;

            conn.query(
                r#"
                    SELECT port FROM championship
                "#,
                &[],
            )
            .await?
        };

        let ports_in_use = rows.iter().map(|row| row.get("port")).collect();

        Ok(ports_in_use)
    }

    pub async fn find(&self, id: &i32) -> AppResult<Option<Championship>> {
        if let Some(championship) = self.cache.championship.get(id).await? {
            return Ok(Some(championship));
        };

        let row = {
            let conn = self.database.pg.get().await?;

            conn.query_opt(
                r#"
                    SELECT * FROM championship
                    WHERE id = $1
                "#,
                &[&id],
            )
            .await?
        };

        if let Some(row) = row {
            let championship = Championship::from_row(&row)?;

            self.cache.championship.set(&championship).await?;
            return Ok(Some(championship));
        }

        Ok(None)
    }

    // TODO: Add cache for this function
    pub async fn exist_by_name(&self, name: &str) -> AppResult<()> {
        let row = {
            let conn = self.database.pg.get().await?;

            conn.query_opt(
                r#"
                    SELECT id FROM championship
                    WHERE name = $1
                "#,
                &[&name],
            )
            .await?
        };

        if row.is_some() {
            Err(ChampionshipError::AlreadyExists)?;
        }

        Ok(())
    }

    // TODO: Move it to cache struct
    pub async fn session_data(&self, id: &i32) -> AppResult<ChampionshipCacheData> {
        let Some(_) = self.find(id).await? else {
            Err(ChampionshipError::NotFound)?
        };

        let mut redis = self.database.redis.get().await?;
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
            .await?;

        let history_data: Vec<Vec<u8>> = redis.mget(&session_history_key).await?;

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

        let rows = {
            let conn = self.database.pg.get().await?;
            conn.query(
                r#"
                    SELECT c.*
                    FROM championship c
                    JOIN user_championships uc ON c.id = uc.championship_id
                    WHERE uc.user_id = $1
                "#,
                &[user_id],
            )
            .await?
        };

        let championships = rows
            .iter()
            .map(Championship::from_row)
            .collect::<Result<Vec<Championship>, AppError>>()?;

        self.cache
            .championship
            .set_all(user_id, &championships)
            .await?;

        Ok(championships)
    }

    // TODO: Add cache for this function
    pub async fn user_champions_len(&self, user_id: &i32) -> AppResult<usize> {
        let rows = {
            let conn = self.database.pg.get().await?;

            conn.query(
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
                &[user_id],
            )
            .await?
        };

        Ok(rows.len())
    }
}
