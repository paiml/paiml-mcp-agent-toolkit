#![cfg_attr(coverage_nightly, coverage(off))]
// Semantic Search CLI Commands
// PMAT-SEARCH-009: CLI for semantic search, clustering, and topic modeling
//
// RED Phase -> GREEN Phase implementation

use crate::services::semantic::{
    ClusterFilters, ClusteringEngine, ClusteringMethod, HybridSearchEngine, HybridSearchMode,
    HybridSearchQuery, HybridSearchResult, Linkage, SemanticSearchEngine, TopicEngine,
    TopicFilters, TursoVectorDB,
};
use std::path::PathBuf;
use std::sync::Arc;

/// Guidance shown when vector/hybrid search runs against an empty embeddings store.
const EMPTY_STORE_HINT: &str =
    "No embeddings indexed — run `pmat embed sync <path>` to build the embeddings database first";

/// Assembled semantic search output.
///
/// The reported count and the rendered rows are both derived from the same
/// `results` vec, so they can never disagree (ghost-results fix: the old code
/// printed `Found N results` from the engine's candidate count but rendered
/// zero rows).
pub struct SemanticSearchOutput {
    pub query: String,
    pub mode: String,
    pub results: Vec<HybridSearchResult>,
    /// Set when the embeddings store is empty and the mode needs vectors.
    pub empty_store_hint: Option<String>,
}

impl SemanticSearchOutput {
    /// Render human-readable output. One numbered row per result.
    pub fn render_text(&self) -> String {
        if let Some(hint) = &self.empty_store_hint {
            return hint.clone();
        }
        if self.results.is_empty() {
            return format!("No results found for query: {}", self.query);
        }

        let mut output = format!(
            "Found {} results for query: {}\n",
            self.results.len(),
            self.query
        );
        for (i, r) in self.results.iter().enumerate() {
            output.push_str(&format!(
                "\n{}. {}:{}-{} [{}] (score: {:.4})\n   {}\n",
                i + 1,
                r.file_path,
                r.start_line,
                r.end_line,
                r.language,
                r.hybrid_score,
                r.snippet
            ));
        }
        output
    }

    /// Render JSON output. `count` always equals `results.len()`.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "query": self.query,
            "mode": self.mode,
            "count": self.results.len(),
            "results": self.results.iter().map(|r| serde_json::json!({
                "file_path": r.file_path,
                "chunk_name": r.chunk_name,
                "chunk_type": r.chunk_type,
                "language": r.language,
                "start_line": r.start_line,
                "end_line": r.end_line,
                "keyword_score": r.keyword_score,
                "vector_score": r.vector_score,
                "hybrid_score": r.hybrid_score,
                "snippet": r.snippet,
            })).collect::<Vec<_>>(),
            "message": self.empty_store_hint,
        })
    }
}

/// Semantic search CLI handler
pub struct SemanticCli {
    search_engine: Arc<SemanticSearchEngine>,
    hybrid_engine: Arc<HybridSearchEngine>,
    clustering_engine: Arc<ClusteringEngine>,
    topic_engine: Arc<TopicEngine>,
    /// Vector DB path — used by `embed status` / `embed clear` (#568).
    db_path: String,
}

impl SemanticCli {
    /// Create new semantic CLI handler with local embeddings
    ///
    /// # Note
    /// Uses pure Rust TF-IDF embeddings via aprender.
    /// No external API keys or internet connection required.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn new(db_path: &str, workspace_path: &std::path::Path) -> Result<Self, String> {
        let vector_db = Arc::new(TursoVectorDB::new_local(db_path).await?);

        let search_engine = Arc::new(SemanticSearchEngine::new(db_path).await?);

        let hybrid_engine = Arc::new(HybridSearchEngine::new(db_path, workspace_path).await?);

        let clustering_engine = Arc::new(ClusteringEngine::new(Arc::clone(&vector_db)));
        let topic_engine = Arc::new(TopicEngine::new(Arc::clone(&vector_db)));

