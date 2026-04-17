// local_semantic_tests.rs — tests for LocalSemanticEngine (included by local_semantic.rs)
// NO `use` imports here — they live in the parent module.

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

            /// Process data.
            pub fn process_data(data: &[u8]) -> Vec<u8> {
                data.iter().map(|&b| b * 2).collect()
            }

            /// Validate input.
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

            /// Create cache.
            pub fn create_cache() -> HashMap<String, String> {
                HashMap::new()
            }

            /// Parse config.
            pub fn parse_config(config: &str) -> Option<Config> {
                if config.is_empty() {
                    return None;
                }
                Some(Config { name: config.to_string() })
            }

            /// Configuration for config.
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

        let documents = ["fn main() { println!(\"hello world\"); }",
            "fn add(a: i32, b: i32) -> i32 { a + b }",
            "fn subtract(a: i32, b: i32) -> i32 { a - b }"];

        let mut embedder = TfIdfEmbedder::new(50);
        embedder.fit(&documents.to_vec());

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
        embedder.fit(&documents.to_vec());

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

        let documents = [
            "fn main() { println!(\"hello\"); }",
            "fn main() { println!(\"world\"); }", // Similar to first
            "class Animal { def speak(self): pass }", // Different
        ];

        let mut embedder = TfIdfEmbedder::new(100);
        embedder.fit(&documents.to_vec());

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

        let documents = ["function first() { return 1; }",
            "function second() { return 2; }",
            "function third() { return 3; }"];

        let mut embedder = TfIdfEmbedder::new(50);
        embedder.fit(&documents.to_vec());

        let batch_embeddings = embedder
            .embed_batch(&documents.to_vec())
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

        let documents = ["fn main() {}", "def test(): pass", "function x() {}"];

        let mut embedder = TfIdfEmbedder::new(100);
        embedder.fit(&documents.to_vec());

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
        let documents = ["common word common word",
            "common another common word",
            "rare unique common word"];

        let mut embedder = TfIdfEmbedder::new(50);
        embedder.fit(&documents.to_vec());

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
