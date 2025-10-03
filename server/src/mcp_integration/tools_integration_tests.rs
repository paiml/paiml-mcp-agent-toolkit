//! Integration tests for MCP tools with actix actors
//!
//! RED phase tests for tool-to-agent communication

use super::tools::*;
use super::*;
use crate::agents::analyzer_actor::AnalyzerActor;
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
