use std::sync::Arc;

use quick_cache::sync::Cache;

use crate::entity::{Championship, Race};

use super::{EntityCache, CACHE_CAPACITY};

pub struct ChampionshipCache {
    inner: Cache<i32, Arc<Championship>>,
    races: Cache<i32, Vec<Arc<Race>>>,
    name_to_id: Cache<String, i32>,
    user_championships: Cache<i32, Vec<Arc<Championship>>>,
}

impl ChampionshipCache {
    pub fn new() -> Self {
        Self {
            inner: Cache::new(CACHE_CAPACITY),
            races: Cache::new(CACHE_CAPACITY),
            name_to_id: Cache::new(CACHE_CAPACITY),
            user_championships: Cache::new(CACHE_CAPACITY),
        }
    }

    pub fn get_races(&self, id: &i32) -> Option<Vec<Arc<Race>>> {
        self.races.get(id)
    }

    pub fn set_races(&self, id: i32, races: Vec<Arc<Race>>) {
        self.races.insert(id, races)
    }

    pub fn delete_races(&self, id: &i32) {
        self.races.remove(id);
    }

    pub fn get_user_championships(&self, user_id: &i32) -> Option<Vec<Arc<Championship>>> {
        self.user_championships.get(user_id)
    }

    pub fn get_by_name(&self, name: &str) -> Option<Arc<Championship>> {
        self.name_to_id.get(name).and_then(|id| self.get(&id))
    }

    pub fn set_user_championships(&self, user_id: i32, championships: Vec<Arc<Championship>>) {
        self.user_championships.insert(user_id, championships);
    }

    #[inline]
    pub fn delete_by_user(&self, user_id: &i32) {
        self.user_championships.remove(user_id);
    }

    pub fn prune(&self, id: i32, users: Vec<i32>) {
        for user in users {
            self.delete_by_user(&user);
        }

        self.delete(id);
    }
}

impl EntityCache for ChampionshipCache {
    type Entity = Championship;

    fn get(&self, id: &i32) -> Option<Arc<Self::Entity>> {
        self.inner.get(id)
    }

    fn set(&self, entity: Arc<Self::Entity>) {
        self.inner.insert(entity.id, entity.clone());
        self.name_to_id.insert(entity.name.clone(), entity.id);
    }

    fn delete(&self, id: i32) {
        if let Some((_, championship)) = self.inner.remove(&id) {
            self.name_to_id.remove(&championship.name);
        }
    }
}
