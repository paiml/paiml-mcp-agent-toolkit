// Coverage tests for analysis contracts: complexity, SATD, dead code, TDG,
// lint hotspot, and entropy with various configurations.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use crate::contracts::{BaseAnalysisContract, OutputFormat, SatdSeverity};
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

    #[test]
    fn test_contract_service_new() {
        let service = ContractService::new();
        assert!(service.is_ok(), "ContractService::new() should succeed");
    }

    #[test]
    fn test_unified_service_alias() {
        // Verify that UnifiedService is an alias for ContractService
        let service: Result<UnifiedService> = ContractService::new();
        assert!(service.is_ok());
    }

    #[test]
    fn test_service_arc_inner() {
        let service = ContractService::new().unwrap();
        // Verify Arc is properly initialized
        assert!(Arc::strong_count(&service.inner) >= 1);
    }

    #[tokio::test]
    async fn test_analyze_complexity_basic() {
        let temp_dir = create_test_dir();
        let service = ContractService::new().unwrap();

        let contract = AnalyzeComplexityContract {
            base: create_base_contract(temp_dir.path().to_path_buf()),
            max_cyclomatic: Some(20),
            max_cognitive: Some(15),
            max_halstead: Some(50.0),
        };

        let result = service.analyze_complexity(contract).await;
        assert!(result.is_ok(), "analyze_complexity should succeed");

        let value = result.unwrap();
        assert!(value.get("summary").is_some());
        assert!(value.get("results").is_some());
        assert!(value.get("metadata").is_some());
    }

    #[tokio::test]
    async fn test_analyze_complexity_no_thresholds() {
        let temp_dir = create_test_dir();
        let service = ContractService::new().unwrap();

        let contract = AnalyzeComplexityContract {
            base: create_base_contract(temp_dir.path().to_path_buf()),
            max_cyclomatic: None,
            max_cognitive: None,
            max_halstead: None,
        };

        let result = service.analyze_complexity(contract).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_satd_with_severity() {
        let temp_dir = create_test_dir();
        let service = ContractService::new().unwrap();

        let contract = AnalyzeSatdContract {
            base: create_base_contract(temp_dir.path().to_path_buf()),
            severity: Some(SatdSeverity::High),
            critical_only: false,
            strict: true,
            fail_on_violation: false,
        };

        let result = service.analyze_satd(contract).await;
        assert!(result.is_ok(), "analyze_satd should succeed");

        let value = result.unwrap();
        assert!(value.get("summary").is_some());
    }

    #[tokio::test]
    async fn test_analyze_satd_no_severity() {
        let temp_dir = create_test_dir();
        let service = ContractService::new().unwrap();

        let contract = AnalyzeSatdContract {
            base: create_base_contract(temp_dir.path().to_path_buf()),
            severity: None,
            critical_only: true,
            strict: false,
            fail_on_violation: false,
        };

        let result = service.analyze_satd(contract).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_dead_code_with_unreachable() {
        let temp_dir = create_test_dir();
        let service = ContractService::new().unwrap();

        let contract = AnalyzeDeadCodeContract {
            base: create_base_contract(temp_dir.path().to_path_buf()),
            include_unreachable: true,
            min_dead_lines: 5,
            max_percentage: 25.0,
            fail_on_violation: false,
        };

        let result = service.analyze_dead_code(contract).await;
        assert!(result.is_ok(), "analyze_dead_code should succeed");

        let value = result.unwrap();
        assert!(value.get("results").is_some());
    }

    #[tokio::test]
    async fn test_analyze_dead_code_without_unreachable() {
        let temp_dir = create_test_dir();
        let service = ContractService::new().unwrap();

        let contract = AnalyzeDeadCodeContract {
            base: create_base_contract(temp_dir.path().to_path_buf()),
            include_unreachable: false,
            min_dead_lines: 1,
            max_percentage: 50.0,
            fail_on_violation: false,
        };

        let result = service.analyze_dead_code(contract).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_tdg_with_components() {
        let temp_dir = create_test_dir();
        let service = ContractService::new().unwrap();

        let contract = AnalyzeTdgContract {
            base: create_base_contract(temp_dir.path().to_path_buf()),
            threshold: 2.0,
            include_components: true,
            critical_only: false,
        };

        let result = service.analyze_tdg(contract).await;
        assert!(result.is_ok(), "analyze_tdg should succeed");

        let value = result.unwrap();
        assert!(value.get("summary").is_some());
    }

    #[tokio::test]
    async fn test_analyze_tdg_critical_only() {
        let temp_dir = create_test_dir();
        let service = ContractService::new().unwrap();

        let contract = AnalyzeTdgContract {
            base: create_base_contract(temp_dir.path().to_path_buf()),
            threshold: 3.0,
            include_components: false,
            critical_only: true,
        };

        let result = service.analyze_tdg(contract).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_lint_hotspot_basic() {
        let temp_dir = create_test_dir();
        let service = ContractService::new().unwrap();

        let contract = AnalyzeLintHotspotContract {
            base: create_base_contract(temp_dir.path().to_path_buf()),
            file: None,
            max_density: 5.0,
            min_confidence: 0.7,
            enforce: false,
            dry_run: false,
        };

        let result = service.analyze_lint_hotspot(contract).await;
        assert!(result.is_ok(), "analyze_lint_hotspot should succeed");
    }

    #[tokio::test]
    async fn test_analyze_lint_hotspot_with_file() {
        let temp_dir = create_test_dir();
        let test_file = create_test_file(&temp_dir);
        let service = ContractService::new().unwrap();

        let contract = AnalyzeLintHotspotContract {
            base: create_base_contract(temp_dir.path().to_path_buf()),
            file: Some(test_file),
            max_density: 10.0,
            min_confidence: 0.5,
            enforce: true,
            dry_run: true,
        };

        let result = service.analyze_lint_hotspot(contract).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_entropy_basic() {
        let temp_dir = create_test_dir();
        let service = ContractService::new().unwrap();

        let contract = AnalyzeEntropyContract {
            base: create_base_contract(temp_dir.path().to_path_buf()),
            min_severity: Some("low".to_string()),
            top_violations: Some(10),
            file: None,
        };

        let result = service.analyze_entropy(contract).await;
        assert!(result.is_ok(), "analyze_entropy should succeed");
    }

    #[tokio::test]
    async fn test_analyze_entropy_medium_severity() {
        let temp_dir = create_test_dir();
        let service = ContractService::new().unwrap();

        let contract = AnalyzeEntropyContract {
            base: create_base_contract(temp_dir.path().to_path_buf()),
            min_severity: Some("medium".to_string()),
            top_violations: None,
            file: None,
        };

        let result = service.analyze_entropy(contract).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_entropy_high_severity() {
        let temp_dir = create_test_dir();
        let service = ContractService::new().unwrap();

        let contract = AnalyzeEntropyContract {
            base: create_base_contract(temp_dir.path().to_path_buf()),
            min_severity: Some("high".to_string()),
            top_violations: Some(5),
            file: None,
        };

        let result = service.analyze_entropy(contract).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_entropy_with_file() {
        let temp_dir = create_test_dir();
        let test_file = create_test_file(&temp_dir);
        let service = ContractService::new().unwrap();

        let contract = AnalyzeEntropyContract {
            base: create_base_contract(temp_dir.path().to_path_buf()),
            min_severity: None,
            top_violations: Some(20),
            file: Some(test_file),
        };

        let result = service.analyze_entropy(contract).await;
        assert!(result.is_ok());
    }
}
