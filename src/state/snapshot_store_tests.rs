// SnapshotStore tests

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
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

    #[tokio::test]
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

    #[tokio::test]
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

    #[tokio::test]
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

    #[tokio::test]
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

    // ============ SnapshotConfig Tests ============

    #[test]
    fn test_snapshot_config_default() {
        let config = SnapshotConfig::default();
        assert_eq!(config.max_snapshots, 10);
        assert_eq!(config.compression_level, 6);
        assert!(config.verify_on_write);
        assert!(config.verify_on_read);
    }

    #[test]
    fn test_snapshot_config_clone() {
        let config = SnapshotConfig {
            max_snapshots: 5,
            compression_level: 9,
            verify_on_write: false,
            verify_on_read: true,
        };
        let cloned = config.clone();
        assert_eq!(cloned.max_snapshots, 5);
        assert_eq!(cloned.compression_level, 9);
        assert!(!cloned.verify_on_write);
    }

    // ============ SnapshotMetadata Tests ============

    #[test]
    fn test_snapshot_metadata_creation() {
        let metadata = SnapshotMetadata {
            id: Uuid::new_v4(),
            timestamp: SystemTime::now(),
            event_id: 42,
            checksum: "abc123".to_string(),
            size_bytes: 1024,
            compressed_size: 512,
            partition_key: Some("partition".to_string()),
        };
        assert_eq!(metadata.event_id, 42);
        assert_eq!(metadata.size_bytes, 1024);
        assert_eq!(metadata.compressed_size, 512);
    }

    #[test]
    fn test_snapshot_metadata_clone() {
        let metadata = SnapshotMetadata {
            id: Uuid::new_v4(),
            timestamp: SystemTime::now(),
            event_id: 100,
            checksum: "hash".to_string(),
            size_bytes: 2048,
            compressed_size: 1024,
            partition_key: None,
        };
        let cloned = metadata.clone();
        assert_eq!(cloned.event_id, metadata.event_id);
        assert_eq!(cloned.checksum, metadata.checksum);
    }

    #[test]
    fn test_snapshot_metadata_debug() {
        let metadata = SnapshotMetadata {
            id: Uuid::new_v4(),
            timestamp: SystemTime::now(),
            event_id: 50,
            checksum: "checksum".to_string(),
            size_bytes: 100,
            compressed_size: 50,
            partition_key: None,
        };
        let debug = format!("{:?}", metadata);
        assert!(debug.contains("SnapshotMetadata"));
    }

    #[test]
    fn test_snapshot_metadata_serialization() {
        let metadata = SnapshotMetadata {
            id: Uuid::new_v4(),
            timestamp: SystemTime::now(),
            event_id: 75,
            checksum: "sha256hash".to_string(),
            size_bytes: 4096,
            compressed_size: 2048,
            partition_key: Some("test".to_string()),
        };
        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: SnapshotMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.event_id, metadata.event_id);
        assert_eq!(deserialized.checksum, metadata.checksum);
    }

    // ============ SnapshotStats Tests ============

    #[test]
    fn test_snapshot_stats_creation() {
        let stats = SnapshotStats {
            total_snapshots: 5,
            total_size_bytes: 10240,
            total_compressed_bytes: 5120,
            compression_ratio: 0.5,
            oldest_snapshot: None,
            newest_snapshot: None,
        };
        assert_eq!(stats.total_snapshots, 5);
        assert_eq!(stats.compression_ratio, 0.5);
    }

    #[test]
    fn test_snapshot_stats_clone() {
        let stats = SnapshotStats {
            total_snapshots: 3,
            total_size_bytes: 1024,
            total_compressed_bytes: 512,
            compression_ratio: 0.5,
            oldest_snapshot: None,
            newest_snapshot: None,
        };
        let cloned = stats.clone();
        assert_eq!(cloned.total_snapshots, stats.total_snapshots);
    }

    #[test]
    fn test_snapshot_stats_debug() {
        let stats = SnapshotStats {
            total_snapshots: 0,
            total_size_bytes: 0,
            total_compressed_bytes: 0,
            compression_ratio: 0.0,
            oldest_snapshot: None,
            newest_snapshot: None,
        };
        let debug = format!("{:?}", stats);
        assert!(debug.contains("SnapshotStats"));
    }

    // ============ SnapshotError Tests ============

    #[test]
    fn test_snapshot_error_display() {
        let err1 = SnapshotError::SnapshotNotFound(Uuid::new_v4());
        assert!(err1.to_string().contains("Snapshot not found"));

        let err2 = SnapshotError::IoError("disk full".to_string());
        assert!(err2.to_string().contains("IO error"));
        assert!(err2.to_string().contains("disk full"));

        let err3 = SnapshotError::SerializationError("invalid json".to_string());
        assert!(err3.to_string().contains("Serialization error"));

        let err4 = SnapshotError::CompressionError("gzip failed".to_string());
        assert!(err4.to_string().contains("Compression error"));

        let err5 = SnapshotError::ChecksumMismatch {
            expected: "abc".to_string(),
            actual: "xyz".to_string(),
        };
        assert!(err5.to_string().contains("Checksum mismatch"));
        assert!(err5.to_string().contains("abc"));
        assert!(err5.to_string().contains("xyz"));
    }

    #[test]
    fn test_snapshot_error_debug() {
        let err = SnapshotError::IoError("test".to_string());
        let debug = format!("{:?}", err);
        assert!(debug.contains("IoError"));
    }

    // ============ SnapshotStore Additional Tests ============

    #[tokio::test]
    async fn test_find_latest_snapshot() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_str().unwrap();

        let store = SnapshotStore::new(path, SnapshotConfig::default())
            .await
            .unwrap();

        // No snapshots initially
        assert!(store.find_latest_snapshot().is_none());

        let state = ExampleState::default();
        store.save_snapshot(&state, 10, None).await.unwrap();
        store.save_snapshot(&state, 20, None).await.unwrap();
        store.save_snapshot(&state, 5, None).await.unwrap();

        let latest = store.find_latest_snapshot().unwrap();
        assert_eq!(latest.event_id, 20);
    }

    #[tokio::test]
    async fn test_delete_snapshot() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_str().unwrap();

        let store = SnapshotStore::new(path, SnapshotConfig::default())
            .await
            .unwrap();

        let state = ExampleState::default();
        let snapshot_id = store.save_snapshot(&state, 100, None).await.unwrap();

        // Verify it exists
        assert!(store.find_latest_snapshot().is_some());

        // Delete it
        store.delete_snapshot(&snapshot_id).await.unwrap();

        // Verify it's gone
        assert!(store.find_latest_snapshot().is_none());
    }

    #[tokio::test]
    async fn test_load_nonexistent_snapshot() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_str().unwrap();

        let store = SnapshotStore::new(path, SnapshotConfig::default())
            .await
            .unwrap();

        let fake_id = Uuid::new_v4();
        let result: Result<ExampleState, _> = store.load_snapshot(&fake_id).await;
        assert!(result.is_err());

        if let Err(SnapshotError::SnapshotNotFound(id)) = result {
            assert_eq!(id, fake_id);
        } else {
            panic!("Expected SnapshotNotFound error");
        }
    }

    #[tokio::test]
    async fn test_get_statistics_empty() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_str().unwrap();

        let store = SnapshotStore::new(path, SnapshotConfig::default())
            .await
            .unwrap();

        let stats = store.get_statistics();
        assert_eq!(stats.total_snapshots, 0);
        assert_eq!(stats.total_size_bytes, 0);
        assert_eq!(stats.compression_ratio, 0.0);
        assert!(stats.oldest_snapshot.is_none());
        assert!(stats.newest_snapshot.is_none());
    }

    #[tokio::test]
    async fn test_get_statistics_with_snapshots() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_str().unwrap();

        let store = SnapshotStore::new(path, SnapshotConfig::default())
            .await
            .unwrap();

        let state = ExampleState::default();
        store.save_snapshot(&state, 10, None).await.unwrap();
        store.save_snapshot(&state, 20, None).await.unwrap();

        let stats = store.get_statistics();
        assert_eq!(stats.total_snapshots, 2);
        assert!(stats.total_size_bytes > 0);
        assert!(stats.oldest_snapshot.is_some());
        assert!(stats.newest_snapshot.is_some());
    }

    #[tokio::test]
    async fn test_cleanup_orphaned_files() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_str().unwrap();

        let store = SnapshotStore::new(path, SnapshotConfig::default())
            .await
            .unwrap();

        // Create an orphaned file
        let orphan_path = format!("{}/{}.snapshot", path, Uuid::new_v4());
        tokio::fs::write(&orphan_path, b"orphan data")
            .await
            .unwrap();

        // Create a valid snapshot
        let state = ExampleState::default();
        store.save_snapshot(&state, 100, None).await.unwrap();

        // Clean up orphans
        let deleted = store.cleanup_orphaned_files().await.unwrap();
        assert_eq!(deleted, 1);

        // Verify valid snapshot still exists
        assert!(store.find_latest_snapshot().is_some());
    }

    #[tokio::test]
    async fn test_no_verify_config() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_str().unwrap();

        let config = SnapshotConfig {
            verify_on_write: false,
            verify_on_read: false,
            ..Default::default()
        };
        let store = SnapshotStore::new(path, config).await.unwrap();

        let state = ExampleState::default();
        let snapshot_id = store.save_snapshot(&state, 100, None).await.unwrap();

        let _loaded: ExampleState = store.load_snapshot(&snapshot_id).await.unwrap();
    }

    #[tokio::test]
    async fn test_find_partition_snapshots_empty() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_str().unwrap();

        let store = SnapshotStore::new(path, SnapshotConfig::default())
            .await
            .unwrap();

        let snapshots = store.find_partition_snapshots("nonexistent");
        assert!(snapshots.is_empty());
    }

    #[tokio::test]
    async fn test_snapshot_with_state_data() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_str().unwrap();

        let store = SnapshotStore::new(path, SnapshotConfig::default())
            .await
            .unwrap();

        let mut state = ExampleState::default();
        state
            .data
            .insert("key1".to_string(), serde_json::json!({"value": 42}));
        state
            .data
            .insert("key2".to_string(), serde_json::json!("string value"));
        state.last_event_id = 999;
        state.event_count = 50;

        let snapshot_id = store.save_snapshot(&state, 999, None).await.unwrap();

        let loaded: ExampleState = store.load_snapshot(&snapshot_id).await.unwrap();
        assert_eq!(loaded.last_event_id, 999);
        assert_eq!(loaded.event_count, 50);
        assert_eq!(loaded.data.len(), 2);
        assert!(loaded.data.contains_key("key1"));
        assert!(loaded.data.contains_key("key2"));
    }
}
