//! Integration tests for MCP tools with actix actors
//!
//! RED phase tests for tool-to-agent communication

use super::tools::*;
use super::*;
use crate::agents::analyzer_actor::AnalyzerActor;
use crate::agents::transformer_actor::TransformerActor;
use crate::agents::validator_actor::ValidatorActor;
use crate::agents::registry::AgentRegistry;
use actix::prelude::*;
use serde_json::json;
use std::sync::Arc;

#[actix::test]
async fn red_analyze_tool_must_communicate_with_analyzer_actor() {
    // Setup: Start analyzer actor
    let analyzer = AnalyzerActor::default().start();
    let registry = Arc::new(AgentRegistry::new());

    // Create tool with actor address
    let tool = AnalyzeTool::new_with_actor(registry, analyzer.clone());

    // Execute analyze request
    let params = json!({
        "code": "fn main() { println!(\"test\"); }",
        "language": "rust"
    });

    let result = tool.execute(params).await;

    // Must succeed
    assert!(result.is_ok(), "Expected Ok but got: {:?}", result);

    // Must return analysis results (not placeholder)
    let response = result.unwrap();
    let text = response["text"].as_str().unwrap();
    assert!(!text.contains("not yet implemented"));
}

#[actix::test]
async fn red_analyze_tool_must_handle_actor_errors() {
    // Setup with actor that will fail
    let analyzer = AnalyzerActor::default().start();
    let registry = Arc::new(AgentRegistry::new());
    let tool = AnalyzeTool::new_with_actor(registry, analyzer);

    // Invalid parameters should be rejected
    let params = json!({
        "invalid": "params"
    });

    let result = tool.execute(params).await;

    // Must return error
    assert!(result.is_err());

    // Must be INVALID_PARAMS error
    let error = result.unwrap_err();
    assert_eq!(error.code, error_codes::INVALID_PARAMS);
}

#[actix::test]
async fn red_analyze_tool_must_return_metrics_in_mcp_format() {
    let analyzer = AnalyzerActor::default().start();
    let registry = Arc::new(AgentRegistry::new());
    let tool = AnalyzeTool::new_with_actor(registry, analyzer);

    let params = json!({
        "code": "fn add(a: i32, b: i32) -> i32 { a + b }",
        "language": "rust"
    });

    let result = tool.execute(params).await.unwrap();

    // Must have MCP format
    assert!(result.is_object());
    assert_eq!(result["type"].as_str(), Some("text"));
    assert!(result["text"].is_string());
}

#[actix::test]
async fn red_analyze_tool_must_forward_priority_to_actor() {
    let analyzer = AnalyzerActor::default().start();
    let registry = Arc::new(AgentRegistry::new());
    let tool = AnalyzeTool::new_with_actor(registry, analyzer.clone());

    let params = json!({
        "code": "fn test() {}",
        "language": "rust",
        "priority": "high"
    });

    // Should not fail - priority is optional
    let result = tool.execute(params).await;
    assert!(result.is_ok(), "Expected Ok but got: {:?}", result);
}

#[actix::test]
async fn red_analyze_tool_constructor_must_accept_actor_address() {
    let analyzer = AnalyzerActor::default().start();
    let registry = Arc::new(AgentRegistry::new());

    // This should compile and work
    let _tool = AnalyzeTool::new_with_actor(registry, analyzer);
}

#[test]
fn red_analyze_tool_metadata_must_include_all_parameters() {
    let registry = Arc::new(AgentRegistry::new());
    let tool = AnalyzeTool::new(registry);
    let metadata = tool.metadata();

    let schema = &metadata.input_schema;

    // Must have code parameter
    assert!(schema["properties"]["code"].is_object());
    assert_eq!(schema["required"][0], "code");

    // Must have language parameter
    assert!(schema["properties"]["language"].is_object());
    assert_eq!(schema["required"][1], "language");

    // Must have optional metrics parameter
    assert!(schema["properties"]["metrics"].is_object());
}

// ================== TransformTool Integration Tests ==================

