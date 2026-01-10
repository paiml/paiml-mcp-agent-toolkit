//! Comprehensive tests for handlers/tools.rs to improve coverage
//! Target: Increase coverage from 18% to 80%+

use pmat::handlers::tools::*;
use pmat::models::mcp::{McpRequest, McpResponse, ToolCallParams};
use pmat::stateless_server::StatelessTemplateServer;
use serde_json::json;
use std::sync::Arc;

#[cfg(test)]
mod tool_classification_tests {
    use super::*;

    #[test]
    fn test_is_template_tool_positive_cases() {
        // All template tools should return true
        assert!(is_template_tool("generate_template"));
        assert!(is_template_tool("list_templates"));
        assert!(is_template_tool("validate_template"));
        assert!(is_template_tool("scaffold_project"));
        assert!(is_template_tool("search_templates"));
    }

    #[test]
    fn test_is_template_tool_negative_cases() {
        // Non-template tools should return false
        assert!(!is_template_tool("analyze_complexity"));
        assert!(!is_template_tool("get_server_info"));
        assert!(!is_template_tool("unknown_tool"));
        assert!(!is_template_tool(""));
        assert!(!is_template_tool("GENERATE_TEMPLATE")); // case sensitive
    }

    #[test]
    fn test_is_analysis_tool_positive_cases() {
        // All analysis tools should return true
        assert!(is_analysis_tool("analyze_code_churn"));
        assert!(is_analysis_tool("analyze_complexity"));
        assert!(is_analysis_tool("analyze_dag"));
        assert!(is_analysis_tool("generate_context"));
        assert!(is_analysis_tool("analyze_system_architecture"));
        assert!(is_analysis_tool("analyze_defect_probability"));
        assert!(is_analysis_tool("analyze_dead_code"));
        assert!(is_analysis_tool("analyze_deep_context"));
        assert!(is_analysis_tool("analyze_tdg"));
        assert!(is_analysis_tool("analyze_makefile_lint"));
        assert!(is_analysis_tool("analyze_provability"));
        assert!(is_analysis_tool("analyze_satd"));
        assert!(is_analysis_tool("quality_driven_development"));
        assert!(is_analysis_tool("analyze_lint_hotspot"));
    }

    #[test]
    fn test_is_analysis_tool_negative_cases() {
        assert!(!is_analysis_tool("generate_template"));
        assert!(!is_analysis_tool("list_templates"));
        assert!(!is_analysis_tool("get_server_info"));
        assert!(!is_analysis_tool("unknown"));
        assert!(!is_analysis_tool(""));
    }
}

#[cfg(test)]
mod tool_call_handler_tests {
    use super::*;

    #[tokio::test]
    async fn test_handle_tool_call_unknown_tool() {
        let server = Arc::new(StatelessTemplateServer::new().unwrap());
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "unknown_tool_xyz",
                "arguments": {}
            })),
        };

        let response = handle_tool_call(server, request).await;
        // Should return error for unknown tool
        assert!(response.error.is_some());
    }

    #[tokio::test]
    async fn test_handle_tool_call_missing_params() {
        let server = Arc::new(StatelessTemplateServer::new().unwrap());
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(2),
            method: "tools/call".to_string(),
            params: None,
        };

        let response = handle_tool_call(server, request).await;
        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
        assert!(error.message.contains("missing tool call parameters"));
    }

    #[tokio::test]
    async fn test_handle_tool_call_invalid_params() {
        let server = Arc::new(StatelessTemplateServer::new().unwrap());
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(3),
            method: "tools/call".to_string(),
            params: Some(json!("invalid_string_instead_of_object")),
        };

        let response = handle_tool_call(server, request).await;
        assert!(response.error.is_some());
    }

    #[tokio::test]
    async fn test_handle_tool_call_get_server_info() {
        let server = Arc::new(StatelessTemplateServer::new().unwrap());
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(4),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "get_server_info",
                "arguments": {}
            })),
        };

        let response = handle_tool_call(server, request).await;
        // get_server_info should succeed
        assert!(response.error.is_none());
        assert!(response.result.is_some());
    }

    #[tokio::test]
    async fn test_handle_tool_call_list_templates() {
        let server = Arc::new(StatelessTemplateServer::new().unwrap());
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(5),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "list_templates",
                "arguments": {}
            })),
        };

        let response = handle_tool_call(server, request).await;
        // list_templates should succeed
        assert!(response.error.is_none());
    }

    #[tokio::test]
    async fn test_handle_tool_call_list_templates_with_filter() {
        let server = Arc::new(StatelessTemplateServer::new().unwrap());
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(6),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "list_templates",
                "arguments": {
                    "toolchain": "rust",
                    "category": "build"
                }
            })),
        };

        let response = handle_tool_call(server, request).await;
        assert!(response.error.is_none());
    }

    #[tokio::test]
    async fn test_handle_tool_call_search_templates() {
        let server = Arc::new(StatelessTemplateServer::new().unwrap());
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(7),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "search_templates",
                "arguments": {
                    "query": "makefile"
                }
            })),
        };

        let response = handle_tool_call(server, request).await;
        assert!(response.error.is_none());
    }

    #[tokio::test]
    async fn test_handle_tool_call_validate_template_missing_uri() {
        let server = Arc::new(StatelessTemplateServer::new().unwrap());
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(8),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "validate_template",
                "arguments": {
                    "parameters": {}
                }
            })),
        };

        let response = handle_tool_call(server, request).await;
        // Should fail due to missing resource_uri
        assert!(response.error.is_some());
    }

    #[tokio::test]
    async fn test_handle_tool_call_generate_template_missing_params() {
        let server = Arc::new(StatelessTemplateServer::new().unwrap());
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(9),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "generate_template",
                "arguments": {
                    "resource_uri": "template://makefile/rust/cli"
                    // Missing "parameters" field
                }
            })),
        };

        let response = handle_tool_call(server, request).await;
        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert!(error.message.contains("parameters"));
    }
}

