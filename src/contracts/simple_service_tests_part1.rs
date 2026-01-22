//! Simple service tests - Part 1
//! Helper functions and analyze_complexity/satd/dead_code/tdg tests

use super::*;
use std::path::PathBuf;
use tempfile::TempDir;

// Helper function to create a valid temp directory for tests
pub(super) fn create_temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

// Helper function to create a valid temp file for tests
pub(super) fn create_temp_file(dir: &TempDir, name: &str) -> PathBuf {
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
    let unreachable_blocks = first_result
        .get("unreachable_blocks")
        .unwrap()
        .as_u64()
        .unwrap();
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
    let unreachable_blocks = first_result
        .get("unreachable_blocks")
        .unwrap()
        .as_u64()
        .unwrap();
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
