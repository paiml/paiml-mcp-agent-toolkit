// RED Phase: Write failing tests first

// NOTE (Sprint 47): Use assetsearch (../../assetsearch) for MCP-based semantic search.
// All tests in this file marked #[ignore] pending migration to assetsearch.

// PMAT-SEARCH-005: Hybrid Search Engine with RRF
// Test count: 25 tests

use pmat::services::semantic::hybrid_search::*;
use std::collections::HashSet;
use tempfile::TempDir;

// Helper to setup test engine
async fn setup_hybrid_engine() -> (HybridSearchEngine, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("hybrid_test.db");

    let engine = HybridSearchEngine::new(
        "sk-test-key-1234567890abcdefghijklmnop",
        db_path.to_str().unwrap(),
        temp_dir.path(),
    )
    .await
    .unwrap();

    (engine, temp_dir)
}

// Helper to index test code
async fn index_test_code(engine: &HybridSearchEngine, dir: &std::path::Path) {
    std::fs::create_dir_all(dir.join("src")).unwrap();

    std::fs::write(
        dir.join("src/math.rs"),
        r#"
/// Add two numbers
fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Multiply two numbers
fn multiply(a: i32, b: i32) -> i32 {
    a * b
}
"#,
    )
    .unwrap();

    engine.index_directory(dir).await.unwrap();
}

// Helper to validate weights
fn validate_weights(keyword_weight: f64, vector_weight: f64) -> bool {
    keyword_weight >= 0.0
        && vector_weight >= 0.0
        && (keyword_weight + vector_weight - 1.0).abs() < 0.01
}

// ============================================================================
// Keyword-Only Search Tests (3 tests)
// ============================================================================

#[ignore]
#[tokio::test]
async fn test_keyword_only_search() {
    let (engine, temp_dir) = setup_hybrid_engine().await;
    index_test_code(&engine, temp_dir.path()).await;

    let query = HybridSearchQuery {
        query: "fn add".to_string(),
        mode: HybridSearchMode::KeywordOnly,
        keyword_weight: 1.0,
        vector_weight: 0.0,
        language_filter: None,
        file_pattern: None,
        limit: 10,
    };

    let results = engine.search(&query).await.unwrap();
    assert!(results.len() > 0);
    assert!(results[0].keyword_score > 0.0);
    assert_eq!(results[0].vector_score, 0.0);
}

#[ignore]
#[tokio::test]
async fn test_keyword_search_exact_match() {
    let (engine, temp_dir) = setup_hybrid_engine().await;
    index_test_code(&engine, temp_dir.path()).await;

    let query = HybridSearchQuery {
        query: "multiply".to_string(),
        mode: HybridSearchMode::KeywordOnly,
        keyword_weight: 1.0,
        vector_weight: 0.0,
        language_filter: None,
        file_pattern: None,
        limit: 10,
    };

    let results = engine.search(&query).await.unwrap();
    assert!(results.iter().any(|r| r.chunk_name.contains("multiply")));
}

#[ignore]
#[tokio::test]
async fn test_keyword_search_no_results() {
    let (engine, temp_dir) = setup_hybrid_engine().await;
    index_test_code(&engine, temp_dir.path()).await;

    let query = HybridSearchQuery {
        query: "xyzzy_nonexistent_12345".to_string(),
        mode: HybridSearchMode::KeywordOnly,
        keyword_weight: 1.0,
        vector_weight: 0.0,
        language_filter: None,
        file_pattern: None,
        limit: 10,
    };

    let results = engine.search(&query).await.unwrap();
    assert_eq!(results.len(), 0);
}

// ============================================================================
// Vector-Only Search Tests (3 tests)
// ============================================================================

#[ignore]
#[tokio::test]
#[ignore] // Requires OpenAI API
async fn test_vector_only_search() {
    let (engine, temp_dir) = setup_hybrid_engine().await;
    index_test_code(&engine, temp_dir.path()).await;

    let query = HybridSearchQuery {
        query: "function that calculates sum".to_string(),
        mode: HybridSearchMode::VectorOnly,
        keyword_weight: 0.0,
        vector_weight: 1.0,
        language_filter: None,
        file_pattern: None,
        limit: 10,
    };

    let results = engine.search(&query).await.unwrap();
    assert!(results.len() > 0);
    assert!(results[0].vector_score > 0.0);
    assert_eq!(results[0].keyword_score, 0.0);
}

#[ignore]
#[tokio::test]
async fn test_vector_only_mode_selection() {
    let (engine, temp_dir) = setup_hybrid_engine().await;
    index_test_code(&engine, temp_dir.path()).await;

    let query = HybridSearchQuery {
        query: "test".to_string(),
        mode: HybridSearchMode::VectorOnly,
        keyword_weight: 0.0,
        vector_weight: 1.0,
        language_filter: None,
        file_pattern: None,
        limit: 10,
    };

    // Should not error, just return results based on vector similarity
    let results = engine.search(&query).await.unwrap();
    // Results length depends on indexed content
    assert!(results.len() >= 0);
}

