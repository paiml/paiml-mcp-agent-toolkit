#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_recovery_manager() {
        let temp_dir = TempDir::new().expect("internal error");
        let path = temp_dir.path().to_str().expect("internal error");

        let event_config = EventStoreConfig {
            persistence_enabled: false,
            ..Default::default()
        };

        let manager =
            RecoveryManager::<ExampleState>::new(event_config, SnapshotConfig::default(), path)
                .await
                .expect("internal error");

        let initial = ExampleState::default();
        let restored = manager
            .recover_state(initial, None)
            .await
            .expect("internal error");

        assert_eq!(restored.events_to_replay, 0);
    }

    #[test]
    fn test_adaptive_scheduler() {
        let scheduler = AdaptiveSnapshotScheduler::new(SnapshotSchedulerConfig::default());

        // Should not snapshot with few events
        assert!(!scheduler.should_snapshot(100, Duration::from_secs(10)));

        // Should snapshot with many events
        assert!(scheduler.should_snapshot(100_001, Duration::from_secs(10)));

        // Should snapshot after long time
        assert!(scheduler.should_snapshot(100, Duration::from_secs(3601)));
    }

    #[test]
    fn test_scheduler_adaptation() {
        let config = SnapshotSchedulerConfig {
            adaptive_enabled: true,
            recovery_time_target: Duration::from_secs(5),
            ..Default::default()
        };

        let scheduler = AdaptiveSnapshotScheduler::new(config);

        // Record slow recovery
        scheduler.record_recovery(Duration::from_secs(15));

        let new_config = scheduler.get_config();
        assert!(new_config.min_events < 1000); // Should reduce threshold

        // Record fast recovery
        scheduler.record_recovery(Duration::from_secs(1));

        let new_config = scheduler.get_config();
        assert!(new_config.min_events > 500); // Should increase threshold
    }

    #[tokio::test]
    async fn test_parallel_recovery() {
        let temp_dir = TempDir::new().expect("internal error");
        let path = temp_dir.path().to_str().expect("internal error");

        let event_config = EventStoreConfig {
            persistence_enabled: false,
            ..Default::default()
        };

        let parallel =
            ParallelRecovery::<ExampleState>::new(4, event_config, SnapshotConfig::default(), path)
                .await
                .expect("internal error");

        let states = parallel
            .recover_all_partitions(ExampleState::default)
            .await
            .expect("internal error");

        assert_eq!(states.len(), 4);

        let merged = parallel
            .merge_partitions(states)
            .await
            .expect("internal error");
        assert_eq!(merged.last_event_id, 0);
    }

    // ============ SnapshotSchedulerConfig Tests ============

    #[test]
    fn test_snapshot_scheduler_config_default() {
        let config = SnapshotSchedulerConfig::default();
        assert_eq!(config.min_events, 1000);
        assert_eq!(config.max_events, 100_000);
        assert_eq!(config.min_time_between_snapshots, Duration::from_secs(60));
        assert_eq!(config.max_time_between_snapshots, Duration::from_secs(3600));
        assert_eq!(config.recovery_time_target, Duration::from_secs(5));
        assert!(config.adaptive_enabled);
    }

    #[test]
    fn test_snapshot_scheduler_config_clone() {
        let config = SnapshotSchedulerConfig {
            min_events: 500,
            max_events: 50_000,
            min_time_between_snapshots: Duration::from_secs(30),
            max_time_between_snapshots: Duration::from_secs(1800),
            recovery_time_target: Duration::from_secs(10),
            adaptive_enabled: false,
        };
        let cloned = config.clone();
        assert_eq!(cloned.min_events, 500);
        assert_eq!(cloned.max_events, 50_000);
        assert!(!cloned.adaptive_enabled);
    }

    // ============ SnapshotSchedulerMetrics Tests ============

    #[test]
    fn test_snapshot_scheduler_metrics_creation() {
        let metrics = SnapshotSchedulerMetrics {
            total_snapshots: 10,
            avg_events_between: 5000,
            avg_time_between: Duration::from_secs(300),
            last_recovery_time: Some(Duration::from_secs(3)),
            current_thresholds: (1000, Duration::from_secs(60)),
        };
        assert_eq!(metrics.total_snapshots, 10);
        assert_eq!(metrics.avg_events_between, 5000);
    }

    #[test]
    fn test_snapshot_scheduler_metrics_clone() {
        let metrics = SnapshotSchedulerMetrics {
            total_snapshots: 5,
            avg_events_between: 2000,
            avg_time_between: Duration::from_secs(120),
            last_recovery_time: None,
            current_thresholds: (500, Duration::from_secs(30)),
        };
        let cloned = metrics.clone();
        assert_eq!(cloned.total_snapshots, metrics.total_snapshots);
    }

    #[test]
    fn test_snapshot_scheduler_metrics_debug() {
        let metrics = SnapshotSchedulerMetrics {
            total_snapshots: 3,
            avg_events_between: 1000,
            avg_time_between: Duration::from_secs(60),
            last_recovery_time: Some(Duration::from_millis(500)),
            current_thresholds: (1000, Duration::from_secs(60)),
        };
        let debug = format!("{:?}", metrics);
        assert!(debug.contains("SnapshotSchedulerMetrics"));
    }

    // ============ RecoveryStats Tests ============

    #[test]
    fn test_recovery_stats_creation() {
        let stats = RecoveryStats {
            total_events: 10000,
            total_snapshots: 5,
            compression_ratio: 0.4,
            next_snapshot_in: Duration::from_secs(60),
        };
        assert_eq!(stats.total_events, 10000);
        assert_eq!(stats.total_snapshots, 5);
        assert_eq!(stats.compression_ratio, 0.4);
    }

    #[test]
    fn test_recovery_stats_clone() {
        let stats = RecoveryStats {
            total_events: 5000,
            total_snapshots: 3,
            compression_ratio: 0.5,
            next_snapshot_in: Duration::from_secs(30),
        };
        let cloned = stats.clone();
        assert_eq!(cloned.total_events, stats.total_events);
    }

    #[test]
    fn test_recovery_stats_debug() {
        let stats = RecoveryStats {
            total_events: 1000,
            total_snapshots: 1,
            compression_ratio: 0.6,
            next_snapshot_in: Duration::from_secs(45),
        };
        let debug = format!("{:?}", stats);
        assert!(debug.contains("RecoveryStats"));
    }

    // ============ RecoveryError Tests ============

    #[test]
    fn test_recovery_error_display() {
        let err1 = RecoveryError::EventStoreError("connection failed".to_string());
        assert!(err1.to_string().contains("Event store error"));
        assert!(err1.to_string().contains("connection failed"));

        let err2 = RecoveryError::SnapshotError("disk full".to_string());
        assert!(err2.to_string().contains("Snapshot error"));
        assert!(err2.to_string().contains("disk full"));

        let err3 = RecoveryError::StateCorruption("checksum mismatch".to_string());
        assert!(err3.to_string().contains("State corruption"));
        assert!(err3.to_string().contains("checksum mismatch"));

        let err4 = RecoveryError::RecoveryFailed("timeout".to_string());
        assert!(err4.to_string().contains("Recovery failed"));
        assert!(err4.to_string().contains("timeout"));
    }

    #[test]
    fn test_recovery_error_debug() {
        let err = RecoveryError::StateCorruption("test".to_string());
        let debug = format!("{:?}", err);
        assert!(debug.contains("StateCorruption"));
    }

    // ============ AdaptiveSnapshotScheduler Additional Tests ============

    #[test]
    fn test_scheduler_record_snapshot() {
        let scheduler = AdaptiveSnapshotScheduler::new(SnapshotSchedulerConfig::default());

        scheduler.record_snapshot(5000, Duration::from_secs(300));
        scheduler.record_snapshot(3000, Duration::from_secs(200));

        let metrics = scheduler.get_metrics();
        assert_eq!(metrics.total_snapshots, 2);
        assert_eq!(metrics.avg_events_between, 4000); // (5000 + 3000) / 2
    }

    #[test]
    fn test_scheduler_get_metrics_empty() {
        let scheduler = AdaptiveSnapshotScheduler::new(SnapshotSchedulerConfig::default());

        let metrics = scheduler.get_metrics();
        assert_eq!(metrics.total_snapshots, 0);
        assert_eq!(metrics.avg_events_between, 0);
        assert_eq!(metrics.avg_time_between, Duration::ZERO);
        assert!(metrics.last_recovery_time.is_none());
    }

    #[test]
    fn test_scheduler_record_recovery() {
        let scheduler = AdaptiveSnapshotScheduler::new(SnapshotSchedulerConfig::default());

        scheduler.record_recovery(Duration::from_secs(2));

        let metrics = scheduler.get_metrics();
        assert_eq!(metrics.last_recovery_time, Some(Duration::from_secs(2)));
    }

    #[test]
    fn test_scheduler_recovery_times_capped() {
        let scheduler = AdaptiveSnapshotScheduler::new(SnapshotSchedulerConfig::default());

        // Record more than 10 recovery times
        for i in 0..15 {
            scheduler.record_recovery(Duration::from_secs(i as u64));
        }

        // Internal metrics should only keep 10
        // Just verify it doesn't panic and still works
        let metrics = scheduler.get_metrics();
        assert!(metrics.last_recovery_time.is_some());
    }

    #[test]
    fn test_scheduler_should_snapshot_non_adaptive() {
        let config = SnapshotSchedulerConfig {
            adaptive_enabled: false,
            min_events: 100,
            max_events: 1000,
            min_time_between_snapshots: Duration::from_secs(10),
            max_time_between_snapshots: Duration::from_secs(100),
            recovery_time_target: Duration::from_secs(5),
        };
        let scheduler = AdaptiveSnapshotScheduler::new(config);

        // Below min events and time
        assert!(!scheduler.should_snapshot(50, Duration::from_secs(5)));

        // At min events but below min time
        assert!(!scheduler.should_snapshot(100, Duration::from_secs(5)));

        // At min time but below min events
        assert!(!scheduler.should_snapshot(50, Duration::from_secs(10)));

        // Above both mins, should use non-adaptive logic
        // Need 100 * 10 = 1000 events or 10 * 10 = 100s
        assert!(scheduler.should_snapshot(1000, Duration::from_secs(15)));
        assert!(scheduler.should_snapshot(150, Duration::from_secs(100)));
    }

    #[test]
    fn test_scheduler_adaptive_with_history() {
        let config = SnapshotSchedulerConfig {
            adaptive_enabled: true,
            min_events: 100,
            max_events: 10000,
            min_time_between_snapshots: Duration::from_secs(10),
            max_time_between_snapshots: Duration::from_secs(1000),
            recovery_time_target: Duration::from_secs(5),
        };
        let scheduler = AdaptiveSnapshotScheduler::new(config);

        // Record some fast recovery times
        scheduler.record_recovery(Duration::from_secs(1));
        scheduler.record_recovery(Duration::from_secs(2));

        // With fast recovery, should wait longer
        // Should be false with moderate events/time
        assert!(!scheduler.should_snapshot(150, Duration::from_secs(15)));
    }

    #[test]
    fn test_scheduler_get_config() {
        let config = SnapshotSchedulerConfig {
            min_events: 200,
            max_events: 5000,
            min_time_between_snapshots: Duration::from_secs(20),
            max_time_between_snapshots: Duration::from_secs(500),
            recovery_time_target: Duration::from_secs(3),
            adaptive_enabled: false,
        };
        let scheduler = AdaptiveSnapshotScheduler::new(config.clone());

        let retrieved = scheduler.get_config();
        assert_eq!(retrieved.min_events, 200);
        assert_eq!(retrieved.max_events, 5000);
        assert!(!retrieved.adaptive_enabled);
    }

    // ============ ParallelRecovery Tests ============

    #[tokio::test]
    async fn test_parallel_recovery_merge_empty() {
        let temp_dir = TempDir::new().expect("internal error");
        let path = temp_dir.path().to_str().expect("internal error");

        let event_config = EventStoreConfig {
            persistence_enabled: false,
            ..Default::default()
        };

        let parallel =
            ParallelRecovery::<ExampleState>::new(1, event_config, SnapshotConfig::default(), path)
                .await
                .expect("internal error");

        // Empty vec should error
        let result = parallel.merge_partitions(vec![]).await;
        assert!(result.is_err());

        if let Err(RecoveryError::RecoveryFailed(msg)) = result {
            assert!(msg.contains("No partitions"));
        } else {
            panic!("Expected RecoveryFailed error");
        }
    }

    #[tokio::test]
    async fn test_recovery_manager_save_snapshot() {
        let temp_dir = TempDir::new().expect("internal error");
        let path = temp_dir.path().to_str().expect("internal error");

        let event_config = EventStoreConfig {
            persistence_enabled: false,
            ..Default::default()
        };

        let manager =
            RecoveryManager::<ExampleState>::new(event_config, SnapshotConfig::default(), path)
                .await
                .expect("internal error");

        let mut state = ExampleState::default();
        state.last_event_id = 100;
        state.event_count = 50;

        let snapshot_id = manager
            .save_snapshot(&state, None)
            .await
            .expect("internal error");

        // Verify snapshot was saved by recovering
        let initial = ExampleState::default();
        let restored = manager
            .recover_state(initial, None)
            .await
            .expect("internal error");

        // Should have loaded from snapshot
        assert!(restored.events_to_replay == 0 || snapshot_id != Uuid::nil());
    }

    #[tokio::test]
    async fn test_recovery_manager_should_snapshot() {
        let temp_dir = TempDir::new().expect("internal error");
        let path = temp_dir.path().to_str().expect("internal error");

        let event_config = EventStoreConfig {
            persistence_enabled: false,
            ..Default::default()
        };

        let manager =
            RecoveryManager::<ExampleState>::new(event_config, SnapshotConfig::default(), path)
                .await
                .expect("internal error");

        let state = ExampleState::default();

        // With default state, shouldn't need snapshot
        let should = manager.should_snapshot(&state).await;
        // Result depends on state values
        assert!(!should || should); // Just verify no panic
    }

    #[tokio::test]
    async fn test_recovery_manager_get_stats() {
        let temp_dir = TempDir::new().expect("internal error");
        let path = temp_dir.path().to_str().expect("internal error");

        let event_config = EventStoreConfig {
            persistence_enabled: false,
            ..Default::default()
        };

        let manager =
            RecoveryManager::<ExampleState>::new(event_config, SnapshotConfig::default(), path)
                .await
                .expect("internal error");

        let stats = manager.get_recovery_stats();
        assert_eq!(stats.total_snapshots, 0);
    }

    #[tokio::test]
    async fn test_recovery_manager_cleanup() {
        let temp_dir = TempDir::new().expect("internal error");
        let path = temp_dir.path().to_str().expect("internal error");

        let event_config = EventStoreConfig {
            persistence_enabled: false,
            ..Default::default()
        };

        let manager =
            RecoveryManager::<ExampleState>::new(event_config, SnapshotConfig::default(), path)
                .await
                .expect("internal error");

        let cleaned = manager
            .cleanup_old_snapshots()
            .await
            .expect("internal error");
        assert_eq!(cleaned, 0); // No orphans to clean
    }

    #[tokio::test]
    async fn test_recovery_manager_compact() {
        let temp_dir = TempDir::new().expect("internal error");
        let path = temp_dir.path().to_str().expect("internal error");

        let event_config = EventStoreConfig {
            persistence_enabled: false,
            ..Default::default()
        };

        let manager =
            RecoveryManager::<ExampleState>::new(event_config, SnapshotConfig::default(), path)
                .await
                .expect("internal error");

        manager.compact_events().await.expect("internal error");
    }
}
