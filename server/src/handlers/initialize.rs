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
                "name": "pmat",
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
                    "Generate just a Makefile: generate_template with resource_uri='template://makefile/rust/cli'",
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
                                "description": "Template URI (e.g., template://makefile/rust/cli)"
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
                },
                {
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
                },
                {
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
                },
                {
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
                },
                {
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
                },
                {
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
                },
                {
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
                },
                // Vectorized tools
                {
                    "name": "analyze_duplicates_vectorized",
                    "description": "High-performance duplicate code detection using SIMD operations",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "project_path": {
                                "type": "string",
                                "description": "Path to the project to analyze"
                            },
                            "detection_type": {
                                "type": "string",
                                "enum": ["exact", "token", "semantic"],
                                "description": "Type of duplicate detection"
                            },
                            "threshold": {
                                "type": "number",
                                "description": "Similarity threshold (0.0-1.0)"
                            },
                            "parallel_threads": {
                                "type": "integer",
                                "description": "Number of parallel threads to use"
                            },
                            "use_simd": {
                                "type": "boolean",
                                "description": "Enable SIMD optimizations"
                            }
                        },
                        "required": ["project_path"]
                    }
                },
                {
                    "name": "analyze_graph_metrics_vectorized",
                    "description": "Compute graph centrality metrics using vectorized algorithms",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "project_path": {
                                "type": "string",
                                "description": "Path to the project to analyze"
                            },
                            "metrics": {
                                "type": "array",
                                "items": {
                                    "type": "string",
                                    "enum": ["pagerank", "betweenness", "closeness", "degree"]
                                },
                                "description": "Metrics to compute"
                            },
                            "use_gpu": {
                                "type": "boolean",
                                "description": "Enable GPU acceleration if available"
                            }
                        },
                        "required": ["project_path"]
                    }
                },
                {
                    "name": "analyze_name_similarity_vectorized",
                    "description": "Fast identifier similarity search using SIMD string operations",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "project_path": {
                                "type": "string",
                                "description": "Path to the project to analyze"
                            },
                            "query": {
                                "type": "string",
                                "description": "Name to search for"
                            },
                            "top_k": {
                                "type": "integer",
                                "description": "Number of top matches to return"
                            },
                            "use_simd": {
                                "type": "boolean",
                                "description": "Enable SIMD optimizations"
                            }
                        },
                        "required": ["project_path", "query"]
                    }
                },
                {
                    "name": "analyze_symbol_table_vectorized",
                    "description": "Build and analyze symbol tables with parallel parsing",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "project_path": {
                                "type": "string",
                                "description": "Path to the project to analyze"
                            },
                            "parallel_parsing": {
                                "type": "boolean",
                                "description": "Enable parallel file parsing"
                            }
                        },
                        "required": ["project_path"]
                    }
                },
                {
                    "name": "analyze_incremental_coverage_vectorized",
                    "description": "Compute coverage changes with vectorized diff operations",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "project_path": {
                                "type": "string",
                                "description": "Path to the project to analyze"
                            },
                            "base_branch": {
                                "type": "string",
                                "description": "Base branch for comparison"
                            },
                            "parallel_diff": {
                                "type": "boolean",
                                "description": "Enable parallel diff computation"
                            }
                        },
                        "required": ["project_path"]
                    }
                },
                {
                    "name": "analyze_big_o_vectorized",
                    "description": "Analyze algorithmic complexity using parallel pattern matching",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "project_path": {
                                "type": "string",
                                "description": "Path to the project to analyze"
                            },
                            "parallel_analysis": {
                                "type": "boolean",
                                "description": "Enable parallel function analysis"
                            }
                        },
                        "required": ["project_path"]
                    }
                },
                {
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
                }
            ]
        }),
    )
}