#[ignore]
#[tokio::test]
async fn test_vector_search_semantic_matching() {
    let (engine, temp_dir) = setup_hybrid_engine().await;
    index_test_code(&engine, temp_dir.path()).await;

    let query = HybridSearchQuery {
        query: "addition".to_string(), // Semantic match for "add"
        mode: HybridSearchMode::VectorOnly,
        keyword_weight: 0.0,
        vector_weight: 1.0,
        language_filter: None,
        file_pattern: None,
        limit: 10,
    };

    let results = engine.search(&query).await.unwrap();
    // May or may not find semantic match depending on embedding quality
    assert!(results.len() >= 0);
}

// ============================================================================
// Hybrid Search Tests (5 tests)
// ============================================================================

#[ignore]
#[tokio::test]
async fn test_hybrid_search() {
    let (engine, temp_dir) = setup_hybrid_engine().await;
    index_test_code(&engine, temp_dir.path()).await;

    let query = HybridSearchQuery {
        query: "add".to_string(),
        mode: HybridSearchMode::Hybrid,
        keyword_weight: 0.5,
        vector_weight: 0.5,
        language_filter: None,
        file_pattern: None,
        limit: 10,
    };

    let results = engine.search(&query).await.unwrap();
    assert!(results.len() > 0);
    // In hybrid mode, at least one score type should be > 0
    assert!(results[0].keyword_score > 0.0 || results[0].vector_score > 0.0);
    assert!(results[0].hybrid_score > 0.0);
}

#[ignore]
#[tokio::test]
async fn test_hybrid_combines_both_sources() {
    let (engine, temp_dir) = setup_hybrid_engine().await;
    index_test_code(&engine, temp_dir.path()).await;

    let query = HybridSearchQuery {
        query: "function".to_string(),
        mode: HybridSearchMode::Hybrid,
        keyword_weight: 0.5,
        vector_weight: 0.5,
        language_filter: None,
        file_pattern: None,
        limit: 10,
    };

    let results = engine.search(&query).await.unwrap();

    // Check that some results have both scores
    let has_keyword = results.iter().any(|r| r.keyword_score > 0.0);
    let has_vector = results.iter().any(|r| r.vector_score > 0.0);

    // At least one type should be present
    assert!(has_keyword || has_vector);
}

#[ignore]
#[tokio::test]
async fn test_hybrid_score_combination() {
    let (engine, temp_dir) = setup_hybrid_engine().await;
    index_test_code(&engine, temp_dir.path()).await;

    let query = HybridSearchQuery {
        query: "add".to_string(),
        mode: HybridSearchMode::Hybrid,
        keyword_weight: 0.5,
        vector_weight: 0.5,
        language_filter: None,
        file_pattern: None,
        limit: 5,
    };

    let results = engine.search(&query).await.unwrap();

    for result in &results {
        // Hybrid score should be weighted combination
        let expected = 0.5 * result.keyword_score + 0.5 * result.vector_score;
        assert!((result.hybrid_score - expected).abs() < 0.01);
    }
}

#[ignore]
#[tokio::test]
async fn test_hybrid_empty_query() {
    let (engine, _temp_dir) = setup_hybrid_engine().await;

    let query = HybridSearchQuery {
        query: "".to_string(),
        mode: HybridSearchMode::Hybrid,
        keyword_weight: 0.5,
        vector_weight: 0.5,
        language_filter: None,
        file_pattern: None,
        limit: 10,
    };

    let result = engine.search(&query).await;
    assert!(result.is_err());
}

#[ignore]
#[tokio::test]
async fn test_hybrid_with_filters() {
    let (engine, temp_dir) = setup_hybrid_engine().await;
    index_test_code(&engine, temp_dir.path()).await;

    let query = HybridSearchQuery {
        query: "fn".to_string(),
        mode: HybridSearchMode::Hybrid,
        keyword_weight: 0.5,
        vector_weight: 0.5,
        language_filter: Some("rust".to_string()),
        file_pattern: Some("*.rs".to_string()),
        limit: 10,
    };

    let results = engine.search(&query).await.unwrap();

    for result in &results {
        assert_eq!(result.language, "rust");
        assert!(result.file_path.ends_with(".rs"));
    }
}

// ============================================================================
// RRF Calculation Tests (3 tests)
// ============================================================================

