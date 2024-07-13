use std::{
    ops::Range,
    simd::{i32x16, num::SimdInt, Simd},
    sync::Arc,
};

use crate::config::constants::IDS_POOL_SIZE;
use bit_set::BitSet;
use parking_lot::Mutex;
use ring::rand::{SecureRandom, SystemRandom};

/// Generates unique IDs within a specified range.
#[derive(Clone)]
pub struct IdsGenerator {
    ids: Arc<Mutex<Vec<i32>>>,
    used_ids: Arc<Mutex<BitSet>>,
    range: Range<i32>,
    valid_range: i32,
}

impl IdsGenerator {
    /// Creates a new `IdsGenerator`.
    ///
    /// # Arguments
    ///
    /// * `range` - A range of integers within which IDs will be generated.
    /// * `in_use_ids` - A vector of IDs that are already in use and should not be generated.
    ///
    /// # Returns
    ///
    /// A new instance of `IdsGenerator`.
    pub fn new(range: Range<i32>, in_use_ids: Vec<i32>) -> Self {
        let valid_range = range.end - range.start;
        let mut used_ids = BitSet::with_capacity(in_use_ids.len());

        for id in in_use_ids {
            used_ids.insert(id as usize);
        }

        let generator = IdsGenerator {
            ids: Arc::new(Mutex::new(Vec::with_capacity(IDS_POOL_SIZE))),
            used_ids: Arc::new(Mutex::new(BitSet::new())),
            range,
            valid_range,
        };

        {
            let mut ids = generator.ids.lock();
            generator.refill(&mut ids);
        }

        generator
    }

    /// Returns the next available ID.
    ///
    /// # Returns
    ///
    /// The next available unique ID.
    ///
    /// # Panics
    ///
    /// Panics if no unique ID can be generated.
    pub fn next(&self) -> i32 {
        let mut ids = self.ids.lock();

        match ids.pop() {
            Some(id) => id,
            None => {
                self.refill(&mut ids);
                ids.pop().unwrap_or_else(|| {
                    panic!("Failed to generate a unique ID: No more unique IDs available")
                })
            }
        }
    }

    /// Refills the pool of available IDs.
    ///
    /// # Arguments
    ///
    /// * `ids` - A mutable reference to a vector of IDs to be refilled.
    fn refill(&self, ids: &mut Vec<i32>) {
        let rng = SystemRandom::new();
        let mut buf = [0i32; IDS_POOL_SIZE];
        let mut used_ids = self.used_ids.lock();

        let byte_buf = unsafe {
            std::slice::from_raw_parts_mut(
                buf.as_mut_ptr() as *mut u8,
                buf.len() * size_of::<i32>(),
            )
        };

        rng.fill(byte_buf).expect("Failed to generate random byte");

        let valid_range_simd = Simd::splat(self.valid_range);
        let range_start_simd = Simd::splat(self.range.start);

        let new_capacity = used_ids.capacity() + buf.len();
        used_ids.reserve_len(new_capacity);

        for chunk in buf.chunks_exact(16) {
            let nums = i32x16::from_slice(chunk).saturating_abs();
            let ids_simd = range_start_simd + (nums % valid_range_simd);

            for i in 0..ids_simd.len() {
                let id = ids_simd[i];

                if used_ids.insert(id as usize) {
                    ids.push(id);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_generator_creation() {
        let range = 0..100000;
        let in_use_ids = vec![1, 2, 3];
        let generator = IdsGenerator::new(range, in_use_ids);

        assert!(!generator.ids.lock().is_empty());
    }

    #[test]
    fn test_id_generation() {
        let range = 0..100;
        let in_use_ids = vec![1, 2, 3];
        let generator = IdsGenerator::new(range, in_use_ids);

        let id = generator.next();
        assert!((0..100).contains(&id));
    }

    #[test]
    fn test_unique_ids() {
        let range = 0..1000;
        let in_use_ids = vec![1, 2, 3];
        let generator = IdsGenerator::new(range, in_use_ids);

        let mut ids = std::collections::HashSet::new();
        for _ in 0..100 {
            let id = generator.next();
            assert!(!ids.contains(&id));
            ids.insert(id);
        }
    }
}
