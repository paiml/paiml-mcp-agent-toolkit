//! Every surface of one run must describe that run.
//!
//! `pmat enforce extreme` answered the same question three ways at once:
//!
//! ```text
//! $ pmat enforce extreme -p empty --list-violations   # Found 0 violations, exit 0
//! $ pmat enforce extreme -p empty --ci-mode           # Violations: 2, exit 1
//! $ pmat enforce extreme -p empty --format json       # "state":"VIOLATING", 2 violations
//! ```
//!
//! `--list-violations` ran its own analysis pipeline that kept only
//! `PhaseOutcome::violations` and dropped every `unmeasured` disclosure, so a
//! directory where nothing could be measured listed as clean and exited 0 while
//! the state machine failed the same directory. The measurement now happens in
//! one place; these tests fail if a second one ever appears.

use super::assessment::{assess_project, phase_score};
use super::config::EnforcementConfig;
use super::enforcement::{execute_main_loop, handle_special_modes, run_enforcement_step};
use super::output::format_violations_output;
use super::states::handle_analyzing_state;
use super::types::{EnforcementState, QualityProfile, QualityViolation};
use crate::cli::EnforceOutputFormat;
use std::path::{Path, PathBuf};

/// An empty-but-valid Rust crate: a manifest, an empty `src/`. Nothing to
/// measure, and therefore nothing measured — the corpus from the bug report.
fn empty_but_valid_crate(dir: &Path) {
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("manifest");
    std::fs::create_dir_all(dir.join("src")).expect("mkdir src");
}

/// Violation types, sorted — the comparable shape of a violation list.
fn types_of(violations: &[serde_json::Value]) -> Vec<String> {
    let mut types: Vec<String> = violations
        .iter()
        .map(|v| v["violation_type"].as_str().unwrap_or("?").to_string())
        .collect();
    types.sort();
    types
}

/// Run one special mode and return the JSON it wrote to `-o`.
async fn special_mode_json(
    list_violations: bool,
    validate_only: bool,
    project: &Path,
    report: &Path,
) -> serde_json::Value {
    handle_special_modes(
        list_violations,
        validate_only,
        &project.to_path_buf(),
        &QualityProfile::default(),
        EnforceOutputFormat::Json,
        false, // ci_mode: process::exit is not testable in-process
        None,
        Some(report),
        None,
        None,
    )
    .await
    .expect("the mode runs")
    .expect("a special mode was requested")
    .expect("the report is emitted");

    serde_json::from_str(&std::fs::read_to_string(report).expect("-o wrote the report"))
        .expect("the report is JSON")
}

#[tokio::test]
async fn list_violations_and_the_state_machine_report_the_same_run() {
    let dir = tempfile::tempdir().expect("tempdir");
    empty_but_valid_crate(dir.path());

    // Surface 0: the assessment itself — the single source.
    let source = assess_project(dir.path(), &QualityProfile::default(), None, None, None)
        .await
        .expect("the assessment runs");
    assert!(
        source.any_unmeasured(),
        "fixture must have something that cannot be measured, or this test proves nothing: \
         {}/{} dimensions measured",
        source.measured_phases,
        source.total_phases
    );

    // Surface 1: --list-violations.
    let listed = special_mode_json(true, false, dir.path(), &dir.path().join("listed.json")).await;
    let listed_violations = listed["violations"]
        .as_array()
        .expect("the listing carries violations")
        .clone();

    // Surface 2: --validate-only / --format json / --ci-mode, i.e. the state
    // machine's `EnforcementResult`.
    let validated =
        special_mode_json(false, true, dir.path(), &dir.path().join("validated.json")).await;
    let validated_violations = validated["violations"]
        .as_array()
        .expect("the result carries violations")
        .clone();

    // Surface 3: the state machine called directly.
    let analyzed = handle_analyzing_state(
        dir.path(),
        &QualityProfile::default(),
        false,
        true,
        None,
        None,
        None,
    )
    .await
    .expect("the state machine runs");

    // `--list-violations` reported 0 here, with exit 0, while the other two
    // reported the not_measured disclosures and exited 1 under --ci-mode.
    assert_eq!(
        types_of(&listed_violations),
        types_of(&validated_violations),
        "--list-violations and --validate-only described different runs of the same directory: \
         {listed_violations:?} vs {validated_violations:?}"
    );
    assert_eq!(
        listed_violations.len(),
        analyzed.violations.len(),
        "--list-violations disagreed with the state machine: {} vs {}",
        listed_violations.len(),
        analyzed.violations.len()
    );
    assert_eq!(
        listed_violations.len(),
        source.violations.len(),
        "a surface invented or dropped violations on the way out of the assessment"
    );

    // The disclosures specifically — the payload that used to be dropped.
    let disclosed = types_of(&listed_violations)
        .into_iter()
        .filter(|t| t == "not_measured")
        .count();
    assert!(
        disclosed > 0,
        "the not_measured disclosures did not survive into --list-violations: {listed_violations:?}"
    );
    assert_eq!(
        disclosed,
        source.total_phases - source.measured_phases,
        "one disclosure per unmeasured dimension"
    );

    // And the verdict every surface's exit code is taken from.
    assert_eq!(validated["state"], "VIOLATING");
    assert_ne!(analyzed.state, EnforcementState::Complete);
    assert_eq!(source.verdict_state(), EnforcementState::Violating);

    // The JSON summary must add up to the list it summarises: `by_type` was a
    // fixed complexity/satd/tdg map, so disclosures were counted in `total` and
    // nowhere else.
    let by_type: usize = listed["summary"]["by_type"]
        .as_object()
        .expect("by_type is a map")
        .values()
        .map(|v| v.as_u64().unwrap_or(0) as usize)
        .sum();
    assert_eq!(
        by_type,
        listed_violations.len(),
        "the summary breakdown does not add up to the violations it summarises: {}",
        listed["summary"]
    );
}

