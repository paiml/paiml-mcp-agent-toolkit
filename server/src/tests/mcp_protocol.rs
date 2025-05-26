use crate::handlers;
use crate::models::mcp::McpRequest;
use crate::stateless_server::StatelessTemplateServer;
use serde_json::{json, Value};
use std::sync::Arc;

fn create_test_server() -> Arc<StatelessTemplateServer> {
    Arc::new(StatelessTemplateServer::new().unwrap())
}

fn create_request(method: &str, params: Option<Value>) -> McpRequest {
    McpRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(1),
        method: method.to_string(),
        params,
    }
}

#[tokio::test]
async fn test_handle_initialize() {
    let server = create_test_server();

    let request = create_request(
        "initialize",
        Some(json!({
            "protocolVersion": "1.0",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        })),
    );

    let response = handlers::handle_request(server, request).await;

    assert!(response.result.is_some());
    assert!(response.error.is_none());

    let result = response.result.unwrap();
    assert_eq!(result["protocolVersion"], "1.0");
    assert_eq!(result["serverInfo"]["name"], "paiml-mcp-agent-toolkit");
    assert_eq!(
        result["serverInfo"]["vendor"],
        "Pragmatic AI Labs (paiml.com)"
    );

    // Check capabilities
    assert!(result["capabilities"].is_object());
    assert!(result["capabilities"]["resources"].is_object());
    assert!(result["capabilities"]["tools"].is_object());
}

#[tokio::test]
async fn test_handle_list_tools() {
    let server = create_test_server();

    let request = create_request("tools/list", None);
    let response = handlers::handle_request(server, request).await;

    assert!(response.result.is_some());
    assert!(response.error.is_none());

    let result = response.result.unwrap();
    let tools = result["tools"].as_array().unwrap();

    // Should have at least one tool
    assert!(!tools.is_empty());

    // Check for generate_template tool
    let generate_tool = tools
        .iter()
        .find(|t| t["name"] == "generate_template")
        .expect("generate_template tool should exist");

    assert!(!generate_tool["description"].as_str().unwrap().is_empty());
    assert_eq!(generate_tool["inputSchema"]["type"], "object");

    // Check required properties
    let required = generate_tool["inputSchema"]["required"].as_array().unwrap();
    assert!(required.contains(&json!("resource_uri")));
    assert!(required.contains(&json!("parameters")));
}

#[tokio::test]
async fn test_handle_list_resources() {
    let server = create_test_server();

    let request = create_request("resources/list", None);
    let response = handlers::handle_request(server, request).await;

    assert!(response.result.is_some());
    assert!(response.error.is_none());

    let result = response.result.unwrap();
    let resources = result["resources"].as_array().unwrap();

    // Should have multiple template resources
    assert!(!resources.is_empty());

    // Check for Rust templates
    let rust_templates: Vec<&Value> = resources
        .iter()
        .filter(|r| r["uri"].as_str().unwrap().contains("/rust/"))
        .collect();

    assert!(!rust_templates.is_empty(), "Should have Rust templates");

    // Verify resource structure
    for resource in resources {
        let uri = resource["uri"].as_str().unwrap();
        assert!(uri.starts_with("template://"));
        assert!(!resource["name"].as_str().unwrap().is_empty());
        assert!(!resource["description"].as_str().unwrap().is_empty());
        assert_eq!(resource["mimeType"], "text/x-handlebars-template");
    }
}

#[tokio::test]
async fn test_handle_call_tool_generate_template() {
    let server = create_test_server();

    let request = create_request(
        "tools/call",
        Some(json!({
            "name": "generate_template",
            "arguments": {
                "resource_uri": "template://makefile/rust/cli-binary",
                "parameters": {
                    "project_name": "test-project",
                    "has_tests": true,
                    "target": "x86_64-unknown-linux-gnu"
                }
            }
        })),
    );

    let response = handlers::handle_request(server, request).await;

    assert!(response.result.is_some());
    assert!(response.error.is_none());

    let result = response.result.unwrap();

    // Check content
    let content = result["content"].as_array().unwrap();
    assert_eq!(content.len(), 1);

    let text_content = &content[0];
    assert_eq!(text_content["type"], "text");

    let generated = text_content["text"].as_str().unwrap();
    assert!(generated.contains("test-project"));
    assert!(generated.contains("Pragmatic AI Labs MCP Agent Toolkit"));
    assert!(generated.contains("cargo check"));
}

#[tokio::test]
async fn test_handle_call_tool_invalid_tool() {
    let server = create_test_server();

    let request = create_request(
        "tools/call",
        Some(json!({
            "name": "invalid_tool",
            "arguments": {}
        })),
    );

    let response = handlers::handle_request(server, request).await;

    assert!(response.error.is_some());
    assert!(response.result.is_none());

    let error = response.error.unwrap();
    assert!(error.message.contains("Unknown tool"));
}

#[tokio::test]
async fn test_handle_call_tool_missing_parameters() {
    let server = create_test_server();

    let request = create_request(
        "tools/call",
        Some(json!({
            "name": "generate_template",
            "arguments": {
                "resource_uri": "template://makefile/rust/cli-binary"
                // Missing parameters
            }
        })),
    );

    let response = handlers::handle_request(server, request).await;

    assert!(response.error.is_some());

    let error = response.error.unwrap();
    assert!(error.message.contains("Missing required field: parameters"));
}

#[tokio::test]
async fn test_handle_invalid_method() {
    let server = create_test_server();

    let request = create_request("invalid/method", None);
    let response = handlers::handle_request(server, request).await;

    assert!(response.error.is_some());
    assert_eq!(response.error.unwrap().code, -32601);
}

#[tokio::test]
async fn test_protocol_version_default() {
    let server = create_test_server();

    let request = create_request("initialize", Some(json!({})));
    let response = handlers::handle_request(server, request).await;

    let result = response.result.unwrap();
    assert_eq!(result["protocolVersion"], "2024-11-05");
}
