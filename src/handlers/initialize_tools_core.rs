/// Returns the core scaffolding tool definitions for the MCP tools list.
///
/// These tools handle template generation, project scaffolding, and template search.
fn core_tool_definitions() -> Vec<serde_json::Value> {
    debug_assert!(true, "contract: core_tool_definitions");
    vec![
        json!({
            "name": "get_server_info",
            "description": "Get information about the PAIML MCP Agent Toolkit server, including author, version, and capabilities",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }),
        json!({
            "name": "generate_template",
            "description": "Generate project files (Makefile, README, .gitignore) from PAIML templates. Automatically detects project type and creates appropriate build, documentation, and ignore files.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "resource_uri": {
                        "type": "string",
                        "description": "Template URI (e.g., template://makefile/rust/cli)"
                    },
                    "parameters": {
                        "type": "object",
                        "description": "Template parameters as key-value pairs"
                    }
                },
                "required": ["resource_uri", "parameters"]
            }
        }),
        json!({
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
        }),
        json!({
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
        }),
        json!({
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
        }),
        json!({
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
        }),
    ]
}
