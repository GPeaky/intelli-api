use crate::{
    cache::{EntityCache, RedisCache},
    config::Database,
    entity::{FromRow, User},
    error::{AppResult, UserError},
};
use async_trait::async_trait;

#[derive(Clone)]
pub struct UserRepository {
    db_conn: Database,
    cache: RedisCache,
}

#[async_trait]
pub trait UserRepositoryTrait {
    fn new(db_conn: &Database, cache: &RedisCache) -> Self;
    async fn find(&self, id: &i32) -> AppResult<Option<User>>;
    async fn user_exists(&self, email: &str) -> AppResult<bool>;
    async fn status(&self, id: &i32) -> AppResult<Option<bool>>;
    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>>;
    fn validate_password(&self, password: &str, hash: &str) -> AppResult<bool>;
}

#[async_trait]
impl UserRepositoryTrait for UserRepository {
    fn new(db_conn: &Database, cache: &RedisCache) -> Self {
        Self {
            cache: cache.clone(),
            db_conn: db_conn.clone(),
        }
    }

    async fn find(&self, id: &i32) -> AppResult<Option<User>> {
        if let Some(user) = self.cache.user.get(id).await? {
            return Ok(Some(user));
        };

        let row = {
            let conn = self.db_conn.pg.get().await?;

            let find_user_stmt = conn
                .prepare_cached(
                    r#"
                        SELECT * FROM users
                        WHERE id = $1
                    "#,
                )
                .await?;

            conn.query_opt(&find_user_stmt, &[id]).await?
        };

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

    async fn user_exists(&self, email: &str) -> AppResult<bool> {
        if (self.find_by_email(email).await?).is_some() {
            return Ok(true);
        };

        let row = {
            let conn = self.db_conn.pg.get().await?;

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

    async fn status(&self, id: &i32) -> AppResult<Option<bool>> {
        let row = {
            let conn = self.db_conn.pg.get().await?;

            let user_status_stmt = conn
                .prepare_cached(
                    r#"
                        SELECT active FROM users
                        WHERE id = $1
                    "#,
                )
                .await?;

            conn.query_opt(&user_status_stmt, &[id]).await?
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
            let conn = self.db_conn.pg.get().await?;

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

    fn validate_password(&self, pwd: &str, hash: &str) -> AppResult<bool> {
        Ok(bcrypt::verify(pwd, hash)?)
    }
}
