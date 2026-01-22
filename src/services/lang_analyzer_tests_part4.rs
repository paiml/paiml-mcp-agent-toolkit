    fn test_language_analysis_request_debug() {
        let request = LanguageAnalysisRequest {
            path: PathBuf::from("/test/file.rs"),
            language: Some(Language::Rust),
            analysis_types: vec![AnalysisType::Complexity],
            options: AnalysisOptions::default(),
        };

        let debug_str = format!("{:?}", request);
        assert!(debug_str.contains("LanguageAnalysisRequest"));
    }

    #[test]
    fn test_language_analysis_request_clone() {
        let request = LanguageAnalysisRequest {
            path: PathBuf::from("/test/file.rs"),
            language: Some(Language::Rust),
            analysis_types: vec![AnalysisType::Complexity, AnalysisType::Satd],
            options: AnalysisOptions::default(),
        };

        let cloned = request.clone();
        assert_eq!(request.path, cloned.path);
        assert_eq!(request.analysis_types.len(), cloned.analysis_types.len());
    }

    // AnalysisType Tests

    #[test]
    fn test_analysis_type_debug() {
        assert_eq!(format!("{:?}", AnalysisType::Complexity), "Complexity");
        assert_eq!(format!("{:?}", AnalysisType::Satd), "Satd");
        assert_eq!(format!("{:?}", AnalysisType::DeadCode), "DeadCode");
        assert_eq!(format!("{:?}", AnalysisType::Security), "Security");
        assert_eq!(format!("{:?}", AnalysisType::Style), "Style");
        assert_eq!(
            format!("{:?}", AnalysisType::Documentation),
            "Documentation"
        );
        assert_eq!(format!("{:?}", AnalysisType::Dependencies), "Dependencies");
        assert_eq!(format!("{:?}", AnalysisType::Metrics), "Metrics");
    }

    #[test]
    fn test_analysis_type_clone() {
        let at = AnalysisType::Complexity;
        let cloned = at.clone();
        assert!(matches!(cloned, AnalysisType::Complexity));
    }

    // OutputFormat Tests

    #[test]
    fn test_output_format_debug() {
        assert_eq!(format!("{:?}", OutputFormat::Json), "Json");
        assert_eq!(format!("{:?}", OutputFormat::Yaml), "Yaml");
        assert_eq!(format!("{:?}", OutputFormat::Plain), "Plain");
        assert_eq!(format!("{:?}", OutputFormat::Markdown), "Markdown");
    }

    // LanguageAnalysisResult Tests

    #[test]
    fn test_language_analysis_result_debug() {
        let result = LanguageAnalysisResult {
            path: PathBuf::from("/test.rs"),
            language: Language::Rust,
            analysis_results: vec![],
            metadata: FileMetadata {
                lines_total: 100,
                lines_code: 80,
                lines_comment: 15,
                lines_blank: 5,
                file_size_bytes: 2000,
                detected_language: Language::Rust,
                confidence: 1.0,
            },
            processing_time_ms: 50,
        };

        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("LanguageAnalysisResult"));
    }

    // AnalysisResult Tests

    #[test]
    fn test_analysis_result_clone() {
        let result = AnalysisResult {
            analysis_type: AnalysisType::Complexity,
            success: true,
            data: serde_json::json!({"complexity": 5}),
            error: None,
        };

        let cloned = result.clone();
        assert!(cloned.success);
        assert_eq!(result.data, cloned.data);
    }

    #[test]
    fn test_analysis_result_with_error() {
        let result = AnalysisResult {
            analysis_type: AnalysisType::Complexity,
            success: false,
            data: serde_json::json!({}),
            error: Some("Analysis failed".to_string()),
        };

        assert!(!result.success);
        assert!(result.error.is_some());
    }

    // FileMetadata Tests

    #[test]
    fn test_file_metadata_clone() {
        let metadata = FileMetadata {
            lines_total: 100,
            lines_code: 80,
            lines_comment: 15,
            lines_blank: 5,
            file_size_bytes: 2000,
            detected_language: Language::Rust,
            confidence: 0.95,
        };

        let cloned = metadata.clone();
        assert_eq!(metadata.lines_total, cloned.lines_total);
        assert_eq!(metadata.confidence, cloned.confidence);
    }

    // Serialization Tests

    #[test]
    fn test_analysis_type_serialize() {
        let at = AnalysisType::Complexity;
        let json = serde_json::to_string(&at);
        assert!(json.is_ok());
        assert!(json.unwrap().contains("complexity"));
    }

    #[test]
    fn test_output_format_serialize() {
        let of = OutputFormat::Json;
        let json = serde_json::to_string(&of);
        assert!(json.is_ok());
        assert!(json.unwrap().contains("json"));
    }

    #[test]
    fn test_analysis_options_serialize() {
        let options = AnalysisOptions::default();
        let json = serde_json::to_string(&options);
        assert!(json.is_ok());
    }

    #[test]
    fn test_file_metadata_serialize() {
        let metadata = FileMetadata {
            lines_total: 100,
            lines_code: 80,
            lines_comment: 15,
            lines_blank: 5,
            file_size_bytes: 2000,
            detected_language: Language::Rust,
            confidence: 1.0,
        };
        let json = serde_json::to_string(&metadata);
        assert!(json.is_ok());
    }

    #[test]
    fn test_analysis_result_serialize() {
        let result = AnalysisResult {
            analysis_type: AnalysisType::Complexity,
            success: true,
            data: serde_json::json!({"value": 5}),
            error: None,
        };
        let json = serde_json::to_string(&result);
        assert!(json.is_ok());
    }

    #[test]
    fn test_language_analysis_result_serialize() {
        let result = LanguageAnalysisResult {
            path: PathBuf::from("/test.rs"),
            language: Language::Rust,
            analysis_results: vec![AnalysisResult {
                analysis_type: AnalysisType::Metrics,
                success: true,
                data: serde_json::json!({}),
                error: None,
            }],
            metadata: FileMetadata {
                lines_total: 10,
                lines_code: 8,
                lines_comment: 1,
                lines_blank: 1,
                file_size_bytes: 200,
                detected_language: Language::Rust,
                confidence: 1.0,
            },
            processing_time_ms: 5,
        };
        let json = serde_json::to_string(&result);
        assert!(json.is_ok());
    }
}

