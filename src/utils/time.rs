use chrono::{DateTime, Utc};
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

pub struct CachedTime {
    time: AtomicPtr<DateTime<Utc>>,
}

impl CachedTime {
    pub fn new() -> Arc<Self> {
        let initial_time = Box::new(Utc::now());
        let time_ptr = Box::into_raw(initial_time);

        let instance = Arc::new(Self {
            time: AtomicPtr::new(time_ptr),
        });

        let instance_clone = Arc::clone(&instance);

        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(60)).await;
                let new_time = Box::new(Utc::now());
                let new_time_ptr = Box::into_raw(new_time);

                let old_ptr = instance_clone.time.swap(new_time_ptr, Ordering::Relaxed);

                unsafe {
                    let _ = Box::from_raw(old_ptr);
                }
            }
        });

        instance
    }

    pub fn get(&self) -> DateTime<Utc> {
        unsafe { *self.time.load(Ordering::Relaxed) }
    }
}
