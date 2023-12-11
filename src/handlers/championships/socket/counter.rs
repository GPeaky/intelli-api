use once_cell::sync::Lazy;
use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: Lazy<RwLock<FxHashMap<i32, AtomicUsize>>> =
    Lazy::new(|| RwLock::new(FxHashMap::default()));

pub fn increment(id: i32) {
    let mut counter = COUNTER.try_upgradable_read().unwrap();

    if let Some(counter) = counter.get(&id) {
        counter.fetch_add(1, Ordering::SeqCst);
    } else {
        counter.with_upgraded(|counter| {
            counter.insert(id, AtomicUsize::new(1));
        })
    }
}

pub fn decrement(id: i32) {
    let counter = COUNTER.read();

    if let Some(counter) = counter.get(&id) {
        counter.fetch_sub(1, Ordering::SeqCst);
    }
}

pub fn get(id: i32) -> Option<usize> {
    let counter = COUNTER.read();

    counter
        .get(&id)
        .map(|counter| counter.load(Ordering::SeqCst))
}
