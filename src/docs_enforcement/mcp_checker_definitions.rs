// MCP checker tool definitions - included by mcp_checker.rs

/// Load MCP tool definitions from MCP server
///
/// This would connect to the actual MCP server to get tool definitions.
/// For testing, we'll parse from the mcp_impl.rs handlers.
pub fn load_mcp_tool_definitions() -> Result<Vec<McpToolDefinition>> {
    // Returns hardcoded tool definitions based on PMAT-6017, PMAT-6019, etc.

    let tools = vec![
        McpToolDefinition {
            name: "scaffold_agent".to_string(),
            description: "Scaffold a deterministic MCP agent with complete project structure, tests, and documentation".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Agent project name (lowercase, alphanumeric, hyphens only)"
                    },
                    "template": {
                        "type": "string",
                        "description": "Template type: mcp-server (basic), state-machine (stateful), hybrid (deterministic + LLM), calculator (math example)",
                        "default": "mcp-server"
                    },
                    "output_dir": {
                        "type": "string",
                        "description": "Output directory where the agent project will be created (default: current directory)"
                    },
                    "quality_level": {
                        "type": "string",
                        "description": "Quality level: standard (fast basic scaffolding), high (thorough with tests), extreme (comprehensive with ML and mutation testing)",
                        "default": "standard"
                    },
                    "features": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Features to include: testing, docs, ci, mutation, property-testing, tui, http-server (comma-separated, default: empty array)"
                    }
                },
                "required": ["name"]
            }),
        },
        McpToolDefinition {
            name: "validate_roadmap".to_string(),
            description: "Validate ROADMAP.md structure and check that all tickets referenced in roadmap exist as files".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "roadmap_path": {
                        "type": "string",
                        "description": "Path to ROADMAP.md file for validation (default: ./ROADMAP.md in project root)"
                    },
                    "tickets_dir": {
                        "type": "string",
                        "description": "Directory containing ticket files (default: ./docs/tickets)"
                    }
                },
                "required": []
            }),
        },
        McpToolDefinition {
            name: "health_check".to_string(),
            description: "Run comprehensive project health checks including build, tests, coverage, complexity, and technical debt analysis".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_dir": {
                        "type": "string",
                        "description": "Path to project directory to analyze (default: current directory)"
                    },
                    "quick": {
                        "type": "boolean",
                        "description": "Quick mode: run only fast checks (build only, skip tests/coverage/analysis)",
                        "default": false
                    },
                    "check_build": {
                        "type": "boolean",
                        "description": "Check if project builds successfully (cargo check/build)",
                        "default": true
                    },
                    "check_tests": {
                        "type": "boolean",
                        "description": "Run test suite and verify all tests pass",
                        "default": true
                    },
                    "check_coverage": {
                        "type": "boolean",
                        "description": "Measure code coverage and verify meets threshold (default: 70%)",
                        "default": true
                    },
                    "check_complexity": {
                        "type": "boolean",
                        "description": "Analyze code complexity and flag violations (cyclomatic >8, cognitive >15)",
                        "default": true
                    },
                    "check_satd": {
                        "type": "boolean",
                        "description": "Scan for Self-Admitted Technical Debt (TODO, FIXME, HACK annotations)",
                        "default": true
                    }
                },
                "required": []
            }),
        },
        McpToolDefinition {
            name: "generate_tickets".to_string(),
            description: "Generate ticket files from ROADMAP.md entries that don't have corresponding ticket files yet".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "roadmap_path": {
                        "type": "string",
                        "description": "Path to ROADMAP.md file containing ticket list (default: ./ROADMAP.md)"
                    },
                    "tickets_dir": {
                        "type": "string",
                        "description": "Directory where ticket files should be created (default: ./docs/tickets)"
                    },
                    "dry_run": {
                        "type": "boolean",
                        "description": "Dry-run mode: show what would be generated without creating files (preview only)",
                        "default": false
                    }
                },
                "required": []
            }),
        },
    ];

    Ok(tools)
}
