use std::sync::Arc;

use quick_cache::sync::Cache;

use entities::{SharedUser, User};

use super::{EntityCache, CACHE_CAPACITY};

pub struct UserCache {
    inner: Cache<i32, SharedUser>,
    email_to_id: Cache<String, i32>,
}

impl UserCache {
    pub fn new() -> Self {
        Self {
            inner: Cache::new(CACHE_CAPACITY),
            email_to_id: Cache::new(CACHE_CAPACITY),
        }
    }

    /// Retrieves a user by their email address.
    pub fn get_by_email(&self, email: &str) -> Option<SharedUser> {
        let id = self.email_to_id.get(email)?;
        self.get(&id)
    }
}

impl EntityCache for UserCache {
    type Entity = User;

    fn get(&self, id: &i32) -> Option<Arc<Self::Entity>> {
        self.inner.get(id)
    }

    fn set(&self, entity: Arc<Self::Entity>) {
        self.inner.insert(entity.id, entity.clone());
        self.email_to_id.insert(entity.email.clone(), entity.id);
    }

    fn delete(&self, id: i32) {
        if let Some((_, user)) = self.inner.remove(&id) {
            self.email_to_id.remove(&user.email);
        }
    }
}
