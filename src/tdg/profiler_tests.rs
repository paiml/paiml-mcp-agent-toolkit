// profiler_tests.rs — Unit tests and property tests for profiler
// Included from profiler.rs — shares parent module scope (no use imports here)

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Duration;

    #[tokio::test]
    #[ignore] // Timing-sensitive test - needs investigation
    async fn test_operation_profiling() {
        let profiler = PerformanceProfiler::new(ProfilerConfig::default());

        let handle = profiler
            .start_operation("test_op_1".to_string(), "analysis".to_string())
            .await
            .expect("internal error");

        tokio::time::sleep(Duration::from_millis(100)).await;

        handle.complete().await.expect("internal error");

        let summary = profiler.get_summary().await;
        assert_eq!(summary.completed_operations, 1);
        assert!(summary.avg_operation_duration_ms >= 100.0);
    }

    #[tokio::test]
    async fn test_bottleneck_detection() {
        let profiler = PerformanceProfiler::new(ProfilerConfig::default());

        let profile = OperationProfile {
            operation_id: "slow_op".to_string(),
            operation_type: "compute".to_string(),
            start_time: Instant::now(),
            end_time: Some(Instant::now() + Duration::from_secs(10)),
            duration_ms: Some(10000.0),
            memory_before_mb: 100.0,
            memory_after_mb: Some(700.0),
            cpu_time_ms: 9000.0,
            io_wait_ms: 500.0,
            children: Vec::new(),
            metadata: HashMap::new(),
        };

        profiler
            .detect_bottlenecks(&profile)
            .await
            .expect("internal error");

        let bottlenecks = profiler.get_top_bottlenecks(10).await;
        assert!(!bottlenecks.is_empty());
        assert!(bottlenecks
            .iter()
            .any(|b| b.bottleneck_type == BottleneckType::CpuBound));
        assert!(bottlenecks
            .iter()
            .any(|b| b.bottleneck_type == BottleneckType::MemoryBound));
    }

    #[tokio::test]
    #[ignore] // Timing-sensitive test - needs investigation
    async fn test_flame_graph_generation() {
        let profiler = PerformanceProfiler::new(ProfilerConfig::default());

        // Create some operations
        for i in 0..3 {
            let handle = profiler
                .start_operation(format!("op_{}", i), "test".to_string())
                .await
                .expect("internal error");

            tokio::time::sleep(Duration::from_millis(50)).await;
            handle.complete().await.expect("internal error");
        }

        let flame_graph = profiler
            .generate_flame_graph()
            .await
            .expect("internal error");
        assert_eq!(flame_graph.name, "root");
        assert_eq!(flame_graph.children.len(), 3);
    }
}