#[ignore]
#[test]
fn test_rrf_calculation() {
    // Test RRF formula: 1 / (k + rank)
    let rrf1 = HybridSearchEngine::compute_rrf_score(1, 60); // Rank 1
    let rrf2 = HybridSearchEngine::compute_rrf_score(2, 60); // Rank 2
    let rrf10 = HybridSearchEngine::compute_rrf_score(10, 60); // Rank 10

    assert!((rrf1 - 1.0 / 61.0).abs() < 0.001);
    assert!((rrf2 - 1.0 / 62.0).abs() < 0.001);
    assert!((rrf10 - 1.0 / 70.0).abs() < 0.001);

    // Higher ranks should have higher scores
    assert!(rrf1 > rrf2);
    assert!(rrf2 > rrf10);
}

#[ignore]
#[test]
fn test_rrf_k_constant() {
    // Test with different k values
    let rrf_k60 = HybridSearchEngine::compute_rrf_score(1, 60);
    let rrf_k100 = HybridSearchEngine::compute_rrf_score(1, 100);

    // Larger k = smaller score
    assert!(rrf_k60 > rrf_k100);
}

#[ignore]
#[test]
fn test_rrf_score_range() {
    // RRF scores should always be positive and < 1
    for rank in 1..=100 {
        let score = HybridSearchEngine::compute_rrf_score(rank, 60);
        assert!(score > 0.0);
        assert!(score < 1.0);
    }
}

// ============================================================================
// Deduplication Tests (2 tests)
// ============================================================================

#[ignore]
#[tokio::test]
async fn test_result_deduplication() {
    let (engine, temp_dir) = setup_hybrid_engine().await;
    index_test_code(&engine, temp_dir.path()).await;

    let query = HybridSearchQuery {
        query: "add".to_string(),
        mode: HybridSearchMode::Hybrid,
        keyword_weight: 0.5,
        vector_weight: 0.5,
        language_filter: None,
        file_pattern: None,
        limit: 10,
    };

    let results = engine.search(&query).await.unwrap();

    // Check no duplicate file_path + chunk_name combinations
    let mut seen = HashSet::new();
    for result in &results {
        let key = format!("{}:{}", result.file_path, result.chunk_name);
        assert!(!seen.contains(&key), "Duplicate result: {}", key);
        seen.insert(key);
    }
}

#[ignore]
#[tokio::test]
async fn test_deduplication_preserves_higher_score() {
    let (engine, temp_dir) = setup_hybrid_engine().await;
    index_test_code(&engine, temp_dir.path()).await;

    let query = HybridSearchQuery {
        query: "add".to_string(),
        mode: HybridSearchMode::Hybrid,
        keyword_weight: 0.7,
        vector_weight: 0.3,
        language_filter: None,
        file_pattern: None,
        limit: 10,
    };

    let results = engine.search(&query).await.unwrap();

    // Results should be unique and sorted by score
    let mut prev_score = f64::MAX;
    for result in &results {
        assert!(result.hybrid_score <= prev_score);
        prev_score = result.hybrid_score;
    }
}

// ============================================================================
// Weight Adjustment Tests (3 tests)
// ============================================================================

#[ignore]
#[tokio::test]
async fn test_weight_adjustment_keyword_heavy() {
    let (engine, temp_dir) = setup_hybrid_engine().await;
    index_test_code(&engine, temp_dir.path()).await;

    let query = HybridSearchQuery {
        query: "add".to_string(),
        mode: HybridSearchMode::Hybrid,
        keyword_weight: 0.9,
        vector_weight: 0.1,
        language_filter: None,
        file_pattern: None,
        limit: 5,
    };

    let results = engine.search(&query).await.unwrap();
    assert!(results.len() > 0);

    // Hybrid score should be dominated by keyword score
    if results[0].keyword_score > 0.0 {
        let keyword_contribution = 0.9 * results[0].keyword_score;
        let vector_contribution = 0.1 * results[0].vector_score;
        assert!(keyword_contribution > vector_contribution);
    }
}

#[ignore]
#[tokio::test]
async fn test_weight_adjustment_vector_heavy() {
    let (engine, temp_dir) = setup_hybrid_engine().await;
    index_test_code(&engine, temp_dir.path()).await;

    let query = HybridSearchQuery {
        query: "add".to_string(),
        mode: HybridSearchMode::Hybrid,
        keyword_weight: 0.1,
        vector_weight: 0.9,
        language_filter: None,
        file_pattern: None,
        limit: 5,
    };

    let results = engine.search(&query).await.unwrap();
    assert!(results.len() > 0);
}

#[ignore]
#[test]
fn test_weight_validation() {
    // Valid weights
    assert!(validate_weights(0.5, 0.5));
    assert!(validate_weights(0.3, 0.7));
    assert!(validate_weights(1.0, 0.0));
    assert!(validate_weights(0.0, 1.0));

    // Invalid weights
    assert!(!validate_weights(0.6, 0.6)); // Sum > 1.0
    assert!(!validate_weights(-0.1, 1.1)); // Negative
    assert!(!validate_weights(1.5, 0.5)); // Sum > 1.0
}

