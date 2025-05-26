use crate::handlers::prompts::{handle_prompt_get, handle_prompts_list};
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
async fn test_handle_prompts_list() {
    let server = create_test_server();
    let request = create_request("prompts/list", None);

    let response = handle_prompts_list(server, request).await;

    assert!(response.result.is_some());
    assert!(response.error.is_none());

    let result = response.result.unwrap();
    let prompts = result["prompts"].as_array().unwrap();

    // Should have 3 prompts (rust, deno, python)
    assert_eq!(prompts.len(), 3);

    // Check Rust prompt
    let rust_prompt = prompts
        .iter()
        .find(|p| p["name"] == "scaffold-rust-project")
        .expect("Rust prompt should exist");

    assert_eq!(rust_prompt["name"], "scaffold-rust-project");
    assert!(rust_prompt["description"]
        .as_str()
        .unwrap()
        .contains("Rust project"));

    let rust_args = rust_prompt["arguments"].as_array().unwrap();
    assert_eq!(rust_args.len(), 4);

    // Check required arguments
    let required_args: Vec<&str> = rust_args
        .iter()
        .filter(|arg| arg["required"] == true)
        .map(|arg| arg["name"].as_str().unwrap())
        .collect();

    assert!(required_args.contains(&"project_name"));
    assert!(required_args.contains(&"project_type"));

    // Check Deno prompt
    let deno_prompt = prompts
        .iter()
        .find(|p| p["name"] == "scaffold-deno-project")
        .expect("Deno prompt should exist");

    assert!(deno_prompt["description"]
        .as_str()
        .unwrap()
        .contains("Deno"));

    // Check Python prompt
    let python_prompt = prompts
        .iter()
        .find(|p| p["name"] == "scaffold-python-project")
        .expect("Python prompt should exist");

    assert!(python_prompt["description"]
        .as_str()
        .unwrap()
        .contains("Python"));
}

#[tokio::test]
async fn test_handle_prompt_get_rust_project() {
    let server = create_test_server();
    let request = create_request(
        "prompts/get",
        Some(json!({
            "name": "scaffold-rust-project"
        })),
    );

    let response = handle_prompt_get(server, request).await;

    assert!(response.result.is_some());
    assert!(response.error.is_none());

    let prompt = response.result.unwrap();
    assert_eq!(prompt["name"], "scaffold-rust-project");

    let arguments = prompt["arguments"].as_array().unwrap();
    assert_eq!(arguments.len(), 4);

    // Verify all expected arguments are present
    let arg_names: Vec<&str> = arguments
        .iter()
        .map(|arg| arg["name"].as_str().unwrap())
        .collect();

    assert!(arg_names.contains(&"project_name"));
    assert!(arg_names.contains(&"project_type"));
    assert!(arg_names.contains(&"has_tests"));
    assert!(arg_names.contains(&"has_benchmarks"));
}

#[tokio::test]
async fn test_handle_prompt_get_deno_project() {
    let server = create_test_server();
    let request = create_request(
        "prompts/get",
        Some(json!({
            "name": "scaffold-deno-project"
        })),
    );

    let response = handle_prompt_get(server, request).await;

    assert!(response.result.is_some());
    assert!(response.error.is_none());

    let prompt = response.result.unwrap();
    assert_eq!(prompt["name"], "scaffold-deno-project");

    let arguments = prompt["arguments"].as_array().unwrap();
    assert_eq!(arguments.len(), 3);

    // Check for permissions argument
    let permissions_arg = arguments
        .iter()
        .find(|arg| arg["name"] == "permissions")
        .expect("permissions argument should exist");

    assert_eq!(permissions_arg["required"], false);
}

#[tokio::test]
async fn test_handle_prompt_get_python_project() {
    let server = create_test_server();
    let request = create_request(
        "prompts/get",
        Some(json!({
            "name": "scaffold-python-project"
        })),
    );

    let response = handle_prompt_get(server, request).await;

    assert!(response.result.is_some());
    assert!(response.error.is_none());

    let prompt = response.result.unwrap();
    assert_eq!(prompt["name"], "scaffold-python-project");

    let arguments = prompt["arguments"].as_array().unwrap();

    // Check for python_version argument
    let version_arg = arguments
        .iter()
        .find(|arg| arg["name"] == "python_version")
        .expect("python_version argument should exist");

    assert_eq!(version_arg["required"], false);
    assert!(version_arg["description"]
        .as_str()
        .unwrap()
        .contains("3.12"));
}

#[tokio::test]
async fn test_handle_prompt_get_missing_params() {
    let server = create_test_server();
    let request = create_request("prompts/get", None);

    let response = handle_prompt_get(server, request).await;

    assert!(response.error.is_some());
    assert!(response.result.is_none());

    let error = response.error.unwrap();
    assert_eq!(error.code, -32602);
    assert!(error.message.contains("missing prompt name"));
}

#[tokio::test]
async fn test_handle_prompt_get_invalid_params() {
    let server = create_test_server();
    let request = create_request(
        "prompts/get",
        Some(json!({
            "invalid_field": "test"
        })),
    );

    let response = handle_prompt_get(server, request).await;

    assert!(response.error.is_some());
    assert!(response.result.is_none());

    let error = response.error.unwrap();
    assert_eq!(error.code, -32602);
    assert!(error.message.contains("Invalid params"));
}

#[tokio::test]
async fn test_handle_prompt_get_unknown_prompt() {
    let server = create_test_server();
    let request = create_request(
        "prompts/get",
        Some(json!({
            "name": "unknown-prompt"
        })),
    );

    let response = handle_prompt_get(server, request).await;

    assert!(response.error.is_some());
    assert!(response.result.is_none());

    let error = response.error.unwrap();
    assert_eq!(error.code, -32602);
    assert!(error.message.contains("Prompt not found: unknown-prompt"));
}
