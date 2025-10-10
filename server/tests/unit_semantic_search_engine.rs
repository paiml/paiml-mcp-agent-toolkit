// RED Phase: Write failing tests first
// PMAT-SEARCH-004: Vector Similarity Search Engine
// Test count: 18 tests

use pmat::services::semantic::search_engine::*;
use pmat::services::semantic::ChunkType;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

// Helper to create test engine
async fn setup_test_engine() -> (SemanticSearchEngine, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_search.db");

    // Use mock API key for tests
    let engine = SemanticSearchEngine::new("sk-test-key-1234567890abcdefghijklmnop", db_path.to_str().unwrap())
        .await
        .unwrap();

    (engine, temp_dir)
}

// Helper to create test fixtures
fn create_test_fixtures(dir: &Path) {
    fs::create_dir_all(dir.join("src")).unwrap();

    // Rust file with add function
    fs::write(
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

    // Python file
    fs::write(
        dir.join("src/calc.py"),
        r#"
def add(a, b):
    """Calculate sum"""
    return a + b

class Calculator:
    """Simple calculator class"""
    def add(self, a, b):
        return a + b
"#,
    )
    .unwrap();
}

// ============================================================================
// Search by Query Tests (3 tests)
// ============================================================================

#[tokio::test]
#[ignore] // Requires actual OpenAI API
async fn test_search_by_query() {
    let (engine, temp_dir) = setup_test_engine().await;
    let fixtures_dir = temp_dir.path().join("fixtures");
    create_test_fixtures(&fixtures_dir);

    // Index test code
    engine.index_directory(&fixtures_dir).await.unwrap();

    // Search for semantic concept
    let query = SearchQuery {
        query: "function that adds two numbers".to_string(),
        mode: SearchMode::SemanticOnly,
        language_filter: None,
        file_pattern: None,
        chunk_type_filter: Some(ChunkType::Function),
        limit: 10,
    };

    let results = engine.search(&query).await.unwrap();
    assert!(results.len() > 0);
    assert!(results[0].similarity_score > 0.7);
}

#[tokio::test]
async fn test_search_empty_query() {
    let (engine, _temp_dir) = setup_test_engine().await;

    let query = SearchQuery {
        query: "".to_string(),
        mode: SearchMode::SemanticOnly,
        language_filter: None,
        file_pattern: None,
        chunk_type_filter: None,
        limit: 10,
    };

    let result = engine.search(&query).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_search_with_limit() {
    let (engine, temp_dir) = setup_test_engine().await;
    let fixtures_dir = temp_dir.path().join("fixtures");
    create_test_fixtures(&fixtures_dir);

    engine.index_directory(&fixtures_dir).await.unwrap();

    let query = SearchQuery {
        query: "function".to_string(),
        mode: SearchMode::SemanticOnly,
        language_filter: None,
        file_pattern: None,
        chunk_type_filter: None,
        limit: 2,
    };

    let results = engine.search(&query).await.unwrap();
    assert!(results.len() <= 2);
}

// ============================================================================
// Find Similar Code Test (1 test)
// ============================================================================

#[tokio::test]
async fn test_find_similar_code() {
    let (engine, temp_dir) = setup_test_engine().await;
    let fixtures_dir = temp_dir.path().join("fixtures");
    create_test_fixtures(&fixtures_dir);

    engine.index_directory(&fixtures_dir).await.unwrap();

    let file_path = fixtures_dir.join("src/math.rs");
    let results = engine.find_similar(file_path.to_str().unwrap(), 5).await.unwrap();

    // Should find at least 1 result (the file itself)
    assert!(results.len() > 0);

    // Results should be sorted by similarity (descending)
    for i in 1..results.len() {
        assert!(results[i - 1].similarity_score >= results[i].similarity_score);
    }
}

// ============================================================================
// Incremental Updates Tests (2 tests)
// ============================================================================

#[tokio::test]
async fn test_incremental_update() {
    let (engine, temp_dir) = setup_test_engine().await;
    let fixtures_dir = temp_dir.path().join("fixtures");
    create_test_fixtures(&fixtures_dir);

    // Initial index
    let stats1 = engine.index_directory(&fixtures_dir).await.unwrap();
    assert!(stats1.total_chunks > 0);
    assert_eq!(stats1.created, stats1.total_chunks);

    // Re-index without changes (should skip all)
    let stats2 = engine.index_directory(&fixtures_dir).await.unwrap();
    assert_eq!(stats2.skipped, stats2.total_chunks);
    assert_eq!(stats2.created, 0);
    assert_eq!(stats2.updated, 0);
}

#[tokio::test]
async fn test_incremental_after_modification() {
    let (engine, temp_dir) = setup_test_engine().await;
    let fixtures_dir = temp_dir.path().join("fixtures");
    create_test_fixtures(&fixtures_dir);

    // Initial index
    engine.index_directory(&fixtures_dir).await.unwrap();

    // Modify file
    fs::write(
        fixtures_dir.join("src/math.rs"),
        r#"
/// Add two numbers - MODIFIED
fn add(a: i32, b: i32) -> i32 {
    a + b + 1  // Changed implementation
}
"#,
    )
    .unwrap();

    // Re-index (should update changed chunks)
    let stats = engine.index_directory(&fixtures_dir).await.unwrap();
    assert!(stats.updated > 0 || stats.created > 0);
}

// ============================================================================
// Filtering Tests (3 tests)
// ============================================================================

#[tokio::test]
async fn test_language_filter() {
    let (engine, temp_dir) = setup_test_engine().await;
    let fixtures_dir = temp_dir.path().join("fixtures");
    create_test_fixtures(&fixtures_dir);

    engine.index_directory(&fixtures_dir).await.unwrap();

    let query = SearchQuery {
        query: "add".to_string(),
        mode: SearchMode::SemanticOnly,
        language_filter: Some("rust".to_string()),
        file_pattern: None,
        chunk_type_filter: None,
        limit: 10,
    };

    let results = engine.search(&query).await.unwrap();
    assert!(results.iter().all(|r| r.language == "rust"));
}

#[tokio::test]
async fn test_file_pattern_filter() {
    let (engine, temp_dir) = setup_test_engine().await;
    let fixtures_dir = temp_dir.path().join("fixtures");
    create_test_fixtures(&fixtures_dir);

    engine.index_directory(&fixtures_dir).await.unwrap();

    let query = SearchQuery {
        query: "function".to_string(),
        mode: SearchMode::SemanticOnly,
        language_filter: None,
        file_pattern: Some("*.rs".to_string()),
        chunk_type_filter: None,
        limit: 10,
    };

    let results = engine.search(&query).await.unwrap();
    assert!(results.iter().all(|r| r.file_path.ends_with(".rs")));
}

#[tokio::test]
async fn test_chunk_type_filter() {
    let (engine, temp_dir) = setup_test_engine().await;
    let fixtures_dir = temp_dir.path().join("fixtures");
    create_test_fixtures(&fixtures_dir);

    engine.index_directory(&fixtures_dir).await.unwrap();

    let query = SearchQuery {
        query: "code".to_string(),
        mode: SearchMode::SemanticOnly,
        language_filter: None,
        file_pattern: None,
        chunk_type_filter: Some(ChunkType::Class),
        limit: 10,
    };

    let results = engine.search(&query).await.unwrap();
    assert!(results.iter().all(|r| r.chunk_type == "class"));
}

// ============================================================================
// Empty Results Test (1 test)
// ============================================================================

#[tokio::test]
async fn test_empty_results() {
    let (engine, _temp_dir) = setup_test_engine().await;

    let query = SearchQuery {
        query: "xyzzy_nonexistent".to_string(),
        mode: SearchMode::SemanticOnly,
        language_filter: None,
        file_pattern: None,
        chunk_type_filter: None,
        limit: 10,
    };

    let results = engine.search(&query).await.unwrap();
    assert_eq!(results.len(), 0);
}

// ============================================================================
// Snippet Extraction Test (1 test)
// ============================================================================

#[tokio::test]
async fn test_snippet_extraction() {
    let (engine, temp_dir) = setup_test_engine().await;
    let fixtures_dir = temp_dir.path().join("fixtures");
    create_test_fixtures(&fixtures_dir);

    engine.index_directory(&fixtures_dir).await.unwrap();

    let query = SearchQuery {
        query: "function".to_string(),
        mode: SearchMode::SemanticOnly,
        language_filter: None,
        file_pattern: None,
        chunk_type_filter: None,
        limit: 1,
    };

    let results = engine.search(&query).await.unwrap();
    if !results.is_empty() {
        assert!(results[0].snippet.len() <= 200);
        assert!(!results[0].snippet.is_empty());
    }
}

// ============================================================================
// Result Ranking Test (1 test)
// ============================================================================

#[tokio::test]
async fn test_result_ranking() {
    let (engine, temp_dir) = setup_test_engine().await;
    let fixtures_dir = temp_dir.path().join("fixtures");
    create_test_fixtures(&fixtures_dir);

    engine.index_directory(&fixtures_dir).await.unwrap();

    let query = SearchQuery {
        query: "function".to_string(),
        mode: SearchMode::SemanticOnly,
        language_filter: None,
        file_pattern: None,
        chunk_type_filter: None,
        limit: 10,
    };

    let results = engine.search(&query).await.unwrap();

    // Results should be sorted by similarity score (descending)
    for i in 1..results.len() {
        assert!(results[i - 1].similarity_score >= results[i].similarity_score);
    }
}

// ============================================================================
// Statistics Test (1 test)
// ============================================================================

#[tokio::test]
async fn test_index_statistics() {
    let (engine, temp_dir) = setup_test_engine().await;
    let fixtures_dir = temp_dir.path().join("fixtures");
    create_test_fixtures(&fixtures_dir);

    let stats = engine.index_directory(&fixtures_dir).await.unwrap();

    assert!(stats.total_files > 0);
    assert!(stats.total_chunks > 0);
    assert_eq!(stats.created, stats.total_chunks); // First index
    assert_eq!(stats.updated, 0);
    assert_eq!(stats.skipped, 0);
    assert!(stats.duration_ms > 0);
}

// ============================================================================
// Embedding Count Test (1 test)
// ============================================================================

#[tokio::test]
async fn test_embedding_count() {
    let (engine, temp_dir) = setup_test_engine().await;

    let count1 = engine.embedding_count().await.unwrap();
    assert_eq!(count1, 0);

    let fixtures_dir = temp_dir.path().join("fixtures");
    create_test_fixtures(&fixtures_dir);
    engine.index_directory(&fixtures_dir).await.unwrap();

    let count2 = engine.embedding_count().await.unwrap();
    assert!(count2 > 0);
}

// ============================================================================
// Search Mode Tests (3 tests)
// ============================================================================

#[test]
fn test_search_mode_enum() {
    assert_eq!(SearchMode::SemanticOnly, SearchMode::SemanticOnly);
    assert_ne!(SearchMode::SemanticOnly, SearchMode::KeywordOnly);
}

#[test]
fn test_search_query_builder() {
    let query = SearchQuery {
        query: "test".to_string(),
        mode: SearchMode::SemanticOnly,
        language_filter: Some("rust".to_string()),
        file_pattern: Some("*.rs".to_string()),
        chunk_type_filter: Some(ChunkType::Function),
        limit: 5,
    };

    assert_eq!(query.query, "test");
    assert_eq!(query.limit, 5);
}

#[test]
fn test_index_stats_display() {
    let stats = IndexStats {
        total_files: 10,
        total_chunks: 50,
        created: 40,
        updated: 5,
        skipped: 5,
        duration_ms: 1000,
    };

    assert_eq!(stats.total_files, 10);
    assert_eq!(stats.total_chunks, 50);
    assert_eq!(stats.created + stats.updated + stats.skipped, stats.total_chunks);
}