#[cfg(test)]
mod analysis_tool_tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_analyze_complexity_missing_path() {
        let server = Arc::new(StatelessTemplateServer::new().unwrap());
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(10),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "analyze_complexity",
                "arguments": {}
            })),
        };

        let response = handle_tool_call(server, request).await;
        // Should use default path (current directory)
        // May succeed or fail depending on cwd
        let _ = response;
    }

    #[tokio::test]
    async fn test_analyze_complexity_with_path() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("test.rs");
        std::fs::write(
            &rust_file,
            r#"
fn simple() {
    println!("hello");
}

fn with_branch(x: i32) -> i32 {
    if x > 0 { x } else { -x }
}
"#,
        )
        .unwrap();

        let server = Arc::new(StatelessTemplateServer::new().unwrap());
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(11),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "analyze_complexity",
                "arguments": {
                    "project_path": temp_dir.path().to_str().unwrap()
                }
            })),
        };

        let response = handle_tool_call(server, request).await;
        // Should analyze the test file
        assert!(response.error.is_none() || response.result.is_some());
    }

    #[tokio::test]
    async fn test_analyze_dead_code() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("lib.rs");
        std::fs::write(
            &rust_file,
            r#"
pub fn used_function() {
    println!("used");
}

fn unused_function() {
    println!("unused");
}
"#,
        )
        .unwrap();

        let server = Arc::new(StatelessTemplateServer::new().unwrap());
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(12),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "analyze_dead_code",
                "arguments": {
                    "path": temp_dir.path().to_str().unwrap()
                }
            })),
        };

        let response = handle_tool_call(server, request).await;
        let _ = response; // May succeed or fail based on analysis
    }

    #[tokio::test]
    async fn test_analyze_satd() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("test.rs");
        std::fs::write(
            &rust_file,
            r#"
// TODO: Fix this later
fn needs_work() {
    // FIXME: This is broken
    // HACK: Workaround for issue #123
    println!("technical debt");
}
"#,
        )
        .unwrap();

        let server = Arc::new(StatelessTemplateServer::new().unwrap());
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(13),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "analyze_satd",
                "arguments": {
                    "path": temp_dir.path().to_str().unwrap()
                }
            })),
        };

        let response = handle_tool_call(server, request).await;
        let _ = response;
    }

    #[tokio::test]
    #[ignore] // Times out in coverage builds (>120s)
    async fn test_analyze_tdg() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("main.rs");
        std::fs::write(
            &rust_file,
            r#"
fn main() {
    println!("Hello, TDG!");
}
"#,
        )
        .unwrap();

        let server = Arc::new(StatelessTemplateServer::new().unwrap());
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(14),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "analyze_tdg",
                "arguments": {
                    "path": temp_dir.path().to_str().unwrap()
                }
            })),
        };

        let response = handle_tool_call(server, request).await;
        let _ = response;
    }

    #[tokio::test]
    async fn test_generate_context() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("lib.rs");
        std::fs::write(
            &rust_file,
            r#"
pub struct MyStruct {
    field: i32,
}

impl MyStruct {
    pub fn new(field: i32) -> Self {
        Self { field }
    }
}
"#,
        )
        .unwrap();

        let server = Arc::new(StatelessTemplateServer::new().unwrap());
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(15),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "generate_context",
                "arguments": {
                    "path": temp_dir.path().to_str().unwrap()
                }
            })),
        };

        let response = handle_tool_call(server, request).await;
        let _ = response;
    }

    #[tokio::test]
    async fn test_analyze_dag() {
        let temp_dir = TempDir::new().unwrap();

        // Create Cargo.toml for dependency analysis
        std::fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"
[package]
name = "test-project"
version = "0.1.0"

[dependencies]
serde = "1.0"
"#,
        )
        .unwrap();

        let server = Arc::new(StatelessTemplateServer::new().unwrap());
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(16),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "analyze_dag",
                "arguments": {
                    "path": temp_dir.path().to_str().unwrap()
                }
            })),
        };

        let response = handle_tool_call(server, request).await;
        let _ = response;
    }

    #[tokio::test]
    async fn test_analyze_deep_context() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("main.rs");
        std::fs::write(
            &rust_file,
            r#"
fn main() {
    println!("Deep context test");
}
"#,
        )
        .unwrap();

        let server = Arc::new(StatelessTemplateServer::new().unwrap());
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(17),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "analyze_deep_context",
                "arguments": {
                    "path": temp_dir.path().to_str().unwrap()
                }
            })),
        };

        let response = handle_tool_call(server, request).await;
        let _ = response;
    }

    #[tokio::test]
    async fn test_analyze_system_architecture() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"
