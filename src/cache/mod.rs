use self::{championship::ChampionshipCache, token::TokenCache, user::UserCache};
use crate::{config::Database, error::AppResult};
use axum::async_trait;
use std::sync::Arc;

mod championship;
mod token;
mod user;

pub struct RedisCache {
    pub user: UserCache,
    pub championship: ChampionshipCache,
    pub token: TokenCache,
}

impl RedisCache {
    pub fn new(db: &Arc<Database>) -> Self {
        Self {
            user: UserCache::new(db),
            championship: ChampionshipCache::new(db),
            token: TokenCache::new(db),
        }
    }
}

// Generic EntityCache trait
#[async_trait]
pub trait EntityCache {
    type Entity;
    const EXPIRATION: usize = 60 * 60 * 24; // 1 day

    // TODO: Implement more generic functions
    async fn get(&self, id: &i32) -> AppResult<Option<Self::Entity>>;
    async fn set(&self, entity: &Self::Entity) -> AppResult<()>;
}
