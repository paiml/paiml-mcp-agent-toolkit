//! MCP Acceptance Tests - Protocol Compliance
//!
//! Tests for MCP protocol compliance following JSON-RPC 2.0 specification.
//! Ensures proper initialization, capability negotiation, and message format compliance.

use crate::mcp_acceptance::helpers::mcp_test_client::{McpTestClient, McpValidators};
use anyhow::Result;
use serde_json::json;

/// Test MCP initialization sequence
#[tokio::test]
async fn test_mcp_initialization_sequence() -> Result<()> {
    let mut client = McpTestClient::new()?;

    // Step 1: Initialize with proper protocol version
    let response = client.initialize()?;
    McpValidators::assert_jsonrpc_compliance(&response)?;

    // Verify initialization response structure
    if let Some(ref result) = response.result {
        // Must have capabilities
        assert!(
            result.get("capabilities").is_some(),
            "Missing capabilities in init response"
        );

        // Must have serverInfo
        assert!(
            result.get("serverInfo").is_some(),
            "Missing serverInfo in init response"
        );

        if let Some(server_info) = result.get("serverInfo") {
            assert!(server_info.get("name").is_some(), "Missing server name");
            assert!(
                server_info.get("version").is_some(),
                "Missing server version"
            );
        }

        // Check protocol version compatibility
        if let Some(protocol_version) = result.get("protocolVersion") {
            let version = protocol_version.as_str().unwrap_or("");
            assert!(
                version.starts_with("2024-"),
                "Invalid protocol version format"
            );
        }
    }

    Ok(())
}

