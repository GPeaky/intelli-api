mod championship;
mod driver;
mod user;

use std::sync::Arc;

use driver::DriverCache;

use self::{championship::ChampionshipCache, user::UserCache};

const CACHE_CAPACITY: usize = 2_000;

/// Caching layer for users, championships, and tokens.
pub struct ServiceCache {
    pub user: UserCache,
    pub driver: DriverCache,
    pub championship: ChampionshipCache,
}

impl ServiceCache {
    /// Creates a new `ServiceCache` instance.
    pub fn new() -> Self {
        Self {
            user: UserCache::new(),
            driver: DriverCache::new(),
            championship: ChampionshipCache::new(),
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
