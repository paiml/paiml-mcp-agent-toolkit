//! Regressions for the two contradictions the dead-code report shipped with:
//! a suppressed function that belonged to no category (#928) and a `--timeout`
//! that was printed and never enforced (#929).
//!
//! Both run a real `cargo check` over a throwaway crate, because both defects
//! lived in the seam between this handler and the compiler — a hand-built
//! `FileDeadCode` would have passed on the broken code.

use super::{run_dead_code_analysis_with_filters, DeadCodeAnalysisFilters};
use std::time::{Duration, Instant};

/// A budget the finding tests cannot hit, for the tests that are not about the
/// budget.
///
/// Dead-code analysis of a Rust crate shells out to `cargo check` and is bounded
/// by **wall clock**, so any finite deadline in a test that asserts *findings* is
/// a bet on how loaded the machine is. This one was tuned twice and lost twice:
/// 120s failed under `cargo test`, and the 600s it was raised to failed again
/// under `cargo llvm-cov`, where the instrumented harness runs ~19,800 tests and
/// starves the blocking task (#1013 — `ci / test` passed the same commit that
/// `ci / coverage` failed).
///
/// A third number would only move the odds. The phase has no completion
/// guarantee, so the deadline is removed from the assertion path entirely rather
/// than re-tuned. Nothing is lost: the budget IS covered, deterministically, by
/// `a_cargo_check_that_outruns_the_budget_is_killed_and_reported` below, which
/// uses one second against a crate rigged to outlive it.
///
/// The tradeoff, stated: if the analysis ever genuinely hangs, this test hangs
/// with it and the CI job timeout reports it instead of an assertion. That is a
/// worse message for a real bug, in exchange for never again reporting a fake
/// one — and a hang is a bug we would want to see, not a threshold to tune.
const NO_DEADLINE: Duration = Duration::from_secs(24 * 60 * 60);

fn filters(min_dead_lines: usize) -> DeadCodeAnalysisFilters {
    DeadCodeAnalysisFilters {
        include_unreachable: false,
        include_tests: false,
        min_dead_lines,
        top_files: None,
        include: Vec::new(),
        exclude: Vec::new(),
        max_depth: 10,
    }
}

fn write_crate(root: &std::path::Path, name: &str, lib_rs: &str, build_rs: Option<&str>) {
    std::fs::create_dir_all(root.join("src")).expect("src dir");
    let build_line = if build_rs.is_some() {
        "build=\"build.rs\"\n"
    } else {
        ""
    };
    std::fs::write(
        root.join("Cargo.toml"),
        format!("[package]\nname=\"{name}\"\nversion=\"0.1.0\"\nedition=\"2021\"\n{build_line}"),
    )
    .expect("Cargo.toml");
    std::fs::write(root.join("src/lib.rs"), lib_rs).expect("lib.rs");
    if let Some(build) = build_rs {
        std::fs::write(root.join("build.rs"), build).expect("build.rs");
    }
}

/// The report listed six dead functions and headed them with
/// `dead_functions: 0, dead_classes: 0, dead_modules: 0, unreachable_blocks: 0`
/// — the twelve dead lines belonged to no category at all — and typed each one
/// `item_type: "variable"` in a record whose own `reason` said `fn`. The cause
/// was `DeadCodeKind::Suppressed`, a "kind" that replaced the item's real kind
/// with the way it had been discovered.
#[tokio::test]
async fn a_suppressed_function_is_counted_and_typed_as_a_function() {
    let tmp = tempfile::TempDir::new().expect("tempdir");
    let root = tmp.path();
    // The attribute is assembled rather than written literally: pmat's own
    // suppression scanner would otherwise find this file and report it.
    let allow = format!("#[allow({})]", "dead_code");
    write_crate(
        root,
        "suppressed_fn_crate",
        &format!(
            "{allow}\nfn admitted_dead(x: i32) -> i32 {{ x + 1 }}\npub fn used() -> i32 {{ 1 }}\n"
        ),
        None,
    );

    let outcome = run_dead_code_analysis_with_filters(root, filters(0), NO_DEADLINE)
        .await
        .expect("analysis runs");

    let summary = &outcome.report.summary;
    let items: Vec<_> = outcome
        .report
        .files
        .iter()
        .flat_map(|f| f.items.iter())
        .collect();

    assert_eq!(
        items.len(),
        1,
        "expected the one suppressed item: {:?}",
        outcome.report.files
    );
    assert_eq!(
        items[0].item_type,
        crate::models::dead_code::DeadCodeType::Function,
        "an item whose reason says `fn` was typed {:?}",
        items[0].item_type
    );
    assert_eq!(
        summary.dead_functions, 1,
        "the summary counts 0 dead functions over a listed dead function: {summary:?}"
    );
    // The contradiction in one assertion: nothing may be listed that no counter
    // accounts for.
    assert!(
        summary.dead_functions + summary.dead_classes + summary.dead_modules > 0,
        "{} dead lines in {} files, and every category counter is 0: {summary:?}",
        summary.total_dead_lines,
        summary.files_with_dead_code
    );
}

