#![cfg_attr(coverage_nightly, coverage(off))]
//! State handler functions for the enforcement state machine
//!
//! Contains the handlers for each enforcement state:
//! Analyzing, Violating, Refactoring, Validating, Complete

use super::assessment::assess_project;
use super::types::{
    EnforcementProgress, EnforcementResult, EnforcementState, QualityProfile, QualityViolation,
};
use crate::cli::colors as c;
use anyhow::Result;
use std::path::{Path, PathBuf};

// ========== SPRINT 82 REFACTORED FUNCTIONS (≤10 COMPLEXITY EACH) ==========

/// Handle analyzing state - a view of [`assess_project`], nothing more.
///
/// This function used to run its own three-phase analysis while
/// `list_all_violations` ran a six-phase one, so `--ci-mode` and
/// `--list-violations` reported different violation counts for the same run.
/// The measurement now happens in exactly one place; this only renders it as an
/// `EnforcementResult`.
///
/// `dry_run` is unused here because the analysing state never writes anything —
/// there is nothing for it to suppress. It used to gate the single-file scope
/// note, which is now printed by the assessment for every surface.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_analyzing_state(
    project_path: &Path,
    profile: &QualityProfile,
    single_file_mode: bool,
    _dry_run: bool,
    specific_file: Option<&PathBuf>,
    include_pattern: Option<&String>,
    exclude_pattern: Option<&String>,
) -> Result<EnforcementResult> {
    let assessment = assess_project(
        project_path,
        profile,
        specific_file.map(PathBuf::as_path),
        include_pattern,
        exclude_pattern,
    )
    .await?;

    let next_state = assessment.verdict_state();

    // `100` used to be reported here for every project, empty directories
    // included; count the files that actually carry a violation instead.
    let files_remaining = if single_file_mode {
        usize::from(assessment.violations.iter().any(is_finding))
    } else {
        distinct_violation_files(&assessment.violations)
    };

    // ...and `files_completed` was the literal `0` for every project, so a run
    // over 124 analysable files reported the same progress as a run over an
    // empty directory. The files the run READ are counted by the phase that
    // enumerates them — one file in `--file` scope, none when nothing could be
    // parsed — and the ones that came back clean are those minus the ones
    // carrying a finding. `saturating_sub` because a finding may name a file
    // outside the analysable source set (SATD reads more file types than the
    // AST phases do), which can only ever understate what was completed.
    let files_completed = assessment.files_examined.saturating_sub(files_remaining);

    Ok(EnforcementResult {
        state: next_state,
        score: assessment.score,
        target: 1.0,
        current_file: specific_file.map(|p| p.display().to_string()),
        violations: assessment.violations,
        next_action: if next_state == EnforcementState::Complete {
            "none".to_string()
        } else {
            "review_violations".to_string()
        },
        progress: EnforcementProgress {
            files_completed,
            files_remaining,
            estimated_iterations: ((1.0 - assessment.score) * 10.0) as u32,
        },
    })
}

/// Does this row name a file, or the run?
///
/// A `not_measured` disclosure is about a dimension of the whole assessment, so
/// its location is the scope (the project directory), not a source file. Counting
/// it as a file made an empty crate report "1 file remaining" for a directory in
/// which no file carried anything, and would have credited the same phantom file
/// against `files_completed`. Both counts read this one rule.
fn is_finding(violation: &QualityViolation) -> bool {
    violation.violation_type != "not_measured"
}

/// Number of distinct files named by the findings, used for progress
/// reporting. Locations are `path:line:name` or a bare path.
fn distinct_violation_files(violations: &[QualityViolation]) -> usize {
    violations
        .iter()
        .filter(|v| is_finding(v))
        .map(|v| v.location.split(':').next().unwrap_or(&v.location))
        .collect::<std::collections::BTreeSet<_>>()
        .len()
}

