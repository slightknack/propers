use std::panic::{set_hook, catch_unwind, resume_unwind, UnwindSafe};
use std::fmt::Debug;
use std::collections::BinaryHeap;
use std::cmp::{Ord, Ordering};

/// A type to be used in different test cases
pub trait Case: Debug + Clone + UnwindSafe {}
impl<T> Case for T where Self: Debug + Clone + UnwindSafe {}

/// A trait that generates test cases
pub trait Faux<T: Case>: Default { fn faux(&mut self) -> T; }
/// A trait that tries to simplify a failing test case
pub trait Shrink: Case {
    fn shrink(self) -> Box<dyn Iterator<Item = Self>>;

    // length is a good proxy for complexity
    fn score(&self) -> isize {
        format!("{:?}", self).len() as isize
    }
}

pub trait  TestResult: Debug + UnwindSafe { fn is_ok(&self) -> bool; }
impl<T, E> TestResult for Result<T, E> where Self: Debug + UnwindSafe { fn is_ok(&self) -> bool { self.is_ok()   } }
impl<T>    TestResult for Option<T>    where Self: Debug + UnwindSafe { fn is_ok(&self) -> bool { self.is_some() } }

pub const NUM_TESTS: usize = 100;
pub const SHRINK_BUDGET: usize = 1000;

pub fn run_case<T: Case, R: TestResult>(
    case: T,
    run: &fn(T) -> R
) -> Result<(), Box<dyn FnOnce() -> ()>> {
    let result = catch_unwind(move || run(case));
    match result {
        Ok(r) if r.is_ok() => Ok(()),
        Ok(e)  => {
            let formatted = format!("Provided test result: {:#?}", e);
            Err(Box::new(move || { panic!("{}", formatted); }))
        },
        Err(e) => Err(Box::new(move || { resume_unwind(e); })),
    }
}

struct Scored<T: Shrink>(T);

impl Cmp for Scored {
    fn cmp(&self, other: &Self) -> Ordering {

    }
}

fn shrink_case<T: Shrink, R: TestResult>(
    case: T,
    run: &fn(T) -> R
) -> T {
    let smallest = (case.clone(), case.score());
    let mut queue: BinaryHeap<T> = case.shrink().collect();

    for trial in 0..SHRINK_BUDGET {
        let case = queue.
    }

    todo!()
}

pub fn test<T: Shrink, R: TestResult>(
    gen: &mut impl Faux<T>,
    run: &fn(T) -> R,
) {
    eprintln!("Running {} tests: ", NUM_TESTS);

    for test_num in 0..NUM_TESTS {
        eprint!("?");
        let case = gen.faux();
        let case_copy = case.clone();

        let failure = match run_case(case, run) {
            Ok(_) => {
                eprint!("\u{8}.");
                continue;
            },
            Err(f) => f,
        };

        // The test failed, handle it!
        eprintln!("!\n");
        eprintln!("Failed on test #{}:\n\n{:#?}\n", test_num, case_copy);
        eprintln!("Reducing the failing case:");


    }

    todo!()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
