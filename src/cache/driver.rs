use std::sync::Arc;

use quick_cache::sync::Cache;

use crate::entity::Driver;

use super::CACHE_CAPACITY;

// TODO: Check if the performance difference between saving in cache or not makes sense
pub struct DriverCache {
    inner: Cache<String, Arc<Driver>>,
}

impl DriverCache {
    pub fn new() -> Self {
        DriverCache {
            inner: Cache::new(CACHE_CAPACITY),
        }
    }

    pub fn get(&self, steam_name: &str) -> Option<Arc<Driver>> {
        self.inner.get(steam_name)
    }

    pub fn set(&self, entity: Arc<Driver>) {
        self.inner.insert(entity.steam_name.clone(), entity)
    }

    pub fn delete(&self, steam_name: &str) {
        self.inner.remove(steam_name);
    }
}