/// Handle violating state - extracted from `run_enforcement_step` (complexity: ≤10)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn handle_violating_state(
    violations: Vec<QualityViolation>,
    total_score: f64,
    apply_suggestions: bool,
    dry_run: bool,
    specific_file: Option<&PathBuf>,
) -> Result<EnforcementResult> {
    if apply_suggestions && !dry_run {
        Ok(EnforcementResult {
            state: EnforcementState::Refactoring,
            score: total_score,
            target: 1.0,
            current_file: specific_file.map(|p| p.display().to_string()),
            violations: violations.clone(),
            next_action: "apply_refactoring".to_string(),
            progress: EnforcementProgress {
                files_completed: 0,
                files_remaining: violations.len(),
                estimated_iterations: violations.len() as u32,
            },
        })
    } else {
        Ok(EnforcementResult {
            state: EnforcementState::Violating,
            score: total_score,
            target: 1.0,
            current_file: specific_file.map(|p| p.display().to_string()),
            violations,
            next_action: "manual_intervention_required".to_string(),
            progress: EnforcementProgress {
                files_completed: 0,
                files_remaining: 0,
                estimated_iterations: 0,
            },
        })
    }
}

/// The refactoring pass — a view of [`assess_project`], like every other state.
///
/// Nothing here edits the tree: `--apply-suggestions` has never had an
/// implementation behind it. The state used to answer for one anyway, with
/// `score: total_score + 0.1  // Assume some improvement` and
/// `violations: vec![]  // Clear after refactoring`, which made it the third
/// hardcoded violation list in this command. The last JSON document a
/// `pmat enforce extreme --apply-suggestions --format json` consumer parsed
/// therefore read `"state":"VALIDATING", "score":0.8, "violations":[]` for a
/// directory that `--validate-only`, `--list-violations` and `--ci-mode` all
/// reported as 0.8333 with two violations.
///
/// A step that changed nothing must report the measurement unchanged, and say
/// out loud that it changed nothing.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_refactoring_pass(
    project_path: &Path,
    profile: &QualityProfile,
    single_file_mode: bool,
    dry_run: bool,
    specific_file: Option<&PathBuf>,
    include_pattern: Option<&String>,
    exclude_pattern: Option<&String>,
) -> Result<EnforcementResult> {
    eprintln!(
        "{}",
        c::warn(
            "--apply-suggestions: no automated refactoring is implemented, so nothing was changed"
        )
    );

    let measured = handle_analyzing_state(
        project_path,
        profile,
        single_file_mode,
        dry_run,
        specific_file,
        include_pattern,
        exclude_pattern,
    )
    .await?;

    Ok(label_refactoring_pass(measured))
}

/// Relabel a measured run as the refactoring pass's output — and change nothing
/// else.
///
/// The state field names the machine's NEXT step; the numbers belong to the run.
/// A pass that found the project clean does not get relabelled — the verdict is
/// `QualityAssessment::verdict_state`'s to give, here as anywhere.
///
/// Extracted from [`handle_refactoring_pass`] so the property that actually
/// matters — *the refactoring pass does not invent numbers* — can be tested
/// without measuring a real tree twice. The predecessor of this code emptied the
/// violation list and added 0.1 to a score the caller passed as the literal
/// `0.7`, so `--apply-suggestions --format json` ended on
/// `"state":"VALIDATING","score":0.8,"violations":[]` for a directory every other
/// surface called violating. That is a pure-data defect and it is now pinned by a
/// pure-data test (#1013): the integration test that used to pin it compared two
/// independent pipeline runs, and flaked whenever the wall-clock-budgeted
/// dead-code phase landed in one run and timed out in the other.
pub(super) fn label_refactoring_pass(mut measured: EnforcementResult) -> EnforcementResult {
    if measured.state != EnforcementState::Complete {
        measured.state = EnforcementState::Validating;
        measured.next_action = "validate_changes".to_string();
    }
    measured
}

/// The fabricating refactoring handler, kept alive ONLY because five tests in
/// three files outside this module still pin its arithmetic
/// (`enforce_coverage_part2.rs`, `enforce_coverage_part4.rs`,
/// `enforce_coverage_part3_state_tests.rs`). It is `#[cfg(test)]` so that no
/// surface of the binary can reach it: the enforcement path now runs
/// [`handle_refactoring_pass`], which reports the measurement instead of
/// inventing one. Delete this together with those tests.
#[cfg(test)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn handle_refactoring_state(
    total_score: f64,
    specific_file: Option<&PathBuf>,
) -> Result<EnforcementResult> {
    eprintln!("🔧 Applying automated refactoring...");

    Ok(EnforcementResult {
        state: EnforcementState::Validating,
        score: total_score + 0.1, // Assume some improvement
        target: 1.0,
        current_file: specific_file.map(|p| p.display().to_string()),
        violations: vec![], // Clear after refactoring
        next_action: "validate_changes".to_string(),
        progress: EnforcementProgress {
            files_completed: 1,
            files_remaining: 0,
            estimated_iterations: 1,
        },
    })
}

