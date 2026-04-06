/// Create scaffold_agent tool definition
fn create_scaffold_agent_tool() -> ToolDefinition {
    ToolDefinition {
        name: "scaffold_agent".to_string(),
        description: "Generate a new MCP agent project with best practices".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Project name"
                },
                "template": {
                    "type": "string",
                    "enum": ["basic", "stateful", "hybrid"],
                    "default": "basic",
                    "description": "Agent template type"
                },
                "output_dir": {
                    "type": "string",
                    "default": ".",
                    "description": "Output directory"
                },
                "quality_level": {
                    "type": "string",
                    "enum": ["extreme", "high", "standard"],
                    "default": "extreme",
                    "description": "Quality enforcement level"
                }
            },
            "required": ["name"]
        }),
    }
}

/// Create scaffold_wasm tool definition
fn create_scaffold_wasm_tool() -> ToolDefinition {
    ToolDefinition {
        name: "scaffold_wasm".to_string(),
        description: "Generate a new WASM project with best practices".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Project name"
                },
                "framework": {
                    "type": "string",
                    "enum": ["wasm-labs", "pure-wasm"],
                    "default": "wasm-labs",
                    "description": "WASM framework"
                },
                "output_dir": {
                    "type": "string",
                    "default": ".",
                    "description": "Output directory"
                },
                "quality_level": {
                    "type": "string",
                    "enum": ["extreme", "high", "standard"],
                    "default": "extreme",
                    "description": "Quality enforcement level"
                }
            },
            "required": ["name"]
        }),
    }
}

/// Create validate_roadmap tool definition
fn create_validate_roadmap_tool() -> ToolDefinition {
    ToolDefinition {
        name: "validate_roadmap".to_string(),
        description: "Validate roadmap structure and ticket consistency".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "roadmap_path": {
                    "type": "string",
                    "default": "ROADMAP.md",
                    "description": "Path to ROADMAP.md"
                },
                "tickets_dir": {
                    "type": "string",
                    "default": "docs/tickets",
                    "description": "Path to tickets directory"
                }
            }
        }),
    }
}

/// Create health_check tool definition
fn create_health_check_tool() -> ToolDefinition {
    ToolDefinition {
        name: "health_check".to_string(),
        description: "Run project health checks (build, tests, coverage, complexity)".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "project_dir": {
                    "type": "string",
                    "default": ".",
                    "description": "Project directory"
                },
                "quick": {
                    "type": "boolean",
                    "default": false,
                    "description": "Quick mode (build check only)"
                },
                "all": {
                    "type": "boolean",
                    "default": false,
                    "description": "Run all checks"
                },
                "check_build": {
                    "type": "boolean",
                    "default": false,
                    "description": "Check build status"
                },
                "check_tests": {
                    "type": "boolean",
                    "default": false,
                    "description": "Check tests"
                },
                "check_coverage": {
                    "type": "boolean",
                    "default": false,
                    "description": "Check coverage"
                }
            }
        }),
    }
}

/// Create generate_tickets tool definition
fn create_generate_tickets_tool() -> ToolDefinition {
    ToolDefinition {
        name: "generate_tickets".to_string(),
        description: "Auto-generate missing ticket files from roadmap entries".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "roadmap_path": {
                    "type": "string",
                    "default": "ROADMAP.md",
                    "description": "Path to ROADMAP.md"
                },
                "tickets_dir": {
                    "type": "string",
                    "default": "docs/tickets",
                    "description": "Path to tickets directory"
                },
                "dry_run": {
                    "type": "boolean",
                    "default": false,
                    "description": "Preview mode without creating files"
                }
            }
        }),
    }
}
