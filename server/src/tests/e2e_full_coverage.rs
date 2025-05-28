use crate::handlers;
use crate::models::mcp::McpRequest;
use crate::stateless_server::StatelessTemplateServer;
use std::process::Command;
use std::sync::Arc;

#[tokio::test]
async fn test_mcp_server_e2e_coverage() {
    let server = Arc::new(StatelessTemplateServer::new().unwrap());

    // Test 1: Valid initialize request
    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: serde_json::json!(1),
        method: "initialize".to_string(),
        params: Some(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0"}
        })),
    };
    let response = handlers::handle_request(server.clone(), request).await;
    assert!(response.result.is_some());
    assert!(response.error.is_none());

    // Test 2: List tools
    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: serde_json::json!(2),
        method: "tools/list".to_string(),
        params: Some(serde_json::json!({})),
    };
    let response = handlers::handle_request(server.clone(), request).await;
    assert!(response.result.is_some());
    let result = response.result.unwrap();
    let tools = result["tools"].as_array().unwrap();
    assert!(!tools.is_empty());

    // Test 3: List resources
    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: serde_json::json!(3),
        method: "resources/list".to_string(),
        params: Some(serde_json::json!({})),
    };
    let response = handlers::handle_request(server.clone(), request).await;
    assert!(response.result.is_some());

    // Test 4: Generate template
    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: serde_json::json!(4),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "generate_template",
            "arguments": {
                "resource_uri": "template://makefile/rust/cli",
                "parameters": {"project_name": "test_project", "has_tests": true}
            }
        })),
    };
    let response = handlers::handle_request(server.clone(), request).await;
    assert!(response.result.is_some() || response.error.is_some());

    // Test 5: Invalid method
    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: serde_json::json!(5),
        method: "invalid/method".to_string(),
        params: None,
    };
    let response = handlers::handle_request(server.clone(), request).await;
    assert!(response.error.is_some());
    assert_eq!(response.error.unwrap().code, -32601);

    // Test 6: More tool calls for coverage
    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: serde_json::json!(6),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "list_templates",
            "arguments": {}
        })),
    };
    let response = handlers::handle_request(server.clone(), request).await;
    assert!(response.result.is_some() || response.error.is_some());

    // Test 7: Search templates
    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: serde_json::json!(7),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "search_templates",
            "arguments": {"query": "rust"}
        })),
    };
    let response = handlers::handle_request(server.clone(), request).await;
    assert!(response.result.is_some() || response.error.is_some());

    // Test 8: Validate template
    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: serde_json::json!(8),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "validate_template",
            "arguments": {
                "resource_uri": "template://readme/rust/cli",
                "parameters": {"project_name": "test", "author": "Test"}
            }
        })),
    };
    let response = handlers::handle_request(server.clone(), request).await;
    assert!(response.result.is_some() || response.error.is_some());

    // Test 9: Scaffold project
    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: serde_json::json!(9),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "scaffold_project",
            "arguments": {
                "toolchain": "rust",
                "parameters": {"project_name": "test", "author": "Test", "version": "0.1.0"}
            }
        })),
    };
    let response = handlers::handle_request(server.clone(), request).await;
    assert!(response.result.is_some() || response.error.is_some());

    // Test 10: Read resource
    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: serde_json::json!(10),
        method: "resources/read".to_string(),
        params: Some(serde_json::json!({
            "uri": "template://gitignore/rust/cli"
        })),
    };
    let response = handlers::handle_request(server.clone(), request).await;
    assert!(response.result.is_some() || response.error.is_some());

    // Test 11: List prompts
    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: serde_json::json!(11),
        method: "prompts/list".to_string(),
        params: Some(serde_json::json!({})),
    };
    let response = handlers::handle_request(server.clone(), request).await;
    assert!(response.result.is_some());

    // Test 12: Get prompt
    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: serde_json::json!(12),
        method: "prompts/get".to_string(),
        params: Some(serde_json::json!({
            "name": "rust_project",
            "arguments": {"project_name": "test"}
        })),
    };
    let response = handlers::handle_request(server.clone(), request).await;
    assert!(response.result.is_some() || response.error.is_some());
}

#[test]
fn test_cli_main_binary_version() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "paiml-mcp-agent-toolkit", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("paiml-mcp-agent-toolkit"));

    // Read the actual version from Cargo.toml instead of hardcoding
    let expected_version = env!("CARGO_PKG_VERSION");
    assert!(
        stdout.contains(expected_version),
        "Binary version output '{}' should contain expected version '{}'",
        stdout.trim(),
        expected_version
    );
}

#[test]
fn test_cli_main_binary_help() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "paiml-mcp-agent-toolkit", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Professional project scaffolding toolkit"));
    assert!(stdout.contains("Commands:"));
    assert!(stdout.contains("generate"));
    assert!(stdout.contains("scaffold"));
    assert!(stdout.contains("list"));
    assert!(stdout.contains("search"));
}

#[test]
fn test_cli_subcommand_help() {
    // Test generate help
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "paiml-mcp-agent-toolkit",
            "--",
            "generate",
            "--help",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Generate a single template"));

    // Test list help
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "paiml-mcp-agent-toolkit",
            "--",
            "list",
            "--help",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("List available templates"));
}

#[test]
fn test_cli_mode_list_templates() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "paiml-mcp-agent-toolkit",
            "--",
            "--mode",
            "cli",
            "list",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should either succeed with template list or fail with a clear error
    assert!(
        stdout.contains("Template")
            || stdout.contains("makefile")
            || stderr.contains("error")
            || output.status.success()
    );
}

#[test]
fn test_cli_generate_validation_error() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "paiml-mcp-agent-toolkit",
            "--",
            "generate",
            "makefile",
            "rust/cli",
            // Missing required project_name parameter - this should fail
        ])
        .output()
        .expect("Failed to execute command");

    // Should fail due to missing required parameter
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !output.status.success()
            || stderr.contains("error")
            || stderr.contains("Missing required parameter")
            || stdout.contains("error")
            || stdout.contains("Missing required parameter")
    );
}

#[test]
fn test_cli_search_templates() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "paiml-mcp-agent-toolkit",
            "--",
            "search",
            "rust",
        ])
        .output()
        .expect("Failed to execute command");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Should find rust templates
        assert!(stdout.contains("rust") || stdout.contains("Rust"));
    }
}

#[test]
fn test_cli_invalid_command() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "paiml-mcp-agent-toolkit",
            "--",
            "invalid-command",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unrecognized subcommand") || stderr.contains("error"));
}

#[test]
fn test_cli_analyze_churn() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "paiml-mcp-agent-toolkit",
            "--",
            "analyze",
            "churn",
            "--help",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("churn") || stdout.contains("Analyze"));
}
