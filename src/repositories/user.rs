use ahash::AHashSet;
use deadpool_postgres::tokio_postgres::Row;

use crate::{
    cache::{EntityCache, RedisCache},
    config::Database,
    entity::User,
    error::{AppResult, UserError},
    utils::{verify_password, UsedIds},
};

/// A repository for managing user data within a db and cache.
///
/// This struct provides an interface to interact with user records, offering capabilities
/// to find, verify, and manage user information. It integrates both a db connection
/// and a caching layer to optimize data retrieval and reduce db load.
pub struct UserRepository {
    db: &'static Database,
    cache: &'static RedisCache,
}

impl UsedIds for &'static UserRepository {
    async fn used_ids(&self) -> AppResult<AHashSet<i32>> {
        let conn = self.db.pg.get().await?;

        let user_ids_stmt = conn
            .prepare_cached(
                r#"
                    SELECT id FROM users
            "#,
            )
            .await?;

        let rows = conn.query(&user_ids_stmt, &[]).await?;

        let mut user_ids = AHashSet::with_capacity(rows.len());

        for row in rows {
            let id: i32 = row.get(0);
            user_ids.insert(id);
        }

        Ok(user_ids)
    }
}

impl UserRepository {
    /// Creates a new `UserRepository` instance.
    ///
    /// # Arguments
    /// - `db`: A reference to a `Database` connection.
    /// - `cache`: A reference to a `RedisCache`.
    ///
    /// # Returns
    /// A new instance of `UserRepository`.
    pub fn new(db: &'static Database, cache: &'static RedisCache) -> Self {
        Self { cache, db }
    }

    /// Finds a user by ID.
    ///
    /// # Arguments
    /// - `id`: The user's ID.
    ///
    /// # Returns
    /// An `AppResult` containing the user if found, or `None`.
    pub async fn find(&self, id: i32) -> AppResult<Option<User>> {
        if let Some(user) = self.cache.user.get(id).await? {
            return Ok(Some(user));
        };

        let row = {
            let conn = self.db.pg.get().await?;

            let find_user_stmt = conn
                .prepare_cached(
                    r#"
                        SELECT * FROM users
                        WHERE id = $1
                    "#,
                )
                .await?;

            conn.query_opt(&find_user_stmt, &[&id]).await?
        };

        self.convert_to_user(row).await
    }

    /// Checks if a user exists by their email.
    ///
    /// # Arguments
    /// - `email`: The email address to check.
    ///
    /// # Returns
    /// `true` if the user exists, otherwise `false`.
    pub async fn user_exists(&self, email: &str) -> AppResult<bool> {
        if self.find_by_email(email).await?.is_some() {
            return Ok(true);
        };

        let row = {
            let conn = self.db.pg.get().await?;

            let user_exists_stmt = conn
                .prepare_cached(
                    r#"
                        SELECT EMAIL FROM users
                        WHERE email = $1
                    "#,
                )
                .await?;

            conn.query_opt(&user_exists_stmt, &[&email]).await?
        };

        Ok(row.is_some())
    }

    /// Retrieves the active status of a user.
    ///
    /// # Arguments
    /// - `id`: The user's ID.
    ///
    /// # Returns
    /// An optional boolean indicating the user's active status, or `None` if not found.
    pub async fn status(&self, id: i32) -> AppResult<Option<bool>> {
        let row = {
            let conn = self.db.pg.get().await?;

            let user_status_stmt = conn
                .prepare_cached(
                    r#"
                        SELECT active FROM users
                        WHERE id = $1
                    "#,
                )
                .await?;

            conn.query_opt(&user_status_stmt, &[&id]).await?
        };

        if let Some(row) = row {
            return Ok(Some(row.get(0)));
        }

        Ok(None)
    }

    /// Finds a user by their email address.
    ///
    /// # Arguments
    /// - `email`: The email address of the user.
    ///
    /// # Returns
    /// An `AppResult` containing the user if found, or `None`.
    pub async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        if let Some(user) = self.cache.user.get_by_email(email).await? {
            return Ok(Some(user));
        };

        let row = {
            let conn = self.db.pg.get().await?;

            let find_by_email_stmt = conn
                .prepare_cached(
                    r#"
                        SELECT * FROM users
                        WHERE email = $1
                    "#,
                )
                .await?;

            conn.query_opt(&find_by_email_stmt, &[&email]).await?
        };

        self.convert_to_user(row).await
    }

    /// Converts a db row into a `User` object.
    ///
    /// This private method attempts to convert a db row into a `User` struct.
    /// If the row exists and the user is active, it caches the user information
    /// and returns the user. If the user is not active, it returns an error.
    ///
    /// # Arguments
    /// - `row`: An optional db row that may contain user data.
    ///
    /// # Returns
    /// - `Ok(Some(User))` if the user is found and active.
    /// - `Ok(None)` if the row is `None`.
    /// - `Err(UserError::NotVerified)` if the user is not active.
    #[inline]
    async fn convert_to_user(&self, row: Option<Row>) -> AppResult<Option<User>> {
        if let Some(row) = row {
            let user = User::try_from(&row)?;

            if !user.active {
                Err(UserError::NotVerified)?
            }

            self.cache.user.set(&user).await?;
            return Ok(Some(user));
        }

        Ok(None)
    }

    /// Validates a user's password against a stored hash.
    ///
    /// # Arguments
    /// - `password`: The password to validate.
    /// - `hash`: The hash to validate against.
    ///
    /// # Returns
    /// `true` if the password matches the hash, otherwise `false`.
    #[inline]
    pub fn validate_password(&self, pwd: &str, hash: &str) -> AppResult<bool> {
        verify_password(hash, pwd)
    }
}
