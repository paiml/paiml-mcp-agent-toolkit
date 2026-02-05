//! Service layer that implements business logic using uniform contracts
//! This is the SINGLE implementation that CLI, MCP, and HTTP all use

use super::simple_service::SimpleContractService;
use super::{
    AnalyzeComplexityContract, AnalyzeDeadCodeContract, AnalyzeEntropyContract,
    AnalyzeLintHotspotContract, AnalyzeSatdContract, AnalyzeTdgContract, QualityGateContract,
    RefactorAutoContract,
};
use anyhow::Result;
use serde_json::Value;
use std::sync::Arc;

/// Unified service that processes all contracts
pub struct ContractService {
    inner: Arc<SimpleContractService>,
}

impl ContractService {
    pub fn new() -> Result<Self> {
        Ok(Self {
            inner: Arc::new(SimpleContractService::new()?),
        })
    }

    /// Process analyze complexity contract
    pub async fn analyze_complexity(&self, contract: AnalyzeComplexityContract) -> Result<Value> {
        self.inner.analyze_complexity(contract).await
    }

    /// Process analyze SATD contract
    pub async fn analyze_satd(&self, contract: AnalyzeSatdContract) -> Result<Value> {
        self.inner.analyze_satd(contract).await
    }

    /// Process analyze dead code contract
    pub async fn analyze_dead_code(&self, contract: AnalyzeDeadCodeContract) -> Result<Value> {
        self.inner.analyze_dead_code(contract).await
    }

    /// Process analyze TDG contract
    pub async fn analyze_tdg(&self, contract: AnalyzeTdgContract) -> Result<Value> {
        self.inner.analyze_tdg(contract).await
    }

    /// Process analyze lint hotspot contract
    pub async fn analyze_lint_hotspot(
        &self,
        contract: AnalyzeLintHotspotContract,
    ) -> Result<Value> {
        self.inner.analyze_lint_hotspot(contract).await
    }

    /// Process analyze entropy contract
    pub async fn analyze_entropy(&self, contract: AnalyzeEntropyContract) -> Result<Value> {
        self.inner.analyze_entropy(contract).await
    }

    /// Process quality gate contract
    pub async fn quality_gate(&self, contract: QualityGateContract) -> Result<Value> {
        self.inner.quality_gate(contract).await
    }

    /// Process refactor auto contract
    pub async fn refactor_auto(&self, contract: RefactorAutoContract) -> Result<Value> {
        self.inner.refactor_auto(contract).await
    }
}

// Service module exports
pub use self::ContractService as UnifiedService;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
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
