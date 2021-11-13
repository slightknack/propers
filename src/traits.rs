use std::fmt::Debug;
use std::hash::Hash;
use std::cmp::{PartialOrd, Ord, Ordering};
use std::iter::repeat_with;
use crate::Rng;

/// A type to be used in different test cases
pub trait Case: Debug + Clone + Eq + Hash {}
impl<T> Case for T where Self: Debug + Clone + Eq + Hash {}

/// A trait that tries to simplify a failing test case
pub trait Shrink: Case {
    #[allow(unused_variables)]
    fn shrink(self, rng: Rng) -> Option<Self> {
        None
    }

    // length is a good proxy for complexity
    fn score(&self) -> usize {
        format!("{:?}", self).len()
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

// TODO: use `Try` trait once stabilized
/// Represents whether a returned result is success or failure.
/// For example, `Result::Err` and `Option::None` indicate failing cases.
pub trait  TestResult: Debug { fn is_ok(&self) -> bool; }
impl<T, E> TestResult for Result<T, E> where Self: Debug { fn is_ok(&self) -> bool { self.is_ok()   } }
impl<T>    TestResult for Option<T>    where Self: Debug { fn is_ok(&self) -> bool { self.is_some() } }
impl       TestResult for bool         where Self: Debug { fn is_ok(&self) -> bool { *self          } }
impl       TestResult for ()           where Self: Debug { fn is_ok(&self) -> bool { true           } }

/// Represents a scored test case.
#[derive(Debug, Clone)]
pub(crate) struct Scored<T: Shrink>(pub T);

impl<T: Shrink> Eq        for Scored<T> {}
impl<T: Shrink> PartialEq for Scored<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.score().eq(&other.0.score())
    }
}

impl<T: Shrink> Ord for Scored<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.score().cmp(&other.0.score()).reverse()
    }
}

impl<T: Shrink> PartialOrd for Scored<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
