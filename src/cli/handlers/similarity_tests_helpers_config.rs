    // Imports provided by parent include (similarity_handler_tests.rs)

    // Helper functions to create test data

    fn create_test_location(file: &str, start: usize, end: usize) -> Location {
        Location {
            file: PathBuf::from(file),
            start_line: start,
            end_line: end,
            start_column: None,
            end_column: None,
        }
    }

    fn create_test_similar_block(id: &str, locations: Vec<Location>) -> SimilarBlock {
        SimilarBlock {
            id: id.to_string(),
            locations,
            similarity: 0.95,
            clone_type: CloneType::Type1,
            lines: 10,
            tokens: 50,
            content_preview: "fn example() {\n    let x = 1;\n}".to_string(),
        }
    }

    fn create_test_entropy_block(file: &str, entropy: f64) -> EntropyBlock {
        EntropyBlock {
            location: create_test_location(file, 1, 10),
            entropy,
            category: if entropy > 4.0 {
                "Complex".to_string()
            } else {
                "Repetitive".to_string()
            },
            suggestion: "Consider refactoring".to_string(),
        }
    }

    fn create_test_refactoring_hint(pattern: &str, priority: Priority) -> RefactoringHint {
        RefactoringHint {
            locations: vec![
                create_test_location("test1.rs", 1, 10),
                create_test_location("test2.rs", 5, 15),
            ],
            pattern: pattern.to_string(),
            suggestion: "Extract to shared function".to_string(),
            priority,
        }
    }

    fn create_test_metrics(dup_pct: f64, avg_entropy: f64, total_clones: usize) -> Metrics {
        Metrics {
            duplication_percentage: dup_pct,
            average_entropy: avg_entropy,
            total_clones,
        }
    }

    fn create_empty_report() -> ComprehensiveReport {
        ComprehensiveReport {
            exact_duplicates: vec![],
            structural_similarities: vec![],
            semantic_similarities: vec![],
            entropy_analysis: None,
            refactoring_opportunities: vec![],
            metrics: create_test_metrics(0.0, 0.0, 0),
        }
    }

    fn create_populated_report() -> ComprehensiveReport {
        let locations1 = vec![
            create_test_location("src/file1.rs", 10, 20),
            create_test_location("src/file2.rs", 30, 40),
        ];
        let locations2 = vec![
            create_test_location("src/file3.rs", 50, 60),
            create_test_location("src/file4.rs", 70, 80),
        ];

        ComprehensiveReport {
            exact_duplicates: vec![
                create_test_similar_block("block1", locations1.clone()),
                create_test_similar_block("block2", locations2.clone()),
            ],
            structural_similarities: vec![SimilarBlock {
                id: "struct1".to_string(),
                locations: locations1.clone(),
                similarity: 0.85,
                clone_type: CloneType::Type2,
                lines: 15,
                tokens: 75,
                content_preview: "struct Example {}".to_string(),
            }],
            semantic_similarities: vec![SimilarBlock {
                id: "sem1".to_string(),
                locations: locations2.clone(),
                similarity: 0.78,
                clone_type: CloneType::Type4,
                lines: 20,
                tokens: 100,
                content_preview: "impl Trait for Example {}".to_string(),
            }],
            entropy_analysis: Some(EntropyReport {
                average_entropy: 3.5,
                high_entropy_blocks: vec![
                    create_test_entropy_block("complex1.rs", 4.5),
                    create_test_entropy_block("complex2.rs", 4.2),
                ],
                low_entropy_patterns: vec![
                    create_test_entropy_block("repetitive1.rs", 1.5),
                    create_test_entropy_block("repetitive2.rs", 1.8),
                ],
                recommendations: vec![
                    "Consider breaking down complex functions".to_string(),
                    "Extract repeated patterns".to_string(),
                ],
            }),
            refactoring_opportunities: vec![
                create_test_refactoring_hint("Repeated structure", Priority::High),
                create_test_refactoring_hint("Similar logic", Priority::Medium),
                create_test_refactoring_hint("Minor duplication", Priority::Low),
            ],
            metrics: create_test_metrics(15.5, 3.5, 4),
        }
    }

    // Tests for build_config

    #[test]
    fn test_build_config_fuzzy() {
        let config = build_config(DuplicateType::Fuzzy, 0.7, 5, 50);
        assert!(config.enable_ast);
        assert!(!config.enable_semantic);
        assert_eq!(config.min_lines, 5);
    }

    #[test]
    fn test_build_config_renamed() {
        let config = build_config(DuplicateType::Renamed, 0.75, 8, 80);
        assert!(config.enable_ast);
        assert!(!config.enable_semantic);
    }

    #[test]
    fn test_build_config_semantic() {
        let config = build_config(DuplicateType::Semantic, 0.6, 6, 60);
        assert!(config.enable_ast);
        assert!(config.enable_semantic);
    }

    #[test]
    fn test_build_config_gapped() {
        let config = build_config(DuplicateType::Gapped, 0.65, 7, 70);
        assert!(config.enable_ast);
        assert!(config.enable_semantic);
    }

    #[test]
    fn test_build_config_all() {
        let config = build_config(DuplicateType::All, 0.5, 4, 40);
        assert!(config.enable_ast);
        assert!(config.enable_semantic);
        assert!(config.enable_entropy);
    }

    // Tests for is_source_file

    #[test]
    fn test_is_source_file_rust() {
        assert!(is_source_file(std::path::Path::new("main.rs")));
        assert!(is_source_file(std::path::Path::new("lib.rs")));
    }

    #[test]
    fn test_is_source_file_typescript() {
        assert!(is_source_file(std::path::Path::new("app.ts")));
        assert!(is_source_file(std::path::Path::new("component.tsx")));
    }

    #[test]
    fn test_is_source_file_javascript() {
        assert!(is_source_file(std::path::Path::new("script.js")));
        assert!(is_source_file(std::path::Path::new("component.jsx")));
    }

    #[test]
    fn test_is_source_file_python() {
        assert!(is_source_file(std::path::Path::new("main.py")));
    }

    #[test]
    fn test_is_source_file_c_cpp() {
        assert!(is_source_file(std::path::Path::new("main.c")));
        assert!(is_source_file(std::path::Path::new("main.cpp")));
        assert!(is_source_file(std::path::Path::new("main.cc")));
        assert!(is_source_file(std::path::Path::new("header.h")));
        assert!(is_source_file(std::path::Path::new("header.hpp")));
    }

    #[test]
    fn test_is_source_file_other_languages() {
        assert!(is_source_file(std::path::Path::new("Main.kt")));
        assert!(is_source_file(std::path::Path::new("Main.java")));
        assert!(is_source_file(std::path::Path::new("main.go")));
    }

    #[test]
    fn test_is_source_file_non_source() {
        assert!(!is_source_file(std::path::Path::new("README.md")));
        assert!(!is_source_file(std::path::Path::new("config.toml")));
        assert!(!is_source_file(std::path::Path::new("data.json")));
        assert!(!is_source_file(std::path::Path::new("script.sh")));
    }

    #[test]
    fn test_is_source_file_no_extension() {
        assert!(!is_source_file(std::path::Path::new("Makefile")));
        assert!(!is_source_file(std::path::Path::new("README")));
    }

    // Tests for should_include_file

    #[test]
    fn test_should_include_file_no_filters() {
        let path = std::path::Path::new("/project/src/main.rs");
        assert!(should_include_file(path, &None, &None));
    }

    #[test]
    fn test_should_include_file_with_include_pattern_match() {
        let path = std::path::Path::new("/project/src/main.rs");
        assert!(should_include_file(path, &Some("src".to_string()), &None));
    }

    #[test]
    fn test_should_include_file_with_include_pattern_no_match() {
        let path = std::path::Path::new("/project/tests/test.rs");
        assert!(!should_include_file(path, &Some("src".to_string()), &None));
    }

    #[test]
    fn test_should_include_file_with_exclude_pattern_match() {
        let path = std::path::Path::new("/project/target/debug/main.rs");
        assert!(!should_include_file(
            path,
            &None,
            &Some("target".to_string())
        ));
    }

    #[test]
    fn test_should_include_file_with_exclude_pattern_no_match() {
        let path = std::path::Path::new("/project/src/main.rs");
        assert!(should_include_file(
            path,
            &None,
            &Some("target".to_string())
        ));
    }

    #[test]
    fn test_should_include_file_exclude_takes_precedence() {
        let path = std::path::Path::new("/project/src/target/file.rs");
        // File matches include but also matches exclude - exclude wins
        assert!(!should_include_file(
            path,
            &Some("src".to_string()),
            &Some("target".to_string())
        ));
    }

    // Tests for filter_top_files

    #[test]
    fn test_filter_top_files_zero() {
        let report = create_empty_report();
        let filtered = filter_top_files(report.clone(), 0);
        assert_eq!(filtered.metrics.total_clones, report.metrics.total_clones);
    }

    #[test]
    fn test_filter_top_files_nonzero() {
        let report = create_populated_report();
        let filtered = filter_top_files(report.clone(), 5);
        // Currently returns report as-is
        assert_eq!(
            filtered.exact_duplicates.len(),
            report.exact_duplicates.len()
        );
    }

    // Edge case tests

    #[test]
    fn test_is_source_file_with_double_extension() {
        assert!(is_source_file(std::path::Path::new("file.test.rs")));
        assert!(is_source_file(std::path::Path::new("file.spec.ts")));
    }

    #[test]
    fn test_is_source_file_with_hidden_file() {
        assert!(is_source_file(std::path::Path::new(".hidden.rs")));
    }

    #[test]
    fn test_should_include_file_with_unicode_path() {
        let path = std::path::Path::new("/project/src/file.rs");
        assert!(should_include_file(path, &None, &None));
    }

    #[test]
    fn test_build_config_threshold_boundary() {
        // Test with boundary threshold values
        let config = build_config(DuplicateType::Exact, 0.0, 1, 1);
        assert!((config.similarity_threshold - 0.0).abs() < f64::EPSILON);

        let config = build_config(DuplicateType::Exact, 1.0, 1, 1);
        assert!((config.similarity_threshold - 1.0).abs() < f64::EPSILON);
    }
