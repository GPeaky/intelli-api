use crate::{
    cache::{EntityCache, RedisCache},
    config::Database,
    entity::{Championship, FromRow},
    error::{AppError, AppResult, ChampionshipError},
};

#[derive(Clone)]
pub struct ChampionshipRepository {
    database: Database,
    cache: RedisCache,
}

impl ChampionshipRepository {
    pub async fn new(db_conn: &Database, cache: &RedisCache) -> Self {
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

    pub async fn exist_by_name(&self, name: &str) -> AppResult<()> {
        if self.cache.championship.get_by_name(name).await?.is_some() {
            Err(ChampionshipError::AlreadyExists)?;
        };

        let row = {
            let conn = self.database.pg.get().await?;

            let cached_statement = conn
                .prepare_cached(
                    r#"
                        SELECT * FROM championship
                        WHERE name = $1
                    "#,
                )
                .await?;

            conn.query_opt(&cached_statement, &[&name]).await?
        };

        if row.is_some() {
            let championship = Championship::from_row(&row.unwrap())?;
            self.cache.championship.set(&championship).await?;
            Err(ChampionshipError::AlreadyExists)?;
        }

        Ok(())
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
