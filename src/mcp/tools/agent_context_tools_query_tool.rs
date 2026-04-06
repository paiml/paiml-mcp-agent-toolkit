// QueryCodeTool implementation
// Split from agent_context_tools.rs for maintainability

impl QueryCodeTool {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(manager: Arc<IndexManager>) -> Self {
        Self { manager }
    }

    pub fn schema() -> Value {
        json!({
            "name": "pmat_query_code",
            "description": "Search code functions by natural language query with TDG quality filtering. Returns functions matching the query ranked by relevance with quality annotations (grade, complexity, Big-O, SATD markers).",
            "parameters": {
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Natural language search query (e.g., 'error handling', 'parse JSON', 'validate user input')"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of results (default: 10, max: 100)",
                        "minimum": 1,
                        "maximum": 100,
                        "default": 10
                    },
                    "min_grade": {
                        "type": "string",
                        "enum": ["A", "B", "C", "D", "F"],
                        "description": "Minimum TDG grade filter (A is best)"
                    },
                    "max_complexity": {
                        "type": "integer",
                        "description": "Maximum cyclomatic complexity filter",
                        "minimum": 1,
                        "maximum": 100
                    },
                    "language": {
                        "type": "string",
                        "enum": ["rust", "typescript", "python", "go", "java", "c", "cpp"],
                        "description": "Filter by programming language"
                    },
                    "path_pattern": {
                        "type": "string",
                        "description": "File path pattern filter (e.g., 'src/handlers', 'tests/')"
                    },
                    "include_source": {
                        "type": "boolean",
                        "description": "Include full source code in results (default: false)",
                        "default": false
                    },
                    "rebuild_index": {
                        "type": "boolean",
                        "description": "Force rebuild the index before querying (default: false)",
                        "default": false
                    }
                },
                "required": ["query"]
            }
        })
    }
}

#[async_trait]
impl McpTool for QueryCodeTool {
    fn name(&self) -> &str {
        "pmat_query_code"
    }

    fn schema(&self) -> Value {
        Self::schema()
    }

    async fn execute(&self, params: Value) -> Result<Value, String> {
        let start = Instant::now();

        // Extract query parameter
        let query = params["query"]
            .as_str()
            .ok_or("Missing required parameter: query")?;

        if query.trim().is_empty() {
            return Err("Query cannot be empty".to_string());
        }

        // Extract optional parameters
        let limit = params["limit"].as_u64().unwrap_or(10) as usize;
        if limit > 100 {
            return Err("Limit exceeds maximum of 100".to_string());
        }

        let min_grade = params["min_grade"].as_str().map(|s| s.to_string());
        let max_complexity = params["max_complexity"].as_u64().map(|n| n as u32);
        let language = params["language"].as_str().map(|s| s.to_string());
        let path_pattern = params["path_pattern"].as_str().map(|s| s.to_string());
        let include_source = params["include_source"].as_bool().unwrap_or(false);
        let rebuild_index = params["rebuild_index"].as_bool().unwrap_or(false);

        // Get or rebuild index
        let index = if rebuild_index {
            self.manager.rebuild_index().await?
        } else {
            self.manager.get_index().await?
        };

        // Build query options
        let options = QueryOptions {
            limit,
            min_grade,
            max_complexity,
            max_loc: None,
            language,
            path_pattern,
            include_source,
            rank_by: Default::default(),
            min_pagerank: None,
            ..Default::default()
        };

        // Execute query
        let results = index.query(query, options)?;

        let query_time_ms = start.elapsed().as_millis() as u64;

        // Format results
        let results_json: Vec<Value> = results
            .iter()
            .map(|r| {
                let mut result = json!({
                    "id": format!("{}::{}", r.file_path, r.function_name),
                    "name": r.function_name,
                    "file_path": r.file_path,
                    "start_line": r.start_line,
                    "end_line": r.end_line,
                    "language": r.language,
                    "relevance_score": r.relevance_score,
                    "quality": {
                        "grade": r.tdg_grade,
                        "complexity": r.complexity,
                        "tdg_score": r.tdg_score,
                        "big_o": r.big_o,
                        "satd_count": r.satd_count,
                        "loc": r.loc
                    },
                    "signature": r.signature
                });

                if let Some(doc) = &r.doc_comment {
                    result["doc_comment"] = json!(doc);
                }

                if let Some(source) = &r.source {
                    result["source"] = json!(source);
                }

                if !r.calls.is_empty() {
                    result["calls"] = json!(r.calls);
                }

                if !r.called_by.is_empty() {
                    result["called_by"] = json!(r.called_by);
                }

                result
            })
            .collect();

        let manifest = index.manifest();

        Ok(json!({
            "results": results_json,
            "total": results.len(),
            "query_time_ms": query_time_ms,
            "index_stats": {
                "function_count": manifest.function_count,
                "file_count": manifest.file_count,
                "avg_tdg_score": manifest.avg_tdg_score
            }
        }))
    }
}
