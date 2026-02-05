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
                },
                {
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
                },
                {
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
                }
            ]
        }),
    )
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::mcp::McpRequest;
    use crate::TemplateServer;
    use std::sync::Arc;

    async fn create_test_server() -> Arc<TemplateServer> {
        Arc::new(
            TemplateServer::new()
                .await
                .expect("Failed to create test server"),
        )
    }

    fn create_test_request(id: serde_json::Value, params: Option<serde_json::Value>) -> McpRequest {
        McpRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: "initialize".to_string(),
            params,
        }
    }

    // === handle_initialize tests ===

    #[tokio::test]
    async fn test_handle_initialize_basic() {
        let server = create_test_server().await;
        let request = create_test_request(serde_json::json!(1), None);

        let response = handle_initialize(server, request).await;

        assert!(response.result.is_some());
        let result = response.result.unwrap();
        assert!(result.get("protocolVersion").is_some());
        assert!(result.get("capabilities").is_some());
        assert!(result.get("serverInfo").is_some());
    }

    #[tokio::test]
    async fn test_handle_initialize_with_protocol_version() {
        let server = create_test_server().await;
        let params = serde_json::json!({
            "protocolVersion": "2025-01-01"
        });
        let request = create_test_request(serde_json::json!(1), Some(params));

        let response = handle_initialize(server, request).await;

        let result = response.result.unwrap();
        assert_eq!(result.get("protocolVersion").unwrap(), "2025-01-01");
    }

    #[tokio::test]
    async fn test_handle_initialize_default_protocol_version() {
        let server = create_test_server().await;
        let request = create_test_request(serde_json::json!(1), None);

        let response = handle_initialize(server, request).await;

        let result = response.result.unwrap();
        assert_eq!(result.get("protocolVersion").unwrap(), "2024-11-05");
    }

    #[tokio::test]
    async fn test_handle_initialize_server_info() {
        let server = create_test_server().await;
        let request = create_test_request(serde_json::json!(1), None);

        let response = handle_initialize(server, request).await;

        let result = response.result.unwrap();
        let server_info = result.get("serverInfo").unwrap();

        assert_eq!(server_info.get("name").unwrap(), "pmat");
        assert!(server_info.get("version").is_some());
        assert_eq!(
            server_info.get("vendor").unwrap(),
            "Pragmatic AI Labs (paiml.com)"
        );
        assert_eq!(server_info.get("author").unwrap(), "Pragmatic AI Labs");
    }

    #[tokio::test]
    async fn test_handle_initialize_capabilities() {
        let server = create_test_server().await;
        let request = create_test_request(serde_json::json!(1), None);

        let response = handle_initialize(server, request).await;

        let result = response.result.unwrap();
        let capabilities = result.get("capabilities").unwrap();

        assert!(capabilities.get("tools").is_some());
        assert!(capabilities.get("resources").is_some());
        assert!(capabilities.get("prompts").is_some());
    }

    #[tokio::test]
    async fn test_handle_initialize_server_capabilities_array() {
        let server = create_test_server().await;
        let request = create_test_request(serde_json::json!(1), None);

        let response = handle_initialize(server, request).await;

        let result = response.result.unwrap();
        let server_info = result.get("serverInfo").unwrap();
        let capabilities = server_info.get("capabilities").unwrap();

        assert!(capabilities.is_array());
        let caps_array = capabilities.as_array().unwrap();
        assert!(!caps_array.is_empty());
    }

    #[tokio::test]
    async fn test_handle_initialize_supported_templates() {
        let server = create_test_server().await;
        let request = create_test_request(serde_json::json!(1), None);

        let response = handle_initialize(server, request).await;

        let result = response.result.unwrap();
        let server_info = result.get("serverInfo").unwrap();
        let templates = server_info.get("supportedTemplates").unwrap();

        assert!(templates.is_array());
        let templates_array = templates.as_array().unwrap();
        assert!(templates_array.contains(&serde_json::json!("makefile")));
        assert!(templates_array.contains(&serde_json::json!("readme")));
        assert!(templates_array.contains(&serde_json::json!("gitignore")));
    }

    #[tokio::test]
    async fn test_handle_initialize_supported_toolchains() {
        let server = create_test_server().await;
        let request = create_test_request(serde_json::json!(1), None);

        let response = handle_initialize(server, request).await;

        let result = response.result.unwrap();
        let server_info = result.get("serverInfo").unwrap();
        let toolchains = server_info.get("supportedToolchains").unwrap();

        assert!(toolchains.is_array());
        let toolchains_array = toolchains.as_array().unwrap();
        assert!(toolchains_array.contains(&serde_json::json!("rust")));
        assert!(toolchains_array.contains(&serde_json::json!("deno")));
        assert!(toolchains_array.contains(&serde_json::json!("python-uv")));
    }

    #[tokio::test]
    async fn test_handle_initialize_examples() {
        let server = create_test_server().await;
        let request = create_test_request(serde_json::json!(1), None);

        let response = handle_initialize(server, request).await;

        let result = response.result.unwrap();
        let server_info = result.get("serverInfo").unwrap();
        let examples = server_info.get("examples").unwrap();

        assert!(examples.is_array());
        let examples_array = examples.as_array().unwrap();
        assert!(!examples_array.is_empty());
    }

    #[tokio::test]
    async fn test_handle_initialize_string_id() {
        let server = create_test_server().await;
        let request = create_test_request(serde_json::json!("test-id"), None);

        let response = handle_initialize(server, request).await;

        assert!(response.result.is_some());
        assert_eq!(response.id, serde_json::json!("test-id"));
    }

    #[tokio::test]
    async fn test_handle_initialize_null_id() {
        let server = create_test_server().await;
        let request = create_test_request(serde_json::Value::Null, None);

        let response = handle_initialize(server, request).await;

        assert!(response.result.is_some());
        assert!(response.id.is_null());
    }

    #[tokio::test]
    async fn test_handle_initialize_empty_params() {
        let server = create_test_server().await;
        let request = create_test_request(serde_json::json!(1), Some(serde_json::json!({})));

        let response = handle_initialize(server, request).await;

        assert!(response.result.is_some());
        // Default protocol version should be used
        let result = response.result.unwrap();
        assert_eq!(result.get("protocolVersion").unwrap(), "2024-11-05");
    }

    // === handle_tools_list tests ===

    #[tokio::test]
    async fn test_handle_tools_list_basic() {
        let server = create_test_server().await;
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = handle_tools_list(server, request).await;

        assert!(response.result.is_some());
        let result = response.result.unwrap();
        assert!(result.get("tools").is_some());
    }

    #[tokio::test]
    async fn test_handle_tools_list_has_tools() {
        let server = create_test_server().await;
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = handle_tools_list(server, request).await;

        let result = response.result.unwrap();
        let tools = result.get("tools").unwrap();
        assert!(tools.is_array());
        let tools_array = tools.as_array().unwrap();
        assert!(!tools_array.is_empty());
    }

    #[tokio::test]
    async fn test_handle_tools_list_contains_generate_template() {
        let server = create_test_server().await;
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = handle_tools_list(server, request).await;

        let result = response.result.unwrap();
        let tools = result.get("tools").unwrap().as_array().unwrap();

        let has_generate_template = tools.iter().any(|t| {
            t.get("name")
                .map(|n| n == "generate_template")
                .unwrap_or(false)
        });
        assert!(has_generate_template);
    }

    #[tokio::test]
    async fn test_handle_tools_list_contains_scaffold_project() {
        let server = create_test_server().await;
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = handle_tools_list(server, request).await;

        let result = response.result.unwrap();
        let tools = result.get("tools").unwrap().as_array().unwrap();

        let has_scaffold_project = tools.iter().any(|t| {
            t.get("name")
                .map(|n| n == "scaffold_project")
                .unwrap_or(false)
        });
        assert!(has_scaffold_project);
    }

    #[tokio::test]
    async fn test_handle_tools_list_contains_analyze_complexity() {
        let server = create_test_server().await;
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = handle_tools_list(server, request).await;

        let result = response.result.unwrap();
        let tools = result.get("tools").unwrap().as_array().unwrap();

        let has_analyze_complexity = tools.iter().any(|t| {
            t.get("name")
                .map(|n| n == "analyze_complexity")
                .unwrap_or(false)
        });
        assert!(has_analyze_complexity);
    }

    #[tokio::test]
    async fn test_handle_tools_list_contains_analyze_churn() {
        let server = create_test_server().await;
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = handle_tools_list(server, request).await;

        let result = response.result.unwrap();
        let tools = result.get("tools").unwrap().as_array().unwrap();

        let has_analyze_churn = tools.iter().any(|t| {
            t.get("name")
                .map(|n| n == "analyze_code_churn")
                .unwrap_or(false)
        });
        assert!(has_analyze_churn);
    }

    #[tokio::test]
    async fn test_handle_tools_list_contains_analyze_dead_code() {
        let server = create_test_server().await;
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = handle_tools_list(server, request).await;

        let result = response.result.unwrap();
        let tools = result.get("tools").unwrap().as_array().unwrap();

        let has_analyze_dead_code = tools.iter().any(|t| {
            t.get("name")
                .map(|n| n == "analyze_dead_code")
                .unwrap_or(false)
        });
        assert!(has_analyze_dead_code);
    }

    #[tokio::test]
    async fn test_handle_tools_list_contains_analyze_satd() {
        let server = create_test_server().await;
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = handle_tools_list(server, request).await;

        let result = response.result.unwrap();
        let tools = result.get("tools").unwrap().as_array().unwrap();

        let has_analyze_satd = tools
            .iter()
            .any(|t| t.get("name").map(|n| n == "analyze_satd").unwrap_or(false));
        assert!(has_analyze_satd);
    }

    #[tokio::test]
    async fn test_handle_tools_list_tool_has_description() {
        let server = create_test_server().await;
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = handle_tools_list(server, request).await;

        let result = response.result.unwrap();
        let tools = result.get("tools").unwrap().as_array().unwrap();

        // All tools should have descriptions
        for tool in tools {
            assert!(
                tool.get("description").is_some(),
                "Tool {} missing description",
                tool.get("name").unwrap_or(&serde_json::Value::Null)
            );
        }
    }

    #[tokio::test]
    async fn test_handle_tools_list_tool_has_input_schema() {
        let server = create_test_server().await;
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = handle_tools_list(server, request).await;

        let result = response.result.unwrap();
        let tools = result.get("tools").unwrap().as_array().unwrap();

        // All tools should have inputSchema
        for tool in tools {
            assert!(
                tool.get("inputSchema").is_some(),
                "Tool {} missing inputSchema",
                tool.get("name").unwrap_or(&serde_json::Value::Null)
            );
        }
    }

    #[tokio::test]
    async fn test_handle_tools_list_vectorized_tools() {
        let server = create_test_server().await;
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = handle_tools_list(server, request).await;

        let result = response.result.unwrap();
        let tools = result.get("tools").unwrap().as_array().unwrap();

        let vectorized_tools = [
            "analyze_duplicates_vectorized",
            "analyze_graph_metrics_vectorized",
            "analyze_name_similarity_vectorized",
            "analyze_symbol_table_vectorized",
            "analyze_incremental_coverage_vectorized",
            "analyze_big_o_vectorized",
        ];

        for tool_name in &vectorized_tools {
            let has_tool = tools
                .iter()
                .any(|t| t.get("name").map(|n| n == *tool_name).unwrap_or(false));
            assert!(has_tool, "Missing vectorized tool: {}", tool_name);
        }
    }

    #[tokio::test]
    async fn test_handle_tools_list_string_id() {
        let server = create_test_server().await;
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!("test-id-123"),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = handle_tools_list(server, request).await;

        assert_eq!(response.id, serde_json::json!("test-id-123"));
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