#[actix::test]
async fn red_transform_tool_must_communicate_with_transformer_actor() {
    // Setup: Start transformer actor
    let transformer = TransformerActor::default().start();
    let registry = Arc::new(AgentRegistry::new());

    // Create tool with actor address
    let tool = TransformTool::new_with_actor(registry, transformer.clone());

    // Execute transform request
    let params = json!({
        "code": "fn test() { let x = 1 + 2; }",
        "language": "rust",
        "transformation": "optimize"
    });

    let result = tool.execute(params).await;

    // Must succeed
    assert!(result.is_ok(), "Expected Ok but got: {:?}", result);

    // Must return transformation results (not placeholder)
    let response = result.unwrap();
    let text = response["text"].as_str().unwrap();
    assert!(!text.contains("not yet implemented"));
}

#[actix::test]
async fn red_transform_tool_must_handle_actor_errors() {
    // Setup with actor that will fail
    let transformer = TransformerActor::default().start();
    let registry = Arc::new(AgentRegistry::new());
    let tool = TransformTool::new_with_actor(registry, transformer);

    // Invalid parameters should be rejected
    let params = json!({
        "invalid": "params"
    });

    let result = tool.execute(params).await;

    // Must return error
    assert!(result.is_err());

    // Must be INVALID_PARAMS error
    let error = result.unwrap_err();
    assert_eq!(error.code, error_codes::INVALID_PARAMS);
}

#[actix::test]
async fn red_transform_tool_must_return_results_in_mcp_format() {
    let transformer = TransformerActor::default().start();
    let registry = Arc::new(AgentRegistry::new());
    let tool = TransformTool::new_with_actor(registry, transformer);

    let params = json!({
        "code": "fn main() { }",
        "language": "rust",
        "transformation": "beautify"
    });

    let result = tool.execute(params).await.unwrap();

    // Must have MCP format
    assert!(result.is_object());
    assert_eq!(result["type"].as_str(), Some("text"));
    assert!(result["text"].is_string());
}

#[actix::test]
async fn red_transform_tool_must_forward_priority_to_actor() {
    let transformer = TransformerActor::default().start();
    let registry = Arc::new(AgentRegistry::new());
    let tool = TransformTool::new_with_actor(registry, transformer.clone());

    let params = json!({
        "code": "fn test() {}",
        "language": "rust",
        "transformation": "refactor",
        "priority": "high"
    });

    // Should not fail - priority is optional
    let result = tool.execute(params).await;
    assert!(result.is_ok(), "Expected Ok but got: {:?}", result);
}

#[actix::test]
async fn red_transform_tool_constructor_must_accept_actor_address() {
    let transformer = TransformerActor::default().start();
    let registry = Arc::new(AgentRegistry::new());

    // This should compile and work
    let _tool = TransformTool::new_with_actor(registry, transformer);
}

#[test]
fn red_transform_tool_metadata_must_include_all_parameters() {
    let registry = Arc::new(AgentRegistry::new());
    let tool = TransformTool::new(registry);
    let metadata = tool.metadata();

    let schema = &metadata.input_schema;

    // Must have code parameter
    assert!(schema["properties"]["code"].is_object());
    assert_eq!(schema["required"][0], "code");

    // Must have language parameter
    assert!(schema["properties"]["language"].is_object());
    assert_eq!(schema["required"][1], "language");

    // Must have transformation parameter
    assert!(schema["properties"]["transformation"].is_object());
    assert_eq!(schema["required"][2], "transformation");

    // Must have optional options parameter
    assert!(schema["properties"]["options"].is_object());

    // Must have transformation enum values
    let transformation_enum = &schema["properties"]["transformation"]["enum"];
    assert!(transformation_enum.is_array());
    assert!(transformation_enum.as_array().unwrap().contains(&json!("optimize")));
    assert!(transformation_enum.as_array().unwrap().contains(&json!("minify")));
    assert!(transformation_enum.as_array().unwrap().contains(&json!("beautify")));
    assert!(transformation_enum.as_array().unwrap().contains(&json!("refactor")));
}

