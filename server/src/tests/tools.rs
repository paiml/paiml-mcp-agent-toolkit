use crate::handlers::tools::handle_tool_call;
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
async fn test_handle_tool_call_missing_params() {
    let server = create_test_server();
    let request = create_request("tools/call", None);

    let response = handle_tool_call(server, request).await;

    assert!(response.error.is_some());
    assert_eq!(response.error.unwrap().code, -32602);
}

#[tokio::test]
async fn test_handle_tool_call_invalid_params() {
    let server = create_test_server();
    let request = create_request("tools/call", Some(json!("invalid")));

    let response = handle_tool_call(server, request).await;

    assert!(response.error.is_some());
    assert_eq!(response.error.unwrap().code, -32602);
}

#[tokio::test]
async fn test_list_templates_all() {
    let server = create_test_server();
    let request = create_request(
        "tools/call",
        Some(json!({
            "name": "list_templates",
            "arguments": {}
        })),
    );

    let response = handle_tool_call(server, request).await;

    assert!(response.result.is_some());
    assert!(response.error.is_none());

    let result = response.result.unwrap();
    let templates = result["templates"].as_array().unwrap();
    assert!(!templates.is_empty());
}

#[tokio::test]
async fn test_list_templates_by_toolchain() {
    let server = create_test_server();
    let request = create_request(
        "tools/call",
        Some(json!({
            "name": "list_templates",
            "arguments": {
                "toolchain": "rust"
            }
        })),
    );

    let response = handle_tool_call(server, request).await;

    assert!(response.result.is_some());

    let result = response.result.unwrap();
    let templates = result["templates"].as_array().unwrap();

    // All templates should be for Rust
    for template in templates {
        assert_eq!(template["toolchain"]["type"], "rust");
    }
}

#[tokio::test]
async fn test_list_templates_by_category() {
    let server = create_test_server();
    let request = create_request(
        "tools/call",
        Some(json!({
            "name": "list_templates",
            "arguments": {
                "category": "makefile"
            }
        })),
    );

    let response = handle_tool_call(server, request).await;

    assert!(response.result.is_some());

    let result = response.result.unwrap();
    let templates = result["templates"].as_array().unwrap();

    // All templates should be makefiles
    for template in templates {
        assert_eq!(template["category"], "makefile");
    }
}

