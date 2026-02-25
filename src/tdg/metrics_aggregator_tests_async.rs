// ============ MetricsAggregator Tests ============

#[tokio::test]
async fn test_metrics_aggregator_default() {
    let aggregator = MetricsAggregator::default();
    let stats = aggregator.aggregate_performance_stats().await;
    assert_eq!(stats.count, 0);
}

#[tokio::test]
async fn test_metrics_aggregation() {
    let aggregator = MetricsAggregator::new();

    for i in 0..10 {
        let metrics = PerformanceMetricPoint {
            avg_analysis_time_ms: (i * 100) as f64,
            active_operations: i as u32,
            queue_depth: i as u32,
            cpu_usage_percent: (i * 10) as f64,
            memory_usage_mb: (i * 100) as f64,
            gc_pause_ms: 0.0,
        };
        aggregator
            .record_performance_metrics(metrics)
            .await
            .expect("internal error");
    }

    let stats = aggregator.aggregate_performance_stats().await;
    assert!(stats.count > 0);
    assert!(stats.mean > 0.0);
}

#[tokio::test]
async fn test_record_storage_metrics() {
    let aggregator = MetricsAggregator::new();
    let metrics = StorageMetricPoint {
        total_entries: 1000,
        cache_hit_ratio: 0.9,
        compression_ratio: 0.5,
        storage_size_mb: 100.0,
        write_throughput: 50.0,
        read_throughput: 200.0,
    };
    let result = aggregator.record_storage_metrics(metrics).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_record_analysis_metrics() {
    let aggregator = MetricsAggregator::new();
    let metrics = AnalysisMetricPoint {
        files_analyzed: 50,
        avg_tdg_score: 80.0,
        critical_issues: 1,
        success_rate: 0.95,
        cache_hits: 40,
        cache_misses: 10,
    };
    let result = aggregator.record_analysis_metrics(metrics).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_alert_detection() {
    let aggregator = MetricsAggregator::new();

    let critical_metrics = PerformanceMetricPoint {
        avg_analysis_time_ms: 10000.0,
        active_operations: 10,
        queue_depth: 200,
        cpu_usage_percent: 95.0,
        memory_usage_mb: 9000.0,
        gc_pause_ms: 100.0,
    };
    aggregator
        .record_performance_metrics(critical_metrics)
        .await
        .expect("internal error");

    let alerts = aggregator.get_alert_status().await;
    assert!(!alerts.is_empty());
    assert!(alerts.iter().any(|a| a.severity == AlertSeverity::Critical));
}

#[tokio::test]
async fn test_get_alert_status_no_alerts() {
    let aggregator = MetricsAggregator::new();
    let alerts = aggregator.get_alert_status().await;
    assert!(alerts.is_empty());
}

#[tokio::test]
async fn test_export_metrics_json() {
    let aggregator = MetricsAggregator::new();
    let result = aggregator.export_metrics(ExportFormat::Json).await;
    assert!(result.is_ok());
    let json = result.unwrap();
    assert!(json.contains("storage"));
    assert!(json.contains("performance"));
}

#[tokio::test]
async fn test_export_metrics_csv() {
    let aggregator = MetricsAggregator::new();
    let result = aggregator.export_metrics(ExportFormat::Csv).await;
    assert!(result.is_ok());
    let csv = result.unwrap();
    assert!(csv.contains("timestamp,metric_type,metric_name,value"));
}

#[tokio::test]
async fn test_export_metrics_prometheus() {
    let aggregator = MetricsAggregator::new();
    let result = aggregator.export_metrics(ExportFormat::Prometheus).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_export_metrics_prometheus_with_data() {
    let aggregator = MetricsAggregator::new();

    let storage = StorageMetricPoint {
        total_entries: 100,
        cache_hit_ratio: 0.8,
        compression_ratio: 0.5,
        storage_size_mb: 50.0,
        write_throughput: 10.0,
        read_throughput: 50.0,
    };
    aggregator.record_storage_metrics(storage).await.unwrap();

    let perf = PerformanceMetricPoint {
        avg_analysis_time_ms: 100.0,
        active_operations: 5,
        queue_depth: 10,
        cpu_usage_percent: 30.0,
        memory_usage_mb: 512.0,
        gc_pause_ms: 1.0,
    };
    aggregator.record_performance_metrics(perf).await.unwrap();

    let result = aggregator.export_metrics(ExportFormat::Prometheus).await;
    assert!(result.is_ok());
    let prom = result.unwrap();
    assert!(prom.contains("tdg_storage_entries"));
    assert!(prom.contains("tdg_analysis_time_ms"));
}

// === Anomaly Detection Tests ===

#[tokio::test]
async fn test_detect_anomalies_critical_severity() {
    let aggregator = MetricsAggregator::new();
    // Create 10 normal points around 100, then one extreme outlier
    for i in 0..10 {
        let metrics = PerformanceMetricPoint {
            avg_analysis_time_ms: 100.0 + (i as f64),
            active_operations: 1,
            queue_depth: 1,
            cpu_usage_percent: 10.0,
            memory_usage_mb: 100.0,
            gc_pause_ms: 0.0,
        };
        aggregator
            .record_performance_metrics(metrics)
            .await
            .unwrap();
    }
    // Add extreme outlier (z-score > 4.0)
    let outlier = PerformanceMetricPoint {
        avg_analysis_time_ms: 500.0,
        active_operations: 1,
        queue_depth: 1,
        cpu_usage_percent: 10.0,
        memory_usage_mb: 100.0,
        gc_pause_ms: 0.0,
    };
    aggregator
        .record_performance_metrics(outlier)
        .await
        .unwrap();

    let stats = aggregator.aggregate_performance_stats().await;
    assert!(!stats.anomalies.is_empty(), "Should detect anomaly");
    assert!(
        stats.anomalies.iter().any(|a| a.deviation > 3.0),
        "Anomaly deviation should exceed 3.0"
    );
}

#[tokio::test]
async fn test_detect_anomalies_zero_std_dev() {
    let aggregator = MetricsAggregator::new();
    // All identical values -> std_dev = 0, no anomalies
    for _ in 0..5 {
        let metrics = PerformanceMetricPoint {
            avg_analysis_time_ms: 100.0,
            active_operations: 1,
            queue_depth: 1,
            cpu_usage_percent: 10.0,
            memory_usage_mb: 100.0,
            gc_pause_ms: 0.0,
        };
        aggregator
            .record_performance_metrics(metrics)
            .await
            .unwrap();
    }
    let stats = aggregator.aggregate_performance_stats().await;
    assert!(stats.anomalies.is_empty(), "No anomalies with zero std_dev");
}

// === Export Metrics with Data ===

#[tokio::test]
async fn test_export_metrics_csv_with_data() {
    let aggregator = MetricsAggregator::new();

    let storage = StorageMetricPoint {
        total_entries: 500,
        cache_hit_ratio: 0.75,
        compression_ratio: 0.6,
        storage_size_mb: 100.0,
        write_throughput: 10.0,
        read_throughput: 50.0,
    };
    aggregator.record_storage_metrics(storage).await.unwrap();

    let perf = PerformanceMetricPoint {
        avg_analysis_time_ms: 250.0,
        active_operations: 3,
        queue_depth: 5,
        cpu_usage_percent: 42.0,
        memory_usage_mb: 512.0,
        gc_pause_ms: 1.0,
    };
    aggregator.record_performance_metrics(perf).await.unwrap();

    let csv = aggregator.export_metrics(ExportFormat::Csv).await.unwrap();
    assert!(csv.contains("storage,total_entries,500"));
    assert!(csv.contains("storage,cache_hit_ratio,0.75"));
    assert!(csv.contains("performance,avg_analysis_time_ms,250"));
    assert!(csv.contains("performance,cpu_usage_percent,42"));
}

#[tokio::test]
async fn test_export_metrics_prometheus_storage_branch() {
    let aggregator = MetricsAggregator::new();

    let storage = StorageMetricPoint {
        total_entries: 999,
        cache_hit_ratio: 0.95,
        compression_ratio: 0.5,
        storage_size_mb: 50.0,
        write_throughput: 10.0,
        read_throughput: 50.0,
    };
    aggregator.record_storage_metrics(storage).await.unwrap();

    let prom = aggregator
        .export_metrics(ExportFormat::Prometheus)
        .await
        .unwrap();
    assert!(prom.contains("tdg_storage_entries 999"));
    assert!(prom.contains("tdg_cache_hit_ratio 0.95"));
    assert!(prom.contains("# HELP tdg_storage_entries"));
    assert!(prom.contains("# TYPE tdg_storage_entries gauge"));
}

#[tokio::test]
async fn test_export_metrics_prometheus_perf_branch() {
    let aggregator = MetricsAggregator::new();

    let perf = PerformanceMetricPoint {
        avg_analysis_time_ms: 123.5,
        active_operations: 2,
        queue_depth: 3,
        cpu_usage_percent: 55.0,
        memory_usage_mb: 256.0,
        gc_pause_ms: 0.5,
    };
    aggregator.record_performance_metrics(perf).await.unwrap();

    let prom = aggregator
        .export_metrics(ExportFormat::Prometheus)
        .await
        .unwrap();
    assert!(prom.contains("tdg_analysis_time_ms 123.5"));
    assert!(prom.contains("tdg_cpu_usage_percent 55"));
    assert!(prom.contains("# HELP tdg_analysis_time_ms"));
    assert!(prom.contains("# TYPE tdg_cpu_usage_percent gauge"));
}
