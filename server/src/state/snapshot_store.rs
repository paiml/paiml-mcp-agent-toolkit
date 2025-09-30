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
pub struct SnapshotStore {
    snapshots: Arc<RwLock<Vec<SnapshotMetadata>>>,
    base_path: String,
    config: SnapshotConfig,
}

#[derive(Clone)]
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
pub struct SnapshotMetadata {
    pub id: SnapshotId,
    pub timestamp: SystemTime,
    pub event_id: EventId,
    pub checksum: String,
    pub size_bytes: usize,
    pub compressed_size: usize,
    pub partition_key: Option<String>,
}

impl SnapshotStore {
    pub async fn new(base_path: &str, config: SnapshotConfig) -> Result<Self, SnapshotError> {
        // Create snapshot directory if it doesn't exist
        create_dir_all(base_path)
            .await
            .map_err(|e| SnapshotError::IoError(e.to_string()))?;

        let mut store = Self {
            snapshots: Arc::new(RwLock::new(Vec::new())),
            base_path: base_path.to_string(),
            config,
        };

        // Load existing snapshot metadata
        store.load_metadata().await?;

        Ok(store)
    }

    #[allow(clippy::await_holding_lock)]
    pub async fn save_snapshot<S: AgentState>(
        &self,
        state: &S,
        event_id: EventId,
        partition_key: Option<String>,
    ) -> Result<SnapshotId, SnapshotError> {
        let snapshot_id = Uuid::new_v4();
        let timestamp = SystemTime::now();

        // Serialize state
        let serialized = serde_json::to_vec(state)
            .map_err(|e| SnapshotError::SerializationError(e.to_string()))?;

        // Calculate checksum
        let mut hasher = Sha256::new();
        hasher.update(&serialized);
        let checksum = format!("{:x}", hasher.finalize());

        // Compress data
        let mut encoder =
            GzEncoder::new(Vec::new(), Compression::new(self.config.compression_level));
        encoder
            .write_all(&serialized)
            .map_err(|e| SnapshotError::CompressionError(e.to_string()))?;
        let compressed = encoder
            .finish()
            .map_err(|e| SnapshotError::CompressionError(e.to_string()))?;

        // Write to file
        let file_path = self.snapshot_path(&snapshot_id);
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&file_path)
            .await
            .map_err(|e| SnapshotError::IoError(e.to_string()))?;

        file.write_all(&compressed)
            .await
            .map_err(|e| SnapshotError::IoError(e.to_string()))?;
        file.flush()
            .await
            .map_err(|e| SnapshotError::IoError(e.to_string()))?;

        // Verify if configured
        if self.config.verify_on_write {
            self.verify_snapshot(&snapshot_id, &checksum).await?;
        }

        // Create metadata
        let metadata = SnapshotMetadata {
            id: snapshot_id,
            timestamp,
            event_id,
            checksum,
            size_bytes: serialized.len(),
            compressed_size: compressed.len(),
            partition_key,
        };

        // Store metadata
        {
            let mut snapshots = self.snapshots.write();
            snapshots.push(metadata.clone());
            snapshots.sort_by_key(|s| s.event_id);

            // Enforce retention policy
            if snapshots.len() > self.config.max_snapshots {
                let to_remove = snapshots.len() - self.config.max_snapshots;
                let removed: Vec<_> = snapshots.drain(..to_remove).collect();

                // Delete old snapshot files
                for old_snapshot in removed {
                    let path = self.snapshot_path(&old_snapshot.id);
                    let _ = tokio::fs::remove_file(path).await;
                }
            }
        }

        // Save metadata
        self.save_metadata().await?;

