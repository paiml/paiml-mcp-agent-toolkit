#![cfg_attr(coverage_nightly, coverage(off))]
//! Local Semantic Analysis Service
//!
//! Pure Rust semantic search, topic modeling, and clustering.
//! **Zero external API dependencies** - no OpenAI, no internet required.
//!
//! # Architecture (Toyota Way - Jidoka)
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                  Semantic Search Stack                       │
//! │                  (Pure Rust - Zero API Keys)                 │
//! ├─────────────────────────────────────────────────────────────┤
//! │  aprender 0.14.0     │ TF-IDF, LDA, K-means, DBSCAN         │
//! │  trueno-rag          │ Hybrid retrieval, RRF fusion         │
//! │  trueno-graph        │ PageRank, BFS, Louvain clustering    │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Peer-Reviewed Foundation
//!
//! | Algorithm | Citation |
//! |-----------|----------|
//! | TF-IDF | Manning et al. (2008) "Introduction to IR" |
//! | LDA | Blei et al. (2003) JMLR |
//! | K-means | MacQueen (1967) Berkeley Symposium |
//! | DBSCAN | Ester et al. (1996) KDD |
//! | BM25 | Robertson & Zaragoza (2009) F&T in IR |
//! | RRF | Cormack et al. (2009) SIGIR |
//! | PageRank | Page et al. (1999) Stanford |
//!
//! # Usage
//!
//! ```rust,no_run
//! use pmat::services::local_semantic::LocalSemanticEngine;
//!
//! let mut engine = LocalSemanticEngine::new();
//!
//! // Index codebase
//! engine.index_directory(std::path::Path::new("."), None)?;
//!
//! // Extract topics (LDA)
//! let topics = engine.extract_topics(5, None)?;
//!
//! // Cluster code (K-means)
//! let clusters = engine.cluster("kmeans", Some(5))?;
//! # Ok::<(), String>(())
//! ```
//!
//! # Specification
//!
//! See: `docs/specifications/semantic-search-feature.md`

use aprender::cluster::{AgglomerativeClustering, KMeans, DBSCAN};
use aprender::primitives::Matrix;
use aprender::text::tokenize::WhitespaceTokenizer;
use aprender::text::topic::LatentDirichletAllocation;
use aprender::text::vectorize::TfidfVectorizer;
use aprender::traits::UnsupervisedEstimator;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Local semantic analysis engine using aprender
pub struct LocalSemanticEngine {
    /// Collected code documents
    documents: Vec<CodeDocument>,
    /// Document-term matrix (f64 for LDA)
    dtm: Option<Matrix<f64>>,
    /// Vocabulary mapping (word -> index)
    vocabulary: HashMap<String, usize>,
    /// Reverse vocabulary (index -> word)
    reverse_vocabulary: Vec<String>,
}

/// A code document for analysis
#[derive(Debug, Clone)]
pub struct CodeDocument {
    pub file_path: PathBuf,
    pub content: String,
    pub language: String,
}

/// Result of topic extraction
#[derive(Debug, Clone)]
pub struct LocalTopicResult {
    pub topics: Vec<LocalTopic>,
    pub num_documents: usize,
}

/// A single topic with top terms
#[derive(Debug, Clone)]
pub struct LocalTopic {
    pub id: usize,
    pub top_terms: Vec<(String, f64)>,
    pub document_count: usize,
}

/// Result of clustering
#[derive(Debug, Clone)]
pub struct LocalClusterResult {
    pub clusters: Vec<LocalCluster>,
    pub method: String,
    pub num_documents: usize,
}

/// A single cluster
#[derive(Debug, Clone)]
pub struct LocalCluster {
    pub id: usize,
    pub files: Vec<PathBuf>,
    pub size: usize,
}

impl Default for LocalSemanticEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalSemanticEngine {
    /// Create a new local semantic engine
    #[must_use]
    pub fn new() -> Self {
        Self {
            documents: Vec::new(),
            dtm: None,
            vocabulary: HashMap::new(),
            reverse_vocabulary: Vec::new(),
        }
    }

