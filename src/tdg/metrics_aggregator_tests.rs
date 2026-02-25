// ============ DataPoint Tests ============

#[test]
fn test_data_point_creation() {
    let point = DataPoint {
        timestamp: SystemTime::now(),
        value: 42.0,
        tags: HashMap::from([("key".to_string(), "value".to_string())]),
    };
    assert_eq!(point.value, 42.0);
    assert!(point.tags.contains_key("key"));
}

#[test]
fn test_data_point_clone() {
    let point = DataPoint {
        timestamp: SystemTime::now(),
        value: 100,
        tags: HashMap::new(),
    };
    let cloned = point.clone();
    assert_eq!(cloned.value, 100);
}

#[test]
fn test_data_point_serialization() {
    let point = DataPoint {
        timestamp: SystemTime::now(),
        value: 55.5,
        tags: HashMap::new(),
    };
    let json = serde_json::to_string(&point).unwrap();
    let deserialized: DataPoint<f64> = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.value, 55.5);
}

// ============ RollingWindow Tests ============

#[tokio::test]
async fn test_rolling_window() {
    let mut window = RollingWindow::new(Duration::from_secs(60), 10);

    for i in 0..5 {
        window.push(i as f64, HashMap::new());
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    assert_eq!(window.get_window().len(), 5);
}

#[test]
fn test_rolling_window_new() {
    let window: RollingWindow<f64> = RollingWindow::new(Duration::from_secs(30), 100);
    assert!(window.is_empty());
    assert_eq!(window.get_window().len(), 0);
}

#[test]
fn test_rolling_window_max_points() {
    let mut window: RollingWindow<i32> = RollingWindow::new(Duration::from_secs(3600), 5);
    for i in 0..10 {
        window.push(i, HashMap::new());
    }
    // Should only keep last 5 points
    assert_eq!(window.get_window().len(), 5);
}

#[test]
fn test_rolling_window_is_empty() {
    let window: RollingWindow<u64> = RollingWindow::new(Duration::from_secs(60), 10);
    assert!(window.is_empty());
}

#[test]
fn test_rolling_window_with_tags() {
    let mut window: RollingWindow<f64> = RollingWindow::new(Duration::from_secs(60), 10);
    let mut tags = HashMap::new();
    tags.insert("alert".to_string(), "warning".to_string());
    window.push(1.0, tags);

    let data = window.get_window();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0].tags.get("alert"), Some(&"warning".to_string()));
}

// ============ AggregatedStats Tests ============

#[test]
fn test_aggregated_stats_serialization() {
    let stats = AggregatedStats {
        count: 10,
        mean: 50.0,
        median: 48.0,
        min: 10.0,
        max: 90.0,
        std_dev: 15.0,
        p95: 85.0,
        p99: 89.0,
        trend: TrendDirection::Rising,
        anomalies: vec![],
    };
    let json = serde_json::to_string(&stats).unwrap();
    let deserialized: AggregatedStats = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.count, 10);
    assert_eq!(deserialized.mean, 50.0);
}

// ============ TrendDirection Tests ============

#[test]
fn test_trend_direction_clone() {
    let trend = TrendDirection::Rising;
    let cloned = trend.clone();
    assert_eq!(cloned, TrendDirection::Rising);
}

#[test]
fn test_trend_direction_serialization() {
    let trend = TrendDirection::Volatile;
    let json = serde_json::to_string(&trend).unwrap();
    let deserialized: TrendDirection = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, TrendDirection::Volatile);
}

#[test]
fn test_trend_direction_all_variants() {
    assert_eq!(TrendDirection::Rising, TrendDirection::Rising);
    assert_eq!(TrendDirection::Falling, TrendDirection::Falling);
    assert_eq!(TrendDirection::Stable, TrendDirection::Stable);
    assert_eq!(TrendDirection::Volatile, TrendDirection::Volatile);
}

// ============ AnomalyPoint Tests ============

#[test]
fn test_anomaly_point_creation() {
    let point = AnomalyPoint {
        timestamp: SystemTime::now(),
        value: 999.9,
        severity: AnomalySeverity::High,
        deviation: 4.5,
    };
    assert_eq!(point.value, 999.9);
    assert_eq!(point.severity, AnomalySeverity::High);
}

#[test]
fn test_anomaly_point_serialization() {
    let point = AnomalyPoint {
        timestamp: SystemTime::now(),
        value: 100.0,
        severity: AnomalySeverity::Critical,
        deviation: 5.0,
    };
    let json = serde_json::to_string(&point).unwrap();
    assert!(json.contains("Critical"));
}

// ============ AnomalySeverity Tests ============

#[test]
fn test_anomaly_severity_all_variants() {
    assert_eq!(AnomalySeverity::Low, AnomalySeverity::Low);
    assert_eq!(AnomalySeverity::Medium, AnomalySeverity::Medium);
    assert_eq!(AnomalySeverity::High, AnomalySeverity::High);
    assert_eq!(AnomalySeverity::Critical, AnomalySeverity::Critical);
}

#[test]
fn test_anomaly_severity_serialization() {
    let severity = AnomalySeverity::Medium;
    let json = serde_json::to_string(&severity).unwrap();
    let deserialized: AnomalySeverity = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, AnomalySeverity::Medium);
}

// ============ StorageMetricPoint Tests ============

