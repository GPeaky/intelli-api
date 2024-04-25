use std::sync::Arc;

use ahash::AHashMap;
use parking_lot::RwLock;
use quick_cache::sync::Cache;
use tracing::error;

use crate::{
    config::{constants::*, Database},
    entity::User,
    error::{AppResult, CacheError},
};

use super::EntityCache;

/// `UserCache` is a caching structure for storing and retrieving user data using Redis.
/// It provides methods to interact with a Redis cache to retrieve user information by email.
///
pub struct UserCache {
    cache: Cache<i32, Arc<User>>,
    email_to_id: Cache<String, i32>,
}

impl UserCache {
    /// Creates a new `UserCache` instance with the provided `Database` reference.
    ///
    /// # Arguments
    ///
    /// * `db` - A reference to the database used for caching.
    ///
    /// # Returns
    ///
    /// A new `UserCache` instance.
    pub fn new() -> Self {
        Self {
            cache: Cache::new(100_000),
            email_to_id: Cache::new(100_000),
        }
    }

    /// Retrieves a user by their email address from the Redis cache.
    ///
    /// # Arguments
    ///
    /// * `email` - The email address of the user to retrieve.
    ///
    /// # Returns
    ///
    /// An `AppResult` containing an `Option<User>`. If a user with the provided email
    /// exists in the cache, it returns `Some(user)`. If not found, it returns `None`.
    ///
    /// # Errors
    ///
    /// If there is an error while interacting with the Redis cache or deserializing
    /// the user data, this function returns an `AppError` indicating the issue.
    ///
    pub fn get_by_email(&self, email: &str) -> Option<Arc<User>> {
        if let Some(id) = self.email_to_id.get(email) {
            return self.get(id);
        }

        None
    }
}

impl EntityCache for UserCache {
    type Entity = User;

    fn get(&self, id: i32) -> Option<Arc<Self::Entity>> {
        match self.cache.get(&id) {
            Some(user) => Some(user),
            None => None,
        }
    }

    fn set(&self, entity: Arc<Self::Entity>) {
        self.cache.insert(entity.id, entity.clone());
        self.email_to_id.insert(entity.email.clone(), entity.id);
    }

    fn delete(&self, id: i32) {
        if let Some((_, user)) = self.cache.remove(&id) {
            self.email_to_id.remove(&user.email);
        }
    }
}
