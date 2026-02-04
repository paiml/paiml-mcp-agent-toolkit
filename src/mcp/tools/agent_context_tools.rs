// MCP Agent Context Tools
// PMAT-470: RAG-powered semantic code search for agents
//
// These tools expose the agent context index via MCP protocol,
// enabling AI agents to search code with quality-aware filtering.

use crate::services::agent_context::{AgentContextIndex, QueryOptions};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// MCP Tool trait (same as semantic_search_tools)
#[async_trait]
pub trait McpTool: Send + Sync {
    fn name(&self) -> &str;
    fn schema(&self) -> Value;
    async fn execute(&self, params: Value) -> Result<Value, String>;
}

// ============================================================================
// Index Manager - Shared index state
// ============================================================================

/// Manages the agent context index lifecycle
pub struct IndexManager {
    index: RwLock<Option<AgentContextIndex>>,
    project_path: PathBuf,
}

impl IndexManager {
    pub fn new(project_path: PathBuf) -> Self {
        Self {
            index: RwLock::new(None),
            project_path,
        }
    }

    /// Get or build the index
    pub async fn get_index(&self) -> Result<AgentContextIndex, String> {
        // First check if we have a cached index
        {
            let guard = self.index.read().await;
            if let Some(idx) = guard.as_ref() {
                return Ok(idx.clone());
            }
        }

        // Build or load index
        let index_path = self.project_path.join(".pmat/context.idx");
        let index = if index_path.exists() {
            AgentContextIndex::load(&index_path)?
        } else {
            let idx = AgentContextIndex::build(&self.project_path)?;
            // Create directory and save
            if let Some(parent) = index_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create .pmat directory: {}", e))?;
            }
            idx.save(&index_path)?;
            idx
        };

        // Cache it
        {
            let mut guard = self.index.write().await;
            *guard = Some(index.clone());
        }

        Ok(index)
    }

    /// Force rebuild the index
    pub async fn rebuild_index(&self) -> Result<AgentContextIndex, String> {
        let index = AgentContextIndex::build(&self.project_path)?;

        let index_path = self.project_path.join(".pmat/context.idx");
        if let Some(parent) = index_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create .pmat directory: {}", e))?;
        }
        index.save(&index_path)?;

        // Update cache
        {
            let mut guard = self.index.write().await;
            *guard = Some(index.clone());
        }

        Ok(index)
    }
}

// ============================================================================
// pmat_query_code Tool
// ============================================================================

/// Search functions by natural language query with quality filtering
pub struct QueryCodeTool {
    manager: Arc<IndexManager>,
}

impl QueryCodeTool {
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

// ============================================================================
// pmat_get_function Tool
// ============================================================================

/// Get details for a specific function by ID
pub struct GetFunctionTool {
    manager: Arc<IndexManager>,
}

impl GetFunctionTool {
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
// pmat_find_similar Tool
// ============================================================================

/// Find functions similar to a reference function
pub struct FindSimilarTool {
    manager: Arc<IndexManager>,
}

impl FindSimilarTool {
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
// pmat_index_stats Tool
// ============================================================================

/// Get index statistics and health
pub struct IndexStatsTool {
    manager: Arc<IndexManager>,
}

impl IndexStatsTool {
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

// ============================================================================
// Helper Functions
// ============================================================================

/// Parse function ID in format "file_path::function_name"
fn parse_function_id(function_id: &str) -> Result<(String, String), String> {
    // Find the last "::" separator
    if let Some(pos) = function_id.rfind("::") {
        let file_path = &function_id[..pos];
        let function_name = &function_id[pos + 2..];
        if file_path.is_empty() || function_name.is_empty() {
            return Err(format!(
                "Invalid function_id format. Expected 'file_path::function_name', got: {}",
                function_id
            ));
        }
        Ok((file_path.to_string(), function_name.to_string()))
    } else {
        Err(format!(
            "Invalid function_id format. Expected 'file_path::function_name', got: {}",
            function_id
        ))
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_code_schema() {
        let schema = QueryCodeTool::schema();
        assert_eq!(schema["name"], "pmat_query_code");
        assert!(schema["parameters"]["properties"]["query"].is_object());
        assert_eq!(schema["parameters"]["required"], json!(["query"]));
    }

    #[test]
    fn test_get_function_schema() {
        let schema = GetFunctionTool::schema();
        assert_eq!(schema["name"], "pmat_get_function");
        assert!(schema["parameters"]["properties"]["function_id"].is_object());
        assert_eq!(schema["parameters"]["required"], json!(["function_id"]));
    }

    #[test]
    fn test_find_similar_schema() {
        let schema = FindSimilarTool::schema();
        assert_eq!(schema["name"], "pmat_find_similar");
        assert!(schema["parameters"]["properties"]["function_id"].is_object());
        assert_eq!(schema["parameters"]["required"], json!(["function_id"]));
    }

    #[test]
    fn test_index_stats_schema() {
        let schema = IndexStatsTool::schema();
        assert_eq!(schema["name"], "pmat_index_stats");
        assert!(schema["parameters"]["properties"]["rebuild"].is_object());
    }

    #[test]
    fn test_all_tool_names() {
        assert_eq!(QueryCodeTool::schema()["name"], "pmat_query_code");
        assert_eq!(GetFunctionTool::schema()["name"], "pmat_get_function");
        assert_eq!(FindSimilarTool::schema()["name"], "pmat_find_similar");
        assert_eq!(IndexStatsTool::schema()["name"], "pmat_index_stats");
    }

    #[test]
    fn test_parse_function_id_valid() {
        let (file, func) = parse_function_id("src/handlers/auth.rs::handle_login").unwrap();
        assert_eq!(file, "src/handlers/auth.rs");
        assert_eq!(func, "handle_login");
    }

    #[test]
    fn test_parse_function_id_nested() {
        let (file, func) = parse_function_id("src/foo/bar.rs::baz::qux").unwrap();
        assert_eq!(file, "src/foo/bar.rs::baz");
        assert_eq!(func, "qux");
    }

    #[test]
    fn test_parse_function_id_invalid_no_separator() {
        let result = parse_function_id("no_separator");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid function_id format"));
    }

    #[test]
    fn test_parse_function_id_invalid_empty_parts() {
        let result = parse_function_id("::function_only");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_index_manager_new() {
        let manager = IndexManager::new(PathBuf::from("/tmp/test"));
        let guard = manager.index.read().await;
        assert!(guard.is_none());
    }
}
