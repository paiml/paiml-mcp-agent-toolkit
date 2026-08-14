#![cfg_attr(coverage_nightly, coverage(off))]
//! Tests for enforce handlers

// Imports at file-level scope so `use super::*` in included test files works
#[allow(unused_imports)]
use super::{
    check_improvement_targets, clear_enforcement_cache, execute_enforcement_iteration,
    finalize_enforcement_run, format_violations_output, handle_analyzing_enforcement_state,
    handle_analyzing_state, handle_complete_enforcement_state, handle_complete_state,
    handle_enforcement_iteration, handle_refactoring_enforcement_state, handle_refactoring_state,
    handle_special_modes, handle_validating_enforcement_state,
    handle_violating_enforcement_state_proxy, handle_violating_state,
    initialize_enforcement_environment, load_quality_profile, output_result,
    print_enforcement_header, print_enforcement_summary, print_progress_bar,
    run_complexity_analysis, run_coverage_analysis, run_dead_code_analysis,
    run_duplication_analysis, run_satd_analysis, run_tdg_analysis, should_continue_enforcement,
    should_stop_for_target_improvement, EnforcementConfig, EnforcementIterationResult,
    EnforcementLoopResult, EnforcementProgress, EnforcementResult, EnforcementState,
    QualityProfile, QualityViolation,
};

#[allow(unused_imports)]
use crate::cli::EnforceOutputFormat;

#[allow(unused_imports)]
use std::collections::HashMap;
#[allow(unused_imports)]
use std::path::{Path, PathBuf};
#[allow(unused_imports)]
use std::time::{Duration, Instant};

