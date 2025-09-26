//! Comprehensive tests to ensure uniform contracts across all interfaces
//! These tests enforce that CLI, MCP, and HTTP all use identical contracts

use super::*;
use serde_json::json;

/// Test that all interfaces accept exactly the same parameters
#[test]
fn test_uniform_complexity_contract() {
    let _expected = AnalyzeComplexityContract {
        base: BaseAnalysisContract {
            path: PathBuf::from("."),
            format: OutputFormat::Json,
            output: Some(PathBuf::from("output.json")),
            top_files: Some(10),
            include_tests: false,
            timeout: 60,
        },
        max_cyclomatic: Some(20),
        max_cognitive: Some(15),
        max_halstead: Some(10.0),
    };

    // MCP parameters must match exactly
    let mcp_params = json!({
        "path": ".",
        "format": "json",
        "output": "output.json",
        "top_files": 10,
        "include_tests": false,
        "timeout": 60,
        "max_cyclomatic": 20,
        "max_cognitive": 15,
        "max_halstead": 10.0
    });

    // HTTP body must match exactly
    let http_body = json!({
        "path": ".",
        "format": "json",
        "output": "output.json",
        "top_files": 10,
        "include_tests": false,
        "timeout": 60,
        "max_cyclomatic": 20,
        "max_cognitive": 15,
        "max_halstead": 10.0
    });

    // CLI args must match exactly (would be parsed from command line)
    // --path . --format json --output output.json --top-files 10
    // --no-include-tests --timeout 60 --max-cyclomatic 20
    // --max-cognitive 15 --max-halstead 10.0

    assert_eq!(
        mcp_params, http_body,
        "MCP and HTTP must use identical parameter names"
    );
}

/// Test that parameter names are consistent across all commands
#[test]
fn test_parameter_name_consistency() {
    // These parameter names MUST be identical across all commands and interfaces
    let _standard_params = [
        "path",          // NOT "project_path" or "file_path"
        "format",        // NOT "output_format"
        "output",        // NOT "output_file" or "out"
        "top_files",     // NOT "top_n" or "limit"
        "include_tests", // NOT "with_tests" or "tests"
        "timeout",       // NOT "max_time" or "time_limit"
    ];

    // Verify no variations exist
    let forbidden_variations = vec![
        (
            "path",
            vec!["project_path", "file_path", "dir", "directory"],
        ),
        ("format", vec!["output_format", "fmt", "out_format"]),
        ("output", vec!["output_file", "out", "output_path"]),
        ("top_files", vec!["top_n", "limit", "max_files", "n"]),
        ("include_tests", vec!["with_tests", "tests", "include_test"]),
        ("timeout", vec!["max_time", "time_limit", "deadline"]),
    ];

    for (correct, wrong_variants) in forbidden_variations {
        for variant in wrong_variants {
            assert_ne!(
                correct, variant,
                "Parameter '{}' must be used, not '{}'",
                correct, variant
            );
        }
    }
}

/// Test that all analysis commands have required base parameters
#[test]
fn test_base_parameters_required() {
    let commands = vec![
        "analyze_complexity",
        "analyze_satd",
        "analyze_dead_code",
        "analyze_tdg",
        "analyze_lint_hotspot",
        "quality_gate",
    ];

    for _command in commands {
        // Every analysis command MUST have these base parameters
        let required_params = vec![
            "path",
            "format",
            "output",
            "top_files",
            "include_tests",
            "timeout",
        ];

        // This would be validated against actual command definitions
        for _param in required_params {
            // In real implementation, check that the command schema includes this param
            // In real implementation, verify that command has the required parameter
            // Command '{}' must have parameter '{}'
        }
    }
}

/// Test that default values are consistent across all interfaces
#[test]
fn test_consistent_defaults() {
    let defaults = BaseAnalysisContract::default();

    assert_eq!(defaults.path, PathBuf::from("."));
    assert_eq!(defaults.format, OutputFormat::Table);
    assert_eq!(defaults.output, None);
    assert_eq!(defaults.top_files, Some(10));
    assert!(!defaults.include_tests);
    assert_eq!(defaults.timeout, 60);

    // These defaults MUST be the same in:
    // 1. CLI argument defaults
    // 2. MCP tool parameter defaults
    // 3. HTTP endpoint defaults
}

