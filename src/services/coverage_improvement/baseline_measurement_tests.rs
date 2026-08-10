//! Regression tests for the improvement loop's stop conditions.
//!
//! `baseline_measurement.rs` is `include!`d into `mod.rs`, so its tests cannot
//! live at the end of that file (the includes after it would become items after
//! a test module).

use super::{zero_progress_stop_reason, IterationReport};
use std::path::PathBuf;

fn report(iteration: usize, tests_generated: usize, coverage_gain: f64) -> IterationReport {
    IterationReport {
        iteration,
        files_targeted: vec![PathBuf::from("src/lib.rs")],
        tests_generated,
        coverage_gain,
        mutation_score: f64::NAN,
    }
}

/// The reported defect: an iteration that generated nothing and gained nothing
/// was repeated `--max-iterations` times with byte-identical results (and a
/// full `make coverage` run each time).
#[test]
fn a_zero_progress_iteration_stops_the_loop() {
    let reason = zero_progress_stop_reason(&report(1, 0, 0.0))
        .expect("0 tests + 0.00% gain must stop the loop");
    assert!(
        reason.contains("No progress in iteration 1"),
        "stop reason must name the stalled iteration: {reason}"
    );
    assert!(
        reason.contains("0 tests generated"),
        "stop reason must state what it observed: {reason}"
    );
}

/// A coverage REGRESSION is also no progress — repeating it cannot help.
#[test]
fn a_negative_gain_iteration_stops_the_loop() {
    assert!(zero_progress_stop_reason(&report(2, 0, -0.5)).is_some());
}

/// Anything that moved: keep going.
#[test]
fn progress_does_not_stop_the_loop() {
    assert!(
        zero_progress_stop_reason(&report(1, 0, 1.5)).is_none(),
        "coverage rose, so the next iteration is not a repeat"
    );
    assert!(
        zero_progress_stop_reason(&report(1, 3, 0.0)).is_none(),
        "tests were written; the next coverage run has new input"
    );
}
