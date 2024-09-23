use crate::config::constants::IDS_POOL_SIZE;
use crate::utils::bitset::Bitset;
use ahash::AHashSet;
use parking_lot::Mutex;
use ring::rand::{SecureRandom, SystemRandom};
use std::{
    ops::Range,
    simd::{i32x16, num::SimdInt, Simd},
};

// Enums
#[derive(PartialEq)]
pub enum ContainerType {
    HashSet,
    BitSet,
}

enum IdsContainer {
    HashSet(AHashSet<i32>, Range<i32>, usize), // TODO: Try to remove metadata
    BitSet(Bitset),
}

// Structs
/// Generates unique IDs within a specified range.
#[derive(Clone)]
pub struct IdsGenerator {
    data: &'static Mutex<IdsData>,
    range: Range<i32>,
    valid_range: i32,
}

struct IdsData {
    ids: Vec<i32>,
    container: IdsContainer,
}

// Implementations
impl IdsContainer {
    /// Creates a new `IdsContainer`.
    ///
    /// # Arguments
    ///
    /// * `range` - The range of valid IDs.
    /// * `in_use_ids` - A vector of IDs already in use.
    /// * `valid_range` - The range of valid IDs.
    ///
    /// # Returns
    ///
    /// A new instance of `IdsContainer`.
    pub fn new(range: Range<i32>, in_use_ids: Vec<i32>, valid_range: i32) -> Self {
        let threshold = (valid_range as usize + 7) / 8;

        if in_use_ids.len() * 9 > threshold {
            let mut bitset = Bitset::new(range.clone());
            for id in in_use_ids {
                unsafe {
                    bitset.insert(id);
                }
            }
            IdsContainer::BitSet(bitset)
        } else {
            let mut hashset = AHashSet::with_capacity(in_use_ids.len());
            for id in in_use_ids {
                hashset.insert(id);
            }
            IdsContainer::HashSet(hashset, range, threshold)
        }
    }

    /// Checks if the number of used IDs exceeds a threshold.
    #[inline]
    pub fn check_threshold(&mut self) {
        if let IdsContainer::HashSet(ref hashset, range, threshold) = self {
            if hashset.len() * 9 > *threshold {
                let mut bitset = Bitset::new(range.clone());
                for id in hashset {
                    unsafe {
                        bitset.insert(*id);
                    }
                }
                *self = IdsContainer::BitSet(bitset)
            }
        }
    }

    /// Inserts a new ID into the container.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID to insert.
    ///
    /// # Returns
    ///
    /// `true` if the ID was successfully inserted, `false` otherwise.
    #[inline]
    pub fn insert(&mut self, id: i32) -> bool {
        match self {
            IdsContainer::BitSet(bitset) => unsafe { bitset.insert(id) },
            IdsContainer::HashSet(hashset, _, _) => hashset.insert(id),
        }
    }

    #[allow(unused)]
    pub fn container_type(&self) -> ContainerType {
        match self {
            IdsContainer::HashSet(..) => ContainerType::HashSet,
            IdsContainer::BitSet(..) => ContainerType::BitSet,
        }
    }
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
        let container = IdsContainer::new(range.clone(), in_use_ids, valid_range);

        let data = Box::leak(Box::new(Mutex::new(IdsData {
            ids: Vec::with_capacity(IDS_POOL_SIZE),
            container,
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
                buf.len() * std::mem::size_of::<i32>(),
            )
        };

        rng.fill(byte_buf).expect("Failed to generate random byte");

        data.container.check_threshold();

        let valid_range_simd = Simd::splat(self.valid_range);
        let range_start_simd = Simd::splat(self.range.start);

        for chunk in buf.chunks_exact(16) {
            let nums = i32x16::from_slice(chunk).saturating_abs();
            let ids_simd = range_start_simd + (nums % valid_range_simd);

            for i in 0..ids_simd.len() {
                let id = ids_simd[i];
                if data.container.insert(id) {
                    data.ids.push(id);
                }
            }
        }
    }
}

// Tests module
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
