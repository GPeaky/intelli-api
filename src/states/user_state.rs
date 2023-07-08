use crate::{
    config::Database,
    repositories::{ChampionshipRepository, UserRepository, UserRepositoryTrait},
    services::{
        ChampionshipService, TokenService, TokenServiceTrait, UserService, UserServiceTrait,
    },
};
use std::sync::Arc;

#[derive(Clone)]
pub struct UserState {
    pub user_service: UserService,
    pub user_repository: UserRepository,
    pub token_service: TokenService,
    pub championship_service: ChampionshipService,
    pub championship_repository: ChampionshipRepository,
}

impl UserState {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            user_service: UserService::new(db_conn),
            user_repository: UserRepository::new(db_conn),
            token_service: TokenService::new(db_conn),
            championship_service: ChampionshipService::new(db_conn),
            championship_repository: ChampionshipRepository::new(db_conn),
        }
    }
}
