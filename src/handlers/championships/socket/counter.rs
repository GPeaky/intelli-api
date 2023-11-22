use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

pub struct WebSocketCounter {
    pub connections: Arc<RwLock<FxHashMap<i32, AtomicUsize>>>,
}

// TODO: Check implementation of the RwLock
impl WebSocketCounter {
    pub fn new() -> Self {
        Self {
            connections: Arc::from(RwLock::new(FxHashMap::default())),
        }
    }

    pub async fn get(&self, championship_id: i32) -> Option<usize> {
        let connections = self.connections.read();

        let Some(counter) = connections.get(&championship_id) else {
            return None;
        };

        Some(counter.load(Ordering::Relaxed))
    }

    pub async fn increment(&self, championship_id: i32) {
        let mut connections = self.connections.write();

        let counter = connections
            .entry(championship_id)
            .or_insert(AtomicUsize::new(0));

        counter.fetch_add(1, Ordering::Relaxed);
    }

    pub async fn decrement(&self, championship_id: i32) {
        let mut connections = self.connections.write();

        let counter = connections
            .entry(championship_id)
            .or_insert(AtomicUsize::new(0));

        counter.fetch_sub(1, Ordering::Relaxed);
    }
}
