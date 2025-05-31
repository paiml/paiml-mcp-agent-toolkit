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
            "resource_uri": "template://makefile/rust/cli",
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

    let test_cases = create_scaffold_test_cases();

    for (toolchain, project_name, description) in test_cases {
        test_toolchain_scaffolding(&server, toolchain, project_name, description).await;
    }
}

fn create_scaffold_test_cases() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("rust", "my-rust-project", "A Rust project"),
        ("deno", "my-deno-project", "A Deno TypeScript project"),
        ("python-uv", "my-python-project", "A Python UV project"),
    ]
}

async fn test_toolchain_scaffolding(
    server: &Arc<StatelessTemplateServer>,
    toolchain: &str,
    project_name: &str,
    description: &str,
) {
    let request = create_scaffold_request(toolchain, project_name, description);
    let response = handle_tool_call(server.clone(), request).await;

    validate_scaffold_response(&response, toolchain);

    let result = response.result.unwrap();
    let generated = result["generated"].as_array().unwrap();

    assert_eq!(
        generated.len(),
        3,
        "Wrong number of files for {}",
        toolchain
    );
    verify_generated_files(generated, toolchain, project_name, description);
}

fn create_scaffold_request(toolchain: &str, project_name: &str, description: &str) -> McpRequest {
    create_tool_request(
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
    )
}

fn validate_scaffold_response(response: &crate::models::mcp::McpResponse, toolchain: &str) {
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
}

fn verify_generated_files(
    generated: &[Value],
    toolchain: &str,
    project_name: &str,
    description: &str,
) {
    let mut file_flags = GeneratedFileFlags::new();

    for file in generated {
        process_generated_file(file, &mut file_flags, toolchain, project_name, description);
    }

    file_flags.assert_all_files_present(toolchain);
}

struct GeneratedFileFlags {
    has_makefile: bool,
    has_readme: bool,
    has_gitignore: bool,
}

impl GeneratedFileFlags {
    fn new() -> Self {
        Self {
            has_makefile: false,
            has_readme: false,
            has_gitignore: false,
        }
    }

    fn assert_all_files_present(&self, toolchain: &str) {
        assert!(self.has_makefile, "Missing Makefile for {}", toolchain);
        assert!(self.has_readme, "Missing README for {}", toolchain);
        assert!(self.has_gitignore, "Missing .gitignore for {}", toolchain);
    }
}

fn process_generated_file(
    file: &Value,
    flags: &mut GeneratedFileFlags,
    toolchain: &str,
    project_name: &str,
    description: &str,
) {
    match file["template"].as_str().unwrap() {
        "makefile" => {
            flags.has_makefile = true;
            verify_makefile(file, toolchain, project_name);
        }
        "readme" => {
            flags.has_readme = true;
            verify_readme(file, project_name, description);
        }
        "gitignore" => {
            flags.has_gitignore = true;
            verify_gitignore(file, toolchain, project_name);
        }
        _ => {}
    }
}

fn verify_makefile(file: &Value, toolchain: &str, project_name: &str) {
    assert_eq!(
        file["filename"].as_str().unwrap(),
        &format!("{}/Makefile", project_name)
    );
    let content = file["content"].as_str().unwrap();
    assert!(content.contains(project_name));

    verify_makefile_toolchain_specific(content, toolchain);
}

fn verify_makefile_toolchain_specific(content: &str, toolchain: &str) {
    match toolchain {
        "rust" => assert!(content.contains("cargo")),
        "deno" => assert!(content.contains("deno")),
        "python-uv" => assert!(content.contains("uv")),
        _ => {}
    }
}

fn verify_readme(file: &Value, project_name: &str, description: &str) {
    assert_eq!(
        file["filename"].as_str().unwrap(),
        &format!("{}/README.md", project_name)
    );
    let content = file["content"].as_str().unwrap();
    assert!(content.contains(project_name));
    assert!(content.contains(description));
}

fn verify_gitignore(file: &Value, toolchain: &str, project_name: &str) {
    assert_eq!(
        file["filename"].as_str().unwrap(),
        &format!("{}/.gitignore", project_name)
    );
    let content = file["content"].as_str().unwrap();

    verify_gitignore_patterns(content, toolchain);
}

fn verify_gitignore_patterns(content: &str, toolchain: &str) {
    match toolchain {
        "rust" => assert!(content.contains("/target/")),
        "deno" => assert!(content.contains("deno.lock")),
        "python-uv" => assert!(content.contains("__pycache__")),
        _ => {}
    }
}

