//! Comprehensive coverage tests for complexity_handlers.rs
//!
//! This module provides extensive test coverage for the complexity handler functions,
//! focusing on pure helper functions, data structures, and error paths.

use super::*;
use proptest::prelude::*;
use std::path::PathBuf;
use tempfile::TempDir;

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

#[cfg(test)]
mod property_tests {
    use super::*;

    proptest! {
        #[test]
        fn test_complexity_config_from_args_preserves_values(
            max_cyc in 1u16..100,
            max_cog in 1u16..100,
            timeout in 1u64..1000,
            top_files in 0usize..100
        ) {
            let config = ComplexityConfig::from_args(
                PathBuf::from("/test"),
                Some("rust".to_string()),
                Some(max_cyc),
                Some(max_cog),
                vec![],
                timeout,
                top_files,
            );

            prop_assert_eq!(config.max_cyclomatic, max_cyc);
            prop_assert_eq!(config.max_cognitive, max_cog);
            prop_assert_eq!(config.timeout, timeout);
            prop_assert_eq!(config.top_files, top_files);
        }

        #[test]
        fn test_apply_top_files_limit_respects_limit(limit in 1usize..20) {
            let mut metrics: Vec<crate::services::complexity::FileComplexityMetrics> = (0..50)
                .map(|i| create_test_metrics_simple(&format!("file{}.rs", i), i as u16))
                .collect();

            apply_top_files_limit(&mut metrics, limit);

            prop_assert!(metrics.len() <= limit);
        }

        #[test]
        fn test_is_source_code_file_deterministic(ext in "[a-z]{2,4}") {
            let path = format!("file.{}", ext);
            let result1 = is_source_code_file(&path);
            let result2 = is_source_code_file(&path);

            prop_assert_eq!(result1, result2);
        }

        #[test]
        fn test_should_include_file_empty_patterns_always_includes(path in "[a-z/]+\\.rs") {
            prop_assert!(should_include_file(&path, &[]));
        }

        #[test]
        fn test_filter_count_never_exceeds_original(count in 1usize..50) {
            let mut metrics: Vec<crate::services::complexity::FileComplexityMetrics> = (0..count)
                .map(|i| create_test_metrics_simple(&format!("file{}.rs", i), (i % 30) as u16))
                .collect();

            let original_count = metrics.len();
            let filtered = apply_complexity_filters(&mut metrics, Some(15), Some(15));

            prop_assert!(filtered + metrics.len() == original_count);
        }
    }

    fn create_test_metrics_simple(
        path: &str,
        complexity: u16,
    ) -> crate::services::complexity::FileComplexityMetrics {
        crate::services::complexity::FileComplexityMetrics {
            path: path.to_string(),
            language: "rust".to_string(),
            total_complexity: crate::services::complexity::ComplexityMetrics {
                cyclomatic: complexity,
                cognitive: complexity,
                nesting_depth: 2,
                line_count: 100,
                function_count: 5,
            },
            functions: vec![crate::services::complexity::FunctionComplexity {
                name: "func".to_string(),
                line: 1,
                metrics: crate::services::complexity::ComplexityMetrics {
                    cyclomatic: complexity,
                    cognitive: complexity,
                    nesting_depth: 2,
                    line_count: 20,
                    function_count: 1,
                },
            }],
            function_count: 1,
        }
    }
}

// Extended Coverage Tests - Dead Code Formatting

#[cfg(test)]
mod dead_code_format_tests {
    use super::*;
    use crate::models::dead_code::{
        ConfidenceLevel, DeadCodeItem, DeadCodeResult, DeadCodeSummary, DeadCodeType,
        FileDeadCodeMetrics,
    };

    fn create_test_dead_code_result_with_items() -> DeadCodeResult {
        DeadCodeResult {
            summary: DeadCodeSummary {
                total_files_analyzed: 10,
                files_with_dead_code: 3,
                total_dead_lines: 150,
                dead_percentage: 15.0,
                dead_functions: 8,
                dead_classes: 2,
                dead_modules: 1,
                unreachable_blocks: 3,
            },
            files: vec![
                FileDeadCodeMetrics {
                    path: "src/module_a.rs".to_string(),
                    dead_lines: 50,
                    total_lines: 200,
                    dead_percentage: 25.0,
                    dead_functions: 3,
                    dead_classes: 1,
                    dead_modules: 0,
                    unreachable_blocks: 1,
                    dead_score: 35.5,
                    confidence: ConfidenceLevel::High,
                    items: vec![
                        DeadCodeItem {
                            name: "unused_helper".to_string(),
                            item_type: DeadCodeType::Function,
                            line: 42,
                            reason: "Never called".to_string(),
                        },
                        DeadCodeItem {
                            name: "OldClass".to_string(),
                            item_type: DeadCodeType::Class,
                            line: 100,
                            reason: "Never instantiated".to_string(),
                        },
                    ],
                },
                FileDeadCodeMetrics {
                    path: "src/module_b.rs".to_string(),
                    dead_lines: 60,
                    total_lines: 300,
                    dead_percentage: 20.0,
                    dead_functions: 4,
                    dead_classes: 0,
                    dead_modules: 1,
                    unreachable_blocks: 2,
                    dead_score: 28.0,
                    confidence: ConfidenceLevel::Medium,
                    items: vec![DeadCodeItem {
                        name: "legacy_code".to_string(),
                        item_type: DeadCodeType::UnreachableCode,
                        line: 50,
                        reason: "Unreachable after return".to_string(),
                    }],
                },
                FileDeadCodeMetrics {
                    path: "src/module_c.rs".to_string(),
                    dead_lines: 40,
                    total_lines: 250,
                    dead_percentage: 16.0,
                    dead_functions: 1,
                    dead_classes: 1,
                    dead_modules: 0,
                    unreachable_blocks: 0,
                    dead_score: 18.0,
                    confidence: ConfidenceLevel::Low,
                    items: vec![DeadCodeItem {
                        name: "dynamic_var".to_string(),
                        item_type: DeadCodeType::Variable,
                        line: 25,
                        reason: "Possibly unused".to_string(),
                    }],
                },
            ],
            total_files: 10,
            analyzed_files: 10,
        }
    }

