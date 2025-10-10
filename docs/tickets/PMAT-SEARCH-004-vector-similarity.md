# PMAT-SEARCH-004: Vector Similarity Search Engine

**Sprint**: 30
**Status**: 🔴 RED PHASE
**Estimated**: 2.5 hours
**Actual**: TBD

## 🎯 Objective

Build high-level semantic search engine that orchestrates chunking, embedding, and vector search into a unified API for code discovery.

## 📋 Requirements

**Must Support:**
- Search code by natural language query
- Find similar code to a reference file/function
- Filter by language, file pattern, chunk type
- Multiple search modes: semantic-only, keyword-only, hybrid
- Incremental embedding updates (only changed files)
- Search result ranking with explainability

**Search Interface:**
```rust
pub struct SemanticSearchEngine {
    chunker: Arc<CodeChunker>,
    embeddings_client: Arc<OpenAIEmbeddingsClient>,
    vector_db: Arc<TursoVectorDB>,
}

pub struct SearchQuery {
    pub query: String,
    pub mode: SearchMode,
    pub language_filter: Option<String>,
    pub file_pattern: Option<String>,
    pub chunk_type_filter: Option<ChunkType>,
    pub limit: usize,
}

pub struct SearchResult {
    pub file_path: String,
    pub chunk_name: String,
    pub chunk_type: String,
    pub language: String,
    pub similarity_score: f64,
    pub snippet: String, // First 200 chars of code
    pub start_line: usize,
    pub end_line: usize,
}
```

## 🔴 RED Phase: Tests First

### Test Suite

```rust
// tests/unit_semantic_search_engine.rs

#[tokio::test]
async fn test_search_by_query() {
    let engine = setup_test_engine().await;

    // Index some test code
    engine.index_directory("tests/fixtures/sample_code").await?;

    // Search for semantic concept
    let query = SearchQuery {
        query: "function that adds two numbers".to_string(),
        mode: SearchMode::SemanticOnly,
        language_filter: None,
        file_pattern: None,
        chunk_type_filter: Some(ChunkType::Function),
        limit: 10,
    };

    let results = engine.search(&query).await?;
    assert!(results.len() > 0);
    assert!(results[0].chunk_name.contains("add") || results[0].snippet.contains("+ "));
    assert!(results[0].similarity_score > 0.7);
}

#[tokio::test]
async fn test_find_similar_code() {
    let engine = setup_test_engine().await;
    engine.index_directory("tests/fixtures/sample_code").await?;

    // Find code similar to reference
    let results = engine
        .find_similar("tests/fixtures/sample_code/math.rs", 5)
        .await?;

    assert_eq!(results.len(), 5);
    assert!(results[0].similarity_score > results[4].similarity_score); // Descending order
}

#[tokio::test]
async fn test_incremental_update() {
    let engine = setup_test_engine().await;

    // Initial index
    engine.index_directory("tests/fixtures/sample_code").await?;
    let count1 = engine.embedding_count().await?;

    // Modify a file (change content, checksum changes)
    modify_test_file("tests/fixtures/sample_code/math.rs");

    // Re-index (should only update changed chunks)
    let stats = engine.index_directory("tests/fixtures/sample_code").await?;
    let count2 = engine.embedding_count().await?;

    assert_eq!(count2, count1); // Same number of chunks
    assert!(stats.updated > 0); // But some were updated
    assert!(stats.created == 0); // No new chunks
}

#[tokio::test]
async fn test_language_filter() {
    let engine = setup_test_engine().await;
    engine.index_directory("tests/fixtures/multilang").await?;

    let query = SearchQuery {
        query: "calculate sum".to_string(),
        mode: SearchMode::SemanticOnly,
        language_filter: Some("rust".to_string()),
        file_pattern: None,
        chunk_type_filter: None,
        limit: 10,
    };

    let results = engine.search(&query).await?;
    assert!(results.iter().all(|r| r.language == "rust"));
}

#[tokio::test]
async fn test_file_pattern_filter() {
    let engine = setup_test_engine().await;
    engine.index_directory("tests/fixtures/sample_code").await?;

    let query = SearchQuery {
        query: "function".to_string(),
        mode: SearchMode::SemanticOnly,
        language_filter: None,
        file_pattern: Some("**/utils/*.rs".to_string()),
        chunk_type_filter: None,
        limit: 10,
    };

    let results = engine.search(&query).await?;
    assert!(results.iter().all(|r| r.file_path.contains("/utils/")));
}

#[tokio::test]
async fn test_chunk_type_filter() {
    let engine = setup_test_engine().await;
    engine.index_directory("tests/fixtures/sample_code").await?;

    let query = SearchQuery {
        query: "code".to_string(),
        mode: SearchMode::SemanticOnly,
        language_filter: None,
        file_pattern: None,
        chunk_type_filter: Some(ChunkType::Class),
        limit: 10,
    };

    let results = engine.search(&query).await?;
    assert!(results.iter().all(|r| r.chunk_type == "class"));
}

#[tokio::test]
async fn test_empty_results() {
    let engine = setup_test_engine().await;
    engine.index_directory("tests/fixtures/sample_code").await?;

    let query = SearchQuery {
        query: "xyzzy_nonexistent_concept_12345".to_string(),
        mode: SearchMode::SemanticOnly,
        language_filter: None,
        file_pattern: None,
        chunk_type_filter: None,
        limit: 10,
    };

    let results = engine.search(&query).await?;
    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn test_snippet_extraction() {
    let engine = setup_test_engine().await;
    engine.index_directory("tests/fixtures/sample_code").await?;

    let query = SearchQuery {
        query: "function".to_string(),
        mode: SearchMode::SemanticOnly,
        language_filter: None,
        file_pattern: None,
        chunk_type_filter: None,
        limit: 1,
    };

    let results = engine.search(&query).await?;
    assert!(results.len() > 0);
    assert!(results[0].snippet.len() <= 200);
    assert!(!results[0].snippet.is_empty());
}

#[tokio::test]
async fn test_result_ranking() {
    let engine = setup_test_engine().await;
    engine.index_directory("tests/fixtures/sample_code").await?;

    let query = SearchQuery {
        query: "add two numbers".to_string(),
        mode: SearchMode::SemanticOnly,
        language_filter: None,
        file_pattern: None,
        chunk_type_filter: None,
        limit: 10,
    };

    let results = engine.search(&query).await?;

    // Results should be sorted by similarity score (descending)
    for i in 1..results.len() {
        assert!(results[i - 1].similarity_score >= results[i].similarity_score);
    }
}

#[tokio::test]
async fn test_index_statistics() {
    let engine = setup_test_engine().await;

    let stats = engine.index_directory("tests/fixtures/sample_code").await?;

    assert!(stats.total_files > 0);
    assert!(stats.total_chunks > 0);
    assert!(stats.created == stats.total_chunks); // First index
    assert_eq!(stats.updated, 0);
    assert_eq!(stats.skipped, 0);
}
```

