// Semantic Search Engine
// PMAT-SEARCH-004: High-level orchestration for code search
//
// GREEN Phase: Full implementation

use super::chunker::{chunk_code, ChunkType, Language};
use super::openai_embeddings::OpenAIEmbeddingsClient;
use super::turso_vector_db::{EmbeddingEntry, TursoVectorDB};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use walkdir::WalkDir;

/// Search mode
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchMode {
    SemanticOnly,
    KeywordOnly,
    Hybrid,
}

/// Search query
#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub query: String,
    pub mode: SearchMode,
    pub language_filter: Option<String>,
    pub file_pattern: Option<String>,
    pub chunk_type_filter: Option<ChunkType>,
    pub limit: usize,
}

/// Search result
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub file_path: String,
    pub chunk_name: String,
    pub chunk_type: String,
    pub language: String,
    pub similarity_score: f64,
    pub snippet: String,
    pub start_line: usize,
    pub end_line: usize,
}

/// Index statistics
#[derive(Debug, Clone)]
pub struct IndexStats {
    pub total_files: usize,
    pub total_chunks: usize,
    pub created: usize,
    pub updated: usize,
    pub skipped: usize,
    pub duration_ms: u64,
}

/// Semantic search engine
pub struct SemanticSearchEngine {
    vector_db: Arc<TursoVectorDB>,
    embeddings_client: Arc<OpenAIEmbeddingsClient>,
}

impl SemanticSearchEngine {
    /// Create new search engine
    ///
    /// # Arguments
    /// * `api_key` - OpenAI API key
    /// * `db_path` - Path to vector database
    pub async fn new(api_key: &str, db_path: &str) -> Result<Self, String> {
        let embeddings_client = OpenAIEmbeddingsClient::new(api_key)?;
        let vector_db = TursoVectorDB::new_local(db_path).await?;

        Ok(Self {
            vector_db: Arc::new(vector_db),
            embeddings_client: Arc::new(embeddings_client),
        })
    }

    /// Search code by natural language query
    ///
    /// # Arguments
    /// * `query` - Search query
    ///
    /// # Returns
    /// Ranked search results
    pub async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, String> {
        if query.query.trim().is_empty() {
            return Err("Query cannot be empty".to_string());
        }

        match query.mode {
            SearchMode::SemanticOnly => self.semantic_search(query).await,
            SearchMode::KeywordOnly => Err("Keyword-only mode not yet implemented".to_string()),
            SearchMode::Hybrid => Err("Hybrid mode not yet implemented".to_string()),
        }
    }