    #[test]
    fn test_format_dead_code_as_json() {
        let result = create_test_dead_code_result_with_items();

        let json_output = format_dead_code_as_json(&result).expect("JSON formatting should work");

        // Verify it's valid JSON
        let parsed: serde_json::Value =
            serde_json::from_str(&json_output).expect("Should be valid JSON");

        // Check key fields are present
        assert!(parsed.get("summary").is_some());
        assert!(parsed.get("files").is_some());
        assert!(parsed.get("total_files").is_some());

        // Check summary values
        let summary = parsed.get("summary").unwrap();
        assert_eq!(summary.get("total_files_analyzed").unwrap(), 10);
        assert_eq!(summary.get("files_with_dead_code").unwrap(), 3);
        assert_eq!(summary.get("dead_functions").unwrap(), 8);
    }

    #[test]
    fn test_format_dead_code_as_sarif() {
        let result = create_test_dead_code_result_with_items();

        let sarif_output =
            format_dead_code_as_sarif(&result).expect("SARIF formatting should work");

        // Verify it's valid JSON
        let parsed: serde_json::Value =
            serde_json::from_str(&sarif_output).expect("Should be valid JSON");

        // Check SARIF schema version
        assert_eq!(parsed.get("version").unwrap(), "2.1.0");

        // Check tool information
        let runs = parsed.get("runs").unwrap().as_array().unwrap();
        assert!(!runs.is_empty());

        let tool = runs[0].get("tool").unwrap();
        let driver = tool.get("driver").unwrap();
        assert_eq!(driver.get("name").unwrap(), "pmat");

        // Check rules
        let rules = driver.get("rules").unwrap().as_array().unwrap();
        assert!(!rules.is_empty());
        assert_eq!(rules[0].get("id").unwrap(), "dead-code");
    }

    #[test]
    fn test_format_dead_code_as_markdown() {
        let result = create_test_dead_code_result_with_items();

        let markdown_output =
            format_dead_code_as_markdown(&result).expect("Markdown formatting should work");

        // Check sections are present
        assert!(markdown_output.contains("# Dead Code Analysis Report"));
        assert!(markdown_output.contains("## Summary"));
        assert!(markdown_output.contains("## Dead Code Breakdown"));
        assert!(markdown_output.contains("## File Details"));
        assert!(markdown_output.contains("## Recommendations"));

        // Check table content
        assert!(markdown_output.contains("Files Analyzed | 10"));
        assert!(markdown_output.contains("Files with Dead Code | 3"));
        assert!(markdown_output.contains("src/module_a.rs"));
    }

    #[test]
    fn test_format_dead_code_file_details_section() {
        let files = vec![
            FileDeadCodeMetrics {
                path: "src/test.rs".to_string(),
                dead_lines: 30,
                total_lines: 100,
                dead_percentage: 30.0,
                dead_functions: 2,
                dead_classes: 1,
                dead_modules: 0,
                unreachable_blocks: 0,
                dead_score: 25.0,
                confidence: ConfidenceLevel::High,
                items: vec![
                    DeadCodeItem {
                        name: "fn1".to_string(),
                        item_type: DeadCodeType::Function,
                        line: 10,
                        reason: "unused".to_string(),
                    },
                    DeadCodeItem {
                        name: "fn2".to_string(),
                        item_type: DeadCodeType::Function,
                        line: 20,
                        reason: "unused".to_string(),
                    },
                ],
            },
        ];

        let section = format_dead_code_file_details_section(&files);

        assert!(section.contains("## File Details"));
        assert!(section.contains("| File | Dead % | Dead Lines | Confidence | Items |"));
        assert!(section.contains("src/test.rs"));
        assert!(section.contains("30.0%"));
        assert!(section.contains("High"));
        assert!(section.contains("| 2 |")); // 2 items
    }

    #[test]
    fn test_format_dead_code_header() {
        let result = create_test_dead_code_result_with_items();
        let mut output = String::new();

        write_dead_code_header(&mut output, &result).expect("Header writing should work");

        assert!(output.contains("# Dead Code Analysis Summary"));
        assert!(output.contains("**Files analyzed**: 10"));
        assert!(output.contains("**Files with dead code**: 3"));
        assert!(output.contains("**Total dead lines**: 150"));
        assert!(output.contains("**Dead code percentage**: 15.00%"));
    }

