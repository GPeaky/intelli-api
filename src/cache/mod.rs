use async_trait::async_trait;

pub(crate) use f123::*;

use crate::{
    config::{constants::REDIS_CACHE_EXPIRATION, Database},
    error::AppResult,
};

use self::{championship::ChampionshipCache, token::TokenCache, user::UserCache};

mod championship;
mod f123;
mod token;
mod user;

#[derive(Clone)]
pub struct RedisCache {
    pub user: UserCache,
    pub championship: ChampionshipCache,
    pub token: TokenCache,
}

impl RedisCache {
    pub fn new(db: &Database) -> Self {
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
    const EXPIRATION: u64 = REDIS_CACHE_EXPIRATION;

    async fn get(&self, id: &i32) -> AppResult<Option<Self::Entity>>;
    async fn set(&self, entity: &Self::Entity) -> AppResult<()>;
    async fn delete(&self, id: &i32) -> AppResult<()>;
}