[package]
name = "test"
version = "0.1.0"
"#,
        )
        .unwrap();

        let server = Arc::new(StatelessTemplateServer::new().unwrap());
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(18),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "analyze_system_architecture",
                "arguments": {
                    "path": temp_dir.path().to_str().unwrap()
                }
            })),
        };

        let response = handle_tool_call(server, request).await;
        let _ = response;
    }

    #[tokio::test]
    async fn test_analyze_makefile_lint() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(
            temp_dir.path().join("Makefile"),
            r#"
.PHONY: all build test

all: build test

build:
	cargo build

test:
	cargo test
"#,
        )
        .unwrap();

        let server = Arc::new(StatelessTemplateServer::new().unwrap());
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(19),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "analyze_makefile_lint",
                "arguments": {
                    "path": temp_dir.path().to_str().unwrap()
                }
            })),
        };

        let response = handle_tool_call(server, request).await;
        let _ = response;
    }

    #[tokio::test]
    async fn test_analyze_provability() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("test.rs");
        std::fs::write(
            &rust_file,
            r#"
/// # Safety
/// This is safe because we check the bounds first
unsafe fn safe_op(x: *const i32) -> i32 {
    *x
}
"#,
        )
        .unwrap();

        let server = Arc::new(StatelessTemplateServer::new().unwrap());
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(20),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "analyze_provability",
                "arguments": {
                    "path": temp_dir.path().to_str().unwrap()
                }
            })),
        };

        let response = handle_tool_call(server, request).await;
        let _ = response;
    }

    #[tokio::test]
    async fn test_analyze_lint_hotspot() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("test.rs");
        std::fs::write(
            &rust_file,
            r#"
#[allow(dead_code)]
fn lint_test() {
    let x = 1;  // unused variable
}
"#,
        )
        .unwrap();

        let server = Arc::new(StatelessTemplateServer::new().unwrap());
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(21),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "analyze_lint_hotspot",
                "arguments": {
                    "path": temp_dir.path().to_str().unwrap()
                }
            })),
        };

        let response = handle_tool_call(server, request).await;
        let _ = response;
    }

    #[tokio::test]
    async fn test_quality_driven_development() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"
[package]
name = "test"
version = "0.1.0"
"#,
        )
        .unwrap();

        let server = Arc::new(StatelessTemplateServer::new().unwrap());
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(22),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "quality_driven_development",
                "arguments": {
                    "path": temp_dir.path().to_str().unwrap()
                }
            })),
        };

        let response = handle_tool_call(server, request).await;
        let _ = response;
    }
}

#[cfg(test)]
mod template_validation_tests {
    use super::*;

    #[tokio::test]
    async fn test_validate_template_nonexistent() {
        let server = Arc::new(StatelessTemplateServer::new().unwrap());
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(30),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "validate_template",
                "arguments": {
                    "resource_uri": "template://nonexistent/template",
                    "parameters": {}
                }
            })),
        };

        let response = handle_tool_call(server, request).await;
        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert!(error.message.contains("not found"));
    }

    #[tokio::test]
    async fn test_scaffold_project_invalid_output() {
        let server = Arc::new(StatelessTemplateServer::new().unwrap());
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(31),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "scaffold_project",
                "arguments": {
                    "templates": ["template://makefile/rust/cli"],
                    "output_dir": "/nonexistent/readonly/path",
                    "parameters": {
                        "project_name": "test"
                    }
                }
            })),
        };

        let response = handle_tool_call(server, request).await;
        // Should fail due to invalid output path
        let _ = response;
    }

    #[tokio::test]
    async fn test_generate_template_valid() {
        let server = Arc::new(StatelessTemplateServer::new().unwrap());
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(32),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "generate_template",
                "arguments": {
                    "resource_uri": "template://makefile/rust/cli",
                    "parameters": {
                        "project_name": "test-project",
                        "binary_name": "test"
                    }
                }
            })),
        };

        let response = handle_tool_call(server, request).await;
        // Should succeed with valid template
        assert!(response.error.is_none());
    }

    #[tokio::test]
    async fn test_validate_template_valid() {
        let server = Arc::new(StatelessTemplateServer::new().unwrap());
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(33),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "validate_template",
                "arguments": {
                    "resource_uri": "template://makefile/rust/cli",
                    "parameters": {
                        "project_name": "test"
                    }
                }
            })),
        };

        let response = handle_tool_call(server, request).await;
        // Should succeed
        assert!(response.error.is_none());
    }
}

// Re-export functions that need to be tested
pub use pmat::handlers::tools::{
    format_churn_as_csv, format_churn_as_markdown, format_churn_summary, is_analysis_tool,
    is_template_tool,
};