    #[test]
    fn test_format_dead_code_sarif_levels() {
        // Test that confidence levels map to correct SARIF levels
        let result = DeadCodeResult {
            summary: DeadCodeSummary {
                total_files_analyzed: 3,
                files_with_dead_code: 3,
                total_dead_lines: 30,
                dead_percentage: 10.0,
                dead_functions: 3,
                dead_classes: 0,
                dead_modules: 0,
                unreachable_blocks: 0,
            },
            files: vec![
                FileDeadCodeMetrics {
                    path: "high.rs".to_string(),
                    dead_lines: 10,
                    total_lines: 100,
                    dead_percentage: 10.0,
                    dead_functions: 1,
                    dead_classes: 0,
                    dead_modules: 0,
                    unreachable_blocks: 0,
                    dead_score: 10.0,
                    confidence: ConfidenceLevel::High,
                    items: vec![DeadCodeItem {
                        name: "fn_high".to_string(),
                        item_type: DeadCodeType::Function,
                        line: 5,
                        reason: "unused".to_string(),
                    }],
                },
                FileDeadCodeMetrics {
                    path: "medium.rs".to_string(),
                    dead_lines: 10,
                    total_lines: 100,
                    dead_percentage: 10.0,
                    dead_functions: 1,
                    dead_classes: 0,
                    dead_modules: 0,
                    unreachable_blocks: 0,
                    dead_score: 10.0,
                    confidence: ConfidenceLevel::Medium,
                    items: vec![DeadCodeItem {
                        name: "fn_medium".to_string(),
                        item_type: DeadCodeType::Function,
                        line: 5,
                        reason: "unused".to_string(),
                    }],
                },
                FileDeadCodeMetrics {
                    path: "low.rs".to_string(),
                    dead_lines: 10,
                    total_lines: 100,
                    dead_percentage: 10.0,
                    dead_functions: 1,
                    dead_classes: 0,
                    dead_modules: 0,
                    unreachable_blocks: 0,
                    dead_score: 10.0,
                    confidence: ConfidenceLevel::Low,
                    items: vec![DeadCodeItem {
                        name: "fn_low".to_string(),
                        item_type: DeadCodeType::Function,
                        line: 5,
                        reason: "unused".to_string(),
                    }],
                },
            ],
            total_files: 3,
            analyzed_files: 3,
        };

        let sarif_output = format_dead_code_as_sarif(&result).expect("SARIF should work");
        let parsed: serde_json::Value = serde_json::from_str(&sarif_output).unwrap();

        let results = parsed["runs"][0]["results"].as_array().unwrap();

        // Check that we have 3 results with different levels
        assert_eq!(results.len(), 3);

        // Verify levels are mapped correctly
        let levels: Vec<&str> = results
            .iter()
            .map(|r| r["level"].as_str().unwrap())
            .collect();

        assert!(levels.contains(&"error")); // High
        assert!(levels.contains(&"warning")); // Medium
        assert!(levels.contains(&"note")); // Low
    }
}

// Extended Coverage Tests - SATD Formatting and Filtering

#[cfg(test)]
mod satd_format_tests {
    use super::*;
    use crate::cli::{SatdOutputFormat, SatdSeverity};
    use crate::services::satd_detector::{
        DebtCategory, SATDAnalysisResult, SATDSummary, Severity, TechnicalDebt,
    };
    use chrono::Utc;
    use std::collections::HashMap;

