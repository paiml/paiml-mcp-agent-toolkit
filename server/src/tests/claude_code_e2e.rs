use crate::handlers::tools::handle_tool_call;
use crate::models::mcp::{McpRequest, ToolCallParams};
use crate::stateless_server::StatelessTemplateServer;
use serde_json::{json, Value};
use std::sync::Arc;

fn create_test_server() -> Arc<StatelessTemplateServer> {
    Arc::new(StatelessTemplateServer::new().unwrap())
}

fn create_tool_request(tool_name: &str, arguments: Value) -> McpRequest {
    let params = ToolCallParams {
        name: tool_name.to_string(),
        arguments,
    };

    McpRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(1),
        method: "tools/call".to_string(),
        params: Some(serde_json::to_value(params).unwrap()),
    }
}

#[tokio::test]
async fn test_claude_code_rust_cli_workflow() {
    let server = create_test_server();

    // Step 1: List templates for Rust (simulating Claude Code's first action)
    let request = create_tool_request(
        "list_templates",
        json!({
            "toolchain": "rust"
        }),
    );

    let response = handle_tool_call(server.clone(), request).await;
    assert!(response.result.is_some());
    assert!(response.error.is_none());

    let result = response.result.unwrap();
    let templates = result["templates"].as_array().unwrap();
    assert_eq!(templates.len(), 3); // Makefile, README, and .gitignore for Rust

    // Step 2: Try scaffold_project (simulating Claude Code's scaffold attempt)
    let request = create_tool_request(
        "scaffold_project",
        json!({
            "toolchain": "rust",
            "templates": ["makefile", "readme", "gitignore"],
            "parameters": {
                "project_name": "my-rust-cli",
                "author_name": "Your Name",
                "author_email": "your.email@example.com",
                "description": "A Rust CLI application"
            }
        }),
    );

    let response = handle_tool_call(server.clone(), request).await;
    assert!(response.result.is_some());
    assert!(response.error.is_none());

    let result = response.result.unwrap();
    let generated = result["generated"].as_array().unwrap();
    assert_eq!(generated.len(), 3);

    // Verify each generated file
    for file in generated {
        let template_type = file["template"].as_str().unwrap();
        let filename = file["filename"].as_str().unwrap();
        let content = file["content"].as_str().unwrap();

        match template_type {
            "makefile" => {
                assert_eq!(filename, "my-rust-cli/Makefile");
                assert!(content.contains("my-rust-cli"));
                assert!(content.contains("cargo build"));
            }
            "readme" => {
                assert_eq!(filename, "my-rust-cli/README.md");
                assert!(content.contains("my-rust-cli"));
                assert!(content.contains("A Rust CLI application"));
            }
            "gitignore" => {
                assert_eq!(filename, "my-rust-cli/.gitignore");
                // The Rust gitignore template uses handlebars variables with defaults
                // It may show "Build artifacts" section header or the actual patterns
                assert!(content.contains("target/") || content.contains("Build artifacts"));
            }
            _ => panic!("Unexpected template type: {}", template_type),
        }
    }

    // Step 3: Test individual generate_template call (simulating Claude Code's fallback)
    let request = create_tool_request(
        "generate_template",
        json!({
            "resource_uri": "template://makefile/rust/cli-binary",
            "parameters": {
                "project_name": "my-rust-cli"
            }
        }),
    );

    let response = handle_tool_call(server.clone(), request).await;
    assert!(response.result.is_some());
    assert!(response.error.is_none());

    let result = response.result.unwrap();
    let content = result["content"][0]["text"].as_str().unwrap();
    assert!(content.contains("my-rust-cli"));
    assert!(content.contains("Rust CLI Binary Makefile"));
}

