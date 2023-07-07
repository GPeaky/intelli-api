use crate::{config::Database, entity::user::User, error::AppResult};
use axum::async_trait;
use std::sync::Arc;

#[derive(Clone)]
pub struct UserRepository {
    db_conn: Arc<Database>,
}

#[async_trait]
pub trait UserRepositoryTrait {
    fn new(db_conn: &Arc<Database>) -> Self;
    async fn find(&self, email: &str) -> AppResult<User>;
    async fn user_exists(&self, email: &str) -> AppResult<bool>;
}

#[async_trait]
impl UserRepositoryTrait for UserRepository {
    fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: db_conn.clone(),
        }
    }

    async fn find(&self, _email: &str) -> AppResult<User> {
        // let session = self.db_conn.get_scylla();

        // let user = session
        //     .execute(
        //         self.db_conn.statements.get("select_user").unwrap(),
        //         (email,),
        //     )
        //     .await
        //     .unwrap()
        //     .single_row_typed::<User>()
        //     .unwrap();

        // Ok(Some(user))

        unimplemented!()
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
