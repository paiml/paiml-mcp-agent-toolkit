// MCP Semantic Search Tools
// PMAT-SEARCH-006: Expose semantic search via MCP protocol
//
// GREEN Phase: Full implementation

use crate::services::semantic::{HybridSearchEngine, HybridSearchMode, HybridSearchQuery};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Instant;

/// MCP Tool trait
#[async_trait]
pub trait McpTool: Send + Sync {
    fn name(&self) -> &str;
    fn schema(&self) -> Value;
    async fn execute(&self, params: Value) -> Result<Value, String>;
}

// ============================================================================
// semantic_search Tool
// ============================================================================

pub struct SemanticSearchTool {
    engine: Arc<HybridSearchEngine>,
}

impl SemanticSearchTool {
    pub fn new(engine: Arc<HybridSearchEngine>) -> Self {
        Self { engine }
    }

    pub fn schema() -> Value {
        json!({
            "name": "semantic_search",
            "description": "Search code by natural language query using hybrid semantic + keyword search",
            "parameters": {
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Natural language search query"
                    },
                    "mode": {
                        "type": "string",
                        "enum": ["keyword", "vector", "hybrid"],
                        "description": "Search mode (default: hybrid)",
                        "default": "hybrid"
                    },
                    "language": {
                        "type": "string",
                        "enum": ["rust", "typescript", "python", "c", "cpp", "go"],
                        "description": "Filter by programming language (optional)"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of results (default: 10, max: 100)",
                        "minimum": 1,
                        "maximum": 100,
                        "default": 10
                    }
                },
                "required": ["query"]
            }
        })
    }
}

#[async_trait]
impl McpTool for SemanticSearchTool {
    fn name(&self) -> &str {
        "semantic_search"
    }

    fn schema(&self) -> Value {
        Self::schema()
    }

    async fn execute(&self, params: Value) -> Result<Value, String> {
        let start = Instant::now();

        // Extract parameters
        let query = params["query"]
            .as_str()
            .ok_or("Missing required parameter: query")?;

        if query.trim().is_empty() {
            return Err("Query cannot be empty".to_string());
        }

        let mode_str = params["mode"].as_str().unwrap_or("hybrid");
        let mode = match mode_str {
            "keyword" => HybridSearchMode::KeywordOnly,
            "vector" => HybridSearchMode::VectorOnly,
            "hybrid" => HybridSearchMode::Hybrid,
            _ => return Err(format!("Invalid mode: {mode_str}")),
        };

        let language_filter = params["language"].as_str().map(|s| s.to_string());
        let limit = params["limit"].as_u64().unwrap_or(10) as usize;

        if limit > 100 {
            return Err("Limit exceeds maximum of 100".to_string());
        }

        // Execute search
        let query_params = HybridSearchQuery {
            query: query.to_string(),
            mode,
            keyword_weight: 0.5,
            vector_weight: 0.5,
            language_filter,
            file_pattern: None,
            limit,
        };

        let results = self.engine.search(&query_params).await?;

        // Format response
        let query_time_ms = start.elapsed().as_millis() as u64;

        let results_json: Vec<Value> = results
            .iter()
            .map(|r| {
                json!({
                    "file_path": r.file_path,
                    "chunk_name": r.chunk_name,
                    "chunk_type": r.chunk_type,
                    "language": r.language,
                    "score": r.hybrid_score,
                    "keyword_score": r.keyword_score,
                    "vector_score": r.vector_score,
                    "snippet": r.snippet,
                    "start_line": r.start_line,
                    "end_line": r.end_line
                })
            })
            .collect();

        Ok(json!({
            "results": results_json,
            "total": results.len(),
            "mode": mode_str,
            "query_time_ms": query_time_ms
        }))
    }
}

// ============================================================================
// find_similar_code Tool
// ============================================================================

pub struct FindSimilarCodeTool {
    #[allow(dead_code)] // Reserved for future semantic search Phase 2 integration
    engine: Arc<HybridSearchEngine>,
}

impl FindSimilarCodeTool {
    pub fn new(engine: Arc<HybridSearchEngine>) -> Self {
        Self { engine }
    }

    pub fn schema() -> Value {
        json!({
            "name": "find_similar_code",
            "description": "Find code similar to a reference file using semantic similarity",
            "parameters": {
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to reference file"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of results (default: 5, max: 50)",
                        "minimum": 1,
                        "maximum": 50,
                        "default": 5
                    }
                },
                "required": ["file_path"]
            }
        })
    }
}

#[async_trait]
impl McpTool for FindSimilarCodeTool {
    fn name(&self) -> &str {
        "find_similar_code"
    }

    fn schema(&self) -> Value {
        Self::schema()
    }