#[tokio::test]
async fn test_claude_code_all_languages_scaffold() {
    let server = create_test_server();

    let test_cases = vec![
        ("rust", "my-rust-project", "A Rust project"),
        ("deno", "my-deno-project", "A Deno TypeScript project"),
        ("python-uv", "my-python-project", "A Python UV project"),
    ];

    for (toolchain, project_name, description) in test_cases {
        // Test scaffolding for each language
        let request = create_tool_request(
            "scaffold_project",
            json!({
                "toolchain": toolchain,
                "templates": ["makefile", "readme", "gitignore"],
                "parameters": {
                    "project_name": project_name,
                    "description": description,
                    "author_name": "Test Author"
                }
            }),
        );

        let response = handle_tool_call(server.clone(), request).await;
        assert!(
            response.result.is_some(),
            "Failed for toolchain: {}",
            toolchain
        );
        assert!(
            response.error.is_none(),
            "Error for toolchain: {}",
            toolchain
        );

        let result = response.result.unwrap();
        let generated = result["generated"].as_array().unwrap();
        assert_eq!(
            generated.len(),
            3,
            "Wrong number of files for {}",
            toolchain
        );

        // Verify all three files were generated
        let mut has_makefile = false;
        let mut has_readme = false;
        let mut has_gitignore = false;

        for file in generated {
            match file["template"].as_str().unwrap() {
                "makefile" => {
                    has_makefile = true;
                    assert_eq!(
                        file["filename"].as_str().unwrap(),
                        &format!("{}/Makefile", project_name)
                    );
                    let content = file["content"].as_str().unwrap();
                    assert!(content.contains(project_name));

                    // Language-specific checks
                    match toolchain {
                        "rust" => assert!(content.contains("cargo")),
                        "deno" => assert!(content.contains("deno")),
                        "python-uv" => assert!(content.contains("uv")),
                        _ => {}
                    }
                }
                "readme" => {
                    has_readme = true;
                    assert_eq!(
                        file["filename"].as_str().unwrap(),
                        &format!("{}/README.md", project_name)
                    );
                    let content = file["content"].as_str().unwrap();
                    assert!(content.contains(project_name));
                    assert!(content.contains(description));
                }
                "gitignore" => {
                    has_gitignore = true;
                    assert_eq!(
                        file["filename"].as_str().unwrap(),
                        &format!("{}/.gitignore", project_name)
                    );
                    let content = file["content"].as_str().unwrap();

                    // Language-specific gitignore patterns
                    match toolchain {
                        "rust" => assert!(content.contains("/target/")),
                        "deno" => assert!(content.contains("deno.lock")),
                        "python-uv" => assert!(content.contains("__pycache__")),
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        assert!(has_makefile, "Missing Makefile for {}", toolchain);
        assert!(has_readme, "Missing README for {}", toolchain);
        assert!(has_gitignore, "Missing .gitignore for {}", toolchain);
    }
}

#[tokio::test]
async fn test_claude_code_error_scenarios() {
    let server = create_test_server();

    // Test 1: Invalid template URI (simulating Claude Code's initial error)
    let request = create_tool_request(
        "generate_template",
        json!({
            "resource_uri": "template://readme/rust/cli-binary", // Wrong variant
            "parameters": {
                "project_name": "test-project"
            }
        }),
    );

    let response = handle_tool_call(server.clone(), request).await;
    assert!(response.error.is_some());
    assert_eq!(response.error.unwrap().code, -32001);

    // Test 2: Missing required parameters
    let request = create_tool_request(
        "generate_template",
        json!({
            "resource_uri": "template://makefile/rust/cli-binary",
            "parameters": {} // Missing project_name
        }),
    );

    let response = handle_tool_call(server.clone(), request).await;
    assert!(response.error.is_some());

    // Test 3: Invalid toolchain in scaffold
    let request = create_tool_request(
        "scaffold_project",
        json!({
            "toolchain": "invalid-toolchain",
            "templates": ["makefile"],
            "parameters": {
                "project_name": "test"
            }
        }),
    );

    let response = handle_tool_call(server.clone(), request).await;
    assert!(response.result.is_some());
    let result = response.result.unwrap();
    let generated = result["generated"].as_array().unwrap();
    assert!(generated.is_empty()); // Should not generate any files for invalid toolchain
}

#[tokio::test]
async fn test_claude_code_search_templates() {
    let server = create_test_server();

    // Test searching for README templates (simulating Claude Code's search)
    let request = create_tool_request(
        "search_templates",
        json!({
            "query": "readme",
            "toolchain": "rust"
        }),
    );

    let response = handle_tool_call(server.clone(), request).await;
    assert!(response.result.is_some());
    assert!(response.error.is_none());

    let result = response.result.unwrap();
    let templates = result["templates"].as_array().unwrap();
    assert_eq!(templates.len(), 1); // Only Rust README
    assert_eq!(
        templates[0]["uri"].as_str().unwrap(),
        "template://readme/rust/cli-application"
    );
}
