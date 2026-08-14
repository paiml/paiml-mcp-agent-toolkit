#![allow(unused)]
#![cfg_attr(coverage_nightly, coverage(off))]
// Turso Vector Database Integration
// PMAT-SEARCH-003: Store and query code embeddings using SIMD-accelerated VectorStore
//
// GREEN Phase: Full implementation with cosine similarity search
// Sprint 76: Migrated from rusqlite to trueno-rag VectorStore for SIMD acceleration

use aprender_rag::index::{VectorStore, VectorStoreConfig};
use aprender_rag::{Chunk, ChunkId, DocumentId};
use std::collections::HashMap;
use std::path::Path;
use std::sync::RwLock;

// Type definitions: TursoVectorDB, EmbeddingMetadata, EmbeddingEntry, SearchResult, DbStats
include!("turso_vector_db_types.rs");

// Core implementation: new_local, insert, batch_insert, query, search, delete, stats, cosine similarity
include!("turso_vector_db_impl.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    // Unit tests: struct creation, cosine similarity edge cases, DB operations
    include!("turso_vector_db_tests_unit.rs");

    // SIMD cosine similarity tests: equivalence, performance, edge cases
    include!("turso_vector_db_tests_simd.rs");

    // Regression: an embedding of a different dimension used to silently
    // replace the whole VectorStore, destroying every vector already indexed.
    mod dimension_change_regression {
        use super::super::{EmbeddingEntry, TursoVectorDB};

        fn entry(file: &str, name: &str, embedding: Vec<f32>) -> EmbeddingEntry {
            EmbeddingEntry {
                file_path: file.to_string(),
                chunk_name: name.to_string(),
                chunk_type: "function".to_string(),
                language: "rust".to_string(),
                start_line: 1,
                end_line: 10,
                content_checksum: format!("sum_{name}"),
                embedding,
                model: "aprender-tfidf-local".to_string(),
            }
        }

        /// A mismatched-dimension insert must be rejected, not allowed to wipe
        /// the index. The old code replaced `store` wholesale while leaving
        /// `metadata`/`file_index` populated, so the DB reported files it no
        /// longer had any vectors for.
        #[tokio::test]
        async fn mismatched_dimension_insert_does_not_destroy_the_index() {
            let db = TursoVectorDB::new_local(":memory:").await.unwrap();
            db.insert(&entry("src/a.rs", "alpha", vec![1.0, 0.0, 0.0, 0.0]))
                .await
                .unwrap();
            db.insert(&entry("src/b.rs", "beta", vec![0.0, 1.0, 0.0, 0.0]))
                .await
                .unwrap();

            let before = db.get_stats().await.unwrap();
            assert_eq!(before.total_entries, 2);
            assert_eq!(before.unique_files, 2);

            let err = db
                .insert(&entry("src/c.rs", "gamma", vec![0.25; 8]))
                .await
                .expect_err("a dimension change against a populated store must be an error");
            assert!(
                err.contains("dimension"),
                "error must name the dimension mismatch, got: {err}"
            );

            let after = db.get_stats().await.unwrap();
            assert_eq!(
                after.total_entries, 2,
                "vectors already indexed must survive a rejected insert"
            );
            assert_eq!(
                after.unique_files, 2,
                "the rejected file must not appear in the file index"
            );
        }

        /// The user-visible harm: `save()` serialized `embedding: []` for every
        /// chunk the wipe had orphaned (`store.get(..).unwrap_or_default()`),
        /// writing a corrupt db file that reloads as an empty search index.
        #[tokio::test]
        async fn save_never_persists_a_fabricated_empty_embedding() {
            let dir = tempfile::tempdir().unwrap();
            let db_path = dir.path().join("embeddings.db");
            let path_str = db_path.to_string_lossy().to_string();

            {
                let db = TursoVectorDB::new_local(path_str.as_str()).await.unwrap();
                db.insert(&entry("src/a.rs", "alpha", vec![1.0, 0.0, 0.0, 0.0]))
                    .await
                    .unwrap();
                db.insert(&entry("src/b.rs", "beta", vec![0.0, 1.0, 0.0, 0.0]))
                    .await
                    .unwrap();
                // Simulates switching embedding model mid-run.
                let _ = db.insert(&entry("src/c.rs", "gamma", vec![0.25; 8])).await;
                db.save().await.unwrap();
            }

            let json = std::fs::read_to_string(&db_path).unwrap();
            let persisted: Vec<EmbeddingEntry> = serde_json::from_str(&json).unwrap();
            for e in &persisted {
                assert!(
                    !e.embedding.is_empty(),
                    "save() persisted a fabricated empty embedding for {}::{}",
                    e.file_path,
                    e.chunk_name
                );
            }

            let reloaded = TursoVectorDB::new_local(path_str.as_str()).await.unwrap();
            let stats = reloaded.get_stats().await.unwrap();
            assert_eq!(
                stats.total_entries, 2,
                "both original vectors must reload as searchable entries"
            );
        }
    }
}
