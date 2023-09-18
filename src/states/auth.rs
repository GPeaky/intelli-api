use crate::{
    config::Database,
    repositories::{GoogleRepository, UserRepository, UserRepositoryTrait},
    services::{EmailService, TokenService, TokenServiceTrait, UserService, UserServiceTrait},
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AuthState {
    pub user_service: UserService,
    pub user_repository: UserRepository,
    pub token_service: TokenService,
    pub email_service: EmailService,
    pub google_repository: GoogleRepository,
}

impl AuthState {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            user_service: UserService::new(db_conn),
            user_repository: UserRepository::new(db_conn),
            token_service: TokenService::new(db_conn),
            email_service: EmailService::new(),
            google_repository: GoogleRepository::new(),
        }
    }
}
