use quick_cache::sync::Cache;

use entities::SharedDriver;

use super::CACHE_CAPACITY;

pub struct DriverCache {
    inner: Cache<String, SharedDriver>,
}

impl DriverCache {
    pub fn new() -> Self {
        DriverCache {
            inner: Cache::new(CACHE_CAPACITY),
        }
    }

    pub fn get(&self, steam_name: &str) -> Option<SharedDriver> {
        self.inner.get(steam_name)
    }

    pub fn set(&self, entity: SharedDriver) {
        self.inner.insert(entity.steam_name.clone(), entity)
    }

    pub fn delete(&self, steam_name: &str) {
        self.inner.remove(steam_name);
    }
}
