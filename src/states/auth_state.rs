use crate::{
    config::Database,
    repositories::{UserRepository, UserRepositoryTrait},
    services::{UserService, UserServiceTrait},
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AuthState {
    pub user_service: UserService,
    pub user_repository: UserRepository,
}

impl AuthState {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            user_service: UserService::new(db_conn),
            user_repository: UserRepository::new(db_conn),
        }
    }
}