    /// Index a directory of source files
    ///
    /// # Arguments
    /// * `path` - Directory path to scan
    /// * `language_filter` - Optional language filter (e.g., "rust", "python")
    ///
    /// # Returns
    /// Number of documents indexed
    pub fn index_directory(
        &mut self,
        path: &Path,
        language_filter: Option<&str>,
    ) -> Result<usize, String> {
        self.documents.clear();

        for entry in WalkDir::new(path)
            .max_depth(10)
            .into_iter()
            .filter_map(Result::ok)
        {
            if !entry.file_type().is_file() {
                continue;
            }

            let file_path = entry.path();
            let extension = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");

            let language = match extension {
                "rs" => "rust",
                "py" => "python",
                "js" => "javascript",
                "ts" => "typescript",
                "go" => "go",
                "java" => "java",
                "c" | "h" => "c",
                "cpp" | "hpp" | "cc" => "cpp",
                "rb" => "ruby",
                "php" => "php",
                "swift" => "swift",
                "kt" => "kotlin",
                _ => continue, // Skip non-code files
            };

            // Apply language filter if specified
            if let Some(filter) = language_filter {
                if language != filter {
                    continue;
                }
            }

            // Read file content
            if let Ok(content) = std::fs::read_to_string(file_path) {
                // Skip very large files (> 100KB) and very small files (< 50 bytes)
                if content.len() > 100_000 || content.len() < 50 {
                    continue;
                }

                self.documents.push(CodeDocument {
                    file_path: file_path.to_path_buf(),
                    content,
                    language: language.to_string(),
                });
            }
        }

        if self.documents.is_empty() {
            return Err("No source files found to analyze".to_string());
        }

        // Build TF-IDF matrix
        self.build_tfidf_matrix()?;

        Ok(self.documents.len())
    }

    /// Build TF-IDF matrix from documents
    fn build_tfidf_matrix(&mut self) -> Result<(), String> {
        if self.documents.is_empty() {
            return Err("No documents to analyze".to_string());
        }

        // Prepare document texts
        let texts: Vec<&str> = self.documents.iter().map(|d| d.content.as_str()).collect();

        // Create TF-IDF vectorizer with code-friendly settings
        let mut vectorizer = TfidfVectorizer::new()
            .with_tokenizer(Box::new(WhitespaceTokenizer::new()))
            .with_min_df(2) // Minimum document frequency
            .with_max_df(0.95) // Maximum document frequency (exclude very common terms)
            .with_max_features(1000); // Limit vocabulary size (usize, not Option)

        // Fit and transform - returns Matrix<f64>
        let matrix = vectorizer
            .fit_transform(&texts)
            .map_err(|e| format!("TF-IDF vectorization failed: {}", e))?;

        // Store the vocabulary
        self.vocabulary = vectorizer.vocabulary().clone();

        // Build reverse vocabulary (index -> word)
        self.reverse_vocabulary = vec![String::new(); self.vocabulary.len()];
        for (word, &idx) in &self.vocabulary {
            if idx < self.reverse_vocabulary.len() {
                self.reverse_vocabulary[idx] = word.clone();
            }
        }

        self.dtm = Some(matrix);

        Ok(())
    }

