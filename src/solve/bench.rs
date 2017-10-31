//! Benchmarking tests.

#![cfg(test)]

extern crate test;
use self::test::Bencher;

use chalk_parse;
use errors::*;
use ir;
use lower::*;
use solve::SolverChoice;
use std::sync::Arc;

fn parse_and_lower_program(text: &str, solver_choice: SolverChoice) -> Result<ir::Program> {
    chalk_parse::parse_program(text)?.lower(solver_choice)
}

fn parse_and_lower_goal(program: &ir::Program, text: &str) -> Result<Box<ir::Goal>> {
    chalk_parse::parse_goal(text)?.lower(program)
}

fn run_bench(
    program_text: &str,
    solver_choice: SolverChoice,
    goal_text: &str,
    bencher: &mut Bencher,
    expected: &str,
) {
    let program = Arc::new(parse_and_lower_program(program_text, solver_choice).unwrap());
    let env = Arc::new(program.environment());
    ir::set_current_program(&program, || {
        let goal = parse_and_lower_goal(&program, goal_text).unwrap();
        let peeled_goal = goal.into_peeled_goal();

        // Execute once to get an expected result.
        let result = match solver_choice.solve_root_goal(&env, &peeled_goal) {
            Ok(Some(v)) => format!("{}", v),
            Ok(None) => format!("No possible solution"),
            Err(e) => format!("{}", e),
        };

        let expected1: String = expected.chars().filter(|w| !w.is_whitespace()).collect();
        let result1: String = result.chars().filter(|w| !w.is_whitespace()).collect();
        assert!(!expected1.is_empty() && result1.starts_with(&expected1));

        bencher.iter(|| solver_choice.solve_root_goal(&env, &peeled_goal));
    });
}

const CYCLEY: &str = "
trait AsRef<T> { }
trait Clone { }
trait Copy where Self: Clone { }
trait Sized { }

struct i32 { }
impl Copy for i32 { }
impl Clone for i32 { }
impl Sized for i32 { }

struct u32 { }
impl Copy for u32 { }
impl Clone for u32 { }
impl Sized for u32 { }

struct Rc<T> { }
impl<T> Clone for Rc<T> { }
impl<T> Sized for Rc<T> { }

struct Box<T> { }
impl<T> AsRef<T> for Box<T> where T: Sized { }
impl<T> Clone for Box<T> where T: Clone { }
impl<T> Sized for Box<T> { }

// Meant to be [T]
struct Slice<T> where T: Sized { }
impl<T> Sized for Slice<T> { }
impl<T> AsRef<Slice<T>> for Slice<T> where T: Sized { }

struct Vec<T> where T: Sized { }
impl<T> AsRef<Slice<T>> for Vec<T> where T: Sized { }
impl<T> AsRef<Vec<T>> for Vec<T> where T: Sized { }
impl<T> Clone for Vec<T> where T: Clone, T: Sized { }
impl<T> Sized for Vec<T> where T: Sized { }

trait SliceExt
  where <Self as SliceExt>::Item: Clone
{
  type Item;
}

impl<T> SliceExt for Slice<T>
  where T: Clone
{
  type Item = T;
}
";

const CYCLEY_GOAL: &str = "
forall<T> {
    if (
        <Slice<T> as SliceExt>::Item: Clone,
        <Slice<T> as SliceExt>::Item: Sized,
        T: Clone,
        T: Sized
    ) {
        T: Sized
    }
}
";

#[bench]
fn cycley_recursive_cached(b: &mut Bencher) {
    run_bench(
        CYCLEY,
        SolverChoice::Recursive {
            overflow_depth: 20,
            caching_enabled: true,
        },
        CYCLEY_GOAL,
        b,
        "Unique",
    );
}

#[bench]
fn cycley_recursive_uncached(b: &mut Bencher) {
    run_bench(
        CYCLEY,
        SolverChoice::Recursive {
            overflow_depth: 20,
            caching_enabled: false,
        },
        CYCLEY_GOAL,
        b,
        "Unique",
    );
}

#[bench]
fn cycley_slg(b: &mut Bencher) {
    run_bench(
        CYCLEY,
        SolverChoice::SLG {
            max_size: 20,
        },
        CYCLEY_GOAL,
        b,
        "No possible solution", // FIXME
    );
}