#[tokio::test]
async fn list_violations_refuses_a_path_it_cannot_read() {
    // `--list-violations -p /nonexistent` printed `Found 0 violations`, exit 0,
    // for a path the state machine refuses to grade at all.
    let missing = PathBuf::from("/nonexistent/pmat/enforce/r04");
    let err = handle_special_modes(
        true,
        false,
        &missing,
        &QualityProfile::default(),
        EnforceOutputFormat::Json,
        false,
        None,
        None,
        None,
        None,
    )
    .await
    .expect("the mode runs")
    .expect("a special mode was requested")
    .expect_err("a path that cannot be read must not be listed as clean");

    assert!(
        err.to_string().contains("path not found"),
        "the refusal must name the cause: {err}"
    );
}

// ===========================================================================
// The same contradiction, on the three surfaces the first fix did not reach.
// ===========================================================================

/// A crate that violates something and carries an lcov report, so every
/// dimension is measured and the verdict is a real one.
fn measured_violating_crate(dir: &Path, lines_hit: u32, lines_found: u32) {
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("manifest");
    std::fs::create_dir_all(dir.join("src")).expect("mkdir src");
    std::fs::write(
        dir.join("src/lib.rs"),
        "// TODO: fix this later\npub fn add(a: i32, b: i32) -> i32 { a + b }\n",
    )
    .expect("source");
    std::fs::write(
        dir.join("lcov.info"),
        format!(
            "SF:src/lib.rs\nDA:1,{lines_hit}\nLF:{lines_found}\nLH:{lines_hit}\nend_of_record\n"
        ),
    )
    .expect("lcov");
}

fn coverage_violation(current: f64, target: f64) -> QualityViolation {
    QualityViolation {
        violation_type: "coverage".to_string(),
        severity: "high".to_string(),
        location: "project".to_string(),
        current,
        target,
        suggestion: "Increase test coverage".to_string(),
    }
}

/// `phase_score` opened with `if v.current <= v.target { 1.0 }`, which is the
/// right comparison for the five ceiling dimensions and exactly backwards for
/// coverage, whose threshold is a floor. Every coverage breach scored a full
/// mark, so a crate with 0% coverage against the extreme profile's 80% printed
/// `Score: 1.00/1.00` beside `Violations: 1`.
#[test]
fn a_coverage_breach_is_scored_from_below_not_above() {
    assert!(
        (phase_score(&[coverage_violation(0.0, 80.0)]) - 0.0).abs() < 1e-9,
        "0% against an 80% floor is a total breach, not a clean phase: {}",
        phase_score(&[coverage_violation(0.0, 80.0)])
    );
    assert!(
        (phase_score(&[coverage_violation(40.0, 80.0)]) - 0.5).abs() < 1e-9,
        "40% against an 80% floor is half the required evidence: {}",
        phase_score(&[coverage_violation(40.0, 80.0)])
    );
    // And the ceiling dimensions keep their orientation.
    assert!(
        (phase_score(&[QualityViolation {
            violation_type: "complexity".to_string(),
            severity: "high".to_string(),
            location: "a.rs:1:f".to_string(),
            current: 40.0,
            target: 10.0,
            suggestion: String::new(),
        }]) - 0.25)
            .abs()
            < 1e-9
    );
}

