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

    pub fn next_u64_max(&mut self, max: u64) -> u64 {
        let mask = 2_u64.pow(64 - (max as u64).leading_zeros()) - 1;
        let mut out = self.next_u64() & mask;
        while out > max { out = self.next_u64() & mask }
        out
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut out = 0;
        for i in self.take(8) {
            out = out << 8 | i as u64;
        }
        out
    }

    pub fn next_byte(&mut self) -> u8 {
        self.next().unwrap()
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
