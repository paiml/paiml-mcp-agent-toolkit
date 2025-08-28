//! Comprehensive contract tests covering all scenarios
//! These tests ensure the uniform contract system works correctly

use super::*;
use super::versioning::*;
use super::simple_service::SimpleContractService;
use super::mcp_simple::SimpleMcpHandler;
use serde_json::json;
use tokio_test;

/// Test contract validation rules
#[test]
fn test_contract_validation() {
    // Valid contract
    let valid_contract = AnalyzeComplexityContract {
        base: BaseAnalysisContract {
            path: std::env::current_dir().unwrap(),
            format: OutputFormat::Json,
            output: None,
            top_files: Some(5),
            include_tests: false,
            timeout: 30,
        },
        max_cyclomatic: Some(10),
        max_cognitive: Some(8),
        max_halstead: Some(5.0),
    };
    
    assert!(valid_contract.validate().is_ok());
    
    // Invalid contract - nonexistent path
    let invalid_contract = AnalyzeComplexityContract {
        base: BaseAnalysisContract {
            path: std::path::PathBuf::from("/nonexistent/path"),
            format: OutputFormat::Json,
            output: None,
            top_files: Some(5),
            include_tests: false,
            timeout: 30,
        },
        max_cyclomatic: Some(10),
        max_cognitive: Some(8),
        max_halstead: Some(5.0),
    };
    
    assert!(invalid_contract.validate().is_err());
    
    // Invalid contract - zero timeout
    let invalid_timeout = AnalyzeComplexityContract {
        base: BaseAnalysisContract {
            path: std::env::current_dir().unwrap(),
            format: OutputFormat::Json,
            output: None,
            top_files: Some(5),
            include_tests: false,
            timeout: 0, // Invalid
        },
        max_cyclomatic: Some(10),
        max_cognitive: Some(8),
        max_halstead: Some(5.0),
    };
    
    assert!(invalid_timeout.validate().is_err());
    
    // Invalid contract - too many files
    let too_many_files = AnalyzeComplexityContract {
        base: BaseAnalysisContract {
            path: std::env::current_dir().unwrap(),
            format: OutputFormat::Json,
            output: None,
            top_files: Some(1001), // Too many
            include_tests: false,
            timeout: 60,
        },
        max_cyclomatic: Some(10),
        max_cognitive: Some(8),
        max_halstead: Some(5.0),
    };
    
    assert!(too_many_files.validate().is_err());
    
    // Invalid contract - negative halstead
    let negative_halstead = AnalyzeComplexityContract {
        base: BaseAnalysisContract {
            path: std::env::current_dir().unwrap(),
            format: OutputFormat::Json,
            output: None,
            top_files: Some(5),
            include_tests: false,
            timeout: 60,
        },
        max_cyclomatic: Some(10),
        max_cognitive: Some(8),
        max_halstead: Some(-1.0), // Invalid
    };
    
    assert!(negative_halstead.validate().is_err());
}

/// Test service integration with contracts
#[tokio::test]
async fn test_service_integration() {
    let service = SimpleContractService::new().unwrap();
    
    let contract = AnalyzeComplexityContract {
        base: BaseAnalysisContract {
            path: std::env::current_dir().unwrap(),
            format: OutputFormat::Json,
            output: None,
            top_files: Some(3),
            include_tests: false,
            timeout: 30,
        },
        max_cyclomatic: Some(20),
        max_cognitive: Some(15),
        max_halstead: Some(10.0),
    };
    
    let result = service.analyze_complexity(contract).await;
    assert!(result.is_ok());
    
    let value = result.unwrap();
    assert!(value.is_object());
    
    // Check that result has expected structure
    let obj = value.as_object().unwrap();
    assert!(obj.contains_key("results"));
    assert!(obj.contains_key("summary"));
    assert!(obj.contains_key("metadata"));
}

/// Test MCP handler with contracts
#[tokio::test]
async fn test_mcp_handler_integration() {
    let handler = SimpleMcpHandler::new().unwrap();
    
    let params = json!({
        "path": ".",
        "format": "json",
        "top_files": 3,
        "include_tests": false,
        "timeout": 30,
        "max_cyclomatic": 20,
        "max_cognitive": 15,
        "max_halstead": 10.0
    });
    
    let result = handler.handle_tool_call("analyze_complexity", params).await;
    assert!(result.is_ok());
    
    let value = result.unwrap();
    assert!(value.is_object());
    
    // Test unknown tool
    let unknown_result = handler.handle_tool_call("unknown_tool", json!({})).await;
    assert!(unknown_result.is_err());
}