    fn create_comprehensive_satd_result() -> SATDAnalysisResult {
        let mut by_severity = HashMap::new();
        by_severity.insert("Critical".to_string(), 2);
        by_severity.insert("High".to_string(), 5);
        by_severity.insert("Medium".to_string(), 8);
        by_severity.insert("Low".to_string(), 10);

        let mut by_category = HashMap::new();
        by_category.insert("Defect".to_string(), 7);
        by_category.insert("Requirement".to_string(), 10);
        by_category.insert("Design".to_string(), 5);
        by_category.insert("Security".to_string(), 3);

        SATDAnalysisResult {
            items: vec![
                TechnicalDebt {
                    category: DebtCategory::Security,
                    severity: Severity::Critical,
                    text: "SECURITY: Validate user input".to_string(),
                    file: PathBuf::from("src/auth.rs"),
                    line: 45,
                    column: 8,
                    context_hash: [0u8; 16],
                },
                TechnicalDebt {
                    category: DebtCategory::Security,
                    severity: Severity::Critical,
                    text: "VULN: SQL injection risk".to_string(),
                    file: PathBuf::from("src/db.rs"),
                    line: 120,
                    column: 4,
                    context_hash: [1u8; 16],
                },
                TechnicalDebt {
                    category: DebtCategory::Defect,
                    severity: Severity::High,
                    text: "BUG: Race condition in cache".to_string(),
                    file: PathBuf::from("src/cache.rs"),
                    line: 88,
                    column: 12,
                    context_hash: [2u8; 16],
                },
                TechnicalDebt {
                    category: DebtCategory::Requirement,
                    severity: Severity::Medium,
                    text: "TODO: Add pagination support".to_string(),
                    file: PathBuf::from("src/api.rs"),
                    line: 200,
                    column: 4,
                    context_hash: [3u8; 16],
                },
                TechnicalDebt {
                    category: DebtCategory::Requirement,
                    severity: Severity::Low,
                    text: "TODO: Nice to have feature".to_string(),
                    file: PathBuf::from("src/utils.rs"),
                    line: 30,
                    column: 4,
                    context_hash: [4u8; 16],
                },
            ],
            summary: SATDSummary {
                total_items: 25,
                by_severity,
                by_category,
                files_with_satd: 8,
                avg_age_days: 45.5,
            },
            total_files_analyzed: 50,
            files_with_debt: 8,
            analysis_timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_generate_satd_sarif() {
        let result = create_comprehensive_satd_result();

        let sarif = generate_satd_sarif(&result);

        // Verify SARIF structure
        assert_eq!(sarif["version"], "2.1.0");
        assert!(sarif["$schema"].as_str().unwrap().contains("sarif-schema"));

        // Check tool information
        let driver = &sarif["runs"][0]["tool"]["driver"];
        assert_eq!(driver["name"], "pmat");
        assert!(driver["rules"].as_array().unwrap().len() > 0);

        // Check results
        let results = sarif["runs"][0]["results"].as_array().unwrap();
        assert_eq!(results.len(), 5);

        // Verify severity mapping (Critical -> error)
        let critical_results: Vec<_> = results
            .iter()
            .filter(|r| r["level"] == "error")
            .collect();
        assert!(critical_results.len() >= 2); // We have 2 Critical items
    }

    #[test]
    fn test_format_satd_output_json() {
        let result = create_comprehensive_satd_result();

        let output = format_satd_output(&result, SatdOutputFormat::Json, false, false, 30)
            .expect("JSON formatting should work");

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&output).expect("Should be valid JSON");

        assert!(parsed.get("items").is_some());
        assert!(parsed.get("summary").is_some());
        assert_eq!(parsed["total_files_analyzed"], 50);
    }

    #[test]
    fn test_format_satd_output_sarif() {
        let result = create_comprehensive_satd_result();

        let output = format_satd_output(&result, SatdOutputFormat::Sarif, false, false, 30)
            .expect("SARIF formatting should work");

        let parsed: serde_json::Value = serde_json::from_str(&output).expect("Should be valid JSON");
        assert_eq!(parsed["version"], "2.1.0");
    }

    #[test]
    fn test_format_satd_output_summary() {
        let result = create_comprehensive_satd_result();

        let output = format_satd_output(&result, SatdOutputFormat::Summary, false, false, 30)
            .expect("Summary formatting should work");

        assert!(output.contains("SATD Analysis Summary"));
        assert!(output.contains("Files analyzed**: 50"));
        assert!(output.contains("Files with SATD**: 8"));
    }

    #[test]
    fn test_format_satd_output_markdown() {
        let result = create_comprehensive_satd_result();

        let output = format_satd_output(&result, SatdOutputFormat::Markdown, true, false, 30)
            .expect("Markdown formatting should work");

        assert!(output.contains("# Self-Admitted Technical Debt Report"));
        assert!(output.contains("## Summary"));
        assert!(output.contains("| Metric | Value |"));
        assert!(output.contains("## Distribution"));
        assert!(output.contains("### By Severity"));
        assert!(output.contains("### By Category"));
    }

    #[test]
    fn test_format_satd_markdown_groups_by_file() {
        let result = create_comprehensive_satd_result();

        let output = format_satd_markdown(&result, true, false, 30);

        // Check that file grouping is present
        assert!(output.contains("## SATD Items by File"));
        assert!(output.contains("### src/auth.rs"));
        assert!(output.contains("| Line | Severity | Category | Text |"));
    }

    #[test]
    fn test_apply_satd_filters_severity_high() {
        let mut result = create_comprehensive_satd_result();
        let original_count = result.items.len();

        apply_satd_filters(&mut result, Some(SatdSeverity::High), false, 0);

        // Should only keep Critical and High items
        assert!(result.items.len() < original_count);
        assert!(result.items.iter().all(|i| i.severity >= Severity::High));
    }

    #[test]
    fn test_apply_satd_filters_critical_only() {
        let mut result = create_comprehensive_satd_result();

        apply_satd_filters(&mut result, None, true, 0);

        // Should only keep Critical items
        assert_eq!(result.items.len(), 2);
        assert!(result
            .items
            .iter()
            .all(|i| i.severity == Severity::Critical));
    }

    #[test]
    fn test_apply_satd_filters_top_files() {
        let mut result = create_comprehensive_satd_result();

        // Apply top files = 2
        apply_satd_filters(&mut result, None, false, 2);

        // Should only have items from top 2 files
        let unique_files: std::collections::HashSet<_> =
            result.items.iter().map(|i| &i.file).collect();
        assert!(unique_files.len() <= 2);
    }

    #[test]
    fn test_filter_top_files_helper() {
        let mut result = create_comprehensive_satd_result();
        let original_item_count = result.items.len();

        filter_top_files(&mut result, 1);

        // Should have fewer items (only from top file)
        assert!(result.items.len() <= original_item_count);
        assert!(!result.items.is_empty());
    }

    #[test]
    fn test_write_top_files_with_satd_section() {
        let result = create_comprehensive_satd_result();
        let mut output = String::new();

        write_top_files_with_satd_section(&mut output, &result);

        assert!(output.contains("## Top Files with SATD"));
        assert!(output.contains("SATD items"));
    }

    #[test]
    fn test_write_critical_items_section() {
        let result = create_comprehensive_satd_result();
        let mut output = String::new();

        write_critical_items_section(&mut output, &result);

        assert!(output.contains("## Critical Items"));
        // Should contain the critical items' file names
        assert!(output.contains("auth.rs") || output.contains("db.rs"));
    }
}

// Extended Coverage Tests - Churn Analysis

#[cfg(test)]
mod churn_tests {
    use super::*;
    use crate::utils::file_filter::FileFilter;