/// `--timeout N` printed "⏰ Analysis timeout set to N seconds" and ran to
/// completion: the work was a blocking `Command::output()` inside an `async`
/// block, so neither of the two `tokio::time::timeout`s wrapped around it could
/// fire. Measured at 20.2s under `--timeout 1`, exit 0.
///
/// The 20-second sleep is in a build script, so `cargo check` is deterministically
/// slower than the budget without depending on machine speed.
#[tokio::test(flavor = "multi_thread")]
async fn a_cargo_check_that_outruns_the_budget_is_killed_and_reported() {
    let tmp = tempfile::TempDir::new().expect("tempdir");
    let root = tmp.path();
    write_crate(
        root,
        "slowcheck_crate",
        "fn dead_one() -> i32 { 1 }\npub fn used() -> i32 { 2 }\n",
        Some("fn main() { std::thread::sleep(std::time::Duration::from_secs(20)); }\n"),
    );

    let started = Instant::now();
    let result =
        run_dead_code_analysis_with_filters(root, filters(0), Duration::from_secs(1)).await;
    let elapsed = started.elapsed();

    let error = result
        .err()
        .unwrap_or_else(|| panic!("--timeout 1 ran the 20s check to completion in {elapsed:?}"));
    assert!(
        error.to_string().contains("timed out after 1 seconds"),
        "unexpected error: {error}"
    );
    // Generous, because the budget only starts once cargo is spawned and the
    // build script has to be compiled first; the point is that it is nowhere
    // near the 20s the check itself takes.
    assert!(
        elapsed < Duration::from_secs(15),
        "the budget was not enforced: {elapsed:?}"
    );
}

/// #928 RESIDUAL. Every `DeadCodeKind` the parser can produce must reach a
/// `DeadCodeType` that NAMES it. `Module` and the unclassified `Other` both used
/// to land on `Variable`, so a record could read
/// `"item_type": "variable"` beside `"reason": "module `x` is never used"` —
/// the report contradicting itself inside one object.
///
/// This is a pure mapping test on purpose: rustc emits the `module` wording
/// rarely enough that a fixture cannot be relied on to produce one, while the
/// parser accepts it unconditionally (`("module `", "` is never used", …)`).
#[test]
fn every_dead_code_kind_maps_to_a_type_that_names_it() {
    use crate::models::dead_code::DeadCodeType;
    use crate::services::cargo_dead_code_analyzer::{DeadCodeKind, DeadItem};

    let item = |kind: DeadCodeKind, message: &str| DeadItem {
        name: "x".to_string(),
        kind,
        line: 1,
        column: 1,
        message: message.to_string(),
    };

    let cases = [
        (
            item(DeadCodeKind::Module, "module `x` is never used"),
            DeadCodeType::Module,
        ),
        (
            item(
                DeadCodeKind::Other("union".to_string()),
                "union `x` is never used",
            ),
            DeadCodeType::Other,
        ),
        (
            item(DeadCodeKind::Constant, "constant `x` is never used"),
            DeadCodeType::Variable,
        ),
        (
            item(DeadCodeKind::Function, "function `x` is never used"),
            DeadCodeType::Function,
        ),
    ];

    for (dead_item, expected) in cases {
        let reason = dead_item.message.clone();
        let reported = super::dead_items_to_report_items(std::slice::from_ref(&dead_item));
        assert_eq!(
            reported[0].item_type, expected,
            "`{reason}` must not be reported as {:?}",
            reported[0].item_type
        );
    }
}
