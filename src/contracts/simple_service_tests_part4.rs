//! Simple service tests - Part 4
//! Output format coverage, edge case, and property-based tests

#[allow(unused_imports)]
use super::simple_service_tests_part1::{create_temp_dir, create_temp_file};
#[allow(unused_imports)]
use super::*;
use std::path::PathBuf;

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
