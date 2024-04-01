use chrono::{DateTime, Utc};
use std::alloc::{alloc, dealloc, Layout};
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[allow(unused)]
pub struct CachedTime {
    time: AtomicPtr<DateTime<Utc>>,
}

impl CachedTime {
    #[allow(unused)]
    pub fn new() -> Arc<Self> {
        let layout = Layout::new::<DateTime<Utc>>();
        let raw_ptr = unsafe { alloc(layout) as *mut DateTime<Utc> };

        if raw_ptr.is_null() {
            unreachable!("Failed to allocate memory for CachedTime instance.");
        }

        unsafe {
            raw_ptr.write(Utc::now());
        }

        let instance = Arc::new(Self {
            time: AtomicPtr::new(raw_ptr),
        });

        let instance_clone = Arc::clone(&instance);

        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(60)).await;
                let ptr = instance_clone.time.load(Ordering::Relaxed);

                if !ptr.is_null() {
                    unsafe {
                        ptr.write(Utc::now());
                    }
                }
            }
        });

        instance
    }

    #[allow(unused)]
    pub fn get(&self) -> DateTime<Utc> {
        unsafe { *self.time.load(Ordering::Relaxed) }
    }
}

impl Drop for CachedTime {
    fn drop(&mut self) {
        unsafe {
            let ptr = self.time.load(Ordering::Relaxed);
            if !ptr.is_null() {
                let layout = Layout::new::<DateTime<Utc>>();
                dealloc(ptr as *mut u8, layout);
            }
        }
    }
}
