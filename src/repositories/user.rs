use crate::{
    cache::{EntityCache, RedisCache},
    config::Database,
    entity::User,
    error::AppResult,
};
use axum::async_trait;
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
    fn active_pools(&self) -> (u32, u32);
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

        let user = sqlx::query_as::<_, User>(
            r#"
                SELECT * FROM users
                WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.db_conn.pg)
        .await?;

        // TODO: Check if handle it in a better way
        if let Some(ref user) = user {
            self.cache.user.set(user).await?;
        }

        Ok(user)
    }

    // TODO: Add cache for this function
    async fn user_exists(&self, email: &str) -> AppResult<bool> {
        let user = sqlx::query_as::<_, (String,)>(
            r#"
                SELECT email FROM users
                WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.db_conn.pg)
        .await?;

        Ok(user.is_some())
    }

    // TODO: Add cache for this function
    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
                SELECT * from users
                WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.db_conn.pg)
        .await?;

        Ok(user)
    }

    // TODO: Remove this function from this trait
    fn active_pools(&self) -> (u32, u32) {
        self.db_conn.active_pools()
    }

    fn validate_password(&self, pwd: &str, hash: &str) -> bool {
        bcrypt::verify(pwd, hash).unwrap()
    }
}
