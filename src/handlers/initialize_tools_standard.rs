/// Standard analysis tools: churn, complexity, DAG, context, dead code, deep context.
fn standard_analysis_tool_definitions() -> Vec<serde_json::Value> {
    vec![
        json!({
            "name": "analyze_code_churn",
            "description": "Analyze code change frequency and patterns to identify maintenance hotspots. Uses git history to find frequently changed files.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to analyze (defaults to current directory)"
                    },
                    "period_days": {
                        "type": "integer",
                        "description": "Number of days to analyze (default: 30)"
                    },
                    "format": {
                        "type": "string",
                        "enum": ["json", "markdown", "csv", "summary"],
                        "description": "Output format (default: summary)"
                    }
                }
            }
        }),
        json!({
            "name": "analyze_complexity",
            "description": "Analyze code complexity using McCabe Cyclomatic and Sonar Cognitive algorithms. Supports multiple output formats including SARIF for IDE integration.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to the project to analyze (defaults to current directory)"
                    },
                    "toolchain": {
                        "type": "string",
                        "description": "Toolchain to use (rust, deno, python-uv). Auto-detected if not specified"
                    },
                    "format": {
                        "type": "string",
                        "enum": ["summary", "full", "json", "sarif"],
                        "description": "Output format (default: summary)"
                    },
                    "max_cyclomatic": {
                        "type": "integer",
                        "description": "Custom cyclomatic complexity threshold"
                    },
                    "max_cognitive": {
                        "type": "integer",
                        "description": "Custom cognitive complexity threshold"
                    },
                    "include": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "File patterns to include in analysis"
                    }
                }
            }
        }),
        json!({
            "name": "analyze_dag",
            "description": "Generate dependency graphs in Mermaid format for visualizing code structure and dependencies",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to analyze (defaults to current directory)"
                    },
                    "dag_type": {
                        "type": "string",
                        "enum": ["call-graph", "import-graph", "inheritance", "full-dependency"],
                        "description": "Type of graph to generate (default: call-graph)"
                    },
                    "max_depth": {
                        "type": "integer",
                        "description": "Maximum depth for graph traversal"
                    },
                    "filter_external": {
                        "type": "boolean",
                        "description": "Filter out external dependencies"
                    },
                    "show_complexity": {
                        "type": "boolean",
                        "description": "Include complexity metrics in the graph"
                    }
                }
            }
        }),
        json!({
            "name": "generate_context",
            "description": "Generate project context using Abstract Syntax Tree (AST) analysis. Features persistent caching for improved performance.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "toolchain": {
                        "type": "string",
                        "enum": ["rust", "deno", "python-uv"],
                        "description": "Target toolchain for analysis"
                    },
                    "project_path": {
                        "type": "string",
                        "description": "Path to analyze (defaults to current directory)"
                    },
                    "format": {
                        "type": "string",
                        "enum": ["markdown", "json"],
                        "description": "Output format (default: markdown)"
                    }
                },
                "required": ["toolchain"]
            }
        }),
        json!({
            "name": "analyze_dead_code",
            "description": "Analyze dead and unreachable code with ranking support. Identifies unused functions, classes, variables, and unreachable code blocks using cross-reference analysis.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to analyze (defaults to current directory)"
                    },
                    "format": {
                        "type": "string",
                        "enum": ["summary", "json", "sarif", "markdown"],
                        "description": "Output format (default: summary)"
                    },
                    "top_files": {
                        "type": "integer",
                        "description": "Show top N files with most dead code (0 = show all files)"
                    },
                    "include_unreachable": {
                        "type": "boolean",
                        "description": "Include unreachable code blocks in analysis (default: false)"
                    },
                    "min_dead_lines": {
                        "type": "integer",
                        "description": "Minimum dead lines to report a file (default: 10)"
                    },
                    "include_tests": {
                        "type": "boolean",
                        "description": "Include test files in analysis (default: false)"
                    }
                }
            }
        }),
        json!({
            "name": "analyze_deep_context",
            "description": "Comprehensive deep context analysis combining AST analysis, complexity metrics, code churn detection, dead code analysis, and SATD detection into a unified quality assessment with defect correlation and prioritized recommendations.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_path": {
                        "type": "string",
                        "description": "Path to analyze (defaults to current directory)"
                    },
                    "format": {
                        "type": "string",
                        "enum": ["markdown", "json", "sarif"],
                        "description": "Output format (default: markdown)"
                    },
                    "include_analyses": {
                        "type": "array",
                        "items": {
                            "type": "string",
                            "enum": ["ast", "complexity", "churn", "dag", "dead_code", "satd", "defect_probability"]
                        },
                        "description": "Which analyses to include (default: ast, complexity, churn)"
                    },
                    "exclude_analyses": {
                        "type": "array",
                        "items": {
                            "type": "string",
                            "enum": ["ast", "complexity", "churn", "dag", "dead_code", "satd", "defect_probability"]
                        },
                        "description": "Which analyses to exclude"
                    },
                    "period_days": {
                        "type": "integer",
                        "description": "Number of days for churn analysis (default: 30)"
                    },
                    "dag_type": {
                        "type": "string",
                        "enum": ["call-graph", "import-graph", "inheritance", "full-dependency"],
                        "description": "Type of dependency graph to generate (default: call-graph)"
                    },
                    "max_depth": {
                        "type": "integer",
                        "description": "Maximum depth for graph traversal"
                    },
                    "include_pattern": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "File patterns to include in analysis"
                    },
                    "exclude_pattern": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "File patterns to exclude from analysis"
                    },
                    "cache_strategy": {
                        "type": "string",
                        "enum": ["normal", "force-refresh", "offline"],
                        "description": "Cache strategy for analysis (default: normal)"
                    },
                    "parallel": {
                        "type": "integer",
                        "description": "Number of parallel analysis workers (default: 4)"
                    }
                }
            }
        }),
    ]
}
