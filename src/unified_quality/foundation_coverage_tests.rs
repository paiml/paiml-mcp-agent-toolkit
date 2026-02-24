#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use std::path::Path;
    use std::time::Duration;

    // ============================================
    // Test Fixtures and Helpers
    // ============================================

    /// Helper to create a default MonitorConfig for testing
    fn create_test_config() -> MonitorConfig {
        MonitorConfig {
            update_interval: Duration::from_secs(1),
            complexity_threshold: 15,
            watch_patterns: vec!["**/*.rs".to_string(), "**/*.py".to_string()],
            debounce_interval: Duration::from_millis(100),
            max_batch_size: 10,
            incremental_parsing: true,
            cache_ast: true,
        }
    }

    /// Helper to create a FileChange for testing
    fn create_test_file_change(path: &str, content: &str) -> FileChange {
        FileChange {
            path: PathBuf::from(path),
            content: content.to_string(),
            old_tree: None,
            timestamp: SystemTime::now(),
        }
    }

    /// Helper to create valid Rust code for testing
    fn create_simple_rust_code() -> &'static str {
        r#"fn main() { println!("hello"); }"#
    }

    /// Helper to create complex Rust code for testing
    fn create_complex_rust_code() -> &'static str {
        r#"
        fn complex_fn(x: i32) -> i32 {
            if x > 0 {
                for i in 0..x {
                    if i % 2 == 0 {
                        println!("{}", i);
                    }
                }
            }
            x
        }
        "#
    }

    // ============================================
    // MonitorConfig Tests
    // ============================================

    #[test]
    fn test_monitor_config_default_values() {
        let config = MonitorConfig::default();

        assert_eq!(config.update_interval, Duration::from_secs(5));
        assert_eq!(config.complexity_threshold, 20);
        assert_eq!(config.debounce_interval, Duration::from_millis(500));
        assert_eq!(config.max_batch_size, 50);
        assert!(config.incremental_parsing);
        assert!(config.cache_ast);
        assert_eq!(config.watch_patterns.len(), 4);
    }

    #[test]
    fn test_monitor_config_custom_values() {
        let config = MonitorConfig {
            update_interval: Duration::from_secs(10),
            complexity_threshold: 30,
            watch_patterns: vec!["**/*.go".to_string()],
            debounce_interval: Duration::from_millis(200),
            max_batch_size: 100,
            incremental_parsing: false,
            cache_ast: false,
        };

        assert_eq!(config.update_interval, Duration::from_secs(10));
        assert_eq!(config.complexity_threshold, 30);
        assert_eq!(config.watch_patterns.len(), 1);
        assert_eq!(config.watch_patterns[0], "**/*.go");
        assert!(!config.incremental_parsing);
        assert!(!config.cache_ast);
    }

    #[test]
    fn test_monitor_config_clone() {
        let config1 = create_test_config();
        let config2 = config1.clone();

        assert_eq!(config1.complexity_threshold, config2.complexity_threshold);
        assert_eq!(config1.max_batch_size, config2.max_batch_size);
        assert_eq!(config1.watch_patterns, config2.watch_patterns);
    }

    #[test]
    fn test_monitor_config_debug() {
        let config = create_test_config();
        let debug_str = format!("{:?}", config);

        assert!(debug_str.contains("MonitorConfig"));
        assert!(debug_str.contains("complexity_threshold"));
    }

    // ============================================
    // FileChange Tests
    // ============================================

    #[test]
    fn test_file_change_creation() {
        let change = create_test_file_change("test.rs", "fn test() {}");

        assert_eq!(change.path, PathBuf::from("test.rs"));
        assert_eq!(change.content, "fn test() {}");
        assert!(change.old_tree.is_none());
    }

    #[test]
    fn test_file_change_with_old_tree() {
        let change = FileChange {
            path: PathBuf::from("test.rs"),
            content: "fn test() {}".to_string(),
            old_tree: Some("previous_tree_data".to_string()),
            timestamp: SystemTime::now(),
        };

        assert!(change.old_tree.is_some());
        assert_eq!(change.old_tree.as_ref().unwrap(), "previous_tree_data");
    }

    #[test]
    fn test_file_change_clone() {
        let change1 = create_test_file_change("test.rs", "fn test() {}");
        let change2 = change1.clone();

        assert_eq!(change1.path, change2.path);
        assert_eq!(change1.content, change2.content);
    }

    #[test]
    fn test_file_change_debug() {
        let change = create_test_file_change("test.rs", "fn test() {}");
        let debug_str = format!("{:?}", change);

        assert!(debug_str.contains("FileChange"));
        assert!(debug_str.contains("test.rs"));
    }

    // ============================================
    // QualityMonitor Creation Tests
    // ============================================

    #[test]
    fn test_quality_monitor_new_default_config() {
        let config = MonitorConfig::default();
        let result = QualityMonitor::new(config);

        assert!(result.is_ok());
        let monitor = result.expect("Failed to create monitor");
        assert_eq!(monitor.metrics.len(), 0);
    }

    #[test]
    fn test_quality_monitor_new_custom_config() {
        let config = create_test_config();
        let result = QualityMonitor::new(config);

        assert!(result.is_ok());
    }

    #[test]
    fn test_quality_monitor_empty_metrics_initially() {
        let monitor =
            QualityMonitor::new(MonitorConfig::default()).expect("Failed to create monitor");
        let all_metrics = monitor.get_all_metrics();

        assert!(all_metrics.is_empty());
    }

    // ============================================
    // should_analyze Tests
    // ============================================

    #[test]
    fn test_should_analyze_rust_file() {
        let patterns = vec!["**/*.rs".to_string()];
        assert!(QualityMonitor::should_analyze(
            Path::new("src/main.rs"),
            &patterns
        ));
    }

    #[test]
    fn test_should_analyze_python_file() {
        let patterns = vec!["**/*.py".to_string()];
        assert!(QualityMonitor::should_analyze(
            Path::new("script.py"),
            &patterns
        ));
    }

    #[test]
    fn test_should_analyze_nested_path() {
        let patterns = vec!["**/*.rs".to_string()];
        assert!(QualityMonitor::should_analyze(
            Path::new("src/module/submodule/file.rs"),
            &patterns
        ));
    }

    #[test]
    fn test_should_not_analyze_non_matching_extension() {
        let patterns = vec!["**/*.rs".to_string()];
        assert!(!QualityMonitor::should_analyze(
            Path::new("file.txt"),
            &patterns
        ));
    }

    #[test]
    fn test_should_not_analyze_markdown() {
        let patterns = vec!["**/*.rs".to_string(), "**/*.py".to_string()];
        assert!(!QualityMonitor::should_analyze(
            Path::new("README.md"),
            &patterns
        ));
    }

    #[test]
    fn test_should_analyze_multiple_patterns() {
        let patterns = vec![
            "**/*.rs".to_string(),
            "**/*.py".to_string(),
            "**/*.js".to_string(),
        ];
        assert!(QualityMonitor::should_analyze(
            Path::new("script.js"),
            &patterns
        ));
    }

    #[test]
    fn test_should_analyze_empty_patterns() {
        let patterns: Vec<String> = vec![];
        assert!(!QualityMonitor::should_analyze(
            Path::new("test.rs"),
            &patterns
        ));
    }

    #[test]
    fn test_should_analyze_contains_pattern() {
        let patterns = vec!["test".to_string()];
        assert!(QualityMonitor::should_analyze(
            Path::new("my_test_file.rs"),
            &patterns
        ));
    }

    #[test]
    fn test_should_analyze_typescript() {
        let patterns = vec!["**/*.ts".to_string()];
        assert!(QualityMonitor::should_analyze(
            Path::new("component.ts"),
            &patterns
        ));
    }

    #[test]
    fn test_should_analyze_javascript() {
        let patterns = vec!["**/*.js".to_string()];
        assert!(QualityMonitor::should_analyze(
            Path::new("index.js"),
            &patterns
        ));
    }
}
