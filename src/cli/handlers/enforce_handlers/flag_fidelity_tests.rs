//! A flag that parses must change something.
//!
//! `enforce extreme` accepted five arguments that had no effect whatsoever:
//!
//! * `-o FILE` was bound to `_output` in the handler — no file was created, the
//!   payload went to stdout, exit 0, so a CI step that reads the file it asked
//!   for got nothing and a success code.
//! * `--include` / `--exclude` reached `handle_validating_enforcement_state` as
//!   `_include_pattern` / `_exclude_pattern`: `--exclude 'src/*'` still reported
//!   the violation in `src/lib.rs`, `--include '*.py'` still reported a Rust one.
//! * `--show-progress` was honoured by exactly one of the four output formats,
//!   and the default format is not that one, so it produced byte-identical
//!   output to a run without it.
//! * `--config /nonexistent.toml` was accepted in silence, exit 0, with a
//!   verdict measured against the built-in thresholds — a file the binary never
//!   opened, reported as if it had.
//!
//! And the default invocation re-measured an unchanged tree 100 times, printing
//! the identical three-line summary a hundred times over 11 seconds.

use super::config::{load_quality_profile, EnforcementConfig};
use super::enforcement::execute_main_loop;
use super::output::output_result;
use super::states::handle_analyzing_state;
use super::types::{
    EnforcementProgress, EnforcementResult, EnforcementState, QualityProfile, QualityViolation,
};
use crate::cli::EnforceOutputFormat;
use std::path::{Path, PathBuf};

/// A crate with one SATD marker in `src/lib.rs` — one violation, one file.
fn write_violating_project(dir: &Path) {
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
}

fn sample_result() -> EnforcementResult {
    EnforcementResult {
        state: EnforcementState::Violating,
        score: 0.67,
        target: 1.0,
        current_file: None,
        violations: vec![QualityViolation {
            violation_type: "satd".to_string(),
            severity: "low".to_string(),
            location: "src/lib.rs:1:1".to_string(),
            current: 1.0,
            target: 0.0,
            suggestion: "resolve the marker".to_string(),
        }],
        next_action: "review_violations".to_string(),
        progress: EnforcementProgress {
            files_completed: 0,
            files_remaining: 1,
            estimated_iterations: 3,
        },
    }
}

#[test]
fn output_flag_writes_the_report_to_the_named_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let report = dir.path().join("oo.json");

    output_result(
        &sample_result(),
        EnforceOutputFormat::Json,
        false,
        Some(report.as_path()),
    )
    .expect("the report is emitted");

    let written = std::fs::read_to_string(&report)
        .expect("-o must create the file it names, not print to stdout and exit 0");
    let parsed: serde_json::Value =
        serde_json::from_str(&written).expect("the file must hold the JSON report");
    assert_eq!(parsed["state"], "VIOLATING", "got {written}");
}

#[test]
fn output_flag_reports_a_path_it_cannot_write() {
    let err = output_result(
        &sample_result(),
        EnforceOutputFormat::Json,
        false,
        Some(Path::new("/nonexistent-pmat-enforce-out/report.json")),
    )
    .expect_err("an unwritable -o path must be an error, not a silent fall back to stdout");
    assert!(
        format!("{err:#}").contains("report.json"),
        "the error must name the path: {err:#}"
    );
}

#[test]
fn show_progress_changes_the_default_format_output() {
    let dir = tempfile::tempdir().expect("tempdir");
    let with = dir.path().join("with.txt");
    let without = dir.path().join("without.txt");

    output_result(
        &sample_result(),
        EnforceOutputFormat::Summary,
        true,
        Some(with.as_path()),
    )
    .expect("emit");
    output_result(
        &sample_result(),
        EnforceOutputFormat::Summary,
        false,
        Some(without.as_path()),
    )
    .expect("emit");

    let with = std::fs::read_to_string(with).expect("read");
    let without = std::fs::read_to_string(without).expect("read");
    assert_ne!(
        with, without,
        "--show-progress produced byte-identical output to a run without it"
    );
    assert!(
        with.contains("Enforcement Progress"),
        "the progress bar must be what it added: {with}"
    );
}

#[tokio::test]
async fn exclude_pattern_removes_the_violation_and_the_score_agrees() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_violating_project(dir.path());
    let profile = QualityProfile::default();

    let unfiltered = handle_analyzing_state(dir.path(), &profile, false, true, None, None, None)
        .await
        .expect("analyze");
    let satd_before = unfiltered
        .violations
        .iter()
        .filter(|v| v.violation_type == "satd")
        .count();
    assert!(
        satd_before > 0,
        "fixture must violate something for the filter to remove: {:?}",
        unfiltered.violations
    );

    let excluded = "src/*".to_string();
    let filtered = handle_analyzing_state(
        dir.path(),
        &profile,
        false,
        true,
        None,
        None,
        Some(&excluded),
    )
    .await
    .expect("analyze");

    assert_eq!(
        filtered
            .violations
            .iter()
            .filter(|v| v.violation_type == "satd")
            .count(),
        0,
        "--exclude 'src/*' still reported the violation in src/lib.rs: {:?}",
        filtered.violations
    );
    assert!(
        filtered.score > unfiltered.score,
        "the score must describe the same files the report does: {} vs {}",
        filtered.score,
        unfiltered.score
    );
}

