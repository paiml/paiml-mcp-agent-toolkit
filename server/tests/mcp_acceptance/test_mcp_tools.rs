//! MCP Acceptance Tests - Tools
//!
//! Tests for all MCP tools following the mcp-acceptance-testing.md specification.
//! Ensures 100% coverage of MCP tool functionality with proper JSON-RPC 2.0 compliance.

use crate::mcp_acceptance::helpers::mcp_test_client::{McpTestClient, McpValidators};
use anyhow::Result;
use serde_json::json;
use std::time::Duration;

/// Test MCP server initialization and capabilities
#[tokio::test]
async fn test_mcp_initialization() -> Result<()> {
    let mut client = McpTestClient::new()?;

    // Test initialization
    let response = client.initialize()?;
    McpValidators::assert_jsonrpc_compliance(&response)?;

    // Should have server capabilities
    if let Some(result) = response.result {
        assert!(result.get("capabilities").is_some());
        assert!(result.get("serverInfo").is_some());
    }

    Ok(())
}

/// Test template management tools
#[tokio::test]
async fn test_template_management_tools() -> Result<()> {
    let mut client = McpTestClient::new()?;
    client.initialize()?;

    // Test pmat_generate_template tool
    let result = client.call_tool(
        "pmat_generate_template",
        json!({
            "template_name": "rust_basic",
            "output_path": "./test_output",
            "variables": {
                "project_name": "test_project"
            }
        }),
    )?;

    McpValidators::assert_tool_success(&result)?;
    McpValidators::assert_performance(&result, Duration::from_secs(10))?;

    // Test pmat_list_templates tool
    let result = client.call_tool("pmat_list_templates", json!({}))?;
    McpValidators::assert_tool_success(&result)?;
    McpValidators::assert_performance(&result, Duration::from_secs(5))?;

    if let Some(ref result_data) = result.response.result {
        assert!(result_data.get("templates").is_some());
    }

    // Test pmat_validate_template tool
    let result = client.call_tool(
        "pmat_validate_template",
        json!({
            "template_path": "./templates/rust_basic.toml"
        }),
    )?;

    McpValidators::assert_tool_success(&result)?;
    McpValidators::assert_performance(&result, Duration::from_secs(5))?;

    Ok(())
}

/// Test analysis tools
#[tokio::test]
async fn test_analysis_tools() -> Result<()> {
    let mut client = McpTestClient::new()?;
    let project_path = client.create_sample_project()?;
    client.initialize()?;

    // Test analyze_complexity tool
    let result = client.call_tool(
        "analyze_complexity",
        json!({
            "path": project_path.join("src/main.rs").to_string_lossy(),
            "format": "json"
        }),
    )?;

    McpValidators::assert_tool_success(&result)?;
    McpValidators::assert_performance(&result, Duration::from_secs(15))?;

    // Verify complexity analysis result structure
    if let Some(ref result_data) = result.response.result {
        assert!(result_data.get("files").is_some() || result_data.get("analysis").is_some());
    }

    // Test analyze_dead_code tool
    let result = client.call_tool(
        "analyze_dead_code",
        json!({
            "path": project_path.to_string_lossy(),
            "format": "json"
        }),
    )?;

    McpValidators::assert_tool_success(&result)?;
    McpValidators::assert_performance(&result, Duration::from_secs(30))?;

    // Test analyze_satd tool
    let result = client.call_tool(
        "analyze_satd",
        json!({
            "path": project_path.to_string_lossy()
        }),
    )?;

    McpValidators::assert_tool_success(&result)?;
    McpValidators::assert_performance(&result, Duration::from_secs(15))?;

    // Test analyze_entropy tool
    let result = client.call_tool(
        "analyze_entropy",
        json!({
            "path": project_path.to_string_lossy(),
            "min_severity": "medium",
            "top_violations": 10
        }),
    )?;

    McpValidators::assert_tool_success(&result)?;
    McpValidators::assert_performance(&result, Duration::from_secs(20))?;

    Ok(())
}

/// Test quality assurance tools
#[tokio::test]
async fn test_quality_assurance_tools() -> Result<()> {
    let mut client = McpTestClient::new()?;
    let project_path = client.create_sample_project()?;
    client.initialize()?;

    // Test quality_gate tool
    let result = client.call_tool(
        "quality_gate",
        json!({
            "file_path": project_path.join("src/main.rs").to_string_lossy(),
            "profile": "standard"
        }),
    )?;

    McpValidators::assert_tool_success(&result)?;
    McpValidators::assert_performance(&result, Duration::from_secs(20))?;

    // Verify quality gate result structure
    if let Some(ref result_data) = result.response.result {
        assert!(
            result_data.get("passed").is_some()
                || result_data.get("status").is_some()
                || result_data.get("result").is_some()
        );
    }

    // Test tdg_analyze tool
    let result = client.call_tool(
        "tdg_analyze",
        json!({
            "path": project_path.join("src/main.rs").to_string_lossy(),
            "include_components": true
        }),
    )?;

    McpValidators::assert_tool_success(&result)?;
    McpValidators::assert_performance(&result, Duration::from_secs(15))?;

    // Test qdd_create tool
    let result = client.call_tool(
        "qdd_create",
        json!({
            "type": "function",
            "name": "test_function",
            "purpose": "Test function creation",
            "profile": "standard"
        }),
    )?;

    McpValidators::assert_tool_success(&result)?;
    McpValidators::assert_performance(&result, Duration::from_secs(10))?;

    Ok(())
}