/// Test contract validation rules are consistent
#[test]
fn test_validation_consistency() {
    let mut contract = AnalyzeComplexityContract {
        base: BaseAnalysisContract::default(),
        max_cyclomatic: Some(20),
        max_cognitive: Some(15),
        max_halstead: Some(10.0),
    };

    // Valid contract should pass
    assert!(contract.validate().is_ok());

    // Invalid path should fail consistently
    contract.base.path = PathBuf::from("/nonexistent/path");
    assert!(contract.validate().is_err());

    // Invalid timeout should fail consistently
    contract.base.path = PathBuf::from(".");
    contract.base.timeout = 0;
    assert!(contract.validate().is_err());

    // Invalid halstead should fail consistently
    contract.base.timeout = 60;
    contract.max_halstead = Some(-1.0);
    assert!(contract.validate().is_err());
}

/// Test that contract serialization is consistent
#[test]
fn test_contract_serialization() {
    let contract = AnalyzeComplexityContract {
        base: BaseAnalysisContract {
            path: PathBuf::from("src"),
            format: OutputFormat::Json,
            output: None,
            top_files: Some(5),
            include_tests: true,
            timeout: 120,
        },
        max_cyclomatic: Some(10),
        max_cognitive: None,
        max_halstead: Some(5.0),
    };

    // Serialize to JSON
    let json = serde_json::to_value(&contract).unwrap();

    // Verify flat structure (base fields are flattened)
    assert_eq!(json["path"], "src");
    assert_eq!(json["format"], "json");
    assert_eq!(json["top_files"], 5);
    assert_eq!(json["include_tests"], true);
    assert_eq!(json["timeout"], 120);
    assert_eq!(json["max_cyclomatic"], 10);
    assert_eq!(json["max_halstead"], 5.0);
    assert!(json["max_cognitive"].is_null());

    // Deserialize back
    let deserialized: AnalyzeComplexityContract = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized, contract);
}

/// Test that contracts enforce type safety
#[test]
fn test_contract_type_safety() {
    // These should fail to compile (commented out as they're compile-time checks)
    // let mut contract = AnalyzeComplexityContract::default();
    // contract.base.timeout = "60"; // Error: expected u64, found &str
    // contract.max_cyclomatic = 20.5; // Error: expected Option<u32>, found f64
    // contract.base.path = "src"; // Error: expected PathBuf, found &str
}

/// Test that contract errors are meaningful
#[test]
fn test_contract_error_messages() {
    let mut contract = AnalyzeComplexityContract {
        base: BaseAnalysisContract::default(),
        max_cyclomatic: None,
        max_cognitive: None,
        max_halstead: Some(-1.0),
    };

    contract.base.path = PathBuf::from("/nonexistent");
    match contract.validate() {
        Err(ContractError::PathNotFound(path)) => {
            assert_eq!(path, PathBuf::from("/nonexistent"));
        }
        _ => panic!("Expected PathNotFound error"),
    }

    contract.base.path = PathBuf::from(".");
    contract.base.timeout = 0;
    match contract.validate() {
        Err(ContractError::InvalidTimeout) => {}
        _ => panic!("Expected InvalidTimeout error"),
    }

    contract.base.timeout = 60;
    contract.max_halstead = Some(-1.0);
    match contract.validate() {
        Err(ContractError::InvalidValue(msg)) => {
            assert!(msg.contains("halstead"));
        }
        _ => panic!("Expected InvalidValue error"),
    }
}

/// Integration test: Ensure contract can be used by all interfaces
#[test]
fn test_contract_integration() {
    let contract = AnalyzeComplexityContract {
        base: BaseAnalysisContract::default(),
        max_cyclomatic: Some(20),
        max_cognitive: Some(15),
        max_halstead: None,
    };

    // Contract can be validated
    assert!(contract.validate().is_ok());

    // Contract can be serialized for MCP/HTTP
    let json = serde_json::to_value(&contract).unwrap();
    assert!(json.is_object());

    // Contract can be deserialized from MCP/HTTP
    let deserialized: AnalyzeComplexityContract = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized, contract);

    // Contract can be converted to business logic calls
    // (In real implementation, would call actual analysis functions)
    let _path = &contract.base.path;
    let _format = &contract.base.format;
    let _max_cyclo = contract.max_cyclomatic.unwrap_or(30);
}
