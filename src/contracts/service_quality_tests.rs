// Coverage tests for quality gate, refactor, output format, configuration
// variations, Arc behavior, metadata generation, and concurrency.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod quality_tests {
    use super::*;
    use crate::contracts::{BaseAnalysisContract, OutputFormat, QualityProfile, SatdSeverity};
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Helper to create a temp directory with a valid path for testing
    fn create_test_dir() -> TempDir {
        tempfile::tempdir().expect("Failed to create temp directory")
    }

    /// Helper to create a temp file for refactor tests
    fn create_test_file(dir: &TempDir) -> PathBuf {
        let file_path = dir.path().join("test_file.rs");
        std::fs::write(
            &file_path,
            "fn example() { let x = 1; if x > 0 { println!(\"positive\"); } }",
        )
        .unwrap();
        file_path
    }

    /// Helper to create a base contract with valid path
    fn create_base_contract(path: PathBuf) -> BaseAnalysisContract {
        BaseAnalysisContract {
            path,
            format: OutputFormat::Json,
            output: None,
            top_files: Some(10),
            include_tests: false,
            timeout: 60,
        }
    }

    #[tokio::test]
    async fn test_quality_gate_standard() {
        let temp_dir = create_test_dir();
        let service = ContractService::new().unwrap();

        let contract = QualityGateContract {
            base: create_base_contract(temp_dir.path().to_path_buf()),
            profile: QualityProfile::Standard,
            file: None,
            fail_on_violation: false,
            verbose: true,
        };

        let result = service.quality_gate(contract).await;
        assert!(result.is_ok(), "quality_gate should succeed");

        let value = result.unwrap();
        assert!(value.get("passed").is_some());
        assert!(value.get("profile").is_some());
    }

    #[tokio::test]
    async fn test_quality_gate_strict() {
        let temp_dir = create_test_dir();
        let service = ContractService::new().unwrap();

        let contract = QualityGateContract {
            base: create_base_contract(temp_dir.path().to_path_buf()),
            profile: QualityProfile::Strict,
            file: None,
            fail_on_violation: false,
            verbose: false,
        };

        let result = service.quality_gate(contract).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_quality_gate_extreme() {
        let temp_dir = create_test_dir();
        let service = ContractService::new().unwrap();

        let contract = QualityGateContract {
            base: create_base_contract(temp_dir.path().to_path_buf()),
            profile: QualityProfile::Extreme,
            file: None,
            fail_on_violation: false,
            verbose: true,
        };

        let result = service.quality_gate(contract).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_quality_gate_toyota() {
        let temp_dir = create_test_dir();
        let service = ContractService::new().unwrap();

        let contract = QualityGateContract {
            base: create_base_contract(temp_dir.path().to_path_buf()),
            profile: QualityProfile::Toyota,
            file: None,
            fail_on_violation: false,
            verbose: false,
        };

        let result = service.quality_gate(contract).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_quality_gate_with_file() {
        let temp_dir = create_test_dir();
        let test_file = create_test_file(&temp_dir);
        let service = ContractService::new().unwrap();

        let contract = QualityGateContract {
            base: create_base_contract(temp_dir.path().to_path_buf()),
            profile: QualityProfile::Standard,
            file: Some(test_file),
            fail_on_violation: false,
            verbose: true,
        };

        let result = service.quality_gate(contract).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_refactor_auto_dry_run() {
        let temp_dir = create_test_dir();
        let test_file = create_test_file(&temp_dir);
        let service = ContractService::new().unwrap();

        let contract = RefactorAutoContract {
            file: test_file,
            format: OutputFormat::Json,
            output: None,
            target_complexity: 10,
            dry_run: true,
            timeout: 60,
        };

        let result = service.refactor_auto(contract).await;
        assert!(result.is_ok(), "refactor_auto should succeed");

        let value = result.unwrap();
        assert!(value.get("plan").is_some());
        assert_eq!(value.get("dry_run").and_then(|v| v.as_bool()), Some(true));
    }

    #[tokio::test]
    async fn test_refactor_auto_apply() {
        let temp_dir = create_test_dir();
        let test_file = create_test_file(&temp_dir);
        let service = ContractService::new().unwrap();

        let contract = RefactorAutoContract {
            file: test_file,
            format: OutputFormat::Markdown,
            output: None,
            target_complexity: 5,
            dry_run: false,
            timeout: 120,
        };

        let result = service.refactor_auto(contract).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        assert_eq!(value.get("dry_run").and_then(|v| v.as_bool()), Some(false));
    }

    #[tokio::test]
    async fn test_output_format_variations() {
        let temp_dir = create_test_dir();
        let service = ContractService::new().unwrap();

        let formats = [
            OutputFormat::Table,
            OutputFormat::Json,
            OutputFormat::Yaml,
            OutputFormat::Markdown,
            OutputFormat::Csv,
            OutputFormat::Summary,
        ];

        for format in formats {
            let mut base = create_base_contract(temp_dir.path().to_path_buf());
            base.format = format;

            let contract = AnalyzeSatdContract {
                base,
                severity: None,
                critical_only: false,
                strict: false,
                fail_on_violation: false,
            };

            let result = service.analyze_satd(contract).await;
            assert!(result.is_ok(), "Should work with format {:?}", format);
        }
    }

    #[tokio::test]
    async fn test_with_output_path() {
        let temp_dir = create_test_dir();
        let service = ContractService::new().unwrap();
        let output_path = temp_dir.path().join("results.json");

        let mut base = create_base_contract(temp_dir.path().to_path_buf());
        base.output = Some(output_path);

        let contract = AnalyzeTdgContract {
            base,
            threshold: 1.0,
            include_components: true,
            critical_only: false,
        };

        let result = service.analyze_tdg(contract).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_include_tests_enabled() {
        let temp_dir = create_test_dir();
        let service = ContractService::new().unwrap();

        let mut base = create_base_contract(temp_dir.path().to_path_buf());
        base.include_tests = true;

        let contract = AnalyzeDeadCodeContract {
            base,
            include_unreachable: false,
            min_dead_lines: 0,
            max_percentage: 100.0,
            fail_on_violation: false,
        };

        let result = service.analyze_dead_code(contract).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_top_files_configuration() {
        let temp_dir = create_test_dir();
        let service = ContractService::new().unwrap();

        // Test with different top_files values
        for top in [Some(1), Some(100), None] {
            let mut base = create_base_contract(temp_dir.path().to_path_buf());
            base.top_files = top;

            let contract = AnalyzeComplexityContract {
                base,
                max_cyclomatic: None,
                max_cognitive: None,
                max_halstead: None,
            };

            let result = service.analyze_complexity(contract).await;
            assert!(result.is_ok(), "Should work with top_files {:?}", top);
        }
    }

    #[tokio::test]
    async fn test_timeout_configuration() {
        let temp_dir = create_test_dir();
        let service = ContractService::new().unwrap();

        let mut base = create_base_contract(temp_dir.path().to_path_buf());
        base.timeout = 300; // 5 minutes

        let contract = AnalyzeLintHotspotContract {
            base,
            file: None,
            max_density: 10.0,
            min_confidence: 0.5,
            enforce: false,
            dry_run: false,
        };

        let result = service.analyze_lint_hotspot(contract).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_all_satd_severity_levels() {
        let temp_dir = create_test_dir();
        let service = ContractService::new().unwrap();

        for severity in [
            SatdSeverity::Low,
            SatdSeverity::Medium,
            SatdSeverity::High,
            SatdSeverity::Critical,
        ] {
            let contract = AnalyzeSatdContract {
                base: create_base_contract(temp_dir.path().to_path_buf()),
                severity: Some(severity),
                critical_only: false,
                strict: false,
                fail_on_violation: false,
            };

            let result = service.analyze_satd(contract).await;
            assert!(result.is_ok(), "Should work with severity {:?}", severity);
        }
    }

    #[test]
    fn test_arc_clone_behavior() {
        let service = ContractService::new().unwrap();

        // Verify Arc cloning works correctly
        let inner_clone = Arc::clone(&service.inner);
        assert_eq!(Arc::strong_count(&service.inner), 2);

        drop(inner_clone);
        assert_eq!(Arc::strong_count(&service.inner), 1);
    }

    #[tokio::test]
    async fn test_metadata_generation() {
        let temp_dir = create_test_dir();
        let service = ContractService::new().unwrap();

        let base = create_base_contract(temp_dir.path().to_path_buf());
        let contract = AnalyzeComplexityContract {
            base: base.clone(),
            max_cyclomatic: None,
            max_cognitive: None,
            max_halstead: None,
        };

        let result = service.analyze_complexity(contract).await.unwrap();
        let metadata = result.get("metadata").unwrap();

        // Verify metadata fields
        assert!(metadata.get("path").is_some());
        assert!(metadata.get("format").is_some());
        assert!(metadata.get("include_tests").is_some());
        assert!(metadata.get("timeout").is_some());
        assert!(metadata.get("timestamp").is_some());
    }

    #[tokio::test]
    async fn test_multiple_concurrent_requests() {
        let temp_dir = create_test_dir();
        let service = ContractService::new().unwrap();

        // Run multiple async requests concurrently
        let futures: Vec<_> = (0..5)
            .map(|_| {
                let contract = AnalyzeComplexityContract {
                    base: create_base_contract(temp_dir.path().to_path_buf()),
                    max_cyclomatic: None,
                    max_cognitive: None,
                    max_halstead: None,
                };
                service.analyze_complexity(contract)
            })
            .collect();

        let results = futures::future::join_all(futures).await;

        for result in results {
            assert!(result.is_ok());
        }
    }
}