#[tokio::test]
async fn test_validate_template_valid() {
    let server = create_test_server();
    let request = create_request(
        "tools/call",
        Some(json!({
            "name": "validate_template",
            "arguments": {
                "resource_uri": "template://makefile/rust/cli-binary",
                "parameters": {
                    "project_name": "test-project",
                    "has_tests": true
                }
            }
        })),
    );

    let response = handle_tool_call(server, request).await;

    assert!(response.result.is_some());
    assert!(response.error.is_none());

    let result = response.result.unwrap();
    assert_eq!(result["valid"], true);
    assert!(result["missing_required"].as_array().unwrap().is_empty());
    assert!(result["validation_errors"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_validate_template_missing_required() {
    let server = create_test_server();
    let request = create_request(
        "tools/call",
        Some(json!({
            "name": "validate_template",
            "arguments": {
                "resource_uri": "template://makefile/rust/cli-binary",
                "parameters": {
                    "has_tests": true
                    // Missing project_name
                }
            }
        })),
    );

    let response = handle_tool_call(server, request).await;

    assert!(response.result.is_some());

    let result = response.result.unwrap();
    assert_eq!(result["valid"], false);

    let missing = result["missing_required"].as_array().unwrap();
    assert!(missing.iter().any(|v| v == "project_name"));
}

#[tokio::test]
async fn test_validate_template_unknown_parameter() {
    let server = create_test_server();
    let request = create_request(
        "tools/call",
        Some(json!({
            "name": "validate_template",
            "arguments": {
                "resource_uri": "template://makefile/rust/cli-binary",
                "parameters": {
                    "project_name": "test",
                    "unknown_param": "value"
                }
            }
        })),
    );

    let response = handle_tool_call(server, request).await;

    assert!(response.result.is_some());

    let result = response.result.unwrap();
    assert_eq!(result["valid"], false);

    let errors = result["validation_errors"].as_array().unwrap();
    assert!(errors
        .iter()
        .any(|e| e.as_str().unwrap().contains("Unknown parameter")));
}

#[tokio::test]
async fn test_validate_template_not_found() {
    let server = create_test_server();
    let request = create_request(
        "tools/call",
        Some(json!({
            "name": "validate_template",
            "arguments": {
                "resource_uri": "template://invalid/template/path",
                "parameters": {}
            }
        })),
    );

    let response = handle_tool_call(server, request).await;

    assert!(response.error.is_some());
    let error = response.error.unwrap();
    assert!(error.message.contains("Template not found"));
}

#[tokio::test]
async fn test_scaffold_project_rust() {
    let server = create_test_server();
    let request = create_request(
        "tools/call",
        Some(json!({
            "name": "scaffold_project",
            "arguments": {
                "toolchain": "rust",
                "templates": ["makefile", "readme", "gitignore"],
                "parameters": {
                    "project_name": "my-rust-app",
                    "description": "A sample Rust application",
                    "has_tests": true
                }
            }
        })),
    );

    let response = handle_tool_call(server, request).await;

    assert!(response.result.is_some());
    assert!(response.error.is_none());

    let result = response.result.unwrap();
    let generated = result["generated"].as_array().unwrap();

    // Should generate 3 files
    assert_eq!(generated.len(), 3);

    // Check each generated file
    for file in generated {
        let template_type = file["template"].as_str().unwrap();
        assert!(["makefile", "readme", "gitignore"].contains(&template_type));
        assert!(!file["content"].as_str().unwrap().is_empty());
        assert!(!file["filename"].as_str().unwrap().is_empty());
    }
}

#[tokio::test]
async fn test_scaffold_project_deno() {
    let server = create_test_server();
    let request = create_request(
        "tools/call",
        Some(json!({
            "name": "scaffold_project",
            "arguments": {
                "toolchain": "deno",
                "templates": ["makefile"],
                "parameters": {
                    "project_name": "my-deno-app",
                    "permissions": "read,write,net"
                }
            }
        })),
    );

    let response = handle_tool_call(server, request).await;

    assert!(response.result.is_some());

    let result = response.result.unwrap();
    let generated = result["generated"].as_array().unwrap();

    // Should generate makefile for deno
    assert_eq!(generated.len(), 1);
    assert_eq!(generated[0]["template"], "makefile");
}

#[tokio::test]
async fn test_search_templates_by_name() {
    let server = create_test_server();
    let request = create_request(
        "tools/call",
        Some(json!({
            "name": "search_templates",
            "arguments": {
                "query": "makefile"
            }
        })),
    );

    let response = handle_tool_call(server, request).await;

    assert!(response.result.is_some());

    let result = response.result.unwrap();
    let templates = result["templates"].as_array().unwrap();

    // Should find makefile templates
    assert!(!templates.is_empty());
    for template in templates {
        assert!(
            template["name"]
                .as_str()
                .unwrap()
                .to_lowercase()
                .contains("makefile")
                || template["description"]
                    .as_str()
                    .unwrap()
                    .to_lowercase()
                    .contains("makefile")
        );
    }
}

#[tokio::test]
async fn test_search_templates_with_toolchain_filter() {
    let server = create_test_server();
    let request = create_request(
        "tools/call",
        Some(json!({
            "name": "search_templates",
            "arguments": {
                "query": "cli",
                "toolchain": "rust"
            }
        })),
    );

    let response = handle_tool_call(server, request).await;

    assert!(response.result.is_some());

    let result = response.result.unwrap();
    let templates = result["templates"].as_array().unwrap();

    // All results should be Rust templates
    for template in templates {
        assert_eq!(template["toolchain"]["type"], "rust");
    }
}

#[tokio::test]
async fn test_search_templates_no_results() {
    let server = create_test_server();
    let request = create_request(
        "tools/call",
        Some(json!({
            "name": "search_templates",
            "arguments": {
                "query": "nonexistenttemplate123"
            }
        })),
    );

    let response = handle_tool_call(server, request).await;

    assert!(response.result.is_some());

    let result = response.result.unwrap();
    let templates = result["templates"].as_array().unwrap();
    assert!(templates.is_empty());
}

#[tokio::test]
async fn test_generate_template_invalid_arguments() {
    let server = create_test_server();
    let request = create_request(
        "tools/call",
        Some(json!({
            "name": "generate_template",
            "arguments": {
                "invalid_field": "value"
            }
        })),
    );

    let response = handle_tool_call(server, request).await;

    assert!(response.error.is_some());
    let error = response.error.unwrap();
    assert!(error
        .message
        .contains("Invalid generate_template arguments"));
}
