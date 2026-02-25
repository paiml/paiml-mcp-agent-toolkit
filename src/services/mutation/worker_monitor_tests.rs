#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_worker_metrics_initialization() {
        let metrics = WorkerMetrics::new(1);

        assert_eq!(metrics.id, 1);
        assert_eq!(metrics.state, WorkerState::Idle);
        assert_eq!(metrics.processed_count, 0);
        assert_eq!(metrics.failed_count, 0);
    }

    #[tokio::test]
    async fn test_worker_metrics_record_success() {
        let mut metrics = WorkerMetrics::new(1);

        metrics.record_success(100);

        assert_eq!(metrics.processed_count, 1);
        assert_eq!(metrics.state, WorkerState::Idle);
        assert_eq!(metrics.avg_processing_time_ms, 100.0);

        metrics.record_success(200);

        assert_eq!(metrics.processed_count, 2);
        assert_eq!(metrics.avg_processing_time_ms, 150.0); // (100 + 200) / 2
    }

    #[tokio::test]
    async fn test_worker_metrics_record_failure() {
        let mut metrics = WorkerMetrics::new(1);

        metrics.record_failure("Test error");

        assert_eq!(metrics.failed_count, 1);
        assert_eq!(metrics.state, WorkerState::Idle);
        assert_eq!(metrics.recent_errors.len(), 1);
        assert_eq!(metrics.recent_errors[0], "Test error");

        // Add more errors and verify the limit is enforced
        for i in 0..10 {
            metrics.record_failure(&format!("Error {}", i));
        }

        assert_eq!(metrics.failed_count, 11);
        assert_eq!(metrics.recent_errors.len(), 5);
        assert_eq!(metrics.recent_errors[0], "Error 5");
    }

    #[tokio::test]
    async fn test_worker_is_stalled() {
        let mut metrics = WorkerMetrics::new(1);

        // Set to processing state
        metrics.set_state(WorkerState::Processing);

        // Not stalled yet
        assert!(!metrics.is_stalled(Duration::from_millis(100)));

        // Wait for timeout
        sleep(Duration::from_millis(20)).await;

        // Still not stalled
        assert!(!metrics.is_stalled(Duration::from_millis(100)));

        // Reduce timeout - now it's stalled
        assert!(metrics.is_stalled(Duration::from_millis(10)));

        // Change state - no longer stalled
        metrics.set_state(WorkerState::Idle);
        assert!(!metrics.is_stalled(Duration::from_millis(10)));
    }

    #[tokio::test]
    async fn test_worker_monitor_initialization() {
        let monitor = WorkerMonitor::new(5, Duration::from_secs(10));

        monitor.initialize_workers().await;

        let all_metrics = monitor.get_all_metrics().await;
        assert_eq!(all_metrics.len(), 5);

        for id in 0..5 {
            let metrics = monitor.get_worker_metrics(id).await;
            assert!(metrics.is_some());
            assert_eq!(metrics.unwrap().id, id);
        }

        let metrics = monitor.get_worker_metrics(10).await;
        assert!(metrics.is_none());
    }

    #[tokio::test]
    async fn test_worker_monitor_record_heartbeat() {
        let monitor = WorkerMonitor::new(2, Duration::from_secs(10));
        monitor.initialize_workers().await;

        // Record initial metrics
        let initial = monitor.get_worker_metrics(0).await.unwrap();
        let initial_heartbeat_count = initial.heartbeat_count;

        // Wait a moment
        sleep(Duration::from_millis(10)).await;

        // Record heartbeat
        monitor.record_heartbeat(0).await;

        // Get updated metrics
        let updated = monitor.get_worker_metrics(0).await.unwrap();

        assert_eq!(updated.heartbeat_count, initial_heartbeat_count + 1);
        assert!(updated.time_since_heartbeat() < initial.time_since_heartbeat());
    }

    #[tokio::test]
    async fn test_worker_monitor_state_changes() {
        let monitor = WorkerMonitor::new(1, Duration::from_secs(10));
        monitor.initialize_workers().await;

        // Initial state
        assert_eq!(
            monitor.get_worker_metrics(0).await.unwrap().state,
            WorkerState::Idle
        );

        // Start processing
        monitor.record_start_processing(0).await;
        assert_eq!(
            monitor.get_worker_metrics(0).await.unwrap().state,
            WorkerState::Processing
        );

        // Success
        monitor.record_success(0, 100).await;
        let metrics = monitor.get_worker_metrics(0).await.unwrap();
        assert_eq!(metrics.state, WorkerState::Idle);
        assert_eq!(metrics.processed_count, 1);

        // Start processing again
        monitor.record_start_processing(0).await;

        // Failure
        monitor.record_failure(0, "Test error").await;
        let metrics = monitor.get_worker_metrics(0).await.unwrap();
        assert_eq!(metrics.state, WorkerState::Idle);
        assert_eq!(metrics.failed_count, 1);

        // Mark failed
        monitor.mark_failed(0, "Fatal error").await;
        let metrics = monitor.get_worker_metrics(0).await.unwrap();
        assert_eq!(metrics.state, WorkerState::Failed);
        assert_eq!(metrics.failed_count, 2);

        // Mark terminated
        monitor.mark_terminated(0).await;
        assert_eq!(
            monitor.get_worker_metrics(0).await.unwrap().state,
            WorkerState::Terminated
        );
    }

    #[tokio::test]
    async fn test_worker_monitor_stalled_detection() {
        let monitor = WorkerMonitor::new(2, Duration::from_millis(50));
        monitor.initialize_workers().await;

        // Both workers idle - none stalled
        assert_eq!(monitor.get_stalled_workers().await.len(), 0);

        // Start processing on worker 0
        monitor.record_start_processing(0).await;

        // Not stalled yet
        assert_eq!(monitor.get_stalled_workers().await.len(), 0);

        // Wait for timeout
        sleep(Duration::from_millis(60)).await;

        // Now worker 0 is stalled
        let stalled = monitor.get_stalled_workers().await;
        assert_eq!(stalled.len(), 1);
        assert_eq!(stalled[0], 0);
    }

    #[tokio::test]
    async fn test_worker_monitor_health_score() {
        let monitor = WorkerMonitor::new(4, Duration::from_millis(50));
        monitor.initialize_workers().await;

        // All workers healthy
        assert_eq!(monitor.calculate_health_score().await, 100.0);

        // Mark one worker as failed
        monitor.mark_failed(0, "Failed").await;
        assert_eq!(monitor.calculate_health_score().await, 75.0);

        // Start processing on worker 1
        monitor.record_start_processing(1).await;

        // Still 75% healthy
        assert_eq!(monitor.calculate_health_score().await, 75.0);

        // Wait for worker 1 to stall
        sleep(Duration::from_millis(60)).await;

        // Now 50% healthy
        assert_eq!(monitor.calculate_health_score().await, 50.0);
    }
}