/// The same defect through the front door: no violation may cost a run nothing.
#[tokio::test]
async fn a_project_with_no_coverage_cannot_score_a_full_mark() {
    let dir = tempfile::tempdir().expect("tempdir");
    // Source that breaches nothing else, and an lcov report that says 0 of 10
    // lines were hit — measured, and measured as zero.
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("manifest");
    std::fs::create_dir_all(dir.path().join("src")).expect("mkdir src");
    std::fs::write(
        dir.path().join("src/lib.rs"),
        "pub fn add(a: i32, b: i32) -> i32 { a + b }\n",
    )
    .expect("source");
    std::fs::write(
        dir.path().join("lcov.info"),
        "SF:src/lib.rs\nDA:1,0\nLF:10\nLH:0\nend_of_record\n",
    )
    .expect("lcov");

    let assessment = assess_project(dir.path(), &QualityProfile::default(), None, None, None)
        .await
        .expect("the assessment runs");

    assert!(
        assessment
            .violations
            .iter()
            .any(|v| v.violation_type == "coverage"),
        "fixture must breach the coverage floor, or this test proves nothing: {:?}",
        assessment.violations
    );
    assert!(
        assessment.score < 1.0,
        "a run holding a coverage violation reported a perfect score of {}",
        assessment.score
    );
    assert_eq!(assessment.verdict_state(), EnforcementState::Violating);
}

/// The refactoring state was the third hardcoded violation list: it emptied the
/// list and added 0.1 to a score the caller passed as the literal `0.7`, after
/// a body that refactored nothing. `--apply-suggestions --format json` therefore
/// ended on `"state":"VALIDATING","score":0.8,"violations":[]` for a directory
/// every other surface reported as violating.
#[tokio::test]
async fn the_refactoring_state_reports_the_run_it_did_not_change() {
    let dir = tempfile::tempdir().expect("tempdir");
    measured_violating_crate(dir.path(), 1, 1);
    let profile = QualityProfile::default();

    let source = assess_project(dir.path(), &profile, None, None, None)
        .await
        .expect("the assessment runs");
    assert!(
        !source.violations.is_empty(),
        "fixture must violate something, or this test proves nothing"
    );

    let refactored = run_enforcement_step(
        &dir.path().to_path_buf(),
        &profile,
        EnforcementState::Refactoring,
        false,
        false,
        true,
        None,
        None,
        None,
    )
    .await
    .expect("the refactoring step runs");

    // #1013: this used to compare `refactored` against `source` field by field
    // — equal violation counts, equal scores. Both assertions are confounded by
    // a dimension neither test is about: dead-code analysis is bounded by wall
    // clock (it shells out to `cargo check`), so under load it lands in one of
    // these two runs and times out in the other, and the counts legitimately
    // differ. That is what reddened `ci / coverage` while `ci / test` passed on
    // the identical commit.
    //
    // The exact-equality property has NOT been dropped, it has been moved
    // somewhere it can be decided: `label_refactoring_pass` is now a pure
    // function and `the_refactoring_label_changes_no_number` asserts, over every
    // input state, that relabelling changes no number at all. That is a stronger
    // check than this one was — it covers `target` and every violation field,
    // not just the count — and it cannot flake.
    //
    // What is left here is the part only an end-to-end run can establish: that
    // the refactoring state is WIRED to the real measurement rather than to the
    // fabricator it replaced. The fabricator emptied the list and returned
    // `0.7 + 0.1`; both are excluded below without comparing the two runs.
    assert!(
        !refactored.violations.is_empty(),
        "the refactoring step reported an empty violation list for a tree the \
         assessment found violating — the fabricator is back: {:?}",
        refactored.violations
    );
    assert!(
        refactored.score < 1.0,
        "the refactoring step reported a perfect score for a violating tree: {}",
        refactored.score
    );
    assert_ne!(
        refactored.state,
        EnforcementState::Complete,
        "a violating tree was relabelled Complete by a step that changed nothing"
    );
}