/// The other handler that answers without looking, kept alive ONLY because
/// tests in three files outside this module still pin its literals
/// (`src/tests/coverage_boost_enforce.rs`, `enforce_coverage_part2.rs`,
/// `enforce_coverage_part3_state_tests.rs`).
///
/// It takes no arguments, so there is nothing it could have measured:
/// `score: 1.0` asserts a project it never opened meets the profile, and
/// `files_completed: 100` carried its own confession — `// Would count actual`.
/// It is `#[cfg(test)]` so no surface of the binary can reach it; the Complete
/// arm of `run_enforcement_step` reads the assessment, as the Validating arm
/// already did. Delete this together with those tests.
#[cfg(test)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn handle_complete_state() -> Result<EnforcementResult> {
    Ok(EnforcementResult {
        state: EnforcementState::Complete,
        score: 1.0,
        target: 1.0,
        current_file: None,
        violations: vec![],
        next_action: "none".to_string(),
        progress: EnforcementProgress {
            files_completed: 100, // Would count actual
            files_remaining: 0,
            estimated_iterations: 0,
        },
    })
}

// ========== SPRINT 84 REFACTORED FUNCTIONS (A+ STANDARD ≤10 COMPLEXITY EACH) ==========

/// Handle violating state proxy - extracted from `run_enforcement_step` (complexity: ≤10)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_violating_enforcement_state_proxy(
    project_path: &PathBuf,
    profile: &QualityProfile,
    single_file_mode: bool,
    dry_run: bool,
    specific_file: Option<&PathBuf>,
    apply_suggestions: bool,
    include_pattern: Option<&String>,
    exclude_pattern: Option<&String>,
) -> Result<EnforcementResult> {
    // Get violations from previous analyzing state
    let analyzing_result = handle_analyzing_state(
        project_path,
        profile,
        single_file_mode,
        dry_run,
        specific_file,
        include_pattern,
        exclude_pattern,
    )
    .await?;
    let measured = analyzing_result.progress.clone();
    let mut result = handle_violating_state(
        analyzing_result.violations,
        analyzing_result.score,
        apply_suggestions,
        dry_run,
        specific_file,
    )?;
    // `handle_violating_state` cannot measure — it is handed a violation list and
    // nothing else — so both of its branches state progress as literals
    // (`files_completed: 0` in each; `files_remaining: 0` in one, a count of
    // VIOLATIONS rather than files in the other). The same run has just been
    // measured one line above, and that is what the step reports. Its signature
    // is pinned by tests in three files outside this module, so the composition
    // is corrected here rather than there.
    result.progress = measured;
    Ok(result)
}

/// Test-only alias of the fabricating handler above; see its doc comment. The
/// enforcement path calls [`handle_refactoring_pass`].
#[cfg(test)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn handle_refactoring_enforcement_state(
    base_score: f64,
    specific_file: Option<&PathBuf>,
) -> Result<EnforcementResult> {
    handle_refactoring_state(base_score, specific_file)
}

/// Handle validating state for enforcement - extracted from `run_enforcement_step` (complexity: ≤10)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_validating_enforcement_state(
    project_path: &PathBuf,
    profile: &QualityProfile,
    single_file_mode: bool,
    dry_run: bool,
    specific_file: Option<&PathBuf>,
    include_pattern: Option<&String>,
    exclude_pattern: Option<&String>,
) -> Result<EnforcementResult> {
    // Re-run analysis to validate improvements. The two patterns arrived here as
    // `_include_pattern` / `_exclude_pattern` — carried the whole way down the
    // call chain and then dropped on the floor.
    //
    // The state used to be recomputed here from `violations.is_empty()`, a
    // second copy of the verdict rule that disagreed with the first: it called a
    // run whose phases could not be measured `Complete`, because "no findings"
    // and "nothing was looked at" produce the same empty list. There is one
    // verdict rule (`QualityAssessment::verdict_state`) and the analysing state
    // has already applied it.
    handle_analyzing_state(
        project_path,
        profile,
        single_file_mode,
        dry_run,
        specific_file,
        include_pattern,
        exclude_pattern,
    )
    .await
}

