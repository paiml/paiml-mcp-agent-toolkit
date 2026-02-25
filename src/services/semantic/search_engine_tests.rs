// Tests for SemanticSearchEngine
// Extracted from search_engine.rs for modularity

#[cfg_attr(coverage_nightly, coverage(off))]
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
        assert!(!SemanticSearchEngine::matches_pattern(
            "src/main.rs",
            "*.py"
        ));
        assert!(SemanticSearchEngine::matches_pattern(
            "src/utils/math.rs",
            "utils"
        ));
    }

    #[test]
    fn test_local_embedder_basic() {
        let embedder = LocalEmbedder::new();

        // Fit on some documents
        let docs = vec![
            "function hello world".to_string(),
            "struct data type".to_string(),
            "impl trait method".to_string(),
        ];

        embedder.fit(&docs).unwrap();

        // Generate embedding
        let embedding = embedder.embed("hello function").unwrap();
        assert!(!embedding.is_empty());

        // Check normalization (should be unit vector)
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.01 || norm == 0.0);
    }

    #[test]
    fn test_local_embedder_empty() {
        let embedder = LocalEmbedder::new();
        let embedding = embedder.embed("test");
        // Should not panic, returns default embedding
        assert!(embedding.is_ok());
    }
}