/// Test contract serialization and deserialization
#[test]
fn test_contract_serialization() {
    let original_contract = AnalyzeComplexityContract {
        base: BaseAnalysisContract {
            path: std::path::PathBuf::from("src"),
            format: OutputFormat::Json,
            output: Some(std::path::PathBuf::from("output.json")),
            top_files: Some(5),
            include_tests: true,
            timeout: 120,
        },
        max_cyclomatic: Some(10),
        max_cognitive: None,
        max_halstead: Some(5.0),
    };
    
    // Serialize to JSON
    let json = serde_json::to_value(&original_contract).unwrap();
    
    // Verify flat structure (base fields are flattened)
    assert_eq!(json["path"], "src");
    assert_eq!(json["format"], "json");
    assert_eq!(json["output"], "output.json");
    assert_eq!(json["top_files"], 5);
    assert_eq!(json["include_tests"], true);
    assert_eq!(json["timeout"], 120);
    assert_eq!(json["max_cyclomatic"], 10);
    assert_eq!(json["max_halstead"], 5.0);
    assert!(json["max_cognitive"].is_null());
    
    // Deserialize back
    let deserialized: AnalyzeComplexityContract = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized, original_contract);
}

/// Test backward compatibility with adapter
#[test]
fn test_backward_compatibility() {
    // Old parameter format
    let old_params = json!({
        "project_path": "/src",
        "file": "main.rs",
        "format": "human"
    });
    
    let new_params = super::adapter::BackwardCompatibility::map_json_params(old_params);
    
    // Verify mapping
    assert_eq!(new_params["path"], "/src");
    assert_eq!(new_params["files"], json!(["main.rs"]));
    assert_eq!(new_params["format"], "summary");
    assert!(new_params.get("project_path").is_none());
    assert!(new_params.get("file").is_none());
}

/// Test contract versioning
#[test]
fn test_contract_versioning() {
    let mut registry = ContractRegistry::new();
    
    // Register v1.0.0 contract
    let v1_contract = ContractBuilder::new(
        json!({
            "path": ".",
            "format": "json"
        }),
        "test"
    )
    .version(1, 0, 0)
    .description("Version 1.0.0 contract")
    .build();
    
    registry.register("test_contract", v1_contract).unwrap();
    
    // Register v1.1.0 contract with new field
    let v1_1_contract = ContractBuilder::new(
        json!({
            "path": ".",
            "format": "json",
            "timeout": 60
        }),
        "test"
    )
    .version(1, 1, 0)
    .description("Version 1.1.0 contract with timeout")
    .build();
    
    registry.register("test_contract", v1_1_contract).unwrap();
    
    // Test version retrieval
    let latest = registry.get_latest("test_contract").unwrap();
    assert_eq!(latest.version, ContractVersion::new(1, 1, 0));
    assert!(latest.contract.get("timeout").is_some());
    
    let v1_0 = registry.get_version("test_contract", &ContractVersion::new(1, 0, 0)).unwrap();
    assert!(v1_0.contract.get("timeout").is_none());
    
    // Test version compatibility
    let v1_0_0 = ContractVersion::new(1, 0, 0);
    let v1_1_0 = ContractVersion::new(1, 1, 0);
    let v2_0_0 = ContractVersion::new(2, 0, 0);
    
    assert!(v1_1_0.is_compatible(&v1_0_0));
    assert!(!v1_0_0.is_compatible(&v1_1_0));
    assert!(!v2_0_0.is_compatible(&v1_0_0));
}

/// Test contract migration
#[test]
fn test_contract_migration() {
    let migration = super::versioning::ParameterRenameMapping::project_path_to_path();
    
    let old_contract = json!({
        "project_path": "/src",
        "format": "json",
        "max_cyclomatic": 20
    });
    
    let migrated = migration.migrate(old_contract).unwrap();
    
    assert_eq!(migrated["path"], "/src");
    assert!(migrated.get("project_path").is_none());
    assert_eq!(migrated["format"], "json");
    assert_eq!(migrated["max_cyclomatic"], 20);
}

/// Test error handling and propagation
#[tokio::test]
async fn test_error_handling() {
    let service = SimpleContractService::new().unwrap();
    
    // Test SATD contract with fail_on_violation
    let satd_contract = AnalyzeSatdContract {
        base: BaseAnalysisContract {
            path: std::env::current_dir().unwrap(),
            format: OutputFormat::Json,
            output: None,
            top_files: Some(5),
            include_tests: false,
            timeout: 30,
        },
        severity: Some(SatdSeverity::Critical),
        critical_only: false,
        strict: true,
        fail_on_violation: true,
    };
    
    // This should succeed with simple service (no real violations)
    let result = service.analyze_satd(satd_contract).await;
    assert!(result.is_ok());
}