    #[test]
    fn test_create_and_report_file_filter_empty() {
        let filter = create_and_report_file_filter(vec![], vec![]).expect("Should create filter");

        assert!(!filter.has_filters());
    }

    #[test]
    fn test_create_and_report_file_filter_with_include() {
        let filter = create_and_report_file_filter(
            vec!["src/**/*.rs".to_string()],
            vec![],
        )
        .expect("Should create filter");

        assert!(filter.has_filters());
        assert!(filter.should_include(Path::new("src/main.rs")));
    }

    #[test]
    fn test_create_and_report_file_filter_with_exclude() {
        let filter = create_and_report_file_filter(
            vec![],
            vec!["target/**".to_string()],
        )
        .expect("Should create filter");

        assert!(filter.has_filters());
        assert!(!filter.should_include(Path::new("target/debug/main")));
    }

    #[test]
    fn test_create_and_report_file_filter_combined() {
        let filter = create_and_report_file_filter(
            vec!["**/*.rs".to_string()],
            vec!["target/**".to_string()],
        )
        .expect("Should create filter");

        assert!(filter.has_filters());
        assert!(filter.should_include(Path::new("src/main.rs")));
        assert!(!filter.should_include(Path::new("target/debug/main.rs")));
    }
}

// Extended Coverage Tests - Watch Mode Helpers

#[cfg(test)]
mod watch_mode_tests {
    use super::*;
    use notify::event::{AccessKind, CreateKind, ModifyKind, RemoveKind};
    use notify::EventKind;

    #[test]
    fn test_should_analyze_path_rust_file() {
        let path = Path::new("/project/src/main.rs");
        assert!(should_analyze_path(path, &[]));
    }

    #[test]
    fn test_should_analyze_path_typescript_file() {
        let path = Path::new("/project/src/app.ts");
        assert!(should_analyze_path(path, &[]));
    }

    #[test]
    fn test_should_analyze_path_python_file() {
        let path = Path::new("/project/app.py");
        assert!(should_analyze_path(path, &[]));
    }

    #[test]
    fn test_should_analyze_path_non_source() {
        let path = Path::new("/project/README.md");
        assert!(!should_analyze_path(path, &[]));
    }

    #[test]
    fn test_should_analyze_path_with_pattern_match() {
        let path = Path::new("/project/src/main.rs");
        let patterns = vec!["src/".to_string()];
        assert!(should_analyze_path(path, &patterns));
    }

    #[test]
    fn test_should_analyze_path_with_pattern_no_match() {
        let path = Path::new("/project/tests/test.rs");
        let patterns = vec!["src/".to_string()];
        assert!(!should_analyze_path(path, &patterns));
    }

    #[test]
    fn test_is_source_code_file_c_extensions() {
        assert!(is_source_code_file("main.c"));
        assert!(is_source_code_file("main.cpp"));
        assert!(is_source_code_file("header.h"));
        assert!(is_source_code_file("header.hpp"));
    }

    #[test]
    fn test_print_watch_mode_intro_does_not_panic() {
        // This just verifies the function doesn't panic
        let path = Path::new("/test/project");
        print_watch_mode_intro(path);
    }
}

// Extended Coverage Tests - Dead Code Conversion Helpers

#[cfg(test)]
mod dead_code_conversion_tests {
    use super::*;
    use crate::models::dead_code::DeadCodeAnalysisConfig;
    use crate::services::cargo_dead_code_analyzer::{
        AccurateDeadCodeReport, DeadCodeKind, DeadItem, FileDeadCode,
    };
    use std::collections::HashMap;

