use std::sync::Arc;

use tokio_stream::StreamExt;

use db::{Database, EntityCache};
use entities::{Championship, SharedUser, User};
use error::AppResult;
use password_hash::PasswordHasher;
use utils::slice_iter;

/// Repository for managing user data with database and cache integration.
pub struct UserRepository {
    db: &'static Database,
    password_hasher: PasswordHasher,
}

impl UserRepository {
    /// Creates a new UserRepository instance.
    ///
    /// # Arguments
    /// - `db`: Database connection.
    ///
    /// # Returns
    /// A new UserRepository instance.
    pub fn new(db: &'static Database) -> Self {
        let password_hasher = PasswordHasher::new(30);
        Self {
            db,
            password_hasher,
        }
    }

    /// Finds a user by ID.
    ///
    /// # Arguments
    /// - `id`: User ID.
    ///
    /// # Returns
    /// An Option containing the user if found.
    pub async fn find(&self, id: i32) -> AppResult<Option<SharedUser>> {
        if let Some(user) = self.db.cache.user.get(&id) {
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

        match row {
            Some(ref row) => {
                let user = User::from_row_arc(row);
                self.db.cache.user.set(user.clone());
                Ok(Some(user))
            }

            None => Ok(None),
        }
    }

    /// Finds a user by email address.
    ///
    /// # Arguments
    /// - `email`: User's email address.
    ///
    /// # Returns
    /// An Option containing the user if found.
    pub async fn find_by_email(&self, email: &str) -> AppResult<Option<SharedUser>> {
        if let Some(user) = self.db.cache.user.get_by_email(email) {
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

        match row {
            Some(ref row) => {
                let user = User::from_row_arc(row);
                self.db.cache.user.set(user.clone());
                Ok(Some(user))
            }

            None => Ok(None),
        }
    }

    /// Retrieves all championships associated with a user.
    ///
    /// # Arguments
    /// - `id`: The ID of the user.
    ///
    /// # Returns
    /// A vector of Championships associated with the user.
    pub async fn championships(&self, id: i32) -> AppResult<Vec<Arc<Championship>>> {
        if let Some(championships) = self.db.cache.championship.get_user_championships(&id) {
            return Ok(championships);
        };

        let stream = {
            let conn = self.db.pg.get().await?;

            let find_all_stmt = conn
                .prepare_cached(
                    r#"
                        SELECT c.*
                        FROM championships c
                        JOIN championship_users cu ON c.id = cu.championship_id
                        WHERE cu.user_id = $1
                    "#,
                )
                .await?;

            conn.query_raw(&find_all_stmt, &[&id]).await?
        };

        let championships = Championship::from_row_stream(stream).await?;

        self.db
            .cache
            .championship
            .set_user_championships(id, championships.clone());

        Ok(championships)
    }

    /// Counts the number of championships associated with a user.
    ///
    /// # Arguments
    /// - `id`: The ID of the user.
    ///
    /// # Returns
    /// The count of championships associated with the user.
    pub async fn championship_len(&self, id: i32) -> AppResult<usize> {
        let stream = {
            let conn = self.db.pg.get().await?;

            let championship_len_stmt = conn
                .prepare_cached(
                    r#"
                        SELECT
                            c.id
                        FROM
                            championships c
                        JOIN
                            championship_users cu ON c.id = cu.championship_id
                        WHERE
                            cu.user_id = $1
                    "#,
                )
                .await?;

            conn.query_raw(&championship_len_stmt, &[&id]).await?
        };

        Ok(stream.rows_affected().unwrap_or(0) as usize)
    }

    /// Checks if a user exists by email.
    ///
    /// # Arguments
    /// - `email`: Email address to check.
    ///
    /// # Returns
    /// Boolean indicating if the user exists.
    #[inline]
    pub async fn user_exists(&self, email: &str) -> AppResult<bool> {
        Ok(self.find_by_email(email).await?.is_some())
    }

    /// Retrieves the active status of a user.
    ///
    /// # Arguments
    /// - `id`: User ID.
    ///
    /// # Returns
    /// An Option containing the user's active status if found.
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

    /// Retrieves all used user IDs.
    ///
    /// This method should only be called once.
    ///
    /// # Returns
    /// A vector of all used user IDs.
    pub async fn _used_ids(&self) -> AppResult<Vec<i32>> {
        let conn = self.db.pg.get().await?;

        let user_ids_stmt = conn
            .prepare_cached(
                r#"
                    SELECT id FROM users
                "#,
            )
            .await?;

        let stream = conn.query_raw(&user_ids_stmt, slice_iter(&[])).await?;

        tokio::pin!(stream);
        let mut used_ids = Vec::new();

        while let Some(row) = stream.try_next().await? {
            used_ids.push(row.get(0));
        }

        Ok(used_ids)
    }

    /// Hashes a password.
    ///
    /// # Arguments
    /// - `password`: Password to hash.
    ///
    /// # Returns
    /// Hashed password as a string.
    #[inline]
    pub async fn hash_password(&self, password: String) -> AppResult<String> {
        self.password_hasher.hash_password(password).await
    }

    /// Validates a password against a stored hash.
    ///
    /// # Arguments
    /// - `pwd`: Password to validate.
    /// - `hash`: Stored hash to validate against.
    ///
    /// # Returns
    /// Boolean indicating if the password is valid.
    #[inline]
    pub async fn validate_password(&self, pwd: String, hash: Box<str>) -> AppResult<bool> {
        self.password_hasher.verify_password(hash, pwd).await
    }
}
