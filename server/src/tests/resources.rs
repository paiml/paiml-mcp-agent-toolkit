use crate::handlers::resources::{handle_resource_list, handle_resource_read};
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
async fn test_handle_resource_list() {
    let server = create_test_server();
    let request = create_request("resources/list", None);

    let response = handle_resource_list(server, request).await;

    assert!(response.result.is_some());
    assert!(response.error.is_none());

    let result = response.result.unwrap();
    let resources = result["resources"].as_array().unwrap();

    // Should have 9 embedded templates (3 file types × 3 languages)
    assert_eq!(resources.len(), 9);

    // Check resource structure
    for resource in resources {
        assert!(resource["uri"].as_str().unwrap().starts_with("template://"));
        assert!(!resource["name"].as_str().unwrap().is_empty());
        assert!(!resource["description"].as_str().unwrap().is_empty());
        assert_eq!(resource["mimeType"], "text/x-handlebars-template");
    }
}

#[tokio::test]
async fn test_handle_resource_read_success() {
    let server = create_test_server();
    let request = create_request(
        "resources/read",
        Some(json!({
            "uri": "template://makefile/rust/cli"
        })),
    );

    let response = handle_resource_read(server, request).await;

    assert!(response.result.is_some());
    assert!(response.error.is_none());

    let result = response.result.unwrap();
    let contents = result["contents"].as_array().unwrap();

    assert_eq!(contents.len(), 1);

    let content = &contents[0];
    assert_eq!(content["uri"], "template://makefile/rust/cli");
    assert_eq!(content["mimeType"], "text/x-handlebars-template");
    assert!(content["text"]
        .as_str()
        .unwrap()
        .contains("{{project_name}}"));
}

#[tokio::test]
async fn test_handle_resource_read_missing_params() {
    let server = create_test_server();
    let request = create_request("resources/read", None);

    let response = handle_resource_read(server, request).await;

    assert!(response.error.is_some());
    assert!(response.result.is_none());

    let error = response.error.unwrap();
    assert_eq!(error.code, -32602);
    assert!(error.message.contains("missing resource read parameters"));
}

#[tokio::test]
async fn test_handle_resource_read_invalid_params() {
    let server = create_test_server();
    let request = create_request(
        "resources/read",
        Some(json!({
            "invalid_field": "value"
        })),
    );

    let response = handle_resource_read(server, request).await;

    assert!(response.error.is_some());
    assert!(response.result.is_none());

    let error = response.error.unwrap();
    assert_eq!(error.code, -32602);
    assert!(error.message.contains("Invalid params"));
}

#[tokio::test]
async fn test_handle_resource_read_not_found() {
    let server = create_test_server();
    let request = create_request(
        "resources/read",
        Some(json!({
            "uri": "template://nonexistent/template/path"
        })),
    );

    let response = handle_resource_read(server, request).await;

    assert!(response.error.is_some());
    assert!(response.result.is_none());

    let error = response.error.unwrap();
    assert_eq!(error.code, -32000);
    assert!(error.message.contains("Failed to read resource"));
}

#[tokio::test]
async fn test_handle_resource_read_all_templates() {
    let server = create_test_server();

    let templates = vec![
        "template://makefile/rust/cli",
        "template://readme/rust/cli",
        "template://gitignore/rust/cli",
        "template://makefile/python-uv/cli",
        "template://makefile/deno/cli",
        "template://readme/deno/cli",
        "template://readme/python-uv/cli",
        "template://gitignore/deno/cli",
        "template://gitignore/python-uv/cli",
    ];

    for uri in templates {
        let request = create_request("resources/read", Some(json!({ "uri": uri })));

        let response = handle_resource_read(server.clone(), request).await;

        assert!(response.result.is_some(), "Failed to read {}", uri);
        assert!(response.error.is_none(), "Error reading {}", uri);

        let result = response.result.unwrap();
        let contents = result["contents"].as_array().unwrap();
        let content = &contents[0];

        assert_eq!(content["uri"], uri);
        assert!(!content["text"].as_str().unwrap().is_empty());
    }
}
