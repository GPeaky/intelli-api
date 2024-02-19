use std::{collections::VecDeque, ops::Range, sync::Arc};

use rand::{rngs::StdRng, Rng, SeedableRng};
use tokio::sync::Mutex;

use crate::error::AppResult;

const DEFAULT_POOL_SIZE: usize = 100;

/// Trait for retrieving used IDs from a data source.
pub trait UsedIds {
    async fn used_ids(&self) -> AppResult<Vec<i32>>;
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
    pool_size: usize,
    repository: T,
}

impl<T: UsedIds> IdsGenerator<T> {
    /// Creates a new `IdsGenerator`.
    ///
    /// # Arguments
    ///
    /// * `range` - The `RangeInclusive<i32>` from which IDs are generated.
    /// * `size` - Optional `usize` specifying the pool size. Uses a default if None.
    pub async fn new(range: Range<i32>, repository: T, size: Option<usize>) -> Self {
        let size = size.unwrap_or(DEFAULT_POOL_SIZE);

        let generator = IdsGenerator {
            ids: Arc::new(Mutex::new(VecDeque::with_capacity(size))),
            range,
            pool_size: size,
            repository,
        };

        {
            let mut ids = generator.ids.lock().await;
            generator.refill(&mut ids).await;
        }

        generator
    }

    async fn refill(&self, ids: &mut VecDeque<i32>) {
        let mut rng = StdRng::from_entropy();
        let used_ids = self.repository.used_ids().await.unwrap_or_default();

        // Todo: Check if the id don't exist on in the db and in the range
        while ids.len() < self.pool_size {
            let id = rng.gen_range(self.range.clone());

            if !ids.contains(&id) && !used_ids.contains(&id) {
                ids.push_back(id);
            }
        }
    }

    /// Generates and returns a unique ID.
    ///
    /// Refills the pool automatically if more than half is consumed.
    ///
    /// # Returns
    ///
    /// Returns an `i32` as the generated ID.
    pub async fn gen_id(&self) -> i32 {
        let mut ids = self.ids.lock().await;

        if ids.len() < self.pool_size / 2 {
            self.refill(&mut ids).await;
        }

        ids.pop_front().unwrap() // Safe to unwrap because we just refilled the queue
    }
}
