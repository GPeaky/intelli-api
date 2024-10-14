use quick_cache::sync::Cache;

use entities::SharedDriver;

use super::CACHE_CAPACITY;

pub struct DriverCache {
    inner: Cache<Box<str>, SharedDriver>,
}

impl DriverCache {
    pub fn new() -> Self {
        DriverCache {
            inner: Cache::new(CACHE_CAPACITY),
        }
    }

    pub fn get(&self, steam_name: impl AsRef<str>) -> Option<SharedDriver> {
        self.inner.get(steam_name.as_ref())
    }

    pub fn set(&self, entity: SharedDriver) {
        self.inner.insert(entity.steam_name.clone(), entity)
    }

    pub fn delete(&self, steam_name: impl AsRef<str>) {
        self.inner.remove(steam_name.as_ref());
    }
}
