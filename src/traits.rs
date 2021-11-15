use std::fmt::Debug;
use std::hash::Hash;
use std::cmp::{PartialOrd, Ord, Ordering};
use std::iter::repeat_with;
use crate::Rng;

/// A type to be used in different test cases
pub trait Case: Debug + Clone {}
impl<T> Case for T where Self: Debug + Clone {}

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

// TODO: use `Try` trait once stabilized
/// Represents whether a returned result is success or failure.
/// For example, `Result::Err` and `Option::None` indicate failing cases.
pub trait  TestResult: Debug { fn is_ok(&self) -> bool; }

macro_rules! test_result {
    ($name:ty, $s:ident, $body:expr) => {
        impl $crate::TestResult for $name where Self: Debug {
            fn is_ok(&$s) -> bool { $body }
        }
    };
    ($name:ty, $s:ident, $body:expr, $($args:tt)*) => {
        impl<$($args)*> $crate::TestResult for $name where Self: Debug {
            fn is_ok(&$s) -> bool { $body }
        }
    };
}

test_result!{ Result<T, E>, self, self.is_ok(), T, E }
test_result!{ Option<T>, self, self.is_some(), T }
test_result!{ bool, self, *self }
test_result!{ (), self, true }
