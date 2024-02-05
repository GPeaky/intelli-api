use async_trait::async_trait;

use crate::{
    config::{constants::REDIS_CACHE_EXPIRATION, Database},
    error::AppResult,
};
pub(crate) use f123::*;

use self::{championship::ChampionshipCache, token::TokenCache, user::UserCache};

mod championship;
mod f123;
mod token;
mod user;

/// Represents a caching layer for various entities using Redis.
///
/// This struct provides separate caches for users, championships, and tokens,
/// allowing for efficient data retrieval and storage.
///
/// # Examples
///
/// ```
/// let db = Database::connect(...); // Assume `Database::connect` is a method that returns a database connection.
/// let cache = RedisCache::new(&db);
/// ```
#[derive(Clone)]
pub struct RedisCache {
    /// Cache for user-related data.
    pub user: UserCache,
    /// Cache for championship-related data.
    pub championship: ChampionshipCache,
    /// Cache for token-related data.
    pub token: TokenCache,
}

impl RedisCache {
    /// Creates a new `RedisCache` instance with caches initialized for users,
    /// championships, and tokens.
    ///
    /// # Arguments
    ///
    /// * `db` - A reference to a `Database` instance to be used for cache initialization.
    pub fn new(db: &Database) -> Self {
        Self {
            user: UserCache::new(db),
            championship: ChampionshipCache::new(db),
            token: TokenCache::new(db),
        }
    }
}

/// A trait for caching entities, providing generic methods for getting, setting,
/// and deleting entities in the cache.
///
/// Implementors must specify the entity type and can override the default cache expiration.
#[async_trait]
pub trait EntityCache {
    /// The type of entity being cached.
    type Entity;

    /// Default expiration time for cache entries, in seconds.
    const EXPIRATION: u64 = REDIS_CACHE_EXPIRATION;

    // Retrieves an entity by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the entity to retrieve.
    ///
    /// # Returns
    ///
    /// A result containing an option with the entity if found, or an none if not.
    async fn get(&self, id: i32) -> AppResult<Option<Self::Entity>>;

    /// Stores an entity in the cache.
    ///
    /// # Arguments
    ///
    /// * `entity` - A reference to the entity to store.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    async fn set(&self, entity: &Self::Entity) -> AppResult<()>;

    /// Removes an entity from the cache by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the entity to remove.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    async fn delete(&self, id: i32) -> AppResult<()>;
}
