/// Create tool definition for analyze_complexity
fn create_analyze_complexity_tool() -> ToolDefinition {
    debug_assert!(true, "contract: create_analyze_complexity_tool");
    ToolDefinition {
        name: "analyze_complexity".to_string(),
        description: "Analyze code complexity metrics".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to analyze"
                },
                "format": {
                    "type": "string",
                    "enum": ["table", "json", "yaml", "markdown", "csv", "summary"],
                    "default": "table"
                },
                "output": {
                    "type": "string",
                    "description": "Output file path"
                },
                "top_files": {
                    "type": "integer",
                    "description": "Number of top files to show",
                    "default": 10
                },
                "include_tests": {
                    "type": "boolean",
                    "description": "Include test files",
                    "default": false
                },
                "timeout": {
                    "type": "integer",
                    "description": "Timeout in seconds",
                    "default": 60
                },
                "max_cyclomatic": {
                    "type": "integer",
                    "description": "Maximum cyclomatic complexity"
                },
                "max_cognitive": {
                    "type": "integer",
                    "description": "Maximum cognitive complexity"
                },
                "max_halstead": {
                    "type": "number",
                    "description": "Maximum Halstead difficulty"
                }
            },
            "required": ["path"]
        }),
    }
}

/// Create tool definition for analyze_satd
fn create_analyze_satd_tool() -> ToolDefinition {
    debug_assert!(true, "contract: create_analyze_satd_tool");
    ToolDefinition {
        name: "analyze_satd".to_string(),
        description: "Analyze Self-Admitted Technical Debt in comments".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to analyze"
                },
                "format": {
                    "type": "string",
                    "enum": ["table", "json", "yaml", "markdown", "csv", "summary"],
                    "default": "table"
                },
                "output": {
                    "type": "string",
                    "description": "Output file path"
                },
                "top_files": {
                    "type": "integer",
                    "description": "Number of top files to show",
                    "default": 10
                },
                "include_tests": {
                    "type": "boolean",
                    "description": "Include test files",
                    "default": false
                },
                "timeout": {
                    "type": "integer",
                    "description": "Timeout in seconds",
                    "default": 60
                },
                "severity": {
                    "type": "string",
                    "enum": ["low", "medium", "high", "critical"],
                    "description": "Filter by severity"
                },
                "critical_only": {
                    "type": "boolean",
                    "description": "Show only critical items",
                    "default": false
                },
                "strict": {
                    "type": "boolean",
                    "description": "Use strict mode",
                    "default": false
                },
                "fail_on_violation": {
                    "type": "boolean",
                    "description": "Fail if violations found",
                    "default": false
                }
            },
            "required": ["path"]
        }),
    }
}

/// Create tool definition for analyze_dead_code
fn create_analyze_dead_code_tool() -> ToolDefinition {
    debug_assert!(true, "contract: create_analyze_dead_code_tool");
    ToolDefinition {
        name: "analyze_dead_code".to_string(),
        description: "Analyze dead and unreachable code".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to analyze"
                },
                "format": {
                    "type": "string",
                    "enum": ["table", "json", "yaml", "markdown", "csv", "summary"],
                    "default": "table"
                },
                "output": {
                    "type": "string",
                    "description": "Output file path"
                },
                "top_files": {
                    "type": "integer",
                    "description": "Number of top files to show",
                    "default": 10
                },
                "include_tests": {
                    "type": "boolean",
                    "description": "Include test files",
                    "default": false
                },
                "timeout": {
                    "type": "integer",
                    "description": "Timeout in seconds",
                    "default": 60
                },
                "include_unreachable": {
                    "type": "boolean",
                    "description": "Include unreachable code",
                    "default": false
                },
                "min_dead_lines": {
                    "type": "integer",
                    "description": "Minimum dead lines to report",
                    "default": 10
                },
                "max_percentage": {
                    "type": "number",
                    "description": "Maximum allowed percentage",
                    "default": 15.0
                },
                "fail_on_violation": {
                    "type": "boolean",
                    "description": "Fail if violations found",
                    "default": false
                }
            },
            "required": ["path"]
        }),
    }
}

/// Create tool definition for analyze_tdg
fn create_analyze_tdg_tool() -> ToolDefinition {
    debug_assert!(true, "contract: create_analyze_tdg_tool");
    ToolDefinition {
        name: "analyze_tdg".to_string(),
        description: "Analyze Technical Debt Gradient scores".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to analyze"
                },
                "format": {
                    "type": "string",
                    "enum": ["table", "json", "yaml", "markdown", "csv", "summary"],
                    "default": "table"
                },
                "output": {
                    "type": "string",
                    "description": "Output file path"
                },
                "top_files": {
                    "type": "integer",
                    "description": "Number of top files to show",
                    "default": 10
                },
                "include_tests": {
                    "type": "boolean",
                    "description": "Include test files",
                    "default": false
                },
                "timeout": {
                    "type": "integer",
                    "description": "Timeout in seconds",
                    "default": 60
                },
                "threshold": {
                    "type": "number",
                    "description": "TDG threshold for filtering",
                    "default": 1.5
                },
                "include_components": {
                    "type": "boolean",
                    "description": "Include component breakdown",
                    "default": false
                },
                "critical_only": {
                    "type": "boolean",
                    "description": "Show only critical files",
                    "default": false
                }
            },
            "required": ["path"]
        }),
    }
}