// External tests - split file boundaries fixed
#[cfg(test)]
#[path = "../enforce_handlers_tests.rs"]
mod enforce_tests_external;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use crate::cli::handlers::enforce_handlers::{
        clear_enforcement_cache, handle_complete_state, handle_violating_state,
        load_quality_profile, output_result, should_continue_enforcement, EnforcementConfig,
        EnforcementProgress, EnforcementResult, EnforcementState, QualityProfile, QualityViolation,
    };
    use crate::cli::EnforceOutputFormat;

    #[test]
    fn test_quality_profile_default() {
        let profile = QualityProfile::default();
        assert_eq!(profile.coverage_min, 80.0);
        assert_eq!(profile.complexity_max, 20);
        assert_eq!(profile.duplication_max_lines, 0);
    }

    #[test]
    fn test_enforcement_state_serde_roundtrip() {
        let states = vec![
            EnforcementState::Analyzing,
            EnforcementState::Violating,
            EnforcementState::Refactoring,
            EnforcementState::Validating,
            EnforcementState::Complete,
        ];
        for state in states {
            let json = serde_json::to_string(&state).unwrap();
            let back: EnforcementState = serde_json::from_str(&json).unwrap();
            assert_eq!(
                std::mem::discriminant(&state),
                std::mem::discriminant(&back)
            );
        }
    }

    #[test]
    fn test_quality_violation_serde() {
        let v = QualityViolation {
            violation_type: "complexity".to_string(),
            severity: "high".to_string(),
            location: "src/main.rs:42".to_string(),
            current: 25.0,
            target: 20.0,
            suggestion: "Extract method".to_string(),
        };
        let json = serde_json::to_string(&v).unwrap();
        let back: QualityViolation = serde_json::from_str(&json).unwrap();
        assert_eq!(back.violation_type, "complexity");
        assert_eq!(back.current, 25.0);
    }

    #[test]
    fn test_enforcement_result_serde() {
        let r = EnforcementResult {
            state: EnforcementState::Complete,
            score: 95.0,
            target: 1.0,
            current_file: None,
            violations: vec![],
            next_action: "none".to_string(),
            progress: EnforcementProgress {
                files_completed: 0,
                files_remaining: 0,
                estimated_iterations: 0,
            },
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: EnforcementResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.score, 95.0);
        assert!(back.violations.is_empty());
    }

    #[test]
    fn test_should_continue_enforcement_complete_returns_false() {
        let config = EnforcementConfig {
            max_iterations: 10,
            target_improvement: None,
            max_time: None,
            apply_suggestions: false,
            specific_file: None,
            include_pattern: None,
            exclude_pattern: None,
            single_file_mode: false,
            dry_run: false,
            show_progress: false,
            format: EnforceOutputFormat::Summary,
            ci_mode: false,
        };
        let start = std::time::Instant::now();
        let result = should_continue_enforcement(EnforcementState::Complete, 0, &config, start);
        assert!(!result, "Complete state should stop enforcement");
    }

    #[test]
    fn test_should_continue_enforcement_max_iterations() {
        let config = EnforcementConfig {
            max_iterations: 5,
            target_improvement: None,
            max_time: None,
            apply_suggestions: false,
            specific_file: None,
            include_pattern: None,
            exclude_pattern: None,
            single_file_mode: false,
            dry_run: false,
            show_progress: false,
            format: EnforceOutputFormat::Summary,
            ci_mode: false,
        };
        let start = std::time::Instant::now();
        let result = should_continue_enforcement(EnforcementState::Analyzing, 5, &config, start);
        assert!(!result, "Should stop at max iterations");
    }

    #[test]
    fn test_should_continue_enforcement_analyzing_continues() {
        let config = EnforcementConfig {
            max_iterations: 10,
            target_improvement: None,
            max_time: None,
            apply_suggestions: false,
            specific_file: None,
            include_pattern: None,
            exclude_pattern: None,
            single_file_mode: false,
            dry_run: false,
            show_progress: false,
            format: EnforceOutputFormat::Summary,
            ci_mode: false,
        };
        let start = std::time::Instant::now();
        let result = should_continue_enforcement(EnforcementState::Analyzing, 0, &config, start);
        assert!(result, "Analyzing with 0 iterations should continue");
    }

    #[test]
    fn test_load_quality_profile_extreme() {
        let profile = load_quality_profile("extreme", None).unwrap();
        assert_eq!(profile.coverage_min, 80.0);
    }

    #[test]
    fn test_load_quality_profile_rejects_unknown_name() {
        // "default" is not one of the three profiles clap accepts. It used to
        // return the extreme profile, which is how every name — including the
        // ones `--help` documents as looser — enforced the extreme thresholds.
        assert!(load_quality_profile("default", None).is_err());
    }

    #[test]
    fn test_handle_complete_state() {
        let result = handle_complete_state().unwrap();
        assert!(matches!(result.state, EnforcementState::Complete));
        assert!(result.violations.is_empty());
    }

    // `test_handle_refactoring_state_no_file` / `_with_file` used to live here.
    // Both asserted `result.score > <input>` — that the refactoring state adds
    // 0.1 to a score after refactoring nothing — which is the defect, not the
    // contract. The enforcement path runs `handle_refactoring_pass`, whose
    // numbers come from the one assessment; see
    // `surface_agreement_tests::the_refactoring_state_reports_the_run_it_did_not_change`.
    // Five more tests pinning the same arithmetic remain in
    // `../enforce_coverage_part2.rs`, `../enforce_coverage_part4.rs` and
    // `../enforce_coverage_part3_state_tests.rs`.

    #[test]
    fn test_handle_violating_state_empty_violations() {
        let result = handle_violating_state(vec![], 75.0, false, false, None).unwrap();
        assert_eq!(result.score, 75.0);
    }

    #[test]
    fn test_handle_violating_state_with_violations() {
        let violations = vec![QualityViolation {
            violation_type: "complexity".to_string(),
            severity: "high".to_string(),
            location: "test.rs:10".to_string(),
            current: 25.0,
            target: 20.0,
            suggestion: "Refactor".to_string(),
        }];
        let result = handle_violating_state(violations, 60.0, false, false, None).unwrap();
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.score, 60.0);
    }

    #[test]
    fn test_output_result_json() {
        let result = EnforcementResult {
            state: EnforcementState::Complete,
            score: 95.0,
            target: 1.0,
            current_file: None,
            violations: vec![],
            next_action: "none".to_string(),
            progress: EnforcementProgress {
                files_completed: 0,
                files_remaining: 0,
                estimated_iterations: 0,
            },
        };
        let out = output_result(&result, EnforceOutputFormat::Json, false, None);
        assert!(out.is_ok());
    }

    #[test]
    fn test_output_result_summary() {
        let result = EnforcementResult {
            state: EnforcementState::Analyzing,
            score: 50.0,
            target: 1.0,
            current_file: None,
            violations: vec![QualityViolation {
                violation_type: "coverage".to_string(),
                severity: "medium".to_string(),
                location: "test.rs".to_string(),
                current: 30.0,
                target: 80.0,
                suggestion: "Add more tests".to_string(),
            }],
            next_action: "analyze".to_string(),
            progress: EnforcementProgress {
                files_completed: 0,
                files_remaining: 1,
                estimated_iterations: 1,
            },
        };
        let out = output_result(&result, EnforceOutputFormat::Summary, false, None);
        assert!(out.is_ok());
    }

    #[test]
    fn test_quality_profile_serde_roundtrip() {
        let profile = QualityProfile {
            coverage_min: 90.0,
            complexity_max: 15,
            duplication_max_lines: 3,
            ..QualityProfile::default()
        };
        let json = serde_json::to_string(&profile).unwrap();
        let back: QualityProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(back.coverage_min, 90.0);
        assert_eq!(back.complexity_max, 15);
    }

    #[test]
    fn test_clear_enforcement_cache_none() {
        // No --cache-dir: enforce keeps no cache of its own, so there is
        // nothing to clear and that is reported, not an error.
        clear_enforcement_cache(&None).expect("no cache dir is not a failure");
    }

    /// The stub version returned `()` after printing "🧹 Clearing cache at: DIR"
    /// and running `// In real implementation, would clear cache`, so the
    /// directory it named still held every entry when the run finished.
    #[test]
    fn test_clear_enforcement_cache_deletes_entries() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("entry.bin"), b"stale").expect("write entry");
        std::fs::create_dir(dir.path().join("sub")).expect("mkdir");
        std::fs::write(dir.path().join("sub/nested.bin"), b"stale").expect("write nested");

        clear_enforcement_cache(&Some(dir.path().to_path_buf())).expect("clear");

        assert!(
            !dir.path().join("entry.bin").exists(),
            "--clear-cache must delete cache files"
        );
        assert!(
            !dir.path().join("sub").exists(),
            "--clear-cache must delete cache subdirectories"
        );
        assert!(dir.path().is_dir(), "the cache directory itself is kept");
    }
}