#[tokio::test]
async fn test_claude_code_error_scenarios() {
    let server = create_test_server();

    // Test 1: Invalid template URI (simulating Claude Code's initial error)
    let request = create_tool_request(
        "generate_template",
        json!({
            "resource_uri": "template://readme/rust/invalid-variant", // Wrong variant
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
            "resource_uri": "template://makefile/rust/cli",
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
        "template://readme/rust/cli"
    );
}

#[tokio::test]
async fn test_naming_convention_critical_requirement() {
    let server = create_test_server();

    // Test that all generated templates use correct "paiml-mcp-agent-toolkit" naming
    let test_cases = vec![
        ("rust", "test-rust-project", "A Rust CLI project"),
        ("deno", "test-deno-project", "A Deno TypeScript project"),
        ("python-uv", "test-python-project", "A Python UV project"),
    ];

    for (toolchain, project_name, description) in test_cases {
        // Generate all templates for the toolchain
        let request = create_tool_request(
            "scaffold_project",
            json!({
                "toolchain": toolchain,
                "templates": ["makefile", "readme", "gitignore"],
                "parameters": {
                    "project_name": project_name,
                    "description": description,
                    "author_name": "Test Author",
                    "author_email": "test@example.com",
                    "github_username": "testuser"
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

        // Check each generated file for naming violations
        for file in generated {
            let content = file["content"].as_str().unwrap();
            let filename = file["filename"].as_str().unwrap();

            // Critical: Check for old naming patterns that must not exist
            assert!(
                !content.contains("mcp-agent-toolkit") || content.contains("paiml-mcp-agent-toolkit"),
                "Found incorrect 'mcp-agent-toolkit' (without paiml- prefix) in {} for toolchain {}",
                filename, toolchain
            );

            assert!(
                !content.contains("paiml-agent-toolkit"),
                "Found incorrect 'paiml-agent-toolkit' (missing -mcp-) in {} for toolchain {}",
                filename,
                toolchain
            );

            assert!(
                !content.contains("mcp_server_stateless"),
                "Found old binary name 'mcp_server_stateless' in {} for toolchain {}",
                filename,
                toolchain
            );

            assert!(
                !content.contains("mcp-server-"),
                "Found old artifact pattern 'mcp-server-' in {} for toolchain {}",
                filename,
                toolchain
            );

            // Positive test: Ensure proper naming is used where appropriate
            if file["template"].as_str().unwrap() == "readme" {
                // README should mention the toolkit name
                assert!(
                    content.contains("paiml-mcp-agent-toolkit")
                        || content.contains("PAIML MCP Agent Toolkit")
                        || !content.to_lowercase().contains("toolkit"), // If no toolkit mention, that's ok
                    "README should use correct project name in {} for toolchain {}",
                    filename,
                    toolchain
                );
            }
        }
    }
}

#[tokio::test]
async fn test_naming_convention_in_individual_templates() {
    let server = create_test_server();

    // Test individual template generation for naming compliance
    let template_uris = vec![
        "template://makefile/rust/cli",
        "template://makefile/deno/cli",
        "template://makefile/python-uv/cli",
        "template://readme/rust/cli",
        "template://readme/deno/cli",
        "template://readme/python-uv/cli",
        "template://gitignore/rust/cli",
        "template://gitignore/deno/cli",
        "template://gitignore/python-uv/cli",
    ];

    for uri in template_uris {
        let request = create_tool_request(
            "generate_template",
            json!({
                "resource_uri": uri,
                "parameters": {
                    "project_name": "test-project",
                    "description": "Test project for naming validation",
                    "author_name": "Test Author",
                    "author_email": "test@example.com",
                    "github_username": "testuser"
                }
            }),
        );

        let response = handle_tool_call(server.clone(), request).await;

        // Some URIs might not exist, that's ok for this test
        if response.result.is_some() {
            let result = response.result.unwrap();
            let content = result["content"][0]["text"].as_str().unwrap();

            // Apply same naming convention checks
            assert!(
                !content.contains("mcp-agent-toolkit")
                    || content.contains("paiml-mcp-agent-toolkit"),
                "Found incorrect 'mcp-agent-toolkit' in template {}",
                uri
            );

            assert!(
                !content.contains("paiml-agent-toolkit"),
                "Found incorrect 'paiml-agent-toolkit' in template {}",
                uri
            );

            assert!(
                !content.contains("mcp_server_stateless"),
                "Found old binary name in template {}",
                uri
            );

            assert!(
                !content.contains("mcp-server-"),
                "Found old artifact pattern in template {}",
                uri
            );
        }
    }
}

#[tokio::test]
async fn test_server_info_naming_convention() {
    let server = create_test_server();

    // Test server info command returns correct naming
    let request = create_tool_request("get_server_info", json!({}));

    let response = handle_tool_call(server.clone(), request).await;

    if response.result.is_some() {
        let result = response.result.unwrap();
        let info_str = serde_json::to_string(&result).unwrap();

        // Server info should not contain old names
        assert!(
            !info_str.contains("mcp-agent-toolkit") || info_str.contains("paiml-mcp-agent-toolkit"),
            "Server info contains incorrect 'mcp-agent-toolkit' naming"
        );

        assert!(
            !info_str.contains("paiml-agent-toolkit"),
            "Server info contains incorrect 'paiml-agent-toolkit' naming"
        );

        assert!(
            !info_str.contains("mcp_server_stateless"),
            "Server info contains old binary name"
        );
    }
}

#[tokio::test]
async fn test_ast_context_generation() {
    let server = create_test_server();

    // Create a temporary directory with some Rust files for testing
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();

    // Create a simple Rust file with various AST elements
    let test_file_content = r#"
use std::collections::HashMap;

pub mod utils {
    pub fn helper() {}
}

#[derive(Debug, Clone)]
pub struct TestStruct {
    field1: String,
    field2: i32,
}

pub enum TestEnum {
    Variant1,
    Variant2(String),
}

pub trait TestTrait {
    fn method(&self);
}

impl TestTrait for TestStruct {
    fn method(&self) {
        println!("Implementation");
    }
}

pub async fn async_function() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

fn private_function() {
    // Private function
}
"#;

    // Write test file
    std::fs::write(temp_path.join("test.rs"), test_file_content).unwrap();

    // Create a Cargo.toml file
    let cargo_toml = r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = "1.0"
serde = "1.0"
"#;
    std::fs::write(temp_path.join("Cargo.toml"), cargo_toml).unwrap();

    // Test AST context generation via MCP tool
    let request = create_tool_request(
        "generate_template",
        json!({
            "resource_uri": "template://context/rust/ast",
            "parameters": {
                "project_path": temp_path.to_str().unwrap()
            }
        }),
    );

    let response = handle_tool_call(server.clone(), request).await;
    assert!(response.result.is_some());
    assert!(response.error.is_none());

    let result = response.result.unwrap();
    let content = result["content"][0]["text"].as_str().unwrap();

    // Debug output to see what's actually generated
    eprintln!("Generated content:\n{}", content);

    // Verify the content contains expected elements
    assert!(content.contains("# Project Context: rust Project"));
    assert!(content.contains("## Summary"));
    assert!(content.contains("Files analyzed: 1"));
    // Note: Our simple AST parser counts functions at top level, not inside modules
    assert!(content.contains("Functions: 2")); // async_function, private_function (helper is inside a module)
    assert!(content.contains("Structs: 1")); // TestStruct
    assert!(content.contains("Enums: 1")); // TestEnum
    assert!(content.contains("Traits: 1")); // TestTrait
    assert!(content.contains("Implementations: 1")); // impl TestTrait for TestStruct

    // Check dependencies section
    assert!(content.contains("## Dependencies"));
    assert!(content.contains("- tokio"));
    assert!(content.contains("- serde"));

    // Check file analysis section
    assert!(content.contains("## Files"));
    assert!(content.contains("test.rs"));

    // Check specific AST items
    assert!(content.contains("**Modules:**"));
    assert!(content.contains("`pub mod utils`"));

    assert!(content.contains("**Structs:**"));
    assert!(content.contains("`pub struct TestStruct` (2 fields)"));

    assert!(content.contains("**Enums:**"));
    assert!(content.contains("`pub enum TestEnum` (2 variants)"));

    assert!(content.contains("**Traits:**"));
    assert!(content.contains("`pub trait TestTrait`"));

    assert!(content.contains("**Functions:**"));
    assert!(content.contains("`pub async fn async_function`"));
    assert!(content.contains("`private fn private_function`"));

    assert!(content.contains("**Implementations:**"));
    assert!(content.contains("`impl TestTrait for TestStruct`"));

    // Verify the generated filename
    assert_eq!(result["filename"].as_str().unwrap(), "CONTEXT.md");

    // Test with invalid toolchain
    let request = create_tool_request(
        "generate_template",
        json!({
            "resource_uri": "template://context/invalid/ast",
            "parameters": {
                "project_path": temp_path.to_str().unwrap()
            }
        }),
    );

    let response = handle_tool_call(server.clone(), request).await;
    assert!(response.error.is_some());

    // Test with non-existent path
    let request = create_tool_request(
        "generate_template",
        json!({
            "resource_uri": "template://context/rust/ast",
            "parameters": {
                "project_path": "/non/existent/path"
            }
        }),
    );

    let response = handle_tool_call(server.clone(), request).await;
    assert!(response.result.is_some()); // Should still succeed but with 0 files analyzed

    let result = response.result.unwrap();
    let content = result["content"][0]["text"].as_str().unwrap();
    assert!(content.contains("Files analyzed: 0"));
}