    fn create_test_accurate_report() -> AccurateDeadCodeReport {
        let mut dead_by_type = HashMap::new();
        dead_by_type.insert("function".to_string(), 5);
        dead_by_type.insert("method".to_string(), 3);
        dead_by_type.insert("struct".to_string(), 2);
        dead_by_type.insert("enum".to_string(), 1);
        dead_by_type.insert("module".to_string(), 1);

        AccurateDeadCodeReport {
            files_with_dead_code: vec![
                FileDeadCode {
                    file_path: PathBuf::from("src/module_a.rs"),
                    dead_items: vec![
                        DeadItem {
                            name: "unused_fn".to_string(),
                            kind: DeadCodeKind::Function,
                            line: 10,
                            column: 1,
                            message: "function never used".to_string(),
                        },
                        DeadItem {
                            name: "unused_method".to_string(),
                            kind: DeadCodeKind::Method,
                            line: 25,
                            column: 5,
                            message: "method never used".to_string(),
                        },
                    ],
                    file_dead_percentage: 15.0,
                },
                FileDeadCode {
                    file_path: PathBuf::from("src/module_b.rs"),
                    dead_items: vec![DeadItem {
                        name: "UnusedStruct".to_string(),
                        kind: DeadCodeKind::Struct,
                        line: 5,
                        column: 1,
                        message: "struct never constructed".to_string(),
                    }],
                    file_dead_percentage: 8.0,
                },
            ],
            total_dead_items: 12,
            dead_code_percentage: 10.5,
            total_lines: 1000,
            dead_lines: 105,
            dead_by_type,
        }
    }

    #[test]
    fn test_count_dead_items_by_kind_functions() {
        let file = FileDeadCode {
            file_path: PathBuf::from("test.rs"),
            dead_items: vec![
                DeadItem {
                    name: "fn1".to_string(),
                    kind: DeadCodeKind::Function,
                    line: 1,
                    column: 1,
                    message: "unused".to_string(),
                },
                DeadItem {
                    name: "fn2".to_string(),
                    kind: DeadCodeKind::Function,
                    line: 10,
                    column: 1,
                    message: "unused".to_string(),
                },
                DeadItem {
                    name: "method1".to_string(),
                    kind: DeadCodeKind::Method,
                    line: 20,
                    column: 5,
                    message: "unused".to_string(),
                },
                DeadItem {
                    name: "Struct1".to_string(),
                    kind: DeadCodeKind::Struct,
                    line: 30,
                    column: 1,
                    message: "unused".to_string(),
                },
            ],
            file_dead_percentage: 20.0,
        };

        let fn_count =
            count_dead_items_by_kind(&file, &[DeadCodeKind::Function, DeadCodeKind::Method]);
        assert_eq!(fn_count, 3); // 2 functions + 1 method

        let struct_count =
            count_dead_items_by_kind(&file, &[DeadCodeKind::Struct, DeadCodeKind::Enum]);
        assert_eq!(struct_count, 1);
    }

    #[test]
    fn test_get_dead_count_by_types() {
        let report = create_test_accurate_report();

        let fn_count = get_dead_count_by_types(&report, &["function", "method"]);
        assert_eq!(fn_count, 8); // 5 functions + 3 methods

        let class_count = get_dead_count_by_types(&report, &["struct", "enum"]);
        assert_eq!(class_count, 3); // 2 structs + 1 enum

        let module_count = get_dead_count_by_types(&report, &["module"]);
        assert_eq!(module_count, 1);

        // Test non-existent type
        let nonexistent = get_dead_count_by_types(&report, &["nonexistent"]);
        assert_eq!(nonexistent, 0);
    }

    #[test]
    fn test_create_dead_code_summary() {
        let report = create_test_accurate_report();

        let summary = create_dead_code_summary(&report, 2);

        assert_eq!(summary.files_with_dead_code, 2);
        assert_eq!(summary.total_dead_lines, 105);
        assert_eq!(summary.dead_percentage, 10.5);
        assert_eq!(summary.dead_functions, 8); // function + method
        assert_eq!(summary.dead_classes, 3); // struct + enum
        assert_eq!(summary.dead_modules, 1);
    }

    #[test]
    fn test_convert_cargo_files_to_metrics() {
        let cargo_files = vec![
            FileDeadCode {
                file_path: PathBuf::from("src/a.rs"),
                dead_items: vec![
                    DeadItem {
                        name: "fn1".to_string(),
                        kind: DeadCodeKind::Function,
                        line: 10,
                        column: 1,
                        message: "unused".to_string(),
                    },
                    DeadItem {
                        name: "fn2".to_string(),
                        kind: DeadCodeKind::Function,
                        line: 20,
                        column: 1,
                        message: "unused".to_string(),
                    },
                ],
                file_dead_percentage: 20.0,
            },
            FileDeadCode {
                file_path: PathBuf::from("src/b.rs"),
                dead_items: vec![DeadItem {
                    name: "Struct1".to_string(),
                    kind: DeadCodeKind::Struct,
                    line: 5,
                    column: 1,
                    message: "unused".to_string(),
                }],
                file_dead_percentage: 5.0,
            },
        ];

        let metrics = convert_cargo_files_to_metrics(cargo_files, 0);

        assert_eq!(metrics.len(), 2);

        let first = &metrics[0];
        assert_eq!(first.path, "src/a.rs");
        assert_eq!(first.dead_functions, 2);
        assert_eq!(first.dead_percentage, 20.0);

        let second = &metrics[1];
        assert_eq!(second.path, "src/b.rs");
        assert_eq!(second.dead_classes, 1);
    }

