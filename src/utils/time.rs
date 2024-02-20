use chrono::{DateTime, Utc};
use std::{cell::UnsafeCell, ptr::NonNull, time::Duration};
use tokio::time::sleep;

#[allow(unused)]
pub struct CachedTime {
    time: NonNull<UnsafeCell<DateTime<Utc>>>,
}

unsafe impl Send for CachedTime {}
unsafe impl Sync for CachedTime {}

#[allow(unused)]

impl CachedTime {
    pub fn new() -> Self {
        let time = Box::new(UnsafeCell::new(Utc::now()));
        let time_ptr = Box::into_raw(time);

        let instance = Self {
            time: unsafe { NonNull::new_unchecked(time_ptr) },
        };

        // TODO: finish this impl
        // tokio::spawn(async move {
        //     loop {
        //         sleep(Duration::from_secs(60)).await;

        //         unsafe {
        //             *(*time_ptr).get() = Utc::now();
        //         }
        //     }
        // });

        instance
    }

    pub unsafe fn get(&self) -> DateTime<Utc> {
        *(self.time.as_ref()).get()
    }
}
