use std::panic::UnwindSafe;
use std::fmt::Debug;
use std::cmp::{PartialOrd, Ord, Ordering};
use crate::Rng;

/// A type to be used in different test cases
pub trait Case: Debug + Clone {}
impl<T> Case for T where Self: Debug + Clone {}

/// A trait that tries to simplify a failing test case
pub trait Shrink: Case {
    // TODO: how many reductions?
    /// Reduces self, must return a finite number of test cases.
    fn shrink(self, rng: &mut Rng) -> Box<dyn Iterator<Item = Self>>;

    // length is a good proxy for complexity
    fn score(&self) -> isize {
        format!("{:?}", self).len() as isize
    }
}

impl Shrink for usize {
    fn shrink(self, rng: &mut Rng) -> Box<dyn Iterator<Item = usize>> {
        rng.next();
        Box::new((0..self).rev())
    }

    fn score(&self) -> isize {
        (*self).try_into().unwrap_or(isize::MAX)
    }
}

impl<T: Case> Shrink for Vec<T> {
    fn shrink(self, rng: &mut Rng) -> Box<dyn Iterator<Item = Self>> {
        Box::new((0..10).map(|_| {
            let new = self.to_owned();
            let index = rng.next_usize() % self.len();
            new.remove(index);
            new
        }))
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
