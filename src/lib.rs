use std::panic::{catch_unwind, resume_unwind, UnwindSafe};
use std::fmt::Debug;
use std::collections::BinaryHeap;
use std::cmp::{PartialOrd, Ord, Ordering};

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

#[derive(Debug, Clone)]
struct Scored<T: Shrink>(T);

impl<T: Shrink> Eq        for Scored<T> {}
impl<T: Shrink> PartialEq for Scored<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.score().eq(&other.0.score())
    }
}

impl<T: Shrink> Ord for Scored<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.score().cmp(&other.0.score())
    }
}

impl<T: Shrink> PartialOrd for Scored<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

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


fn shrink_case<T: Shrink, R: TestResult>(
    case: T,
    run: &fn(T) -> R
) -> T {
    eprintln!("Reducing case, max {} steps: ", SHRINK_BUDGET);

    let mut times = 0;
    let mut smallest = Scored(case.clone());
    let mut potentials: BinaryHeap<Scored<T>> = case.shrink().map(|x| Scored(x)).collect();

    for test_num in 0..SHRINK_BUDGET {
        eprint!("?");

        let case = match potentials.pop() {
            None => {
                println!("\nExhausted sampled possibilities");
                break;
            },
            Some(case) => case,
        };

        if run_case(case.0.clone(), run).is_ok() {
            eprint!("\u{8}o");
            // we don't add the passing case to the reduction set
            continue;
        } if case < smallest {
            times += 1;
            smallest = case.clone();
            eprintln!("\u{8}!\n");
            eprintln!("Shrunk case to score {} on test #{}:\n\n{:#?}\n", test_num, case.0.score(), case.0);
        } else {
            eprint!("\u{8}.");
        }

        let new_potentials = &mut case.0.shrink().map(|x| Scored(x)).collect();
        potentials.append(new_potentials);
    }

    eprintln!("\n\nExhausted reduction budget, reduced {} times\n", times);
    return smallest.0;
}

pub fn test<T: Shrink, R: TestResult>(
    gen: &mut impl Faux<T>,
    run: &fn(T) -> R,
) {
    eprintln!("Running {} tests: ", NUM_TESTS);

    for test_num in 0..NUM_TESTS {
        eprint!("?");
        let case = gen.faux();

        let failure = match run_case(case.clone(), run) {
            Ok(_) => {
                eprint!("\u{8}.");
                continue;
            },
            Err(f) => f,
        };

        // The test failed, handle it!
        eprintln!("!\n");
        eprintln!("Failed on test #{}:\n\n{:#?}\n", test_num, case);
        let reduced = shrink_case(case, run);
        eprintln!("Smallest failing case:\n\n{:#?}\n", reduced);
        eprintln!("Happy debugging!");

        // raise the panic
        failure();
    }

    eprintln!(" :)\n");
    eprintln!("All tests passed");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
