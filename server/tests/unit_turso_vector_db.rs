// RED Phase: Write failing tests first
// PMAT-SEARCH-003: Turso Vector Database Integration
// Test count: 12 tests

use pmat::services::semantic::turso_vector_db::*;
use std::fs;

// Helper function to create test entries
fn create_test_entry(file_path: &str, chunk_name: &str, embedding: Vec<f32>) -> EmbeddingEntry {
    // Pad embedding to 1536 if needed
    let mut padded_embedding = embedding;
    while padded_embedding.len() < 1536 {
        padded_embedding.push(0.0);
    }

    EmbeddingEntry {
        file_path: file_path.to_string(),
        chunk_name: chunk_name.to_string(),
        chunk_type: "function".to_string(),
        language: "rust".to_string(),
        start_line: 10,
        end_line: 12,
        content_checksum: "test_checksum".to_string(),
        embedding: padded_embedding,
        model: "text-embedding-3-small".to_string(),
    }
}

// Cleanup helper
fn cleanup_test_db(path: &str) {
    let _ = fs::remove_file(path);
}

// ============================================================================
// Insert/Upsert Tests (2 tests)
// ============================================================================

#[tokio::test]
async fn test_insert_embedding() {
    let db_path = "test_insert.db";
    cleanup_test_db(db_path);

    let db = TursoVectorDB::new_local(db_path).await.unwrap();

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

    let id = db.insert(&entry).await.unwrap();
    assert!(id > 0);

    cleanup_test_db(db_path);
}

#[tokio::test]
async fn test_upsert_on_duplicate() {
    let db_path = "test_upsert.db";
    cleanup_test_db(db_path);

    let db = TursoVectorDB::new_local(db_path).await.unwrap();

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

    let id1 = db.insert(&entry).await.unwrap();
    let id2 = db.insert(&entry).await.unwrap(); // Should update, not error
    assert_eq!(id1, id2);

    cleanup_test_db(db_path);
}

// ============================================================================
// Query Operations Tests (2 tests)
// ============================================================================

#[tokio::test]
async fn test_query_by_file() {
    let db_path = "test_query_file.db";
    cleanup_test_db(db_path);

    let db = TursoVectorDB::new_local(db_path).await.unwrap();

    // Insert test data
    let entry1 = create_test_entry("src/main.rs", "fn1", vec![0.1; 1536]);
    let entry2 = create_test_entry("src/main.rs", "fn2", vec![0.2; 1536]);
    let entry3 = create_test_entry("src/lib.rs", "fn3", vec![0.3; 1536]);

    db.insert(&entry1).await.unwrap();
    db.insert(&entry2).await.unwrap();
    db.insert(&entry3).await.unwrap();

    let results = db.query_by_file("src/main.rs").await.unwrap();
    assert_eq!(results.len(), 2);

    cleanup_test_db(db_path);
}

#[tokio::test]
async fn test_query_by_language() {
    let db_path = "test_query_lang.db";
    cleanup_test_db(db_path);

    let db = TursoVectorDB::new_local(db_path).await.unwrap();

    let mut entry1 = create_test_entry("src/main.rs", "fn1", vec![0.1; 1536]);
    entry1.language = "rust".to_string();

    let mut entry2 = create_test_entry("src/main.py", "fn2", vec![0.2; 1536]);
    entry2.language = "python".to_string();

    db.insert(&entry1).await.unwrap();
    db.insert(&entry2).await.unwrap();

    let results = db.query_by_language("rust").await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].language, "rust");

    cleanup_test_db(db_path);
}

// ============================================================================
// Similarity Search Tests (2 tests)
// ============================================================================

#[tokio::test]
async fn test_vector_similarity_search() {
    let db_path = "test_similarity.db";
    cleanup_test_db(db_path);

    let db = TursoVectorDB::new_local(db_path).await.unwrap();

    // Insert embeddings with distinct patterns
    let mut entry1 = create_test_entry("src/main.rs", "add", vec![1.0, 0.0, 0.0]);
    entry1.embedding[0] = 1.0;
    entry1.embedding[1] = 0.0;
    entry1.embedding[2] = 0.0;

    let mut entry2 = create_test_entry("src/lib.rs", "multiply", vec![0.9, 0.1, 0.0]);
    entry2.embedding[0] = 0.9;
    entry2.embedding[1] = 0.1;
    entry2.embedding[2] = 0.0;

    let mut entry3 = create_test_entry("src/utils.rs", "divide", vec![0.0, 1.0, 0.0]);
    entry3.embedding[0] = 0.0;
    entry3.embedding[1] = 1.0;
    entry3.embedding[2] = 0.0;

    db.insert(&entry1).await.unwrap();
    db.insert(&entry2).await.unwrap();
    db.insert(&entry3).await.unwrap();

    // Query with vector similar to entry1
    let mut query_vector = vec![0.0; 1536];
    query_vector[0] = 0.95;
    query_vector[1] = 0.05;
    query_vector[2] = 0.0;

    let results = db.similarity_search(&query_vector, 2).await.unwrap();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].chunk_name, "add"); // Most similar
    assert!(results[0].similarity > 0.9);

    cleanup_test_db(db_path);
}

