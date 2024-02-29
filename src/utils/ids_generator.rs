use core::panic;
use std::{collections::VecDeque, ops::Range, ptr, sync::Arc};

use ahash::AHashSet;
use ring::rand::{SecureRandom, SystemRandom};
use tokio::sync::Mutex;

use crate::error::AppResult;

const POOL_SIZE: usize = 1000;

/// Trait for retrieving used IDs from a data source.
pub trait UsedIds {
    async fn used_ids(&self) -> AppResult<AHashSet<i32>>;
}

/// Struct for generating unique IDs within a specific range.
///
/// Maintains a buffer of available IDs to minimize on-demand generation. Automatically refills
/// when half of the buffer is consumed.
///
/// # Examples
///
/// ```
/// use crate::utils::IdsGenerator;
///
/// let generator = IdsGenerator::new(1..=100, Some(100));
///
/// // Generate an ID.
/// let id = generator.gen_id();
/// assert!(id >= 1 && id <= 100);
/// ```
#[derive(Clone)]
pub struct IdsGenerator<T: UsedIds> {
    ids: Arc<Mutex<VecDeque<i32>>>,
    range: Range<i32>,
    valid_range: i32,
    repo: T,
}

impl<T: UsedIds> IdsGenerator<T> {
    /// Creates a new `IdsGenerator`.
    ///
    /// # Arguments
    ///
    /// * `range` - The `RangeInclusive<i32>` from which IDs are generated.
    /// * `size` - Optional `usize` specifying the pool size. Uses a default if None.
    pub async fn new(range: Range<i32>, repo: T) -> Self {
        let valid_range = range.end - range.start;

        let generator = IdsGenerator {
            ids: Arc::new(Mutex::new(VecDeque::with_capacity(POOL_SIZE))),
            range,
            valid_range,
            repo,
        };

        {
            let mut ids = generator.ids.lock().await;
            generator.refill(&mut ids).await;
        }

        generator
    }

    /// Refills the ID pool asynchronously when it becomes empty.
    ///
    /// This method generates unique IDs within the specified range, ensuring that
    /// each ID is not already present in the pool or among previously used IDs.
    /// It's designed to be called automatically, maintaining a sufficient supply
    /// of unique IDs for the application's needs.
    ///
    /// Note: `refill` is not intended to be called directly; it is triggered
    /// internally when the pool of available IDs is depleted.
    async fn refill(&self, ids: &mut VecDeque<i32>) {
        let rng = SystemRandom::new();
        let mut local_set = AHashSet::with_capacity(POOL_SIZE);
        let used_ids = self.repo.used_ids().await.unwrap_or_default();

        let mut buf = [0u8; 4 * POOL_SIZE];

        if let Err(e) = rng.fill(&mut buf) {
            panic!("Failed to generate random bytes: {}", e);
        }

        for chunk in buf.chunks(4) {
            let num = unsafe {
                let chunk_ptr = chunk.as_ptr() as *const i32;
                ptr::read_unaligned(chunk_ptr).abs()
            };
            let id = self.range.start + (num % self.valid_range);

            // Todo: if the id is already used, add it to a list of used ids and try again
            if !used_ids.contains(&id) && !local_set.contains(&id) {
                local_set.insert(id);
                ids.push_back(id);
            }
        }
    }

    /// Returns a unique ID.
    ///
    /// Refills the pool automatically if more than half is consumed.
    ///
    /// # Returns
    ///
    /// Returns an `i32` as the generated ID.
    pub async fn next(&self) -> i32 {
        let mut ids = self.ids.lock().await;

        match ids.pop_front() {
            Some(id) => id,
            None => {
                self.refill(&mut ids).await;
                ids.pop_front().unwrap() // Safe to unwrap because we just refilled the queue
            }
        }
    }
}
