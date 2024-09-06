use std::{
    ops::Range,
    simd::{i32x16, num::SimdInt, Simd},
};

use crate::config::constants::IDS_POOL_SIZE;
use bit_set::BitSet;
use parking_lot::Mutex;
use ring::rand::{SecureRandom, SystemRandom};

/// Generates unique IDs within a specified range.
#[derive(Clone)]
pub struct IdsGenerator {
    data: &'static Mutex<IdsData>,
    range: Range<i32>,
    valid_range: i32,
}

struct IdsData {
    ids: Vec<i32>,
    used_ids: BitSet,
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

        let data = Box::leak(Box::new(Mutex::new(IdsData {
            ids: Vec::with_capacity(IDS_POOL_SIZE),
            used_ids,
        })));

        let generator = IdsGenerator {
            data,
            range,
            valid_range,
        };

        {
            let mut data = generator.data.lock();
            generator.refill(&mut data);
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
        let mut data = self.data.lock();

        match data.ids.pop() {
            Some(id) => id,
            None => {
                self.refill(&mut data);
                data.ids.pop().unwrap_or_else(|| {
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
    fn refill(&self, data: &mut IdsData) {
        let rng = SystemRandom::new();
        let mut buf = [0i32; IDS_POOL_SIZE];

        let byte_buf = unsafe {
            std::slice::from_raw_parts_mut(
                buf.as_mut_ptr() as *mut u8,
                buf.len() * size_of::<i32>(),
            )
        };

        rng.fill(byte_buf).expect("Failed to generate random byte");

        // Check if we need to increase the capacity of the used IDs bitset
        {
            let current_capacity = data.used_ids.capacity();
            let desired_capacity = data.used_ids.len() + IDS_POOL_SIZE;

            if current_capacity < desired_capacity {
                data.used_ids
                    .reserve_len(desired_capacity - data.used_ids.len());
            }
        }

        let valid_range_simd = Simd::splat(self.valid_range);
        let range_start_simd = Simd::splat(self.range.start);

        for chunk in buf.chunks_exact(16) {
            let nums = i32x16::from_slice(chunk).saturating_abs();
            let ids_simd = range_start_simd + (nums % valid_range_simd);

            for i in 0..ids_simd.len() {
                let id = ids_simd[i];

                if data.used_ids.insert(id as usize) {
                    data.ids.push(id);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn id_generator_creation() {
        let range = 0..100000;
        let in_use_ids = vec![1, 2, 3];
        let generator = IdsGenerator::new(range, in_use_ids);

        assert!(!generator.data.lock().ids.is_empty());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn id_generation() {
        let range = 0..100;
        let in_use_ids = vec![1, 2, 3];
        let generator = IdsGenerator::new(range, in_use_ids);

        let id = generator.next();
        assert!((0..100).contains(&id));
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn unique_ids() {
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

    #[test]
    #[cfg_attr(miri, ignore)]
    fn ids_within_range() {
        let range = 100..200;
        let generator = IdsGenerator::new(range.clone(), vec![]);

        for _ in 0..99 {
            // Generate only up to the range size
            let id = generator.next();
            assert!(range.contains(&id), "ID {} is out of range {:?}", id, range);
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn respects_in_use_ids() {
        let range = 0..100;
        let in_use_ids = vec![1, 2, 3];
        let generator = IdsGenerator::new(range, in_use_ids.clone());

        let mut generated_ids = std::collections::HashSet::new();
        for _ in 0..97 {
            generated_ids.insert(generator.next());
        }

        for id in in_use_ids {
            assert!(
                !generated_ids.contains(&id),
                "Generated an ID that was supposed to be in use: {}",
                id
            );
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    #[should_panic(expected = "Failed to generate a unique ID")]
    fn exhausts_range() {
        let range = 0..10;
        let generator = IdsGenerator::new(range, vec![]);

        for _ in 0..11 {
            // Try to generate one more ID than the range allows
            generator.next();
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn concurrent_generation() {
        let range = 0..10000;
        let generator = Arc::new(IdsGenerator::new(range, vec![]));
        let mut handles = vec![];

        for _ in 0..10 {
            let gen = Arc::clone(&generator);
            handles.push(thread::spawn(move || {
                let mut ids = std::collections::HashSet::new();
                for _ in 0..100 {
                    ids.insert(gen.next());
                }
                ids
            }));
        }

        let mut all_ids = std::collections::HashSet::new();
        for handle in handles {
            all_ids.extend(handle.join().unwrap());
        }

        assert_eq!(all_ids.len(), 1000, "Duplicate IDs were generated");
    }
}