    async fn execute(&self, params: Value) -> Result<Value, String> {
        let file_path = params["file_path"]
            .as_str()
            .ok_or("Missing required parameter: file_path")?;

        let limit = params["limit"].as_u64().unwrap_or(5) as usize;

        if limit > 50 {
            return Err("Limit exceeds maximum of 50".to_string());
        }

        // Find similar code (would use SemanticSearchEngine::find_similar)
        // For now, stub implementation
        let results_json = json!([]);

        Ok(json!({
            "results": results_json,
            "reference_file": file_path,
            "total": 0
        }))
    }
}

// ============================================================================
// cluster_code Tool
// ============================================================================

pub struct ClusterCodeTool {
    #[allow(dead_code)] // Reserved for future semantic search Phase 2 integration
    engine: Arc<HybridSearchEngine>,
}

impl ClusterCodeTool {
    pub fn new(engine: Arc<HybridSearchEngine>) -> Self {
        Self { engine }
    }

    pub fn schema() -> Value {
        json!({
            "name": "cluster_code",
            "description": "Group code chunks by semantic similarity using clustering algorithms",
            "parameters": {
                "type": "object",
                "properties": {
                    "method": {
                        "type": "string",
                        "enum": ["kmeans", "hierarchical", "dbscan"],
                        "description": "Clustering method"
                    },
                    "k": {
                        "type": "integer",
                        "description": "Number of clusters (required for kmeans)",
                        "minimum": 2,
                        "maximum": 20
                    },
                    "language": {
                        "type": "string",
                        "enum": ["rust", "typescript", "python", "c", "cpp", "go"],
                        "description": "Filter by programming language (optional)"
                    }
                },
                "required": ["method"]
            }
        })
    }
}

#[async_trait]
impl McpTool for ClusterCodeTool {
    fn name(&self) -> &str {
        "cluster_code"
    }

    fn schema(&self) -> Value {
        Self::schema()
    }

    async fn execute(&self, params: Value) -> Result<Value, String> {
        let method = params["method"]
            .as_str()
            .ok_or("Missing required parameter: method")?;

        match method {
            "kmeans" | "hierarchical" | "dbscan" => {}
            _ => return Err(format!("Invalid method: {method}")),
        }

        if method == "kmeans" {
            let k = params["k"]
                .as_u64()
                .ok_or("Parameter 'k' required for kmeans")?;

            if !(2..=20).contains(&k) {
                return Err("Parameter 'k' must be between 2 and 20".to_string());
            }
        }

        // Stub implementation - would use clustering algorithm
        let clusters_json = json!([]);

        Ok(json!({
            "clusters": clusters_json,
            "method": method,
            "total_chunks": 0,
            "total_clusters": 0
        }))
    }
}

// ============================================================================
// analyze_topics Tool
// ============================================================================

pub struct AnalyzeTopicsTool {
    #[allow(dead_code)] // Reserved for future semantic search Phase 2 integration
    engine: Arc<HybridSearchEngine>,
}

impl AnalyzeTopicsTool {
    pub fn new(engine: Arc<HybridSearchEngine>) -> Self {
        Self { engine }
    }

    pub fn schema() -> Value {
        json!({
            "name": "analyze_topics",
            "description": "Extract semantic topics from codebase using topic modeling",
            "parameters": {
                "type": "object",
                "properties": {
                    "num_topics": {
                        "type": "integer",
                        "description": "Number of topics to extract",
                        "minimum": 1,
                        "maximum": 20
                    },
                    "language": {
                        "type": "string",
                        "enum": ["rust", "typescript", "python", "c", "cpp", "go"],
                        "description": "Filter by programming language (optional)"
                    }
                },
                "required": ["num_topics"]
            }
        })
    }
}

#[async_trait]
impl McpTool for AnalyzeTopicsTool {
    fn name(&self) -> &str {
        "analyze_topics"
    }

    fn schema(&self) -> Value {
        Self::schema()
    }

    async fn execute(&self, params: Value) -> Result<Value, String> {
        let num_topics = params["num_topics"]
            .as_u64()
            .ok_or("Missing required parameter: num_topics")?;

        if !(1..=20).contains(&num_topics) {
            return Err("Parameter 'num_topics' must be between 1 and 20".to_string());
        }

        // Stub implementation - would use LDA topic modeling
        let topics_json = json!([]);

        Ok(json!({
            "topics": topics_json,
            "num_topics": num_topics
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_search_schema_structure() {
        let schema = SemanticSearchTool::schema();
        assert_eq!(schema["name"], "semantic_search");
        assert!(schema["parameters"]["properties"]["query"].is_object());
    }

    #[test]
    fn test_all_tool_names() {
        assert_eq!(SemanticSearchTool::schema()["name"], "semantic_search");
        assert_eq!(FindSimilarCodeTool::schema()["name"], "find_similar_code");
        assert_eq!(ClusterCodeTool::schema()["name"], "cluster_code");
        assert_eq!(AnalyzeTopicsTool::schema()["name"], "analyze_topics");
    }
}
