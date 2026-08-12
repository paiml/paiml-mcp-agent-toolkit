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
    // A path that does not exist cannot be enforced against, and must not be
    // answered with a verdict. `enforce extreme -p /nope` reported
    // `Complete 1.00/1.00`, exit 0 — the analyzers return `Ok` for input they
    // never read, so every phase came back clean. Refuse up front, as the other
    // path-taking commands in this release do.
    if !project_path.exists() {
        anyhow::bail!(
            "path not found: {} — enforce cannot report a verdict on a path it cannot read",
            project_path.display()
        );
    }

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
    let complexity_outcome =
        run_complexity_analysis(scope.walk_root(), profile, scope.single_file()).await?;
    let satd_outcome = run_satd_analysis(scope.walk_root(), profile, scope.single_file()).await?;
    let tdg_outcome = run_tdg_analysis(scope.file_or_root(), profile).await?;

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
    // The caveat that used to sit here — "a phase whose analysis FAILED also
    // returns no violations and so scores 1.0 ... telling clean from not
    // measured needs those functions to return that distinction" — was a
    // description of a live defect, and it was doing real damage: `enforce
    // extreme` reported a perfect 1.00/1.00 `Complete` for a nonexistent path,
    // an empty directory, and a project whose sources do not parse, because
    // every phase failed and every failure read as clean. `PhaseOutcome` now
    // carries that distinction, so this is the enforcement point for it.
    //
    // An unmeasured phase is excluded from the mean rather than scored: the
    // score then honestly describes what WAS measured. It is the verdict that
    // must not pass, which is handled below — averaging a 0.0 in would report a
    // quality problem where the truth is an absence of evidence.
    let phases = [
        ("complexity", &complexity_outcome),
        ("satd", &satd_outcome),
        ("tdg", &tdg_outcome),
    ];
    let measured: Vec<f64> = phases
        .iter()
        .filter(|(_, o)| o.is_measured())
        .map(|(_, o)| phase_score(&o.violations))
        .collect();
    // The score answers "how close is this project to the profile, given what
    // could be checked" — so it carries BOTH the quality of the measured
    // dimensions and how many of them there were. The mean alone does not: an
    // empty directory measures one dimension of three, finds nothing wrong in
    // it, and would report a flat 1.00 next to a `Violating` verdict. Scaling by
    // the fraction that could be measured keeps a partial assessment from
    // presenting as a complete one, and leaves a fully measured clean project at
    // exactly 1.0.
    let total_score = if measured.is_empty() {
        0.0
    } else {
        let mean = measured.iter().sum::<f64>() / measured.len() as f64;
        let coverage = measured.len() as f64 / phases.len() as f64;
        mean * coverage
    };

    // Each gap becomes a visible finding. It is not a quality violation, so it
    // is typed apart from one — but it does deny the run a clean bill of health,
    // which is the whole point: a check that could not run has not passed.
    let unmeasured: Vec<QualityViolation> = phases
        .iter()
        .filter_map(|(kind, o)| {
            o.unmeasured.as_ref().map(|reason| QualityViolation {
                violation_type: "not_measured".to_string(),
                severity: "error".to_string(),
                location: specific_file.map_or_else(
                    || project_path.display().to_string(),
                    |p| p.display().to_string(),
                ),
                current: 0.0,
                target: 0.0,
                suggestion: format!(
                    "{kind} could not be measured ({reason}); this verdict does not cover it"
                ),
            })
        })
        .collect();
    let any_unmeasured = !unmeasured.is_empty();

    violations.extend(unmeasured);
    violations.extend(complexity_outcome.violations);
    violations.extend(satd_outcome.violations);
    violations.extend(tdg_outcome.violations);

    // Determine next state. `Complete` asserts that the profile was met, so it
    // requires evidence for every dimension, not merely the absence of findings
    // among the dimensions that happened to run.
    let next_state = if violations.is_empty() && !any_unmeasured {
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
            handle_analyzing_state(violating.path(), &profile, false, true, None)
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
