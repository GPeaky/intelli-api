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
    async fn find(&self, id: &str) -> AppResult<User>;
    async fn find_by_email(&self, email: &str) -> AppResult<User>;
    async fn user_exists(&self, email: &str) -> AppResult<bool>;
    fn validate_password(&self, password: &str, hash: &str) -> bool;
}

#[async_trait]
impl UserRepositoryTrait for UserRepository {
    fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: db_conn.clone(),
        }
    }

    async fn find_by_email(&self, email: &str) -> AppResult<User> {
        let session = self.db_conn.get_scylla();

        let user = session
            .execute(
                self.db_conn.statements.get("select_user_by_email").unwrap(),
                (email,),
            )
            .await?
            .single_row_typed::<User>()?;

        Ok(user)
    }

    async fn find(&self, id: &str) -> AppResult<User> {
        let session = self.db_conn.get_scylla();

        let user = session
            .execute(
                self.db_conn.statements.get("select_user_by_id").unwrap(),
                (id,),
            )
            .await?
            .single_row_typed::<User>()?;

        Ok(user)
    }

    async fn user_exists(&self, email: &str) -> AppResult<bool> {
        let session = self.db_conn.get_scylla();

        let rows = session
            .execute(
                self.db_conn
                    .statements
                    .get("select_email_by_email")
                    .unwrap(),
                (email,),
            )
            .await?
            .rows_num()?;

        Ok(rows > 0)
    }

    fn validate_password(&self, pwd: &str, hash: &str) -> bool {
        argon2::verify_encoded(hash, pwd.as_bytes()).unwrap()
    }
}
