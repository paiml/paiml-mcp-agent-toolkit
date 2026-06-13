// SnapshotStore lifecycle: deletion, cleanup, and integrity verification

impl SnapshotStore {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
        let checksum = hasher.finalize().iter().map(|b| format!("{b:02x}")).collect::<String>();

        if checksum != expected_checksum {
            return Err(SnapshotError::ChecksumMismatch {
                expected: expected_checksum.to_string(),
                actual: checksum,
            });
        }

        Ok(())
    }
}
