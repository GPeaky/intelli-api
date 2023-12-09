mod championship;
mod f123;
mod token;
mod user;

use self::{championship::ChampionshipCache, token::TokenCache, user::UserCache};
use crate::{
    config::{constants::REDIS_CACHE_EXPIRATION, Database},
    error::AppResult,
};
use async_trait::async_trait;
pub(crate) use f123::*;
use std::sync::Arc;

#[derive(Clone)]
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
    const EXPIRATION: usize = REDIS_CACHE_EXPIRATION;

    async fn get(&self, id: &i32) -> AppResult<Option<Self::Entity>>;
    async fn set(&self, entity: &Self::Entity) -> AppResult<()>;
    async fn delete(&self, id: &i32) -> AppResult<()>;
}