/// Handle analyzing state for enforcement - alias for clarity (complexity: ≤10)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_analyzing_enforcement_state(
    project_path: &PathBuf,
    profile: &QualityProfile,
    single_file_mode: bool,
    dry_run: bool,
    specific_file: Option<&PathBuf>,
) -> Result<EnforcementResult> {
    handle_analyzing_state(
        project_path,
        profile,
        single_file_mode,
        dry_run,
        specific_file,
        None,
        None,
    )
    .await
}

/// Test-only alias of the non-measuring handler above; see its doc comment.
#[cfg(test)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn handle_complete_enforcement_state() -> Result<EnforcementResult> {
    handle_complete_state()
}

/// Regression tests for the composite enforcement score.
///
/// The score used to be the mean of four hardcoded constants, so
/// `pmat enforce extreme -p <anything> --dry-run` printed `Score: 0.79/1.00`
/// for an empty directory and for a 3252-file repository alike.
#[cfg(test)]
mod composite_score_regression_tests {
    use super::super::assessment::phase_score;
    use super::*;

    fn strict_profile() -> QualityProfile {
        QualityProfile {
            // Isolate the complexity dimension: every function in the fixture
            // exceeds this, while TDG is given headroom it cannot breach.
            complexity_max: 1,
            complexity_target: 1,
            tdg_max: 1000.0,
            ..QualityProfile::default()
        }
    }

