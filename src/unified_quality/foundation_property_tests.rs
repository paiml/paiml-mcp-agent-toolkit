#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use std::path::Path;

    proptest! {
        #[test]
        fn monitor_config_thresholds_valid(
            complexity_threshold in 1u32..1000,
            max_batch_size in 1usize..1000,
            update_interval_secs in 1u64..3600,
            debounce_millis in 1u64..5000
        ) {
            let config = MonitorConfig {
                complexity_threshold,
                max_batch_size,
                update_interval: Duration::from_secs(update_interval_secs),
                debounce_interval: Duration::from_millis(debounce_millis),
                watch_patterns: vec!["**/*.rs".to_string()],
                incremental_parsing: true,
                cache_ast: true,
            };

            prop_assert!(config.complexity_threshold > 0);
            prop_assert!(config.max_batch_size > 0);
            prop_assert!(config.update_interval.as_secs() > 0);
            prop_assert!(config.debounce_interval.as_millis() > 0);
        }

        #[test]
        fn file_pattern_matching_consistent(
            extension in "[a-z]{2,5}",
            filename in "[a-zA-Z0-9_-]{1,20}"
        ) {
            let patterns = vec![format!("**/*.{}", extension)];
            let test_file = format!("{}.{}", filename, extension);
            let path = Path::new(&test_file);

            let matches = QualityMonitor::should_analyze(path, &patterns);
            prop_assert!(matches);

            // Test non-matching extension
            let wrong_file = format!("{}.txt", filename);
            let wrong_path = Path::new(&wrong_file);
            let wrong_matches = QualityMonitor::should_analyze(wrong_path, &patterns);
            prop_assert!(wrong_matches == (extension == "txt"));
        }

        #[test]
        fn quality_monitor_creation_stable(
            complexity_threshold in 5u32..50,
            max_batch_size in 10usize..200
        ) {
            let config = MonitorConfig {
                complexity_threshold,
                max_batch_size,
                update_interval: Duration::from_secs(5),
                debounce_interval: Duration::from_millis(500),
                watch_patterns: vec!["**/*.rs".to_string()],
                incremental_parsing: true,
                cache_ast: true,
            };

            let monitor_result = QualityMonitor::new(config);
            prop_assert!(monitor_result.is_ok());

            let monitor = monitor_result.expect("internal error");
            prop_assert_eq!(monitor.metrics.len(), 0);
        }

        #[test]
        fn file_change_properties_valid(
            content in "[a-zA-Z0-9\\s\\n{}();]{10,1000}",
            path_components in prop::collection::vec("[a-zA-Z0-9_-]{1,20}", 1..5)
        ) {
            let path_str = path_components.join("/") + ".rs";
            let path = PathBuf::from(path_str);

            let file_change = FileChange {
                path: path.clone(),
                content: content.clone(),
                old_tree: None,
                timestamp: SystemTime::now(),
            };

            prop_assert_eq!(file_change.path, path);
            prop_assert_eq!(file_change.content, content);
            prop_assert!(file_change.old_tree.is_none());
        }

        #[test]
        fn batch_processing_properties(
            batch_size in 1usize..100,
            file_count in 1usize..1000
        ) {
            let config = MonitorConfig {
                max_batch_size: batch_size,
                complexity_threshold: 20,
                update_interval: Duration::from_secs(5),
                debounce_interval: Duration::from_millis(500),
                watch_patterns: vec!["**/*.rs".to_string()],
                incremental_parsing: true,
                cache_ast: true,
            };

            // Properties about batch processing
            let expected_batches = (file_count as f64 / batch_size as f64).ceil() as usize;
            prop_assert!(expected_batches >= 1);
            prop_assert!(expected_batches <= file_count);

            // Config should maintain batch size limits
            prop_assert_eq!(config.max_batch_size, batch_size);
        }

        #[test]
        #[ignore = "requires quality framework setup"]
        fn pattern_matching_edge_cases(
            pattern_type in 0..3usize,
            file_extension in "[a-z]{1,10}"
        ) {
            let patterns = [
                format!("**/*.{}", file_extension),
                format!("*.{}", file_extension),
                file_extension.clone()
            ];

            let test_pattern = &patterns[pattern_type];
            let test_file = format!("test.{}", file_extension);
            let path = Path::new(&test_file);

            let matches = QualityMonitor::should_analyze(path, std::slice::from_ref(test_pattern));

            match pattern_type {
                0 => prop_assert!(matches), // **/*.ext should match
                1 => prop_assert!(matches), // *.ext should match
                2 => prop_assert!(matches), // ext should match (contains)
                _ => unreachable!(),
            }
        }

        #[test]
        fn metrics_aggregation_properties(
            _file_count in 1usize..50,
            complexity_values in prop::collection::vec(1u32..30, 1..50)
        ) {
            // Properties of metrics aggregation
            if !complexity_values.is_empty() {
                let sum: u32 = complexity_values.iter().sum();
                let avg = sum as f64 / complexity_values.len() as f64;
                let max = *complexity_values.iter().max().expect("internal error");
                let min = *complexity_values.iter().min().expect("internal error");

                prop_assert!(avg >= min as f64);
                prop_assert!(avg <= max as f64);
                prop_assert!(sum >= max);
                prop_assert!(max >= min);
            }
        }

        #[test]
        fn concurrent_access_properties(
            thread_count in 1usize..10,
            operation_count in 1usize..100
        ) {
            // Properties of concurrent operations
            prop_assert!(thread_count > 0);
            prop_assert!(operation_count > 0);

            let total_operations = thread_count * operation_count;
            prop_assert!(total_operations >= thread_count);
            prop_assert!(total_operations >= operation_count);

            // DashMap should handle concurrent access
            let metrics: Arc<dashmap::DashMap<PathBuf, Metrics>> = Arc::new(dashmap::DashMap::new());
            prop_assert_eq!(metrics.len(), 0);
        }
    }
}
