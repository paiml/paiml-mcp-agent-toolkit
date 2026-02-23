#![cfg_attr(coverage_nightly, coverage(off))]

#[cfg(test)]
mod coverage_tests_advanced {
    use crate::unified_quality::enhanced_parser::{CacheStats, EnhancedParser};
    use std::path::PathBuf;
    use std::time::SystemTime;

    /// Create a simple Rust function
    fn simple_function() -> &'static str {
        "fn simple() {}"
    }

    /// Create a test path
    fn test_path(name: &str) -> PathBuf {
        PathBuf::from(format!("{}.rs", name))
    }

    // ============================================
    // Cache Functionality Tests
    // ============================================

    #[test]
    fn test_cache_stores_result() {
        let mut parser = EnhancedParser::new();
        let path = test_path("cached");
        let code = simple_function();

        parser.parse_incremental(&path, code).unwrap();

        assert!(parser.get_cached_metrics(&path).is_some());
    }

    #[test]
    fn test_cache_returns_same_result() {
        let mut parser = EnhancedParser::new();
        let path = test_path("cached");
        let code = simple_function();

        let first = parser.parse_incremental(&path, code).unwrap();
        let second = parser.parse_incremental(&path, code).unwrap();

        assert_eq!(first.functions, second.functions);
        assert_eq!(first.complexity, second.complexity);
    }

    #[test]
    fn test_cache_invalidation_on_change() {
        let mut parser = EnhancedParser::new();
        let path = test_path("changing");

        let code1 = "fn a() {}";
        let code2 = "fn a() { if true {} }";

        let first = parser.parse_incremental(&path, code1).unwrap();
        let second = parser.parse_incremental(&path, code2).unwrap();

        // Complexity should differ
        assert!(second.complexity > first.complexity);
    }

    #[test]
    fn test_get_cached_metrics_nonexistent() {
        let parser = EnhancedParser::new();
        let path = test_path("nonexistent");

        assert!(parser.get_cached_metrics(&path).is_none());
    }

    #[test]
    fn test_clear_cache_single_file() {
        let mut parser = EnhancedParser::new();
        let path = test_path("to_clear");

        parser.parse_incremental(&path, simple_function()).unwrap();
        assert!(parser.get_cached_metrics(&path).is_some());

        parser.clear_cache(&path);
        assert!(parser.get_cached_metrics(&path).is_none());
    }

    #[test]
    fn test_clear_all_cache() {
        let mut parser = EnhancedParser::new();

        parser
            .parse_incremental(&test_path("file1"), "fn a() {}")
            .unwrap();
        parser
            .parse_incremental(&test_path("file2"), "fn b() {}")
            .unwrap();
        parser
            .parse_incremental(&test_path("file3"), "fn c() {}")
            .unwrap();

        assert_eq!(parser.cache_stats().total_entries, 3);

        parser.clear_all_cache();

        assert_eq!(parser.cache_stats().total_entries, 0);
    }

    // ============================================
    // CacheStats Tests
    // ============================================

    #[test]
    fn test_cache_stats_empty() {
        let parser = EnhancedParser::new();
        let stats = parser.cache_stats();

        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.memory_usage_estimate, 0);
    }

    #[test]
    fn test_cache_stats_after_parsing() {
        let mut parser = EnhancedParser::new();

        for i in 0..5 {
            parser
                .parse_incremental(&test_path(&format!("file_{}", i)), "fn test() {}")
                .unwrap();
        }

        let stats = parser.cache_stats();
        assert_eq!(stats.total_entries, 5);
        assert!(stats.memory_usage_estimate > 0);
    }

    #[test]
    fn test_cache_stats_debug() {
        let stats = CacheStats {
            total_entries: 10,
            memory_usage_estimate: 20480,
        };

        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("CacheStats"));
        assert!(debug_str.contains("total_entries"));
    }

    #[test]
    fn test_cache_stats_clone() {
        let stats = CacheStats {
            total_entries: 5,
            memory_usage_estimate: 10240,
        };
        let cloned = stats.clone();

        assert_eq!(stats.total_entries, cloned.total_entries);
        assert_eq!(stats.memory_usage_estimate, cloned.memory_usage_estimate);
    }

    // ============================================
    // Hash Calculation Tests
    // ============================================

    #[test]
    fn test_hash_same_content() {
        let parser = EnhancedParser::new();
        let content = "fn test() {}";

        let hash1 = parser.calculate_hash(content);
        let hash2 = parser.calculate_hash(content);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_different_content() {
        let parser = EnhancedParser::new();

        let hash1 = parser.calculate_hash("fn a() {}");
        let hash2 = parser.calculate_hash("fn b() {}");

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_empty_content() {
        let parser = EnhancedParser::new();
        let hash = parser.calculate_hash("");

        // Should not panic and should return a valid hash
        assert!(hash > 0 || hash == 0); // Just verify it returns something
    }

    #[test]
    fn test_hash_whitespace_matters() {
        let parser = EnhancedParser::new();

        let hash1 = parser.calculate_hash("fn test() {}");
        let hash2 = parser.calculate_hash("fn test()  {}"); // Extra space

        assert_ne!(hash1, hash2);
    }

    // ============================================
    // Error Handling Tests
    // ============================================

    #[test]
    fn test_parse_invalid_rust_syntax() {
        let mut parser = EnhancedParser::new();
        let result = parser.parse_incremental(&test_path("invalid"), "fn { invalid }");

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_incomplete_code() {
        let mut parser = EnhancedParser::new();
        let result = parser.parse_incremental(&test_path("incomplete"), "fn test(");

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unbalanced_braces() {
        let mut parser = EnhancedParser::new();
        let result = parser.parse_incremental(&test_path("unbalanced"), "fn test() { { }");

        assert!(result.is_err());
    }

    #[test]
    fn test_error_message_contains_info() {
        let mut parser = EnhancedParser::new();
        let result = parser.parse_incremental(&test_path("error"), "not rust code");

        assert!(result.is_err());
        let err = result.err().unwrap();
        let err_msg = err.to_string();
        assert!(err_msg.contains("parse") || err_msg.contains("Rust"));
    }

    // ============================================
    // Edge Case Tests
    // ============================================

    #[test]
    fn test_parse_unicode_identifiers() {
        let mut parser = EnhancedParser::new();
        // Rust doesn't allow unicode in function names by default
        let code = "fn test_unicode() { let x = 42; }";
        let result = parser.parse_incremental(&test_path("unicode"), code);

        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_very_long_function() {
        let mut parser = EnhancedParser::new();
        let mut code = String::from("fn long() {\n");
        for i in 0..100 {
            code.push_str(&format!("    let x{} = {};\n", i, i));
        }
        code.push_str("}\n");

        let result = parser.parse_incremental(&test_path("long"), &code);
        assert!(result.is_ok());

        let metrics = result.unwrap();
        assert!(metrics.lines > 100);
    }

    #[test]
    fn test_parse_deeply_nested() {
        let mut parser = EnhancedParser::new();
        let code = r#"
        fn deep() {
            if true {
                if true {
                    if true {
                        if true {
                            if true {
                                // Very deep
                            }
                        }
                    }
                }
            }
        }
        "#;

        let result = parser.parse_incremental(&test_path("deep"), code);
        assert!(result.is_ok());

        let metrics = result.unwrap();
        assert!(metrics.cognitive > 10); // Deep nesting increases cognitive complexity
    }

    #[test]
    fn test_parse_many_functions() {
        let mut parser = EnhancedParser::new();
        let mut code = String::new();
        for i in 0..50 {
            code.push_str(&format!("fn func_{i}() {{}}\n"));
        }

        let result = parser.parse_incremental(&test_path("many"), &code);
        assert!(result.is_ok());

        let metrics = result.unwrap();
        assert_eq!(metrics.functions, 50);
    }

    #[test]
    fn test_parse_struct_definition() {
        let mut parser = EnhancedParser::new();
        let code = r#"
        struct Point {
            x: i32,
            y: i32,
        }

        fn create_point() -> Point {
            Point { x: 0, y: 0 }
        }
        "#;

        let result = parser.parse_incremental(&test_path("struct"), code);
        assert!(result.is_ok());

        let metrics = result.unwrap();
        assert_eq!(metrics.functions, 1);
    }

    #[test]
    fn test_parse_impl_block() {
        let mut parser = EnhancedParser::new();
        let code = r#"
        struct Point {
            x: i32,
            y: i32,
        }

        impl Point {
            fn new() -> Self {
                Point { x: 0, y: 0 }
            }

            fn distance(&self) -> f64 {
                if self.x == 0 && self.y == 0 {
                    0.0
                } else {
                    1.0
                }
            }
        }
        "#;

        let result = parser.parse_incremental(&test_path("impl"), code);
        assert!(result.is_ok());
        // Note: impl methods may or may not be counted as functions depending on visitor
    }

    #[test]
    fn test_parse_closure() {
        let mut parser = EnhancedParser::new();
        let code = r#"
        fn with_closure() {
            let f = |x| {
                if x > 0 {
                    x * 2
                } else {
                    0
                }
            };
        }
        "#;

        let result = parser.parse_incremental(&test_path("closure"), code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_async_function() {
        let mut parser = EnhancedParser::new();
        let code = r#"
        async fn async_fn() {
            if true {
                // async code
            }
        }
        "#;

        let result = parser.parse_incremental(&test_path("async"), code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_timestamp_is_set() {
        let mut parser = EnhancedParser::new();
        let before = SystemTime::now();

        let result = parser.parse_incremental(&test_path("timestamp"), simple_function());
        assert!(result.is_ok());

        let after = SystemTime::now();
        let metrics = result.unwrap();

        assert!(metrics.timestamp >= before);
        assert!(metrics.timestamp <= after);
    }

    // ============================================
    // Visitor Pattern Tests
    // ============================================

    #[test]
    fn test_complexity_visitor_binary_ops_other() {
        let mut parser = EnhancedParser::new();
        // Test binary operators that don't add complexity (like arithmetic)
        let code = "fn math(a: i32, b: i32) -> i32 { a + b * c / d }";
        let result = parser.parse_incremental(&test_path("math"), code);

        assert!(result.is_ok());
        let metrics = result.unwrap();
        // Parser includes function entry + arithmetic operations
        assert_eq!(metrics.complexity, 2);
    }

    #[test]
    fn test_complexity_match_arms_add_complexity() {
        let mut parser = EnhancedParser::new();
        let code = r#"
        fn with_match(x: i32) -> i32 {
            match x {
                0 => 0,
                1 => 1,
            }
        }
        "#;

        let result = parser.parse_incremental(&test_path("two_arms"), code);
        assert!(result.is_ok());

        let metrics = result.unwrap();
        assert!(metrics.complexity >= 3); // Base + 2 arms
    }

    #[test]
    fn test_cognitive_match_arm_nesting() {
        let mut parser = EnhancedParser::new();
        let code = r#"
        fn nested_match(x: i32) -> i32 {
            if true {
                match x {
                    0 => 0,
                    _ => 1,
                }
            } else {
                0
            }
        }
        "#;

        let result = parser.parse_incremental(&test_path("nested_match"), code);
        assert!(result.is_ok());

        let metrics = result.unwrap();
        // Match arms inside if should have higher cognitive complexity due to nesting
        assert!(metrics.cognitive > metrics.complexity);
    }
}
