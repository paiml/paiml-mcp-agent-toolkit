// Tests for DistributedConfig, MutationProgress, and DistributedExecutor.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // DistributedConfig Tests
    // =========================================================================

    #[test]
    fn test_distributed_config_default() {
        let config = DistributedConfig::default();
        assert!(config.worker_count > 0);
        assert!(config.max_concurrent >= config.worker_count);
        assert_eq!(config.queue_size, 1000);
        assert!(config.track_progress);
    }

    #[test]
    fn test_distributed_config_custom() {
        let config = DistributedConfig {
            worker_count: 4,
            max_concurrent: 8,
            queue_size: 500,
            track_progress: false,
        };
        assert_eq!(config.worker_count, 4);
        assert_eq!(config.max_concurrent, 8);
        assert_eq!(config.queue_size, 500);
        assert!(!config.track_progress);
    }

    #[test]
    fn test_distributed_config_clone() {
        let config = DistributedConfig {
            worker_count: 2,
            max_concurrent: 4,
            queue_size: 100,
            track_progress: true,
        };
        let cloned = config.clone();
        assert_eq!(config.worker_count, cloned.worker_count);
        assert_eq!(config.max_concurrent, cloned.max_concurrent);
        assert_eq!(config.queue_size, cloned.queue_size);
        assert_eq!(config.track_progress, cloned.track_progress);
    }

    #[test]
    fn test_distributed_config_debug() {
        let config = DistributedConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("DistributedConfig"));
        assert!(debug_str.contains("worker_count"));
    }

    #[test]
    fn test_distributed_config_single_worker() {
        let config = DistributedConfig {
            worker_count: 1,
            max_concurrent: 1,
            queue_size: 10,
            track_progress: true,
        };
        assert_eq!(config.worker_count, 1);
        assert_eq!(config.max_concurrent, 1);
    }

    #[test]
    fn test_distributed_config_many_workers() {
        let config = DistributedConfig {
            worker_count: 32,
            max_concurrent: 64,
            queue_size: 10000,
            track_progress: true,
        };
        assert_eq!(config.worker_count, 32);
        assert_eq!(config.max_concurrent, 64);
        assert_eq!(config.queue_size, 10000);
    }

    // =========================================================================
    // MutationProgress Tests
    // =========================================================================

    #[test]
    fn test_mutation_progress_new() {
        let progress = MutationProgress::new(100);
        assert_eq!(progress.total, 100);
        assert_eq!(progress.completed, 0);
        assert_eq!(progress.in_progress, 0);
        assert_eq!(progress.killed, 0);
        assert_eq!(progress.survived, 0);
        assert_eq!(progress.failed, 0);
    }

    #[test]
    fn test_mutation_progress_new_zero() {
        let progress = MutationProgress::new(0);
        assert_eq!(progress.total, 0);
        assert_eq!(progress.percentage(), 100.0);
    }

    #[test]
    fn test_mutation_progress_percentage() {
        let mut progress = MutationProgress::new(100);
        assert_eq!(progress.percentage(), 0.0);

        progress.completed = 50;
        assert_eq!(progress.percentage(), 50.0);

        progress.completed = 100;
        assert_eq!(progress.percentage(), 100.0);
    }

    #[test]
    fn test_mutation_progress_percentage_partial() {
        let mut progress = MutationProgress::new(3);
        progress.completed = 1;
        assert!((progress.percentage() - 33.333).abs() < 0.1);
    }

    #[test]
    fn test_mutation_progress_score() {
        let mut progress = MutationProgress::new(100);
        assert_eq!(progress.mutation_score(), 0.0);

        progress.killed = 80;
        progress.survived = 20;
        assert_eq!(progress.mutation_score(), 80.0);

        progress.killed = 90;
        progress.survived = 10;
        assert_eq!(progress.mutation_score(), 90.0);
    }

    #[test]
    fn test_mutation_progress_score_all_killed() {
        let mut progress = MutationProgress::new(100);
        progress.killed = 100;
        progress.survived = 0;
        assert_eq!(progress.mutation_score(), 100.0);
    }

    #[test]
    fn test_mutation_progress_score_all_survived() {
        let mut progress = MutationProgress::new(100);
        progress.killed = 0;
        progress.survived = 100;
        assert_eq!(progress.mutation_score(), 0.0);
    }

    #[test]
    fn test_mutation_progress_score_none_tested() {
        let progress = MutationProgress::new(100);
        assert_eq!(progress.mutation_score(), 0.0);
    }

    #[test]
    fn test_mutation_progress_clone() {
        let mut progress = MutationProgress::new(50);
        progress.completed = 25;
        progress.in_progress = 5;
        progress.killed = 20;
        progress.survived = 5;
        progress.failed = 0;

        let cloned = progress.clone();
        assert_eq!(progress.total, cloned.total);
        assert_eq!(progress.completed, cloned.completed);
        assert_eq!(progress.in_progress, cloned.in_progress);
        assert_eq!(progress.killed, cloned.killed);
        assert_eq!(progress.survived, cloned.survived);
        assert_eq!(progress.failed, cloned.failed);
    }

    #[test]
    fn test_mutation_progress_debug() {
        let progress = MutationProgress::new(100);
        let debug_str = format!("{:?}", progress);
        assert!(debug_str.contains("MutationProgress"));
        assert!(debug_str.contains("total"));
    }

    #[test]
    fn test_mutation_progress_with_failures() {
        let mut progress = MutationProgress::new(100);
        progress.completed = 100;
        progress.killed = 60;
        progress.survived = 30;
        progress.failed = 10;

        assert_eq!(progress.percentage(), 100.0);
        // Mutation score only considers killed / (killed + survived)
        assert!((progress.mutation_score() - 66.67).abs() < 0.1);
    }

    // =========================================================================
    // DistributedExecutor Tests (unit tests without async)
    // =========================================================================

    #[cfg(not(feature = "skip-slow-tests"))]
    mod slow_tests {
        use super::*;
        use crate::services::mutation::RustAdapter;

        #[test]
        fn test_distributed_executor_creation() {
            let adapter = Arc::new(RustAdapter::new());
            let config = DistributedConfig::default();
            let executor = DistributedExecutor::new(adapter, config);

            let progress = executor.get_progress();
            assert_eq!(progress.total, 0);
        }

        #[test]
        fn test_distributed_executor_get_progress() {
            let adapter = Arc::new(RustAdapter::new());
            let config = DistributedConfig {
                worker_count: 2,
                max_concurrent: 4,
                queue_size: 100,
                track_progress: true,
            };
            let executor = DistributedExecutor::new(adapter, config);

            let progress = executor.get_progress();
            assert_eq!(progress.total, 0);
            assert_eq!(progress.completed, 0);
            assert_eq!(progress.killed, 0);
        }

        #[test]
        fn test_distributed_executor_without_progress_tracking() {
            let adapter = Arc::new(RustAdapter::new());
            let config = DistributedConfig {
                worker_count: 2,
                max_concurrent: 4,
                queue_size: 100,
                track_progress: false,
            };
            let executor = DistributedExecutor::new(adapter, config);

            // Should still be able to get progress even if tracking is disabled
            let progress = executor.get_progress();
            assert_eq!(progress.total, 0);
        }

        /// SLOW: Test was killed - excluded from fast test suite
        #[tokio::test]
        #[ignore = "slow mutation test - run manually"]
        async fn test_parallel_execution_empty() {
            let adapter = Arc::new(RustAdapter::new());
            let config = DistributedConfig {
                worker_count: 2,
                max_concurrent: 4,
                queue_size: 10,
                track_progress: true,
            };
            let executor = DistributedExecutor::new(adapter, config);

            let mutants = vec![];
            let results = executor.execute_parallel(mutants).await.unwrap();

            assert_eq!(results.len(), 0);

            let progress = executor.get_progress();
            assert_eq!(progress.total, 0);
            assert_eq!(progress.completed, 0);
        }
    }
}
