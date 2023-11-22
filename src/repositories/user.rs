use crate::{
    cache::{EntityCache, RedisCache},
    config::Database,
    entity::{FromRow, User},
    error::AppResult,
};
use async_trait::async_trait;
use std::sync::Arc;

pub struct UserRepository {
    db_conn: Arc<Database>,
    cache: Arc<RedisCache>,
}

#[async_trait]
pub trait UserRepositoryTrait {
    fn new(db_conn: &Arc<Database>, cache: &Arc<RedisCache>) -> Self;
    async fn find(&self, id: &i32) -> AppResult<Option<User>>;
    async fn user_exists(&self, email: &str) -> AppResult<bool>;
    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>>;
    fn validate_password(&self, password: &str, hash: &str) -> bool;
    fn active_pools(&self) -> (usize, usize);
}

#[async_trait]
impl UserRepositoryTrait for UserRepository {
    fn new(db_conn: &Arc<Database>, cache: &Arc<RedisCache>) -> Self {
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

            let cached_statement = conn
                .prepare_cached(
                    r#"
                    SELECT * FROM users
                    WHERE id = $1
                "#,
                )
                .await?;

            conn.query_opt(&cached_statement, &[id]).await?
        };

        if let Some(row) = row {
            let user = User::from_row(&row)?;
            self.cache.user.set(&user).await?;

            return Ok(Some(user));
        }

        Ok(None)
    }

    // TODO: Check if this function is necessary with cache
    async fn user_exists(&self, email: &str) -> AppResult<bool> {
        if (self.find_by_email(email).await?).is_some() {
            return Ok(true);
        };

        let row = {
            let conn = self.db_conn.pg.get().await?;

            let cached_statement = conn
                .prepare_cached(
                    r#"
                    SELECT EMAIL FROM users
                    WHERE email = $1
                "#,
                )
                .await?;

            conn.query_opt(&cached_statement, &[&email]).await?
        };

        Ok(row.is_some())
    }

    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        if let Some(user) = self.cache.user.get_by_email(email).await? {
            return Ok(Some(user));
        };

        let row = {
            let conn = self.db_conn.pg.get().await?;

            let cached_statement = conn
                .prepare_cached(
                    r#"
                    SELECT * FROM users
                    WHERE email = $1
                "#,
                )
                .await?;

            conn.query_opt(&cached_statement, &[&email]).await?
        };

        if let Some(row) = row {
            let user = User::from_row(&row)?;
            self.cache.user.set(&user).await?;

            return Ok(Some(user));
        }

        Ok(None)
    }

    // TODO: Remove this function from this trait
    fn active_pools(&self) -> (usize, usize) {
        self.db_conn.active_pools()
    }

    fn validate_password(&self, pwd: &str, hash: &str) -> bool {
        bcrypt::verify(pwd, hash).unwrap()
    }
}
