// GetFunctionTool, FindSimilarTool, IndexStatsTool implementations
// Split from agent_context_tools.rs for maintainability

// ============================================================================
// GetFunctionTool
// ============================================================================

impl GetFunctionTool {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(manager: Arc<IndexManager>) -> Self {
        Self { manager }
    }

    pub fn schema() -> Value {
        json!({
            "name": "pmat_get_function",
            "description": "Get detailed information about a specific function by its ID. Returns full function metadata including source code, quality metrics, and SATD markers.",
            "parameters": {
                "type": "object",
                "properties": {
                    "function_id": {
                        "type": "string",
                        "description": "Function ID from pmat_query_code results (e.g., 'src/handlers/auth.rs::handle_login')"
                    },
                    "include_source": {
                        "type": "boolean",
                        "description": "Include full source code (default: true)",
                        "default": true
                    }
                },
                "required": ["function_id"]
            }
        })
    }
}

#[async_trait]
impl McpTool for GetFunctionTool {
    fn name(&self) -> &str {
        "pmat_get_function"
    }

    fn schema(&self) -> Value {
        Self::schema()
    }

    async fn execute(&self, params: Value) -> Result<Value, String> {
        let function_id = params["function_id"]
            .as_str()
            .ok_or("Missing required parameter: function_id")?;

        let _include_source = params["include_source"].as_bool().unwrap_or(true);

        // Parse function_id: "file_path::function_name"
        let (file_path, function_name) = parse_function_id(function_id)?;

        let index = self.manager.get_index().await?;

        let result = index
            .get_function(&file_path, &function_name)
            .ok_or_else(|| format!("Function not found: {}", function_id))?;

        let mut response = json!({
            "id": function_id,
            "name": result.function_name,
            "signature": result.signature,
            "file_path": result.file_path,
            "start_line": result.start_line,
            "end_line": result.end_line,
            "language": result.language,
            "quality": {
                "grade": result.tdg_grade,
                "complexity": result.complexity,
                "tdg_score": result.tdg_score,
                "loc": result.loc,
                "big_o": result.big_o,
                "satd_count": result.satd_count
            }
        });

        if let Some(doc) = &result.doc_comment {
            response["doc_comment"] = json!(doc);
        }

        if let Some(source) = &result.source {
            response["source"] = json!(source);
        }

        Ok(response)
    }
}

// ============================================================================
// FindSimilarTool
// ============================================================================

impl FindSimilarTool {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(manager: Arc<IndexManager>) -> Self {
        Self { manager }
    }

    pub fn schema() -> Value {
        json!({
            "name": "pmat_find_similar",
            "description": "Find functions similar to a reference function. Useful for finding related code, potential duplicates, or implementations of similar patterns.",
            "parameters": {
                "type": "object",
                "properties": {
                    "function_id": {
                        "type": "string",
                        "description": "Function ID to find similar functions for"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of similar functions (default: 5, max: 20)",
                        "minimum": 1,
                        "maximum": 20,
                        "default": 5
                    },
                    "min_similarity": {
                        "type": "number",
                        "description": "Minimum similarity score (0.0-1.0, default: 0.3)",
                        "minimum": 0.0,
                        "maximum": 1.0,
                        "default": 0.3
                    }
                },
                "required": ["function_id"]
            }
        })
    }
}

#[async_trait]
impl McpTool for FindSimilarTool {
    fn name(&self) -> &str {
        "pmat_find_similar"
    }

    fn schema(&self) -> Value {
        Self::schema()
    }

    async fn execute(&self, params: Value) -> Result<Value, String> {
        let start = Instant::now();

        let function_id = params["function_id"]
            .as_str()
            .ok_or("Missing required parameter: function_id")?;

        let limit = params["limit"].as_u64().unwrap_or(5) as usize;
        if limit > 20 {
            return Err("Limit exceeds maximum of 20".to_string());
        }

        let min_similarity = params["min_similarity"].as_f64().unwrap_or(0.3) as f32;
        if !(0.0..=1.0).contains(&min_similarity) {
            return Err("min_similarity must be between 0.0 and 1.0".to_string());
        }

        // Parse function_id: "file_path::function_name"
        let (file_path, function_name) = parse_function_id(function_id)?;

        let index = self.manager.get_index().await?;

        let similar = index.find_similar(&file_path, &function_name, limit)?;

        let query_time_ms = start.elapsed().as_millis() as u64;

        // Filter by minimum similarity
        let results_json: Vec<Value> = similar
            .iter()
            .filter(|r| r.relevance_score >= min_similarity)
            .map(|r| {
                json!({
                    "id": format!("{}::{}", r.file_path, r.function_name),
                    "name": r.function_name,
                    "file_path": r.file_path,
                    "start_line": r.start_line,
                    "end_line": r.end_line,
                    "language": r.language,
                    "similarity": r.relevance_score,
                    "quality": {
                        "grade": r.tdg_grade,
                        "complexity": r.complexity,
                        "tdg_score": r.tdg_score
                    }
                })
            })
            .collect();

        Ok(json!({
            "reference_function": function_id,
            "similar_functions": results_json,
            "total": results_json.len(),
            "query_time_ms": query_time_ms
        }))
    }
}

// ============================================================================
// IndexStatsTool
// ============================================================================

impl IndexStatsTool {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(manager: Arc<IndexManager>) -> Self {
        Self { manager }
    }

    pub fn schema() -> Value {
        json!({
            "name": "pmat_index_stats",
            "description": "Get statistics about the code index including function counts, quality distribution, and index health.",
            "parameters": {
                "type": "object",
                "properties": {
                    "rebuild": {
                        "type": "boolean",
                        "description": "Rebuild the index before returning stats (default: false)",
                        "default": false
                    }
                }
            }
        })
    }
}

#[async_trait]
impl McpTool for IndexStatsTool {
    fn name(&self) -> &str {
        "pmat_index_stats"
    }

    fn schema(&self) -> Value {
        Self::schema()
    }

    async fn execute(&self, params: Value) -> Result<Value, String> {
        let rebuild = params["rebuild"].as_bool().unwrap_or(false);

        let index = if rebuild {
            self.manager.rebuild_index().await?
        } else {
            self.manager.get_index().await?
        };

        let manifest = index.manifest();
        let stats = index.stats();

        Ok(json!({
            "manifest": {
                "version": manifest.version,
                "function_count": manifest.function_count,
                "file_count": manifest.file_count,
                "avg_tdg_score": manifest.avg_tdg_score,
                "built_at": manifest.built_at,
                "languages": manifest.languages
            },
            "quality_distribution": stats.by_grade,
            "language_distribution": stats.by_language,
            "avg_complexity": stats.avg_complexity,
            "total_functions": stats.total_functions,
            "index_size_bytes": stats.index_size_bytes
        }))
    }
}
