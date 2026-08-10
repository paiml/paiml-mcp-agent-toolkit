// QDD handlers tests extracted from qdd_handlers.rs for file health (CB-040).
// This file is include!()'d into qdd_handlers.rs scope.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use crate::cli::commands::{QddCodeType, QddOutputFormat, QddQualityProfile};
    use crate::qdd::{Checkpoint, CodeType, QddResult, QualityMetrics, QualityScore, RollbackPlan};
    use proptest::prelude::*;
    use std::fs;
    use tempfile::TempDir;

    // ===============================================================
    // Tests for convert_code_type
    // ===============================================================

    #[test]
    fn test_convert_code_type_function() {
        let result = convert_code_type(QddCodeType::Function);
        assert!(matches!(result, CodeType::Function));
    }

    #[test]
    fn test_convert_code_type_module() {
        let result = convert_code_type(QddCodeType::Module);
        assert!(matches!(result, CodeType::Module));
    }

    #[test]
    fn test_convert_code_type_service() {
        let result = convert_code_type(QddCodeType::Service);
        assert!(matches!(result, CodeType::Service));
    }

    #[test]
    fn test_convert_code_type_test() {
        let result = convert_code_type(QddCodeType::Test);
        assert!(matches!(result, CodeType::Test));
    }

    // ===============================================================
    // Tests for convert_quality_profile
    // ===============================================================

    #[test]
    fn test_convert_quality_profile_extreme() {
        let result = convert_quality_profile(QddQualityProfile::Extreme);
        assert_eq!(result.name, "extreme");
        assert_eq!(result.thresholds.max_complexity, 5);
        assert_eq!(result.thresholds.min_coverage, 90);
    }

    #[test]
    fn test_convert_quality_profile_standard() {
        let result = convert_quality_profile(QddQualityProfile::Standard);
        assert_eq!(result.name, "standard");
        assert_eq!(result.thresholds.max_complexity, 10);
        assert_eq!(result.thresholds.min_coverage, 80);
    }

    #[test]
    fn test_convert_quality_profile_relaxed() {
        let result = convert_quality_profile(QddQualityProfile::Relaxed);
        assert_eq!(result.name, "relaxed");
        assert_eq!(result.thresholds.max_complexity, 20);
        assert_eq!(result.thresholds.min_coverage, 60);
    }

    // ===============================================================
    // Tests for convert_parameters
    // ===============================================================

    #[test]
    fn test_convert_parameters_empty() {
        let result = convert_parameters(vec![]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_convert_parameters_single() {
        let inputs = vec![("String".to_string(), "name".to_string())];
        let result = convert_parameters(inputs);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "name");
        assert_eq!(result[0].param_type, "String");
        assert!(result[0].description.is_none());
    }

    #[test]
    fn test_convert_parameters_multiple() {
        let inputs = vec![
            ("i32".to_string(), "count".to_string()),
            ("String".to_string(), "message".to_string()),
            ("bool".to_string(), "enabled".to_string()),
        ];
        let result = convert_parameters(inputs);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].name, "count");
        assert_eq!(result[0].param_type, "i32");
        assert_eq!(result[1].name, "message");
        assert_eq!(result[1].param_type, "String");
        assert_eq!(result[2].name, "enabled");
        assert_eq!(result[2].param_type, "bool");
    }

    // ===============================================================
    // Tests for build_create_spec
    // ===============================================================

    #[test]
    fn test_build_create_spec_basic() {
        let code_type = CodeType::Function;
        let name = "calculate_sum".to_string();
        let purpose = "Calculate the sum of two numbers".to_string();
        let inputs = vec![Parameter {
            name: "a".to_string(),
            param_type: "i32".to_string(),
            description: None,
        }];
        let output_type = "i32".to_string();

        let spec = build_create_spec(
            code_type,
            name.clone(),
            purpose.clone(),
            inputs.clone(),
            output_type.clone(),
        );

        assert!(matches!(spec.code_type, CodeType::Function));
        assert_eq!(spec.name, name);
        assert_eq!(spec.purpose, purpose);
        assert_eq!(spec.inputs.len(), 1);
        assert_eq!(spec.outputs.name, "result");
        assert_eq!(spec.outputs.param_type, output_type);
        assert_eq!(
            spec.outputs.description,
            Some("Function output".to_string())
        );
    }

    #[test]
    fn test_build_create_spec_module_type() {
        let spec = build_create_spec(
            CodeType::Module,
            "my_module".to_string(),
            "A utility module".to_string(),
            vec![],
            "()".to_string(),
        );

        assert!(matches!(spec.code_type, CodeType::Module));
        assert_eq!(spec.name, "my_module");
    }

    #[test]
    fn test_build_create_spec_service_type() {
        let spec = build_create_spec(
            CodeType::Service,
            "AuthService".to_string(),
            "Authentication service".to_string(),
            vec![],
            "Result<User>".to_string(),
        );

        assert!(matches!(spec.code_type, CodeType::Service));
        assert_eq!(spec.name, "AuthService");
    }

    #[test]
    fn test_build_create_spec_test_type() {
        let spec = build_create_spec(
            CodeType::Test,
            "test_calculation".to_string(),
            "Test for calculation function".to_string(),
            vec![],
            "()".to_string(),
        );

        assert!(matches!(spec.code_type, CodeType::Test));
    }

    // ===============================================================
    // Tests for validate_file_exists
    // ===============================================================

    #[test]
    fn test_validate_file_exists_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        fs::write(&file_path, "fn main() {}").unwrap();

        let result = validate_file_exists(&file_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_file_exists_failure() {
        let non_existent = PathBuf::from("/non/existent/path/file.rs");
        let result = validate_file_exists(&non_existent);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("File does not exist"));
    }

    // ===============================================================
    // Tests for create_quality_profile
    // ===============================================================

    #[test]
    fn test_create_quality_profile_extreme_no_overrides() {
        let profile = create_quality_profile(QddQualityProfile::Extreme, None, None);
        assert_eq!(profile.thresholds.max_complexity, 5);
        assert_eq!(profile.thresholds.min_coverage, 90);
    }

    #[test]
    fn test_create_quality_profile_standard_no_overrides() {
        let profile = create_quality_profile(QddQualityProfile::Standard, None, None);
        assert_eq!(profile.thresholds.max_complexity, 10);
        assert_eq!(profile.thresholds.min_coverage, 80);
    }

    #[test]
    fn test_create_quality_profile_relaxed_no_overrides() {
        let profile = create_quality_profile(QddQualityProfile::Relaxed, None, None);
        assert_eq!(profile.thresholds.max_complexity, 20);
        assert_eq!(profile.thresholds.min_coverage, 60);
    }

    #[test]
    fn test_create_quality_profile_with_complexity_override() {
        let profile = create_quality_profile(QddQualityProfile::Standard, Some(15), None);
        assert_eq!(profile.thresholds.max_complexity, 15);
        assert_eq!(profile.thresholds.min_coverage, 80); // unchanged
    }

    #[test]
    fn test_create_quality_profile_with_coverage_override() {
        let profile = create_quality_profile(QddQualityProfile::Standard, None, Some(95));
        assert_eq!(profile.thresholds.max_complexity, 10); // unchanged
        assert_eq!(profile.thresholds.min_coverage, 95);
    }

    #[test]
    fn test_create_quality_profile_with_both_overrides() {
        let profile = create_quality_profile(QddQualityProfile::Relaxed, Some(8), Some(85));
        assert_eq!(profile.thresholds.max_complexity, 8);
        assert_eq!(profile.thresholds.min_coverage, 85);
    }

    // ===============================================================
    // Tests for create_refactor_spec
    // ===============================================================

    #[test]
    fn test_create_refactor_spec_without_function() {
        let file = PathBuf::from("/path/to/file.rs");
        let profile = QualityProfile::standard();

        let spec = create_refactor_spec(&file, None, &profile);

        assert_eq!(spec.file_path, file);
        assert!(spec.function_name.is_none());
        assert_eq!(
            spec.target_metrics.max_complexity,
            profile.thresholds.max_complexity
        );
    }

    #[test]
    fn test_create_refactor_spec_with_function() {
        let file = PathBuf::from("/path/to/file.rs");
        let profile = QualityProfile::extreme();
        let function = Some("process_data".to_string());

        let spec = create_refactor_spec(&file, function.clone(), &profile);

        assert_eq!(spec.file_path, file);
        assert_eq!(spec.function_name, function);
        assert_eq!(spec.target_metrics.min_coverage, 90);
    }

    // ===============================================================
    // Tests for display_create_results (output verification)
    // ===============================================================

    #[test]
    fn test_display_create_results_does_not_panic() {
        let result = QddResult {
            code: "fn test() {}".to_string(),
            tests: "#[test] fn test_test() {}".to_string(),
            documentation: "/// Test function".to_string(),
            quality_score: QualityScore {
                overall: 85.0,
                complexity: 5,
                coverage: 90.0,
                tdg: 2,
            },
            metrics: QualityMetrics::default(),
            rollback_plan: RollbackPlan {
                original: String::new(),
                checkpoints: vec![],
            },
        };

        // This should not panic
        display_create_results(QddQualityProfile::Standard, &result);
    }

    #[test]
    fn test_display_create_results_extreme_profile() {
        let result = QddResult {
            code: "fn extreme() {}".to_string(),
            tests: "".to_string(),
            documentation: "".to_string(),
            quality_score: QualityScore {
                overall: 95.0,
                complexity: 2,
                coverage: 98.0,
                tdg: 1,
            },
            metrics: QualityMetrics::default(),
            rollback_plan: RollbackPlan {
                original: String::new(),
                checkpoints: vec![],
            },
        };

        display_create_results(QddQualityProfile::Extreme, &result);
    }

    // ===============================================================
    // Tests for output_generated_code
    // ===============================================================

    #[test]
    fn test_output_generated_code_to_file() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("output.rs");

        let result = QddResult {
            code: "fn main() { println!(\"Hello\"); }".to_string(),
            tests: "#[test] fn test_main() { assert!(true); }".to_string(),
            documentation: "/// Main function".to_string(),
            quality_score: QualityScore {
                overall: 80.0,
                complexity: 3,
                coverage: 85.0,
                tdg: 2,
            },
            metrics: QualityMetrics::default(),
            rollback_plan: RollbackPlan {
                original: String::new(),
                checkpoints: vec![],
            },
        };

        let res = output_generated_code(Some(output_path.clone()), &result);
        assert!(res.is_ok());

        let content = fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("fn main()"));
        assert!(content.contains("#[test]"));
        assert!(content.contains("/// Main function"));
    }

    #[test]
    fn test_output_generated_code_to_stdout() {
        let result = QddResult {
            code: "fn stdout_test() {}".to_string(),
            tests: "#[test] fn t() {}".to_string(),
            documentation: "/// Doc".to_string(),
            quality_score: QualityScore {
                overall: 75.0,
                complexity: 4,
                coverage: 80.0,
                tdg: 3,
            },
            metrics: QualityMetrics::default(),
            rollback_plan: RollbackPlan {
                original: String::new(),
                checkpoints: vec![],
            },
        };

        // Should not panic when output_file is None
        let res = output_generated_code(None, &result);
        assert!(res.is_ok());
    }

    // ===============================================================
    // Tests for display_refactor_results
    // ===============================================================

    #[test]
    fn test_display_refactor_results_without_function() {
        let file = PathBuf::from("/path/to/file.rs");
        let result = QddResult {
            code: "refactored code".to_string(),
            tests: "".to_string(),
            documentation: "".to_string(),
            quality_score: QualityScore {
                overall: 88.0,
                complexity: 6,
                coverage: 92.0,
                tdg: 2,
            },
            metrics: QualityMetrics::default(),
            rollback_plan: RollbackPlan {
                original: String::new(),
                checkpoints: vec![],
            },
        };

        // Should not panic
        display_refactor_results(&file, None, QddQualityProfile::Standard, &result);
    }

    #[test]
    fn test_display_refactor_results_with_function() {
        let file = PathBuf::from("/path/to/file.rs");
        let result = QddResult {
            code: "refactored code".to_string(),
            tests: "".to_string(),
            documentation: "".to_string(),
            quality_score: QualityScore {
                overall: 90.0,
                complexity: 4,
                coverage: 95.0,
                tdg: 1,
            },
            metrics: QualityMetrics::default(),
            rollback_plan: RollbackPlan {
                original: String::new(),
                checkpoints: vec![],
            },
        };

        display_refactor_results(
            &file,
            Some("process_data".to_string()),
            QddQualityProfile::Extreme,
            &result,
        );
    }

    // ===============================================================
    // Tests for save_refactored_code
    // ===============================================================

    #[test]
    fn test_save_refactored_code_success() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("refactored.rs");
        let code = "fn refactored() { todo!() }";

        let result = save_refactored_code(&output_path, code);
        assert!(result.is_ok());

        let content = fs::read_to_string(&output_path).unwrap();
        assert_eq!(content, code);
    }

    #[test]
    fn test_save_refactored_code_invalid_path() {
        let invalid_path = PathBuf::from("/non/existent/directory/file.rs");
        let code = "fn test() {}";

        let result = save_refactored_code(&invalid_path, code);
        assert!(result.is_err());
    }

    // ===============================================================
    // Tests for display_rollback_info
    // ===============================================================

    #[test]
    fn test_display_rollback_info_empty_checkpoints() {
        let result = QddResult {
            code: "".to_string(),
            tests: "".to_string(),
            documentation: "".to_string(),
            quality_score: QualityScore {
                overall: 0.0,
                complexity: 0,
                coverage: 0.0,
                tdg: 0,
            },
            metrics: QualityMetrics::default(),
            rollback_plan: RollbackPlan {
                original: String::new(),
                checkpoints: vec![],
            },
        };

        // Should not panic and should not print anything
        display_rollback_info(&result);
    }

    #[test]
    fn test_display_rollback_info_with_checkpoints() {
        let result = QddResult {
            code: "".to_string(),
            tests: "".to_string(),
            documentation: "".to_string(),
            quality_score: QualityScore {
                overall: 0.0,
                complexity: 0,
                coverage: 0.0,
                tdg: 0,
            },
            metrics: QualityMetrics::default(),
            rollback_plan: RollbackPlan {
                original: "original code".to_string(),
                checkpoints: vec![
                    Checkpoint {
                        step: "step1".to_string(),
                        code: "code1".to_string(),
                        quality_metrics: QualityMetrics::default(),
                    },
                    Checkpoint {
                        step: "step2".to_string(),
                        code: "code2".to_string(),
                        quality_metrics: QualityMetrics::default(),
                    },
                ],
            },
        };

        // Should print rollback info
        display_rollback_info(&result);
    }

    // ===============================================================
    // Tests for handle_dry_run
    // ===============================================================

    #[test]
    fn test_handle_dry_run_without_function() {
        let file = PathBuf::from("/path/to/file.rs");
        let profile = QualityProfile::standard();

        let result = handle_dry_run(&file, &None, QddQualityProfile::Standard, &profile);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_dry_run_with_function() {
        let file = PathBuf::from("/path/to/file.rs");
        let profile = QualityProfile::extreme();
        let function = Some("my_function".to_string());

        let result = handle_dry_run(&file, &function, QddQualityProfile::Extreme, &profile);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_dry_run_relaxed_profile() {
        let file = PathBuf::from("/another/path.rs");
        let profile = QualityProfile::relaxed();

        let result = handle_dry_run(&file, &None, QddQualityProfile::Relaxed, &profile);
        assert!(result.is_ok());
    }

    // ===============================================================
    // Tests for handle_qdd_validate (async)
    // ===============================================================

    #[tokio::test]
    async fn test_handle_qdd_validate_summary_format() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();

        let result = handle_qdd_validate(
            path,
            QddQualityProfile::Standard,
            QddOutputFormat::Summary,
            None,
            false,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_qdd_validate_detailed_format() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();

        let result = handle_qdd_validate(
            path,
            QddQualityProfile::Extreme,
            QddOutputFormat::Detailed,
            None,
            false,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_qdd_validate_json_format() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();

        let result = handle_qdd_validate(
            path,
            QddQualityProfile::Relaxed,
            QddOutputFormat::Json,
            None,
            false,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_qdd_validate_markdown_format() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();

        let result = handle_qdd_validate(
            path,
            QddQualityProfile::Standard,
            QddOutputFormat::Markdown,
            None,
            false,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_qdd_validate_with_output_file() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();
        let output_file = temp_dir.path().join("report.txt");

        let result = handle_qdd_validate(
            path,
            QddQualityProfile::Standard,
            QddOutputFormat::Summary,
            Some(output_file),
            false,
        )
        .await;

        assert!(result.is_ok());
    }

    /// This test used to read "Since validation_passed is hardcoded to true,
    /// strict mode should pass" — it asserted the defect. An EMPTY directory
    /// measures nothing, and two of the four thresholds the header prints
    /// (coverage, TDG) are not measured at all, so strict mode must refuse
    /// rather than certify.
    #[tokio::test]
    async fn test_handle_qdd_validate_strict_mode_refuses_unmeasured_thresholds() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();

        let err = handle_qdd_validate(
            path,
            QddQualityProfile::Extreme,
            QddOutputFormat::Summary,
            None,
            true,
        )
        .await
        .expect_err("strict mode cannot pass on thresholds nothing measured");

        let rendered = format!("{err:#}");
        assert!(
            rendered.contains("not measured"),
            "the error must say what was not measured, got: {rendered}"
        );
    }

    /// The stub passed for a path that does not exist.
    #[tokio::test]
    async fn test_handle_qdd_validate_rejects_a_missing_path() {
        let missing = PathBuf::from("/does/not/exist.rs");
        let err = handle_qdd_validate(
            missing.clone(),
            QddQualityProfile::Standard,
            QddOutputFormat::Summary,
            None,
            false,
        )
        .await
        .expect_err("a path that does not exist must not be validated");
        assert!(format!("{err:#}").contains("/does/not/exist.rs"));
    }

    /// A file that violates the thresholds the command prints must FAIL. The
    /// stub reported "Validation Summary: PASSED" for this repository, whose own
    /// printed limits ("Max Complexity: 10, Zero SATD: true") it breaks.
    #[tokio::test]
    async fn test_handle_qdd_validate_fails_on_real_violations() {
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("hot.rs");
        let mut body = String::from("// TODO: this is self-admitted debt\nfn hot(n: i32) -> i32 {\n    let mut acc = 0;\n");
        for i in 0..30 {
            body.push_str(&format!("    if n > {i} {{ acc += {i}; }}\n"));
        }
        body.push_str("    acc\n}\n");
        std::fs::write(&file, body).unwrap();

        let outcome = run_validation_checks(&file, &QualityProfile::standard()).await;
        assert_eq!(outcome.status(), "failed", "{}", outcome.strict_reason());
        assert!(!outcome.passed());

        let json = build_validation_json(&outcome, QddQualityProfile::Standard, &file);
        assert_eq!(json["status"], "failed");
        let violations = json["violations"].as_array().expect("violations array");
        assert!(
            violations.len() >= 2,
            "complexity and SATD must both be reported: {violations:?}"
        );
    }

    /// Coverage and TDG are printed as thresholds but nothing here measures
    /// them, so they must be reported as unmeasured rather than PASSED.
    #[tokio::test]
    async fn test_handle_qdd_validate_never_claims_unmeasured_checks_passed() {
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("clean.rs");
        std::fs::write(&file, "fn clean() -> i32 {\n    1\n}\n").unwrap();

        let outcome = run_validation_checks(&file, &QualityProfile::standard()).await;
        assert_eq!(
            outcome.status(),
            "incomplete",
            "clean code with unmeasured thresholds is not a pass: {}",
            outcome.strict_reason()
        );

        let json = build_validation_json(&outcome, QddQualityProfile::Standard, &file);
        let unmeasured: Vec<String> = json["not_measured"]
            .as_array()
            .expect("not_measured array")
            .iter()
            .map(|v| v["check"].as_str().unwrap_or_default().to_string())
            .collect();
        assert!(unmeasured.contains(&"coverage".to_string()), "{unmeasured:?}");
        assert!(unmeasured.contains(&"tdg".to_string()), "{unmeasured:?}");
    }

    // ===============================================================
    // Tests for execute_create_operation (async)
    // ===============================================================

    #[tokio::test]
    async fn test_execute_create_operation() {
        let profile = QualityProfile::standard();
        let spec = CreateSpec {
            code_type: CodeType::Function,
            name: "test_func".to_string(),
            purpose: "A test function".to_string(),
            inputs: vec![],
            outputs: Parameter {
                name: "result".to_string(),
                param_type: "i32".to_string(),
                description: None,
            },
        };

        let result = execute_create_operation(profile, spec).await;
        assert!(result.is_ok());

        let qdd_result = result.unwrap();
        assert!(!qdd_result.code.is_empty());
    }

    // ===============================================================
    // Tests for execute_refactoring (async)
    // ===============================================================

    #[tokio::test]
    async fn test_execute_refactoring() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        fs::write(&file_path, "fn old_code() { println!(\"old\"); }").unwrap();

        let profile = QualityProfile::relaxed();
        let spec = RefactorSpec {
            file_path,
            function_name: None,
            target_metrics: profile.thresholds.clone(),
        };

        let result = execute_refactoring(profile, spec).await;
        assert!(result.is_ok());
    }

    // ===============================================================
    // Tests for handle_qdd_create (async)
    // ===============================================================

    #[tokio::test]
    async fn test_handle_qdd_create_function() {
        let temp_dir = TempDir::new().unwrap();
        let output_file = temp_dir.path().join("created.rs");

        let result = handle_qdd_create(
            QddCodeType::Function,
            "add_numbers".to_string(),
            "Add two numbers together".to_string(),
            QddQualityProfile::Standard,
            vec![
                ("i32".to_string(), "a".to_string()),
                ("i32".to_string(), "b".to_string()),
            ],
            "i32".to_string(),
            Some(output_file.clone()),
        )
        .await;

        assert!(result.is_ok());
        assert!(output_file.exists());
    }

    #[tokio::test]
    async fn test_handle_qdd_create_module() {
        let result = handle_qdd_create(
            QddCodeType::Module,
            "utils".to_string(),
            "Utility module".to_string(),
            QddQualityProfile::Relaxed,
            vec![],
            "()".to_string(),
            None,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_qdd_create_service() {
        let result = handle_qdd_create(
            QddCodeType::Service,
            "DataService".to_string(),
            "Data processing service".to_string(),
            QddQualityProfile::Extreme,
            vec![("String".to_string(), "input".to_string())],
            "Result<Data>".to_string(),
            None,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_qdd_create_test() {
        let result = handle_qdd_create(
            QddCodeType::Test,
            "test_addition".to_string(),
            "Test for addition function".to_string(),
            QddQualityProfile::Standard,
            vec![],
            "()".to_string(),
            None,
        )
        .await;

        assert!(result.is_ok());
    }

    // ===============================================================
    // Tests for handle_qdd_refactor (async)
    // ===============================================================

    #[tokio::test]
    async fn test_handle_qdd_refactor_dry_run() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("refactor_target.rs");
        fs::write(&file_path, "fn complex_function() { /* complex code */ }").unwrap();

        let result = handle_qdd_refactor(
            file_path,
            None,
            QddQualityProfile::Standard,
            None,
            None,
            None,
            true, // dry_run
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_qdd_refactor_dry_run_with_function() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("refactor_target.rs");
        fs::write(&file_path, "fn target_func() {}").unwrap();

        let result = handle_qdd_refactor(
            file_path,
            Some("target_func".to_string()),
            QddQualityProfile::Extreme,
            Some(5),
            Some(90),
            None,
            true,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_qdd_refactor_file_not_found() {
        let non_existent = PathBuf::from("/non/existent/file.rs");

        let result = handle_qdd_refactor(
            non_existent,
            None,
            QddQualityProfile::Standard,
            None,
            None,
            None,
            false,
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("File does not exist"));
    }

    #[tokio::test]
    async fn test_handle_qdd_refactor_actual_refactoring() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("to_refactor.rs");
        fs::write(&file_path, "fn needs_refactoring() { println!(\"old\"); }").unwrap();

        let output_path = temp_dir.path().join("refactored_output.rs");

        let result = handle_qdd_refactor(
            file_path,
            None,
            QddQualityProfile::Relaxed,
            Some(15),
            Some(70),
            Some(output_path.clone()),
            false,
        )
        .await;

        assert!(result.is_ok());
        assert!(output_path.exists());
    }

    #[tokio::test]
    async fn test_handle_qdd_refactor_with_function_name() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("source.rs");
        fs::write(&file_path, "fn specific_function() { todo!() }").unwrap();

        let result = handle_qdd_refactor(
            file_path.clone(),
            Some("specific_function".to_string()),
            QddQualityProfile::Standard,
            None,
            None,
            None, // Output to same file
            false,
        )
        .await;

        assert!(result.is_ok());
    }

    // ===============================================================
    // Tests for handle_qdd_command (async)
    // ===============================================================

    #[tokio::test]
    async fn test_handle_qdd_command_create() {
        let command = QddCommands::Create {
            code_type: QddCodeType::Function,
            name: "my_func".to_string(),
            purpose: "Do something".to_string(),
            profile: QddQualityProfile::Standard,
            input: vec![],
            output: "()".to_string(),
            output_file: None,
        };

        let result = handle_qdd_command(command).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_qdd_command_validate() {
        let temp_dir = TempDir::new().unwrap();

        let command = QddCommands::Validate {
            path: temp_dir.path().to_path_buf(),
            profile: QddQualityProfile::Relaxed,
            format: QddOutputFormat::Summary,
            output: None,
            strict: false,
        };

        let result = handle_qdd_command(command).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_qdd_command_refactor_dry_run() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("cmd_refactor.rs");
        fs::write(&file_path, "fn code() {}").unwrap();

        let command = QddCommands::Refactor {
            file: file_path,
            function: None,
            profile: QddQualityProfile::Standard,
            max_complexity: None,
            min_coverage: None,
            output: None,
            dry_run: true,
        };

        let result = handle_qdd_command(command).await;
        assert!(result.is_ok());
    }

    // ===============================================================
    // Property-based tests using proptest
    // ===============================================================

    proptest! {
        #[test]
        fn prop_convert_parameters_preserves_count(
            inputs in proptest::collection::vec(
                (proptest::string::string_regex("[a-zA-Z]+").unwrap(),
                 proptest::string::string_regex("[a-zA-Z]+").unwrap()),
                0..10
            )
        ) {
            let result = convert_parameters(inputs.clone());
            prop_assert_eq!(result.len(), inputs.len());
        }

        #[test]
        fn prop_convert_parameters_preserves_order(
            types in proptest::collection::vec(proptest::string::string_regex("[a-zA-Z]+").unwrap(), 1..5),
            names in proptest::collection::vec(proptest::string::string_regex("[a-zA-Z]+").unwrap(), 1..5)
        ) {
            let len = types.len().min(names.len());
            let inputs: Vec<(String, String)> = types.into_iter().zip(names.into_iter()).take(len).collect();
            let result = convert_parameters(inputs.clone());

            for (i, (param_type, name)) in inputs.iter().enumerate() {
                prop_assert_eq!(&result[i].param_type, param_type);
                prop_assert_eq!(&result[i].name, name);
            }
        }

        #[test]
        fn prop_build_create_spec_name_preserved(name in "[a-zA-Z_][a-zA-Z0-9_]*") {
            let spec = build_create_spec(
                CodeType::Function,
                name.clone(),
                "purpose".to_string(),
                vec![],
                "()".to_string(),
            );
            prop_assert_eq!(spec.name, name);
        }

        #[test]
        fn prop_build_create_spec_purpose_preserved(purpose in ".*") {
            let spec = build_create_spec(
                CodeType::Module,
                "name".to_string(),
                purpose.clone(),
                vec![],
                "()".to_string(),
            );
            prop_assert_eq!(spec.purpose, purpose);
        }

        #[test]
        fn prop_build_create_spec_output_type_preserved(output_type in "[a-zA-Z<>()]+") {
            let spec = build_create_spec(
                CodeType::Service,
                "name".to_string(),
                "purpose".to_string(),
                vec![],
                output_type.clone(),
            );
            prop_assert_eq!(spec.outputs.param_type, output_type);
        }

        #[test]
        fn prop_create_quality_profile_complexity_override(complexity in 1u32..100) {
            let profile = create_quality_profile(QddQualityProfile::Standard, Some(complexity), None);
            prop_assert_eq!(profile.thresholds.max_complexity, complexity);
        }

        #[test]
        fn prop_create_quality_profile_coverage_override(coverage in 1u32..100) {
            let profile = create_quality_profile(QddQualityProfile::Relaxed, None, Some(coverage));
            prop_assert_eq!(profile.thresholds.min_coverage, coverage);
        }

        #[test]
        fn prop_quality_profile_consistency(
            max_complexity in proptest::option::of(1u32..50),
            min_coverage in proptest::option::of(1u32..100)
        ) {
            let profile = create_quality_profile(QddQualityProfile::Standard, max_complexity, min_coverage);

            if let Some(c) = max_complexity {
                prop_assert_eq!(profile.thresholds.max_complexity, c);
            } else {
                prop_assert_eq!(profile.thresholds.max_complexity, 10); // standard default
            }

            if let Some(cov) = min_coverage {
                prop_assert_eq!(profile.thresholds.min_coverage, cov);
            } else {
                prop_assert_eq!(profile.thresholds.min_coverage, 80); // standard default
            }
        }
    }

    // ===============================================================
    // Edge case tests
    // ===============================================================

    #[test]
    fn test_convert_parameters_unicode_names() {
        let inputs = vec![
            ("String".to_string(), "nombre".to_string()),
            ("i32".to_string(), "contador".to_string()),
        ];
        let result = convert_parameters(inputs);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "nombre");
        assert_eq!(result[1].name, "contador");
    }

    #[test]
    fn test_convert_parameters_complex_types() {
        let inputs = vec![
            ("Vec<String>".to_string(), "items".to_string()),
            ("HashMap<String, i32>".to_string(), "map".to_string()),
            ("Option<Result<T, E>>".to_string(), "complex".to_string()),
        ];
        let result = convert_parameters(inputs);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].param_type, "Vec<String>");
        assert_eq!(result[1].param_type, "HashMap<String, i32>");
        assert_eq!(result[2].param_type, "Option<Result<T, E>>");
    }

    #[test]
    fn test_build_create_spec_empty_purpose() {
        let spec = build_create_spec(
            CodeType::Function,
            "empty_purpose_func".to_string(),
            String::new(),
            vec![],
            "()".to_string(),
        );
        assert!(spec.purpose.is_empty());
    }

    #[test]
    fn test_build_create_spec_many_inputs() {
        let inputs: Vec<Parameter> = (0..20)
            .map(|i| Parameter {
                name: format!("param{i}"),
                param_type: "i32".to_string(),
                description: None,
            })
            .collect();

        let spec = build_create_spec(
            CodeType::Function,
            "many_params".to_string(),
            "Function with many parameters".to_string(),
            inputs.clone(),
            "()".to_string(),
        );

        assert_eq!(spec.inputs.len(), 20);
    }

    #[test]
    fn test_create_refactor_spec_complex_path() {
        let file = PathBuf::from("/very/deep/nested/path/to/some/file.rs");
        let profile = QualityProfile::standard();

        let spec = create_refactor_spec(&file, None, &profile);
        assert_eq!(
            spec.file_path.to_string_lossy(),
            "/very/deep/nested/path/to/some/file.rs"
        );
    }

    #[tokio::test]
    async fn test_handle_qdd_validate_all_profiles() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();

        // Test all profile types
        for profile in [
            QddQualityProfile::Extreme,
            QddQualityProfile::Standard,
            QddQualityProfile::Relaxed,
        ] {
            let result =
                handle_qdd_validate(path.clone(), profile, QddOutputFormat::Summary, None, false)
                    .await;
            assert!(result.is_ok(), "Failed for profile {:?}", profile);
        }
    }

    // ===============================================================
    // Tests for build_validation_json (JSON purity)
    // ===============================================================

    fn outcome_of(checks: Vec<(&'static str, CheckOutcome)>) -> ValidationOutcome {
        ValidationOutcome { checks }
    }

    #[test]
    fn test_build_validation_json_passed() {
        let path = PathBuf::from("src/utils/scratch.rs");
        let outcome = outcome_of(vec![("complexity", CheckOutcome::Passed("ok".into()))]);
        let json = build_validation_json(&outcome, QddQualityProfile::Standard, &path);

        assert_eq!(json["status"], "passed");
        assert_eq!(json["profile"], "standard");
        assert_eq!(json["path"], "src/utils/scratch.rs");
        assert!(json["validation_time"].is_string());
        assert!(json["violations"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_build_validation_json_failed() {
        let path = PathBuf::from("/some/dir");
        let outcome = outcome_of(vec![(
            "complexity",
            CheckOutcome::Failed("a::b has cyclomatic complexity 31".into()),
        )]);
        let json = build_validation_json(&outcome, QddQualityProfile::Extreme, &path);

        assert_eq!(json["status"], "failed");
        assert_eq!(json["profile"], "extreme");
        assert_eq!(json["path"], "/some/dir");
        assert_eq!(json["violations"][0]["check"], "complexity");
    }

    #[test]
    fn test_build_validation_json_pretty_output_is_pure_json() {
        // Stdout in JSON mode is exactly this pretty-printed payload: it must
        // start with '{' (no ANSI header) and round-trip through a JSON parser.
        let path = PathBuf::from(".");
        let outcome = outcome_of(vec![("complexity", CheckOutcome::Passed("ok".into()))]);
        let json = build_validation_json(&outcome, QddQualityProfile::Relaxed, &path);
        let pretty = serde_json::to_string_pretty(&json).unwrap();

        assert!(pretty.starts_with('{'));
        assert!(!pretty.contains('\x1b'));
        let reparsed: serde_json::Value = serde_json::from_str(&pretty).unwrap();
        assert_eq!(reparsed["profile"], "relaxed");
    }

    #[tokio::test]
    async fn test_handle_qdd_validate_all_formats() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();

        // Test all output formats
        for format in [
            QddOutputFormat::Summary,
            QddOutputFormat::Detailed,
            QddOutputFormat::Json,
            QddOutputFormat::Markdown,
        ] {
            let result = handle_qdd_validate(
                path.clone(),
                QddQualityProfile::Standard,
                format.clone(),
                None,
                false,
            )
            .await;
            assert!(result.is_ok(), "Failed for format {:?}", format);
        }
    }
}