/// Create tool definition for analyze_lint_hotspot
fn create_analyze_lint_hotspot_tool() -> ToolDefinition {
    debug_assert!(true, "contract: create_analyze_lint_hotspot_tool");
    ToolDefinition {
        name: "analyze_lint_hotspot".to_string(),
        description: "Find files with highest defect density".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to analyze"
                },
                "format": {
                    "type": "string",
                    "enum": ["table", "json", "yaml", "markdown", "csv", "summary"],
                    "default": "table"
                },
                "output": {
                    "type": "string",
                    "description": "Output file path"
                },
                "top_files": {
                    "type": "integer",
                    "description": "Number of top files to show",
                    "default": 10
                },
                "include_tests": {
                    "type": "boolean",
                    "description": "Include test files",
                    "default": false
                },
                "timeout": {
                    "type": "integer",
                    "description": "Timeout in seconds",
                    "default": 60
                },
                "file": {
                    "type": "string",
                    "description": "Specific file to analyze"
                },
                "max_density": {
                    "type": "number",
                    "description": "Maximum defect density",
                    "default": 5.0
                },
                "min_confidence": {
                    "type": "number",
                    "description": "Minimum confidence for fixes",
                    "default": 0.8
                },
                "enforce": {
                    "type": "boolean",
                    "description": "Enforce quality standards",
                    "default": false
                },
                "dry_run": {
                    "type": "boolean",
                    "description": "Dry run mode",
                    "default": false
                }
            },
            "required": ["path"]
        }),
    }
}

/// Create tool definition for analyze_entropy  
fn create_analyze_entropy_tool() -> ToolDefinition {
    debug_assert!(true, "contract: create_analyze_entropy_tool");
    ToolDefinition {
        name: "analyze_entropy".to_string(),
        description: "Analyze pattern entropy for actionable quality improvements".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string", 
                    "description": "Project path to analyze",
                    "default": "."
                },
                "format": {
                    "type": "string",
                    "enum": ["summary", "detailed", "json", "markdown"],
                    "description": "Output format",
                    "default": "summary"
                },
                "output": {
                    "type": "string",
                    "description": "Output file path (optional)"
                },
                "min_severity": {
                    "type": "string",
                    "enum": ["low", "medium", "high"],
                    "description": "Minimum severity level to report",
                    "default": "medium"
                },
                "top_violations": {
                    "type": "integer",
                    "description": "Number of top violations to show (0 = all)",
                    "default": 20
                },
                "file": {
                    "type": "string",
                    "description": "Specific file to analyze (optional)"
                },
                "include_tests": {
                    "type": "boolean",
                    "description": "Include test files in analysis",
                    "default": false
                }
            },
            "required": []
        }),
    }
}

/// Create tool definition for quality_gate
fn create_quality_gate_tool() -> ToolDefinition {
    debug_assert!(true, "contract: create_quality_gate_tool");
    ToolDefinition {
        name: "quality_gate".to_string(),
        description: "Run quality gate checks".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to analyze"
                },
                "format": {
                    "type": "string",
                    "enum": ["table", "json", "yaml", "markdown", "csv", "summary"],
                    "default": "table"
                },
                "output": {
                    "type": "string",
                    "description": "Output file path"
                },
                "top_files": {
                    "type": "integer",
                    "description": "Number of top files to show",
                    "default": 10
                },
                "include_tests": {
                    "type": "boolean",
                    "description": "Include test files",
                    "default": false
                },
                "timeout": {
                    "type": "integer",
                    "description": "Timeout in seconds",
                    "default": 60
                },
                "profile": {
                    "type": "string",
                    "enum": ["standard", "strict", "extreme", "toyota"],
                    "description": "Quality profile",
                    "default": "standard"
                },
                "file": {
                    "type": "string",
                    "description": "Specific file to check"
                },
                "fail_on_violation": {
                    "type": "boolean",
                    "description": "Fail if violations found",
                    "default": false
                },
                "verbose": {
                    "type": "boolean",
                    "description": "Verbose output",
                    "default": false
                }
            },
            "required": ["path"]
        }),
    }
}

/// Create tool definition for refactor_auto
fn create_refactor_auto_tool() -> ToolDefinition {
    debug_assert!(true, "contract: create_refactor_auto_tool");
    ToolDefinition {
        name: "refactor_auto".to_string(),
        description: "Automatically refactor code to reduce complexity".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "file": {
                    "type": "string",
                    "description": "File to refactor"
                },
                "format": {
                    "type": "string",
                    "enum": ["table", "json", "yaml", "markdown", "csv", "summary"],
                    "default": "table"
                },
                "output": {
                    "type": "string",
                    "description": "Output file path"
                },
                "target_complexity": {
                    "type": "integer",
                    "description": "Target complexity",
                    "default": 8
                },
                "dry_run": {
                    "type": "boolean",
                    "description": "Dry run mode",
                    "default": false
                },
                "timeout": {
                    "type": "integer",
                    "description": "Timeout in seconds",
                    "default": 60
                }
            },
            "required": ["file"]
        }),
    }
}