**Total Tests**: 18
- Search by query (3 tests)
- Find similar code (1 test)
- Incremental updates (2 tests)
- Filtering (3 tests)
- Empty results (1 test)
- Snippet extraction (1 test)
- Result ranking (1 test)
- Statistics (1 test)
- Performance (5 tests - separate suite)

## 🟢 GREEN Phase: Implementation

**File**: `server/src/services/semantic/search_engine.rs`

**Key Structures:**

```rust
pub struct SemanticSearchEngine {
    vector_db: Arc<TursoVectorDB>,
    embeddings_client: Arc<OpenAIEmbeddingsClient>,
}

pub struct SearchQuery {
    pub query: String,
    pub mode: SearchMode,
    pub language_filter: Option<String>,
    pub file_pattern: Option<String>,
    pub chunk_type_filter: Option<ChunkType>,
    pub limit: usize,
}

pub struct IndexStats {
    pub total_files: usize,
    pub total_chunks: usize,
    pub created: usize,
    pub updated: usize,
    pub skipped: usize,
    pub duration_ms: u64,
}
```

**Key Functions:**
- `new(api_key: &str, db_path: &str) -> Result<Self>`
- `search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>>`
- `find_similar(&self, file_path: &str, limit: usize) -> Result<Vec<SearchResult>>`
- `index_directory(&self, path: &Path) -> Result<IndexStats>`
- `embedding_count(&self) -> Result<usize>`

## 🔵 REFACTOR Phase: Quality

**Complexity Target**: ≤10 cyclomatic per function
**Coverage Target**: ≥95%
**SATD**: 0 violations

**Refactoring checklist:**
- Extract indexing logic to separate module
- Extract filtering logic to builder pattern
- Add progress reporting for large directories
- Add caching for repeated queries
- Document search algorithms

## ✅ Exit Criteria

- [ ] 18 tests passing
- [ ] Unified search API for query and similarity search
- [ ] Incremental updates based on checksums
- [ ] Filtering by language, file pattern, chunk type
- [ ] Result ranking by similarity score
- [ ] Snippet extraction (200 chars)
- [ ] Index statistics tracking
- [ ] Cyclomatic ≤10 for all functions
- [ ] Zero clippy warnings

## 🔗 Integration

Will be used by:
- PMAT-SEARCH-005: Hybrid Search Engine (combine with ripgrep)
- PMAT-SEARCH-006: MCP Tools (expose via protocol)
- PMAT-SEARCH-009: CLI Commands (user interface)

## 🚀 Usage Examples

```rust
// Create engine
let engine = SemanticSearchEngine::new("sk-...", "embeddings.db").await?;

// Index codebase
let stats = engine.index_directory("src/").await?;
println!("Indexed {} chunks from {} files", stats.total_chunks, stats.total_files);

// Search by natural language
let results = engine.search(&SearchQuery {
    query: "error handling with Result type".to_string(),
    mode: SearchMode::SemanticOnly,
    language_filter: Some("rust".to_string()),
    file_pattern: None,
    chunk_type_filter: Some(ChunkType::Function),
    limit: 5,
}).await?;

for result in results {
    println!("{}: {} (score: {:.2})",
        result.file_path,
        result.chunk_name,
        result.similarity_score
    );
}

// Find similar code
let similar = engine.find_similar("src/parser.rs", 10).await?;
```
