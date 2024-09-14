use std::sync::Arc;

use tokio_stream::StreamExt;

use crate::{
    cache::{EntityCache, ServiceCache},
    config::Database,
    entity::{Championship, Race},
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
        if let Some(championship) = self.cache.championship.get(&id) {
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

    pub async fn races(&self, id: i32) -> AppResult<Vec<Arc<Race>>> {
        if let Some(races) = self.cache.championship.get_races(&id) {
            return Ok(races);
        }

        let stream = {
            let conn = self.db.pg.get().await?;

            let championship_races_stmt = conn
                .prepare_cached(
                    r#"
                        SELECT * FROM races
                        WHERE championship_id = $1
                    "#,
                )
                .await?;

            conn.query_raw(&championship_races_stmt, &[&id]).await?
        };

        tokio::pin!(stream);
        let mut races = Vec::new();

        while let Some(row) = stream.try_next().await? {
            races.push(Race::from_row_arc(&row))
        }

        self.cache.championship.set_races(id, races.clone());

        Ok(races)
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

        tokio::pin!(stream);
        let mut users = Vec::new();

        while let Some(row) = stream.try_next().await? {
            users.push(row.get(0));
        }

        Ok(users)
    }

    pub async fn drivers_linked(&self, id: i32) -> AppResult<Vec<String>> {
        let stream = {
            let conn = self.db.pg.get().await?;

            let linked_drivers_stmt = conn
                .prepare_cached(
                    r#"
                        SELECT steam_name
                        FROM championship_drivers
                        WHERE championship_id = $1
                    "#,
                )
                .await?;

            conn.query_raw(&linked_drivers_stmt, &[&id]).await?
        };

        tokio::pin!(stream);
        let mut drivers = Vec::new();

        while let Some(row) = stream.try_next().await? {
            drivers.push(row.get(0));
        }

        Ok(drivers)
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

        tokio::pin!(stream);
        let mut championships = Vec::new();

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

        tokio::pin!(stream);
        let mut ports = Vec::new();

        while let Some(row) = stream.try_next().await? {
            ports.push(row.get(0));
        }

        Ok(ports)
    }
}
