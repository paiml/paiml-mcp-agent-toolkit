// SnapshotStore persistence: construction, save/load snapshots, metadata I/O

impl SnapshotStore {
    pub async fn new(base_path: &str, config: SnapshotConfig) -> Result<Self, SnapshotError> {
        debug_assert!(!base_path.is_empty(), "base_path must not be empty");
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
}
