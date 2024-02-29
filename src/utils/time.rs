use chrono::{DateTime, Utc};
use std::{
    alloc::{alloc, dealloc, Layout},
    sync::{
        atomic::{AtomicPtr, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::time::sleep;

pub struct CachedTime {
    time: AtomicPtr<DateTime<Utc>>,
    instant: AtomicPtr<Instant>,
}

impl CachedTime {
    pub fn new() -> Arc<Self> {
        let layout: Layout = Layout::new::<DateTime<Utc>>();
        let instant_layout: Layout = Layout::new::<Instant>();
        let instant_ptr = unsafe { alloc(instant_layout) as *mut Instant };
        let utc_ptr = unsafe { alloc(layout) as *mut DateTime<Utc> };

        if !utc_ptr.is_null() && !instant_ptr.is_null() {
            unsafe {
                instant_ptr.write(Instant::now());
                utc_ptr.write(Utc::now());
            }

            let instance = Arc::new(Self {
                time: AtomicPtr::new(utc_ptr),
                instant: AtomicPtr::new(instant_ptr),
            });

            let instance_clone = Arc::clone(&instance);

            tokio::spawn(async move {
                loop {
                    sleep(Duration::from_secs(10)).await;
                    let utc_ptr = instance_clone.time.load(Ordering::Relaxed);
                    let instant_ptr = instance_clone.instant.load(Ordering::Relaxed);

                    if !utc_ptr.is_null() && !instant_ptr.is_null() {
                        unsafe {
                            utc_ptr.write(Utc::now());
                            instant_ptr.write(Instant::now());
                        }
                    }
                }
            });

            instance
        } else {
            unreachable!("Failed to allocate memory for CachedTime instance.");
        }
    }

    #[allow(unused)]

    pub fn utc(&self) -> DateTime<Utc> {
        unsafe { *self.time.load(Ordering::Relaxed) }
    }

    pub fn instant(&self) -> Instant {
        unsafe { *self.instant.load(Ordering::Relaxed) }
    }
}

impl Drop for CachedTime {
    fn drop(&mut self) {
        unsafe {
            let instant_ptr = self.instant.load(Ordering::Relaxed);
            let utc_ptr = self.time.load(Ordering::Relaxed);

            if !utc_ptr.is_null() {
                let layout = Layout::new::<DateTime<Utc>>();
                dealloc(utc_ptr as *mut u8, layout);
            }

            if !instant_ptr.is_null() {
                let layout = Layout::new::<Instant>();
                dealloc(instant_ptr as *mut u8, layout);
            }
        }
    }
}