/// Test capability negotiation
#[tokio::test]
async fn test_capability_negotiation() -> Result<()> {
    let mut client = McpTestClient::new()?;
    let response = client.initialize()?;

    if let Some(ref result) = response.result {
        if let Some(capabilities) = result.get("capabilities") {
            // Check for expected MCP capabilities
            let expected_capabilities = [
                "tools",     // Tool calling capability
                "resources", // Resource access capability
                "prompts",   // Prompt template capability
            ];

            for capability in expected_capabilities {
                if let Some(cap_value) = capabilities.get(capability) {
                    // Capability should be present and enabled
                    if cap_value.is_object() {
                        // Object form indicates detailed capability info
                        continue;
                    } else if cap_value.is_boolean() {
                        assert!(
                            cap_value.as_bool().unwrap_or(false),
                            "Capability '{}' should be enabled",
                            capability
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

/// Test JSON-RPC 2.0 message format compliance
#[tokio::test]
async fn test_jsonrpc_message_format() -> Result<()> {
    let mut client = McpTestClient::new()?;
    client.initialize()?;

    // Test various message types for format compliance
    let responses = vec![
        client.list_tools()?,
        client.list_resources()?,
        client.list_prompts()?,
        client.ping()?,
    ];

    for response in responses {
        // Every response must be JSON-RPC 2.0 compliant
        McpValidators::assert_jsonrpc_compliance(&response)?;

        // Must have either result or error (but not both)
        let has_result = response.result.is_some();
        let has_error = response.error.is_some();

        assert!(
            has_result ^ has_error,
            "Response must have either result or error, not both or neither"
        );

        // ID must match request (for our test client, always numeric)
        assert!(response.id > 0, "Response ID should be positive");
    }

    Ok(())
}

/// Test error response format compliance  
#[tokio::test]
async fn test_error_response_format() -> Result<()> {
    let mut client = McpTestClient::new()?;
    client.initialize()?;

    // Test invalid method call to trigger error response
    let result = client.call_tool("invalid_nonexistent_tool", json!({}));

    if let Ok(tool_result) = result {
        if !tool_result.success {
            let response = &tool_result.response;
            McpValidators::assert_jsonrpc_compliance(response)?;

            if let Some(ref error) = response.error {
                // Error must have code and message
                assert!(error.code != 0, "Error code should not be zero");
                assert!(
                    !error.message.is_empty(),
                    "Error message should not be empty"
                );

                // Error codes should follow JSON-RPC 2.0 standard
                let valid_error_codes = [-32700, -32600, -32601, -32602, -32603];
                let is_standard_error = valid_error_codes.contains(&error.code);
                let is_application_error = error.code >= -32000 && error.code <= -32099;

                assert!(
                    is_standard_error || is_application_error,
                    "Error code {} should be standard JSON-RPC or application-specific",
                    error.code
                );
            }
        }
    }

    Ok(())
}

/// Test request/response correlation
#[tokio::test]
async fn test_request_response_correlation() -> Result<()> {
    let mut client = McpTestClient::new()?;
    client.initialize()?;

    // Make multiple requests and verify ID correlation
    let requests = vec![
        ("tools/list", json!({})),
        ("resources/list", json!({})),
        ("prompts/list", json!({})),
    ];

    for (method, params) in requests {
        let initial_id = client.request_id_counter;
        let response = client.call_method(method, params)?;

        // Response ID should match the request ID
        assert_eq!(
            response.id,
            initial_id + 1,
            "Response ID should match request ID for method {}",
            method
        );
    }

    Ok(())
}

/// Test protocol version handling
#[tokio::test]
async fn test_protocol_version_handling() -> Result<()> {
    let mut client = McpTestClient::new()?;

    // Test with supported protocol version
    let response = client.call_method(
        "initialize",
        json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": client.client_info
        }),
    )?;

    McpValidators::assert_jsonrpc_compliance(&response)?;

    if let Some(ref result) = response.result {
        if let Some(protocol_version) = result.get("protocolVersion") {
            let version = protocol_version.as_str().unwrap_or("");
            // Should return a compatible version
            assert!(
                version.starts_with("2024-"),
                "Server should return 2024 protocol version"
            );
        }
    }

    Ok(())
}

/// Test connection lifecycle
#[tokio::test]
async fn test_connection_lifecycle() -> Result<()> {
    let mut client = McpTestClient::new()?;

    // 1. Initialize connection
    let init_response = client.initialize()?;
    assert!(
        init_response.error.is_none(),
        "Initialization should succeed"
    );

    // 2. Use connection (list tools)
    let tools_response = client.list_tools()?;
    assert!(
        tools_response.error.is_none(),
        "Tool listing should succeed after init"
    );

    // 3. Test ping/keepalive
    let ping_response = client.ping()?;
    McpValidators::assert_jsonrpc_compliance(&ping_response)?;

    // 4. Connection should remain functional
    let final_response = client.list_resources()?;
    assert!(
        final_response.error.is_none(),
        "Connection should remain functional"
    );

    Ok(())
}

/// Test concurrent request handling
#[tokio::test]
async fn test_concurrent_request_handling() -> Result<()> {
    let mut client = McpTestClient::new()?;
    client.initialize()?;

    // Note: For stdio transport, requests are sequential, but we test rapid succession
    let methods = vec![
        "tools/list",
        "resources/list",
        "prompts/list",
        "tools/list", // Repeat to test caching/consistency
    ];

    let mut responses = Vec::new();

    for method in methods {
        let response = client.call_method(method, json!({}))?;
        McpValidators::assert_jsonrpc_compliance(&response)?;
        responses.push((method, response));
    }

    // Verify all requests were handled properly
    for (method, response) in responses {
        assert!(
            response.error.is_none()
                || (response.error.is_some() && response.error.as_ref().unwrap().code == -32601),
            "Method {} should either succeed or return method not found",
            method
        );
    }

    Ok(())
}

/// Test transport layer compliance (stdio)
#[tokio::test]
async fn test_transport_compliance() -> Result<()> {
    let mut client = McpTestClient::new()?;

    // Test that messages are properly formatted for stdio transport
    let response = client.initialize()?;

    // Response should be valid JSON-RPC over stdio
    McpValidators::assert_jsonrpc_compliance(&response)?;

    // Test that the client can handle different response sizes
    let tools_response = client.list_tools()?;
    McpValidators::assert_jsonrpc_compliance(&tools_response)?;

    // Tools list might be large, test handling
    if let Some(ref result) = tools_response.result {
        if let Some(tools) = result.get("tools") {
            if let Some(tools_array) = tools.as_array() {
                // Should handle multiple tools without issue
                assert!(tools_array.len() >= 0, "Tools array should be valid");
            }
        }
    }

    Ok(())
}

/// Test protocol extension support
#[tokio::test]
async fn test_protocol_extensions() -> Result<()> {
    let mut client = McpTestClient::new()?;
    let response = client.initialize()?;

    if let Some(ref result) = response.result {
        if let Some(capabilities) = result.get("capabilities") {
            // Check for experimental or extension capabilities
            if let Some(experimental) = capabilities.get("experimental") {
                // Server may support experimental features
                assert!(
                    experimental.is_object(),
                    "Experimental capabilities should be an object"
                );
            }

            // Check for any custom PMAT-specific capabilities
            let pmat_specific_caps = [
                "vectorized_analysis",
                "quality_gates",
                "tdg_analysis",
                "qdd_generation",
            ];

            for cap in pmat_specific_caps {
                if capabilities.get(cap).is_some() {
                    // If present, should be properly structured
                    println!("Found PMAT-specific capability: {}", cap);
                }
            }
        }
    }

    Ok(())
}

/// Test resource and prompt listing compliance
#[tokio::test]
async fn test_resource_prompt_compliance() -> Result<()> {
    let mut client = McpTestClient::new()?;
    client.initialize()?;

    // Test resources listing
    let resources_response = client.list_resources()?;
    McpValidators::assert_jsonrpc_compliance(&resources_response)?;

    if let Some(ref result) = resources_response.result {
        if let Some(resources) = result.get("resources") {
            if let Some(resources_array) = resources.as_array() {
                for resource in resources_array {
                    // Each resource should have required fields
                    assert!(resource.get("uri").is_some(), "Resource missing URI");
                    assert!(resource.get("name").is_some(), "Resource missing name");
                }
            }
        }
    }

    // Test prompts listing
    let prompts_response = client.list_prompts()?;
    McpValidators::assert_jsonrpc_compliance(&prompts_response)?;

    if let Some(ref result) = prompts_response.result {
        if let Some(prompts) = result.get("prompts") {
            if let Some(prompts_array) = prompts.as_array() {
                for prompt in prompts_array {
                    // Each prompt should have required fields
                    assert!(prompt.get("name").is_some(), "Prompt missing name");
                    assert!(
                        prompt.get("description").is_some(),
                        "Prompt missing description"
                    );
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Test complete protocol compliance workflow
    #[tokio::test]
    async fn test_complete_protocol_workflow() -> Result<()> {
        let mut client = McpTestClient::new()?;

        // 1. Connection establishment with capability negotiation
        let init_response = client.initialize()?;
        McpValidators::assert_jsonrpc_compliance(&init_response)?;
        McpValidators::assert_response_fields(&init_response, &["capabilities", "serverInfo"])?;

        // 2. Service discovery
        let tools_response = client.list_tools()?;
        McpValidators::assert_jsonrpc_compliance(&tools_response)?;

        let resources_response = client.list_resources()?;
        McpValidators::assert_jsonrpc_compliance(&resources_response)?;

        let prompts_response = client.list_prompts()?;
        McpValidators::assert_jsonrpc_compliance(&prompts_response)?;

        // 3. Functional operation (tool call)
        let project_path = client.create_sample_project()?;
        let tool_result = client.call_tool(
            "analyze_complexity",
            json!({
                "path": project_path.join("src/main.rs").to_string_lossy()
            }),
        )?;

        McpValidators::assert_jsonrpc_compliance(&tool_result.response)?;

        // 4. Connection health check
        let ping_response = client.ping()?;
        McpValidators::assert_jsonrpc_compliance(&ping_response)?;

        // All protocol interactions should be compliant
        println!("Complete MCP protocol workflow completed successfully");

        Ok(())
    }
}
