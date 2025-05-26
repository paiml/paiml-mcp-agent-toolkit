use crate::models::mcp::{McpRequest, McpResponse};
use crate::TemplateServerTrait;
use serde_json::json;
use std::sync::Arc;

pub async fn handle_initialize<T: TemplateServerTrait>(
    _server: Arc<T>,
    request: McpRequest,
) -> McpResponse {
    // Extract protocol version from params if provided
    let protocol_version = request
        .params
        .as_ref()
        .and_then(|p| p.get("protocolVersion"))
        .and_then(|v| v.as_str())
        .unwrap_or("2024-11-05");

    // Return initialization response with server info
    McpResponse::success(
        request.id,
        json!({
            "protocolVersion": protocol_version,
            "capabilities": {
                "tools": {},
                "resources": {},
                "prompts": {},
            },
            "serverInfo": {
                "name": "paiml-mcp-agent-toolkit",
                "version": env!("CARGO_PKG_VERSION"),
                "vendor": "Pragmatic AI Labs (paiml.com)",
                "author": "Pragmatic AI Labs",
                "description": "Professional project scaffolding toolkit that generates Makefiles, README.md files, and .gitignore files for Rust, Deno, and Python projects. Created by Pragmatic AI Labs to streamline project setup with best practices.",
                "capabilities": [
                    "Generate individual project files (Makefile, README.md, .gitignore)",
                    "Scaffold complete projects with all files at once",
                    "Support for Rust CLI/library projects",
                    "Support for Deno/TypeScript applications",
                    "Support for Python UV projects",
                    "Smart subdirectory creation for organized project structure"
                ],
                "supportedTemplates": ["makefile", "readme", "gitignore"],
                "supportedToolchains": ["rust", "deno", "python-uv"],
                "examples": [
                    "Create a new Rust CLI project: scaffold_project with toolchain='rust'",
                    "Generate just a Makefile: generate_template with resource_uri='template://makefile/rust/cli-binary'",
                    "Search for Python templates: search_templates with query='python'"
                ]
            }
        }),
    )
}

pub async fn handle_tools_list<T: TemplateServerTrait>(
    _server: Arc<T>,
    request: McpRequest,
) -> McpResponse {
    // Return list of available tools
    McpResponse::success(
        request.id,
        json!({
            "tools": [
                {
                    "name": "get_server_info",
                    "description": "Get information about the PAIML MCP Agent Toolkit server, including author, version, and capabilities",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                },
                {
                    "name": "generate_template",
                    "description": "Generate project files (Makefile, README, .gitignore) from PAIML templates. Automatically detects project type and creates appropriate build, documentation, and ignore files.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "resource_uri": {
                                "type": "string",
                                "description": "Template URI (e.g., template://makefile/rust/cli-binary)"
                            },
                            "parameters": {
                                "type": "object",
                                "description": "Template parameters as key-value pairs"
                            }
                        },
                        "required": ["resource_uri", "parameters"]
                    }
                },
                {
                    "name": "list_templates",
                    "description": "List all available PAIML templates for project scaffolding. Shows templates for Makefiles, READMEs, and .gitignore files across Rust, Deno, and Python toolchains.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "toolchain": {
                                "type": "string",
                                "description": "Filter by toolchain (rust, deno, python-uv)"
                            },
                            "category": {
                                "type": "string",
                                "description": "Filter by category (makefile, readme, gitignore)"
                            }
                        }
                    }
                },
                {
                    "name": "validate_template",
                    "description": "Validate template parameters before generation. Checks if all required parameters are provided and have valid values.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "resource_uri": {
                                "type": "string",
                                "description": "Template URI to validate"
                            },
                            "parameters": {
                                "type": "object",
                                "description": "Parameters to validate"
                            }
                        },
                        "required": ["resource_uri", "parameters"]
                    }
                },
                {
                    "name": "scaffold_project",
                    "description": "Create a complete project structure with Makefile, README.md, and .gitignore. Perfect for starting new Rust, Deno, or Python projects with best practices. Files are created in a project subdirectory.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "toolchain": {
                                "type": "string",
                                "description": "Toolchain to use (rust, deno, python-uv)"
                            },
                            "templates": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "List of template types to generate (makefile, readme, gitignore)"
                            },
                            "parameters": {
                                "type": "object",
                                "description": "Common parameters for all templates"
                            }
                        },
                        "required": ["toolchain", "templates", "parameters"]
                    }
                },
                {
                    "name": "search_templates",
                    "description": "Search for templates matching a query string. Searches in template names, descriptions, and parameter names.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "Search query"
                            },
                            "toolchain": {
                                "type": "string",
                                "description": "Optional toolchain filter"
                            }
                        },
                        "required": ["query"]
                    }
                }
            ]
        }),
    )
}
