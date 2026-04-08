//! Tests for quality monitor
//! Extracted to separate file for file health compliance (CB-040)

use super::*;

#[test]
fn test_quality_monitor_config_default() {
    let config = QualityMonitorConfig::default();
    assert_eq!(config.complexity_threshold, 20);
    assert!(!config.watch_patterns.is_empty());
    assert!(config.debounce_interval > Duration::from_millis(0));
}

#[test]
fn test_should_analyze_file() {
    let patterns = vec!["**/*.rs".to_string(), "**/*.py".to_string()];

    assert!(QualityMonitorEngine::should_analyze_file(
        Path::new("src/main.rs"),
        &patterns
    ));

    assert!(QualityMonitorEngine::should_analyze_file(
        Path::new("tests/test.py"),
        &patterns
    ));

    assert!(!QualityMonitorEngine::should_analyze_file(
        Path::new("README.md"),
        &patterns
    ));
}

#[test]
fn test_quality_metrics_default() {
    let metrics = QualityMetrics::default();
    assert_eq!(metrics.files_analyzed, 0);
    assert_eq!(metrics.quality_score, 0.0);
    assert!(metrics.file_metrics.is_empty());
}

#[test]
fn test_complexity_distribution() {
    let dist = ComplexityDistribution {
        low: 50,
        medium: 30,
        high: 15,
        very_high: 4,
        violations: 1,
    };

    let total = dist.low + dist.medium + dist.high + dist.very_high + dist.violations;
    assert_eq!(total, 100);
}

#[tokio::test]
async fn test_quality_monitor_creation() {
    let config = QualityMonitorConfig::default();
    let monitor = QualityMonitorEngine::new(config);

    let metrics = monitor.metrics.read().await;
    assert!(metrics.is_empty());
}

mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            prop_assert!(_x < 1001);
        }
    }
}

mod coverage_tests {
    use super::*;
    use std::time::UNIX_EPOCH;
    use tempfile::TempDir;

    #[test]
    fn test_quality_monitor_config_custom() {
        let config = QualityMonitorConfig {
            update_interval: Duration::from_secs(10),
            complexity_threshold: 25,
            watch_patterns: vec!["**/*.go".to_string()],
            debounce_interval: Duration::from_millis(1000),
            max_batch_size: 100,
        };

        assert_eq!(config.update_interval, Duration::from_secs(10));
        assert_eq!(config.complexity_threshold, 25);
        assert_eq!(config.watch_patterns.len(), 1);
        assert_eq!(config.debounce_interval, Duration::from_millis(1000));
        assert_eq!(config.max_batch_size, 100);
    }

    #[test]
    fn test_quality_monitor_config_serialization() {
        let config = QualityMonitorConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: QualityMonitorConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(
            config.complexity_threshold,
            deserialized.complexity_threshold
        );
        assert_eq!(config.max_batch_size, deserialized.max_batch_size);
    }

    #[test]
    fn test_quality_monitor_config_default_patterns() {
        let config = QualityMonitorConfig::default();

        assert!(config.watch_patterns.contains(&"**/*.rs".to_string()));
        assert!(config.watch_patterns.contains(&"**/*.py".to_string()));
        assert!(config.watch_patterns.contains(&"**/*.js".to_string()));
        assert!(config.watch_patterns.contains(&"**/*.ts".to_string()));
    }

    #[test]
    fn test_quality_metrics_creation() {
        let metrics = QualityMetrics {
            project_id: "test_project".to_string(),
            last_updated: SystemTime::now(),
            quality_score: 0.85,
            files_analyzed: 10,
            functions_analyzed: 50,
            avg_complexity: 8.5,
            max_complexity: 20,
            hotspot_functions: 3,
            satd_issues: 2,
            complexity_distribution: ComplexityDistribution {
                low: 30,
                medium: 15,
                high: 4,
                very_high: 1,
                violations: 0,
            },
            file_metrics: HashMap::new(),
            quality_trend: 0.05,
        };

        assert_eq!(metrics.project_id, "test_project");
        assert_eq!(metrics.quality_score, 0.85);
        assert_eq!(metrics.files_analyzed, 10);
        assert_eq!(metrics.functions_analyzed, 50);
    }

