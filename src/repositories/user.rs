use crate::{config::Database, entity::User, error::AppResult};
use axum::async_trait;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct UserRepository {
    db_conn: Arc<Database>,
}

#[async_trait]
pub trait UserRepositoryTrait {
    fn new(db_conn: &Arc<Database>) -> Self;
    async fn find(&self, id: &u32) -> AppResult<Option<User>>;
    async fn user_exists(&self, email: &str) -> AppResult<bool>;
    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>>;
    fn validate_password(&self, password: &str, hash: &str) -> bool;
}

#[async_trait]
impl UserRepositoryTrait for UserRepository {
    fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: db_conn.clone(),
        }
    }

    async fn find(&self, id: &u32) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
                SELECT * FROM user
                WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.db_conn.mysql)
        .await?;

        Ok(user)
    }

    async fn user_exists(&self, email: &str) -> AppResult<bool> {
        let user = sqlx::query_as::<_, (String,)>(
            r#"
                SELECT email FROM user
                WHERE email = ?
            "#,
        )
        .bind(email)
        .fetch_optional(&self.db_conn.mysql)
        .await?;

        Ok(user.is_some())
    }

    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
                SELECT * from user
                WHERE email = ?
            "#,
        )
        .bind(email)
        .fetch_optional(&self.db_conn.mysql)
        .await?;

        Ok(user)
    }

    fn validate_password(&self, pwd: &str, hash: &str) -> bool {
        bcrypt::verify(pwd, hash).is_ok()
    }
}
