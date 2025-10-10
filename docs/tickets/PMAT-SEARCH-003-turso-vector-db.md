# PMAT-SEARCH-003: Turso Vector Database Integration

**Sprint**: 29
**Status**: 🔴 RED PHASE
**Estimated**: 3 hours
**Actual**: TBD

## 🎯 Objective

Implement Turso vector database integration for storing and querying code embeddings using SQLite with JSON-based vector storage.

## 📋 Requirements

**Must Support:**
- Store embeddings with metadata (file_path, chunk_name, language)
- Query by vector similarity (cosine similarity)
- Incremental updates (upsert semantics)
- Batch operations for efficiency
- Local mode for development/testing
- Content checksums for cache invalidation

**Database Schema:**
```sql
CREATE TABLE IF NOT EXISTS code_embeddings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_path TEXT NOT NULL,
    chunk_name TEXT NOT NULL,
    chunk_type TEXT NOT NULL, -- "function" | "class" | "module" | "file"
    language TEXT NOT NULL,
    start_line INTEGER NOT NULL,
    end_line INTEGER NOT NULL,
    content_checksum TEXT NOT NULL, -- SHA256 hex
    embedding TEXT NOT NULL, -- JSON array of 1536 floats
    model TEXT NOT NULL, -- "text-embedding-3-small"
    created_at INTEGER NOT NULL, -- Unix timestamp
    UNIQUE(file_path, chunk_name, content_checksum)
);

CREATE INDEX IF NOT EXISTS idx_file_path ON code_embeddings(file_path);
CREATE INDEX IF NOT EXISTS idx_language ON code_embeddings(language);
CREATE INDEX IF NOT EXISTS idx_checksum ON code_embeddings(content_checksum);
```

## 🔴 RED Phase: Tests First

### Test Suite

```rust
// tests/unit_turso_vector_db.rs

#[tokio::test]
async fn test_insert_embedding() {
    let db = TursoVectorDB::new_local("test.db").await?;

    let entry = EmbeddingEntry {
        file_path: "src/main.rs".to_string(),
        chunk_name: "add".to_string(),
        chunk_type: "function".to_string(),
        language: "rust".to_string(),
        start_line: 10,
        end_line: 12,
        content_checksum: "abc123".to_string(),
        embedding: vec![0.1; 1536],
        model: "text-embedding-3-small".to_string(),
    };

    let id = db.insert(&entry).await?;
    assert!(id > 0);
}

#[tokio::test]
async fn test_upsert_on_duplicate() {
    let db = TursoVectorDB::new_local("test.db").await?;

    let entry = EmbeddingEntry {
        file_path: "src/main.rs".to_string(),
        chunk_name: "add".to_string(),
        chunk_type: "function".to_string(),
        language: "rust".to_string(),
        start_line: 10,
        end_line: 12,
        content_checksum: "abc123".to_string(),
        embedding: vec![0.1; 1536],
        model: "text-embedding-3-small".to_string(),
    };

    let id1 = db.insert(&entry).await?;
    let id2 = db.insert(&entry).await?; // Should update, not error
    assert_eq!(id1, id2);
}

#[tokio::test]
async fn test_query_by_file() {
    let db = TursoVectorDB::new_local("test.db").await?;

    // Insert test data
    let entry1 = create_test_entry("src/main.rs", "fn1", vec![0.1; 1536]);
    let entry2 = create_test_entry("src/main.rs", "fn2", vec![0.2; 1536]);
    let entry3 = create_test_entry("src/lib.rs", "fn3", vec![0.3; 1536]);

    db.insert(&entry1).await?;
    db.insert(&entry2).await?;
    db.insert(&entry3).await?;

    let results = db.query_by_file("src/main.rs").await?;
    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn test_vector_similarity_search() {
    let db = TursoVectorDB::new_local("test.db").await?;

    // Insert embeddings
    let entry1 = create_test_entry("src/main.rs", "add", vec![1.0, 0.0, 0.0]);
    let entry2 = create_test_entry("src/lib.rs", "multiply", vec![0.9, 0.1, 0.0]);
    let entry3 = create_test_entry("src/utils.rs", "divide", vec![0.0, 1.0, 0.0]);

    db.insert(&entry1).await?;
    db.insert(&entry2).await?;
    db.insert(&entry3).await?;

    // Query with vector similar to entry1
    let query_vector = vec![0.95, 0.05, 0.0];
    let results = db.similarity_search(&query_vector, 2).await?;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].chunk_name, "add"); // Most similar
    assert!(results[0].similarity > 0.9);
}

#[tokio::test]
async fn test_batch_insert() {
    let db = TursoVectorDB::new_local("test.db").await?;

    let entries = vec![
        create_test_entry("src/a.rs", "fn1", vec![0.1; 1536]),
        create_test_entry("src/b.rs", "fn2", vec![0.2; 1536]),
        create_test_entry("src/c.rs", "fn3", vec![0.3; 1536]),
    ];

    let ids = db.batch_insert(&entries).await?;
    assert_eq!(ids.len(), 3);
}

#[tokio::test]
async fn test_delete_by_file() {
    let db = TursoVectorDB::new_local("test.db").await?;

    let entry = create_test_entry("src/main.rs", "add", vec![0.1; 1536]);
    db.insert(&entry).await?;

    let deleted = db.delete_by_file("src/main.rs").await?;
    assert_eq!(deleted, 1);

    let results = db.query_by_file("src/main.rs").await?;
    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn test_checksum_invalidation() {
    let db = TursoVectorDB::new_local("test.db").await?;

    // Insert with checksum1
    let entry1 = EmbeddingEntry {
        file_path: "src/main.rs".to_string(),
        chunk_name: "add".to_string(),
        chunk_type: "function".to_string(),
        language: "rust".to_string(),
        start_line: 10,
        end_line: 12,
        content_checksum: "checksum1".to_string(),
        embedding: vec![0.1; 1536],
        model: "text-embedding-3-small".to_string(),
    };
    db.insert(&entry1).await?;

    // Update with checksum2 (content changed)
    let entry2 = EmbeddingEntry {
        content_checksum: "checksum2".to_string(),
        ..entry1
    };
    db.insert(&entry2).await?;

    // Should have 2 entries (different checksums)
    let results = db.query_by_file("src/main.rs").await?;
    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn test_cosine_similarity_calculation() {
    let db = TursoVectorDB::new_local("test.db").await?;

    // Test similarity computation
    let v1 = vec![1.0, 0.0, 0.0];
    let v2 = vec![1.0, 0.0, 0.0];
    let similarity = db.cosine_similarity(&v1, &v2);
    assert!((similarity - 1.0).abs() < 0.001); // Identical vectors

    let v3 = vec![0.0, 1.0, 0.0];
    let similarity2 = db.cosine_similarity(&v1, &v3);
    assert!((similarity2 - 0.0).abs() < 0.001); // Orthogonal vectors
}
```