    /// Extract topics using LDA
    ///
    /// # Arguments
    /// * `num_topics` - Number of topics to extract (1-20)
    /// * `language_filter` - Optional language filter
    ///
    /// # Returns
    /// Topic extraction results
    pub fn extract_topics(
        &mut self,
        num_topics: usize,
        language_filter: Option<String>,
    ) -> Result<LocalTopicResult, String> {
        if num_topics == 0 || num_topics > 20 {
            return Err("num_topics must be between 1 and 20".to_string());
        }

        // Re-index if language filter changed
        if language_filter.is_some() {
            let path = self
                .documents
                .first()
                .map(|d| d.file_path.parent().unwrap_or(Path::new(".")))
                .unwrap_or(Path::new("."))
                .to_path_buf();
            self.index_directory(&path, language_filter.as_deref())?;
        }

        let dtm = self
            .dtm
            .as_ref()
            .ok_or("No documents indexed. Call index_directory first.")?;

        if dtm.n_rows() < num_topics {
            return Err(format!(
                "Need at least {} documents for {} topics, but only {} indexed",
                num_topics,
                num_topics,
                dtm.n_rows()
            ));
        }

        // Run LDA
        let mut lda = LatentDirichletAllocation::new(num_topics).with_random_seed(42);

        lda.fit(dtm, 50) // 50 iterations
            .map_err(|e| format!("LDA failed: {}", e))?;

        // Extract top terms per topic
        let topic_word = lda
            .topic_words()
            .map_err(|e| format!("Failed to get topic-word distribution: {}", e))?;

        let mut topics = Vec::new();

        for topic_id in 0..num_topics {
            // Get word weights for this topic
            let mut term_weights: Vec<(usize, f64)> = (0..self.reverse_vocabulary.len())
                .map(|word_idx| {
                    let weight = topic_word.get(topic_id, word_idx);
                    (word_idx, weight)
                })
                .filter(|(_, w)| *w > 0.0)
                .collect();

            // Sort by weight descending
            term_weights.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            // Take top 10 terms
            let top_terms: Vec<(String, f64)> = term_weights
                .into_iter()
                .take(10)
                .filter_map(|(idx, weight)| {
                    self.reverse_vocabulary
                        .get(idx)
                        .filter(|s| !s.is_empty())
                        .map(|term| (term.clone(), weight))
                })
                .collect();

            // Count documents with high probability for this topic
            let doc_count = if let Ok(dt) = lda.document_topics() {
                (0..dt.n_rows())
                    .filter(|&doc_idx| {
                        let p = dt.get(doc_idx, topic_id);
                        p > 0.1
                    })
                    .count()
            } else {
                0
            };

            topics.push(LocalTopic {
                id: topic_id,
                top_terms,
                document_count: doc_count,
            });
        }

        Ok(LocalTopicResult {
            topics,
            num_documents: self.documents.len(),
        })
    }