    /// Semantic search using vector similarity
    async fn semantic_search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, String> {
        // Generate embedding for query
        let embedding_result = self.embeddings_client.embed(&query.query).await?;
        let query_embedding = &embedding_result.embeddings[0];

        // Search vector database
        let db_results = self
            .vector_db
            .similarity_search(query_embedding, query.limit * 2) // Get extra for filtering
            .await?;

        // Apply filters and convert to SearchResult
        let mut results: Vec<SearchResult> = db_results
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

                // Chunk type filter
                if let Some(ref chunk_type) = query.chunk_type_filter {
                    let chunk_type_str = match chunk_type {
                        ChunkType::Function => "function",
                        ChunkType::Class => "class",
                        ChunkType::Module => "module",
                        ChunkType::File => "file",
                    };
                    if r.chunk_type != chunk_type_str {
                        return false;
                    }
                }

                true
            })
            .map(|r| {
                // Create snippet from chunk metadata
                // TODO: Store actual content snippet in database for better display
                let snippet = format!("{} {} ({}:{})", r.chunk_type, r.chunk_name, r.start_line, r.end_line);

                SearchResult {
                    file_path: r.file_path,
                    chunk_name: r.chunk_name,
                    chunk_type: r.chunk_type,
                    language: r.language,
                    similarity_score: r.similarity,
                    snippet,
                    start_line: r.start_line,
                    end_line: r.end_line,
                }
            })
            .collect();

        // Apply limit after filtering
        results.truncate(query.limit);

        Ok(results)
    }

    /// Find code similar to a reference file
    ///
    /// # Arguments
    /// * `file_path` - Path to reference file
    /// * `limit` - Maximum results
    ///
    /// # Returns
    /// Similar code chunks
    pub async fn find_similar(
        &self,
        file_path: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, String> {
        // Get embeddings for the file
        let file_embeddings = self.vector_db.query_by_file(file_path).await?;

        if file_embeddings.is_empty() {
            return Err(format!("File not indexed: {file_path}"));
        }

        // Use first chunk's embedding as reference
        let reference_embedding = &file_embeddings[0].embedding;

        // Search for similar chunks
        let results = self
            .vector_db
            .similarity_search(reference_embedding, limit)
            .await?;

        // Convert to SearchResult
        let search_results = results
            .into_iter()
            .map(|r| {
                // Create snippet from chunk metadata
                let snippet = format!("{} {} ({}:{})", r.chunk_type, r.chunk_name, r.start_line, r.end_line);

                SearchResult {
                    file_path: r.file_path,
                    chunk_name: r.chunk_name,
                    chunk_type: r.chunk_type,
                    language: r.language,
                    similarity_score: r.similarity,
                    snippet,
                    start_line: r.start_line,
                    end_line: r.end_line,
                }
            })
            .collect();

        Ok(search_results)
    }

    /// Index a directory
    ///
    /// # Arguments
    /// * `path` - Directory path
    ///
    /// # Returns
    /// Index statistics
    pub async fn index_directory(&self, path: &Path) -> Result<IndexStats, String> {
        let start = Instant::now();
        let mut stats = IndexStats {
            total_files: 0,
            total_chunks: 0,
            created: 0,
            updated: 0,
            skipped: 0,
            duration_ms: 0,
        };

        // Walk directory
        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let file_path = entry.path();

            // Detect language
            let language = match Self::detect_language(file_path) {
                Some(lang) => lang,
                None => continue, // Skip unsupported files
            };

            // Read file
            let content = match std::fs::read_to_string(file_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            // Chunk code
            let chunks = match chunk_code(&content, language) {
                Ok(chunks) => chunks,
                Err(_) => continue,
            };

            if chunks.is_empty() {
                continue;
            }

            stats.total_files += 1;
            stats.total_chunks += chunks.len();

            // Process chunks
            for chunk in chunks {
                // Check if chunk already exists with same checksum
                let existing = self
                    .vector_db
                    .query_by_file(file_path.to_str().unwrap())
                    .await?;

                let should_skip = existing.iter().any(|e| {
                    e.chunk_name == chunk.chunk_name
                        && e.file_path == file_path.to_str().unwrap()
                });

                if should_skip {
                    stats.skipped += 1;
                    continue;
                }

                // Generate embedding
                let embedding_result = self.embeddings_client.embed(&chunk.content).await?;
                let embedding = &embedding_result.embeddings[0];

                // Store in database
                let entry = EmbeddingEntry {
                    file_path: file_path.to_str().unwrap().to_string(),
                    chunk_name: chunk.chunk_name,
                    chunk_type: format!("{:?}", chunk.chunk_type).to_lowercase(),
                    language: chunk.language,
                    start_line: chunk.start_line,
                    end_line: chunk.end_line,
                    content_checksum: chunk.content_checksum,
                    embedding: embedding.clone(),
                    model: "text-embedding-3-small".to_string(),
                };

                self.vector_db.insert(&entry).await?;
                stats.created += 1;
            }
        }

        stats.duration_ms = start.elapsed().as_millis() as u64;

        Ok(stats)
    }

    /// Get total embedding count
    pub async fn embedding_count(&self) -> Result<usize, String> {
        // Query all embeddings and count
        let all = self.vector_db.similarity_search(&vec![0.0; 1536], usize::MAX).await?;
        Ok(all.len())
    }

    /// Detect programming language from file extension
    fn detect_language(path: &Path) -> Option<Language> {
        let extension = path.extension()?.to_str()?;

        match extension {
            "rs" => Some(Language::Rust),
            "ts" | "tsx" => Some(Language::TypeScript),
            "py" => Some(Language::Python),
            "c" | "h" => Some(Language::C),
            "cpp" | "cc" | "cxx" | "hpp" => Some(Language::Cpp),
            "go" => Some(Language::Go),
            _ => None,
        }
    }

    /// Check if path matches pattern
    fn matches_pattern(path: &str, pattern: &str) -> bool {
        // Simple glob matching (just check suffix for now)
        if let Some(suffix) = pattern.strip_prefix('*') {
            path.ends_with(suffix)
        } else {
            path.contains(pattern)
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_language() {
        assert_eq!(
            SemanticSearchEngine::detect_language(Path::new("test.rs")),
            Some(Language::Rust)
        );
        assert_eq!(
            SemanticSearchEngine::detect_language(Path::new("test.py")),
            Some(Language::Python)
        );
        assert_eq!(
            SemanticSearchEngine::detect_language(Path::new("test.txt")),
            None
        );
    }

    #[test]
    fn test_matches_pattern() {
        assert!(SemanticSearchEngine::matches_pattern("src/main.rs", "*.rs"));
        assert!(!SemanticSearchEngine::matches_pattern("src/main.rs", "*.py"));
        assert!(SemanticSearchEngine::matches_pattern("src/utils/math.rs", "utils"));
    }

}