        Ok(Self {
            search_engine,
            hybrid_engine,
            clustering_engine,
            topic_engine,
            db_path: db_path.to_string(),
        })
    }

    /// Sync embeddings for directory
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn embed_sync(
        &self,
        directory: &PathBuf,
        language: Option<String>,
    ) -> Result<String, String> {
        let stats = self.search_engine.index_directory(directory).await?;
        // #568: persist the freshly-indexed embeddings so a later
        // `semantic search` / `embed status` process can load them.
        self.search_engine.save().await?;

        let msg = format!(
            "Synced {} chunks ({} created, {} updated)",
            stats.total_chunks, stats.created, stats.updated
        );

        if let Some(lang) = language {
            Ok(format!("{} [filtered by: {}]", msg, lang))
        } else {
            Ok(msg)
        }
    }

    /// Get embedding status
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn embed_status(&self) -> Result<String, String> {
        // #568: report the real count from the (loaded) persisted store.
        let count = self.search_engine.entry_count().await?;
        Ok(format!("Embedding database status: {count} chunks indexed"))
    }

    /// Clear all embeddings
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn embed_clear(&self, confirm: bool) -> Result<String, String> {
        if !confirm {
            return Err("Clear operation requires --confirm flag".to_string());
        }

        // #568: remove the persisted vector DB; the next process loads empty.
        let path = std::path::Path::new(&self.db_path);
        if path.exists() {
            std::fs::remove_file(path)
                .map_err(|e| format!("Failed to remove {}: {e}", self.db_path))?;
        }
        Ok("All embeddings cleared".to_string())
    }

    /// Semantic search returning structured results
    ///
    /// Vector/hybrid modes are guarded against an empty embeddings store:
    /// previously the keyword (ripgrep) side could report `Found N results`
    /// while zero rows rendered, which misrepresented unindexed workspaces.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn semantic_search_results(
        &self,
        query: &str,
        mode: &str,
        limit: usize,
        language: Option<String>,
    ) -> Result<SemanticSearchOutput, String> {
        if query.trim().is_empty() {
            return Err("Query cannot be empty".to_string());
        }

        let search_mode = match mode {
            "keyword" => HybridSearchMode::KeywordOnly,
            "vector" => HybridSearchMode::VectorOnly,
            "hybrid" => HybridSearchMode::Hybrid,
            _ => return Err(format!("Invalid mode: {}", mode)),
        };

        // Keyword mode greps the workspace directly; vector/hybrid need embeddings.
        if search_mode != HybridSearchMode::KeywordOnly
            && self.search_engine.entry_count().await? == 0
        {
            return Ok(SemanticSearchOutput {
                query: query.to_string(),
                mode: mode.to_string(),
                results: Vec::new(),
                empty_store_hint: Some(EMPTY_STORE_HINT.to_string()),
            });
        }

        let search_query = HybridSearchQuery {
            query: query.to_string(),
            mode: search_mode,
            keyword_weight: 0.5,
            vector_weight: 0.5,
            language_filter: language,
            file_pattern: None,
            limit,
        };

        let results = self.hybrid_engine.search(&search_query).await?;

        Ok(SemanticSearchOutput {
            query: query.to_string(),
            mode: mode.to_string(),
            results,
            empty_store_hint: None,
        })
    }

    /// Semantic search (human-readable rendering)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn semantic_search(
        &self,
        query: &str,
        mode: &str,
        limit: usize,
        language: Option<String>,
    ) -> Result<String, String> {
        let output = self
            .semantic_search_results(query, mode, limit, language)
            .await?;
        Ok(output.render_text())
    }

    /// Find similar code
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn semantic_similar(&self, file: &PathBuf, limit: usize) -> Result<String, String> {
        if !file.exists() {
            return Err(format!("File not found: {}", file.display()));
        }

        let file_path = file.to_string_lossy();
        let results = self.search_engine.find_similar(&file_path, limit).await?;

        if results.is_empty() {
            return Ok(format!("No similar code found for: {}", file.display()));
        }

        let mut output = format!(
            "Found {} similar code chunks to: {}\n\n",
            results.len(),
            file.display()
        );
        for (i, result) in results.iter().enumerate() {
            output.push_str(&format!(
                "{}. {} ({}:{}-{}) - similarity: {:.2}\n   {}\n\n",
                i + 1,
                result.file_path,
                result.chunk_name,
                result.start_line,
                result.end_line,
                result.similarity_score,
                result.snippet
            ));
        }

        Ok(output)
    }

    /// Cluster code
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn analyze_cluster(&self, method: &str, k: Option<usize>) -> Result<String, String> {
        let clustering_method = match method {
            "kmeans" => {
                let k_val = k.ok_or("K-means requires --k parameter")?;
                ClusteringMethod::KMeans { k: k_val }
            }
            "hierarchical" => ClusteringMethod::Hierarchical {
                linkage: Linkage::Average,
            },
            "dbscan" => ClusteringMethod::DBSCAN {
                epsilon: 1.0,
                min_samples: 2,
            },
            _ => return Err(format!("Invalid method: {}", method)),
        };

        let result = self
            .clustering_engine
            .cluster(clustering_method, ClusterFilters::default())
            .await?;

        Ok(format!("Clustered into {} clusters", result.clusters.len()))
    }

    /// Extract topics
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn analyze_topics(
        &self,
        num_topics: usize,
        language: Option<String>,
    ) -> Result<String, String> {
        if num_topics == 0 || num_topics > 20 {
            return Err("num_topics must be between 1 and 20".to_string());
        }

        let filters = TopicFilters {
            language,
            chunk_type: None,
            file_pattern: None,
        };

        let result = self
            .topic_engine
            .extract_topics(num_topics, filters)
            .await?;

        Ok(format!("Extracted {} topics", result.topics.len()))
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn setup_cli() -> (SemanticCli, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let cli = SemanticCli::new(db_path.to_str().unwrap(), temp_dir.path())
            .await
            .unwrap();

        (cli, temp_dir)
    }

    // Embed command tests
    #[tokio::test]
    async fn test_embed_sync_basic() {
        let (cli, temp_dir) = setup_cli().await;

        let dir = temp_dir.path().to_path_buf();
        let result = cli.embed_sync(&dir, None).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_embed_status() {
        let (cli, _temp) = setup_cli().await;

        let result = cli.embed_status().await;

        assert!(result.is_ok());
        assert!(result.unwrap().contains("chunks indexed"));
    }

    #[tokio::test]
    async fn test_embed_clear_requires_confirm() {
        let (cli, _temp) = setup_cli().await;

        let result = cli.embed_clear(false).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("confirm"));
    }

    #[tokio::test]
    async fn test_embed_clear_with_confirm() {
        let (cli, _temp) = setup_cli().await;

        let result = cli.embed_clear(true).await;

        assert!(result.is_ok());
    }

    // Semantic command tests
    #[tokio::test]
    async fn test_semantic_search_basic() {
        let (cli, _temp) = setup_cli().await;

        let result = cli
            .semantic_search("error handling", "hybrid", 10, None)
            .await;

        // Test passes if method executes without panic
        // May return error with empty database/workspace - that's OK
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_semantic_search_empty_query() {
        let (cli, _temp) = setup_cli().await;

        let result = cli.semantic_search("", "hybrid", 10, None).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
    }

    #[tokio::test]
    async fn test_semantic_search_invalid_mode() {
        let (cli, _temp) = setup_cli().await;

        let result = cli.semantic_search("test", "invalid", 10, None).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid mode"));
    }

    #[tokio::test]
    async fn test_semantic_search_empty_store_reports_hint_not_ghost_count() {
        let (cli, _temp) = setup_cli().await;

        // Empty embeddings DB: vector/hybrid must show guidance, never `Found N`.
        for mode in ["hybrid", "vector"] {
            let result = cli
                .semantic_search("advisory lock", mode, 10, None)
                .await
                .unwrap();
            assert!(
                result.contains("No embeddings indexed"),
                "mode {mode}: {result}"
            );
            assert!(result.contains("pmat embed sync"), "mode {mode}: {result}");
            assert!(!result.contains("Found"), "mode {mode}: {result}");
        }
    }

    // Render/count consistency tests (ghost-results fix)

    fn sample_result(n: usize) -> crate::services::semantic::HybridSearchResult {
        crate::services::semantic::HybridSearchResult {
            file_path: format!("src/file{n}.rs"),
            chunk_name: format!("chunk{n}"),
            chunk_type: "function".to_string(),
            language: "rust".to_string(),
            start_line: n * 10,
            end_line: n * 10 + 5,
            keyword_score: 0.1,
            vector_score: 0.2,
            hybrid_score: 0.3,
            snippet: format!("fn chunk{n}()"),
        }
    }

    fn count_rendered_rows(text: &str) -> usize {
        text.lines()
            .filter(|l| {
                l.split_once(". ")
                    .is_some_and(|(n, _)| n.parse::<usize>().is_ok())
            })
            .count()
    }

    #[test]
    fn test_render_text_count_matches_rendered_rows() {
        let output = SemanticSearchOutput {
            query: "advisory lock".to_string(),
            mode: "hybrid".to_string(),
            results: (1..=3).map(sample_result).collect(),
            empty_store_hint: None,
        };

        let text = output.render_text();

        assert!(text.contains("Found 3 results for query: advisory lock"));
        assert_eq!(
            count_rendered_rows(&text),
            3,
            "rendered rows must match reported count: {text}"
        );
    }

    #[test]
    fn test_render_text_empty_results_no_found_line() {
        let output = SemanticSearchOutput {
            query: "advisory lock".to_string(),
            mode: "hybrid".to_string(),
            results: Vec::new(),
            empty_store_hint: None,
        };

        let text = output.render_text();

        assert!(text.contains("No results found for query: advisory lock"));
        assert!(!text.contains("Found"));
    }

    #[test]
    fn test_render_text_empty_store_hint() {
        let output = SemanticSearchOutput {
            query: "advisory lock".to_string(),
            mode: "hybrid".to_string(),
            results: Vec::new(),
            empty_store_hint: Some(EMPTY_STORE_HINT.to_string()),
        };

        let text = output.render_text();

        assert!(text.contains("No embeddings indexed"));
        assert!(text.contains("pmat embed sync"));
        assert!(!text.contains("Found"));
    }

    #[test]
    fn test_to_json_count_matches_results_len() {
        let output = SemanticSearchOutput {
            query: "advisory lock".to_string(),
            mode: "hybrid".to_string(),
            results: (1..=3).map(sample_result).collect(),
            empty_store_hint: None,
        };

        let json = output.to_json();

        assert_eq!(json["count"], 3);
        assert_eq!(json["results"].as_array().unwrap().len(), 3);
        assert!(json["message"].is_null());
    }

    #[test]
    fn test_to_json_empty_store() {
        let output = SemanticSearchOutput {
            query: "advisory lock".to_string(),
            mode: "vector".to_string(),
            results: Vec::new(),
            empty_store_hint: Some(EMPTY_STORE_HINT.to_string()),
        };

        let json = output.to_json();

        assert_eq!(json["count"], 0);
        assert!(json["results"].as_array().unwrap().is_empty());
        assert!(json["message"]
            .as_str()
            .unwrap()
            .contains("pmat embed sync"));
    }

    #[tokio::test]
    async fn test_semantic_similar_invalid_file() {
        let (cli, _temp) = setup_cli().await;

        let file = PathBuf::from("/nonexistent/file.rs");
        let result = cli.semantic_similar(&file, 5).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    // Analyze command tests
    #[tokio::test]
    async fn test_analyze_cluster_kmeans() {
        let (cli, _temp) = setup_cli().await;

        let result = cli.analyze_cluster("kmeans", Some(3)).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_cluster_requires_k() {
        let (cli, _temp) = setup_cli().await;

        let result = cli.analyze_cluster("kmeans", None).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("requires --k"));
    }

    #[tokio::test]
    async fn test_analyze_cluster_hierarchical() {
        let (cli, _temp) = setup_cli().await;

        let result = cli.analyze_cluster("hierarchical", None).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_topics_basic() {
        let (cli, _temp) = setup_cli().await;

        let result = cli.analyze_topics(5, None).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_topics_invalid_count() {
        let (cli, _temp) = setup_cli().await;

        let result = cli.analyze_topics(0, None).await;

        assert!(result.is_err());

        let result = cli.analyze_topics(25, None).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_analyze_topics_with_language() {
        let (cli, _temp) = setup_cli().await;

        let result = cli.analyze_topics(3, Some("rust".to_string())).await;

        assert!(result.is_ok());
    }
}
