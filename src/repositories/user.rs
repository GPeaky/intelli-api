use crate::{config::Database, entity::User, error::AppResult};
use axum::async_trait;
use std::sync::Arc;

#[derive(Clone)]
pub struct UserRepository {
    db_conn: Arc<Database>,
}

#[async_trait]
pub trait UserRepositoryTrait {
    fn new(db_conn: &Arc<Database>) -> Self;
    async fn find(&self, id: &i32) -> AppResult<User>;
    async fn find_by_email(&self, email: &str) -> AppResult<User>;
    fn validate_password(&self, password: &str, hash: &str) -> bool;
}

#[async_trait]
impl UserRepositoryTrait for UserRepository {
    fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: db_conn.clone(),
        }
    }

    // TODO: Check why not finding any user
    async fn find_by_email(&self, email: &str) -> AppResult<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
                SELECT * from user
                WHERE email = ?
            "#,
        )
        .bind(email)
        .fetch_one(&self.db_conn.mysql)
        .await?;

        Ok(user)
    }

    async fn find(&self, id: &i32) -> AppResult<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
                SELECT * FROM user
                WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&self.db_conn.mysql)
        .await?;

        Ok(user)
    }

    fn validate_password(&self, pwd: &str, hash: &str) -> bool {
        argon2::verify_encoded(hash, pwd.as_bytes()).unwrap()
    }
}
