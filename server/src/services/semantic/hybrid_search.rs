// Hybrid Search Engine with RRF
// PMAT-SEARCH-005: Combine keyword (ripgrep) and vector search
//
// GREEN Phase: Full implementation

use super::search_engine::{SearchQuery, SearchResult, SemanticSearchEngine};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

/// Search mode for hybrid engine
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HybridSearchMode {
    KeywordOnly,
    VectorOnly,
    Hybrid,
}

/// Hybrid search query
#[derive(Debug, Clone)]
pub struct HybridSearchQuery {
    pub query: String,
    pub mode: HybridSearchMode,
    pub keyword_weight: f64,
    pub vector_weight: f64,
    pub language_filter: Option<String>,
    pub file_pattern: Option<String>,
    pub limit: usize,
}

/// Hybrid search result
#[derive(Debug, Clone)]
pub struct HybridSearchResult {
    pub file_path: String,
    pub chunk_name: String,
    pub chunk_type: String,
    pub language: String,
    pub start_line: usize,
    pub end_line: usize,
    pub keyword_score: f64,
    pub vector_score: f64,
    pub hybrid_score: f64,
    pub snippet: String,
}

/// Keyword match from ripgrep
#[derive(Debug, Clone)]
struct KeywordMatch {
    file_path: String,
    line_number: usize,
    content: String,
}

/// Hybrid search engine combining keyword and vector search
pub struct HybridSearchEngine {
    semantic_engine: Arc<SemanticSearchEngine>,
    search_root: std::path::PathBuf,
}

impl HybridSearchEngine {
    /// Create new hybrid search engine
    ///
    /// # Arguments
    /// * `api_key` - OpenAI API key
    /// * `db_path` - Vector database path
    /// * `search_root` - Root directory for keyword search
    pub async fn new(api_key: &str, db_path: &str, search_root: &Path) -> Result<Self, String> {
        let semantic_engine = SemanticSearchEngine::new(api_key, db_path).await?;

        Ok(Self {
            semantic_engine: Arc::new(semantic_engine),
            search_root: search_root.to_path_buf(),
        })
    }

    /// Search using hybrid mode
    ///
    /// # Arguments
    /// * `query` - Search query
    ///
    /// # Returns
    /// Ranked hybrid search results
    pub async fn search(
        &self,
        query: &HybridSearchQuery,
    ) -> Result<Vec<HybridSearchResult>, String> {
        if query.query.trim().is_empty() {
            return Err("Query cannot be empty".to_string());
        }

        match query.mode {
            HybridSearchMode::KeywordOnly => self.keyword_only_search(query).await,
            HybridSearchMode::VectorOnly => self.vector_only_search(query).await,
            HybridSearchMode::Hybrid => self.hybrid_search(query).await,
        }
    }

    /// Keyword-only search using ripgrep
    async fn keyword_only_search(
        &self,
        query: &HybridSearchQuery,
    ) -> Result<Vec<HybridSearchResult>, String> {
        let matches = self.keyword_search(&query.query, query.limit * 2).await?;

        let mut results: Vec<HybridSearchResult> = matches
            .into_iter()
            .enumerate()
            .map(|(rank, m)| {
                let keyword_score = Self::compute_rrf_score(rank + 1, 60);

                HybridSearchResult {
                    file_path: m.file_path.clone(),
                    chunk_name: Self::extract_chunk_name(&m.content),
                    chunk_type: "file".to_string(),
                    language: Self::detect_language(&m.file_path),
                    start_line: m.line_number,
                    end_line: m.line_number,
                    keyword_score,
                    vector_score: 0.0,
                    hybrid_score: keyword_score,
                    snippet: Self::truncate(&m.content, 200),
                }
            })
            .collect();

        // Apply filters
        results = Self::apply_filters(results, query);
        results.truncate(query.limit);

        Ok(results)
    }

    /// Vector-only search using semantic engine
    async fn vector_only_search(
        &self,
        query: &HybridSearchQuery,
    ) -> Result<Vec<HybridSearchResult>, String> {
        let semantic_query = SearchQuery {
            query: query.query.clone(),
            mode: super::SearchMode::SemanticOnly,
            language_filter: query.language_filter.clone(),
            file_pattern: query.file_pattern.clone(),
            chunk_type_filter: None,
            limit: query.limit,
        };

        let semantic_results = self.semantic_engine.search(&semantic_query).await?;

        let results = semantic_results
            .into_iter()
            .map(|r| HybridSearchResult {
                file_path: r.file_path,
                chunk_name: r.chunk_name,
                chunk_type: r.chunk_type,
                language: r.language,
                start_line: r.start_line,
                end_line: r.end_line,
                keyword_score: 0.0,
                vector_score: r.similarity_score,
                hybrid_score: r.similarity_score,
                snippet: r.snippet,
            })
            .collect();

        Ok(results)
    }

