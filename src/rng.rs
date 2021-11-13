use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

pub const RNG_SEED: u64 = 13157594558721065924;

pub struct Rng {
    data:   u64,
    hasher: DefaultHasher,
}

impl Rng {
    pub fn new() -> Rng { Rng::new_with_seed(RNG_SEED) }

    pub fn new_with_seed(seed: u64) -> Rng {
        Rng {
            data:   seed,
            hasher: DefaultHasher::new(),
        }
    }

    pub fn next_usize_max(&mut self, max: usize) -> usize {
        let mask = 2_usize.pow(64 - (max as u64).leading_zeros()) - 1;
        let mut out = self.next_usize() & mask;
        while out > max { out = self.next_usize() & mask }
        out
    }

    pub fn next_usize(&mut self) -> usize {
        let mut out = 0;
        for i in self.take(std::mem::size_of::<usize>()) {
            out = out << 8 | i as usize;
        }
        out
    }

    pub fn next_bool(&mut self) -> bool {
        self.next().unwrap() % 2 == 0
    }
}

impl Iterator for Rng {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        self.hasher.write_u64(self.data);
        self.data = self.hasher.finish();
        Some(self.data.to_be_bytes().iter().fold(0, |a, b| a^b))
    }
}