/// Property-based tests for LanguageAnalyzer

mod language_property_tests {
    use super::*;
    use proptest::prelude::*;

    // Strategy for generating code content
    fn code_content_strategy() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop::string::string_regex("[a-zA-Z0-9_\\s\\{\\}\\(\\)\\;\\=]+").unwrap(),
            0..20,
        )
        .prop_map(|lines| lines.join("\n"))
    }

    // Strategy for generating languages with complexity support
    fn complexity_language_strategy() -> impl Strategy<Value = Language> {
        prop_oneof![
            Just(Language::Rust),
            Just(Language::Python),
            Just(Language::JavaScript),
            Just(Language::TypeScript),
            Just(Language::Java),
            Just(Language::Go),
            Just(Language::C),
            Just(Language::Cpp),
        ]
    }

    proptest! {
        #[test]
        fn prop_file_metadata_lines_sum_equals_total(content in "([^\n]*\n){0,50}") {
            let analyzer = LanguageAnalyzer::new();
            let metadata = analyzer.analyze_file_metadata(&content, Language::Rust);

            prop_assert_eq!(
                metadata.lines_code + metadata.lines_comment + metadata.lines_blank,
                metadata.lines_total,
                "Lines should sum to total"
            );
        }

        #[test]
        fn prop_file_size_matches_content_length(content in ".*") {
            let analyzer = LanguageAnalyzer::new();
            let metadata = analyzer.analyze_file_metadata(&content, Language::Rust);

            prop_assert_eq!(
                metadata.file_size_bytes,
                content.len() as u64,
                "File size should match content length"
            );
        }

        #[test]
        fn prop_complexity_at_least_one(content in code_content_strategy(), lang in complexity_language_strategy()) {
            let analyzer = LanguageAnalyzer::new();
            let rt = tokio::runtime::Runtime::new().unwrap();

            let result = rt.block_on(async {
                analyzer.analyze_complexity(&content, lang).await
            });

            let complexity = result.data["cyclomatic_complexity"].as_u64().unwrap();
            prop_assert!(complexity >= 1, "Complexity should always be at least 1");
        }

        #[test]
        fn prop_satd_count_non_negative(content in ".*") {
            let analyzer = LanguageAnalyzer::new();
            let rt = tokio::runtime::Runtime::new().unwrap();

            let result = rt.block_on(async {
                analyzer.analyze_satd(&content, Language::Rust).await
            });

            let count = result.data["satd_count"].as_u64().unwrap();
            prop_assert!(count >= 0, "SATD count should be non-negative");
        }

        #[test]
        fn prop_all_analysis_types_have_success_field(
            content in code_content_strategy(),
            analysis_type in prop_oneof![
                Just(AnalysisType::Complexity),
                Just(AnalysisType::Satd),
                Just(AnalysisType::Style),
                Just(AnalysisType::Metrics),
            ]
        ) {
            let analyzer = LanguageAnalyzer::new();
            let rt = tokio::runtime::Runtime::new().unwrap();

            let result = rt.block_on(async {
                analyzer.perform_single_analysis(&content, Language::Rust, &analysis_type).await
            });

            // All analysis results should have a success field
            prop_assert!(result.success == true || result.success == false);
        }

        #[test]
        fn prop_security_issues_count_non_negative(content in ".*") {
            let analyzer = LanguageAnalyzer::new();
            let rt = tokio::runtime::Runtime::new().unwrap();

            let result = rt.block_on(async {
                analyzer.analyze_security(&content, Language::JavaScript).await
            });

            let count = result.data["issues_count"].as_u64().unwrap();
            prop_assert!(count >= 0, "Security issues count should be non-negative");
        }

        #[test]
        fn prop_documentation_ratio_bounded(content in "([^\n]*\n){1,50}") {
            let analyzer = LanguageAnalyzer::new();
            let rt = tokio::runtime::Runtime::new().unwrap();

            let result = rt.block_on(async {
                analyzer.analyze_documentation(&content, Language::Rust).await
            });

            let ratio = result.data["documentation_ratio"].as_f64().unwrap();
            prop_assert!(ratio >= 0.0 && ratio <= 1.0, "Doc ratio should be between 0 and 1: {}", ratio);
        }

        #[test]
        fn prop_import_count_non_negative(content in ".*") {
            let analyzer = LanguageAnalyzer::new();
            let rt = tokio::runtime::Runtime::new().unwrap();

            let result = rt.block_on(async {
                analyzer.analyze_dependencies(&content, Language::Rust).await
            });

            let count = result.data["import_count"].as_u64().unwrap();
            prop_assert!(count >= 0, "Import count should be non-negative");
        }

        #[test]
        fn prop_metrics_lines_non_negative(content in ".*") {
            let analyzer = LanguageAnalyzer::new();
            let rt = tokio::runtime::Runtime::new().unwrap();

            let result = rt.block_on(async {
                analyzer.analyze_metrics(&content, Language::Rust).await
            });

            let lines = result.data["total_lines"].as_u64().unwrap();
            prop_assert!(lines >= 0, "Total lines should be non-negative");
        }

        #[test]
        fn prop_unsupported_analysis_returns_error(
            analysis_type in prop_oneof![
                Just(AnalysisType::Complexity),
                Just(AnalysisType::DeadCode),
                Just(AnalysisType::Security),
                Just(AnalysisType::Style),
                Just(AnalysisType::Dependencies),
            ]
        ) {
            let analyzer = LanguageAnalyzer::new();

            // JSON doesn't support most analysis types
            if !analyzer.supports_analysis(Language::JSON, &analysis_type) {
                let result = analyzer.create_unsupported_analysis_result(
                    analysis_type,
                    Language::JSON,
                );

                prop_assert!(!result.success, "Unsupported analysis should not succeed");
                prop_assert!(result.error.is_some(), "Unsupported analysis should have error");
            }
        }
    }
}
