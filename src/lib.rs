use std::panic::{catch_unwind, resume_unwind, UnwindSafe, AssertUnwindSafe};
use std::collections::{BTreeMap, BinaryHeap, VecDeque};
use std::iter::Iterator;

pub mod traits;
pub use traits::*;
pub mod rng;
pub use rng::Rng;

pub const NUM_TESTS: usize = 100;
pub const SHRINK_BUDGET: usize = 1000;

pub fn run_case<T: Case, R: TestResult>(
    case: T,
    run: &impl Fn(T) -> R,
) -> Result<(), Box<dyn FnOnce() -> ()>> {
    let result = catch_unwind(AssertUnwindSafe(|| run(case)));

    match result {
        Ok(r) if r.is_ok() => Ok(()),
        Ok(e)  => {
            let formatted = format!("Failed with test result: {:#?}", e);
            Err(Box::new(move || { panic!("{}", formatted); }))
        },
        Err(e) => Err(Box::new(move || { resume_unwind(e); })),
    }
}

fn shrink_case<T: Shrink, R: TestResult>(
    case: T,
    run: &impl Fn(T) -> R,
) -> T {
    eprintln!("Reducing case, max {} steps ", SHRINK_BUDGET);

    let mut times = 0;
    let mut rng = Rng::new();
    let mut best = Scored(case.clone());
    let mut potentials: VecDeque<Scored<T>>
        = case.shrink(&mut rng).map(|x| Scored(x)).collect();

    for test_num in 0..SHRINK_BUDGET {
        let case = match potentials.pop_front() {
            None => break,
            Some(case) => case,
        };

        if run_case(case.0.clone(), run).is_ok() {
            eprint!(".");
            // we don't add the passing case to the reduction set
            continue;
        } if case > best {
            times += 1;
            best = case.clone();
            eprint!("!\n{:?} ", case.0);
        } else {
            eprint!(",");
        }

        let new_potentials = &mut case.0.shrink(&mut rng).map(|x| Scored(x)).collect();
        potentials.append(new_potentials);
    }

    eprintln!("\nReduced {} times, smallest failing case:\n\n{:#?}", times, best.0);
    return best.0;
}

pub fn verify<T: Shrink, R: TestResult, I: Iterator<Item = T>>(
    mut strat: I,
    run: &impl Fn(T) -> R,
) {
    eprintln!("Running max {} tests", NUM_TESTS);

    for test_num in 0..NUM_TESTS {
        let case = match strat.next() {
            Some(case) => case,
            None => {
                eprintln!(" :D");
                eprintln!("Exhausted all {} test cases, everything passed", test_num);
                eprintln!("(Just make sure you're generating everything!)");
                return;
            }
        };

        let failure = match run_case(case.clone(), run) {
            Ok(_) => {
                eprint!(".");
                continue;
            },
            Err(f) => f,
        };

        // The test failed, handle it!
        eprintln!("!");
        eprintln!("Failed on test {}:\n\n{:#?}\n", test_num, case);
        shrink_case(case, run);
        eprintln!("\nHappy debugging!\n");

        // raise the panic
        failure();
    }

    eprintln!(" :)");
    eprintln!("All tests passed");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        verify(0..100, &|case| {
            case + case != 40
        })
    }

    fn quicksort(list: Vec<usize>) -> Vec<usize> {
        return vec![];
    }

    #[test]
    fn is_sorted() {
        verify(
            [vec![1, 2, 3]].iter(),
            &|case| {
                let sorted = quicksort(case.to_owned());
                assert_eq!(sorted.len(), case.len());
                if sorted.len() == 0 { return }
                for item in 0..sorted.len() - 1 {
                    assert!(sorted[item] < sorted[item + 1])
                }
            }
        )
    }
}