    /// Hybrid search combining keyword and vector results with RRF
    async fn hybrid_search(
        &self,
        query: &HybridSearchQuery,
    ) -> Result<Vec<HybridSearchResult>, String> {
        // Run both searches in parallel
        let keyword_matches = self.keyword_search(&query.query, query.limit * 2).await?;

        let semantic_query = SearchQuery {
            query: query.query.clone(),
            mode: super::SearchMode::SemanticOnly,
            language_filter: query.language_filter.clone(),
            file_pattern: query.file_pattern.clone(),
            chunk_type_filter: None,
            limit: query.limit * 2,
        };

        let semantic_results = self.semantic_engine.search(&semantic_query).await?;

        // Merge results using RRF
        let merged = self.merge_results(
            keyword_matches,
            semantic_results,
            (query.keyword_weight, query.vector_weight),
        );

        // Apply filters and limit
        let mut filtered = Self::apply_filters(merged, query);
        filtered.truncate(query.limit);

        Ok(filtered)
    }

    /// Keyword search using ripgrep
    async fn keyword_search(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<KeywordMatch>, String> {
        let output = Command::new("rg")
            .arg("--line-number")
            .arg("--no-heading")
            .arg("--max-count")
            .arg(limit.to_string())
            .arg(query)
            .arg(&self.search_root)
            .output()
            .map_err(|e| format!("Failed to run ripgrep: {e}"))?;

        if !output.status.success() && !output.stdout.is_empty() {
            // ripgrep returns exit code 1 when no matches found, which is not an error
            if output.stdout.is_empty() {
                return Ok(Vec::new());
            }
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut matches = Vec::new();

        for line in stdout.lines().take(limit) {
            // Format: path:line_number:content
            let parts: Vec<&str> = line.splitn(3, ':').collect();
            if parts.len() == 3 {
                if let Ok(line_num) = parts[1].parse::<usize>() {
                    matches.push(KeywordMatch {
                        file_path: parts[0].to_string(),
                        line_number: line_num,
                        content: parts[2].to_string(),
                    });
                }
            }
        }

        Ok(matches)
    }

    /// Merge keyword and vector results using RRF
    fn merge_results(
        &self,
        keyword_matches: Vec<KeywordMatch>,
        semantic_results: Vec<SearchResult>,
        weights: (f64, f64),
    ) -> Vec<HybridSearchResult> {
        let mut result_map: HashMap<String, HybridSearchResult> = HashMap::new();

        // Add keyword results with RRF scores
        for (rank, km) in keyword_matches.iter().enumerate() {
            let keyword_score = Self::compute_rrf_score(rank + 1, 60);
            let key = format!("{}:{}", km.file_path, km.line_number);

            result_map.insert(
                key,
                HybridSearchResult {
                    file_path: km.file_path.clone(),
                    chunk_name: Self::extract_chunk_name(&km.content),
                    chunk_type: "file".to_string(),
                    language: Self::detect_language(&km.file_path),
                    start_line: km.line_number,
                    end_line: km.line_number,
                    keyword_score,
                    vector_score: 0.0,
                    hybrid_score: weights.0 * keyword_score,
                    snippet: Self::truncate(&km.content, 200),
                },
            );
        }

        // Add/merge vector results with RRF scores
        for (rank, sr) in semantic_results.iter().enumerate() {
            let vector_score = Self::compute_rrf_score(rank + 1, 60);
            let key = format!("{}:{}", sr.file_path, sr.chunk_name);

            if let Some(existing) = result_map.get_mut(&key) {
                // Merge: update vector score and recalculate hybrid
                existing.vector_score = vector_score;
                existing.hybrid_score = weights.0 * existing.keyword_score + weights.1 * vector_score;
            } else {
                // New entry from vector search
                result_map.insert(
                    key,
                    HybridSearchResult {
                        file_path: sr.file_path.clone(),
                        chunk_name: sr.chunk_name.clone(),
                        chunk_type: sr.chunk_type.clone(),
                        language: sr.language.clone(),
                        start_line: sr.start_line,
                        end_line: sr.end_line,
                        keyword_score: 0.0,
                        vector_score,
                        hybrid_score: weights.1 * vector_score,
                        snippet: sr.snippet.clone(),
                    },
                );
            }
        }

        // Convert to vec and sort by hybrid score
        let mut results: Vec<HybridSearchResult> = result_map.into_values().collect();
        results.sort_by(|a, b| b.hybrid_score.partial_cmp(&a.hybrid_score).unwrap());

        results
    }

    /// Compute RRF score for a given rank
    ///
    /// # Arguments
    /// * `rank` - Position in result set (1-indexed)
    /// * `k` - Constant (typically 60)
    ///
    /// # Returns
    /// RRF score (higher is better)
    pub fn compute_rrf_score(rank: usize, k: usize) -> f64 {
        1.0 / (k as f64 + rank as f64)
    }

    /// Apply filters to results
    fn apply_filters(
        results: Vec<HybridSearchResult>,
        query: &HybridSearchQuery,
    ) -> Vec<HybridSearchResult> {
        results
            .into_iter()
            .filter(|r| {
                // Language filter
                if let Some(ref lang) = query.language_filter {
                    if &r.language != lang {
                        return false;
                    }
                }

                // File pattern filter
                if let Some(ref pattern) = query.file_pattern {
                    if !Self::matches_pattern(&r.file_path, pattern) {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    /// Index a directory
    pub async fn index_directory(&self, path: &Path) -> Result<(), String> {
        self.semantic_engine.index_directory(path).await?;
        Ok(())
    }

    /// Detect language from file path
    fn detect_language(path: &str) -> String {
        if path.ends_with(".rs") {
            "rust".to_string()
        } else if path.ends_with(".ts") || path.ends_with(".tsx") {
            "typescript".to_string()
        } else if path.ends_with(".py") {
            "python".to_string()
        } else if path.ends_with(".go") {
            "go".to_string()
        } else if path.ends_with(".c") || path.ends_with(".h") {
            "c".to_string()
        } else if path.ends_with(".cpp") || path.ends_with(".hpp") {
            "cpp".to_string()
        } else {
            "unknown".to_string()
        }
    }

    /// Extract chunk name from content
    fn extract_chunk_name(content: &str) -> String {
        // Simple heuristic: first word or identifier
        content
            .split_whitespace()
            .find(|s| s.chars().all(|c| c.is_alphanumeric() || c == '_'))
            .unwrap_or("unknown")
            .to_string()
    }

    /// Check if path matches pattern
    fn matches_pattern(path: &str, pattern: &str) -> bool {
        if pattern.starts_with('*') {
            path.ends_with(&pattern[1..])
        } else {
            path.contains(pattern)
        }
    }

    /// Truncate string to max length
    fn truncate(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[..max_len])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rrf_score_calculation() {
        let score1 = HybridSearchEngine::compute_rrf_score(1, 60);
        let score2 = HybridSearchEngine::compute_rrf_score(2, 60);
        let score10 = HybridSearchEngine::compute_rrf_score(10, 60);

        assert!((score1 - 1.0 / 61.0).abs() < 0.001);
        assert!((score2 - 1.0 / 62.0).abs() < 0.001);
        assert!((score10 - 1.0 / 70.0).abs() < 0.001);

        assert!(score1 > score2);
        assert!(score2 > score10);
    }

    #[test]
    fn test_detect_language() {
        assert_eq!(HybridSearchEngine::detect_language("src/main.rs"), "rust");
        assert_eq!(HybridSearchEngine::detect_language("app.ts"), "typescript");
        assert_eq!(HybridSearchEngine::detect_language("script.py"), "python");
        assert_eq!(HybridSearchEngine::detect_language("main.go"), "go");
        assert_eq!(HybridSearchEngine::detect_language("test.c"), "c");
        assert_eq!(HybridSearchEngine::detect_language("test.cpp"), "cpp");
    }

    #[test]
    fn test_matches_pattern() {
        assert!(HybridSearchEngine::matches_pattern("src/main.rs", "*.rs"));
        assert!(!HybridSearchEngine::matches_pattern("src/main.rs", "*.py"));
        assert!(HybridSearchEngine::matches_pattern("src/utils/math.rs", "utils"));
    }

    #[test]
    fn test_truncate() {
        let short = "hello";
        assert_eq!(HybridSearchEngine::truncate(short, 10), "hello");

        let long = "a".repeat(300);
        let truncated = HybridSearchEngine::truncate(&long, 200);
        assert_eq!(truncated.len(), 203); // 200 + "..."
        assert!(truncated.ends_with("..."));
    }
}