    #[test]
    fn test_convert_cargo_files_to_metrics_with_min_lines_filter() {
        let cargo_files = vec![
            FileDeadCode {
                file_path: PathBuf::from("src/small.rs"),
                dead_items: vec![DeadItem {
                    name: "fn1".to_string(),
                    kind: DeadCodeKind::Function,
                    line: 1,
                    column: 1,
                    message: "unused".to_string(),
                }],
                file_dead_percentage: 5.0,
            },
            FileDeadCode {
                file_path: PathBuf::from("src/large.rs"),
                dead_items: vec![
                    DeadItem {
                        name: "fn1".to_string(),
                        kind: DeadCodeKind::Function,
                        line: 1,
                        column: 1,
                        message: "unused".to_string(),
                    },
                    DeadItem {
                        name: "fn2".to_string(),
                        kind: DeadCodeKind::Function,
                        line: 10,
                        column: 1,
                        message: "unused".to_string(),
                    },
                    DeadItem {
                        name: "fn3".to_string(),
                        kind: DeadCodeKind::Function,
                        line: 20,
                        column: 1,
                        message: "unused".to_string(),
                    },
                ],
                file_dead_percentage: 30.0,
            },
        ];

        // With min_dead_lines = 10, only the larger file should be included
        // Each item is estimated at 4 lines, so small.rs has 4 lines, large.rs has 12
        let metrics = convert_cargo_files_to_metrics(cargo_files, 10);

        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].path, "src/large.rs");
    }

    #[test]
    fn test_create_dead_code_ranking_result() {
        let report = create_test_accurate_report();
        let config = DeadCodeAnalysisConfig {
            include_unreachable: true,
            include_tests: false,
            min_dead_lines: 0,
        };

        let result = create_dead_code_ranking_result(report, 2, 0, config);

        assert_eq!(result.ranked_files.len(), 2);
        assert_eq!(result.summary.files_with_dead_code, 2);
        assert_eq!(result.config.include_unreachable, true);
        assert_eq!(result.config.include_tests, false);
    }
}

// Extended Coverage Tests - Complexity Config Additional

#[cfg(test)]
mod complexity_config_additional_tests {
    use super::*;

    #[test]
    fn test_complexity_config_debug_format() {
        let config = ComplexityConfig::from_args(
            PathBuf::from("/test"),
            Some("python".to_string()),
            Some(15),
            Some(20),
            vec!["*.py".to_string()],
            60,
            10,
        );

        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("ComplexityConfig"));
        assert!(debug_str.contains("python"));
        assert!(debug_str.contains("15"));
        assert!(debug_str.contains("20"));
    }

    #[test]
    fn test_complexity_config_with_multiple_include_patterns() {
        let config = ComplexityConfig::from_args(
            PathBuf::from("/project"),
            None,
            None,
            None,
            vec![
                "src/**/*.rs".to_string(),
                "lib/**/*.rs".to_string(),
                "crates/**/*.rs".to_string(),
            ],
            120,
            20,
        );

        assert_eq!(config.include.len(), 3);
        assert!(config.include.contains(&"src/**/*.rs".to_string()));
        assert!(config.include.contains(&"lib/**/*.rs".to_string()));
        assert!(config.include.contains(&"crates/**/*.rs".to_string()));
    }
}

// Extended Coverage Tests - Edge Cases

#[cfg(test)]
mod edge_case_tests {
    use super::*;
    use crate::models::dead_code::{
        ConfidenceLevel, DeadCodeItem, DeadCodeResult, DeadCodeSummary, DeadCodeType,
        FileDeadCodeMetrics,
    };

    #[test]
    fn test_format_dead_code_as_summary_empty_result() {
        let result = DeadCodeResult {
            summary: DeadCodeSummary {
                total_files_analyzed: 0,
                files_with_dead_code: 0,
                total_dead_lines: 0,
                dead_percentage: 0.0,
                dead_functions: 0,
                dead_classes: 0,
                dead_modules: 0,
                unreachable_blocks: 0,
            },
            files: vec![],
            total_files: 0,
            analyzed_files: 0,
        };

        let summary = format_dead_code_as_summary(&result).expect("Should handle empty result");

        assert!(summary.contains("# Dead Code Analysis Summary"));
        assert!(summary.contains("**Files analyzed**: 0"));
        assert!(!summary.contains("## Top Files")); // Should not have this section
    }

    #[test]
    fn test_apply_complexity_filters_edge_cases() {
        // Test with exact threshold values
        let mut metrics = vec![create_test_file_metrics_simple("exact.rs", 20, 15)];

        // Threshold is exclusive (must exceed, not equal)
        let filtered = apply_complexity_filters(&mut metrics, Some(20), Some(15));
        assert_eq!(filtered, 1); // Should be filtered out since not exceeding
        assert!(metrics.is_empty());
    }

    #[test]
    fn test_apply_top_files_limit_larger_than_input() {
        let mut metrics = vec![
            create_test_file_metrics_simple("a.rs", 10, 10),
            create_test_file_metrics_simple("b.rs", 20, 20),
        ];

        apply_top_files_limit(&mut metrics, 100);

        // Should keep all files
        assert_eq!(metrics.len(), 2);
    }

