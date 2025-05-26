use crate::models::mcp::{McpRequest, McpResponse, Prompt, PromptArgument, PromptGetParams};
use crate::TemplateServerTrait;
use serde_json::json;
use std::sync::Arc;

pub async fn handle_prompts_list<T: TemplateServerTrait>(
    _server: Arc<T>,
    request: McpRequest,
) -> McpResponse {
    // Define available prompts for scaffolding projects
    let prompts = vec![
        Prompt {
            name: "scaffold-rust-project".to_string(),
            description:
                "Create a complete Rust project structure with Makefile, README, and .gitignore"
                    .to_string(),
            arguments: vec![
                PromptArgument {
                    name: "project_name".to_string(),
                    description: Some("Name of the Rust project".to_string()),
                    required: true,
                },
                PromptArgument {
                    name: "project_type".to_string(),
                    description: Some("Type of project: cli or library-crate".to_string()),
                    required: true,
                },
                PromptArgument {
                    name: "has_tests".to_string(),
                    description: Some("Include test targets in Makefile".to_string()),
                    required: false,
                },
                PromptArgument {
                    name: "has_benchmarks".to_string(),
                    description: Some("Include benchmark targets in Makefile".to_string()),
                    required: false,
                },
            ],
        },
        Prompt {
            name: "scaffold-deno-project".to_string(),
            description: "Create a complete Deno/TypeScript project structure".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "project_name".to_string(),
                    description: Some("Name of the Deno project".to_string()),
                    required: true,
                },
                PromptArgument {
                    name: "project_type".to_string(),
                    description: Some("Type of project: cli or web-service".to_string()),
                    required: true,
                },
                PromptArgument {
                    name: "permissions".to_string(),
                    description: Some("Deno permissions needed (comma-separated)".to_string()),
                    required: false,
                },
            ],
        },
        Prompt {
            name: "scaffold-python-project".to_string(),
            description: "Create a complete Python UV project structure".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "project_name".to_string(),
                    description: Some("Name of the Python project".to_string()),
                    required: true,
                },
                PromptArgument {
                    name: "project_type".to_string(),
                    description: Some("Type of project: cli or library-package".to_string()),
                    required: true,
                },
                PromptArgument {
                    name: "python_version".to_string(),
                    description: Some("Python version to use (e.g., 3.12)".to_string()),
                    required: false,
                },
            ],
        },
    ];

    McpResponse::success(
        request.id,
        json!({
            "prompts": prompts
        }),
    )
}

pub async fn handle_prompt_get<T: TemplateServerTrait>(
    _server: Arc<T>,
    request: McpRequest,
) -> McpResponse {
    let params = match request.params {
        Some(p) => p,
        None => {
            return McpResponse::error(
                request.id,
                -32602,
                "Invalid params: missing prompt name".to_string(),
            );
        }
    };

    let get_params: PromptGetParams = match serde_json::from_value(params) {
        Ok(p) => p,
        Err(e) => {
            return McpResponse::error(request.id, -32602, format!("Invalid params: {}", e));
        }
    };

    // Get the specific prompt by name
    let prompt = match get_params.name.as_str() {
        "scaffold-rust-project" => Prompt {
            name: "scaffold-rust-project".to_string(),
            description:
                "Create a complete Rust project structure with Makefile, README, and .gitignore"
                    .to_string(),
            arguments: vec![
                PromptArgument {
                    name: "project_name".to_string(),
                    description: Some("Name of the Rust project".to_string()),
                    required: true,
                },
                PromptArgument {
                    name: "project_type".to_string(),
                    description: Some("Type of project: cli or library-crate".to_string()),
                    required: true,
                },
                PromptArgument {
                    name: "has_tests".to_string(),
                    description: Some("Include test targets in Makefile".to_string()),
                    required: false,
                },
                PromptArgument {
                    name: "has_benchmarks".to_string(),
                    description: Some("Include benchmark targets in Makefile".to_string()),
                    required: false,
                },
            ],
        },
        "scaffold-deno-project" => Prompt {
            name: "scaffold-deno-project".to_string(),
            description: "Create a complete Deno/TypeScript project structure".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "project_name".to_string(),
                    description: Some("Name of the Deno project".to_string()),
                    required: true,
                },
                PromptArgument {
                    name: "project_type".to_string(),
                    description: Some("Type of project: cli or web-service".to_string()),
                    required: true,
                },
                PromptArgument {
                    name: "permissions".to_string(),
                    description: Some("Deno permissions needed (comma-separated)".to_string()),
                    required: false,
                },
            ],
        },
        "scaffold-python-project" => Prompt {
            name: "scaffold-python-project".to_string(),
            description: "Create a complete Python UV project structure".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "project_name".to_string(),
                    description: Some("Name of the Python project".to_string()),
                    required: true,
                },
                PromptArgument {
                    name: "project_type".to_string(),
                    description: Some("Type of project: cli or library-package".to_string()),
                    required: true,
                },
                PromptArgument {
                    name: "python_version".to_string(),
                    description: Some("Python version to use (e.g., 3.12)".to_string()),
                    required: false,
                },
            ],
        },
        _ => {
            return McpResponse::error(
                request.id,
                -32602,
                format!("Prompt not found: {}", get_params.name),
            );
        }
    };

    McpResponse::success(request.id, json!(prompt))
}