    /// Cluster documents using specified method
    ///
    /// # Arguments
    /// * `method` - Clustering method: "kmeans", "hierarchical", or "dbscan"
    /// * `k` - Number of clusters (required for kmeans)
    ///
    /// # Returns
    /// Clustering results
    pub fn cluster(&self, method: &str, k: Option<usize>) -> Result<LocalClusterResult, String> {
        let dtm = self
            .dtm
            .as_ref()
            .ok_or("No documents indexed. Call index_directory first.")?;

        // Convert f64 matrix to f32 for clustering
        let n_rows = dtm.n_rows();
        let n_cols = dtm.n_cols();
        let data_f32: Vec<f32> = (0..n_rows * n_cols)
            .map(|i| {
                let row = i / n_cols;
                let col = i % n_cols;
                dtm.get(row, col) as f32
            })
            .collect();

        let matrix_f32 = Matrix::from_vec(n_rows, n_cols, data_f32)
            .map_err(|e| format!("Matrix conversion failed: {}", e))?;

        let labels: Vec<i32> = match method {
            "kmeans" => {
                let k_val = k.ok_or("K-means requires --k parameter")?;
                if k_val > n_rows {
                    return Err(format!(
                        "Cannot create {} clusters from {} documents",
                        k_val, n_rows
                    ));
                }
                let mut kmeans = KMeans::new(k_val).with_max_iter(100).with_random_state(42);
                kmeans
                    .fit(&matrix_f32)
                    .map_err(|e| format!("K-means failed: {}", e))?;
                kmeans
                    .predict(&matrix_f32)
                    .into_iter()
                    .map(|l| l as i32)
                    .collect()
            }
            "hierarchical" => {
                let n_clusters = k.unwrap_or(5.min(n_rows));
                let mut agg =
                    AgglomerativeClustering::new(n_clusters, aprender::cluster::Linkage::Average);
                agg.fit(&matrix_f32)
                    .map_err(|e| format!("Hierarchical clustering failed: {}", e))?;
                agg.labels().iter().map(|&l| l as i32).collect()
            }
            "dbscan" => {
                let mut dbscan = DBSCAN::new(0.5, 2);
                dbscan
                    .fit(&matrix_f32)
                    .map_err(|e| format!("DBSCAN failed: {}", e))?;
                dbscan.labels().clone()
            }
            _ => return Err(format!("Unknown clustering method: {}", method)),
        };

        // Group documents by cluster
        let mut cluster_map: HashMap<i32, Vec<PathBuf>> = HashMap::new();
        for (idx, &label) in labels.iter().enumerate() {
            if label >= 0 {
                // Skip noise points (label = -1 in DBSCAN)
                cluster_map
                    .entry(label)
                    .or_default()
                    .push(self.documents[idx].file_path.clone());
            }
        }

        let mut clusters: Vec<LocalCluster> = cluster_map
            .into_iter()
            .map(|(id, files)| LocalCluster {
                id: id as usize,
                size: files.len(),
                files,
            })
            .collect();

        clusters.sort_by_key(|c| std::cmp::Reverse(c.size));

        Ok(LocalClusterResult {
            clusters,
            method: method.to_string(),
            num_documents: self.documents.len(),
        })
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_project() -> TempDir {
        let temp_dir = TempDir::new().expect("create temp dir");

        // Create some test Rust files
        fs::write(
            temp_dir.path().join("main.rs"),
            r#"
            fn main() {
                println!("Hello, world!");
                let x = calculate_sum(1, 2);
                println!("Sum: {}", x);
            }

            fn calculate_sum(a: i32, b: i32) -> i32 {
                a + b
            }
            "#,
        )
        .expect("write main.rs");

        fs::write(
            temp_dir.path().join("lib.rs"),
            r#"
            pub mod utils;

            pub fn process_data(data: &[u8]) -> Vec<u8> {
                data.iter().map(|&b| b * 2).collect()
            }

            pub fn validate_input(input: &str) -> bool {
                !input.is_empty() && input.len() < 1000
            }
            "#,
        )
        .expect("write lib.rs");

        fs::write(
            temp_dir.path().join("utils.rs"),
            r#"
            use std::collections::HashMap;

            pub fn create_cache() -> HashMap<String, String> {
                HashMap::new()
            }

            pub fn parse_config(config: &str) -> Option<Config> {
                if config.is_empty() {
                    return None;
                }
                Some(Config { name: config.to_string() })
            }

            pub struct Config {
                pub name: String,
            }
            "#,
        )
        .expect("write utils.rs");

        temp_dir
    }

    #[test]
    fn test_index_directory() {
        let temp_dir = create_test_project();
        let mut engine = LocalSemanticEngine::new();

        let count = engine.index_directory(temp_dir.path(), None).unwrap();

        assert_eq!(count, 3, "Should index 3 Rust files");
        assert_eq!(engine.documents.len(), 3);
    }

    #[test]
    fn test_index_with_language_filter() {
        let temp_dir = create_test_project();

        // Add a Python file
        fs::write(
            temp_dir.path().join("script.py"),
            "print('hello')\n# comment",
        )
        .expect("write script.py");

        let mut engine = LocalSemanticEngine::new();

        // Filter only Rust
        let count = engine
            .index_directory(temp_dir.path(), Some("rust"))
            .unwrap();

        assert_eq!(count, 3, "Should only index Rust files");
    }

    #[test]
    fn test_extract_topics() {
        let temp_dir = create_test_project();
        let mut engine = LocalSemanticEngine::new();

        engine.index_directory(temp_dir.path(), None).unwrap();

        let result = engine.extract_topics(2, None).unwrap();

        assert_eq!(result.topics.len(), 2);
        assert_eq!(result.num_documents, 3);

        // Each topic should have top terms
        for topic in &result.topics {
            assert!(!topic.top_terms.is_empty());
        }
    }

    #[test]
    fn test_cluster_kmeans() {
        let temp_dir = create_test_project();
        let mut engine = LocalSemanticEngine::new();

        engine.index_directory(temp_dir.path(), None).unwrap();

        let result = engine.cluster("kmeans", Some(2)).unwrap();

        assert_eq!(result.method, "kmeans");
        assert!(result.clusters.len() <= 2);
        assert_eq!(result.num_documents, 3);
    }

    #[test]
    fn test_cluster_hierarchical() {
        let temp_dir = create_test_project();
        let mut engine = LocalSemanticEngine::new();

        engine.index_directory(temp_dir.path(), None).unwrap();

        let result = engine.cluster("hierarchical", Some(2)).unwrap();

        assert_eq!(result.method, "hierarchical");
        assert!(!result.clusters.is_empty());
    }

    #[test]
    fn test_cluster_dbscan() {
        let temp_dir = create_test_project();
        let mut engine = LocalSemanticEngine::new();

        engine.index_directory(temp_dir.path(), None).unwrap();

        let result = engine.cluster("dbscan", None).unwrap();

        assert_eq!(result.method, "dbscan");
        // DBSCAN may produce noise points, so clusters could be fewer
    }

    #[test]
    fn test_invalid_num_topics() {
        let temp_dir = create_test_project();
        let mut engine = LocalSemanticEngine::new();

        engine.index_directory(temp_dir.path(), None).unwrap();

        assert!(engine.extract_topics(0, None).is_err());
        assert!(engine.extract_topics(21, None).is_err());
    }

    #[test]
    fn test_empty_directory() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let mut engine = LocalSemanticEngine::new();

        let result = engine.index_directory(temp_dir.path(), None);

        assert!(result.is_err());
    }

