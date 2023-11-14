use crate::{config::Database, entity::User, error::AppResult};
use axum::async_trait;
use bb8_redis::redis::AsyncCommands;
use rkyv::{Deserialize, Infallible};
use std::sync::Arc;
use tracing::error;

pub struct UserRepository {
    db_conn: Arc<Database>,
}
#[async_trait]
pub trait UserRepositoryTrait {
    fn new(db_conn: &Arc<Database>) -> Self;
    async fn find(&self, id: &i32) -> AppResult<Option<User>>;
    async fn user_exists(&self, email: &str) -> AppResult<bool>;
    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>>;
    fn validate_password(&self, password: &str, hash: &str) -> bool;
    fn active_pools(&self) -> (u32, u32);
}

#[async_trait]
impl UserRepositoryTrait for UserRepository {
    fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: db_conn.clone(),
        }
    }

    async fn find(&self, id: &i32) -> AppResult<Option<User>> {
        if let Some(user) = self.get_from_cache(id).await? {
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
            self.set_to_cache(user).await?;
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

// TODO: Move this to a trait and implement it for all repositories
impl UserRepository {
    async fn get_from_cache(&self, id: &i32) -> AppResult<Option<User>> {
        let mut conn = self.db_conn.redis.get().await.unwrap();
        let user = conn.get::<_, Vec<u8>>(&format!("user:{}", id)).await;

        match user {
            Ok(user) if !user.is_empty() => {
                let archived = unsafe { rkyv::archived_root::<User>(&user) };

                let Ok(user) = archived.deserialize(&mut Infallible) else {
                    error!("Failed to deserialize user from cache");
                    return Ok(None);
                };

                Ok(Some(user))
            }

            Ok(_) => Ok(None),
            Err(_) => Ok(None),
        }
    }

    async fn set_to_cache(&self, user: &User) -> AppResult<()> {
        let mut conn = self.db_conn.redis.get().await.unwrap();
        let bytes = rkyv::to_bytes::<_, 128>(user).unwrap();

        conn.set_ex::<&str, &[u8], ()>(&format!("user:{}", user.id), &bytes[..], 60 * 60)
            .await
            .unwrap();

        Ok(())
    }
}