    fn write_project(dir: &Path, source: &str) {
        std::fs::write(
            dir.join("Cargo.toml"),
            "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .expect("write Cargo.toml");
        std::fs::create_dir_all(dir.join("src")).expect("create src");
        std::fs::write(dir.join("src").join("lib.rs"), source).expect("write lib.rs");
        // The verdict covers coverage too, so a fixture meant to be "fully
        // measured" carries the lcov report a real project produces with
        // `cargo llvm-cov`; without one, coverage is honestly disclosed as
        // unmeasured.
        std::fs::write(
            dir.join("lcov.info"),
            "SF:src/lib.rs\nDA:1,1\nLF:1\nLH:1\nend_of_record\n",
        )
        .expect("write lcov.info");
    }

    #[tokio::test]
    async fn score_distinguishes_an_empty_directory_from_a_violating_project() {
        let empty = tempfile::tempdir().expect("tempdir");
        let profile = strict_profile();

        let empty_result =
            handle_analyzing_state(empty.path(), &profile, false, true, None, None, None)
                .await
                .expect("analyze empty directory");

        // An empty directory breaches no threshold...
        assert!(
            empty_result
                .violations
                .iter()
                .all(|v| v.violation_type == "not_measured"),
            "an empty directory violates nothing: {:?}",
            empty_result.violations
        );
        // ...but it does not demonstrate compliance either. This used to assert
        // a flat 1.0, which credited an assessment that never happened: with no
        // source files, complexity and SATD have nothing to measure, and only
        // TDG returns a (vacuous) clean result. Full marks for one dimension of
        // three is how `enforce extreme` came to report `Complete 1.00/1.00` on
        // an empty tree.
        assert!(
            empty_result
                .violations
                .iter()
                .any(|v| v.violation_type == "not_measured"),
            "the unmeasured dimensions must be visible: {:?}",
            empty_result.violations
        );
        assert!(
            empty_result.score < 1.0,
            "an assessment covering one dimension of three cannot score full marks: got {}",
            empty_result.score
        );
        assert_ne!(empty_result.state, EnforcementState::Complete);

        let violating = tempfile::tempdir().expect("tempdir");
        write_project(
            violating.path(),
            "pub fn branchy(a: i32, b: i32) -> i32 {\n\
             \x20   if a > 0 {\n\
             \x20       if b > 0 {\n\
             \x20           return 1;\n\
             \x20       }\n\
             \x20       return 2;\n\
             \x20   }\n\
             \x20   match a {\n\
             \x20       1 => 3,\n\
             \x20       2 => 4,\n\
             \x20       3 => 5,\n\
             \x20       _ => 6,\n\
             \x20   }\n\
             }\n",
        );

        let violating_result =
            handle_analyzing_state(violating.path(), &profile, false, true, None, None, None)
                .await
                .expect("analyze violating project");

        assert!(
            violating_result
                .violations
                .iter()
                .any(|v| v.violation_type == "complexity"),
            "fixture must breach complexity_max = 1: {:?}",
            violating_result.violations
        );
        // The defect this test was written for: both inputs scored a literal
        // 0.79, so the number described neither. They must still be
        // distinguishable — but no longer by which is *lower*, since the two are
        // now penalised on different axes (missing evidence vs real violations),
        // and ordering them on one scale would be its own false precision.
        assert!(
            (violating_result.score - empty_result.score).abs() > f64::EPSILON,
            "a violating project ({}) must not score the same as an empty directory ({})",
            violating_result.score,
            empty_result.score
        );
        // NOT asserted here: "a parseable project measures every dimension".
        //
        // That assertion flaked, and it flaked because it is not true. The
        // dead-code phase is a wall-clock budget around a `cargo check`, so
        // whether a dimension is measured depends on how loaded the machine is
        // — a contended runner tripped even a 300s budget on a two-function
        // fixture. The verdict therefore varies for identical code, which is a
        // property of the PRODUCT, not of this test, and one that cannot be
        // asserted away here.
        //
        // Raising the budget or setting one just for this test would only move
        // the flake; a test that passes because the machine happened to be idle
        // is not a test. The disclosure invariant that actually matters — an
        // unmeasured dimension is never silently scored as clean — is asserted
        // deterministically in `not_measured_is_disclosed_not_scored` below,
        // which needs no subprocess. The nondeterminism itself is tracked
        // separately.
    }

    /// `progress.files_completed` was the literal `0` on every path a CLI
    /// invocation could reach (`states.rs` had five writers, four of them `0`
    /// and one `100, // Would count actual`). It reported 0 for an empty
    /// directory, for a one-file crate and for a 121-file corpus with 60
    /// complexity violations alike, which is the same number a run that
    /// examined nothing would report.
    ///
    /// The count is now the files the run READ minus the files carrying a
    /// finding, so it has to move when the tree does.
    #[tokio::test]
    async fn files_completed_counts_the_clean_files_the_run_actually_read() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .expect("write Cargo.toml");
        std::fs::create_dir_all(dir.path().join("src")).expect("create src");
        // Two files that meet the profile...
        std::fs::write(
            dir.path().join("src/lib.rs"),
            "pub mod clean;\npub mod branchy;\n",
        )
        .expect("write lib.rs");
        std::fs::write(
            dir.path().join("src/clean.rs"),
            "pub fn double(a: i32) -> i32 {\n    a * 2\n}\n",
        )
        .expect("write clean.rs");
        // ...and one that does not.
        std::fs::write(
            dir.path().join("src/branchy.rs"),
            "pub fn branchy(a: i32, b: i32) -> i32 {\n\
             \x20   if a > 0 {\n\
             \x20       if b > 0 {\n\
             \x20           return 1;\n\
             \x20       }\n\
             \x20       return 2;\n\
             \x20   }\n\
             \x20   match a {\n\
             \x20       1 => 3,\n\
             \x20       2 => 4,\n\
             \x20       3 => 5,\n\
             \x20       _ => 6,\n\
             \x20   }\n\
             }\n",
        )
        .expect("write branchy.rs");

        let profile = QualityProfile {
            complexity_max: 2,
            complexity_target: 2,
            tdg_max: 1000.0,
            ..QualityProfile::default()
        };

        let result = handle_analyzing_state(dir.path(), &profile, false, true, None, None, None)
            .await
            .expect("analyze fixture");

        assert_eq!(
            result.progress.files_remaining, 1,
            "exactly one file breaches the profile: {:?}",
            result.violations
        );
        assert_eq!(
            result.progress.files_completed, 2,
            "three source files were read and one carries a finding, so two are \
             done — this was the literal 0: {:?}",
            result.progress
        );

        // The unmeasured-coverage disclosure names the run, not a file, so it
        // must not be counted against either side of the progress.
        assert!(
            result
                .violations
                .iter()
                .any(|v| v.violation_type == "not_measured"),
            "the fixture carries no lcov report, so coverage is disclosed: {:?}",
            result.violations
        );

        let empty = tempfile::tempdir().expect("tempdir");
        let empty_result =
            handle_analyzing_state(empty.path(), &profile, false, true, None, None, None)
                .await
                .expect("analyze empty directory");
        assert_eq!(
            empty_result.progress.files_completed, 0,
            "a directory with no source file completes none of them: {:?}",
            empty_result.progress
        );
        assert_eq!(
            empty_result.progress.files_remaining, 0,
            "and no file carries a finding there either: {:?}",
            empty_result.progress
        );
    }

