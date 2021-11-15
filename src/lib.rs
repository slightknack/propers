use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe, set_hook, take_hook};
use std::collections::{HashMap, VecDeque};
use std::iter::Iterator;

pub mod traits;
pub use traits::*;
pub mod rng;
pub use rng::Rng;
pub mod gen;

pub const NUM_TESTS: usize = 100;
pub const SHRINK_BUDGET: usize = 1000;

pub fn run_case<T: Case, R: TestResult>(
    case: T,
    run: &impl Fn(T) -> R,
) -> Result<(), Box<dyn FnOnce() -> ()>> {
    // install a silent hook...
    let prev_hook = take_hook();
    set_hook(Box::new(|_| {}));
    // run the test case
    let result = catch_unwind(AssertUnwindSafe(|| run(case.clone())));
    // restore the old hook
    set_hook(prev_hook);

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
    cache: &mut HashMap<T, bool>,
) -> T {
    eprintln!("Reducing case, max {} steps ", SHRINK_BUDGET);

    let mut times = 0;
    let mut best = Scored(case.clone());
    let mut potentials: VecDeque<Scored<T>> = VecDeque::new();
    // let mut seen: BTreeSet<Scored<T>> = BTreeSet::new();

    for test_num in 0..SHRINK_BUDGET {
        let case = match potentials.pop_front() {
            None => match best.0.clone().shrink(Rng::new_with_seed(test_num as u64)) {
                None => break,
                Some(case) => Scored(case),
            },
            Some(case) => case,
        };

        if run_case(case.0.clone(), run, cache).is_ok() {
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

        if let Some(maybe_better) = case.0.shrink(Rng::new_with_seed(test_num as u64)) {
            potentials.push_back(Scored(maybe_better));
        }
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

        let mut cache = HashMap::new();
        let failure = match run_case(case.clone(), run, &mut cache) {
            Ok(_) => {
                eprint!(".");
                continue;
            },
            Err(f) => f,
        };

        // The test failed, handle it!
        eprintln!("!");
        eprintln!("Failed on test {}:\n\n{:#?}\n", test_num, case);
        shrink_case(case, run, &mut cache);
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

    fn quicksort(list: Vec<usize>) -> Vec<usize> {
        if list.len() <= 1 { return list }
        let pivot = list[0];
        let list = &list[1..];
        let mut lo = quicksort(list.iter().filter(|x| x < &&pivot).map(Clone::clone).collect());
        let mut hi = quicksort(list.iter().filter(|x| x >= &&pivot).map(Clone::clone).collect());
        lo.push(pivot);
        lo.append(&mut hi);
        return lo
    }

    #[test]
    fn is_sorted() {
        verify(
            gen::growing_list(gen::smart_usize(gen::random_usize())),
            &|case| {
                let sorted = quicksort(case.clone());
                assert_eq!(sorted.len(), case.len());
                if sorted.len() == 0 { return }
                for item in 0..sorted.len() - 1 {
                    assert!(sorted[item] <= sorted[item + 1])
                }
            }
        )
    }
}
