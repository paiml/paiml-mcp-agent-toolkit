#![cfg_attr(coverage_nightly, coverage(off))]
// Git History Index (GH-RAG-003)
// Toyota Way: Poka-Yoke - Error-proof schema constraints
// Spec: docs/specifications/git-history-rag-integration.md

use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

// Type definitions: ChangeType, FileChange, CommitInfo, SyncResult, GitHistoryError, GitHistoryIndex
include!("git_history_types.rs");

// Schema initialization: open, in_memory, init_schema
include!("git_history_schema.rs");

// Write operations: insert_commit, insert_commits, update_embedding, sync_incremental
include!("git_history_mutations.rs");

// Read operations: queries, metadata, checksum
include!("git_history_queries.rs");

// Tests
include!("git_history_tests.rs");
