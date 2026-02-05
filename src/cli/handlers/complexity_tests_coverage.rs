#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;

    // ComplexityConfig Tests

    #[test]
    fn test_complexity_config_from_args_with_defaults() {
        let config = ComplexityConfig::from_args(
            PathBuf::from("/test/project"),
            None,
            None,
            None,
            vec![],
            60,
            10,
        );

        assert_eq!(config.project_path, PathBuf::from("/test/project"));
        assert!(config.toolchain.is_none());
        assert_eq!(config.max_cyclomatic, 10); // default
        assert_eq!(config.max_cognitive, 15); // default
        assert!(config.include.is_empty());
        assert_eq!(config.timeout, 60);
        assert_eq!(config.top_files, 10);
    }

    #[test]
    fn test_complexity_config_from_args_with_custom_values() {
        let config = ComplexityConfig::from_args(
            PathBuf::from("/custom/path"),
            Some("rust".to_string()),
            Some(25),
            Some(30),
            vec!["src/**/*.rs".to_string()],
            120,
            5,
        );

        assert_eq!(config.project_path, PathBuf::from("/custom/path"));
        assert_eq!(config.toolchain, Some("rust".to_string()));
        assert_eq!(config.max_cyclomatic, 25);
        assert_eq!(config.max_cognitive, 30);
        assert_eq!(config.include.len(), 1);
        assert_eq!(config.timeout, 120);
        assert_eq!(config.top_files, 5);
    }

    #[test]
    fn test_complexity_config_clone() {
        let original = ComplexityConfig::from_args(
            PathBuf::from("/test"),
            Some("python".to_string()),
            Some(15),
            Some(20),
            vec!["*.py".to_string()],
            90,
            15,
        );

        let cloned = original.clone();

        assert_eq!(cloned.project_path, original.project_path);
        assert_eq!(cloned.toolchain, original.toolchain);
        assert_eq!(cloned.max_cyclomatic, original.max_cyclomatic);
        assert_eq!(cloned.max_cognitive, original.max_cognitive);
        assert_eq!(cloned.include, original.include);
        assert_eq!(cloned.timeout, original.timeout);
        assert_eq!(cloned.top_files, original.top_files);
    }

    // apply_complexity_filters Tests

    #[test]
    fn test_apply_complexity_filters_no_filters() {
        let mut metrics = vec![create_test_file_metrics("test.rs", 5, 10)];

        let filtered = apply_complexity_filters(&mut metrics, None, None);

        assert_eq!(filtered, 0);
        assert_eq!(metrics.len(), 1);
    }

    #[test]
    fn test_apply_complexity_filters_cyclomatic_only() {
        let mut metrics = vec![
            create_test_file_metrics("low.rs", 5, 10),
            create_test_file_metrics("high.rs", 25, 10),
        ];

        let filtered = apply_complexity_filters(&mut metrics, Some(20), None);

        assert_eq!(filtered, 1);
        assert_eq!(metrics.len(), 1);
        assert!(metrics[0].path.contains("high"));
    }

    #[test]
    fn test_apply_complexity_filters_cognitive_only() {
        let mut metrics = vec![
            create_test_file_metrics("low.rs", 5, 10),
            create_test_file_metrics("high.rs", 5, 30),
        ];

        let filtered = apply_complexity_filters(&mut metrics, None, Some(25));

        assert_eq!(filtered, 1);
        assert_eq!(metrics.len(), 1);
        assert!(metrics[0].path.contains("high"));
    }

    #[test]
    fn test_apply_complexity_filters_both_thresholds() {
        let mut metrics = vec![
            create_test_file_metrics("low.rs", 5, 10),
            create_test_file_metrics("high_cyc.rs", 25, 10),
            create_test_file_metrics("high_cog.rs", 5, 30),
            create_test_file_metrics("both_high.rs", 25, 30),
        ];

        let filtered = apply_complexity_filters(&mut metrics, Some(20), Some(25));

        assert_eq!(filtered, 1); // Only "low.rs" is filtered out
        assert_eq!(metrics.len(), 3);
    }

    #[test]
    fn test_apply_complexity_filters_all_filtered() {
        let mut metrics = vec![
            create_test_file_metrics("a.rs", 5, 10),
            create_test_file_metrics("b.rs", 8, 12),
        ];

        let filtered = apply_complexity_filters(&mut metrics, Some(50), Some(50));

        assert_eq!(filtered, 2);
        assert!(metrics.is_empty());
    }

    // apply_top_files_limit Tests

    #[test]
    fn test_apply_top_files_limit_zero_no_limit() {
        let mut metrics = vec![
            create_test_file_metrics("a.rs", 5, 10),
            create_test_file_metrics("b.rs", 10, 20),
            create_test_file_metrics("c.rs", 15, 30),
        ];

        apply_top_files_limit(&mut metrics, 0);

        assert_eq!(metrics.len(), 3);
    }

    #[test]
    fn test_apply_top_files_limit_truncates() {
        let mut metrics = vec![
            create_test_file_metrics("a.rs", 5, 10),
            create_test_file_metrics("b.rs", 10, 20),
            create_test_file_metrics("c.rs", 15, 30),
        ];

        apply_top_files_limit(&mut metrics, 2);

        assert_eq!(metrics.len(), 2);
        // Highest complexity should be first
        assert!(metrics[0].path.contains("c.rs"));
    }

    #[test]
    fn test_apply_top_files_limit_sorts_by_complexity() {
        let mut metrics = vec![
            create_test_file_metrics("low.rs", 5, 5),
            create_test_file_metrics("high.rs", 50, 50),
            create_test_file_metrics("medium.rs", 25, 25),
        ];

        apply_top_files_limit(&mut metrics, 3);

        assert!(metrics[0].path.contains("high"));
        assert!(metrics[1].path.contains("medium"));
        assert!(metrics[2].path.contains("low"));
    }

    #[test]
    fn test_apply_top_files_limit_empty_metrics() {
        let mut metrics: Vec<crate::services::complexity::FileComplexityMetrics> = vec![];

        apply_top_files_limit(&mut metrics, 5);

        assert!(metrics.is_empty());
    }

    // File Analysis Path Tests

    #[test]
    fn test_is_source_code_file_rust() {
        assert!(is_source_code_file("src/main.rs"));
        assert!(is_source_code_file("/path/to/lib.rs"));
    }

    #[test]
    fn test_is_source_code_file_typescript() {
        assert!(is_source_code_file("src/app.ts"));
        assert!(is_source_code_file("components/Button.tsx"));
    }

    #[test]
    fn test_is_source_code_file_javascript() {
        assert!(is_source_code_file("src/index.js"));
        assert!(is_source_code_file("components/App.jsx"));
    }

    #[test]
    fn test_is_source_code_file_python() {
        assert!(is_source_code_file("app.py"));
        assert!(is_source_code_file("/path/to/module.py"));
    }

    #[test]
    fn test_is_source_code_file_cpp() {
        assert!(is_source_code_file("main.cpp"));
        assert!(is_source_code_file("header.h"));
        assert!(is_source_code_file("header.hpp"));
    }

    #[test]
    fn test_is_source_code_file_non_source() {
        assert!(!is_source_code_file("README.md"));
        assert!(!is_source_code_file("Cargo.toml"));
        assert!(!is_source_code_file("package.json"));
        assert!(!is_source_code_file(".gitignore"));
    }

    #[test]
    fn test_should_include_file_empty_patterns() {
        assert!(should_include_file("src/main.rs", &[]));
        assert!(should_include_file("any/path/file.py", &[]));
    }

    #[test]
    fn test_should_include_file_matching_pattern() {
        let patterns = vec!["src/".to_string()];
        assert!(should_include_file("src/main.rs", &patterns));
        assert!(should_include_file("src/lib.rs", &patterns));
        assert!(!should_include_file("tests/test.rs", &patterns));
    }

    #[test]
    fn test_should_include_file_multiple_patterns() {
        let patterns = vec!["src/".to_string(), "tests/".to_string()];
        assert!(should_include_file("src/main.rs", &patterns));
        assert!(should_include_file("tests/test.rs", &patterns));
        assert!(!should_include_file("examples/demo.rs", &patterns));
    }

    // has_complexity_violations Tests

    #[test]
    fn test_has_complexity_violations_no_violations() {
        let metrics = vec![create_test_file_metrics("test.rs", 10, 10)];

        let has_violations = has_complexity_violations(&metrics, Some(20), Some(15));

        assert!(!has_violations);
    }

    #[test]
    fn test_has_complexity_violations_cyclomatic_exceeded() {
        let metrics = vec![create_test_file_metrics("test.rs", 25, 10)];

        let has_violations = has_complexity_violations(&metrics, Some(20), Some(15));

        assert!(has_violations);
    }

    #[test]
    fn test_has_complexity_violations_cognitive_exceeded() {
        let metrics = vec![create_test_file_metrics("test.rs", 10, 20)];

        let has_violations = has_complexity_violations(&metrics, Some(20), Some(15));

        assert!(has_violations);
    }

    #[test]
    fn test_has_complexity_violations_uses_defaults() {
        // Default thresholds are 20 for cyclomatic, 15 for cognitive
        let metrics = vec![create_test_file_metrics("test.rs", 21, 10)];

        let has_violations = has_complexity_violations(&metrics, None, None);

        assert!(has_violations);
    }

    // Dead Code Formatting Tests

    #[test]
    fn test_format_dead_code_summary_section() {
        let result = create_test_dead_code_result(5, 2, 45, 15.5);

        let section = format_dead_code_summary_section(&result);

        assert!(section.contains("Dead Code Analysis Report"));
        assert!(section.contains("Files Analyzed | 5"));
        assert!(section.contains("Files with Dead Code | 2"));
        assert!(section.contains("Total Dead Lines | 45"));
        assert!(section.contains("15.50%"));
    }

    #[test]
    fn test_format_dead_code_breakdown_section() {
        let summary = crate::models::dead_code::DeadCodeSummary {
            total_files_analyzed: 10,
            files_with_dead_code: 3,
            total_dead_lines: 100,
            dead_percentage: 10.0,
            dead_functions: 5,
            dead_classes: 2,
            dead_modules: 1,
            unreachable_blocks: 3,
        };

        let section = format_dead_code_breakdown_section(&summary);

        assert!(section.contains("Dead Code Breakdown"));
        assert!(section.contains("Functions | 5"));
        assert!(section.contains("Classes | 2"));
        assert!(section.contains("Variables | 1")); // dead_modules shows as Variables
        assert!(section.contains("Unreachable Blocks | 3"));
    }

    #[test]
    fn test_format_dead_code_recommendations_section() {
        let section = format_dead_code_recommendations_section();

        assert!(section.contains("Recommendations"));
        assert!(section.contains("Review High Confidence Dead Code"));
        assert!(section.contains("Check Test Coverage"));
        assert!(section.contains("Consider Refactoring"));
        assert!(section.contains("Remove Carefully"));
    }

    // SATD Formatting Tests

    #[test]
    fn test_format_satd_summary_basic() {
        let result = create_test_satd_result(10, 3, 15);

        let summary = format_satd_summary(&result, false);

        assert!(summary.contains("SATD Analysis Summary"));
        assert!(summary.contains("Files analyzed**: 10"));
        assert!(summary.contains("Files with SATD**: 3"));
        assert!(summary.contains("Total SATD items**: 15"));
    }

    #[test]
    fn test_format_satd_summary_with_metrics() {
        let mut result = create_test_satd_result(10, 3, 5);
        result.summary.by_severity.insert("High".to_string(), 2);
        result.summary.by_severity.insert("Medium".to_string(), 3);
        result.summary.by_category.insert("TODO".to_string(), 3);
        result.summary.by_category.insert("FIXME".to_string(), 2);

        let summary = format_satd_summary(&result, true);

        assert!(summary.contains("By Severity"));
        assert!(summary.contains("High**: 2"));
        assert!(summary.contains("Medium**: 3"));
        assert!(summary.contains("By Category"));
        assert!(summary.contains("TODO**: 3"));
        assert!(summary.contains("FIXME**: 2"));
    }

    // SyncAnalysisConfig Tests

    #[test]
    fn test_sync_analysis_config_creation() {
        let path = PathBuf::from("/test");
        let include = vec!["src/".to_string()];
        let output_path = PathBuf::from("/output.json");

        let config = create_sync_config(
            &path,
            Some("rust"),
            Some(20),
            Some(15),
            &include,
            60,
            10,
            ComplexityOutputFormat::Json,
            Some(&output_path),
        );

        assert_eq!(config.path, path.as_path());
        assert_eq!(config.toolchain, Some("rust"));
        assert_eq!(config.max_cyclomatic, Some(20));
        assert_eq!(config.max_cognitive, Some(15));
        assert_eq!(config.timeout, 60);
        assert_eq!(config.top_files, 10);
    }

    #[test]
    fn test_sync_analysis_config_clone() {
        let path = PathBuf::from("/test");
        let include: Vec<String> = vec![];

        let config = create_sync_config(
            &path,
            None,
            None,
            None,
            &include,
            120,
            5,
            ComplexityOutputFormat::Summary,
            None,
        );

        let cloned = config.clone();

        assert_eq!(cloned.path, config.path);
        assert_eq!(cloned.toolchain, config.toolchain);
        assert_eq!(cloned.timeout, config.timeout);
        assert_eq!(cloned.top_files, config.top_files);
    }

    // DeadCodeAnalysisFilters Tests

    #[test]
    fn test_dead_code_analysis_filters_defaults() {
        let filters = DeadCodeAnalysisFilters {
            include_unreachable: false,
            include_tests: false,
            min_dead_lines: 0,
            top_files: None,
            include: vec![],
            exclude: vec![],
            max_depth: 10,
        };

        assert!(!filters.include_unreachable);
        assert!(!filters.include_tests);
        assert_eq!(filters.min_dead_lines, 0);
        assert!(filters.top_files.is_none());
        assert!(filters.include.is_empty());
        assert!(filters.exclude.is_empty());
        assert_eq!(filters.max_depth, 10);
    }

    #[test]
    fn test_dead_code_analysis_filters_custom() {
        let filters = DeadCodeAnalysisFilters {
            include_unreachable: true,
            include_tests: true,
            min_dead_lines: 5,
            top_files: Some(20),
            include: vec!["src/".to_string()],
            exclude: vec!["vendor/".to_string()],
            max_depth: 5,
        };

        assert!(filters.include_unreachable);
        assert!(filters.include_tests);
        assert_eq!(filters.min_dead_lines, 5);
        assert_eq!(filters.top_files, Some(20));
        assert_eq!(filters.include.len(), 1);
        assert_eq!(filters.exclude.len(), 1);
        assert_eq!(filters.max_depth, 5);
    }

    // get_changed_paths Tests

    #[test]
    fn test_get_changed_paths_empty() {
        let event = notify::Event {
            kind: notify::EventKind::Modify(notify::event::ModifyKind::Any),
            paths: vec![],
            attrs: Default::default(),
        };

        assert!(get_changed_paths(&event).is_none());
    }

    #[test]
    fn test_get_changed_paths_with_paths() {
        let event = notify::Event {
            kind: notify::EventKind::Modify(notify::event::ModifyKind::Any),
            paths: vec![PathBuf::from("/test/file.rs")],
            attrs: Default::default(),
        };

        let paths = get_changed_paths(&event);
        assert!(paths.is_some());
        assert_eq!(paths.unwrap().len(), 1);
    }

    // should_reanalyze Tests

    #[test]
    fn test_should_reanalyze_create_event() {
        let event = notify::Event {
            kind: notify::EventKind::Create(notify::event::CreateKind::File),
            paths: vec![PathBuf::from("/test/file.rs")],
            attrs: Default::default(),
        };

        assert!(should_reanalyze(&event, &[]));
    }

    #[test]
    fn test_should_reanalyze_modify_event() {
        let event = notify::Event {
            kind: notify::EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content,
            )),
            paths: vec![PathBuf::from("/test/file.rs")],
            attrs: Default::default(),
        };

        assert!(should_reanalyze(&event, &[]));
    }

    #[test]
    fn test_should_reanalyze_remove_event() {
        let event = notify::Event {
            kind: notify::EventKind::Remove(notify::event::RemoveKind::File),
            paths: vec![PathBuf::from("/test/file.rs")],
            attrs: Default::default(),
        };

        assert!(should_reanalyze(&event, &[]));
    }

    #[test]
    fn test_should_reanalyze_access_event_ignored() {
        let event = notify::Event {
            kind: notify::EventKind::Access(notify::event::AccessKind::Read),
            paths: vec![PathBuf::from("/test/file.rs")],
            attrs: Default::default(),
        };

        assert!(!should_reanalyze(&event, &[]));
    }

    #[test]
    fn test_should_reanalyze_non_source_file_ignored() {
        let event = notify::Event {
            kind: notify::EventKind::Modify(notify::event::ModifyKind::Any),
            paths: vec![PathBuf::from("/test/README.md")],
            attrs: Default::default(),
        };

        assert!(!should_reanalyze(&event, &[]));
    }

    // Helper Functions for Tests

    fn create_test_file_metrics(
        path: &str,
        cyclomatic: u16,
        cognitive: u16,
    ) -> crate::services::complexity::FileComplexityMetrics {
        crate::services::complexity::FileComplexityMetrics {
            path: path.to_string(),
            language: "rust".to_string(),
            total_complexity: crate::services::complexity::ComplexityMetrics {
                cyclomatic,
                cognitive,
                nesting_depth: 2,
                line_count: 100,
                function_count: 5,
            },
            functions: vec![crate::services::complexity::FunctionComplexity {
                name: "test_function".to_string(),
                line: 1,
                metrics: crate::services::complexity::ComplexityMetrics {
                    cyclomatic,
                    cognitive,
                    nesting_depth: 2,
                    line_count: 20,
                    function_count: 1,
                },
            }],
            function_count: 1,
        }
    }

    fn create_test_dead_code_result(
        total_files: usize,
        files_with_dead: usize,
        dead_lines: usize,
        dead_percentage: f32,
    ) -> crate::models::dead_code::DeadCodeResult {
        crate::models::dead_code::DeadCodeResult {
            summary: crate::models::dead_code::DeadCodeSummary {
                total_files_analyzed: total_files,
                files_with_dead_code: files_with_dead,
                total_dead_lines: dead_lines,
                dead_percentage,
                dead_functions: 3,
                dead_classes: 1,
                dead_modules: 0,
                unreachable_blocks: 0,
            },
            files: vec![],
            total_files,
            analyzed_files: total_files,
        }
    }

    fn create_test_satd_result(
        total_files: usize,
        files_with_debt: usize,
        items_count: usize,
    ) -> crate::services::satd_detector::SATDAnalysisResult {
        use chrono::Utc;
        use std::collections::HashMap;

        crate::services::satd_detector::SATDAnalysisResult {
            items: (0..items_count)
                .map(|i| crate::services::satd_detector::TechnicalDebt {
                    category: crate::services::satd_detector::DebtCategory::Requirement,
                    severity: crate::services::satd_detector::Severity::Medium,
                    text: format!("TODO item {}", i),
                    file: PathBuf::from(format!("file{}.rs", i % files_with_debt)),
                    line: (i + 1) as u32,
                    column: 1,
                    context_hash: [0u8; 16],
                })
                .collect(),
            summary: crate::services::satd_detector::SATDSummary {
                total_items: items_count,
                by_severity: HashMap::new(),
                by_category: HashMap::new(),
                files_with_satd: files_with_debt,
                avg_age_days: 30.0,
            },
            total_files_analyzed: total_files,
            files_with_debt,
            analysis_timestamp: Utc::now(),
        }
    }
}

