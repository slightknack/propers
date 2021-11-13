use crate::Rng;
use crate::Case;

pub struct Gen<S, T> {
    item: S,
    step: fn(&mut S) -> T,
}

impl<S, T> Gen<S, T> {
    fn next_case(&mut self) -> T {
        (self.step)(&mut self.item)
    }
}

impl<S, T> Iterator for Gen<S, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next_case())
    }
}

pub fn increasing_usize() -> Gen<usize, usize> {
    Gen {
        item: 0,
        step: |old| {
            *old += 1;
            *old - 1
        }
    }
}

pub fn random_usize() -> Gen<Rng, usize> {
    Gen {
        item: Rng::new(),
        step: |rng| rng.next_usize()
    }
}

pub fn smart_usize<S>(and_then: Gen<S, usize>) -> Gen<(usize, Gen<S, usize>), usize> {
    Gen {
        item: (0, and_then),
        step: |(case, and_then)| {
            let cases = &[0, 1, 0, 2, usize::MAX, 3, 4, 5, 6, 7];
            let result = match cases.get(*case) {
                Some(c) => *c,
                None => and_then.next_case(),
            };

            *case += 1;
            result
        }
    }
}

pub fn growing_list<S, T: Case>(gen: Gen<S, T>) -> Gen<(Vec<T>, Gen<S, T>), Vec<T>> {
    Gen {
        item: (vec![], gen),
        step: move |(old, gen)| {
            old.push(gen.next_case());
            old.clone()
        }
    }
}
