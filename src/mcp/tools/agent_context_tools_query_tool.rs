// QueryCodeTool implementation
// Split from agent_context_tools.rs for maintainability

impl QueryCodeTool {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(manager: Arc<IndexManager>) -> Self {
        Self { manager }
    }

    /// Schema.
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
                    // The enum used to list five letters while the filter
                    // underneath accepted all eleven grades case-insensitively,
                    // so `A-` and `a` worked but were undocumented, and `Z`
                    // was neither documented nor rejected — it just returned
                    // `total: 0`. Schema and validator now name the same set.
                    "min_grade": {
                        "type": "string",
                        "enum": min_grade_enum(),
                        "description": "Minimum TDG grade filter (A+ is best). Case-insensitive."
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

    async fn execute(&self, params: Value) -> Result<Value, ToolError> {
        let start = Instant::now();

        // Extract query parameter
        let query = params["query"]
            .as_str()
            .ok_or_else(|| ToolError::invalid("Missing required parameter: query"))?;

        if query.trim().is_empty() {
            return Err(ToolError::invalid("Query cannot be empty"));
        }

        // Extract optional parameters. Every bound below is the one this tool's
        // own schema advertises, enforced at BOTH ends: `limit: 9999` was
        // already refused while `limit: -1` and `limit: "10"` silently became
        // the default 10, so a typo was indistinguishable from an intended
        // value — the exact hole the upper bound was added to close.
        let limit = bounded_integer(&params, "limit", 1, 100)?.unwrap_or(10) as usize;

        let min_grade = string(&params, "min_grade")?
            .map(validate_min_grade)
            .transpose()?;
        // A silently dropped `max_complexity` is worse than a silently defaulted
        // `limit`: the caller asked for a FILTER and got unfiltered results.
        let max_complexity = bounded_integer(&params, "max_complexity", 1, 100)?.map(|n| n as u32);
        let language = string(&params, "language")?.map(std::string::ToString::to_string);
        let path_pattern = string(&params, "path_pattern")?.map(std::string::ToString::to_string);
        let include_source = boolean(&params, "include_source")?.unwrap_or(false);
        let rebuild_index = boolean(&params, "rebuild_index")?.unwrap_or(false);

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