/// Test refactoring tools
#[tokio::test]
async fn test_refactoring_tools() -> Result<()> {
    let mut client = McpTestClient::new()?;
    let project_path = client.create_sample_project()?;
    client.initialize()?;

    // Test refactor_start tool
    let result = client.call_tool(
        "refactor_start",
        json!({
            "file_path": project_path.join("src/main.rs").to_string_lossy()
        }),
    )?;

    McpValidators::assert_tool_success(&result)?;
    McpValidators::assert_performance(&result, Duration::from_secs(30))?;

    // Verify refactor result structure
    if let Some(ref result_data) = result.response.result {
        assert!(
            result_data.get("suggestions").is_some()
                || result_data.get("plan").is_some()
                || result_data.get("refactoring_plan").is_some()
        );
    }

    Ok(())
}

/// Test reporting tools
#[tokio::test]
async fn test_reporting_tools() -> Result<()> {
    let mut client = McpTestClient::new()?;
    let project_path = client.create_sample_project()?;
    client.initialize()?;

    // Test generate_report tool
    let result = client.call_tool(
        "generate_report",
        json!({
            "path": project_path.to_string_lossy(),
            "format": "json",
            "include_metrics": true
        }),
    )?;

    McpValidators::assert_tool_success(&result)?;
    McpValidators::assert_performance(&result, Duration::from_secs(30))?;

    // Verify report structure
    if let Some(ref result_data) = result.response.result {
        assert!(
            result_data.get("report").is_some()
                || result_data.get("summary").is_some()
                || result_data.get("analysis").is_some()
        );
    }

    Ok(())
}

/// Test context management tools
#[tokio::test]
async fn test_context_management_tools() -> Result<()> {
    let mut client = McpTestClient::new()?;
    let project_path = client.create_sample_project()?;
    client.initialize()?;

    // Test context_create tool
    let result = client.call_tool(
        "context_create",
        json!({
            "files": [project_path.join("src/main.rs").to_string_lossy()],
            "include_dependencies": true
        }),
    )?;

    McpValidators::assert_tool_success(&result)?;
    McpValidators::assert_performance(&result, Duration::from_secs(20))?;

    // Test deep_context_analysis tool
    let result = client.call_tool(
        "deep_context_analysis",
        json!({
            "file_path": project_path.join("src/main.rs").to_string_lossy()
        }),
    )?;

    McpValidators::assert_tool_success(&result)?;
    McpValidators::assert_performance(&result, Duration::from_secs(30))?;

    Ok(())
}

/// Test vectorized tools
#[tokio::test]
async fn test_vectorized_tools() -> Result<()> {
    let mut client = McpTestClient::new()?;
    let project_path = client.create_sample_project()?;
    client.initialize()?;

    // Test vectorized_complexity_analysis tool
    let result = client.call_tool(
        "vectorized_complexity_analysis",
        json!({
            "paths": [
                project_path.join("src/main.rs").to_string_lossy(),
                project_path.join("src/lib.rs").to_string_lossy()
            ],
            "batch_size": 10
        }),
    )?;

    McpValidators::assert_tool_success(&result)?;
    McpValidators::assert_performance(&result, Duration::from_secs(45))?;

    // Test vectorized_quality_analysis tool
    let result = client.call_tool(
        "vectorized_quality_analysis",
        json!({
            "directory": project_path.to_string_lossy(),
            "analysis_types": ["complexity", "satd", "dead_code"],
            "parallel": true
        }),
    )?;

    McpValidators::assert_tool_success(&result)?;
    McpValidators::assert_performance(&result, Duration::from_secs(60))?;

    Ok(())
}

/// Test PDMT todo generation tools
#[tokio::test]
async fn test_pdmt_tools() -> Result<()> {
    let mut client = McpTestClient::new()?;
    client.initialize()?;

    // Test pdmt_deterministic_todos tool
    let result = client.call_tool(
        "pdmt_deterministic_todos",
        json!({
            "requirement": "Implement user authentication system",
            "granularity": "medium",
            "seed": 42
        }),
    )?;

    McpValidators::assert_tool_success(&result)?;
    McpValidators::assert_performance(&result, Duration::from_secs(10))?;

    // Verify PDMT result structure
    if let Some(ref result_data) = result.response.result {
        assert!(
            result_data.get("todos").is_some()
                || result_data.get("tasks").is_some()
                || result_data.get("breakdown").is_some()
        );
    }

    Ok(())
}