// ================== ValidateTool Integration Tests ==================

#[actix::test]
async fn red_validate_tool_must_communicate_with_analyzer_and_validator_actors() {
    // Setup: Start both analyzer and validator actors
    let analyzer = AnalyzerActor::default().start();
    let validator = ValidatorActor::default().start();
    let registry = Arc::new(AgentRegistry::new());

    // Create tool with actor addresses
    let tool = ValidateTool::new_with_actors(registry, analyzer.clone(), validator.clone());

    // Execute validate request
    let params = json!({
        "code": "fn complex() { if true { if false { for i in 0..10 { } } } }",
        "language": "rust"
    });

    let result = tool.execute(params).await;

    // Must succeed
    assert!(result.is_ok(), "Expected Ok but got: {:?}", result);

    // Must return validation results (not placeholder)
    let response = result.unwrap();
    let text = response["text"].as_str().unwrap();
    assert!(text.contains("Validation Results"));
}

#[actix::test]
async fn red_validate_tool_must_handle_actor_errors() {
    // Setup with actors
    let analyzer = AnalyzerActor::default().start();
    let validator = ValidatorActor::default().start();
    let registry = Arc::new(AgentRegistry::new());
    let tool = ValidateTool::new_with_actors(registry, analyzer, validator);

    // Invalid parameters should be rejected
    let params = json!({
        "invalid": "params"
    });

    let result = tool.execute(params).await;

    // Must return error
    assert!(result.is_err());

    // Must be INVALID_PARAMS error
    let error = result.unwrap_err();
    assert_eq!(error.code, error_codes::INVALID_PARAMS);
}

#[actix::test]
async fn red_validate_tool_must_return_results_in_mcp_format() {
    let analyzer = AnalyzerActor::default().start();
    let validator = ValidatorActor::default().start();
    let registry = Arc::new(AgentRegistry::new());
    let tool = ValidateTool::new_with_actors(registry, analyzer, validator);

    let params = json!({
        "code": "fn simple() { }",
        "language": "rust"
    });

    let result = tool.execute(params).await.unwrap();

    // Must have MCP format
    assert!(result.is_object());
    assert_eq!(result["type"].as_str(), Some("text"));
    assert!(result["text"].is_string());
}

#[actix::test]
async fn red_validate_tool_must_handle_optional_rules_parameter() {
    let analyzer = AnalyzerActor::default().start();
    let validator = ValidatorActor::default().start();
    let registry = Arc::new(AgentRegistry::new());
    let tool = ValidateTool::new_with_actors(registry, analyzer.clone(), validator.clone());

    let params = json!({
        "code": "fn test() {}",
        "language": "rust",
        "rules": ["complexity", "naming"]
    });

    // Should not fail - rules are optional
    let result = tool.execute(params).await;
    assert!(result.is_ok(), "Expected Ok but got: {:?}", result);
}

#[actix::test]
async fn red_validate_tool_constructor_must_accept_actor_addresses() {
    let analyzer = AnalyzerActor::default().start();
    let validator = ValidatorActor::default().start();
    let registry = Arc::new(AgentRegistry::new());

    // This should compile and work
    let _tool = ValidateTool::new_with_actors(registry, analyzer, validator);
}

#[test]
fn red_validate_tool_metadata_must_include_all_parameters() {
    let registry = Arc::new(AgentRegistry::new());
    let tool = ValidateTool::new(registry);
    let metadata = tool.metadata();

    let schema = &metadata.input_schema;

    // Must have code parameter
    assert!(schema["properties"]["code"].is_object());
    assert_eq!(schema["required"][0], "code");

    // Must have language parameter
    assert!(schema["properties"]["language"].is_object());
    assert_eq!(schema["required"][1], "language");

    // Must have optional rules parameter
    assert!(schema["properties"]["rules"].is_object());
    assert_eq!(schema["properties"]["rules"]["type"], "array");

    // Must have optional thresholds parameter
    assert!(schema["properties"]["thresholds"].is_object());
}