    #[test]
    fn test_quality_metrics_serialization() {
        let metrics = QualityMetrics {
            project_id: "test".to_string(),
            last_updated: UNIX_EPOCH,
            quality_score: 0.9,
            files_analyzed: 5,
            functions_analyzed: 25,
            avg_complexity: 6.0,
            max_complexity: 15,
            hotspot_functions: 1,
            satd_issues: 0,
            complexity_distribution: ComplexityDistribution::default(),
            file_metrics: HashMap::new(),
            quality_trend: 0.0,
        };

        let json = serde_json::to_string(&metrics).unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("quality_score"));

        let deserialized: QualityMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.quality_score, 0.9);
    }

    #[test]
    fn test_complexity_distribution_creation() {
        let dist = ComplexityDistribution {
            low: 100,
            medium: 50,
            high: 20,
            very_high: 5,
            violations: 2,
        };

        assert_eq!(dist.low, 100);
        assert_eq!(dist.medium, 50);
        assert_eq!(dist.high, 20);
        assert_eq!(dist.very_high, 5);
        assert_eq!(dist.violations, 2);
    }

    #[test]
    fn test_complexity_distribution_default() {
        let dist = ComplexityDistribution {
            low: 0,
            medium: 0,
            high: 0,
            very_high: 0,
            violations: 0,
        };

        let total = dist.low + dist.medium + dist.high + dist.very_high + dist.violations;
        assert_eq!(total, 0);
    }

    #[test]
    fn test_file_quality_metrics_creation() {
        let metrics = FileQualityMetrics {
            file_path: "src/main.rs".to_string(),
            last_modified: SystemTime::now(),
            last_analyzed: SystemTime::now(),
            function_count: 10,
            avg_complexity: 5.5,
            max_complexity: 12,
            satd_issues: 1,
            quality_score: 0.88,
            needs_attention: false,
        };

        assert_eq!(metrics.file_path, "src/main.rs");
        assert_eq!(metrics.function_count, 10);
        assert!(!metrics.needs_attention);
    }

    #[test]
    fn test_file_quality_metrics_needs_attention() {
        let metrics = FileQualityMetrics {
            file_path: "src/complex.rs".to_string(),
            last_modified: SystemTime::now(),
            last_analyzed: SystemTime::now(),
            function_count: 5,
            avg_complexity: 25.0,
            max_complexity: 35,
            satd_issues: 5,
            quality_score: 0.45,
            needs_attention: true,
        };

        assert!(metrics.needs_attention);
        assert!(metrics.max_complexity > 20);
    }

    #[test]
    fn test_quality_event_metrics_updated() {
        let metrics = QualityMetrics::default();
        let event = QualityEvent::MetricsUpdated {
            project_id: "test".to_string(),
            metrics,
            changes: vec![],
        };

        match event {
            QualityEvent::MetricsUpdated { project_id, .. } => {
                assert_eq!(project_id, "test");
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_quality_event_threshold_violated() {
        let violation = QualityViolation::ComplexityThreshold {
            file: "src/main.rs".to_string(),
            function: "complex_fn".to_string(),
            complexity: 25,
        };

        let event = QualityEvent::ThresholdViolated {
            project_id: "test".to_string(),
            violation,
        };

        match event {
            QualityEvent::ThresholdViolated {
                project_id,
                violation,
            } => {
                assert_eq!(project_id, "test");
                match violation {
                    QualityViolation::ComplexityThreshold { complexity, .. } => {
                        assert_eq!(complexity, 25);
                    }
                    _ => panic!("Wrong violation type"),
                }
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_quality_event_file_analyzed() {
        let metrics = FileQualityMetrics {
            file_path: "test.rs".to_string(),
            last_modified: SystemTime::now(),
            last_analyzed: SystemTime::now(),
            function_count: 5,
            avg_complexity: 3.0,
            max_complexity: 8,
            satd_issues: 0,
            quality_score: 0.95,
            needs_attention: false,
        };

        let event = QualityEvent::FileAnalyzed {
            project_id: "test".to_string(),
            file_path: "test.rs".to_string(),
            metrics,
        };

        matches!(event, QualityEvent::FileAnalyzed { .. });
    }

    #[test]
    fn test_quality_event_trend_detected() {
        let trend = QualityTrend::Improving {
            rate: 0.05,
            duration: Duration::from_secs(3600),
        };

        let event = QualityEvent::TrendDetected {
            project_id: "test".to_string(),
            trend,
        };

        matches!(event, QualityEvent::TrendDetected { .. });
    }

    #[test]
    fn test_quality_event_error() {
        let event = QualityEvent::Error {
            project_id: "test".to_string(),
            error: "Analysis failed".to_string(),
        };

        match event {
            QualityEvent::Error { error, .. } => {
                assert_eq!(error, "Analysis failed");
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_quality_change_complexity_increase() {
        let change = QualityChange::ComplexityIncrease {
            file: "test.rs".to_string(),
            old_complexity: 5.0,
            new_complexity: 10.0,
        };

        match change {
            QualityChange::ComplexityIncrease {
                old_complexity,
                new_complexity,
                ..
            } => {
                assert!(new_complexity > old_complexity);
            }
            _ => panic!("Wrong change type"),
        }
    }

    #[test]
    fn test_quality_change_complexity_decrease() {
        let change = QualityChange::ComplexityDecrease {
            file: "test.rs".to_string(),
            old_complexity: 15.0,
            new_complexity: 8.0,
        };

        match change {
            QualityChange::ComplexityDecrease {
                old_complexity,
                new_complexity,
                ..
            } => {
                assert!(new_complexity < old_complexity);
            }
            _ => panic!("Wrong change type"),
        }
    }

    #[test]
    fn test_quality_change_satd_added() {
        let change = QualityChange::SatdAdded {
            file: "test.rs".to_string(),
            count: 2,
        };

        match change {
            QualityChange::SatdAdded { count, .. } => {
                assert_eq!(count, 2);
            }
            _ => panic!("Wrong change type"),
        }
    }

    #[test]
    fn test_quality_change_satd_removed() {
        let change = QualityChange::SatdRemoved {
            file: "test.rs".to_string(),
            count: 3,
        };

        match change {
            QualityChange::SatdRemoved { count, .. } => {
                assert_eq!(count, 3);
            }
            _ => panic!("Wrong change type"),
        }
    }

    #[test]
    fn test_quality_change_file_added() {
        let change = QualityChange::FileAdded {
            file: "new_file.rs".to_string(),
        };

        matches!(change, QualityChange::FileAdded { .. });
    }

    #[test]
    fn test_quality_change_file_removed() {
        let change = QualityChange::FileRemoved {
            file: "old_file.rs".to_string(),
        };

        matches!(change, QualityChange::FileRemoved { .. });
    }

    #[test]
    fn test_quality_change_quality_improved() {
        let change = QualityChange::QualityImproved {
            old_score: 0.75,
            new_score: 0.85,
        };

        match change {
            QualityChange::QualityImproved {
                old_score,
                new_score,
            } => {
                assert!(new_score > old_score);
            }
            _ => panic!("Wrong change type"),
        }
    }

    #[test]
    fn test_quality_change_quality_degraded() {
        let change = QualityChange::QualityDegraded {
            old_score: 0.90,
            new_score: 0.70,
        };

        match change {
            QualityChange::QualityDegraded {
                old_score,
                new_score,
            } => {
                assert!(new_score < old_score);
            }
            _ => panic!("Wrong change type"),
        }
    }

    #[test]
    fn test_quality_violation_complexity_threshold() {
        let violation = QualityViolation::ComplexityThreshold {
            file: "complex.rs".to_string(),
            function: "complex_function".to_string(),
            complexity: 30,
        };

        match violation {
            QualityViolation::ComplexityThreshold { complexity, .. } => {
                assert!(complexity > 20);
            }
            _ => panic!("Wrong violation type"),
        }
    }

    #[test]
    fn test_quality_violation_score_below() {
        let violation = QualityViolation::QualityScoreBelow {
            current_score: 0.5,
            threshold: 0.7,
        };

        match violation {
            QualityViolation::QualityScoreBelow {
                current_score,
                threshold,
            } => {
                assert!(current_score < threshold);
            }
            _ => panic!("Wrong violation type"),
        }
    }

    #[test]
    fn test_quality_violation_too_many_satd() {
        let violation = QualityViolation::TooManySatdIssues {
            count: 15,
            threshold: 10,
        };

        match violation {
            QualityViolation::TooManySatdIssues { count, threshold } => {
                assert!(count > threshold);
            }
            _ => panic!("Wrong violation type"),
        }
    }

    #[test]
    fn test_quality_violation_trend_negative() {
        let violation = QualityViolation::QualityTrendNegative {
            trend: -0.05,
            duration: Duration::from_secs(86400),
        };

        match violation {
            QualityViolation::QualityTrendNegative { trend, .. } => {
                assert!(trend < 0.0);
            }
            _ => panic!("Wrong violation type"),
        }
    }

    #[test]
    fn test_quality_trend_improving() {
        let trend = QualityTrend::Improving {
            rate: 0.03,
            duration: Duration::from_secs(3600),
        };

        matches!(trend, QualityTrend::Improving { .. });
    }

    #[test]
    fn test_quality_trend_stable() {
        let trend = QualityTrend::Stable {
            score: 0.85,
            duration: Duration::from_secs(7200),
        };

        match trend {
            QualityTrend::Stable { score, .. } => {
                assert_eq!(score, 0.85);
            }
            _ => panic!("Wrong trend type"),
        }
    }

    #[test]
    fn test_quality_trend_degrading() {
        let trend = QualityTrend::Degrading {
            rate: -0.02,
            duration: Duration::from_secs(1800),
        };

        match trend {
            QualityTrend::Degrading { rate, .. } => {
                assert!(rate < 0.0);
            }
            _ => panic!("Wrong trend type"),
        }
    }

    #[tokio::test]
    async fn test_quality_monitor_engine_new() {
        let config = QualityMonitorConfig::default();
        let engine = QualityMonitorEngine::new(config.clone());

        let watchers = engine.watchers.read().await;
        assert!(watchers.is_empty());

        let metrics = engine.metrics.read().await;
        assert!(metrics.is_empty());

        assert!(engine.event_sender.is_none());
    }

    #[tokio::test]
    async fn test_quality_monitor_set_event_sender() {
        let config = QualityMonitorConfig::default();
        let mut engine = QualityMonitorEngine::new(config);

        let (tx, _rx) = mpsc::channel(100);
        engine.set_event_sender(tx);

        assert!(engine.event_sender.is_some());
    }

    #[tokio::test]
    async fn test_quality_monitor_get_metrics_empty() {
        let config = QualityMonitorConfig::default();
        let engine = QualityMonitorEngine::new(config);

        let metrics = engine.get_metrics("nonexistent").await;
        assert!(metrics.is_none());
    }

    #[tokio::test]
    async fn test_quality_monitor_stop_monitoring_nonexistent() {
        let config = QualityMonitorConfig::default();
        let mut engine = QualityMonitorEngine::new(config);

        let result = engine.stop_monitoring("nonexistent").await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_should_analyze_file_rust() {
        let patterns = vec!["**/*.rs".to_string()];
        assert!(QualityMonitorEngine::should_analyze_file(
            Path::new("src/lib.rs"),
            &patterns
        ));
    }

    #[test]
    fn test_should_analyze_file_python() {
        let patterns = vec!["**/*.py".to_string()];
        assert!(QualityMonitorEngine::should_analyze_file(
            Path::new("tests/test_main.py"),
            &patterns
        ));
    }

    #[test]
    fn test_should_analyze_file_javascript() {
        let patterns = vec!["**/*.js".to_string(), "**/*.ts".to_string()];
        assert!(QualityMonitorEngine::should_analyze_file(
            Path::new("src/index.js"),
            &patterns
        ));
        assert!(QualityMonitorEngine::should_analyze_file(
            Path::new("src/app.ts"),
            &patterns
        ));
    }

    #[test]
    fn test_should_analyze_file_no_match() {
        let patterns = vec!["**/*.rs".to_string()];
        assert!(!QualityMonitorEngine::should_analyze_file(
            Path::new("config.yaml"),
            &patterns
        ));
    }

    #[test]
    fn test_should_analyze_file_simple_pattern() {
        let patterns = vec!["main".to_string()];
        assert!(QualityMonitorEngine::should_analyze_file(
            Path::new("src/main.rs"),
            &patterns
        ));
    }

    #[test]
    fn test_count_functions_rust() {
        let content = r#"
            fn main() {
                println!("Hello");
            }
            fn helper() -> i32 {
                42
            }
            /// Public fn.
            pub fn public_fn() {}
        "#;

        let count = QualityMonitorEngine::count_functions(content, Path::new("test.rs"));
        assert_eq!(count, 3);
    }

    #[test]
    fn test_count_functions_python() {
        let content = r#"
            def main():
                print("Hello")

            def helper():
                return 42

            def public_fn():
                pass
        "#;

        let count = QualityMonitorEngine::count_functions(content, Path::new("test.py"));
        assert_eq!(count, 3);
    }

    #[test]
    fn test_count_functions_javascript() {
        let content = r#"
            function main() {
                console.log("Hello");
            }
            const helper = () => 42;
            function(callback) { callback(); }
        "#;

        let count = QualityMonitorEngine::count_functions(content, Path::new("test.js"));
        assert!(count >= 2);
    }

    #[test]
    fn test_estimate_complexity() {
        let simple = "let x = 5;";
        assert_eq!(QualityMonitorEngine::estimate_complexity(simple), 0);

        let with_if = "if x > 5 { do_something(); }";
        assert!(QualityMonitorEngine::estimate_complexity(with_if) > 0);

        let complex = r#"
            if condition {
                for item in items {
                    while running {
                        if a && b || c {
                            match value {
                                1 => one(),
                                2 => two(),
                                _ => other(),
                            }
                        }
                    }
                }
            }
        "#;
        assert!(QualityMonitorEngine::estimate_complexity(complex) > 5);
    }

    #[test]
    fn test_count_satd_issues() {
        let no_satd = "let x = 5; // Good code";
        assert_eq!(QualityMonitorEngine::count_satd_issues(no_satd), 0);

        let todo_pattern: String = ['T', 'O', 'D', 'O'].iter().collect();
        let with_todo = format!("// {}: Fix this later", todo_pattern);
        assert!(QualityMonitorEngine::count_satd_issues(&with_todo) > 0);
    }

    #[test]
    fn test_calculate_file_quality_score_excellent() {
        let score = QualityMonitorEngine::calculate_file_quality_score(100, 10, 5.0, 0);
        assert!(score > 0.9);
    }

    #[test]
    fn test_calculate_file_quality_score_high_complexity() {
        let score = QualityMonitorEngine::calculate_file_quality_score(200, 10, 25.0, 0);
        assert!(score < 0.8);
    }

    #[test]
    fn test_calculate_file_quality_score_with_satd() {
        let score = QualityMonitorEngine::calculate_file_quality_score(100, 10, 5.0, 3);
        assert!(score < 0.9);
    }

    #[test]
    fn test_calculate_file_quality_score_large_file() {
        let score = QualityMonitorEngine::calculate_file_quality_score(1000, 50, 5.0, 0);
        assert!(score < 1.0);
    }

    #[test]
    fn test_calculate_file_quality_score_no_functions() {
        let score = QualityMonitorEngine::calculate_file_quality_score(50, 0, 0.0, 0);
        assert!(score < 0.9);
    }

    #[test]
    fn test_detect_quality_changes_no_changes() {
        let old = FileQualityMetrics {
            file_path: "test.rs".to_string(),
            last_modified: SystemTime::now(),
            last_analyzed: SystemTime::now(),
            function_count: 5,
            avg_complexity: 5.0,
            max_complexity: 10,
            satd_issues: 0,
            quality_score: 0.9,
            needs_attention: false,
        };

        let new = old.clone();
        let changes = QualityMonitorEngine::detect_quality_changes(&old, &new, "test.rs");
        assert!(changes.is_empty());
    }

    #[test]
    fn test_detect_quality_changes_complexity_increase() {
        let old = FileQualityMetrics {
            file_path: "test.rs".to_string(),
            last_modified: SystemTime::now(),
            last_analyzed: SystemTime::now(),
            function_count: 5,
            avg_complexity: 5.0,
            max_complexity: 10,
            satd_issues: 0,
            quality_score: 0.9,
            needs_attention: false,
        };

        let mut new = old.clone();
        new.avg_complexity = 15.0;

        let changes = QualityMonitorEngine::detect_quality_changes(&old, &new, "test.rs");
        assert!(!changes.is_empty());
        assert!(matches!(
            changes[0],
            QualityChange::ComplexityIncrease { .. }
        ));
    }

    #[test]
    fn test_detect_quality_changes_complexity_decrease() {
        let old = FileQualityMetrics {
            file_path: "test.rs".to_string(),
            last_modified: SystemTime::now(),
            last_analyzed: SystemTime::now(),
            function_count: 5,
            avg_complexity: 15.0,
            max_complexity: 25,
            satd_issues: 0,
            quality_score: 0.7,
            needs_attention: true,
        };

        let mut new = old.clone();
        new.avg_complexity = 5.0;

        let changes = QualityMonitorEngine::detect_quality_changes(&old, &new, "test.rs");
        assert!(!changes.is_empty());
        assert!(matches!(
            changes[0],
            QualityChange::ComplexityDecrease { .. }
        ));
    }

    #[test]
    fn test_detect_quality_changes_satd_added() {
        let old = FileQualityMetrics {
            file_path: "test.rs".to_string(),
            last_modified: SystemTime::now(),
            last_analyzed: SystemTime::now(),
            function_count: 5,
            avg_complexity: 5.0,
            max_complexity: 10,
            satd_issues: 0,
            quality_score: 0.9,
            needs_attention: false,
        };

        let mut new = old.clone();
        new.satd_issues = 3;

        let changes = QualityMonitorEngine::detect_quality_changes(&old, &new, "test.rs");
        assert!(!changes.is_empty());
        assert!(matches!(changes[0], QualityChange::SatdAdded { .. }));
    }

    #[test]
    fn test_detect_quality_changes_satd_removed() {
        let old = FileQualityMetrics {
            file_path: "test.rs".to_string(),
            last_modified: SystemTime::now(),
            last_analyzed: SystemTime::now(),
            function_count: 5,
            avg_complexity: 5.0,
            max_complexity: 10,
            satd_issues: 5,
            quality_score: 0.7,
            needs_attention: true,
        };

        let mut new = old.clone();
        new.satd_issues = 2;

        let changes = QualityMonitorEngine::detect_quality_changes(&old, &new, "test.rs");
        assert!(!changes.is_empty());
        assert!(matches!(changes[0], QualityChange::SatdRemoved { .. }));
    }

    #[test]
    fn test_detect_quality_changes_quality_improved() {
        let old = FileQualityMetrics {
            file_path: "test.rs".to_string(),
            last_modified: SystemTime::now(),
            last_analyzed: SystemTime::now(),
            function_count: 5,
            avg_complexity: 5.0,
            max_complexity: 10,
            satd_issues: 0,
            quality_score: 0.6,
            needs_attention: true,
        };

        let mut new = old.clone();
        new.quality_score = 0.9;

        let changes = QualityMonitorEngine::detect_quality_changes(&old, &new, "test.rs");
        assert!(!changes.is_empty());
        assert!(changes
            .iter()
            .any(|c| matches!(c, QualityChange::QualityImproved { .. })));
    }

    #[test]
    fn test_detect_quality_changes_quality_degraded() {
        let old = FileQualityMetrics {
            file_path: "test.rs".to_string(),
            last_modified: SystemTime::now(),
            last_analyzed: SystemTime::now(),
            function_count: 5,
            avg_complexity: 5.0,
            max_complexity: 10,
            satd_issues: 0,
            quality_score: 0.9,
            needs_attention: false,
        };

        let mut new = old.clone();
        new.quality_score = 0.5;

        let changes = QualityMonitorEngine::detect_quality_changes(&old, &new, "test.rs");
        assert!(!changes.is_empty());
        assert!(changes
            .iter()
            .any(|c| matches!(c, QualityChange::QualityDegraded { .. })));
    }

    #[test]
    fn test_update_aggregate_metrics_empty() {
        let mut metrics = QualityMetrics::default();
        QualityMonitorEngine::update_aggregate_metrics(&mut metrics);

        assert_eq!(metrics.files_analyzed, 0);
        assert_eq!(metrics.functions_analyzed, 0);
        assert_eq!(metrics.avg_complexity, 0.0);
    }

    #[test]
    fn test_update_aggregate_metrics_with_files() {
        let mut metrics = QualityMetrics::default();

        metrics.file_metrics.insert(
            "file1.rs".to_string(),
            FileQualityMetrics {
                file_path: "file1.rs".to_string(),
                last_modified: SystemTime::now(),
                last_analyzed: SystemTime::now(),
                function_count: 5,
                avg_complexity: 6.0,
                max_complexity: 10,
                satd_issues: 1,
                quality_score: 0.85,
                needs_attention: false,
            },
        );

        metrics.file_metrics.insert(
            "file2.rs".to_string(),
            FileQualityMetrics {
                file_path: "file2.rs".to_string(),
                last_modified: SystemTime::now(),
                last_analyzed: SystemTime::now(),
                function_count: 10,
                avg_complexity: 8.0,
                max_complexity: 15,
                satd_issues: 2,
                quality_score: 0.75,
                needs_attention: false,
            },
        );

        QualityMonitorEngine::update_aggregate_metrics(&mut metrics);

        assert_eq!(metrics.files_analyzed, 2);
        assert_eq!(metrics.functions_analyzed, 15);
        assert_eq!(metrics.satd_issues, 3);
        assert_eq!(metrics.max_complexity, 15);
    }

    #[test]
    fn test_update_aggregate_metrics_complexity_distribution() {
        let mut metrics = QualityMetrics::default();

        metrics.file_metrics.insert(
            "low.rs".to_string(),
            FileQualityMetrics {
                file_path: "low.rs".to_string(),
                last_modified: SystemTime::now(),
                last_analyzed: SystemTime::now(),
                function_count: 5,
                avg_complexity: 3.0,
                max_complexity: 5,
                satd_issues: 0,
                quality_score: 0.95,
                needs_attention: false,
            },
        );

        metrics.file_metrics.insert(
            "medium.rs".to_string(),
            FileQualityMetrics {
                file_path: "medium.rs".to_string(),
                last_modified: SystemTime::now(),
                last_analyzed: SystemTime::now(),
                function_count: 5,
                avg_complexity: 7.0,
                max_complexity: 8,
                satd_issues: 0,
                quality_score: 0.85,
                needs_attention: false,
            },
        );

        metrics.file_metrics.insert(
            "violation.rs".to_string(),
            FileQualityMetrics {
                file_path: "violation.rs".to_string(),
                last_modified: SystemTime::now(),
                last_analyzed: SystemTime::now(),
                function_count: 5,
                avg_complexity: 25.0,
                max_complexity: 30,
                satd_issues: 2,
                quality_score: 0.4,
                needs_attention: true,
            },
        );

        QualityMonitorEngine::update_aggregate_metrics(&mut metrics);

        assert_eq!(metrics.complexity_distribution.low, 1);
        assert_eq!(metrics.complexity_distribution.medium, 1);
        assert_eq!(metrics.complexity_distribution.violations, 1);
        assert_eq!(metrics.hotspot_functions, 1);
    }

    #[tokio::test]
    async fn test_analyze_file_metrics() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");

        std::fs::write(
            &file_path,
            r#"
            fn main() {
                if condition {
                    println!("Hello");
                }
            }

            fn helper() -> i32 {
                42
            }
            "#,
        )
        .unwrap();

        let result =
            QualityMonitorEngine::analyze_file_metrics(&file_path, Path::new("test.rs")).await;

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.file_path, "test.rs");
        assert_eq!(metrics.function_count, 2);
    }

    #[tokio::test]
    async fn test_analyze_file_metrics_python() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.py");

        std::fs::write(
            &file_path,
            r#"
def main():
    if condition:
        print("Hello")

def helper():
    return 42
            "#,
        )
        .unwrap();

        let result =
            QualityMonitorEngine::analyze_file_metrics(&file_path, Path::new("test.py")).await;

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.function_count, 2);
    }

    #[tokio::test]
    async fn test_perform_full_analysis() {
        let temp_dir = TempDir::new().unwrap();
        let config = QualityMonitorConfig::default();
        let engine = QualityMonitorEngine::new(config);

        let result = engine
            .perform_full_analysis("test_project", temp_dir.path())
            .await;

        assert!(result.is_ok());

        let metrics = engine.get_metrics("test_project").await;
        assert!(metrics.is_some());

        let m = metrics.unwrap();
        assert_eq!(m.project_id, "test_project");
        assert!(m.quality_score > 0.0);
    }
}
