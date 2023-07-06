use crate::{config::Database, error::AppResult};
use axum::async_trait;
use std::sync::Arc;

#[derive(Clone)]
pub struct UserRepository {
    db_conn: Arc<Database>,
}

#[async_trait]
pub trait UserRepositoryTrait {
    fn new(db_conn: &Arc<Database>) -> Self;
    async fn user_exists(&self, email: &str) -> AppResult<bool>;
}

#[async_trait]
impl UserRepositoryTrait for UserRepository {
    fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: db_conn.clone(),
        }
    }

    async fn user_exists(&self, email: &str) -> AppResult<bool> {
        let session = self.db_conn.get_scylla();

        let rows = session
            .query(
                "SELECT email FROM intelli_api.users where email = ? ALLOW FILTERING",
                (email,),
            )
            .await
            .unwrap()
            .rows_num()
            .unwrap();

        Ok(rows > 0)
    }
}
