use crate::Rng;
use crate::Case;

pub trait Shrink: Case {
    #[allow(unused_variables)]
    fn shrink(self, rng: Rng) -> Option<Self> {
        None
    }
}

pub struct Shrink<T> {
    possibilities: fn(T) -> Iterator<Item=T>,
}

impl<S, T> Iterator for Gen<S, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next_case())
    }
}

impl Shrink for usize {
    fn shrink(self, mut rng: Rng) -> Option<Self> {
        if self == 0 { return None }
        if rng.next_bool() {
            repeat_with(|| rng.next_usize_max(self - 1))
                .next()
        } else {
            Some(self - 1)
        }
    }

    fn score(&self) -> usize {
        *self
    }
}

impl<T: Shrink> Shrink for Vec<T> {
    fn shrink(mut self, mut rng: Rng) -> Option<Self> {
        if self.len() == 0 { return None; }
        let index = rng.next_usize_max(self.len() - 1);
        if rng.next_bool() {
            self.remove(index);
        } else if let Some(smaller) = self[index].clone()
            .shrink(Rng::new_with_seed(rng.next_usize() as u64))
        {
            self[index] = smaller;
        }

        Some(self)
    }
}
