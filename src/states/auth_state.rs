use crate::{
    config::Database,
    services::{UserService, UserServiceTrait},
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AuthState {
    pub user_service: UserService,
}

impl AuthState {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            user_service: UserService::new(db_conn),
        }
    }
}
