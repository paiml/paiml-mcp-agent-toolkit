
        // Double dash comments
        assert!(analyzer.is_comment_line("-- comment", Language::SQL));
        assert!(analyzer.is_comment_line("-- comment", Language::Haskell));
    }

    #[test]
    fn test_is_comment_line_xml() {
        let analyzer = LanguageAnalyzer::new();

        // XML comments
        assert!(analyzer.is_comment_line("<!-- comment -->", Language::XML));
    }

    #[test]
    fn test_is_comment_line_non_comment() {
        let analyzer = LanguageAnalyzer::new();

        // Non-comment lines
        assert!(!analyzer.is_comment_line("let x = 5;", Language::Rust));
        assert!(!analyzer.is_comment_line("x = 5", Language::Python));
        assert!(!analyzer.is_comment_line("var x = 5;", Language::JavaScript));
    }

    #[test]
    fn test_is_comment_line_no_comment_style() {
        let analyzer = LanguageAnalyzer::new();

        // Languages with no comment style return false
        assert!(!analyzer.is_comment_line("anything", Language::Unknown));
        assert!(!analyzer.is_comment_line("# could be comment", Language::JSON));
    }

    // Comment Style Tests

    #[test]
    fn test_get_comment_style_c_style() {
        let analyzer = LanguageAnalyzer::new();

        assert_eq!(
            analyzer.get_comment_style(Language::Rust),
            CommentStyle::CStyle
        );
        assert_eq!(
            analyzer.get_comment_style(Language::C),
            CommentStyle::CStyle
        );
        assert_eq!(
            analyzer.get_comment_style(Language::Cpp),
            CommentStyle::CStyle
        );
        assert_eq!(
            analyzer.get_comment_style(Language::Go),
            CommentStyle::CStyle
        );
        assert_eq!(
            analyzer.get_comment_style(Language::Java),
            CommentStyle::CStyle
        );
        assert_eq!(
            analyzer.get_comment_style(Language::JavaScript),
            CommentStyle::CStyle
        );
        assert_eq!(
            analyzer.get_comment_style(Language::TypeScript),
            CommentStyle::CStyle
        );
        assert_eq!(
            analyzer.get_comment_style(Language::CSharp),
            CommentStyle::CStyle
        );
        assert_eq!(
            analyzer.get_comment_style(Language::Swift),
            CommentStyle::CStyle
        );
        assert_eq!(
            analyzer.get_comment_style(Language::Dart),
            CommentStyle::CStyle
        );
        assert_eq!(
            analyzer.get_comment_style(Language::Scala),
            CommentStyle::CStyle
        );
        assert_eq!(
            analyzer.get_comment_style(Language::Groovy),
            CommentStyle::CStyle
        );
        assert_eq!(
            analyzer.get_comment_style(Language::Kotlin),
            CommentStyle::CStyle
        );
    }

    #[test]
    fn test_get_comment_style_hash() {
        let analyzer = LanguageAnalyzer::new();

        assert_eq!(
            analyzer.get_comment_style(Language::Python),
            CommentStyle::Hash
        );
        assert_eq!(
            analyzer.get_comment_style(Language::Ruby),
            CommentStyle::Hash
        );
        assert_eq!(
            analyzer.get_comment_style(Language::Bash),
            CommentStyle::Hash
        );
        assert_eq!(
            analyzer.get_comment_style(Language::Zsh),
            CommentStyle::Hash
        );
        assert_eq!(
            analyzer.get_comment_style(Language::Fish),
            CommentStyle::Hash
        );
        assert_eq!(
            analyzer.get_comment_style(Language::Perl),
            CommentStyle::Hash
        );
        assert_eq!(analyzer.get_comment_style(Language::R), CommentStyle::Hash);
        assert_eq!(
            analyzer.get_comment_style(Language::YAML),
            CommentStyle::Hash
        );
        assert_eq!(
            analyzer.get_comment_style(Language::TOML),
            CommentStyle::Hash
        );
        assert_eq!(
            analyzer.get_comment_style(Language::Makefile),
            CommentStyle::Hash
        );
    }

    #[test]
    fn test_get_comment_style_other() {
        let analyzer = LanguageAnalyzer::new();

        assert_eq!(
            analyzer.get_comment_style(Language::Clojure),
            CommentStyle::Semicolon
        );
        assert_eq!(
            analyzer.get_comment_style(Language::Erlang),
            CommentStyle::Percent
        );
        assert_eq!(
            analyzer.get_comment_style(Language::Matlab),
            CommentStyle::Percent
        );
        assert_eq!(
            analyzer.get_comment_style(Language::SQL),
            CommentStyle::DoubleDash
        );
        assert_eq!(
            analyzer.get_comment_style(Language::Haskell),
            CommentStyle::DoubleDash
        );
        assert_eq!(analyzer.get_comment_style(Language::XML), CommentStyle::Xml);
    }

    #[test]
    fn test_get_comment_style_none() {
        let analyzer = LanguageAnalyzer::new();

        assert_eq!(
            analyzer.get_comment_style(Language::Unknown),
            CommentStyle::None
        );
        assert_eq!(
            analyzer.get_comment_style(Language::JSON),
            CommentStyle::None
        );
    }

    // File Metadata Analysis Tests

    #[test]
    fn test_analyze_file_metadata_rust() {
        let analyzer = LanguageAnalyzer::new();
        let content = sample_rust_code();

        let metadata = analyzer.analyze_file_metadata(content, Language::Rust);

        assert!(metadata.lines_total > 0);
        assert!(metadata.lines_code > 0);
        assert!(metadata.lines_comment > 0);
        assert!(metadata.lines_blank > 0);
        assert_eq!(metadata.file_size_bytes, content.len() as u64);
        assert_eq!(metadata.detected_language, Language::Rust);
        assert_eq!(metadata.confidence, 1.0);

        // Sum should equal total
        assert_eq!(
            metadata.lines_code + metadata.lines_comment + metadata.lines_blank,
            metadata.lines_total
        );
    }

    #[test]
    fn test_analyze_file_metadata_python() {
        let analyzer = LanguageAnalyzer::new();
        let content = sample_python_code();

        let metadata = analyzer.analyze_file_metadata(content, Language::Python);

        assert!(metadata.lines_total > 0);
        assert!(metadata.lines_code > 0);
        assert!(metadata.lines_comment > 0);
        assert!(metadata.lines_blank > 0);
    }

    #[test]
    fn test_analyze_file_metadata_empty() {
        let analyzer = LanguageAnalyzer::new();
        let content = "";

        let metadata = analyzer.analyze_file_metadata(content, Language::Rust);

        assert_eq!(metadata.lines_total, 0);
        assert_eq!(metadata.lines_code, 0);
        assert_eq!(metadata.lines_comment, 0);
        assert_eq!(metadata.lines_blank, 0);
        assert_eq!(metadata.file_size_bytes, 0);
    }

    #[test]
    fn test_analyze_file_metadata_only_blank() {
        let analyzer = LanguageAnalyzer::new();
        let content = "\n\n\n";

        let metadata = analyzer.analyze_file_metadata(content, Language::Rust);

        assert_eq!(metadata.lines_total, 3);
        assert_eq!(metadata.lines_code, 0);
        assert_eq!(metadata.lines_comment, 0);
        assert_eq!(metadata.lines_blank, 3);
    }

    #[test]
    fn test_analyze_file_metadata_only_comments() {
        let analyzer = LanguageAnalyzer::new();
        let content = "// comment 1\n// comment 2\n// comment 3";

        let metadata = analyzer.analyze_file_metadata(content, Language::Rust);

        assert_eq!(metadata.lines_total, 3);
        assert_eq!(metadata.lines_code, 0);
        assert_eq!(metadata.lines_comment, 3);
        assert_eq!(metadata.lines_blank, 0);
    }

    #[test]
    fn test_analyze_file_metadata_only_code() {
        let analyzer = LanguageAnalyzer::new();
        let content = "fn main() {\n    println!(\"hello\");\n}";

        let metadata = analyzer.analyze_file_metadata(content, Language::Rust);

        assert_eq!(metadata.lines_total, 3);
        assert_eq!(metadata.lines_code, 3);
        assert_eq!(metadata.lines_comment, 0);
        assert_eq!(metadata.lines_blank, 0);
    }

    // Complexity Analysis Tests

    #[tokio::test]
    async fn test_analyze_complexity_rust() {
        let analyzer = LanguageAnalyzer::new();
        let content = sample_rust_code();

        let result = analyzer.analyze_complexity(content, Language::Rust).await;

        assert!(result.success);
        assert!(matches!(result.analysis_type, AnalysisType::Complexity));
        assert!(result.error.is_none());

        // Should detect if, else, for, while, match keywords
        let complexity = result.data["cyclomatic_complexity"].as_u64().unwrap();
        assert!(complexity > 1);
    }

    #[tokio::test]
    async fn test_analyze_complexity_python() {
        let analyzer = LanguageAnalyzer::new();
        let content = sample_python_code();

        let result = analyzer.analyze_complexity(content, Language::Python).await;

        assert!(result.success);
        let complexity = result.data["cyclomatic_complexity"].as_u64().unwrap();
        assert!(complexity > 1);
    }

    #[tokio::test]
    async fn test_analyze_complexity_javascript() {
        let analyzer = LanguageAnalyzer::new();
        let content = sample_javascript_code();

        let result = analyzer
            .analyze_complexity(content, Language::JavaScript)
            .await;

        assert!(result.success);
        let complexity = result.data["cyclomatic_complexity"].as_u64().unwrap();
        assert!(complexity > 1);
    }

    #[tokio::test]
    async fn test_analyze_complexity_empty() {
        let analyzer = LanguageAnalyzer::new();

        let result = analyzer.analyze_complexity("", Language::Rust).await;

        assert!(result.success);
        let complexity = result.data["cyclomatic_complexity"].as_u64().unwrap();
        assert_eq!(complexity, 1); // Base complexity
    }

    // Complexity Keywords Tests

    #[test]
    fn test_get_complexity_keywords_rust() {
        let analyzer = LanguageAnalyzer::new();
        let keywords = analyzer.get_complexity_keywords(Language::Rust);

        assert!(keywords.contains(&"if"));
        assert!(keywords.contains(&"else"));
        assert!(keywords.contains(&"for"));
        assert!(keywords.contains(&"while"));
        assert!(keywords.contains(&"match"));
    }

    #[test]
    fn test_get_complexity_keywords_python() {
        let analyzer = LanguageAnalyzer::new();
        let keywords = analyzer.get_complexity_keywords(Language::Python);

        assert!(keywords.contains(&"if"));
        assert!(keywords.contains(&"elif"));
        assert!(keywords.contains(&"else"));
        assert!(keywords.contains(&"for"));
        assert!(keywords.contains(&"while"));
        assert!(keywords.contains(&"try"));
        assert!(keywords.contains(&"except"));
    }

    #[test]
    fn test_get_complexity_keywords_javascript() {
        let analyzer = LanguageAnalyzer::new();
        let keywords = analyzer.get_complexity_keywords(Language::JavaScript);

        assert!(keywords.contains(&"if"));
        assert!(keywords.contains(&"switch"));
        assert!(keywords.contains(&"try"));
        assert!(keywords.contains(&"catch"));
    }

    #[test]
    fn test_get_complexity_keywords_java() {
        let analyzer = LanguageAnalyzer::new();
        let keywords = analyzer.get_complexity_keywords(Language::Java);

        assert!(keywords.contains(&"if"));
        assert!(keywords.contains(&"when"));
        assert!(keywords.contains(&"switch"));
        assert!(keywords.contains(&"case"));
    }

    #[test]
    fn test_get_complexity_keywords_unknown() {
        let analyzer = LanguageAnalyzer::new();
        let keywords = analyzer.get_complexity_keywords(Language::Unknown);

        // Should have basic keywords
        assert!(keywords.contains(&"if"));
        assert!(keywords.contains(&"else"));
        assert!(keywords.contains(&"for"));
        assert!(keywords.contains(&"while"));
    }

    // Keyword Complexity Calculation Tests

    #[test]
    fn test_calculate_keyword_complexity_empty() {
        let analyzer = LanguageAnalyzer::new();
        let keywords = vec!["if", "else"];

        let complexity = analyzer.calculate_keyword_complexity("", &keywords);

        assert_eq!(complexity, 1); // Base complexity
    }

    #[test]
    fn test_calculate_keyword_complexity_with_keywords() {
        let analyzer = LanguageAnalyzer::new();
        let keywords = vec!["if", "else"];
        let content = "if (true) { } else { } if (false) { }";

        let complexity = analyzer.calculate_keyword_complexity(content, &keywords);

        // 1 (base) + 2 (if) + 1 (else) = 4
        assert_eq!(complexity, 4);
    }

    #[test]
    fn test_calculate_keyword_complexity_no_matches() {
        let analyzer = LanguageAnalyzer::new();
        let keywords = vec!["if", "else"];
        let content = "function main() { return 0; }";

        let complexity = analyzer.calculate_keyword_complexity(content, &keywords);

        assert_eq!(complexity, 1); // Base complexity only
    }

    // SATD Analysis Tests

    #[tokio::test]
    async fn test_analyze_satd_with_todos() {
        let analyzer = LanguageAnalyzer::new();
        let content = "// TODO: fix this\n// FIXME: broken\n// HACK: temporary";

        let result = analyzer.analyze_satd(content, Language::Rust).await;

        assert!(result.success);
        let count = result.data["satd_count"].as_u64().unwrap();
        assert_eq!(count, 3);

        let items = result.data["items"].as_array().unwrap();
        assert_eq!(items.len(), 3);
    }

    #[tokio::test]
    async fn test_analyze_satd_all_keywords() {
        let analyzer = LanguageAnalyzer::new();
        let content = "TODO\nFIXME\nHACK\nXXX\nBUG\nKLUDGE";

        let result = analyzer.analyze_satd(content, Language::Rust).await;

        let count = result.data["satd_count"].as_u64().unwrap();
        assert_eq!(count, 6);
    }

    #[tokio::test]
    async fn test_analyze_satd_case_insensitive() {
        let analyzer = LanguageAnalyzer::new();
        let content = "todo\nTODO\nToDo";

        let result = analyzer.analyze_satd(content, Language::Rust).await;

        let count = result.data["satd_count"].as_u64().unwrap();
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn test_analyze_satd_no_matches() {
        let analyzer = LanguageAnalyzer::new();
        let content = "fn main() {\n    println!(\"hello\");\n}";

        let result = analyzer.analyze_satd(content, Language::Rust).await;

        let count = result.data["satd_count"].as_u64().unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_analyze_satd_line_numbers() {
        let analyzer = LanguageAnalyzer::new();
        let content = "line 1\n// TODO: line 2\nline 3\n// FIXME: line 4";

        let result = analyzer.analyze_satd(content, Language::Rust).await;

        let items = result.data["items"].as_array().unwrap();
        assert_eq!(items[0]["line"].as_u64().unwrap(), 2);
        assert_eq!(items[1]["line"].as_u64().unwrap(), 4);
    }

    // Dead Code Analysis Tests

    #[tokio::test]
    async fn test_analyze_dead_code() {
        let analyzer = LanguageAnalyzer::new();
        let content = sample_rust_code();

        let result = analyzer.analyze_dead_code(content, Language::Rust).await;

        assert!(result.success);
        assert!(result.error.is_none());
        // Note: simplified implementation returns false for dead_code_detected
        assert_eq!(result.data["dead_code_detected"].as_bool().unwrap(), false);
    }

    // Security Analysis Tests

    #[tokio::test]
    async fn test_analyze_security_javascript_eval() {
