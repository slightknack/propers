use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

pub const RNG_SEED: u64 = 13157594558721065924;

pub struct Rng {
    data:   u64,
    hasher: DefaultHasher,
}

impl Rng {
    pub fn new() -> Rng {
        Rng {
            data:   RNG_SEED,
            hasher: DefaultHasher::new(),
        }
    }

    pub fn next_usize(&mut self) -> usize {
        let mut out = 0;
        for i in self.take(std::mem::size_of::<usize>()) {
            out = out << 8 | i as usize;
        }
        out
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
