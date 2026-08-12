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

use super::assessment::assess_project;
use super::enforcement::handle_special_modes;
use super::states::handle_analyzing_state;
use super::types::{EnforcementState, QualityProfile};
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
