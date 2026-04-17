// Tests for hybrid search engine (BM25 and RRF)
// Contains: BM25 RED phase tests and original RRF backward-compatibility tests.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // TRUENO-RAG-1-BM25: RED Phase Tests
    // These tests define the expected BM25 behavior
    // ============================================================================

    #[test]
    fn test_bm25_engine_creation() {
        let engine = Bm25SearchEngine::new();
        assert!(engine.is_empty());
        assert_eq!(engine.len(), 0);
    }

    #[test]
    fn test_bm25_engine_with_params() {
        let engine = Bm25SearchEngine::with_params(1.5, 0.5);
        assert!(engine.is_empty());
    }

    #[test]
    fn test_bm25_index_single_file() {
        let mut engine = Bm25SearchEngine::new();

        let content = r#"
fn main() {
    println!("Hello, world!");
}

fn calculate_sum(a: i32, b: i32) -> i32 {
    a + b
}
"#;

        engine.index_file("src/main.rs", content, "rust");

        // Should have indexed non-empty lines
        assert!(!engine.is_empty());
    }

    #[test]
    fn test_bm25_search_returns_results() {
        let mut engine = Bm25SearchEngine::new();

        engine.index_file(
            "src/lib.rs",
            "fn process_data() { /* process */ }\nfn handle_request() { /* handle */ }",
            "rust",
        );

        let results = engine.search("process", 10);
        assert!(!results.is_empty(), "BM25 should find 'process'");
    }

    #[test]
    fn test_bm25_score_is_not_rank_based() {
        // BM25 scores should reflect actual term relevance, not just position
        let mut engine = Bm25SearchEngine::new();

        // Document with term appearing multiple times should score higher
        engine.index_file("a.rs", "fn test_foo() { foo(); foo(); foo(); }", "rust");
        engine.index_file("b.rs", "fn bar() { foo(); }", "rust");

        let results = engine.search("foo", 10);
        assert!(results.len() >= 2);

        // File with more "foo" occurrences should have higher score
        let a_score = results.iter().find(|(m, _)| m.file_path == "a.rs");
        let b_score = results.iter().find(|(m, _)| m.file_path == "b.rs");

        if let (Some((_, score_a)), Some((_, score_b))) = (a_score, b_score) {
            assert!(
                score_a > score_b,
                "BM25: More term occurrences should score higher (a={}, b={})",
                score_a,
                score_b
            );
        }
    }

    #[test]
    fn test_bm25_idf_affects_scoring() {
        // Rare terms should have higher IDF and thus higher scores
        let mut engine = Bm25SearchEngine::new();

        // "common" appears in many documents, "rare" in only one
        engine.index_file("1.rs", "common word common", "rust");
        engine.index_file("2.rs", "common word common", "rust");
        engine.index_file("3.rs", "common word common", "rust");
        engine.index_file("4.rs", "rare unique term", "rust");

        let common_results = engine.search("common", 10);
        let rare_results = engine.search("rare", 10);

        // Both should find results
        assert!(!common_results.is_empty());
        assert!(!rare_results.is_empty());

        // Rare term should have higher score due to higher IDF
        let common_max_score = common_results
            .iter()
            .map(|(_, s)| *s)
            .fold(0.0f32, f32::max);
        let rare_max_score = rare_results.iter().map(|(_, s)| *s).fold(0.0f32, f32::max);

        assert!(
            rare_max_score > common_max_score,
            "BM25: Rare terms should have higher IDF score (rare={}, common={})",
            rare_max_score,
            common_max_score
        );
    }

    #[test]
    fn test_bm25_empty_query_returns_empty() {
        let mut engine = Bm25SearchEngine::new();
        engine.index_file("test.rs", "fn test() {}", "rust");

        let results = engine.search("", 10);
        assert!(results.is_empty());
    }

    #[test]
    fn test_bm25_stopwords_filtered() {
        let mut engine = Bm25SearchEngine::new();
        engine.index_file("test.rs", "the quick brown fox", "rust");

        // Search for stopword should return empty (BM25Index filters stopwords)
        let results = engine.search("the", 10);
        assert!(results.is_empty(), "Stopwords should be filtered");

        // Non-stopword should work
        let results = engine.search("quick", 10);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_bm25_no_match_returns_empty() {
        let mut engine = Bm25SearchEngine::new();
        engine.index_file("test.rs", "fn hello() {}", "rust");

        let results = engine.search("nonexistent_term_xyz", 10);
        assert!(results.is_empty());
    }

    #[test]
    fn test_bm25_respects_limit() {
        let mut engine = Bm25SearchEngine::new();

        for i in 0..20 {
            engine.index_file(
                &format!("file{}.rs", i),
                &format!("fn test{i}() {{}}", i = i),
                "rust",
            );
        }

        let results = engine.search("fn", 5);
        assert!(results.len() <= 5);
    }

    #[test]
    fn test_bm25_returns_file_metadata() {
        let mut engine = Bm25SearchEngine::new();
        engine.index_file("src/utils/helper.rs", "fn helper_function() {}", "rust");

        let results = engine.search("helper", 10);
        assert!(!results.is_empty());

        let (meta, _score) = &results[0];
        assert_eq!(meta.file_path, "src/utils/helper.rs");
        assert!(meta.line_number > 0);
        assert!(meta.content.contains("helper"));
    }

    // ============================================================================
    // Original RRF tests (kept for backward compatibility)
    // ============================================================================

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
        assert!(HybridSearchEngine::matches_pattern(
            "src/utils/math.rs",
            "utils"
        ));
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
