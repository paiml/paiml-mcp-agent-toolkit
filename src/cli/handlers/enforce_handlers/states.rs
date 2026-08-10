#![cfg_attr(coverage_nightly, coverage(off))]
//! State handler functions for the enforcement state machine
//!
//! Contains the handlers for each enforcement state:
//! Analyzing, Violating, Refactoring, Validating, Complete

use super::analysis::{
    run_complexity_analysis, run_satd_analysis, run_tdg_analysis, AnalysisScope,
};
use super::types::{
    EnforcementProgress, EnforcementResult, EnforcementState, QualityProfile, QualityViolation,
};
use anyhow::Result;
use std::path::{Path, PathBuf};

// ========== SPRINT 82 REFACTORED FUNCTIONS (≤10 COMPLEXITY EACH) ==========

/// Handle analyzing state - extracted from `run_enforcement_step` (complexity: ≤10)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_analyzing_state(
    project_path: &Path,
    profile: &QualityProfile,
    single_file_mode: bool,
    dry_run: bool,
    specific_file: Option<&PathBuf>,
) -> Result<EnforcementResult> {
    let scope = AnalysisScope::resolve(project_path, specific_file.map(PathBuf::as_path));

    // SATD walks a directory tree and cannot target a lone file; surface the
    // parent-module fallback so --dry-run output is honest about scope
    if dry_run {
        if let AnalysisScope::SingleFile { module_dir, .. } = &scope {
            eprintln!(
                "📍 Single-file mode: SATD scoped to parent module {}",
                module_dir.display()
            );
        }
    }

    let mut violations = Vec::new();

    // Run all analyses in sequence, each scoped per-phase when --file is given
    let complexity_violations =
        run_complexity_analysis(scope.walk_root(), profile, scope.single_file()).await?;
    let satd_violations = run_satd_analysis(scope.walk_root(), profile).await?;
    let tdg_violations = run_tdg_analysis(scope.file_or_root(), profile).await?;

    // Composite score, derived from the analyses that were just run.
    //
    // These four lines used to be
    //   let complexity_score = 0.8;  // Would calculate from actual results
    //   let satd_score = if profile.satd_allowed == 0 { 1.0 } else { 0.5 };
    //   let tdg_score = 0.7;         // Would calculate from actual TDG
    //   let coverage_score = 0.65;   // Would parse from coverage tool
    // whose mean is 0.7875, so `pmat enforce extreme` printed
    // `Score: 0.79/1.00` verbatim for an empty directory, a three-line crate
    // and this repository alike. A score that cannot tell an empty directory
    // from a 3252-file repo measures nothing.
    //
    // Each phase that ran contributes one dimension. Coverage is deliberately
    // absent: nothing in this state machine measures it, and the 0.65 stand-in
    // is exactly the kind of invented number this replaces.
    //
    // Caveat worth keeping in view: a phase whose analysis FAILED also returns
    // no violations and so scores 1.0 here — `run_*_analysis` swallows its own
    // errors (it warns "not measured" and returns an empty vec) rather than
    // reporting them upwards. Telling "clean" from "not measured" needs those
    // functions to return that distinction.
    let phase_scores = [
        phase_score(&complexity_violations),
        phase_score(&satd_violations),
        phase_score(&tdg_violations),
    ];
    let total_score = phase_scores.iter().sum::<f64>() / phase_scores.len() as f64;

    violations.extend(complexity_violations);
    violations.extend(satd_violations);
    violations.extend(tdg_violations);

    // Determine next state
    let next_state = if violations.is_empty() {
        EnforcementState::Complete
    } else {
        EnforcementState::Violating
    };

    // `100` used to be reported here for every project, empty directories
    // included; count the files that actually carry a violation instead.
    let files_remaining = if single_file_mode {
        usize::from(!violations.is_empty())
    } else {
        distinct_violation_files(&violations)
    };

    Ok(EnforcementResult {
        state: next_state,
        score: total_score,
        target: 1.0,
        current_file: specific_file.map(|p| p.display().to_string()),
        violations,
        next_action: if next_state == EnforcementState::Complete {
            "none".to_string()
        } else {
            "review_violations".to_string()
        },
        progress: EnforcementProgress {
            files_completed: 0,
            files_remaining,
            estimated_iterations: ((1.0 - total_score) * 10.0) as u32,
        },
    })
}

/// Score one analysis phase in `[0.0, 1.0]`: 1.0 when the phase found nothing
/// to report, otherwise how close its worst violation sits to the limit the
/// profile allows (a function at twice the allowed complexity scores 0.5).
fn phase_score(violations: &[QualityViolation]) -> f64 {
    violations
        .iter()
        .map(|v| {
            if v.current <= v.target {
                1.0
            } else if v.target > 0.0 {
                (v.target / v.current).clamp(0.0, 1.0)
            } else {
                // A zero-tolerance dimension (e.g. satd_allowed = 0) that was
                // nevertheless violated.
                0.0
            }
        })
        .fold(1.0_f64, f64::min)
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

/// Handle refactoring state - extracted from `run_enforcement_step` (complexity: ≤10)
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
) -> Result<EnforcementResult> {
    // Get violations from previous analyzing state
    let analyzing_result = handle_analyzing_state(
        project_path,
        profile,
        single_file_mode,
        dry_run,
        specific_file,
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

/// Handle refactoring state for enforcement - extracted from `run_enforcement_step` (complexity: ≤10)
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
    _include_pattern: Option<&String>,
    _exclude_pattern: Option<&String>,
) -> Result<EnforcementResult> {
    // Re-run analysis to validate improvements
    let mut result = handle_analyzing_state(
        project_path,
        profile,
        single_file_mode,
        dry_run,
        specific_file,
    )
    .await?;

    // Override state based on validation results
    if result.violations.is_empty() {
        result.state = EnforcementState::Complete;
    } else {
        result.state = EnforcementState::Violating;
    }

    Ok(result)
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
    }

    #[tokio::test]
    async fn score_distinguishes_an_empty_directory_from_a_violating_project() {
        let empty = tempfile::tempdir().expect("tempdir");
        let profile = strict_profile();

        let empty_result = handle_analyzing_state(empty.path(), &profile, false, true, None)
            .await
            .expect("analyze empty directory");

        assert!(
            empty_result.violations.is_empty(),
            "an empty directory violates nothing: {:?}",
            empty_result.violations
        );
        assert!(
            (empty_result.score - 1.0).abs() < f64::EPSILON,
            "empty directory scored {} — the score is not derived from the analysis",
            empty_result.score
        );

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
            handle_analyzing_state(violating.path(), &profile, false, true, None)
                .await
                .expect("analyze violating project");

        assert!(
            !violating_result.violations.is_empty(),
            "fixture must breach complexity_max = 1"
        );
        assert!(
            violating_result.score < empty_result.score,
            "a violating project ({}) must not score the same as an empty directory ({})",
            violating_result.score,
            empty_result.score
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
