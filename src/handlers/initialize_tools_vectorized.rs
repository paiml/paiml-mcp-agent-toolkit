/// Vectorized SIMD-accelerated tool definitions.
fn vectorized_tool_definitions() -> Vec<serde_json::Value> {
    vec![
        json!({
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
        }),
        json!({
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
        }),
        json!({
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
        }),
        json!({
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
        }),
        json!({
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
        }),
        json!({
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
        }),
    ]
}
