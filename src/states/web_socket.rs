use crate::{
    config::Database,
    repositories::{ChampionshipRepository, F123Repository},
    services::F123Service,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct WebSocketState {
    pub f123_service: F123Service,
    pub f123_repository: F123Repository,
    pub championship_repository: ChampionshipRepository,
}

impl WebSocketState {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            f123_service: F123Service::new(db_conn),
            f123_repository: F123Repository::new(db_conn),
            championship_repository: ChampionshipRepository::new(db_conn),
        }
    }
}