**Total Tests**: 12
- Insert/upsert (2 tests)
- Query operations (2 tests)
- Similarity search (2 tests)
- Batch operations (1 test)
- Deletion (1 test)
- Checksum handling (1 test)
- Similarity calculation (1 test)
- Edge cases (2 tests)

## 🟢 GREEN Phase: Implementation

**File**: `server/src/services/semantic/turso_vector_db.rs`

**Key Structures:**

```rust
pub struct TursoVectorDB {
    connection: rusqlite::Connection,
}

pub struct EmbeddingEntry {
    pub file_path: String,
    pub chunk_name: String,
    pub chunk_type: String,
    pub language: String,
    pub start_line: usize,
    pub end_line: usize,
    pub content_checksum: String,
    pub embedding: Vec<f32>,
    pub model: String,
}

pub struct SearchResult {
    pub id: i64,
    pub file_path: String,
    pub chunk_name: String,
    pub chunk_type: String,
    pub language: String,
    pub start_line: usize,
    pub end_line: usize,
    pub similarity: f64,
    pub embedding: Vec<f32>,
}
```

**Key Functions:**
- `new_local(path: &str) -> Result<Self>` - Create local SQLite database
- `insert(&self, entry: &EmbeddingEntry) -> Result<i64>` - Insert/upsert embedding
- `batch_insert(&self, entries: &[EmbeddingEntry]) -> Result<Vec<i64>>` - Batch insert
- `similarity_search(&self, query: &[f32], limit: usize) -> Result<Vec<SearchResult>>` - Vector search
- `query_by_file(&self, file_path: &str) -> Result<Vec<SearchResult>>` - Get all embeddings for file
- `delete_by_file(&self, file_path: &str) -> Result<usize>` - Delete embeddings for file
- `cosine_similarity(&self, v1: &[f32], v2: &[f32]) -> f64` - Compute cosine similarity

**Dependencies:**
- `rusqlite` for SQLite database
- `serde_json` for JSON vector serialization

## 🔵 REFACTOR Phase: Quality

**Complexity Target**: ≤10 cyclomatic per function
**Coverage Target**: ≥95%
**SATD**: 0 violations

**Refactoring checklist:**
- Extract SQL queries to constants
- Extract similarity computation to separate module
- Add connection pooling for concurrent access
- Add comprehensive error types
- Document vector format expectations
- Add query optimization hints

## ✅ Exit Criteria

- [ ] 12 tests passing
- [ ] Supports insert/upsert with UNIQUE constraint
- [ ] Vector similarity search with cosine distance
- [ ] Batch operations complete in <1s for 100 entries
- [ ] Checksums enable incremental updates
- [ ] Local mode works without external dependencies
- [ ] Cyclomatic ≤10 for all functions
- [ ] Zero clippy warnings

## 📊 Performance Targets

**Similarity Search**:
- 10 vectors: <10ms
- 100 vectors: <50ms
- 1,000 vectors: <200ms
- 10,000 vectors: <1s (may need optimization)

**Batch Insert**:
- 100 entries: <500ms
- 1,000 entries: <5s

## 🔗 Integration

Will be used by:
- PMAT-SEARCH-004: Vector Similarity Search (query interface)
- PMAT-SEARCH-005: Hybrid Search Engine (combine with ripgrep)
- PMAT-SEARCH-009: CLI Commands (database management)

## 💡 Future Optimizations

**Phase 2 (if needed)**:
- Add FAISS index for >10K vectors
- Add quantization for smaller storage
- Add remote Turso support
- Add vector compression
- Add approximate nearest neighbor (ANN)