#[test]
fn test_storage_metric_point_creation() {
    let point = StorageMetricPoint {
        total_entries: 1000,
        cache_hit_ratio: 0.85,
        compression_ratio: 0.6,
        storage_size_mb: 512.0,
        write_throughput: 100.0,
        read_throughput: 500.0,
    };
    assert_eq!(point.total_entries, 1000);
    assert_eq!(point.cache_hit_ratio, 0.85);
}

#[test]
fn test_storage_metric_point_serialization() {
    let point = StorageMetricPoint {
        total_entries: 500,
        cache_hit_ratio: 0.9,
        compression_ratio: 0.5,
        storage_size_mb: 256.0,
        write_throughput: 50.0,
        read_throughput: 200.0,
    };
    let json = serde_json::to_string(&point).unwrap();
    let deserialized: StorageMetricPoint = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.total_entries, 500);
}

// ============ PerformanceMetricPoint Tests ============

#[test]
fn test_performance_metric_point_creation() {
    let point = PerformanceMetricPoint {
        avg_analysis_time_ms: 150.0,
        active_operations: 5,
        queue_depth: 10,
        cpu_usage_percent: 45.0,
        memory_usage_mb: 1024.0,
        gc_pause_ms: 5.0,
    };
    assert_eq!(point.avg_analysis_time_ms, 150.0);
    assert_eq!(point.active_operations, 5);
}

#[test]
fn test_performance_metric_point_serialization() {
    let point = PerformanceMetricPoint {
        avg_analysis_time_ms: 200.0,
        active_operations: 3,
        queue_depth: 5,
        cpu_usage_percent: 30.0,
        memory_usage_mb: 512.0,
        gc_pause_ms: 2.0,
    };
    let json = serde_json::to_string(&point).unwrap();
    let deserialized: PerformanceMetricPoint = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.queue_depth, 5);
}

// ============ AnalysisMetricPoint Tests ============

#[test]
fn test_analysis_metric_point_creation() {
    let point = AnalysisMetricPoint {
        files_analyzed: 100,
        avg_tdg_score: 85.5,
        critical_issues: 2,
        success_rate: 0.98,
        cache_hits: 80,
        cache_misses: 20,
    };
    assert_eq!(point.files_analyzed, 100);
    assert_eq!(point.success_rate, 0.98);
}

#[test]
fn test_analysis_metric_point_serialization() {
    let point = AnalysisMetricPoint {
        files_analyzed: 50,
        avg_tdg_score: 75.0,
        critical_issues: 0,
        success_rate: 1.0,
        cache_hits: 45,
        cache_misses: 5,
    };
    let json = serde_json::to_string(&point).unwrap();
    let deserialized: AnalysisMetricPoint = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.cache_hits, 45);
}

// ============ AlertThresholds Tests ============

#[test]
fn test_alert_thresholds_default() {
    let thresholds = AlertThresholds::default();
    assert_eq!(thresholds.cpu_critical, 90.0);
    assert_eq!(thresholds.memory_critical_mb, 8192.0);
    assert_eq!(thresholds.queue_depth_warning, 100);
    assert_eq!(thresholds.analysis_time_warning_ms, 5000.0);
    assert_eq!(thresholds.cache_hit_ratio_warning, 0.5);
    assert_eq!(thresholds.storage_usage_warning_percent, 85.0);
}

#[test]
fn test_alert_thresholds_serialization() {
    let thresholds = AlertThresholds::default();
    let json = serde_json::to_string(&thresholds).unwrap();
    let deserialized: AlertThresholds = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.cpu_critical, 90.0);
}

// ============ Alert Tests ============

#[test]
fn test_alert_creation() {
    let alert = Alert {
        severity: AlertSeverity::Warning,
        message: "Test warning".to_string(),
        timestamp: SystemTime::now(),
        metric: "test_metric".to_string(),
    };
    assert_eq!(alert.severity, AlertSeverity::Warning);
    assert_eq!(alert.metric, "test_metric");
}

#[test]
fn test_alert_serialization() {
    let alert = Alert {
        severity: AlertSeverity::Critical,
        message: "Critical issue".to_string(),
        timestamp: SystemTime::now(),
        metric: "cpu".to_string(),
    };
    let json = serde_json::to_string(&alert).unwrap();
    assert!(json.contains("Critical"));
}

// ============ AlertSeverity Tests ============

#[test]
fn test_alert_severity_all_variants() {
    assert_eq!(AlertSeverity::Info, AlertSeverity::Info);
    assert_eq!(AlertSeverity::Warning, AlertSeverity::Warning);
    assert_eq!(AlertSeverity::Error, AlertSeverity::Error);
    assert_eq!(AlertSeverity::Critical, AlertSeverity::Critical);
}

#[test]
fn test_alert_severity_serialization() {
    let severity = AlertSeverity::Error;
    let json = serde_json::to_string(&severity).unwrap();
    let deserialized: AlertSeverity = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, AlertSeverity::Error);
}

// ============ ExportFormat Tests ============

#[test]
fn test_export_format_debug() {
    let format = ExportFormat::Json;
    let debug = format!("{:?}", format);
    assert!(debug.contains("Json"));
}

#[test]
fn test_export_format_clone_copy() {
    let format = ExportFormat::Csv;
    let cloned = format;
    assert!(matches!(cloned, ExportFormat::Csv));
}
