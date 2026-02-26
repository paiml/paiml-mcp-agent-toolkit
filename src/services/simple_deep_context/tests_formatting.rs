    // ============ generate_recommendations Tests ============

    #[test]
    fn test_generate_recommendations_high_complexity() {
        let analyzer = SimpleDeepContext;
        let metrics = ComplexityMetrics {
            total_functions: 10,
            high_complexity_count: 3,
            avg_complexity: 4.0,
        };

        let recommendations = analyzer.generate_recommendations(&metrics);
        assert!(!recommendations.is_empty());
        assert!(recommendations
            .iter()
            .any(|r| r.contains("refactoring") && r.contains("3")));
    }

    #[test]
    fn test_generate_recommendations_high_avg_complexity() {
        let analyzer = SimpleDeepContext;
        let metrics = ComplexityMetrics {
            total_functions: 10,
            high_complexity_count: 0,
            avg_complexity: 7.5,
        };

        let recommendations = analyzer.generate_recommendations(&metrics);
        assert!(!recommendations.is_empty());
        assert!(recommendations
            .iter()
            .any(|r| r.contains("Average") && r.contains("7.5")));
    }

    #[test]
    fn test_generate_recommendations_no_functions() {
        let analyzer = SimpleDeepContext;
        let metrics = ComplexityMetrics {
            total_functions: 0,
            high_complexity_count: 0,
            avg_complexity: 0.0,
        };

        let recommendations = analyzer.generate_recommendations(&metrics);
        assert!(!recommendations.is_empty());
        assert!(recommendations
            .iter()
            .any(|r| r.contains("No functions detected")));
    }

    #[test]
    fn test_generate_recommendations_good_code() {
        let analyzer = SimpleDeepContext;
        let metrics = ComplexityMetrics {
            total_functions: 20,
            high_complexity_count: 0,
            avg_complexity: 2.5,
        };

        let recommendations = analyzer.generate_recommendations(&metrics);
        assert!(!recommendations.is_empty());
        assert!(recommendations.iter().any(|r| r.contains("looks good")));
    }

    // ============ format_as_json Tests ============

    #[test]
    fn test_format_as_json_empty_report() {
        let analyzer = SimpleDeepContext;
        let report = SimpleAnalysisReport {
            file_count: 0,
            analysis_duration: std::time::Duration::from_millis(100),
            complexity_metrics: ComplexityMetrics {
                total_functions: 0,
                high_complexity_count: 0,
                avg_complexity: 0.0,
            },
            recommendations: vec!["No files found".to_string()],
            file_complexity_details: vec![],
        };

        let json = analyzer.format_as_json(&report).unwrap();
        assert!(json.contains("\"file_count\": 0"));
        assert!(json.contains("\"total_functions\": 0"));
        assert!(json.contains("\"recommendations\""));
    }

    #[test]
    fn test_format_as_json_with_files() {
        let analyzer = SimpleDeepContext;
        let report = SimpleAnalysisReport {
            file_count: 2,
            analysis_duration: std::time::Duration::from_millis(500),
            complexity_metrics: ComplexityMetrics {
                total_functions: 15,
                high_complexity_count: 3,
                avg_complexity: 5.5,
            },
            recommendations: vec!["Consider refactoring".to_string()],
            file_complexity_details: vec![
                FileComplexityDetail {
                    file_path: std::path::PathBuf::from("src/main.rs"),
                    function_count: 10,
                    high_complexity_functions: 2,
                    avg_complexity: 6.0,
                    complexity_score: 8.0,
                    function_names: vec!["main".to_string(), "helper".to_string()],
                },
                FileComplexityDetail {
                    file_path: std::path::PathBuf::from("src/lib.rs"),
                    function_count: 5,
                    high_complexity_functions: 1,
                    avg_complexity: 4.5,
                    complexity_score: 5.0,
                    function_names: vec!["process".to_string()],
                },
            ],
        };

        let json = analyzer.format_as_json(&report).unwrap();
        assert!(json.contains("\"file_count\": 2"));
        assert!(json.contains("\"total_functions\": 15"));
        assert!(json.contains("main.rs"));
        assert!(json.contains("lib.rs"));
        // Check JSON is valid by parsing it
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["summary"]["file_count"], 2);
    }

    // ============ format_as_markdown Tests ============

    #[test]
    fn test_format_as_markdown_empty_report() {
        let analyzer = SimpleDeepContext;
        let report = SimpleAnalysisReport {
            file_count: 0,
            analysis_duration: std::time::Duration::from_millis(50),
            complexity_metrics: ComplexityMetrics {
                total_functions: 0,
                high_complexity_count: 0,
                avg_complexity: 0.0,
            },
            recommendations: vec!["No files found".to_string()],
            file_complexity_details: vec![],
        };

        let markdown = analyzer.format_as_markdown(&report, 10);
        assert!(markdown.contains("# Deep Context Analysis Report"));
        assert!(markdown.contains("**Files Analyzed**: 0"));
        assert!(markdown.contains("## Recommendations"));
    }

    #[test]
    fn test_format_as_markdown_with_files() {
        let analyzer = SimpleDeepContext;
        let report = SimpleAnalysisReport {
            file_count: 3,
            analysis_duration: std::time::Duration::from_secs(1),
            complexity_metrics: ComplexityMetrics {
                total_functions: 30,
                high_complexity_count: 5,
                avg_complexity: 6.2,
            },
            recommendations: vec!["Consider refactoring".to_string(), "Add tests".to_string()],
            file_complexity_details: vec![
                FileComplexityDetail {
                    file_path: std::path::PathBuf::from("complex.rs"),
                    function_count: 15,
                    high_complexity_functions: 3,
                    avg_complexity: 8.0,
                    complexity_score: 12.0,
                    function_names: vec![],
                },
                FileComplexityDetail {
                    file_path: std::path::PathBuf::from("medium.rs"),
                    function_count: 10,
                    high_complexity_functions: 2,
                    avg_complexity: 5.0,
                    complexity_score: 7.0,
                    function_names: vec![],
                },
            ],
        };

        let markdown = analyzer.format_as_markdown(&report, 10);
        assert!(markdown.contains("**Files Analyzed**: 3"));
        assert!(markdown.contains("**Total Functions**: 30"));
        assert!(markdown.contains("**High Complexity Functions**: 5"));
        assert!(markdown.contains("## Top Files by Complexity"));
        // Check that files are sorted by complexity score (descending)
        let complex_pos = markdown.find("complex.rs").unwrap_or(usize::MAX);
        let medium_pos = markdown.find("medium.rs").unwrap_or(usize::MAX);
        assert!(
            complex_pos < medium_pos,
            "Higher complexity file should appear first"
        );
    }

    #[test]
    fn test_format_as_markdown_zero_top_files() {
        let analyzer = SimpleDeepContext;
        let report = SimpleAnalysisReport {
            file_count: 1,
            analysis_duration: std::time::Duration::from_millis(100),
            complexity_metrics: ComplexityMetrics {
                total_functions: 5,
                high_complexity_count: 1,
                avg_complexity: 3.0,
            },
            recommendations: vec![],
            file_complexity_details: vec![FileComplexityDetail {
                file_path: std::path::PathBuf::from("test.rs"),
                function_count: 5,
                high_complexity_functions: 1,
                avg_complexity: 3.0,
                complexity_score: 5.0,
                function_names: vec![],
            }],
        };

        // When top_files is 0, it should default to 10
        let markdown = analyzer.format_as_markdown(&report, 0);
        assert!(markdown.contains("test.rs"));
    }