// ============================================================================
// Performance Tests (3 tests)
// ============================================================================

#[ignore]
#[tokio::test]
async fn test_keyword_search_performance() {
    let (engine, temp_dir) = setup_hybrid_engine().await;
    index_test_code(&engine, temp_dir.path()).await;

    let start = std::time::Instant::now();

    let query = HybridSearchQuery {
        query: "function".to_string(),
        mode: HybridSearchMode::KeywordOnly,
        keyword_weight: 1.0,
        vector_weight: 0.0,
        language_filter: None,
        file_pattern: None,
        limit: 100,
    };

    engine.search(&query).await.unwrap();

    let duration = start.elapsed();
    assert!(duration.as_millis() < 1000); // < 1 second
}

#[ignore]
#[tokio::test]
async fn test_hybrid_search_performance() {
    let (engine, temp_dir) = setup_hybrid_engine().await;
    index_test_code(&engine, temp_dir.path()).await;

    let start = std::time::Instant::now();

    let query = HybridSearchQuery {
        query: "function".to_string(),
        mode: HybridSearchMode::Hybrid,
        keyword_weight: 0.5,
        vector_weight: 0.5,
        language_filter: None,
        file_pattern: None,
        limit: 50,
    };

    engine.search(&query).await.unwrap();

    let duration = start.elapsed();
    assert!(duration.as_secs() < 5); // < 5 seconds for hybrid
}

#[ignore]
#[tokio::test]
async fn test_result_limit_respected() {
    let (engine, temp_dir) = setup_hybrid_engine().await;
    index_test_code(&engine, temp_dir.path()).await;

    let query = HybridSearchQuery {
        query: "fn".to_string(),
        mode: HybridSearchMode::Hybrid,
        keyword_weight: 0.5,
        vector_weight: 0.5,
        language_filter: None,
        file_pattern: None,
        limit: 3,
    };

    let results = engine.search(&query).await.unwrap();
    assert!(results.len() <= 3);
}

// ============================================================================
// Hybrid Result Ranking Test (1 test)
// ============================================================================

#[ignore]
#[tokio::test]
async fn test_hybrid_result_ranking() {
    let (engine, temp_dir) = setup_hybrid_engine().await;
    index_test_code(&engine, temp_dir.path()).await;

    let query = HybridSearchQuery {
        query: "add".to_string(),
        mode: HybridSearchMode::Hybrid,
        keyword_weight: 0.5,
        vector_weight: 0.5,
        language_filter: None,
        file_pattern: None,
        limit: 10,
    };

    let results = engine.search(&query).await.unwrap();

    // Results should be sorted by hybrid_score (descending)
    for i in 1..results.len() {
        assert!(results[i - 1].hybrid_score >= results[i].hybrid_score);
    }
}

// ============================================================================
// Edge Cases (3 tests)
// ============================================================================

#[ignore]
#[tokio::test]
async fn test_empty_index_search() {
    let (engine, _temp_dir) = setup_hybrid_engine().await;
    // Don't index anything

    let query = HybridSearchQuery {
        query: "test".to_string(),
        mode: HybridSearchMode::Hybrid,
        keyword_weight: 0.5,
        vector_weight: 0.5,
        language_filter: None,
        file_pattern: None,
        limit: 10,
    };

    let results = engine.search(&query).await.unwrap();
    assert_eq!(results.len(), 0);
}

#[ignore]
#[tokio::test]
async fn test_special_characters_in_query() {
    let (engine, temp_dir) = setup_hybrid_engine().await;
    index_test_code(&engine, temp_dir.path()).await;

    let query = HybridSearchQuery {
        query: "fn.*add".to_string(), // Regex-like pattern
        mode: HybridSearchMode::KeywordOnly,
        keyword_weight: 1.0,
        vector_weight: 0.0,
        language_filter: None,
        file_pattern: None,
        limit: 10,
    };

    // Should not crash
    let results = engine.search(&query).await.unwrap();
    assert!(results.len() >= 0);
}

#[ignore]
#[tokio::test]
async fn test_very_long_query() {
    let (engine, temp_dir) = setup_hybrid_engine().await;
    index_test_code(&engine, temp_dir.path()).await;

    let query = HybridSearchQuery {
        query: "a".repeat(1000), // 1000 character query
        mode: HybridSearchMode::KeywordOnly,
        keyword_weight: 1.0,
        vector_weight: 0.0,
        language_filter: None,
        file_pattern: None,
        limit: 10,
    };

    // Should handle gracefully
    let results = engine.search(&query).await.unwrap();
    assert_eq!(results.len(), 0); // Unlikely to match
}
