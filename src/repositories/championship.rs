use std::sync::Arc;

use tokio_stream::StreamExt;

use crate::{
    cache::{EntityCache, ServiceCache},
    config::Database,
    entity::Championship,
    error::AppResult,
    utils::slice_iter,
};

/// Repository for managing championship data with caching support.
pub struct ChampionshipRepository {
    db: &'static Database,
    cache: &'static ServiceCache,
}

impl ChampionshipRepository {
    /// Creates a new ChampionshipRepository instance.
    ///
    /// # Arguments
    /// - `db`: Database connection.
    /// - `cache`: Service cache.
    ///
    /// # Returns
    /// A new ChampionshipRepository instance.
    pub fn new(db: &'static Database, cache: &'static ServiceCache) -> Self {
        Self { db, cache }
    }

    /// Finds a championship by its ID.
    ///
    /// # Arguments
    /// - `id`: The ID of the championship to find.
    ///
    /// # Returns
    /// An Option containing the Championship if found.
    pub async fn find(&self, id: i32) -> AppResult<Option<Arc<Championship>>> {
        if let Some(championship) = self.cache.championship.get(id) {
            return Ok(Some(championship));
        };

        let row = {
            let conn = self.db.pg.get().await?;

            let find_championship_stmt = conn
                .prepare_cached(
                    r#"
                        SELECT * FROM championships
                        WHERE id = $1
                    "#,
                )
                .await?;

            conn.query_opt(&find_championship_stmt, &[&id]).await?
        };

        match row {
            Some(ref row) => {
                let championship = Championship::from_row_arc(row);
                self.cache.championship.set(championship.clone());
                Ok(Some(championship))
            }

            None => Ok(None),
        }
    }

    /// Finds a championship by its name.
    ///
    /// # Arguments
    /// - `name`: The name of the championship to find.
    ///
    /// # Returns
    /// An Option containing the Championship if found.
    pub async fn find_by_name(&self, name: &str) -> AppResult<Option<Arc<Championship>>> {
        if let Some(championship) = self.cache.championship.get_by_name(name) {
            return Ok(Some(championship));
        };

        let row = {
            let conn = self.db.pg.get().await?;

            let find_by_name_stmt = conn
                .prepare_cached(
                    r#"
                        SELECT * FROM championships
                        WHERE name = $1
                    "#,
                )
                .await?;

            conn.query_opt(&find_by_name_stmt, &[&name]).await?
        };

        match row {
            Some(ref row) => {
                let championship = Championship::from_row_arc(row);
                self.cache.championship.set(championship.clone());
                Ok(Some(championship))
            }

            None => Ok(None),
        }
    }

    /// Retrieves user IDs associated with a championship.
    ///
    /// # Arguments
    /// - `id`: The ID of the championship.
    ///
    /// # Returns
    /// A vector of user IDs.
    pub async fn users(&self, id: i32) -> AppResult<Vec<i32>> {
        let stream = {
            let conn = self.db.pg.get().await?;

            let championship_users_stmt = conn
                .prepare_cached(
                    r#"
                        SELECT user_id FROM championship_users
                        WHERE championship_id = $1
                    "#,
                )
                .await?;

            conn.query_raw(&championship_users_stmt, &[&id]).await?
        };

        let mut users = Vec::with_capacity(stream.rows_affected().unwrap_or(0) as usize);

        tokio::pin!(stream);

        while let Some(row) = stream.try_next().await? {
            users.push(row.get(0));
        }

        Ok(users)
    }

    pub async fn is_driver_linked(&self, id: i32, steam_name: &str) -> AppResult<bool> {
        let conn = self.db.pg.get().await?;

        let driver_exists_stmt = conn
            .prepare_cached(
                r#"
                    SELECT 1
                    FROM championship_drivers
                    WHERE steam_name = $1 AND championship_id = $2
                    LIMIT 1
                "#,
            )
            .await?;

        let res = conn
            .query_opt(&driver_exists_stmt, &[&steam_name, &id])
            .await?;

        Ok(res.is_some())
    }

    /// Retrieves all used championship IDs.
    ///
    /// This method should only be called once.
    ///
    /// # Returns
    /// A vector of all used championship IDs.
    pub async fn _used_ids(&self) -> AppResult<Vec<i32>> {
        let conn = self.db.pg.get().await?;

        let championship_ids_stmt = conn
            .prepare_cached(
                r#"
                    SELECT id FROM championships
                "#,
            )
            .await?;

        let stream = conn
            .query_raw(&championship_ids_stmt, slice_iter(&[]))
            .await?;

        let mut championships = Vec::with_capacity(stream.rows_affected().unwrap_or(0) as usize);

        tokio::pin!(stream);

        while let Some(row) = stream.try_next().await? {
            championships.push(row.get(0));
        }

        Ok(championships)
    }

    /// Retrieves a list of ports currently in use by championships.
    ///
    /// # Returns
    /// A vector of port numbers in use.
    pub async fn _ports_in_use(&self) -> AppResult<Vec<i32>> {
        let stream = {
            let conn = self.db.pg.get().await?;

            let ports_in_use_stmt = conn
                .prepare_cached(
                    r#"
                        SELECT port FROM championships
                    "#,
                )
                .await?;

            conn.query_raw(&ports_in_use_stmt, slice_iter(&[])).await?
        };

        let mut ports = Vec::with_capacity(stream.rows_affected().unwrap_or(0) as usize);

        tokio::pin!(stream);

        while let Some(row) = stream.try_next().await? {
            ports.push(row.get(0));
        }

        Ok(ports)
    }
}
