// BM25 Search Engine (trueno-rag integration)
// Contains: Bm25SearchEngine struct, impl, and Default trait.
// TRUENO-RAG-1-BM25: Replace ripgrep+RRF with true BM25 scoring.

/// BM25-based keyword search engine using trueno-rag
/// Provides true BM25 scoring instead of rank-based RRF heuristics
pub struct Bm25SearchEngine {
    index: BM25Index,
    /// Maps ChunkId to file metadata
    chunk_metadata: HashMap<ChunkId, KeywordMatch>,
}

impl Bm25SearchEngine {
    /// Create a new BM25 search engine
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            index: BM25Index::new(),
            chunk_metadata: HashMap::new(),
        }
    }

    /// Create with custom BM25 parameters
    ///
    /// # Arguments
    /// * `k1` - Term frequency saturation (default 1.2)
    /// * `b` - Length normalization (default 0.75)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_params(k1: f32, b: f32) -> Self {
        Self {
            index: BM25Index::with_params(k1, b),
            chunk_metadata: HashMap::new(),
        }
    }

    /// Index a code file
    ///
    /// # Arguments
    /// * `file_path` - Path to the file
    /// * `content` - File content
    /// * `language` - Programming language
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "non_empty_index")]
    pub fn index_file(&mut self, file_path: &str, content: &str, _language: &str) {
        // Split content into lines and index each
        for (line_num, line) in content.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }

            let doc_id = DocumentId::new();
            let chunk = Chunk::new(doc_id, line.to_string(), line_num, line_num + 1);
            let chunk_id = chunk.id;

            self.index.add(&chunk);
            self.chunk_metadata.insert(
                chunk_id,
                KeywordMatch {
                    file_path: file_path.to_string(),
                    line_number: line_num + 1,
                    content: line.to_string(),
                },
            );
        }
    }

    /// Search using BM25 scoring
    ///
    /// # Arguments
    /// * `query` - Search query
    /// * `limit` - Maximum results
    ///
    /// # Returns
    /// Results with true BM25 scores (not rank-based)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn search(&self, query: &str, limit: usize) -> Vec<(KeywordMatch, f32)> {
        let results = self.index.search(query, limit);

        results
            .into_iter()
            .filter_map(|(chunk_id, score)| {
                self.chunk_metadata
                    .get(&chunk_id)
                    .map(|meta| (meta.clone(), score))
            })
            .collect()
    }

    /// Get the number of indexed documents
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn len(&self) -> usize {
        self.index.len()
    }

    /// Check if the index is empty
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }
}

impl Default for Bm25SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}
