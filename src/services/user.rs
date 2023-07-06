use crate::{config::Database, dtos::RegisterUserDto, error::AppResult};
use axum::async_trait;
use std::sync::Arc;

#[derive(Clone)]
pub struct UserService {
    db_conn: Arc<Database>,
}

#[async_trait]
pub trait UserServiceTrait {
    fn new(db_conn: &Arc<Database>) -> Self;
    async fn register(&self, register: RegisterUserDto) -> AppResult<()>;
}

#[async_trait]
impl UserServiceTrait for UserService {
    fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: db_conn.clone(),
        }
    }

    async fn register(&self, register: RegisterUserDto) -> AppResult<()> {
        println!("Registering user");
        println!("Registering user: {:#?}", register);

        Ok(())
    }
}