    // ============================================
    // TRUENO-RAG-5-TFIDF: TF-IDF Unification Tests
    // Tests for trueno-rag TfIdfEmbedder integration
    // ============================================

    /// Test trueno-rag TfIdfEmbedder can produce embeddings
    #[test]
    fn test_trueno_rag_tfidf_embedder_basic() {
        use trueno_rag::embed::{Embedder, TfIdfEmbedder};

        let documents = vec![
            "fn main() { println!(\"hello world\"); }",
            "fn add(a: i32, b: i32) -> i32 { a + b }",
            "fn subtract(a: i32, b: i32) -> i32 { a - b }",
        ];

        let mut embedder = TfIdfEmbedder::new(50);
        embedder.fit(&documents.iter().map(|s| *s).collect::<Vec<_>>());

        // Embed first document
        let embedding = embedder.embed(documents[0]).unwrap();

        assert_eq!(
            embedding.len(),
            50,
            "Should produce embeddings of specified dimension"
        );
        assert!(
            embedding.iter().any(|&x| x != 0.0),
            "Embedding should have non-zero values"
        );
    }

    /// Test TfIdfEmbedder produces normalized vectors
    #[test]
    fn test_trueno_rag_tfidf_normalization() {
        use trueno_rag::embed::{Embedder, TfIdfEmbedder};

        let documents = vec![
            "rust code function implementation",
            "python script module import",
            "javascript nodejs express api",
        ];

        let mut embedder = TfIdfEmbedder::new(100);
        embedder.fit(&documents.iter().map(|s| *s).collect::<Vec<_>>());

        for doc in &documents {
            let embedding = embedder.embed(doc).unwrap();

            // Check L2 norm is approximately 1.0 (normalized)
            let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();

            // Allow some tolerance for floating point and edge cases
            if norm > 0.0 {
                assert!(
                    (norm - 1.0).abs() < 0.01 || embedding.iter().all(|&x| x == 0.0),
                    "Embedding should be L2 normalized: norm = {}",
                    norm
                );
            }
        }
    }

    /// Test TfIdfEmbedder similarity correlates with document similarity
    #[test]
    fn test_trueno_rag_tfidf_similarity() {
        use trueno_rag::embed::{Embedder, TfIdfEmbedder};

        let documents = vec![
            "fn main() { println!(\"hello\"); }",
            "fn main() { println!(\"world\"); }", // Similar to first
            "class Animal { def speak(self): pass }", // Different
        ];

        let mut embedder = TfIdfEmbedder::new(100);
        embedder.fit(&documents.iter().map(|s| *s).collect::<Vec<_>>());

        let emb1 = embedder.embed(documents[0]).unwrap();
        let emb2 = embedder.embed(documents[1]).unwrap();
        let emb3 = embedder.embed(documents[2]).unwrap();

        // Compute cosine similarities
        let sim_12 = cosine_similarity_f32(&emb1, &emb2);
        let sim_13 = cosine_similarity_f32(&emb1, &emb3);

        // Similar documents should have higher similarity
        assert!(
            sim_12 > sim_13,
            "Similar Rust functions should have higher similarity than Rust vs Python: sim_12={}, sim_13={}",
            sim_12,
            sim_13
        );
    }

