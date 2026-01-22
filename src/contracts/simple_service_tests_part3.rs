//! Simple service tests - Part 3
//! quality_gate, refactor_auto, create_metadata, and response serialization tests

use super::simple_service_tests_part1::{create_temp_dir, create_temp_file};
use super::*;
use std::path::PathBuf;

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
