// SnapshotStore retrieval: queries, lookups, and statistics

impl SnapshotStore {
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
