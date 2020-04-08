use std::{
    collections::{
        HashSet,
        VecDeque,
    },
    hash::{
        BuildHasherDefault,
        Hasher,
    },
};

use twox_hash::XxHash64;

struct CustomHasher {
    result: u64,
}

impl CustomHasher {
    fn finish(&self) -> u64 {
        self.result
    }
    fn write(&mut self, i: u64) {
        self.result = i;
    }
}

impl Default for CustomHasher {
    fn default() -> Self {
        Self { result: 17241709254077376921 }
    }
}

impl Hasher for CustomHasher {
    fn finish(&self) -> u64 {
        CustomHasher::finish(self)
    }
    fn write(&mut self, bytes: &[u8]) {
        use std::convert::TryInto;
        let (int_bytes, _rest) = bytes.split_at(std::mem::size_of::<u64>());
        let i = u64::from_ne_bytes(int_bytes.try_into().unwrap());
        CustomHasher::write(self, i);
    }
}

pub(crate) struct TinyHashCache {
    max_capacity: usize,
    cache: HashSet<u64, BuildHasherDefault<CustomHasher>>,
    elem_order: VecDeque<u64>,
}

impl TinyHashCache {
    pub fn new(max_capacity: usize) -> Self {
        Self {
            max_capacity,
            cache: HashSet::default(),
            elem_order: VecDeque::new(),
        }
    }

    pub fn insert(&mut self, bytes: &[u8]) -> bool {
        let hash = xx_hash(bytes);

        if self.contains(&hash) {
            return false;
        }

        if self.cache.len() >= self.max_capacity {
            let first = self.elem_order.pop_front().unwrap();
            self.cache.remove(&first);
        }

        self.cache.insert(hash.clone());
        self.elem_order.push_back(hash);

        true
    }

    fn contains(&self, hash: &u64) -> bool {
        self.cache.contains(hash)
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }
}

fn xx_hash(buf: &[u8]) -> u64 {
    let mut hasher = XxHash64::default();

    hasher.write(buf);
    hasher.finish()
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_cache_insert_same_elements() {
        let mut cache = TinyHashCache::new(10);

        let first_buf = &[1, 2, 3];
        let second_buf = &[1, 2, 3];

        assert_eq!(cache.insert(first_buf), true);
        assert_eq!(cache.insert(second_buf), false);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_cache_insert_different_elements() {
        let mut cache = TinyHashCache::new(10);

        let first_buf = &[1, 2, 3];
        let second_buf = &[3, 4, 5];

        assert_eq!(cache.insert(first_buf), true);
        assert_eq!(cache.insert(second_buf), true);
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_cache_max_capacity() {
        let mut cache = TinyHashCache::new(1);

        let first_buf = &[1, 2, 3];
        let second_buf = &[3, 4, 5];
        let second_buf_clone = second_buf.clone();

        assert_eq!(cache.insert(first_buf), true);
        assert_eq!(cache.insert(second_buf), true);
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.insert(&second_buf_clone), false);
    }
}
