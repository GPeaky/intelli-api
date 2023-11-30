use crate::{
    cache::{EntityCache, RedisCache},
    config::Database,
    dtos::ChampionshipCacheData,
    entity::{Championship, FromRow},
    error::{AppError, AppResult, ChampionshipError},
};
use deadpool_redis::redis::{self, AsyncCommands};
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

            let cached_statement = conn
                .prepare_cached(
                    r#"
                    SELECT port FROM championship
                "#,
                )
                .await?;

            conn.query(&cached_statement, &[]).await?
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

            let cached_statement = conn
                .prepare_cached(
                    r#"
                    SELECT * FROM championship
                    WHERE id = $1
                "#,
                )
                .await?;

            conn.query_opt(&cached_statement, &[id]).await?
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

            let cached_statement = conn
                .prepare_cached(
                    r#"
                    SELECT id FROM championship
                    WHERE name = $1
                "#,
                )
                .await?;

            conn.query_opt(&cached_statement, &[&name]).await?
        };

        if row.is_some() {
            Err(ChampionshipError::AlreadyExists)?;
        }

        Ok(())
    }

    // TODO: Move it to cache struct
    #[allow(unused)]
    pub async fn session_data(&self, id: &i32) -> AppResult<ChampionshipCacheData> {
        let Some(_) = self.find(id).await? else {
            Err(ChampionshipError::NotFound)?
        };

        let mut redis = self.database.redis.get().await?;
        let (session_data, motion_data, participants_data, history_keys): (
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

        let history_data = {
            if history_keys.is_empty() {
                None
            } else {
                let history_data = redis.mget(&history_keys).await?;
                Some(history_data)
            }
        };

        Ok(ChampionshipCacheData {
            session_data,
            motion_data,
            participants_data,
            history_data,
        })
    }

    pub async fn find_all(&self, user_id: &i32) -> AppResult<Vec<Championship>> {
        if let Some(championships) = self.cache.championship.get_all(user_id).await? {
            return Ok(championships);
        };

        let rows = {
            let conn = self.database.pg.get().await?;

            let cached_statement = conn
                .prepare_cached(
                    r#"
                        SELECT c.*
                        FROM championship c
                        JOIN user_championships uc ON c.id = uc.championship_id
                        WHERE uc.user_id = $1
                    "#,
                )
                .await?;

            conn.query(&cached_statement, &[user_id]).await?
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

            let cached_statement = conn
                .prepare_cached(
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
                .await?;

            conn.query(&cached_statement, &[user_id]).await?
        };

        Ok(rows.len())
    }
}
