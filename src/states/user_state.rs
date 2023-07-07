use crate::{
    config::Database,
    repositories::{UserRepository, UserRepositoryTrait},
    services::{TokenService, TokenServiceTrait, UserService, UserServiceTrait},
};
use std::sync::Arc;

#[derive(Clone)]
pub struct UserState {
    pub user_service: UserService,
    pub user_repository: UserRepository,
    pub token_service: TokenService,
}

impl UserState {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            user_service: UserService::new(db_conn),
            user_repository: UserRepository::new(db_conn),
            token_service: TokenService::new(db_conn),
        }
    }
}