/// Test tool error handling
#[tokio::test]
async fn test_tool_error_handling() -> Result<()> {
    let mut client = McpTestClient::new()?;
    client.initialize()?;

    // Test invalid tool call
    let result = client.call_tool("nonexistent_tool", json!({}));
    assert!(result.is_ok()); // Should handle gracefully
    let result = result?;
    assert!(!result.success); // But should indicate failure

    // Test invalid parameters
    let result = client.call_tool(
        "analyze_complexity",
        json!({
            "invalid_param": "invalid_value"
        }),
    )?;

    // Should handle invalid parameters gracefully
    McpValidators::assert_performance(&result, Duration::from_secs(5))?;

    // Test missing required parameters
    let result = client.call_tool("quality_gate", json!({}))?;
    McpValidators::assert_performance(&result, Duration::from_secs(5))?;

    Ok(())
}

/// Test tool list functionality
#[tokio::test]
async fn test_tool_listing() -> Result<()> {
    let mut client = McpTestClient::new()?;
    client.initialize()?;

    // Test tools list
    let response = client.list_tools()?;
    McpValidators::assert_jsonrpc_compliance(&response)?;

    if let Some(ref result) = response.result {
        if let Some(tools) = result.get("tools") {
            let tools_array = tools.as_array().expect("Tools should be an array");

            // Verify we have the expected tools
            let expected_tools = vec![
                "pmat_generate_template",
                "pmat_list_templates",
                "pmat_validate_template",
                "analyze_complexity",
                "analyze_dead_code",
                "analyze_satd",
                "analyze_entropy",
                "quality_gate",
                "tdg_analyze",
                "qdd_create",
                "refactor_start",
                "generate_report",
                "context_create",
                "deep_context_analysis",
                "vectorized_complexity_analysis",
                "vectorized_quality_analysis",
                "pdmt_deterministic_todos",
            ];

            for expected_tool in expected_tools {
                let tool_found = tools_array.iter().any(|tool| {
                    tool.get("name")
                        .and_then(|n| n.as_str())
                        .map(|name| name == expected_tool)
                        .unwrap_or(false)
                });
                assert!(
                    tool_found,
                    "Expected tool '{}' not found in tools list",
                    expected_tool
                );
            }
        }
    }

    Ok(())
}

/// Test concurrent tool calls
#[tokio::test]
async fn test_concurrent_tool_calls() -> Result<()> {
    let mut client = McpTestClient::new()?;
    let project_path = client.create_sample_project()?;
    client.initialize()?;

    // Simulate concurrent calls (in sequence for stdio)
    let tools_to_test = vec![
        (
            "analyze_complexity",
            json!({
                "path": project_path.join("src/main.rs").to_string_lossy(),
                "format": "json"
            }),
        ),
        (
            "analyze_satd",
            json!({
                "path": project_path.to_string_lossy()
            }),
        ),
        (
            "tdg_analyze",
            json!({
                "path": project_path.join("src/main.rs").to_string_lossy()
            }),
        ),
    ];

    for (tool_name, params) in tools_to_test {
        let result = client.call_tool(tool_name, params)?;
        McpValidators::assert_tool_success(&result)?;
        McpValidators::assert_performance(&result, Duration::from_secs(30))?;
    }

    Ok(())
}

/// Test MCP protocol compliance
#[tokio::test]
async fn test_protocol_compliance() -> Result<()> {
    let mut client = McpTestClient::new()?;

    // Test initialization protocol
    let response = client.initialize()?;
    McpValidators::assert_jsonrpc_compliance(&response)?;

    // Verify required initialization response fields
    McpValidators::assert_response_fields(&response, &["capabilities", "serverInfo"])?;

    // Test ping protocol
    let response = client.ping()?;
    McpValidators::assert_jsonrpc_compliance(&response)?;

    // Test that all responses follow JSON-RPC 2.0
    let tools_response = client.list_tools()?;
    McpValidators::assert_jsonrpc_compliance(&tools_response)?;

    let resources_response = client.list_resources()?;
    McpValidators::assert_jsonrpc_compliance(&resources_response)?;

    let prompts_response = client.list_prompts()?;
    McpValidators::assert_jsonrpc_compliance(&prompts_response)?;

    Ok(())
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Test full MCP workflow
    #[tokio::test]
    async fn test_full_mcp_workflow() -> Result<()> {
        let mut client = McpTestClient::new()?;
        let project_path = client.create_sample_project()?;

        // Initialize connection
        let init_response = client.initialize()?;
        assert!(init_response.error.is_none());

        // List available tools
        let tools_response = client.list_tools()?;
        assert!(tools_response.error.is_none());

        // Run analysis workflow
        let complexity_result = client.call_tool(
            "analyze_complexity",
            json!({
                "path": project_path.join("src/main.rs").to_string_lossy()
            }),
        )?;
        assert!(complexity_result.success);

        let quality_result = client.call_tool(
            "quality_gate",
            json!({
                "file_path": project_path.join("src/main.rs").to_string_lossy()
            }),
        )?;
        assert!(quality_result.success);

        let report_result = client.call_tool(
            "generate_report",
            json!({
                "path": project_path.to_string_lossy(),
                "format": "json"
            }),
        )?;
        assert!(report_result.success);

        Ok(())
    }
}
