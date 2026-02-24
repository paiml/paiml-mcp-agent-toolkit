#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests_part2 {
    use super::*;
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
    // analyze_incremental Tests
    // ============================================

    #[test]
    fn test_analyze_incremental_simple_code() {
        let config = MonitorConfig::default();
        let monitor = QualityMonitor::new(config).expect("Failed to create monitor");
        let change = create_test_file_change("test.rs", create_simple_rust_code());

        let result = monitor.analyze_incremental(change);
        assert!(result.is_ok());

        let metrics = result.expect("Failed to analyze");
        assert!(metrics.functions > 0);
        assert!(metrics.lines > 0);
    }

    #[test]
    fn test_analyze_incremental_complex_code() {
        let config = MonitorConfig::default();
        let monitor = QualityMonitor::new(config).expect("Failed to create monitor");
        let change = create_test_file_change("complex.rs", create_complex_rust_code());

        let result = monitor.analyze_incremental(change);
        assert!(result.is_ok());

        let metrics = result.expect("Failed to analyze");
        assert!(metrics.complexity > 1);
        assert!(metrics.functions > 0);
    }

    #[test]
    fn test_analyze_incremental_empty_function() {
        let config = MonitorConfig::default();
        let monitor = QualityMonitor::new(config).expect("Failed to create monitor");
        let change = create_test_file_change("empty.rs", "fn empty() {}");

        let result = monitor.analyze_incremental(change);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_incremental_with_satd() {
        let config = MonitorConfig::default();
        let monitor = QualityMonitor::new(config).expect("Failed to create monitor");
        let code = r#"
            fn test() {
                // TODO: implement this
                // FIXME: fix the bug
            }
        "#;
        let change = create_test_file_change("satd.rs", code);

        let result = monitor.analyze_incremental(change);
        assert!(result.is_ok());

        let metrics = result.expect("Failed to analyze");
        assert!(metrics.satd_count >= 2);
    }

    // ============================================
    // get_metrics and get_all_metrics Tests
    // ============================================

    #[test]
    fn test_get_metrics_nonexistent_path() {
        let monitor =
            QualityMonitor::new(MonitorConfig::default()).expect("Failed to create monitor");
        let result = monitor.get_metrics(Path::new("nonexistent.rs"));

        assert!(result.is_none());
    }

    #[test]
    fn test_get_all_metrics_empty() {
        let monitor =
            QualityMonitor::new(MonitorConfig::default()).expect("Failed to create monitor");
        let all_metrics = monitor.get_all_metrics();

        assert!(all_metrics.is_empty());
    }

    // ============================================
    // subscribe Tests
    // ============================================

    #[test]
    fn test_subscribe_returns_receiver() {
        let monitor =
            QualityMonitor::new(MonitorConfig::default()).expect("Failed to create monitor");
        let receiver = monitor.subscribe();

        // Receiver should be empty since no events have been sent
        assert!(receiver.try_recv().is_err());
    }

    // ============================================
    // DashMap Re-export Tests
    // ============================================

    #[test]
    fn test_dashmap_reexport() {
        let map: DashMap<String, i32> = DashMap::new();
        map.insert("key".to_string(), 42);

        assert_eq!(map.len(), 1);
        assert_eq!(*map.get("key").expect("Key should exist"), 42);
    }

    // ============================================
    // Edge Case Tests
    // ============================================

    #[test]
    fn test_analyze_incremental_invalid_rust_code() {
        let config = MonitorConfig::default();
        let monitor = QualityMonitor::new(config).expect("Failed to create monitor");
        let change = create_test_file_change("invalid.rs", "this is not valid rust code { {");

        let result = monitor.analyze_incremental(change);
        // Invalid code should return an error
        assert!(result.is_err());
    }

    #[test]
    fn test_analyze_incremental_empty_content() {
        let config = MonitorConfig::default();
        let monitor = QualityMonitor::new(config).expect("Failed to create monitor");
        let change = create_test_file_change("empty.rs", "");

        let result = monitor.analyze_incremental(change);
        // Empty file should be parseable (no functions, no complexity)
        assert!(result.is_ok());
    }

    #[test]
    fn test_should_analyze_path_with_spaces() {
        let patterns = vec!["**/*.rs".to_string()];
        assert!(QualityMonitor::should_analyze(
            Path::new("path with spaces/file.rs"),
            &patterns
        ));
    }

    #[test]
    fn test_should_analyze_hidden_file() {
        let patterns = vec!["**/*.rs".to_string()];
        assert!(QualityMonitor::should_analyze(
            Path::new(".hidden_file.rs"),
            &patterns
        ));
    }

    #[test]
    fn test_config_zero_batch_size() {
        let config = MonitorConfig {
            max_batch_size: 0,
            ..MonitorConfig::default()
        };
        // Should still be able to create monitor
        let result = QualityMonitor::new(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_zero_complexity_threshold() {
        let config = MonitorConfig {
            complexity_threshold: 0,
            ..MonitorConfig::default()
        };
        let result = QualityMonitor::new(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_file_change_timestamp_is_recent() {
        let before = SystemTime::now();
        let change = create_test_file_change("test.rs", "fn test() {}");
        let after = SystemTime::now();

        assert!(change.timestamp >= before);
        assert!(change.timestamp <= after);
    }

    // ============================================
    // Concurrency Tests
    // ============================================

    #[test]
    fn test_dashmap_concurrent_insert() {
        use std::sync::Arc;
        use std::thread;

        let map: Arc<DashMap<i32, i32>> = Arc::new(DashMap::new());
        let mut handles = vec![];

        for i in 0..10 {
            let map_clone = Arc::clone(&map);
            handles.push(thread::spawn(move || {
                map_clone.insert(i, i * 2);
            }));
        }

        for handle in handles {
            handle.join().expect("Thread panicked");
        }

        assert_eq!(map.len(), 10);
    }

    // ============================================
    // Property-Based Coverage Tests
    // ============================================

    #[test]
    fn test_monitor_config_serialization() {
        let config = create_test_config();
        let json = serde_json::to_string(&config).expect("Serialization failed");

        assert!(json.contains("complexity_threshold"));
        assert!(json.contains("max_batch_size"));

        let deserialized: MonitorConfig =
            serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(
            config.complexity_threshold,
            deserialized.complexity_threshold
        );
    }

    #[test]
    fn test_multiple_file_analyses() {
        let monitor =
            QualityMonitor::new(MonitorConfig::default()).expect("Failed to create monitor");

        let files = vec![
            ("file1.rs", "fn a() {}"),
            ("file2.rs", "fn b() { if true {} }"),
            ("file3.rs", "fn c() { for i in 0..10 {} }"),
        ];

        for (name, content) in files {
            let change = create_test_file_change(name, content);
            let result = monitor.analyze_incremental(change);
            assert!(result.is_ok(), "Failed to analyze {}", name);
        }
    }

    #[test]
    fn test_analyze_code_with_loops() {
        let monitor =
            QualityMonitor::new(MonitorConfig::default()).expect("Failed to create monitor");
        let code = r#"
            fn loops() {
                for i in 0..10 {
                    while true {
                        loop {
                            break;
                        }
                    }
                }
            }
        "#;
        let change = create_test_file_change("loops.rs", code);
        let result = monitor.analyze_incremental(change);
        assert!(result.is_ok());

        let metrics = result.expect("Failed to analyze");
        assert!(metrics.complexity > 3); // Multiple loop constructs
    }

    #[test]
    fn test_analyze_code_with_match() {
        let monitor =
            QualityMonitor::new(MonitorConfig::default()).expect("Failed to create monitor");
        let code = r#"
            fn matcher(x: i32) -> i32 {
                match x {
                    0 => 0,
                    1 => 1,
                    2 => 2,
                    _ => 3,
                }
            }
        "#;
        let change = create_test_file_change("match.rs", code);
        let result = monitor.analyze_incremental(change);
        assert!(result.is_ok());

        let metrics = result.expect("Failed to analyze");
        assert!(metrics.complexity >= 4); // Match arms add complexity
    }
}
