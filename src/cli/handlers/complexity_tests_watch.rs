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

