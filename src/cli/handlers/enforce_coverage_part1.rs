    // ========== Test Fixtures ==========

    /// Create a test project directory with source files
    fn create_test_project() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create a simple Rust file for testing
        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir_all(&src_dir).expect("Failed to create src dir");

        let rust_file = src_dir.join("lib.rs");
        std::fs::write(
            &rust_file,
            r#"
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
/// Hello.
pub fn hello() {
    println!("Hello, world!");
}

/// Complex function.
pub fn complex_function(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            if x > 100 {
                x * 3
            } else {
                x * 2
            }
        } else {
            x + 1
        }
    } else {
        0
    }
}
"#,
        )
        .expect("Failed to write test file");

        temp_dir
    }

    /// Create a test project with Cargo.toml for more realistic testing
    fn create_test_project_with_cargo() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create src directory
        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir_all(&src_dir).expect("Failed to create src dir");

        // Create Cargo.toml
        let cargo_toml = temp_dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
        )
        .expect("Failed to write Cargo.toml");

        // Create lib.rs with various code patterns
        let rust_file = src_dir.join("lib.rs");
        std::fs::write(
            &rust_file,
            r#"
//! Test library for enforce handlers

// Placeholder: Add documentation
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn hello() -> String {
    "Hello, world!".to_string()
}

// Note: This function has high cyclomatic complexity
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
/// Complex function.
pub fn complex_function(x: i32, y: i32) -> i32 {
    if x > 0 {
        if y > 0 {
            if x > y {
                x * 2
            } else {
                y * 2
            }
        } else {
            x
        }
    } else if y > 0 {
        y
    } else {
        0
    }
}

fn unused_function() {
    println!("This is unused");
}

mod tests {
    use super::*;

    #[test]
    fn test_hello() {
        assert_eq!(hello(), "Hello, world!");
    }
}
"#,
        )
        .expect("Failed to write test file");

        temp_dir
    }

    /// Create a QualityProfile for testing
    fn make_test_profile() -> QualityProfile {
        QualityProfile {
            coverage_min: 80.0,
            complexity_max: 20,
            complexity_target: 10,
            tdg_max: 1.0,
            satd_allowed: 0,
            duplication_max_lines: 0,
            big_o_max: "O(n)".to_string(),
            provability_min: 0.9,
        }
    }

    /// Create a relaxed QualityProfile for testing
    fn make_relaxed_profile() -> QualityProfile {
        QualityProfile {
            coverage_min: 50.0,
            complexity_max: 50,
            complexity_target: 30,
            tdg_max: 5.0,
            satd_allowed: 10,
            duplication_max_lines: 50,
            big_o_max: "O(n^2)".to_string(),
            provability_min: 0.5,
        }
    }

    /// Create a test EnforcementConfig
    fn make_test_enforcement_config() -> EnforcementConfig {
        EnforcementConfig {
            max_iterations: 5,
            target_improvement: None,
            max_time: Some(60),
            apply_suggestions: false,
            specific_file: None,
            include_pattern: None,
            exclude_pattern: None,
            single_file_mode: false,
            dry_run: true,
            show_progress: false,
            format: EnforceOutputFormat::Summary,
            ci_mode: false,
        }
    }

    /// Create a test EnforcementConfig with all options enabled
    fn make_full_enforcement_config() -> EnforcementConfig {
        EnforcementConfig {
            max_iterations: 10,
            target_improvement: Some(0.1),
            max_time: Some(120),
            apply_suggestions: true,
            specific_file: Some(PathBuf::from("test.rs")),
            include_pattern: Some("*.rs".to_string()),
            exclude_pattern: Some("*_test.rs".to_string()),
            single_file_mode: true,
            dry_run: false,
            show_progress: true,
            format: EnforceOutputFormat::Json,
            ci_mode: true,
        }
    }

    /// Create a test violation
    fn make_test_violation(violation_type: &str, severity: &str) -> QualityViolation {
        QualityViolation {
            violation_type: violation_type.to_string(),
            severity: severity.to_string(),
            location: "test.rs:10".to_string(),
            current: 25.0,
            target: 10.0,
            suggestion: "Reduce complexity by extracting functions".to_string(),
        }
    }

    /// Create a test violation with custom values
    fn make_custom_violation(
        violation_type: &str,
        severity: &str,
        location: &str,
        current: f64,
        target: f64,
    ) -> QualityViolation {
        QualityViolation {
            violation_type: violation_type.to_string(),
            severity: severity.to_string(),
            location: location.to_string(),
            current,
            target,
            suggestion: format!("Fix {} at {}", violation_type, location),
        }
    }

    // ========== EnforcementState Tests ==========

    mod enforcement_state_tests {
        use super::*;

        #[test]
        fn test_state_analyzing() {
            let state = EnforcementState::Analyzing;
            assert_eq!(serde_json::to_string(&state).unwrap(), "\"ANALYZING\"");
        }

        #[test]
        fn test_state_violating() {
            let state = EnforcementState::Violating;
            assert_eq!(serde_json::to_string(&state).unwrap(), "\"VIOLATING\"");
        }

        #[test]
        fn test_state_refactoring() {
            let state = EnforcementState::Refactoring;
            assert_eq!(serde_json::to_string(&state).unwrap(), "\"REFACTORING\"");
        }

        #[test]
        fn test_state_validating() {
            let state = EnforcementState::Validating;
            assert_eq!(serde_json::to_string(&state).unwrap(), "\"VALIDATING\"");
        }

        #[test]
        fn test_state_complete() {
            let state = EnforcementState::Complete;
            assert_eq!(serde_json::to_string(&state).unwrap(), "\"COMPLETE\"");
        }

        #[test]
        fn test_state_deserialize() {
            let state: EnforcementState = serde_json::from_str("\"ANALYZING\"").unwrap();
            assert_eq!(state, EnforcementState::Analyzing);
        }

        #[test]
        fn test_state_equality() {
            assert_eq!(EnforcementState::Complete, EnforcementState::Complete);
            assert_ne!(EnforcementState::Analyzing, EnforcementState::Complete);
        }
    }

    // ========== QualityProfile Tests ==========

    mod quality_profile_tests {
        use super::*;

        #[test]
        fn test_default_profile() {
            let profile = QualityProfile::default();
            assert_eq!(profile.coverage_min, 80.0);
            assert_eq!(profile.complexity_max, 20);
            assert_eq!(profile.complexity_target, 10);
            assert_eq!(profile.tdg_max, 1.0);
            assert_eq!(profile.satd_allowed, 0);
            assert_eq!(profile.duplication_max_lines, 0);
            assert_eq!(profile.big_o_max, "O(n)");
            assert_eq!(profile.provability_min, 0.9);
        }

        #[test]
        fn test_custom_profile() {
            let profile = QualityProfile {
                coverage_min: 95.0,
                complexity_max: 15,
                complexity_target: 8,
                tdg_max: 0.5,
                satd_allowed: 5,
                duplication_max_lines: 10,
                big_o_max: "O(log n)".to_string(),
                provability_min: 0.95,
            };

            assert_eq!(profile.coverage_min, 95.0);
            assert_eq!(profile.complexity_max, 15);
        }

        #[test]
        fn test_profile_serialization() {
            let profile = QualityProfile::default();
            let json = serde_json::to_string(&profile).unwrap();
            assert!(json.contains("coverage_min"));
            assert!(json.contains("80"));
            assert!(json.contains("complexity_max"));
            assert!(json.contains("20"));
        }

        #[test]
        fn test_profile_deserialization() {
            let json = r#"{
                "coverage_min": 90.0,
                "complexity_max": 15,
                "complexity_target": 8,
                "tdg_max": 0.8,
                "satd_allowed": 2,
                "duplication_max_lines": 5,
                "big_o_max": "O(n)",
                "provability_min": 0.85
            }"#;

            let profile: QualityProfile = serde_json::from_str(json).unwrap();
            assert_eq!(profile.coverage_min, 90.0);
            assert_eq!(profile.complexity_max, 15);
            assert_eq!(profile.satd_allowed, 2);
        }
    }

    // ========== QualityViolation Tests ==========

    mod quality_violation_tests {
        use super::*;

        #[test]
        fn test_violation_creation() {
            let violation = make_test_violation("complexity", "high");
            assert_eq!(violation.violation_type, "complexity");
            assert_eq!(violation.severity, "high");
            assert_eq!(violation.location, "test.rs:10");
            assert_eq!(violation.current, 25.0);
            assert_eq!(violation.target, 10.0);
        }

        #[test]
        fn test_violation_serialization() {
            let violation = make_test_violation("satd", "medium");
            let json = serde_json::to_string(&violation).unwrap();
            assert!(json.contains("satd"));
            assert!(json.contains("medium"));
            assert!(json.contains("test.rs:10"));
        }

        #[test]
        fn test_violation_deserialization() {
            let json = r#"{
                "violation_type": "coverage",
                "severity": "high",
                "location": "src/lib.rs:25",
                "current": 65.0,
                "target": 80.0,
                "suggestion": "Add more tests"
            }"#;

            let violation: QualityViolation = serde_json::from_str(json).unwrap();
            assert_eq!(violation.violation_type, "coverage");
            assert_eq!(violation.current, 65.0);
            assert_eq!(violation.target, 80.0);
        }
    }

    // ========== EnforcementProgress Tests ==========

    mod enforcement_progress_tests {
        use super::*;

        #[test]
        fn test_progress_creation() {
            let progress = EnforcementProgress {
                files_completed: 10,
                files_remaining: 5,
                estimated_iterations: 3,
            };

            assert_eq!(progress.files_completed, 10);
            assert_eq!(progress.files_remaining, 5);
            assert_eq!(progress.estimated_iterations, 3);
        }

        #[test]
        fn test_progress_serialization() {
            let progress = EnforcementProgress {
                files_completed: 50,
                files_remaining: 25,
                estimated_iterations: 2,
            };

            let json = serde_json::to_string(&progress).unwrap();
            assert!(json.contains("50"));
            assert!(json.contains("25"));
            assert!(json.contains("2"));
        }
    }

    // ========== EnforcementResult Tests ==========

    mod enforcement_result_tests {
        use super::*;

        #[test]
        fn test_result_creation() {
            let result = EnforcementResult {
                state: EnforcementState::Analyzing,
                score: 0.75,
                target: 1.0,
                current_file: Some("test.rs".to_string()),
                violations: vec![],
                next_action: "analyze".to_string(),
                progress: EnforcementProgress {
                    files_completed: 0,
                    files_remaining: 10,
                    estimated_iterations: 5,
                },
            };

            assert_eq!(result.state, EnforcementState::Analyzing);
            assert_eq!(result.score, 0.75);
            assert_eq!(result.target, 1.0);
        }

        #[test]
        fn test_result_with_violations() {
            let violations = vec![
                make_test_violation("complexity", "high"),
                make_test_violation("satd", "medium"),
            ];

            let result = EnforcementResult {
                state: EnforcementState::Violating,
                score: 0.5,
                target: 1.0,
                current_file: None,
                violations: violations.clone(),
                next_action: "fix_violations".to_string(),
                progress: EnforcementProgress {
                    files_completed: 5,
                    files_remaining: 15,
                    estimated_iterations: 10,
                },
            };

            assert_eq!(result.violations.len(), 2);
            assert_eq!(result.state, EnforcementState::Violating);
        }

        #[test]
        fn test_result_serialization() {
            let result = EnforcementResult {
                state: EnforcementState::Complete,
                score: 1.0,
                target: 1.0,
                current_file: None,
                violations: vec![],
                next_action: "none".to_string(),
                progress: EnforcementProgress {
                    files_completed: 100,
                    files_remaining: 0,
                    estimated_iterations: 0,
                },
            };

            let json = serde_json::to_string(&result).unwrap();
            assert!(json.contains("COMPLETE"));
            assert!(json.contains("1.0") || json.contains("1")); // Allow both formats
            assert!(json.contains("none"));
        }
    }

    // ========== Helper Function Tests ==========

    mod helper_function_tests {
        use super::*;

        #[test]
        fn test_load_quality_profile_extreme() {
            let profile = load_quality_profile("extreme", None).unwrap();
            assert_eq!(profile.coverage_min, 80.0);
        }

        #[test]
        fn test_load_quality_profile_default() {
            let profile = load_quality_profile("default", None).unwrap();
            // Should return extreme profile as default
            assert_eq!(profile.coverage_min, 80.0);
        }

        #[test]
        fn test_should_continue_enforcement_complete() {
            let config = make_test_enforcement_config();
            let start_time = Instant::now();
            let result =
                should_continue_enforcement(EnforcementState::Complete, 0, &config, start_time);
            assert!(!result);
        }
    }
