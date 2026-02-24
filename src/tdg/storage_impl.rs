use super::storage_backend::{StorageBackend, StorageBackendFactory, StorageConfig};
use super::TdgScore;
use anyhow::{anyhow, Result};
use blake3::Hash as Blake3Hash;
use dashmap::DashMap;
use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

// Data types: FileIdentity, ComponentScores, SemanticSignature, AnalysisMetadata,
// FullTdgRecord, HotCacheEntry, StorageStatistics
include!("storage_impl_types.rs");

// TieredStore struct definition and core methods:
// new, with_config, in_memory, store, get_hot, retrieve_full,
// should_archive, archive_to_cold, cleanup_hot_cache, migrate_backend,
// flush, get_statistics
include!("storage_impl_tiered.rs");

// TieredStore git query methods: get_by_commit, get_all_with_git_context, get_by_path
// TieredStorageFactory
include!("storage_impl_queries.rs");

// Tests
include!("storage_impl_tests.rs");