/// And end to end: the last document a `--apply-suggestions --format json`
/// consumer parses must describe the same run as every other surface, and the
/// loop must not spin 100 times over a tree nothing edits. The no-progress
/// guard compared only the immediately preceding iteration, so the
/// Violating -> Refactoring -> Validating cycle walked past it forever.
#[tokio::test]
async fn apply_suggestions_agrees_with_every_other_surface_and_terminates() {
    let dir = tempfile::tempdir().expect("tempdir");
    measured_violating_crate(dir.path(), 1, 1);
    let profile = QualityProfile::default();
    let report = dir.path().join("iteration.json");

    let source = assess_project(dir.path(), &profile, None, None, None)
        .await
        .expect("the assessment runs");
    assert!(
        !source.violations.is_empty(),
        "fixture must violate something"
    );

    let config = EnforcementConfig {
        max_iterations: 100,
        target_improvement: None,
        max_time: None,
        apply_suggestions: true,
        specific_file: None,
        include_pattern: None,
        exclude_pattern: None,
        single_file_mode: false,
        dry_run: false,
        show_progress: false,
        format: EnforceOutputFormat::Json,
        ci_mode: false,
    };

    let result = execute_main_loop(
        &dir.path().to_path_buf(),
        &profile,
        &config,
        std::time::Instant::now(),
        Some(report.as_path()),
    )
    .await
    .expect("the loop reports");

    assert!(
        result.final_iteration <= 6,
        "no automated refactoring exists, so re-measuring the same tree {} times measures \
         nothing new",
        result.final_iteration
    );
    // #1013: `final_score == source.score` and `document.violations.len() ==
    // source.violations.len()` both compare two independent measurements of the
    // same tree, and the wall-clock-budgeted dead-code phase can land in one and
    // time out in the other. Exact agreement between a labelled result and the
    // measurement it wraps is pinned deterministically instead, by
    // `the_refactoring_label_changes_no_number` in `states.rs`.
    //
    // The end-to-end properties that only this test can establish are kept: the
    // loop terminates, it does not declare victory, and the document a consumer
    // actually parses agrees with the in-process result OF THE SAME RUN — which
    // is the contradiction the test was written for (`"state":"VALIDATING",
    // "score":0.8, "violations":[]` for a tree every other surface called
    // violating). Comparing the document to `result` rather than to `source`
    // compares one run with itself, so it is exact AND stable.
    assert_ne!(result.final_state, EnforcementState::Complete);
    assert!(
        result.final_score < 1.0,
        "the loop finished on a perfect score for a violating tree: {}",
        result.final_score
    );

    // The document on disk is the one a consumer reads.
    let last: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&report).expect("-o wrote the report"))
            .expect("the report is JSON");
    let documented = last["violations"]
        .as_array()
        .expect("the document carries violations")
        .len();
    assert!(
        documented > 0,
        "the final --apply-suggestions document reported no violations for a tree \
         the assessment found violating: {last}"
    );
    let documented_score = last["score"]
        .as_f64()
        .expect("the document carries a score");
    assert!(
        (documented_score - result.final_score).abs() < f64::EPSILON,
        "the document and the loop disagree about the SAME run: {documented_score} vs {}",
        result.final_score
    );
}

/// `--format sarif` produced a real SARIF document from the state machine and a
/// block of ANSI-coloured prose from `--list-violations`, which is the same
/// contradiction one layer out: one flag, two documents.
#[test]
fn list_violations_honours_sarif_like_every_other_surface() {
    let violations = vec![
        coverage_violation(0.0, 80.0),
        QualityViolation {
            violation_type: "not_measured".to_string(),
            severity: "error".to_string(),
            location: "/tmp/x".to_string(),
            current: 0.0,
            target: 0.0,
            suggestion: "dead code could not be measured".to_string(),
        },
    ];

    let rendered = format_violations_output(
        &violations,
        &QualityProfile::default(),
        EnforceOutputFormat::Sarif,
    )
    .expect("the listing renders");

    let sarif: serde_json::Value = serde_json::from_str(&rendered)
        .expect("--list-violations --format sarif emitted something a SARIF consumer cannot parse");
    assert_eq!(sarif["version"], "2.1.0");
    let results = sarif["runs"][0]["results"]
        .as_array()
        .expect("the run carries results");
    assert_eq!(results.len(), violations.len());
    assert_eq!(
        results[1]["level"], "error",
        "the not_measured disclosure must not be downgraded on this surface either: {rendered}"
    );
}
