//! Simple service tests - Part 2
//! analyze_lint_hotspot and analyze_entropy tests

use super::simple_service_tests_part1::{create_temp_dir, create_temp_file};
use super::*;
use std::path::PathBuf;

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
