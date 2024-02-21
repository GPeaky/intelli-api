use ahash::AHashSet;
use tokio_postgres::Row;

use crate::{
    cache::{EntityCache, RedisCache},
    config::Database,
    entity::{Championship, FromRow},
    error::{AppError, AppResult},
    utils::UsedIds,
};

/// A repository for managing championship data, with support for caching.
///
/// This struct provides an interface to interact with championship data stored in both a
/// db and a cache layer. It abstracts away the details of querying and caching, offering
/// methods to retrieve and manage championship information efficiently.
pub struct ChampionshipRepository {
    /// The db connection used for querying championship data.
    db: &'static Database,
    /// The cache layer used for storing and retrieving cached championship data.
    cache: &'static RedisCache,
}

impl UsedIds for &'static ChampionshipRepository {
    async fn used_ids(&self) -> AppResult<AHashSet<i32>> {
        let conn = self.db.pg.get().await?;

        let championship_ids_stmt = conn
            .prepare_cached(
                r#"
                    SELECT id FROM championship
                "#,
            )
            .await?;

        let rows = conn.query(&championship_ids_stmt, &[]).await?;
        let mut championships_ids = AHashSet::with_capacity(rows.len());

        for row in rows {
            let id: i32 = row.get("id");
            championships_ids.insert(id);
        }

        Ok(championships_ids)
    }
}

impl ChampionshipRepository {
    /// Creates a new instance of `ChampionshipRepository`.
    ///
    /// # Arguments
    ///
    /// * `db` - A reference to the db connection.
    /// * `cache` - A reference to the cache layer.
    ///
    /// # Returns
    ///
    /// A new `ChampionshipRepository` instance.
    pub fn new(db: &'static Database, cache: &'static RedisCache) -> Self {
        Self { db, cache }
    }

    /// Retrieves a list of ports currently in use by championships.
    ///
    /// # Returns
    ///
    /// A vector of integers representing the ports in use.
    pub async fn ports_in_use(&self) -> AppResult<Vec<i32>> {
        let rows = {
            let conn = self.db.pg.get().await?;

            let ports_in_use_stmt = conn
                .prepare_cached(
                    r#"
                        SELECT port FROM championship
                    "#,
                )
                .await?;

            conn.query(&ports_in_use_stmt, &[]).await?
        };

        let ports_in_use = rows.iter().map(|row| row.get("port")).collect();

        Ok(ports_in_use)
    }

    /// Finds a championship by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the championship to find.
    ///
    /// # Returns
    ///
    /// An optional `Championship` instance if found.
    pub async fn find(&self, id: i32) -> AppResult<Option<Championship>> {
        if let Some(championship) = self.cache.championship.get(id).await? {
            return Ok(Some(championship));
        };

        let row = {
            let conn = self.db.pg.get().await?;

            let find_championship_stmt = conn
                .prepare_cached(
                    r#"
                        SELECT * FROM championship
                        WHERE id = $1
                    "#,
                )
                .await?;

            conn.query_opt(&find_championship_stmt, &[&id]).await?
        };

        self.convert_to_championship(row).await
    }

    /// Finds a championship by its name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the championship to find.
    ///
    /// # Returns
    ///
    /// An optional `Championship` instance if found.
    pub async fn find_by_name(&self, name: &str) -> AppResult<Option<Championship>> {
        if let Some(championship) = self.cache.championship.get_by_name(name).await? {
            return Ok(Some(championship));
        };

        let row = {
            let conn = self.db.pg.get().await?;

            let find_by_name_stmt = conn
                .prepare_cached(
                    r#"
                        SELECT * FROM championship
                        WHERE name = $1
                    "#,
                )
                .await?;

            conn.query_opt(&find_by_name_stmt, &[&name]).await?
        };

        self.convert_to_championship(row).await
    }

    /// Retrieves all championships associated with a user ID.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The ID of the user.
    ///
    /// # Returns
    ///
    /// A vector of `Championship` instances associated with the user.
    pub async fn find_all(&self, user_id: i32) -> AppResult<Vec<Championship>> {
        if let Some(championships) = self.cache.championship.get_all(user_id).await? {
            return Ok(championships);
        };

        let rows = {
            let conn = self.db.pg.get().await?;

            let find_all_stmt = conn
                .prepare_cached(
                    r#"
                        SELECT c.*
                        FROM championship c
                        JOIN user_championships uc ON c.id = uc.championship_id
                        WHERE uc.user_id = $1
                    "#,
                )
                .await?;

            conn.query(&find_all_stmt, &[&user_id]).await?
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

    /// Retrieves a list of user IDs associated with a championship ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the championship.
    ///
    /// # Returns
    ///
    /// A vector of integers representing the user IDs.
    pub async fn users(&self, id: i32) -> AppResult<Vec<i32>> {
        let rows = {
            let conn = self.db.pg.get().await?;

            let championship_users_stmt = conn
                .prepare_cached(
                    r#"
                        SELECT user_id FROM user_championships
                        WHERE championship_id = $1
                    "#,
                )
                .await?;

            conn.query(&championship_users_stmt, &[&id]).await?
        };

        let users = rows.iter().map(|row| row.get("user_id")).collect();

        Ok(users)
    }

    /// Counts the number of championships associated with a user ID.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The ID of the user.
    ///
    /// # Returns
    ///
    /// The number of championships associated with the user.
    pub async fn championship_len(&self, user_id: i32) -> AppResult<usize> {
        let rows = {
            let conn = self.db.pg.get().await?;

            let championship_len_stmt = conn
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

            conn.query(&championship_len_stmt, &[&user_id]).await?
        };

        Ok(rows.len())
    }

    /// Converts a db row to a `Championship` instance and caches it.
    ///
    /// # Arguments
    ///
    /// * `row` - An optional row from the db.
    ///
    /// # Returns
    ///
    /// An optional `Championship` instance if the row is present.
    #[inline]
    async fn convert_to_championship(&self, row: Option<Row>) -> AppResult<Option<Championship>> {
        if let Some(row) = row {
            let championship = Championship::from_row(&row)?;
            self.cache.championship.set(&championship).await?;
            return Ok(Some(championship));
        }

        Ok(None)
    }
}
