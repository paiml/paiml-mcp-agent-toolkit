#![cfg_attr(coverage_nightly, coverage(off))]

#[cfg(test)]
mod coverage_tests_core {
    use crate::unified_quality::enhanced_parser::{CachedSyntax, EnhancedParser};
    use crate::unified_quality::metrics::Metrics;
    use std::path::PathBuf;
    use std::time::SystemTime;

    // ============================================
    // Test Fixtures and Helpers
    // ============================================

    /// Create a simple Rust function
    fn simple_function() -> &'static str {
        "fn simple() {}"
    }

    /// Create a function with if statement
    fn function_with_if() -> &'static str {
        r#"
        fn with_if(x: i32) {
            if x > 0 {
                println!("positive");
            }
        }
        "#
    }

    /// Create a function with multiple control flow structures
    fn complex_function() -> &'static str {
        r#"
        fn complex(x: i32, y: i32) -> i32 {
            if x > 0 {
                for i in 0..10 {
                    if i % 2 == 0 {
                        while y > 0 {
                            return x + y;
                        }
                    }
                }
            }
            match x {
                0 => 0,
                1 => 1,
                _ => 2,
            }
        }
        "#
    }

    /// Create code with SATD comments
    fn code_with_satd() -> &'static str {
        r#"
        fn needs_work() {
            // TODO: implement this function
            // FIXME: this is broken
            // HACK: temporary workaround
            // XXX: needs review
            // BUG: known issue here
        }
        "#
    }

    /// Create code with logical operators
    fn code_with_logical_ops() -> &'static str {
        r#"
        fn check_conditions(a: bool, b: bool, c: bool) -> bool {
            if a && b || c && !a {
                true
            } else {
                false
            }
        }
        "#
    }

    /// Create code with multiple functions
    fn multiple_functions() -> &'static str {
        r#"
        fn first() {}
        fn second() {}
        fn third() {
            if true {}
        }
        "#
    }

    /// Create a test path
    fn test_path(name: &str) -> PathBuf {
        PathBuf::from(format!("{}.rs", name))
    }

    // ============================================
    // EnhancedParser Creation Tests
    // ============================================

    #[test]
    fn test_parser_new() {
        let parser = EnhancedParser::new();
        assert_eq!(parser.cache_stats().total_entries, 0);
    }

    #[test]
    fn test_parser_default() {
        let parser = EnhancedParser::default();
        assert_eq!(parser.cache_stats().total_entries, 0);
    }

    #[test]
    fn test_parser_default_equals_new() {
        let parser1 = EnhancedParser::new();
        let parser2 = EnhancedParser::default();

        assert_eq!(
            parser1.cache_stats().total_entries,
            parser2.cache_stats().total_entries
        );
    }

    // ============================================
    // parse_incremental Basic Tests
    // ============================================

    #[test]
    fn test_parse_simple_function() {
        let mut parser = EnhancedParser::new();
        let result = parser.parse_incremental(&test_path("simple"), simple_function());

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.functions, 1);
        assert!(metrics.complexity >= 1);
    }

    #[test]
    fn test_parse_function_with_if() {
        let mut parser = EnhancedParser::new();
        let result = parser.parse_incremental(&test_path("with_if"), function_with_if());

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.functions, 1);
        assert!(metrics.complexity >= 2); // Base + if
    }

    #[test]
    fn test_parse_complex_function() {
        let mut parser = EnhancedParser::new();
        let result = parser.parse_incremental(&test_path("complex"), complex_function());

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.functions, 1);
        assert!(metrics.complexity > 5); // Multiple control structures
        assert!(metrics.cognitive >= metrics.complexity); // Nesting adds cognitive complexity
    }

    #[test]
    fn test_parse_multiple_functions() {
        let mut parser = EnhancedParser::new();
        let result = parser.parse_incremental(&test_path("multi"), multiple_functions());

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.functions, 3);
    }

    #[test]
    fn test_parse_empty_file() {
        let mut parser = EnhancedParser::new();
        let result = parser.parse_incremental(&test_path("empty"), "");

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.functions, 0);
    }

    // ============================================
    // SATD Detection Tests
    // ============================================

    #[test]
    fn test_satd_detection_all_types() {
        let mut parser = EnhancedParser::new();
        let result = parser.parse_incremental(&test_path("satd"), code_with_satd());

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.satd_count, 5); // TODO, FIXME, HACK, XXX, BUG
    }

    #[test]
    fn test_satd_detection_single_todo() {
        let mut parser = EnhancedParser::new();
        // Use valid Rust syntax with TODO comment on its own line
        let code = "fn test() {\n    // TODO: implement\n}";
        let result = parser.parse_incremental(&test_path("todo"), code);

        assert!(result.is_ok(), "Parse failed: {:?}", result.as_ref().err());
        let metrics = result.unwrap();
        assert_eq!(metrics.satd_count, 1);
    }

    #[test]
    fn test_satd_detection_no_satd() {
        let mut parser = EnhancedParser::new();
        let code = "fn clean() { /* Clean code */ }";
        let result = parser.parse_incremental(&test_path("clean"), code);

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.satd_count, 0);
    }

    #[test]
    fn test_satd_detection_multiple_same_type() {
        let mut parser = EnhancedParser::new();
        let code = r#"
        fn many_todos() {
            // TODO: first
            // TODO: second
            // TODO: third
        }
        "#;
        let result = parser.parse_incremental(&test_path("todos"), code);

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.satd_count, 3);
    }

    // ============================================
    // Complexity Calculation Tests
    // ============================================

    #[test]
    fn test_complexity_if_statement() {
        let mut parser = EnhancedParser::new();
        let code = "fn test() { if true {} }";
        let result = parser.parse_incremental(&test_path("if"), code);

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.complexity >= 2); // Base + if
    }

    #[test]
    fn test_complexity_for_loop() {
        let mut parser = EnhancedParser::new();
        let code = "fn test() { for i in 0..10 {} }";
        let result = parser.parse_incremental(&test_path("for"), code);

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.complexity >= 2); // Base + for
    }

    #[test]
    fn test_complexity_while_loop() {
        let mut parser = EnhancedParser::new();
        let code = "fn test() { while true {} }";
        let result = parser.parse_incremental(&test_path("while"), code);

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.complexity >= 2); // Base + while
    }

    #[test]
    fn test_complexity_loop() {
        let mut parser = EnhancedParser::new();
        let code = "fn test() { loop { break; } }";
        let result = parser.parse_incremental(&test_path("loop"), code);

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.complexity >= 2); // Base + loop
    }

    #[test]
    fn test_complexity_match() {
        let mut parser = EnhancedParser::new();
        let code = r#"
        fn test(x: i32) -> i32 {
            match x {
                0 => 0,
                1 => 1,
                2 => 2,
                _ => 3,
            }
        }
        "#;
        let result = parser.parse_incremental(&test_path("match"), code);

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.complexity >= 5); // Base + 4 match arms
    }

    #[test]
    fn test_complexity_logical_and() {
        let mut parser = EnhancedParser::new();
        let code = "fn test(a: bool, b: bool) -> bool { a && b }";
        let result = parser.parse_incremental(&test_path("and"), code);

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.complexity >= 2); // Base + &&
    }

    #[test]
    fn test_complexity_logical_or() {
        let mut parser = EnhancedParser::new();
        let code = "fn test(a: bool, b: bool) -> bool { a || b }";
        let result = parser.parse_incremental(&test_path("or"), code);

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.complexity >= 2); // Base + ||
    }

    #[test]
    fn test_complexity_multiple_logical_ops() {
        let mut parser = EnhancedParser::new();
        let result = parser.parse_incremental(&test_path("logical"), code_with_logical_ops());

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.complexity >= 4); // Base + if + logical ops
    }

    // ============================================
    // Cognitive Complexity Tests
    // ============================================

    #[test]
    fn test_cognitive_nested_if() {
        let mut parser = EnhancedParser::new();
        let code = r#"
        fn nested() {
            if true {
                if true {
                    if true {
                        // Deeply nested
                    }
                }
            }
        }
        "#;
        let result = parser.parse_incremental(&test_path("nested"), code);

        assert!(result.is_ok());
        let metrics = result.unwrap();
        // Cognitive should account for nesting
        assert!(metrics.cognitive >= 6); // 1 + 2 + 3 for nesting levels
    }

    #[test]
    fn test_cognitive_flat_structure() {
        let mut parser = EnhancedParser::new();
        let code = r#"
        fn flat() {
            if true {}
            if true {}
            if true {}
        }
        "#;
        let result = parser.parse_incremental(&test_path("flat"), code);

        assert!(result.is_ok());
        let metrics = result.unwrap();
        // Flat structures have lower cognitive complexity
        assert!(metrics.cognitive >= 3);
    }

    #[test]
    fn test_cognitive_vs_cyclomatic() {
        let mut parser = EnhancedParser::new();
        let result = parser.parse_incremental(&test_path("complex"), complex_function());

        assert!(result.is_ok());
        let metrics = result.unwrap();
        // For deeply nested code, cognitive should be >= cyclomatic
        assert!(metrics.cognitive >= metrics.complexity);
    }

    // ============================================
    // Line Counting Tests
    // ============================================

    #[test]
    fn test_line_count_single_line() {
        let mut parser = EnhancedParser::new();
        let code = "fn test() {}";
        let result = parser.parse_incremental(&test_path("single"), code);

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.lines, 1);
    }

    #[test]
    fn test_line_count_multiple_lines() {
        let mut parser = EnhancedParser::new();
        let code = "fn test() {\n    // line 2\n    // line 3\n}";
        let result = parser.parse_incremental(&test_path("multi_line"), code);

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.lines, 4);
    }

    #[test]
    fn test_line_count_empty_lines() {
        let mut parser = EnhancedParser::new();
        let code = "fn test() {\n\n\n}";
        let result = parser.parse_incremental(&test_path("empty_lines"), code);

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.lines, 4);
    }

    // ============================================
    // CachedSyntax Tests
    // ============================================

    #[test]
    fn test_cached_syntax_fields() {
        let cached = CachedSyntax {
            syntax_str: "AST representation".to_string(),
            content: "fn test() {}".to_string(),
            last_modified: SystemTime::now(),
            content_hash: 12345,
            metrics: None,
        };

        assert_eq!(cached.syntax_str, "AST representation");
        assert_eq!(cached.content, "fn test() {}");
        assert_eq!(cached.content_hash, 12345);
        assert!(cached.metrics.is_none());
    }

    #[test]
    fn test_cached_syntax_with_metrics() {
        let metrics = Metrics {
            complexity: 5,
            cognitive: 3,
            satd_count: 1,
            coverage: 0.8,
            lines: 10,
            functions: 2,
            timestamp: SystemTime::now(),
        };

        let cached = CachedSyntax {
            syntax_str: String::new(),
            content: String::new(),
            last_modified: SystemTime::now(),
            content_hash: 0,
            metrics: Some(metrics.clone()),
        };

        assert!(cached.metrics.is_some());
        let stored = cached.metrics.unwrap();
        assert_eq!(stored.complexity, 5);
    }
}
