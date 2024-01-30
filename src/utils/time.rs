use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use std::{sync::Arc, thread, time::Duration};

#[allow(unused)]
pub struct CachedTime {
    time: Arc<RwLock<DateTime<Utc>>>,
}

#[allow(unused)]

impl CachedTime {
    pub fn new() -> Self {
        let time = Arc::from(RwLock::new(Utc::now()));
        let time_clone = time.clone();

        let instance = Self { time };

        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(60));
            let mut time = time_clone.write();
            *time = Utc::now();
        });

        instance
    }

    pub fn get(&self) -> DateTime<Utc> {
        *self.time.read()
    }
}
