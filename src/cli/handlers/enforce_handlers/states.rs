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
        usize::from(!assessment.violations.is_empty())
    } else {
        distinct_violation_files(&assessment.violations)
    };

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
            files_completed: 0,
            files_remaining,
            estimated_iterations: ((1.0 - assessment.score) * 10.0) as u32,
        },
    })
}

/// Number of distinct files named by the violations, used for progress
/// reporting. Locations are `path:line:name` or a bare path.
fn distinct_violation_files(violations: &[QualityViolation]) -> usize {
    violations
        .iter()
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

    let mut measured = handle_analyzing_state(
        project_path,
        profile,
        single_file_mode,
        dry_run,
        specific_file,
        include_pattern,
        exclude_pattern,
    )
    .await?;

    // The state field names the machine's next step; the numbers belong to the
    // run. A pass that found the project clean does not get relabelled — the
    // verdict is `QualityAssessment::verdict_state`'s to give, here as anywhere.
    if measured.state != EnforcementState::Complete {
        measured.state = EnforcementState::Validating;
        measured.next_action = "validate_changes".to_string();
    }
    Ok(measured)
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

/// Handle complete state - extracted from `run_enforcement_step` (complexity: ≤10)
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
    handle_violating_state(
        analyzing_result.violations,
        analyzing_result.score,
        apply_suggestions,
        dry_run,
        specific_file,
    )
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

/// Handle complete state for enforcement - alias for clarity (complexity: ≤10)
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
        assert!(
            violating_result
                .violations
                .iter()
                .all(|v| v.violation_type != "not_measured"),
            "a parseable project measures every dimension: {:?}",
            violating_result.violations
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
}
