//\! Tests for simple service
//\! Extracted to separate file for file health compliance (CB-040)

use super::*;

mod coverage_tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    // Helper function to create a valid temp directory for tests
    fn create_temp_dir() -> TempDir {
        tempfile::tempdir().expect("Failed to create temp directory")
    }

    // Helper function to create a valid temp file for tests
    fn create_temp_file(dir: &TempDir, name: &str) -> PathBuf {
        let file_path = dir.path().join(name);
        std::fs::write(&file_path, "fn main() {}").expect("Failed to write temp file");
        file_path
    }

    // === SimpleContractService::new tests ===

    #[test]
    fn test_service_new_succeeds() {
        let service = SimpleContractService::new();
        assert!(service.is_ok());
    }

    // === analyze_complexity tests ===

    #[tokio::test]
    async fn test_analyze_complexity_success() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();

        let contract = AnalyzeComplexityContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(10),
                include_tests: false,
                timeout: 60,
            },
            max_cyclomatic: Some(20),
            max_cognitive: Some(15),
            max_halstead: Some(10.0),
        };

        let result = service.analyze_complexity(contract).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        assert!(value.get("results").is_some());
        assert!(value.get("summary").is_some());
        assert!(value.get("metadata").is_some());

        // Verify metadata structure
        let metadata = value.get("metadata").unwrap();
        assert!(metadata.get("path").is_some());
        assert!(metadata.get("format").is_some());
        assert!(metadata.get("include_tests").is_some());
        assert!(metadata.get("timeout").is_some());
        assert!(metadata.get("timestamp").is_some());
    }

    #[tokio::test]
    async fn test_analyze_complexity_with_invalid_path() {
        let service = SimpleContractService::new().unwrap();

        let contract = AnalyzeComplexityContract {
            base: BaseAnalysisContract {
                path: PathBuf::from("/nonexistent/path/that/does/not/exist"),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(10),
                include_tests: false,
                timeout: 60,
            },
            max_cyclomatic: None,
            max_cognitive: None,
            max_halstead: None,
        };

        let result = service.analyze_complexity(contract).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_analyze_complexity_with_invalid_halstead() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();

        let contract = AnalyzeComplexityContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(10),
                include_tests: false,
                timeout: 60,
            },
            max_cyclomatic: None,
            max_cognitive: None,
            max_halstead: Some(-1.0), // Invalid - must be positive
        };

        let result = service.analyze_complexity(contract).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_analyze_complexity_with_zero_timeout() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();

        let contract = AnalyzeComplexityContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(10),
                include_tests: false,
                timeout: 0, // Invalid timeout
            },
            max_cyclomatic: None,
            max_cognitive: None,
            max_halstead: None,
        };

        let result = service.analyze_complexity(contract).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_analyze_complexity_with_too_many_files() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();

        let contract = AnalyzeComplexityContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(2000), // Too many files
                include_tests: false,
                timeout: 60,
            },
            max_cyclomatic: None,
            max_cognitive: None,
            max_halstead: None,
        };

        let result = service.analyze_complexity(contract).await;
        assert!(result.is_err());
    }

    // === analyze_satd tests ===

    #[tokio::test]
    async fn test_analyze_satd_success() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();

        let contract = AnalyzeSatdContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(10),
                include_tests: false,
                timeout: 60,
            },
            severity: Some(SatdSeverity::Medium),
            critical_only: false,
            strict: true,
            fail_on_violation: false,
        };

        let result = service.analyze_satd(contract).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        assert!(value.get("results").is_some());
        assert!(value.get("summary").is_some());

        // Check summary includes strict mode info
        let summary = value.get("summary").unwrap().as_str().unwrap();
        assert!(summary.contains("strict mode: true"));
    }

    #[tokio::test]
    async fn test_analyze_satd_not_strict() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();

        let contract = AnalyzeSatdContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(10),
                include_tests: false,
                timeout: 60,
            },
            severity: None,
            critical_only: true,
            strict: false,
            fail_on_violation: true,
        };

        let result = service.analyze_satd(contract).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        let summary = value.get("summary").unwrap().as_str().unwrap();
        assert!(summary.contains("strict mode: false"));
    }

    #[tokio::test]
    async fn test_analyze_satd_with_invalid_path() {
        let service = SimpleContractService::new().unwrap();

        let contract = AnalyzeSatdContract {
            base: BaseAnalysisContract {
                path: PathBuf::from("/nonexistent/path"),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(10),
                include_tests: false,
                timeout: 60,
            },
            severity: None,
            critical_only: false,
            strict: false,
            fail_on_violation: false,
        };

        let result = service.analyze_satd(contract).await;
        assert!(result.is_err());
    }

    // === analyze_dead_code tests ===

    #[tokio::test]
    async fn test_analyze_dead_code_success() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();

        let contract = AnalyzeDeadCodeContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(10),
                include_tests: false,
                timeout: 60,
            },
            include_unreachable: true,
            min_dead_lines: 5,
            max_percentage: 50.0,
            fail_on_violation: false,
        };

        let result = service.analyze_dead_code(contract).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        let results = value.get("results").unwrap().as_array().unwrap();
        assert_eq!(results.len(), 1);

        // Check unreachable blocks are included
        let first_result = &results[0];
        let unreachable_blocks = first_result.get("unreachable_blocks").unwrap().as_u64().unwrap();
        assert_eq!(unreachable_blocks, 2); // include_unreachable is true
    }

    #[tokio::test]
    async fn test_analyze_dead_code_without_unreachable() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();

        let contract = AnalyzeDeadCodeContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(10),
                include_tests: false,
                timeout: 60,
            },
            include_unreachable: false,
            min_dead_lines: 0,
            max_percentage: 100.0,
            fail_on_violation: false,
        };

        let result = service.analyze_dead_code(contract).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        let results = value.get("results").unwrap().as_array().unwrap();
        let first_result = &results[0];
        let unreachable_blocks = first_result.get("unreachable_blocks").unwrap().as_u64().unwrap();
        assert_eq!(unreachable_blocks, 0); // include_unreachable is false
    }

    #[tokio::test]
    async fn test_analyze_dead_code_invalid_percentage() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();

        let contract = AnalyzeDeadCodeContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(10),
                include_tests: false,
                timeout: 60,
            },
            include_unreachable: false,
            min_dead_lines: 0,
            max_percentage: 150.0, // Invalid - must be 0-100
            fail_on_violation: false,
        };

        let result = service.analyze_dead_code(contract).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_analyze_dead_code_negative_percentage() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();

        let contract = AnalyzeDeadCodeContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(10),
                include_tests: false,
                timeout: 60,
            },
            include_unreachable: false,
            min_dead_lines: 0,
            max_percentage: -10.0, // Invalid - must be >= 0
            fail_on_violation: false,
        };

        let result = service.analyze_dead_code(contract).await;
        assert!(result.is_err());
    }

    // === analyze_tdg tests ===

    #[tokio::test]
    async fn test_analyze_tdg_success_with_components() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();

        let contract = AnalyzeTdgContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(10),
                include_tests: false,
                timeout: 60,
            },
            threshold: 1.0,
            include_components: true,
            critical_only: false,
        };

        let result = service.analyze_tdg(contract).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        let results = value.get("results").unwrap().as_array().unwrap();
        let first_result = &results[0];

        // Components should be present
        assert!(first_result.get("components").is_some());
        let components = first_result.get("components").unwrap();
        assert!(components.get("complexity").is_some());
        assert!(components.get("churn").is_some());
        assert!(components.get("coverage").is_some());
    }

    #[tokio::test]
    async fn test_analyze_tdg_success_without_components() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();

        let contract = AnalyzeTdgContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(10),
                include_tests: false,
                timeout: 60,
            },
            threshold: 2.0,
            include_components: false,
            critical_only: true,
        };

        let result = service.analyze_tdg(contract).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        let results = value.get("results").unwrap().as_array().unwrap();
        let first_result = &results[0];

        // Components should be null
        assert!(first_result.get("components").unwrap().is_null());
    }

    #[tokio::test]
    async fn test_analyze_tdg_invalid_threshold() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();

        let contract = AnalyzeTdgContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(10),
                include_tests: false,
                timeout: 60,
            },
            threshold: -1.0, // Invalid - must be non-negative
            include_components: false,
            critical_only: false,
        };

        let result = service.analyze_tdg(contract).await;
        assert!(result.is_err());
    }

    // === analyze_lint_hotspot tests ===

    #[tokio::test]
    async fn test_analyze_lint_hotspot_success() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();

        let contract = AnalyzeLintHotspotContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(10),
                include_tests: false,
                timeout: 60,
            },
            file: None,
            max_density: 5.0,
            min_confidence: 0.8,
            enforce: false,
            dry_run: false,
        };

        let result = service.analyze_lint_hotspot(contract).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        let results = value.get("results").unwrap().as_array().unwrap();
        let first_result = &results[0];

        // Fixable should be true when dry_run is false
        let fixable = first_result.get("fixable").unwrap().as_bool().unwrap();
        assert!(fixable);
    }

    #[tokio::test]
    async fn test_analyze_lint_hotspot_dry_run() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();

        let contract = AnalyzeLintHotspotContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(10),
                include_tests: false,
                timeout: 60,
            },
            file: None,
            max_density: 5.0,
            min_confidence: 0.8,
            enforce: true,
            dry_run: true,
        };

        let result = service.analyze_lint_hotspot(contract).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        let results = value.get("results").unwrap().as_array().unwrap();
        let first_result = &results[0];

        // Fixable should be false when dry_run is true
        let fixable = first_result.get("fixable").unwrap().as_bool().unwrap();
        assert!(!fixable);
    }

    #[tokio::test]
    async fn test_analyze_lint_hotspot_invalid_density() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();

        let contract = AnalyzeLintHotspotContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(10),
                include_tests: false,
                timeout: 60,
            },
            file: None,
            max_density: -1.0, // Invalid
            min_confidence: 0.8,
            enforce: false,
            dry_run: false,
        };

        let result = service.analyze_lint_hotspot(contract).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_analyze_lint_hotspot_invalid_confidence() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();

        let contract = AnalyzeLintHotspotContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(10),
                include_tests: false,
                timeout: 60,
            },
            file: None,
            max_density: 5.0,
            min_confidence: 1.5, // Invalid - must be 0-1
            enforce: false,
            dry_run: false,
        };

        let result = service.analyze_lint_hotspot(contract).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_analyze_lint_hotspot_negative_confidence() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();

        let contract = AnalyzeLintHotspotContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(10),
                include_tests: false,
                timeout: 60,
            },
            file: None,
            max_density: 5.0,
            min_confidence: -0.5, // Invalid - must be >= 0
            enforce: false,
            dry_run: false,
        };

        let result = service.analyze_lint_hotspot(contract).await;
        assert!(result.is_err());
    }

    // === analyze_entropy tests ===

    #[tokio::test]
    async fn test_analyze_entropy_success() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();
        // Create a simple Rust file for entropy analysis
        create_temp_file(&temp_dir, "test.rs");

        let contract = AnalyzeEntropyContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(10),
                include_tests: false,
                timeout: 60,
            },
            min_severity: Some("medium".to_string()),
            top_violations: Some(10),
            file: None,
        };

        let result = service.analyze_entropy(contract).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        assert!(value.get("total_files_analyzed").is_some());
        assert!(value.get("total_violations").is_some());
        assert!(value.get("potential_loc_reduction").is_some());
        assert!(value.get("reduction_percentage").is_some());
        assert!(value.get("summary").is_some());
    }

    #[tokio::test]
    async fn test_analyze_entropy_with_low_severity() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();
        create_temp_file(&temp_dir, "test.rs");

        let contract = AnalyzeEntropyContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(10),
                include_tests: false,
                timeout: 60,
            },
            min_severity: Some("low".to_string()),
            top_violations: None,
            file: None,
        };

        let result = service.analyze_entropy(contract).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_entropy_with_high_severity() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();
        create_temp_file(&temp_dir, "test.rs");

        let contract = AnalyzeEntropyContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(10),
                include_tests: false,
                timeout: 60,
            },
            min_severity: Some("high".to_string()),
            top_violations: Some(5),
            file: None,
        };

        let result = service.analyze_entropy(contract).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_entropy_with_unknown_severity() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();
        create_temp_file(&temp_dir, "test.rs");

        // Unknown severity should default to medium
        let contract = AnalyzeEntropyContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(10),
                include_tests: false,
                timeout: 60,
            },
            min_severity: Some("unknown_severity".to_string()),
            top_violations: None,
            file: None,
        };

        let result = service.analyze_entropy(contract).await;
        // Should fail validation because unknown_severity is invalid
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_analyze_entropy_with_specific_file() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();
        let file_path = create_temp_file(&temp_dir, "specific_test.rs");

        let contract = AnalyzeEntropyContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(10),
                include_tests: false,
                timeout: 60,
            },
            min_severity: None,
            top_violations: Some(10),
            file: Some(file_path),
        };

        let result = service.analyze_entropy(contract).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_entropy_excluding_tests() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();
        create_temp_file(&temp_dir, "test.rs");

        let contract = AnalyzeEntropyContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(10),
                include_tests: false, // Should exclude test files
                timeout: 60,
            },
            min_severity: None,
            top_violations: None,
            file: None,
        };

        let result = service.analyze_entropy(contract).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_entropy_including_tests() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();
        create_temp_file(&temp_dir, "test.rs");

        let contract = AnalyzeEntropyContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(10),
                include_tests: true, // Should include test files
                timeout: 60,
            },
            min_severity: None,
            top_violations: None,
            file: None,
        };

        let result = service.analyze_entropy(contract).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_entropy_with_too_many_violations() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();

        let contract = AnalyzeEntropyContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(10),
                include_tests: false,
                timeout: 60,
            },
            min_severity: None,
            top_violations: Some(2000), // Too many - exceeds 1000 limit
            file: None,
        };

        let result = service.analyze_entropy(contract).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_analyze_entropy_with_invalid_file_path() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();

        let contract = AnalyzeEntropyContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(10),
                include_tests: false,
                timeout: 60,
            },
            min_severity: None,
            top_violations: None,
            file: Some(PathBuf::from("/nonexistent/file.rs")),
        };

        let result = service.analyze_entropy(contract).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_analyze_entropy_with_zero_top_violations() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();
        create_temp_file(&temp_dir, "test.rs");

        // Zero means show all violations, should work
        let contract = AnalyzeEntropyContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(10),
                include_tests: false,
                timeout: 60,
            },
            min_severity: None,
            top_violations: Some(0), // Zero should be allowed
            file: None,
        };

        let result = service.analyze_entropy(contract).await;
        assert!(result.is_ok());
    }

    // === quality_gate tests ===

    #[tokio::test]
    async fn test_quality_gate_standard_profile() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();
        create_temp_file(&temp_dir, "test.rs");

        let contract = QualityGateContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(10),
                include_tests: false,
                timeout: 60,
            },
            profile: QualityProfile::Standard,
            file: None,
            fail_on_violation: false,
            verbose: false,
        };

        let result = service.quality_gate(contract).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        assert!(value.get("passed").is_some());
        assert!(value.get("violations").is_some());
        assert!(value.get("profile").is_some());
    }

    #[tokio::test]
    async fn test_quality_gate_strict_profile() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();
        create_temp_file(&temp_dir, "test.rs");

        let contract = QualityGateContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(10),
                include_tests: false,
                timeout: 60,
            },
            profile: QualityProfile::Strict,
            file: None,
            fail_on_violation: false,
            verbose: true,
        };

        let result = service.quality_gate(contract).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        // Strict profile should run entropy analysis
        let summary = value.get("summary").unwrap().as_str().unwrap();
        assert!(summary.contains("Strict"));
    }

    #[tokio::test]
    async fn test_quality_gate_extreme_profile() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();
        create_temp_file(&temp_dir, "test.rs");

        let contract = QualityGateContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(10),
                include_tests: false,
                timeout: 60,
            },
            profile: QualityProfile::Extreme,
            file: None,
            fail_on_violation: false,
            verbose: false,
        };

        let result = service.quality_gate(contract).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        let summary = value.get("summary").unwrap().as_str().unwrap();
        assert!(summary.contains("Extreme"));
    }

    #[tokio::test]
    async fn test_quality_gate_toyota_profile() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();
        create_temp_file(&temp_dir, "test.rs");

        let contract = QualityGateContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(10),
                include_tests: false,
                timeout: 60,
            },
            profile: QualityProfile::Toyota,
            file: None,
            fail_on_violation: false,
            verbose: false,
        };

        let result = service.quality_gate(contract).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_quality_gate_with_specific_file() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();
        let file_path = create_temp_file(&temp_dir, "specific.rs");

        let contract = QualityGateContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(10),
                include_tests: false,
                timeout: 60,
            },
            profile: QualityProfile::Standard,
            file: Some(file_path),
            fail_on_violation: false,
            verbose: false,
        };

        let result = service.quality_gate(contract).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_quality_gate_with_invalid_file() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();

        let contract = QualityGateContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(10),
                include_tests: false,
                timeout: 60,
            },
            profile: QualityProfile::Standard,
            file: Some(PathBuf::from("/nonexistent/file.rs")),
            fail_on_violation: false,
            verbose: false,
        };

        let result = service.quality_gate(contract).await;
        assert!(result.is_err());
    }

    // === refactor_auto tests ===

    #[tokio::test]
    async fn test_refactor_auto_success() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();
        let file_path = create_temp_file(&temp_dir, "refactor_target.rs");

        let contract = RefactorAutoContract {
            file: file_path,
            format: OutputFormat::Json,
            output: None,
            target_complexity: 10,
            dry_run: false,
            timeout: 60,
        };

        let result = service.refactor_auto(contract).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        assert!(value.get("plan").is_some());
        assert!(value.get("dry_run").is_some());
        assert!(value.get("summary").is_some());

        let plan = value.get("plan").unwrap();
        assert!(plan.get("file").is_some());
        assert!(plan.get("current_complexity").is_some());
        assert!(plan.get("target_complexity").is_some());
        assert!(plan.get("operations").is_some());
        assert!(plan.get("estimated_reduction").is_some());

        // Applied should be true when dry_run is false
        let applied = plan.get("applied").unwrap().as_bool().unwrap();
        assert!(applied);
    }

    #[tokio::test]
    async fn test_refactor_auto_dry_run() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();
        let file_path = create_temp_file(&temp_dir, "refactor_target.rs");

        let contract = RefactorAutoContract {
            file: file_path,
            format: OutputFormat::Json,
            output: None,
            target_complexity: 10,
            dry_run: true,
            timeout: 60,
        };

        let result = service.refactor_auto(contract).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        let dry_run = value.get("dry_run").unwrap().as_bool().unwrap();
        assert!(dry_run);

        let plan = value.get("plan").unwrap();
        let applied = plan.get("applied").unwrap().as_bool().unwrap();
        assert!(!applied);
    }

    #[tokio::test]
    async fn test_refactor_auto_with_invalid_file() {
        let service = SimpleContractService::new().unwrap();

        let contract = RefactorAutoContract {
            file: PathBuf::from("/nonexistent/file.rs"),
            format: OutputFormat::Json,
            output: None,
            target_complexity: 10,
            dry_run: false,
            timeout: 60,
        };

        let result = service.refactor_auto(contract).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_refactor_auto_with_zero_complexity() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();
        let file_path = create_temp_file(&temp_dir, "refactor_target.rs");

        let contract = RefactorAutoContract {
            file: file_path,
            format: OutputFormat::Json,
            output: None,
            target_complexity: 0, // Invalid - must be > 0
            dry_run: false,
            timeout: 60,
        };

        let result = service.refactor_auto(contract).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_refactor_auto_with_zero_timeout() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();
        let file_path = create_temp_file(&temp_dir, "refactor_target.rs");

        let contract = RefactorAutoContract {
            file: file_path,
            format: OutputFormat::Json,
            output: None,
            target_complexity: 10,
            dry_run: false,
            timeout: 0, // Invalid timeout
        };

        let result = service.refactor_auto(contract).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_refactor_auto_directory_not_file() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();

        // Try to use a directory instead of a file
        let contract = RefactorAutoContract {
            file: temp_dir.path().to_path_buf(),
            format: OutputFormat::Json,
            output: None,
            target_complexity: 10,
            dry_run: false,
            timeout: 60,
        };

        let result = service.refactor_auto(contract).await;
        assert!(result.is_err());
    }

    // === create_metadata tests ===

    #[test]
    fn test_create_metadata() {
        let service = SimpleContractService::new().unwrap();

        let base = BaseAnalysisContract {
            path: PathBuf::from("/test/path"),
            format: OutputFormat::Json,
            output: Some(PathBuf::from("/output")),
            top_files: Some(20),
            include_tests: true,
            timeout: 120,
        };

        let metadata = service.create_metadata(&base);

        assert_eq!(metadata.path, "/test/path");
        assert_eq!(metadata.format, OutputFormat::Json);
        assert!(metadata.include_tests);
        assert_eq!(metadata.timeout, 120);
        assert!(metadata.timestamp > 0);
    }

    #[test]
    fn test_create_metadata_various_formats() {
        let service = SimpleContractService::new().unwrap();

        // Test with Table format
        let base_table = BaseAnalysisContract {
            path: PathBuf::from("/test"),
            format: OutputFormat::Table,
            output: None,
            top_files: None,
            include_tests: false,
            timeout: 60,
        };
        let metadata = service.create_metadata(&base_table);
        assert_eq!(metadata.format, OutputFormat::Table);

        // Test with Markdown format
        let base_md = BaseAnalysisContract {
            path: PathBuf::from("/test"),
            format: OutputFormat::Markdown,
            output: None,
            top_files: None,
            include_tests: false,
            timeout: 60,
        };
        let metadata = service.create_metadata(&base_md);
        assert_eq!(metadata.format, OutputFormat::Markdown);

        // Test with Yaml format
        let base_yaml = BaseAnalysisContract {
            path: PathBuf::from("/test"),
            format: OutputFormat::Yaml,
            output: None,
            top_files: None,
            include_tests: false,
            timeout: 60,
        };
        let metadata = service.create_metadata(&base_yaml);
        assert_eq!(metadata.format, OutputFormat::Yaml);

        // Test with Csv format
        let base_csv = BaseAnalysisContract {
            path: PathBuf::from("/test"),
            format: OutputFormat::Csv,
            output: None,
            top_files: None,
            include_tests: false,
            timeout: 60,
        };
        let metadata = service.create_metadata(&base_csv);
        assert_eq!(metadata.format, OutputFormat::Csv);

        // Test with Summary format
        let base_summary = BaseAnalysisContract {
            path: PathBuf::from("/test"),
            format: OutputFormat::Summary,
            output: None,
            top_files: None,
            include_tests: false,
            timeout: 60,
        };
        let metadata = service.create_metadata(&base_summary);
        assert_eq!(metadata.format, OutputFormat::Summary);
    }

    // === Response types serialization tests ===

    #[test]
    fn test_complexity_result_serialization() {
        let result = ComplexityResult {
            file: "test.rs".to_string(),
            cyclomatic: 10,
            cognitive: 5,
            halstead: 3.5,
            functions: vec![FunctionResult {
                name: "test_fn".to_string(),
                cyclomatic: 5,
                cognitive: 3,
                line_start: 1,
                line_end: 10,
            }],
        };

        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["file"], "test.rs");
        assert_eq!(json["cyclomatic"], 10);
        assert_eq!(json["cognitive"], 5);
        assert_eq!(json["halstead"], 3.5);
        assert_eq!(json["functions"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_satd_result_serialization() {
        let result = SatdResult {
            file: "test.rs".to_string(),
            comment: "// TODO: fix this".to_string(),
            line: 42,
            severity: SatdSeverity::High,
            debt_type: "TODO".to_string(),
        };

        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["file"], "test.rs");
        assert_eq!(json["comment"], "// TODO: fix this");
        assert_eq!(json["line"], 42);
        assert_eq!(json["debt_type"], "TODO");
    }

    #[test]
    fn test_dead_code_result_serialization() {
        let result = DeadCodeResult {
            file: "test.rs".to_string(),
            dead_lines: 50,
            total_lines: 200,
            percentage: 25.0,
            unreachable_blocks: 5,
        };

        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["file"], "test.rs");
        assert_eq!(json["dead_lines"], 50);
        assert_eq!(json["total_lines"], 200);
        assert_eq!(json["percentage"], 25.0);
        assert_eq!(json["unreachable_blocks"], 5);
    }

    #[test]
    fn test_tdg_result_serialization_with_components() {
        let result = TdgResult {
            file: "test.rs".to_string(),
            score: 2.5,
            components: Some(TdgComponents {
                complexity: 1.0,
                churn: 0.8,
                coverage: 0.7,
            }),
            grade: "A".to_string(),
        };

        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["file"], "test.rs");
        assert_eq!(json["score"], 2.5);
        assert!(json["components"].is_object());
        assert_eq!(json["grade"], "A");
    }

    #[test]
    fn test_tdg_result_serialization_without_components() {
        let result = TdgResult {
            file: "test.rs".to_string(),
            score: 1.5,
            components: None,
            grade: "B".to_string(),
        };

        let json = serde_json::to_value(&result).unwrap();
        assert!(json["components"].is_null());
    }

    #[test]
    fn test_lint_hotspot_result_serialization() {
        let result = LintHotspotResult {
            file: "test.rs".to_string(),
            density: 4.5,
            violations: vec!["warning1".to_string(), "warning2".to_string()],
            total_lines: 100,
            fixable: true,
        };

        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["file"], "test.rs");
        assert_eq!(json["density"], 4.5);
        assert_eq!(json["violations"].as_array().unwrap().len(), 2);
        assert_eq!(json["total_lines"], 100);
        assert!(json["fixable"].as_bool().unwrap());
    }

    #[test]
    fn test_refactor_plan_serialization() {
        let plan = RefactorPlan {
            file: "test.rs".to_string(),
            current_complexity: 20,
            target_complexity: 10,
            operations: vec![RefactorOperation {
                operation_type: "extract_method".to_string(),
                description: "Extract large function".to_string(),
                line_start: 10,
                line_end: 50,
                confidence: 0.95,
            }],
            estimated_reduction: 10,
            applied: true,
        };

        let json = serde_json::to_value(&plan).unwrap();
        assert_eq!(json["file"], "test.rs");
        assert_eq!(json["current_complexity"], 20);
        assert_eq!(json["target_complexity"], 10);
        assert_eq!(json["operations"].as_array().unwrap().len(), 1);
        assert_eq!(json["estimated_reduction"], 10);
        assert!(json["applied"].as_bool().unwrap());
    }

    #[test]
    fn test_quality_violation_serialization() {
        let violation = QualityViolation {
            rule: "complexity".to_string(),
            severity: ViolationSeverity::Error,
            message: "Function too complex".to_string(),
            file: "test.rs".to_string(),
            line: 42,
        };

        let json = serde_json::to_value(&violation).unwrap();
        assert_eq!(json["rule"], "complexity");
        assert_eq!(json["severity"], "Error");
        assert_eq!(json["message"], "Function too complex");
        assert_eq!(json["file"], "test.rs");
        assert_eq!(json["line"], 42);
    }

    #[test]
    fn test_violation_severity_serialization() {
        let error = ViolationSeverity::Error;
        let warning = ViolationSeverity::Warning;
        let info = ViolationSeverity::Info;

        assert_eq!(serde_json::to_value(&error).unwrap(), "Error");
        assert_eq!(serde_json::to_value(&warning).unwrap(), "Warning");
        assert_eq!(serde_json::to_value(&info).unwrap(), "Info");
    }

    // === Output format coverage tests ===

    #[tokio::test]
    async fn test_analyze_complexity_with_all_formats() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();

        let formats = vec![
            OutputFormat::Table,
            OutputFormat::Json,
            OutputFormat::Yaml,
            OutputFormat::Markdown,
            OutputFormat::Csv,
            OutputFormat::Summary,
        ];

        for format in formats {
            let contract = AnalyzeComplexityContract {
                base: BaseAnalysisContract {
                    path: temp_dir.path().to_path_buf(),
                    format,
                    output: None,
                    top_files: Some(10),
                    include_tests: false,
                    timeout: 60,
                },
                max_cyclomatic: None,
                max_cognitive: None,
                max_halstead: None,
            };

            let result = service.analyze_complexity(contract).await;
            assert!(result.is_ok(), "Failed for format: {format:?}");
        }
    }

    // === Edge case tests ===

    #[tokio::test]
    async fn test_analyze_complexity_with_output_file() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();

        let contract = AnalyzeComplexityContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: Some(PathBuf::from("/tmp/output.json")),
                top_files: Some(10),
                include_tests: true,
                timeout: 60,
            },
            max_cyclomatic: Some(30),
            max_cognitive: Some(20),
            max_halstead: Some(15.0),
        };

        let result = service.analyze_complexity(contract).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_complexity_include_tests() {
        let service = SimpleContractService::new().unwrap();
        let temp_dir = create_temp_dir();

        let contract = AnalyzeComplexityContract {
            base: BaseAnalysisContract {
                path: temp_dir.path().to_path_buf(),
                format: OutputFormat::Json,
                output: None,
                top_files: None, // No limit on top files
                include_tests: true,
                timeout: 60,
            },
            max_cyclomatic: None,
            max_cognitive: None,
            max_halstead: None,
        };

        let result = service.analyze_complexity(contract).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        let metadata = value.get("metadata").unwrap();
        assert!(metadata.get("include_tests").unwrap().as_bool().unwrap());
    }

    // === Property-based tests ===

    #[test]
    fn test_service_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SimpleContractService>();
    }

    #[test]
    fn test_analysis_metadata_fields() {
        let metadata = AnalysisMetadata {
            path: "/test/path".to_string(),
            format: OutputFormat::Json,
            include_tests: true,
            timeout: 120,
            timestamp: 1234567890,
        };

        let json = serde_json::to_value(&metadata).unwrap();
        assert_eq!(json["path"], "/test/path");
        assert_eq!(json["format"], "json");
        assert!(json["include_tests"].as_bool().unwrap());
        assert_eq!(json["timeout"], 120);
        assert_eq!(json["timestamp"], 1234567890);
    }

    #[test]
    fn test_analysis_response_serialization() {
        let response: AnalysisResponse<ComplexityResult> = AnalysisResponse {
            results: vec![ComplexityResult {
                file: "test.rs".to_string(),
                cyclomatic: 5,
                cognitive: 3,
                halstead: 2.0,
                functions: vec![],
            }],
            summary: "Test summary".to_string(),
            metadata: AnalysisMetadata {
                path: "/test".to_string(),
                format: OutputFormat::Json,
                include_tests: false,
                timeout: 60,
                timestamp: 1234567890,
            },
        };

        let json = serde_json::to_value(&response).unwrap();
        assert!(json.get("results").is_some());
        assert!(json.get("summary").is_some());
        assert!(json.get("metadata").is_some());
    }

    #[test]
    fn test_refactor_response_serialization() {
        let response = RefactorResponse {
            plan: RefactorPlan {
                file: "test.rs".to_string(),
                current_complexity: 15,
                target_complexity: 10,
                operations: vec![],
                estimated_reduction: 5,
                applied: false,
            },
            dry_run: true,
            summary: "Refactor plan summary".to_string(),
        };

        let json = serde_json::to_value(&response).unwrap();
        assert!(json.get("plan").is_some());
        assert!(json["dry_run"].as_bool().unwrap());
        assert_eq!(json["summary"], "Refactor plan summary");
    }

    #[test]
    fn test_quality_gate_response_serialization() {
        let response = QualityGateResponse {
            passed: true,
            violations: vec![],
            profile: QualityProfile::Standard,
            summary: "Quality gate passed".to_string(),
            metadata: AnalysisMetadata {
                path: "/test".to_string(),
                format: OutputFormat::Json,
                include_tests: false,
                timeout: 60,
                timestamp: 1234567890,
            },
        };

        let json = serde_json::to_value(&response).unwrap();
        assert!(json["passed"].as_bool().unwrap());
        assert!(json["violations"].as_array().unwrap().is_empty());
        assert_eq!(json["profile"], "standard");
    }

    #[test]
    fn test_function_result_serialization() {
        let result = FunctionResult {
            name: "my_function".to_string(),
            cyclomatic: 7,
            cognitive: 4,
            line_start: 100,
            line_end: 150,
        };

        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["name"], "my_function");
        assert_eq!(json["cyclomatic"], 7);
        assert_eq!(json["cognitive"], 4);
        assert_eq!(json["line_start"], 100);
        assert_eq!(json["line_end"], 150);
    }

    #[test]
    fn test_refactor_operation_serialization() {
        let operation = RefactorOperation {
            operation_type: "inline_variable".to_string(),
            description: "Inline unused variable".to_string(),
            line_start: 25,
            line_end: 25,
            confidence: 0.85,
        };

        let json = serde_json::to_value(&operation).unwrap();
        assert_eq!(json["operation_type"], "inline_variable");
        assert_eq!(json["description"], "Inline unused variable");
        assert_eq!(json["line_start"], 25);
        assert_eq!(json["line_end"], 25);
        assert_eq!(json["confidence"], 0.85);
    }

    #[test]
    fn test_tdg_components_serialization() {
        let components = TdgComponents {
            complexity: 1.5,
            churn: 0.8,
            coverage: 0.9,
        };

        let json = serde_json::to_value(&components).unwrap();
        assert_eq!(json["complexity"], 1.5);
        assert_eq!(json["churn"], 0.8);
        assert_eq!(json["coverage"], 0.9);
    }
}
