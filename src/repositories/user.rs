use async_trait::async_trait;
use tokio_postgres::Row;

use crate::{
    cache::{EntityCache, RedisCache},
    config::Database,
    entity::{FromRow, User},
    error::{AppResult, UserError},
    utils::UsedIds,
};

/// A repository for managing user data within a db and cache.
///
/// This struct provides an interface to interact with user records, offering capabilities
/// to find, verify, and manage user information. It integrates both a db connection
/// and a caching layer to optimize data retrieval and reduce db load.
#[derive(Clone)]
pub struct UserRepository {
    db: &'static Database,
    cache: &'static RedisCache,
}

impl UserRepository {
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
            let user = User::from_row(&row)?;

            if !user.active {
                Err(UserError::NotVerified)?
            }

            self.cache.user.set(&user).await?;
            return Ok(Some(user));
        }

        Ok(None)
    }
}

impl UsedIds for UserRepository {
    async fn used_ids(&self) -> AppResult<Vec<i32>> {
        let conn = self.db.pg.get().await?;

        let user_ids_stmt = conn
            .prepare_cached(
                r#"
                    SELECT id FROM users
            "#,
            )
            .await?;

        let rows = conn.query(&user_ids_stmt, &[]).await?;

        let user_ids = rows.iter().map(|row| row.get(0)).collect();

        Ok(user_ids)
    }
}

#[async_trait]
pub trait UserRepositoryTrait {
    /// Creates a new `UserRepository` instance.
    ///
    /// # Arguments
    /// - `db`: A reference to a `Database` connection.
    /// - `cache`: A reference to a `RedisCache`.
    ///
    /// # Returns
    /// A new instance of `UserRepository`.
    fn new(db: &'static Database, cache: &'static RedisCache) -> Self;

    /// Finds a user by ID.
    ///
    /// # Arguments
    /// - `id`: The user's ID.
    ///
    /// # Returns
    /// An `AppResult` containing the user if found, or `None`.
    async fn find(&self, id: i32) -> AppResult<Option<User>>;

    /// Checks if a user exists by their email.
    ///
    /// # Arguments
    /// - `email`: The email address to check.
    ///
    /// # Returns
    /// `true` if the user exists, otherwise `false`.
    async fn user_exists(&self, email: &str) -> AppResult<bool>;

    /// Retrieves the active status of a user.
    ///
    /// # Arguments
    /// - `id`: The user's ID.
    ///
    /// # Returns
    /// An optional boolean indicating the user's active status, or `None` if not found.
    async fn status(&self, id: i32) -> AppResult<Option<bool>>;

    /// Finds a user by their email address.
    ///
    /// # Arguments
    /// - `email`: The email address of the user.
    ///
    /// # Returns
    /// An `AppResult` containing the user if found, or `None`.
    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>>;

    /// Validates a user's password against a stored hash.
    ///
    /// # Arguments
    /// - `password`: The password to validate.
    /// - `hash`: The hash to validate against.
    ///
    /// # Returns
    /// `true` if the password matches the hash, otherwise `false`.
    fn validate_password(&self, password: &str, hash: &str) -> AppResult<bool>;
}

#[async_trait]
impl UserRepositoryTrait for UserRepository {
    fn new(db: &'static Database, cache: &'static RedisCache) -> Self {
        Self { cache, db }
    }

    async fn find(&self, id: i32) -> AppResult<Option<User>> {
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

    async fn user_exists(&self, email: &str) -> AppResult<bool> {
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

    async fn status(&self, id: i32) -> AppResult<Option<bool>> {
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

    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
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

    fn validate_password(&self, pwd: &str, hash: &str) -> AppResult<bool> {
        Ok(bcrypt::verify(pwd, hash)?)
    }
}