    #[test]
    fn test_dead_code_sarif_with_empty_items() {
        let result = DeadCodeResult {
            summary: DeadCodeSummary {
                total_files_analyzed: 5,
                files_with_dead_code: 0,
                total_dead_lines: 0,
                dead_percentage: 0.0,
                dead_functions: 0,
                dead_classes: 0,
                dead_modules: 0,
                unreachable_blocks: 0,
            },
            files: vec![],
            total_files: 5,
            analyzed_files: 5,
        };

        let sarif = format_dead_code_as_sarif(&result).expect("Should handle empty files");
        let parsed: serde_json::Value = serde_json::from_str(&sarif).unwrap();

        let results = parsed["runs"][0]["results"].as_array().unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_dead_code_markdown_with_many_files() {
        // Test that file details section limits to 20 files
        let files: Vec<FileDeadCodeMetrics> = (0..30)
            .map(|i| FileDeadCodeMetrics {
                path: format!("src/file_{}.rs", i),
                dead_lines: 10,
                total_lines: 100,
                dead_percentage: 10.0,
                dead_functions: 1,
                dead_classes: 0,
                dead_modules: 0,
                unreachable_blocks: 0,
                dead_score: 10.0,
                confidence: ConfidenceLevel::Medium,
                items: vec![],
            })
            .collect();

        let section = format_dead_code_file_details_section(&files);

        // Should only contain first 20 files
        assert!(section.contains("file_0.rs"));
        assert!(section.contains("file_19.rs"));
        assert!(!section.contains("file_20.rs")); // Should not appear
    }

    #[test]
    fn test_has_complexity_violations_empty_metrics() {
        let metrics: Vec<crate::services::complexity::FileComplexityMetrics> = vec![];

        let has_violations = has_complexity_violations(&metrics, Some(10), Some(10));
        assert!(!has_violations);
    }

    fn create_test_file_metrics_simple(
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
                name: "test_fn".to_string(),
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
}

// Extended Property Tests

#[cfg(test)]
mod extended_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_dead_code_summary_section_always_valid(
            total_files in 0usize..1000,
            files_with_dead in 0usize..1000,
            dead_lines in 0usize..10000,
            percentage in 0.0f32..100.0f32
        ) {
            let result = crate::models::dead_code::DeadCodeResult {
                summary: crate::models::dead_code::DeadCodeSummary {
                    total_files_analyzed: total_files,
                    files_with_dead_code: files_with_dead,
                    total_dead_lines: dead_lines,
                    dead_percentage: percentage,
                    dead_functions: 0,
                    dead_classes: 0,
                    dead_modules: 0,
                    unreachable_blocks: 0,
                },
                files: vec![],
                total_files,
                analyzed_files: total_files,
            };

            let section = format_dead_code_summary_section(&result);

            prop_assert!(section.contains("Dead Code Analysis Report"));
            prop_assert!(!section.is_empty());
        }

        #[test]
        fn test_top_files_limit_never_exceeds_original(
            file_count in 1usize..100,
            limit in 1usize..50
        ) {
            let mut metrics: Vec<crate::services::complexity::FileComplexityMetrics> = (0..file_count)
                .map(|i| crate::services::complexity::FileComplexityMetrics {
                    path: format!("file{}.rs", i),
                    language: "rust".to_string(),
                    total_complexity: crate::services::complexity::ComplexityMetrics {
                        cyclomatic: (i as u16) + 1,
                        cognitive: (i as u16) + 1,
                        nesting_depth: 2,
                        line_count: 100,
                        function_count: 1,
                    },
                    functions: vec![],
                    function_count: 0,
                })
                .collect();

            let original_count = metrics.len();
            apply_top_files_limit(&mut metrics, limit);

            prop_assert!(metrics.len() <= limit.min(original_count));
        }

        #[test]
        fn test_complexity_filters_preserve_invariant(
            file_count in 1usize..50,
            threshold in 1u16..100
        ) {
            let mut metrics: Vec<crate::services::complexity::FileComplexityMetrics> = (0..file_count)
                .map(|i| crate::services::complexity::FileComplexityMetrics {
                    path: format!("file{}.rs", i),
                    language: "rust".to_string(),
                    total_complexity: crate::services::complexity::ComplexityMetrics {
                        cyclomatic: (i as u16) % 50 + 1,
                        cognitive: (i as u16) % 50 + 1,
                        nesting_depth: 2,
                        line_count: 100,
                        function_count: 1,
                    },
                    functions: vec![crate::services::complexity::FunctionComplexity {
                        name: "fn".to_string(),
                        line: 1,
                        metrics: crate::services::complexity::ComplexityMetrics {
                            cyclomatic: (i as u16) % 50 + 1,
                            cognitive: (i as u16) % 50 + 1,
                            nesting_depth: 2,
                            line_count: 20,
                            function_count: 1,
                        },
                    }],
                    function_count: 1,
                })
                .collect();

            let original_count = metrics.len();
            let filtered = apply_complexity_filters(&mut metrics, Some(threshold), Some(threshold));

            // Invariant: filtered + remaining = original
            prop_assert_eq!(filtered + metrics.len(), original_count);
        }
    }
}
