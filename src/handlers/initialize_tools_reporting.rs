/// Reporting, SATD, and lint hotspot tool definitions.
fn reporting_tool_definitions() -> Vec<serde_json::Value> {
    vec![
        json!({
            "name": "generate_enhanced_report",
            "description": "Generate comprehensive analysis reports with visualizations",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to the project to analyze"
                    },
                    "output_format": {
                        "type": "string",
                        "enum": ["html", "markdown", "json", "pdf"],
                        "description": "Output format for the report"
                    },
                    "analyses": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "Analyses to include in the report"
                    }
                },
                "required": ["project_path"]
            }
        }),
        json!({
            "name": "analyze_satd",
            "description": "Analyze Self-Admitted Technical Debt (SATD) in source code. Detects TODO, FIXME, HACK, and other technical debt markers with categorization and severity assessment.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to analyze (defaults to current directory)"
                    },
                    "strict": {
                        "type": "boolean",
                        "description": "Use strict mode (only detect explicit SATD markers with colons)"
                    },
                    "exclude_tests": {
                        "type": "boolean",
                        "description": "Exclude test files from analysis (default: true)"
                    },
                    "critical_only": {
                        "type": "boolean",
                        "description": "Show only critical technical debt items"
                    },
                    "format": {
                        "type": "string",
                        "enum": ["summary", "json", "sarif", "markdown"],
                        "description": "Output format (default: summary)"
                    }
                }
            }
        }),
        json!({
            "name": "analyze_lint_hotspot",
            "description": "Find files with highest lint violation density (defects per line of code). Identifies code quality hotspots that need immediate attention.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to analyze (defaults to current directory)"
                    },
                    "top_files": {
                        "type": "integer",
                        "description": "Number of top files to show (default: 10)"
                    },
                    "min_violations": {
                        "type": "integer",
                        "description": "Minimum violations to include file (default: 1)"
                    },
                    "include": {
                        "type": "string",
                        "description": "Include patterns (comma-separated)"
                    },
                    "exclude": {
                        "type": "string",
                        "description": "Exclude patterns (comma-separated)"
                    },
                    "format": {
                        "type": "string",
                        "enum": ["table", "json", "csv"],
                        "description": "Output format (default: table)"
                    }
                }
            }
        }),
    ]
}