#[tokio::test]
async fn include_pattern_of_another_language_reports_no_rust_violation() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_violating_project(dir.path());
    let profile = QualityProfile::default();
    let include = "**/*.py".to_string();

    let filtered = handle_analyzing_state(
        dir.path(),
        &profile,
        false,
        true,
        None,
        Some(&include),
        None,
    )
    .await
    .expect("analyze");

    // Locations are `path:line:col`, so the path has to be split out first — an
    // assertion on the whole location string is vacuously true and would pass
    // against the very defect this test exists to catch.
    assert!(
        filtered.violations.iter().all(|v| {
            let path = v.location.split(':').next().unwrap_or(&v.location);
            !path.ends_with(".rs")
        }),
        "--include '**/*.py' on a project with no Python still reported a Rust violation: {:?}",
        filtered.violations
    );
}

#[test]
fn config_path_that_cannot_be_read_is_an_error() {
    let err = load_quality_profile(
        "extreme",
        Some(PathBuf::from("/nonexistent-pmat-enforce/config.toml")),
    )
    .expect_err("a config path that cannot be read must not be accepted in silence");
    assert!(
        format!("{err:#}").contains("config.toml"),
        "the error must name the file: {err:#}"
    );
}

#[test]
fn config_file_actually_changes_the_thresholds() {
    let dir = tempfile::tempdir().expect("tempdir");
    let config = dir.path().join("quality.toml");
    std::fs::write(&config, "satd_allowed = 7\ncomplexity_max = 42\n").expect("config");

    let profile = load_quality_profile("extreme", Some(config)).expect("config loads");
    assert_eq!(profile.satd_allowed, 7);
    assert_eq!(profile.complexity_max, 42);
}

#[test]
fn config_file_with_an_unknown_key_is_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");
    let config = dir.path().join("quality.toml");
    std::fs::write(&config, "satd_alowed = 7\n").expect("config");

    load_quality_profile("extreme", Some(config))
        .expect_err("a key that changes nothing must not be accepted as if it had");
}

/// `--max-iterations 0` ran the loop zero times and still exited 0. With
/// `--format json` that meant ZERO BYTES on stdout: an unparseable document and
/// a success code for a consumer that asked for JSON.
#[tokio::test]
async fn max_iterations_zero_is_refused_rather_than_answered_with_nothing() {
    use crate::cli::commands::EnforceCommands;
    use crate::cli::QualityProfile as ProfileArg;

    let dir = tempfile::tempdir().expect("tempdir");
    write_violating_project(dir.path());

    let err = super::route_enforce_command(EnforceCommands::Extreme {
        project_path: dir.path().to_path_buf(),
        single_file_mode: false,
        file: None,
        dry_run: true,
        profile: ProfileArg::Extreme,
        show_progress: false,
        format: crate::cli::EnforceOutputFormat::Json,
        output: None,
        max_iterations: 0,
        target_improvement: None,
        max_time: None,
        apply_suggestions: false,
        validate_only: false,
        list_violations: false,
        config: None,
        ci_mode: false,
        include: None,
        exclude: None,
        cache_dir: None,
        clear_cache: false,
    })
    .await
    .expect_err("zero iterations is zero measurements, and must not exit 0 with an empty document");

    assert!(
        format!("{err:#}").contains("--max-iterations"),
        "the refusal must name the argument: {err:#}"
    );
}

#[tokio::test]
async fn a_project_that_cannot_converge_stops_instead_of_spinning_100_times() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_violating_project(dir.path());
    let report = dir.path().join("report.json");

    let config = EnforcementConfig {
        max_iterations: 100,
        target_improvement: None,
        max_time: None,
        apply_suggestions: false,
        specific_file: None,
        include_pattern: None,
        exclude_pattern: None,
        single_file_mode: false,
        dry_run: true,
        show_progress: false,
        format: EnforceOutputFormat::Json,
        ci_mode: false,
    };

    let result = execute_main_loop(
        &dir.path().to_path_buf(),
        &QualityProfile::default(),
        &config,
        std::time::Instant::now(),
        Some(report.as_path()),
    )
    .await
    .expect("the loop reports");

    assert!(
        result.final_iteration <= 3,
        "nothing changes between iterations of a --dry-run over an unchanged tree, \
         so re-measuring it {} times measures nothing new",
        result.final_iteration
    );
}
