use std::sync::Arc;

use driver::DriverCache;

use self::{championship::ChampionshipCache, token::TokenCache, user::UserCache};

mod championship;
mod driver;
mod token;
mod user;

const CACHE_CAPACITY: usize = 2_000;

/// Caching layer for users, championships, and tokens.
pub struct ServiceCache {
    pub user: UserCache,
    pub driver: DriverCache,
    pub championship: ChampionshipCache,
    pub token: TokenCache,
}

impl ServiceCache {
    /// Creates a new `ServiceCache` instance.
    pub fn new() -> Self {
        Self {
            user: UserCache::new(),
            driver: DriverCache::new(),
            championship: ChampionshipCache::new(),
            token: TokenCache::new(),
        }
    }
}

/// Generic trait for entity caching operations.
pub trait EntityCache {
    type Entity;

    /// Retrieves an entity by ID.
    fn get(&self, id: &i32) -> Option<Arc<Self::Entity>>;

    /// Stores an entity in the cache.
    fn set(&self, entity: Arc<Self::Entity>);

    /// Removes an entity from the cache by ID.
    fn delete(&self, id: i32);
}
