//! Fast, non-cryptographic hasher for internal compiler, simulation and cache keys.
use std::collections::HashMap;
use std::hash::{BuildHasher, Hasher};

const K: u64 = 0x517cc1b727220a95;

#[derive(Default, Clone, Copy, Debug)]
pub struct FxHasher(u64);

impl Hasher for FxHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.0 = self.0.rotate_left(5) ^ (byte as u64).wrapping_mul(K);
        }
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.0 = self.0.rotate_left(5) ^ (i as u64).wrapping_mul(K);
    }

    #[inline]
    fn write_u16(&mut self, i: u16) {
        self.0 = self.0.rotate_left(5) ^ (i as u64).wrapping_mul(K);
    }

    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.0 = self.0.rotate_left(5) ^ (i as u64).wrapping_mul(K);
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.0 = self.0.rotate_left(5) ^ i.wrapping_mul(K);
    }

    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.write_u64(i as u64);
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct FxBuildHasher;

impl BuildHasher for FxBuildHasher {
    type Hasher = FxHasher;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        FxHasher(0)
    }
}

pub type FastHashMap<K, V> = HashMap<K, V, FxBuildHasher>;
pub type FastHashSet<T> = std::collections::HashSet<T, FxBuildHasher>;
