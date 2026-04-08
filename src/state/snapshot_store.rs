#![cfg_attr(coverage_nightly, coverage(off))]
use super::*;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use parking_lot::RwLock;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::sync::Arc;
use tokio::fs::{create_dir_all, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// Snapshot storage with compression and integrity checks
/// Snapshot store.
pub struct SnapshotStore {
    snapshots: Arc<RwLock<Vec<SnapshotMetadata>>>,
    base_path: String,
    config: SnapshotConfig,
}

#[derive(Clone)]
/// Configuration for snapshot.
pub struct SnapshotConfig {
    pub max_snapshots: usize,
    pub compression_level: u32,
    pub verify_on_write: bool,
    pub verify_on_read: bool,
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            max_snapshots: 10,
            compression_level: 6,
            verify_on_write: true,
            verify_on_read: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Metadata for snapshot.
pub struct SnapshotMetadata {
    pub id: SnapshotId,
    pub timestamp: SystemTime,
    pub event_id: EventId,
    pub checksum: String,
    pub size_bytes: usize,
    pub compressed_size: usize,
    pub partition_key: Option<String>,
}

#[derive(Debug, Clone)]
/// Statistics for snapshot.
pub struct SnapshotStats {
    pub total_snapshots: usize,
    pub total_size_bytes: usize,
    pub total_compressed_bytes: usize,
    pub compression_ratio: f64,
    pub oldest_snapshot: Option<SnapshotMetadata>,
    pub newest_snapshot: Option<SnapshotMetadata>,
}

#[derive(Debug, thiserror::Error)]
/// Error variants for snapshot operations.
pub enum SnapshotError {
    #[error("Snapshot not found: {0}")]
    SnapshotNotFound(SnapshotId),
    #[error("IO error: {0}")]
    IoError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Compression error: {0}")]
    CompressionError(String),
    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },
}

// Construction, save/load snapshots, metadata I/O
include!("snapshot_store_persistence.rs");

// Queries, lookups, and statistics
include!("snapshot_store_retrieval.rs");

// Deletion, cleanup, and integrity verification
include!("snapshot_store_lifecycle.rs");

// Tests
include!("snapshot_store_tests.rs");