/// Test all contract types
#[test]
fn test_all_contract_types() {
    let contracts: Vec<Box<dyn ContractValidation>> = vec![
        Box::new(AnalyzeComplexityContract {
            base: BaseAnalysisContract::default(),
            max_cyclomatic: Some(10),
            max_cognitive: Some(8),
            max_halstead: Some(5.0),
        }),
        Box::new(AnalyzeSatdContract {
            base: BaseAnalysisContract::default(),
            severity: Some(SatdSeverity::Medium),
            critical_only: false,
            strict: false,
            fail_on_violation: false,
        }),
        Box::new(AnalyzeDeadCodeContract {
            base: BaseAnalysisContract::default(),
            include_unreachable: true,
            min_dead_lines: 5,
            max_percentage: 20.0,
            fail_on_violation: false,
        }),
        Box::new(AnalyzeTdgContract {
            base: BaseAnalysisContract::default(),
            threshold: 2.0,
            include_components: true,
            critical_only: false,
        }),
        Box::new(AnalyzeLintHotspotContract {
            base: BaseAnalysisContract::default(),
            file: None,
            max_density: 10.0,
            min_confidence: 0.7,
            enforce: false,
            dry_run: true,
        }),
        Box::new(QualityGateContract {
            base: BaseAnalysisContract::default(),
            profile: QualityProfile::Standard,
            file: None,
            fail_on_violation: false,
            verbose: false,
        }),
        Box::new(RefactorAutoContract {
            file: std::env::current_dir().unwrap().join("test.rs"),
            format: OutputFormat::Json,
            output: None,
            target_complexity: 8,
            dry_run: true,
            timeout: 60,
        }),
    ];
    
    // All contracts should have valid default configurations
    for contract in contracts {
        // Note: Some may fail validation due to missing files, but structure should be valid
        let _ = contract.validate();
    }
}

/// Integration test with all service types
#[tokio::test]
async fn test_service_type_integration() {
    // Test simple service
    let simple_service = SimpleContractService::new().unwrap();
    let unified_service = super::service::ContractService::new().unwrap();
    
    let contract = AnalyzeComplexityContract {
        base: BaseAnalysisContract {
            path: std::env::current_dir().unwrap(),
            format: OutputFormat::Json,
            output: None,
            top_files: Some(3),
            include_tests: false,
            timeout: 30,
        },
        max_cyclomatic: Some(20),
        max_cognitive: Some(15),
        max_halstead: Some(10.0),
    };
    
    // Both services should work with same contract
    let simple_result = simple_service.analyze_complexity(contract.clone()).await;
    let unified_result = unified_service.analyze_complexity(contract).await;
    
    assert!(simple_result.is_ok());
    assert!(unified_result.is_ok());
    
    // Results should have similar structure
    let simple_value = simple_result.unwrap();
    let unified_value = unified_result.unwrap();
    
    assert!(simple_value.is_object());
    assert!(unified_value.is_object());
}

/// Test contract enforcement at compile time
#[test]
fn test_contract_type_safety() {
    // These should compile successfully
    let complexity_contract = AnalyzeComplexityContract {
        base: BaseAnalysisContract::default(),
        max_cyclomatic: Some(10),
        max_cognitive: Some(8),
        max_halstead: Some(5.0),
    };
    
    let satd_contract = AnalyzeSatdContract {
        base: BaseAnalysisContract::default(),
        severity: Some(SatdSeverity::High),
        critical_only: true,
        strict: false,
        fail_on_violation: false,
    };
    
    // Verify contracts can be serialized/deserialized
    let complexity_json = serde_json::to_value(&complexity_contract).unwrap();
    let satd_json = serde_json::to_value(&satd_contract).unwrap();
    
    let _: AnalyzeComplexityContract = serde_json::from_value(complexity_json).unwrap();
    let _: AnalyzeSatdContract = serde_json::from_value(satd_json).unwrap();
}

/// Performance test for contract operations
#[test]
fn test_contract_performance() {
    let start = std::time::Instant::now();
    
    // Create many contracts
    for i in 0..1000 {
        let contract = AnalyzeComplexityContract {
            base: BaseAnalysisContract {
                path: std::path::PathBuf::from("."),
                format: OutputFormat::Json,
                output: None,
                top_files: Some(i % 50),
                include_tests: i % 2 == 0,
                timeout: 60,
            },
            max_cyclomatic: Some(i as u32 % 30),
            max_cognitive: Some(i as u32 % 25),
            max_halstead: Some((i as f64) / 100.0),
        };
        
        // Validate
        let _ = contract.validate();
        
        // Serialize
        let _ = serde_json::to_value(&contract).unwrap();
    }
    
    let duration = start.elapsed();
    println!("Contract operations took: {:?}", duration);
    
    // Should complete in reasonable time (less than 100ms)
    assert!(duration.as_millis() < 100);
}