use std::{
    ops::Range,
    simd::{i32x16, num::SimdInt, Simd},
    sync::Arc,
};

use ahash::AHashSet;
use parking_lot::Mutex;
use ring::rand::{SecureRandom, SystemRandom};

use super::bitset::Bitset;

const POOL_SIZE: usize = 1000;

/// Enum for manage HashSet or Bitset;
pub enum IdContainer {
    HashSet(AHashSet<i32>),
    Bitset(Bitset),
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
pub struct IdsGenerator {
    ids: Arc<Mutex<Vec<i32>>>,
    container: Arc<Mutex<IdContainer>>,
    range: Range<i32>,
    valid_range: i32,
    threshold: usize,
}

impl IdsGenerator {
    /// Creates a new `IdsGenerator`.
    ///
    /// # Arguments
    ///
    /// * `range` - The `RangeInclusive<i32>` from which IDs are generated.
    /// * `size` - Optional `usize` specifying the pool size. Uses a default if None.
    pub fn new(range: Range<i32>, in_use_ids: Vec<i32>) -> Self {
        let valid_range = range.end - range.start;
        let threshold = (valid_range as usize + 7) / 8;

        let container = if in_use_ids.len() * 9 > threshold {
            let mut bitset = Bitset::new(range.clone());
            for id in in_use_ids {
                unsafe {
                    bitset.set(id);
                }
            }

            IdContainer::Bitset(bitset)
        } else {
            let mut hashset = AHashSet::with_capacity(in_use_ids.len());
            for id in in_use_ids {
                hashset.insert(id);
            }

            IdContainer::HashSet(hashset)
        };

        let generator = IdsGenerator {
            ids: Arc::new(Mutex::new(Vec::with_capacity(POOL_SIZE))),
            container: Arc::new(Mutex::new(container)),
            range,
            valid_range,
            threshold,
        };

        {
            let mut ids = generator.ids.lock();
            generator.refill(&mut ids);
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
    fn refill(&self, ids: &mut Vec<i32>) {
        let rng = SystemRandom::new();
        let mut buf = [0i32; POOL_SIZE];
        let mut container = self.container.lock();

        let byte_buf = unsafe {
            std::slice::from_raw_parts_mut(
                buf.as_mut_ptr() as *mut u8,
                buf.len() * std::mem::size_of::<i32>(),
            )
        };

        rng.fill(byte_buf).expect("Failed to generate random byte");

        let valid_range_simd = Simd::splat(self.valid_range);
        let range_start_simd = Simd::splat(self.range.start);

        if let IdContainer::HashSet(ref hashset) = *container {
            if hashset.len() * 9 > self.threshold {
                let mut bitset = Bitset::new(self.range.clone());

                for id in hashset {
                    unsafe {
                        bitset.set(*id);
                    }
                }

                *container = IdContainer::Bitset(bitset);
            }
        }

        for chunk in buf.chunks_exact(16) {
            let nums = i32x16::from_slice(chunk).saturating_abs();
            let ids_simd = range_start_simd + (nums % valid_range_simd);

            for i in 0..ids_simd.len() {
                let id = ids_simd[i];

                match &mut *container {
                    IdContainer::Bitset(bitset) => unsafe {
                        if !bitset.check(id) {
                            bitset.set(id);
                            ids.push(id);
                        }
                    },

                    IdContainer::HashSet(hashset) => {
                        if !hashset.contains(&id) {
                            hashset.insert(id);
                            ids.push(id);
                        }
                    }
                }
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
    pub fn next(&self) -> i32 {
        let mut ids = self.ids.lock();

        match ids.pop() {
            Some(id) => id,
            None => {
                self.refill(&mut ids);
                ids.pop().unwrap() // Safe to unwrap because we just refilled the queue
            }
        }
    }
}