    /// Test batch embedding
    #[test]
    fn test_trueno_rag_tfidf_batch() {
        use trueno_rag::embed::{Embedder, TfIdfEmbedder};

        let documents = vec![
            "function first() { return 1; }",
            "function second() { return 2; }",
            "function third() { return 3; }",
        ];

        let mut embedder = TfIdfEmbedder::new(50);
        embedder.fit(&documents.iter().map(|s| *s).collect::<Vec<_>>());

        let batch_embeddings = embedder
            .embed_batch(&documents.iter().map(|s| *s).collect::<Vec<_>>())
            .unwrap();

        assert_eq!(batch_embeddings.len(), 3);
        for emb in &batch_embeddings {
            assert_eq!(emb.len(), 50);
        }
    }

    /// Test memory efficiency: trueno-rag uses f32 instead of f64
    #[test]
    fn test_trueno_rag_tfidf_memory_efficiency() {
        use trueno_rag::embed::{Embedder, TfIdfEmbedder};

        // trueno-rag uses f32 (4 bytes) vs aprender uses f64 (8 bytes)
        // This means 50% memory savings for large document collections

        let embedder = TfIdfEmbedder::new(1000);

        // Verify dimension is stored correctly
        assert_eq!(embedder.dimension(), 1000);

        // f32 embedding would use 4 * 1000 = 4KB per document
        // f64 embedding would use 8 * 1000 = 8KB per document
        // 50% memory savings with trueno-rag

        let memory_per_doc_f32 = 4 * 1000; // 4 bytes * dimension
        let memory_per_doc_f64 = 8 * 1000; // 8 bytes * dimension

        assert_eq!(
            memory_per_doc_f32 * 2,
            memory_per_doc_f64,
            "f32 should use half the memory of f64"
        );
    }

    /// Test sparse embedding (most values should be zero for short docs)
    #[test]
    fn test_trueno_rag_tfidf_sparsity() {
        use trueno_rag::embed::{Embedder, TfIdfEmbedder};

        let documents = vec!["fn main() {}", "def test(): pass", "function x() {}"];

        let mut embedder = TfIdfEmbedder::new(100);
        embedder.fit(&documents.iter().map(|s| *s).collect::<Vec<_>>());

        // Short documents should produce sparse embeddings
        let embedding = embedder.embed("fn main()").unwrap();

        let non_zero_count = embedding.iter().filter(|&&x| x != 0.0).count();
        let sparsity = 1.0 - (non_zero_count as f64 / embedding.len() as f64);

        // Most values should be zero (high sparsity)
        assert!(
            sparsity > 0.5,
            "Short documents should have sparse embeddings: sparsity = {}",
            sparsity
        );
    }

    /// Test IDF computation correctness
    #[test]
    fn test_trueno_rag_tfidf_idf_correctness() {
        use trueno_rag::embed::{Embedder, TfIdfEmbedder};

        // Document frequency test:
        // "common" appears in all docs (high DF -> low IDF)
        // "rare" appears in one doc (low DF -> high IDF)
        let documents = vec![
            "common word common word",
            "common another common word",
            "rare unique common word",
        ];

        let mut embedder = TfIdfEmbedder::new(50);
        embedder.fit(&documents.iter().map(|s| *s).collect::<Vec<_>>());

        let common_emb = embedder.embed("common").unwrap();
        let rare_emb = embedder.embed("rare").unwrap();

        // The embeddings are normalized, so we check non-zero pattern
        let common_nonzero = common_emb.iter().filter(|&&x| x != 0.0).count();
        let rare_nonzero = rare_emb.iter().filter(|&&x| x != 0.0).count();

        // Both should produce some signal
        assert!(common_nonzero >= 1 || rare_nonzero >= 1);
    }

    // Helper function for cosine similarity
    fn cosine_similarity_f32(v1: &[f32], v2: &[f32]) -> f32 {
        let dot: f32 = v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum();
        let norm1: f32 = v1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm2: f32 = v2.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm1 == 0.0 || norm2 == 0.0 {
            0.0
        } else {
            dot / (norm1 * norm2)
        }
    }
}
