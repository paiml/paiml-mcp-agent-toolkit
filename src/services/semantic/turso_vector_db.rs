#![allow(unused)]
#![cfg_attr(coverage_nightly, coverage(off))]
// Turso Vector Database Integration
// PMAT-SEARCH-003: Store and query code embeddings using SIMD-accelerated VectorStore
//
// GREEN Phase: Full implementation with cosine similarity search
// Sprint 76: Migrated from rusqlite to trueno-rag VectorStore for SIMD acceleration

use std::collections::HashMap;
use std::path::Path;
use std::sync::RwLock;
use trueno_rag::index::{VectorStore, VectorStoreConfig};
use trueno_rag::{Chunk, ChunkId, DocumentId};

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
}
