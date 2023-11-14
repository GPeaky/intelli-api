use crate::{
    cache::RedisCache,
    config::Database,
    repositories::{GoogleRepository, UserRepository, UserRepositoryTrait},
    services::{EmailService, TokenService, TokenServiceTrait, UserService, UserServiceTrait},
};
use std::sync::Arc;

pub type AuthState = Arc<AuthStateInner>;

pub struct AuthStateInner {
    pub user_service: UserService,
    pub user_repository: UserRepository,
    pub token_service: TokenService,
    pub email_service: EmailService,
    pub google_repository: GoogleRepository,
}

impl AuthStateInner {
    pub fn new(db_conn: &Arc<Database>, cache: &Arc<RedisCache>) -> Self {
        Self {
            user_service: UserService::new(db_conn, cache),
            user_repository: UserRepository::new(db_conn, cache),
            token_service: TokenService::new(cache),
            email_service: EmailService::new(),
            google_repository: GoogleRepository::new(),
        }
    }
}