    /// `--validate-only` held a measured run and re-stated its progress as
    /// `0 / 0`; the state machine's Violating step overwrote it with literals
    /// too. Both now report what `handle_analyzing_state` measured.
    #[tokio::test]
    async fn every_state_reports_the_progress_that_was_measured() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_project(
            dir.path(),
            "pub fn branchy(a: i32, b: i32) -> i32 {\n\
             \x20   if a > 0 {\n\
             \x20       if b > 0 {\n\
             \x20           return 1;\n\
             \x20       }\n\
             \x20       return 2;\n\
             \x20   }\n\
             \x20   a\n\
             }\n",
        );
        let profile = QualityProfile {
            complexity_max: 1,
            complexity_target: 1,
            tdg_max: 1000.0,
            ..QualityProfile::default()
        };

        let analyzing = handle_analyzing_state(dir.path(), &profile, false, true, None, None, None)
            .await
            .expect("analyze");
        assert_eq!(analyzing.progress.files_remaining, 1);

        let violating = handle_violating_enforcement_state_proxy(
            &dir.path().to_path_buf(),
            &profile,
            false,
            true,
            None,
            false,
            None,
            None,
        )
        .await
        .expect("violating step");

        assert_eq!(
            violating.progress.files_remaining, analyzing.progress.files_remaining,
            "the violating step describes the same run: {:?} vs {:?}",
            violating.progress, analyzing.progress
        );
        assert_eq!(
            violating.progress.files_completed, analyzing.progress.files_completed,
            "the violating step describes the same run: {:?} vs {:?}",
            violating.progress, analyzing.progress
        );
    }

    #[test]
    fn phase_score_is_clean_when_nothing_was_reported() {
        assert!((phase_score(&[]) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn phase_score_scales_with_the_worst_overshoot() {
        let violations = vec![
            QualityViolation {
                violation_type: "complexity".to_string(),
                severity: "medium".to_string(),
                location: "a.rs:1:f".to_string(),
                current: 20.0,
                target: 10.0,
                suggestion: String::new(),
            },
            QualityViolation {
                violation_type: "complexity".to_string(),
                severity: "high".to_string(),
                location: "b.rs:1:g".to_string(),
                current: 40.0,
                target: 10.0,
                suggestion: String::new(),
            },
        ];

        assert!((phase_score(&violations) - 0.25).abs() < 1e-9);
        assert_eq!(distinct_violation_files(&violations), 2);
    }
    /// The invariant the removed assertion was reaching for, asserted where it
    /// is actually decidable.
    ///
    /// `score_distinguishes_an_empty_directory_from_a_violating_project` used
    /// to end with "a parseable project measures every dimension". That is not
    /// a property the product guarantees: the dead-code phase is a wall-clock
    /// budget around a `cargo check`, so on a loaded machine a dimension goes
    /// unmeasured and the assertion fails for reasons that have nothing to do
    /// with the code under test.
    ///
    /// What DOES hold, always, is the disclosure rule: a phase that could not
    /// run is surfaced as a `not_measured` violation rather than being silently
    /// credited as clean. `summarize` is a pure function over phase outcomes,
    /// so this needs no subprocess and cannot flake.
    #[test]
    fn not_measured_is_disclosed_not_scored() {
        use crate::cli::handlers::enforce_handlers::assessment::summarize;
        use crate::cli::handlers::enforce_handlers::types::PhaseOutcome;

        let assessment = summarize(
            vec![
                ("complexity", PhaseOutcome::measured(vec![])),
                (
                    "dead code",
                    PhaseOutcome::unmeasured("Dead code analysis timed out after 300 seconds"),
                ),
            ],
            std::path::Path::new("/tmp/whatever"),
            None,
            None,
            None,
        )
        .expect("summarize");

        let disclosed: Vec<&crate::cli::handlers::enforce_handlers::types::QualityViolation> =
            assessment
                .violations
                .iter()
                .filter(|v| v.violation_type == "not_measured")
                .collect();
        assert_eq!(
            disclosed.len(),
            1,
            "the phase that could not run must be disclosed: {:?}",
            assessment.violations
        );
        assert!(
            disclosed[0].suggestion.contains("dead code"),
            "the disclosure must name WHICH phase: {}",
            disclosed[0].suggestion
        );
        assert!(
            disclosed[0].suggestion.contains("timed out"),
            "and carry the reason, so a timeout is not mistaken for a clean result: {}",
            disclosed[0].suggestion
        );
        assert!(
            assessment.score < 1.0,
            "a run with an unmeasured dimension must not score full marks: {}",
            assessment.score
        );
    }

    /// #1013: the deterministic replacement for the cross-run comparison in
    /// `surface_agreement_tests`.
    ///
    /// The property is "the refactoring pass reports the run it did not change".
    /// The old test asserted it by measuring a real tree TWICE and comparing the
    /// two violation lists and scores — which fails whenever the dead-code phase
    /// (a wall-clock budget around `cargo check`) lands in one run and times out
    /// in the other. That happened under `cargo llvm-cov`, where the instrumented
    /// harness starves the blocking task; `ci / test` passed the same commit.
    ///
    /// The defect being guarded is pure data — the predecessor emptied
    /// `violations` and added 0.1 to a score handed in as the literal `0.7` — so
    /// it is guarded with pure data here. Every field except `state` and
    /// `next_action` must survive relabelling untouched, for every input state.
    #[test]
    fn the_refactoring_label_changes_no_number() {
        let violations = vec![QualityViolation {
            violation_type: "satd".to_string(),
            severity: "low".to_string(),
            location: "src/lib.rs:1:1".to_string(),
            current: 1.0,
            target: 0.0,
            suggestion: "Resolve the debt marker".to_string(),
        }];

        for state in [
            EnforcementState::Analyzing,
            EnforcementState::Violating,
            EnforcementState::Refactoring,
            EnforcementState::Validating,
            EnforcementState::Complete,
        ] {
            let measured = EnforcementResult {
                state,
                score: 0.7,
                target: 0.9,
                current_file: None,
                violations: violations.clone(),
                next_action: "measured_next".to_string(),
                progress: EnforcementProgress {
                    files_completed: 3,
                    files_remaining: 1,
                    estimated_iterations: 2,
                },
            };
            let labelled = label_refactoring_pass(measured.clone());

            // Projected rather than compared whole: QualityViolation derives
            // neither PartialEq nor Default, and adding derives to a production
            // type to satisfy a test is a change to the product for the test's
            // convenience. The projection covers every field the fabricating
            // predecessor got wrong.
            let project = |v: &[QualityViolation]| -> Vec<(String, String, String, f64, f64)> {
                v.iter()
                    .map(|x| {
                        (
                            x.violation_type.clone(),
                            x.severity.clone(),
                            x.location.clone(),
                            x.current,
                            x.target,
                        )
                    })
                    .collect()
            };
            assert_eq!(
                project(&labelled.violations),
                project(&measured.violations),
                "the refactoring label cleared a violation list it never fixed (from {state:?})"
            );
            assert!(
                (labelled.score - measured.score).abs() < f64::EPSILON,
                "the refactoring label moved the score {} -> {} (from {state:?})",
                measured.score,
                labelled.score
            );
            assert!((labelled.target - measured.target).abs() < f64::EPSILON);

            // Only the machine's next step is allowed to move, and only when the
            // run was not already Complete.
            if state == EnforcementState::Complete {
                assert_eq!(
                    labelled.state,
                    EnforcementState::Complete,
                    "a clean run was relabelled; the verdict is verdict_state's to give"
                );
                assert_eq!(labelled.next_action, "measured_next");
            } else {
                assert_eq!(labelled.state, EnforcementState::Validating);
                assert_eq!(labelled.next_action, "validate_changes");
            }
        }
    }
}