#[tokio::test]
async fn test_similarity_with_limit() {
    let db_path = "test_similarity_limit.db";
    cleanup_test_db(db_path);

    let db = TursoVectorDB::new_local(db_path).await.unwrap();

    // Insert 5 entries
    for i in 0..5 {
        let entry = create_test_entry(&format!("src/file{i}.rs"), &format!("fn{i}"), vec![0.1; 1536]);
        db.insert(&entry).await.unwrap();
    }

    let query_vector = vec![0.1; 1536];
    let results = db.similarity_search(&query_vector, 3).await.unwrap();

    assert_eq!(results.len(), 3); // Limited to 3

    cleanup_test_db(db_path);
}

// ============================================================================
// Batch Operations Test (1 test)
// ============================================================================

#[tokio::test]
async fn test_batch_insert() {
    let db_path = "test_batch.db";
    cleanup_test_db(db_path);

    let db = TursoVectorDB::new_local(db_path).await.unwrap();

    let entries = vec![
        create_test_entry("src/a.rs", "fn1", vec![0.1; 1536]),
        create_test_entry("src/b.rs", "fn2", vec![0.2; 1536]),
        create_test_entry("src/c.rs", "fn3", vec![0.3; 1536]),
    ];

    let ids = db.batch_insert(&entries).await.unwrap();
    assert_eq!(ids.len(), 3);
    assert!(ids.iter().all(|id| *id > 0));

    cleanup_test_db(db_path);
}

// ============================================================================
// Deletion Test (1 test)
// ============================================================================

#[tokio::test]
async fn test_delete_by_file() {
    let db_path = "test_delete.db";
    cleanup_test_db(db_path);

    let db = TursoVectorDB::new_local(db_path).await.unwrap();

    let entry = create_test_entry("src/main.rs", "add", vec![0.1; 1536]);
    db.insert(&entry).await.unwrap();

    let deleted = db.delete_by_file("src/main.rs").await.unwrap();
    assert_eq!(deleted, 1);

    let results = db.query_by_file("src/main.rs").await.unwrap();
    assert_eq!(results.len(), 0);

    cleanup_test_db(db_path);
}

// ============================================================================
// Checksum Handling Test (1 test)
// ============================================================================

#[tokio::test]
async fn test_checksum_invalidation() {
    let db_path = "test_checksum.db";
    cleanup_test_db(db_path);

    let db = TursoVectorDB::new_local(db_path).await.unwrap();

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
    db.insert(&entry1).await.unwrap();

    // Update with checksum2 (content changed)
    let entry2 = EmbeddingEntry {
        content_checksum: "checksum2".to_string(),
        ..entry1
    };
    db.insert(&entry2).await.unwrap();

    // Should have 2 entries (different checksums)
    let results = db.query_by_file("src/main.rs").await.unwrap();
    assert_eq!(results.len(), 2);

    cleanup_test_db(db_path);
}

// ============================================================================
// Similarity Calculation Test (1 test)
// ============================================================================

#[test]
fn test_cosine_similarity_calculation() {
    // Test similarity computation (no DB needed)
    let v1 = vec![1.0, 0.0, 0.0];
    let v2 = vec![1.0, 0.0, 0.0];
    let similarity = TursoVectorDB::cosine_similarity(&v1, &v2);
    assert!((similarity - 1.0).abs() < 0.001); // Identical vectors

    let v3 = vec![0.0, 1.0, 0.0];
    let similarity2 = TursoVectorDB::cosine_similarity(&v1, &v3);
    assert!((similarity2 - 0.0).abs() < 0.001); // Orthogonal vectors

    let v4 = vec![0.6, 0.8, 0.0];
    let v5 = vec![0.8, 0.6, 0.0];
    let similarity3 = TursoVectorDB::cosine_similarity(&v4, &v5);
    assert!(similarity3 > 0.9); // Similar vectors
}

// ============================================================================
// Edge Cases Tests (2 tests)
// ============================================================================

#[tokio::test]
async fn test_empty_database_query() {
    let db_path = "test_empty.db";
    cleanup_test_db(db_path);

    let db = TursoVectorDB::new_local(db_path).await.unwrap();

    let results = db.query_by_file("nonexistent.rs").await.unwrap();
    assert_eq!(results.len(), 0);

    let query_vector = vec![0.5; 1536];
    let search_results = db.similarity_search(&query_vector, 10).await.unwrap();
    assert_eq!(search_results.len(), 0);

    cleanup_test_db(db_path);
}

#[tokio::test]
async fn test_large_batch_insert() {
    let db_path = "test_large_batch.db";
    cleanup_test_db(db_path);

    let db = TursoVectorDB::new_local(db_path).await.unwrap();

    // Create 100 entries
    let entries: Vec<EmbeddingEntry> = (0..100)
        .map(|i| create_test_entry(&format!("src/file{i}.rs"), &format!("fn{i}"), vec![0.1; 1536]))
        .collect();

    let ids = db.batch_insert(&entries).await.unwrap();
    assert_eq!(ids.len(), 100);

    cleanup_test_db(db_path);
}