        Ok(snapshot_id)
    }

    pub async fn load_snapshot<S: AgentState>(
        &self,
        snapshot_id: &SnapshotId,
    ) -> Result<S, SnapshotError> {
        // Find metadata
        let metadata = {
            let snapshots = self.snapshots.read();
            snapshots
                .iter()
                .find(|s| s.id == *snapshot_id)
                .cloned()
                .ok_or(SnapshotError::SnapshotNotFound(*snapshot_id))?
        };

        // Read compressed data
        let file_path = self.snapshot_path(snapshot_id);
        let mut file = tokio::fs::File::open(&file_path)
            .await
            .map_err(|e| SnapshotError::IoError(e.to_string()))?;

        let mut compressed = Vec::new();
        file.read_to_end(&mut compressed)
            .await
            .map_err(|e| SnapshotError::IoError(e.to_string()))?;

        // Decompress
        let mut decoder = GzDecoder::new(&compressed[..]);
        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| SnapshotError::CompressionError(e.to_string()))?;

        // Verify checksum if configured
        if self.config.verify_on_read {
            let mut hasher = Sha256::new();
            hasher.update(&decompressed);
            let checksum = format!("{:x}", hasher.finalize());

            if checksum != metadata.checksum {
                return Err(SnapshotError::ChecksumMismatch {
                    expected: metadata.checksum,
                    actual: checksum,
                });
            }
        }

        // Deserialize from JSON
        let state = serde_json::from_slice(&decompressed)
            .map_err(|e| SnapshotError::SerializationError(e.to_string()))?;

        Ok(state)
    }

    pub fn find_latest_snapshot_before(&self, event_id: EventId) -> Option<SnapshotMetadata> {
        let snapshots = self.snapshots.read();
        snapshots
            .iter()
            .filter(|s| s.event_id <= event_id)
            .max_by_key(|s| s.event_id)
            .cloned()
    }

    pub fn find_latest_snapshot(&self) -> Option<SnapshotMetadata> {
        let snapshots = self.snapshots.read();
        snapshots.iter().max_by_key(|s| s.event_id).cloned()
    }

    pub fn find_partition_snapshots(&self, partition_key: &str) -> Vec<SnapshotMetadata> {
        let snapshots = self.snapshots.read();
        snapshots
            .iter()
            .filter(|s| s.partition_key.as_ref() == Some(&partition_key.to_string()))
            .cloned()
            .collect()
    }

    pub async fn delete_snapshot(&self, snapshot_id: &SnapshotId) -> Result<(), SnapshotError> {
        // Remove from metadata
        {
            let mut snapshots = self.snapshots.write();
            snapshots.retain(|s| s.id != *snapshot_id);
        }

        // Delete file
        let file_path = self.snapshot_path(snapshot_id);
        tokio::fs::remove_file(file_path)
            .await
            .map_err(|e| SnapshotError::IoError(e.to_string()))?;

        // Save updated metadata
        self.save_metadata().await?;

        Ok(())
    }

    pub async fn cleanup_orphaned_files(&self) -> Result<usize, SnapshotError> {
        let mut deleted = 0;

        let mut entries = tokio::fs::read_dir(&self.base_path)
            .await
            .map_err(|e| SnapshotError::IoError(e.to_string()))?;

        let valid_ids: std::collections::HashSet<_> = {
            let snapshots = self.snapshots.read();
            snapshots.iter().map(|s| s.id).collect()
        };

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| SnapshotError::IoError(e.to_string()))?
        {
            let path = entry.path();
            if let Some(file_name) = path.file_name() {
                if let Some(name_str) = file_name.to_str() {
                    if name_str.ends_with(".snapshot") {
                        let id_str = name_str.trim_end_matches(".snapshot");
                        if let Ok(id) = Uuid::parse_str(id_str) {
                            if !valid_ids.contains(&id) {
                                tokio::fs::remove_file(path)
                                    .await
                                    .map_err(|e| SnapshotError::IoError(e.to_string()))?;
                                deleted += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(deleted)
    }

    async fn verify_snapshot(
        &self,
        snapshot_id: &SnapshotId,
        expected_checksum: &str,
    ) -> Result<(), SnapshotError> {
        // Read and decompress
        let file_path = self.snapshot_path(snapshot_id);
        let mut file = tokio::fs::File::open(&file_path)
            .await
            .map_err(|e| SnapshotError::IoError(e.to_string()))?;

        let mut compressed = Vec::new();
        file.read_to_end(&mut compressed)
            .await
            .map_err(|e| SnapshotError::IoError(e.to_string()))?;

        let mut decoder = GzDecoder::new(&compressed[..]);
        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| SnapshotError::CompressionError(e.to_string()))?;

        // Calculate checksum
        let mut hasher = Sha256::new();
        hasher.update(&decompressed);
        let checksum = format!("{:x}", hasher.finalize());

        if checksum != expected_checksum {
            return Err(SnapshotError::ChecksumMismatch {
                expected: expected_checksum.to_string(),
                actual: checksum,
            });
        }

        Ok(())
    }

    fn snapshot_path(&self, snapshot_id: &SnapshotId) -> String {
        format!("{}/{}.snapshot", self.base_path, snapshot_id)
    }

    fn metadata_path(&self) -> String {
        format!("{}/metadata.json", self.base_path)
    }

    async fn save_metadata(&self) -> Result<(), SnapshotError> {
        let snapshots = self.snapshots.read().clone();
        let json = serde_json::to_string_pretty(&snapshots)
            .map_err(|e| SnapshotError::SerializationError(e.to_string()))?;

        let path = self.metadata_path();
        tokio::fs::write(path, json)
            .await
            .map_err(|e| SnapshotError::IoError(e.to_string()))?;

        Ok(())
    }

    async fn load_metadata(&mut self) -> Result<(), SnapshotError> {
        let path = self.metadata_path();

        if !tokio::fs::try_exists(&path).await.unwrap_or(false) {
            return Ok(()); // No metadata file yet
        }

        let json = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| SnapshotError::IoError(e.to_string()))?;

        let snapshots: Vec<SnapshotMetadata> = serde_json::from_str(&json)
            .map_err(|e| SnapshotError::SerializationError(e.to_string()))?;

        *self.snapshots.write() = snapshots;
        Ok(())
    }

    pub fn get_statistics(&self) -> SnapshotStats {
        let snapshots = self.snapshots.read();

        let total_size: usize = snapshots.iter().map(|s| s.size_bytes).sum();
        let compressed_size: usize = snapshots.iter().map(|s| s.compressed_size).sum();

        SnapshotStats {
            total_snapshots: snapshots.len(),
            total_size_bytes: total_size,
            total_compressed_bytes: compressed_size,
            compression_ratio: if total_size > 0 {
                compressed_size as f64 / total_size as f64
            } else {
                0.0
            },
            oldest_snapshot: snapshots.iter().min_by_key(|s| s.timestamp).cloned(),
            newest_snapshot: snapshots.iter().max_by_key(|s| s.timestamp).cloned(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SnapshotStats {
    pub total_snapshots: usize,
    pub total_size_bytes: usize,
    pub total_compressed_bytes: usize,
    pub compression_ratio: f64,
    pub oldest_snapshot: Option<SnapshotMetadata>,
    pub newest_snapshot: Option<SnapshotMetadata>,
}

#[derive(Debug, thiserror::Error)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[actix_rt::test]
    async fn test_save_and_load_snapshot() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_str().unwrap();

        let store = SnapshotStore::new(path, SnapshotConfig::default())
            .await
            .unwrap();

        let state = ExampleState::default();
        let snapshot_id = store.save_snapshot(&state, 100, None).await.unwrap();

        let loaded: ExampleState = store.load_snapshot(&snapshot_id).await.unwrap();
        assert_eq!(loaded.last_event_id, state.last_event_id);
    }

    #[actix_rt::test]
    async fn test_compression() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_str().unwrap();

        let store = SnapshotStore::new(path, SnapshotConfig::default())
            .await
            .unwrap();

        let mut state = ExampleState::default();
        // Add some data to compress
        for i in 0..1000 {
            state.data.insert(
                format!("key_{}", i),
                serde_json::json!({"value": i, "data": "test_data_that_compresses_well"}),
            );
        }

        let snapshot_id = store.save_snapshot(&state, 100, None).await.unwrap();

        let stats = store.get_statistics();
        assert!(stats.compression_ratio < 0.5); // Should achieve good compression

        let loaded: ExampleState = store.load_snapshot(&snapshot_id).await.unwrap();
        assert_eq!(loaded.data.len(), state.data.len());
    }

    #[actix_rt::test]
    async fn test_retention_policy() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_str().unwrap();

        let config = SnapshotConfig {
            max_snapshots: 3,
            ..Default::default()
        };
        let store = SnapshotStore::new(path, config).await.unwrap();

        let state = ExampleState::default();

        // Create 5 snapshots
        for i in 1..=5 {
            store
                .save_snapshot(&state, i as EventId, None)
                .await
                .unwrap();
        }

        // Should only keep 3
        let stats = store.get_statistics();
        assert_eq!(stats.total_snapshots, 3);

        // Verify the oldest were deleted
        let snapshots = store.snapshots.read();
        let event_ids: Vec<_> = snapshots.iter().map(|s| s.event_id).collect();
        assert_eq!(event_ids, vec![3, 4, 5]);
    }

    #[actix_rt::test]
    async fn test_find_latest_snapshot_before() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_str().unwrap();

        let store = SnapshotStore::new(path, SnapshotConfig::default())
            .await
            .unwrap();

        let state = ExampleState::default();

        store.save_snapshot(&state, 10, None).await.unwrap();
        store.save_snapshot(&state, 20, None).await.unwrap();
        store.save_snapshot(&state, 30, None).await.unwrap();

        let snapshot = store.find_latest_snapshot_before(25).unwrap();
        assert_eq!(snapshot.event_id, 20);

        let snapshot = store.find_latest_snapshot_before(35).unwrap();
        assert_eq!(snapshot.event_id, 30);

        assert!(store.find_latest_snapshot_before(5).is_none());
    }

    #[actix_rt::test]
    async fn test_partition_snapshots() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_str().unwrap();

        let store = SnapshotStore::new(path, SnapshotConfig::default())
            .await
            .unwrap();

        let state = ExampleState::default();

        store
            .save_snapshot(&state, 10, Some("partition1".to_string()))
            .await
            .unwrap();
        store
            .save_snapshot(&state, 20, Some("partition2".to_string()))
            .await
            .unwrap();
        store
            .save_snapshot(&state, 30, Some("partition1".to_string()))
            .await
            .unwrap();

        let p1_snapshots = store.find_partition_snapshots("partition1");
        assert_eq!(p1_snapshots.len(), 2);

        let p2_snapshots = store.find_partition_snapshots("partition2");
        assert_eq!(p2_snapshots.len(), 1);
    }
}
