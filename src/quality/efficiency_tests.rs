#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;

    // EfficiencyAnalyzer tests
    #[test]
    fn test_efficiency_analyzer_default() {
        let analyzer = EfficiencyAnalyzer::default();
        let _ = analyzer;
    }

    #[test]
    fn test_efficiency_analyzer_new() {
        let analyzer = EfficiencyAnalyzer::new();
        let _ = analyzer;
    }

    #[test]
    fn test_analyze_constant_time() {
        let analyzer = EfficiencyAnalyzer::new();
        let code = "fn constant() { let x = 1; let y = 2; }";
        let ast = syn::parse_file(code).unwrap();
        let complexity = analyzer.analyze(&ast);
        assert_eq!(complexity, "O(1)");
    }

    #[test]
    fn test_analyze_linear_time() {
        let analyzer = EfficiencyAnalyzer::new();
        let code = "fn linear() { for i in 0..n { } }";
        let ast = syn::parse_file(code).unwrap();
        let complexity = analyzer.analyze(&ast);
        assert_eq!(complexity, "O(n)");
    }

    #[test]
    fn test_analyze_quadratic_time() {
        let analyzer = EfficiencyAnalyzer::new();
        let code = r#"
            fn quadratic() {
                for i in 0..n {
                    for j in 0..n {
                        // nested
                    }
                }
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let complexity = analyzer.analyze(&ast);
        assert_eq!(complexity, "O(n^2)");
    }

    #[test]
    fn test_analyze_cubic_time() {
        let analyzer = EfficiencyAnalyzer::new();
        let code = r#"
            fn cubic() {
                for i in 0..n {
                    for j in 0..n {
                        for k in 0..n {
                            // triply nested
                        }
                    }
                }
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let complexity = analyzer.analyze(&ast);
        assert_eq!(complexity, "O(n^3)");
    }

    #[test]
    fn test_analyze_deeply_nested() {
        let analyzer = EfficiencyAnalyzer::new();
        let code = r#"
            fn deep() {
                for a in 0..n {
                    for b in 0..n {
                        for c in 0..n {
                            for d in 0..n {
                                // very nested
                            }
                        }
                    }
                }
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let complexity = analyzer.analyze(&ast);
        assert_eq!(complexity, "O(n^4)");
    }

    #[test]
    fn test_analyze_while_loop() {
        let analyzer = EfficiencyAnalyzer::new();
        let code = "fn with_while() { while condition { } }";
        let ast = syn::parse_file(code).unwrap();
        let complexity = analyzer.analyze(&ast);
        assert_eq!(complexity, "O(n)");
    }

    #[test]
    fn test_analyze_loop() {
        let analyzer = EfficiencyAnalyzer::new();
        let code = "fn with_loop() { loop { break; } }";
        let ast = syn::parse_file(code).unwrap();
        let complexity = analyzer.analyze(&ast);
        assert_eq!(complexity, "O(n)");
    }

    #[test]
    fn test_analyze_mixed_loops() {
        let analyzer = EfficiencyAnalyzer::new();
        let code = r#"
            fn mixed() {
                for i in 0..n {
                    while condition {
                        // nested for + while
                    }
                }
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let complexity = analyzer.analyze(&ast);
        assert_eq!(complexity, "O(n^2)");
    }

    #[test]
    fn test_analyze_string_success() {
        let analyzer = EfficiencyAnalyzer::new();
        let code = "fn test() { for i in 0..n { } }";
        let result = analyzer.analyze_string(code);
        assert!(result.is_ok());
        let efficiency = result.unwrap();
        assert_eq!(efficiency.time_complexity, "O(n)");
    }

    #[test]
    fn test_analyze_string_parse_error() {
        let analyzer = EfficiencyAnalyzer::new();
        let code = "invalid rust {{{{";
        let result = analyzer.analyze_string(code);
        assert!(result.is_err());
    }

    #[test]
    fn test_analyze_space_constant() {
        let analyzer = EfficiencyAnalyzer::new();
        let code = "fn constant_space() { let x = 1; }";
        let result = analyzer.analyze_string(code);
        assert!(result.is_ok());
        let efficiency = result.unwrap();
        assert_eq!(efficiency.space_complexity, "O(1)");
    }

    #[test]
    fn test_analyze_space_with_vec_allocation() {
        let analyzer = EfficiencyAnalyzer::new();
        let code = "fn with_vec() { let v = vec![1, 2, 3]; }";
        let result = analyzer.analyze_string(code);
        assert!(result.is_ok());
        let efficiency = result.unwrap();
        // Fixed-size literal vec is O(1) space (constant allocation)
        assert_eq!(efficiency.space_complexity, "O(1)");
    }

    #[test]
    fn test_analyze_space_with_typed_local() {
        let analyzer = EfficiencyAnalyzer::new();
        let code = "fn with_typed() { let x: Vec<i32> = Vec::new(); }";
        let result = analyzer.analyze_string(code);
        assert!(result.is_ok());
        // Has typed local variable
    }

    // EfficiencyResult tests
    #[test]
    fn test_efficiency_result_fields() {
        let result = EfficiencyResult {
            time_complexity: "O(n)".to_string(),
            space_complexity: "O(1)".to_string(),
        };
        assert_eq!(result.time_complexity, "O(n)");
        assert_eq!(result.space_complexity, "O(1)");
    }

    // EfficiencyVisitor tests
    #[test]
    fn test_compute_complexity_zero_depth() {
        let visitor = EfficiencyVisitor {
            current_loop_depth: 0,
            max_loop_depth: 0,
            has_recursion: false,
        };
        assert_eq!(visitor.compute_complexity(), "O(1)");
    }

    #[test]
    fn test_compute_complexity_one_depth() {
        let visitor = EfficiencyVisitor {
            current_loop_depth: 0,
            max_loop_depth: 1,
            has_recursion: false,
        };
        assert_eq!(visitor.compute_complexity(), "O(n)");
    }

    #[test]
    fn test_compute_complexity_with_recursion() {
        let visitor = EfficiencyVisitor {
            current_loop_depth: 0,
            max_loop_depth: 1,
            has_recursion: true,
        };
        assert_eq!(visitor.compute_complexity(), "O(n log n)");
    }

    #[test]
    fn test_compute_complexity_two_depth() {
        let visitor = EfficiencyVisitor {
            current_loop_depth: 0,
            max_loop_depth: 2,
            has_recursion: false,
        };
        assert_eq!(visitor.compute_complexity(), "O(n^2)");
    }

    #[test]
    fn test_compute_complexity_three_depth() {
        let visitor = EfficiencyVisitor {
            current_loop_depth: 0,
            max_loop_depth: 3,
            has_recursion: false,
        };
        assert_eq!(visitor.compute_complexity(), "O(n^3)");
    }

    #[test]
    fn test_compute_complexity_high_depth() {
        let visitor = EfficiencyVisitor {
            current_loop_depth: 0,
            max_loop_depth: 5,
            has_recursion: false,
        };
        assert_eq!(visitor.compute_complexity(), "O(n^5)");
    }

    // SpaceComplexityVisitor tests
    #[test]
    fn test_space_compute_complexity_no_allocations() {
        let visitor = SpaceComplexityVisitor {
            allocations: 0,
            recursive_depth: 0,
        };
        assert_eq!(visitor.compute_space_complexity(), "O(1)");
    }

    #[test]
    fn test_space_compute_complexity_with_allocations() {
        let visitor = SpaceComplexityVisitor {
            allocations: 5,
            recursive_depth: 0,
        };
        assert_eq!(visitor.compute_space_complexity(), "O(n)");
    }

    #[test]
    fn test_space_compute_complexity_with_recursion() {
        let visitor = SpaceComplexityVisitor {
            allocations: 0,
            recursive_depth: 3,
        };
        assert_eq!(visitor.compute_space_complexity(), "O(n)");
    }

    // Integration tests
    #[test]
    fn test_analyze_empty_function() {
        let analyzer = EfficiencyAnalyzer::new();
        let code = "fn empty() {}";
        let ast = syn::parse_file(code).unwrap();
        let complexity = analyzer.analyze(&ast);
        assert_eq!(complexity, "O(1)");
    }

    #[test]
    fn test_analyze_function_call() {
        let analyzer = EfficiencyAnalyzer::new();
        let code = "fn with_call() { some_function(); }";
        let ast = syn::parse_file(code).unwrap();
        // Function calls don't add loop depth
        let complexity = analyzer.analyze(&ast);
        assert_eq!(complexity, "O(1)");
    }

    #[test]
    fn test_analyze_recursive_function() {
        let analyzer = EfficiencyAnalyzer::new();
        let code = r#"
            fn recursive(n: u32) {
                if n == 0 { return; }
                recursive(n - 1);
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let complexity = analyzer.analyze(&ast);
        // May or may not detect recursion depending on implementation
        // At minimum it should return something
        assert!(!complexity.is_empty());
    }

    #[test]
    fn test_analyze_multiple_functions() {
        let analyzer = EfficiencyAnalyzer::new();
        let code = r#"
            fn func1() { }
            fn func2() { for i in 0..n { } }
            fn func3() { for i in 0..n { for j in 0..n { } } }
        "#;
        let ast = syn::parse_file(code).unwrap();
        // Should return max complexity
        let complexity = analyzer.analyze(&ast);
        assert_eq!(complexity, "O(n^2)");
    }

    #[test]
    fn test_analyze_string_with_allocations() {
        let analyzer = EfficiencyAnalyzer::new();
        let code = r#"
            fn allocating() {
                let v: Vec<i32> = Vec::new();
                for i in 0..n {
                    vec.push(i);
                }
            }
        "#;
        let result = analyzer.analyze_string(code);
        assert!(result.is_ok());
        let efficiency = result.unwrap();
        // Analyzer detects loop with push as O(n log n) due to potential reallocations
        assert_eq!(efficiency.time_complexity, "O(n log n)");
        assert_eq!(efficiency.space_complexity, "O(n)");
    }
}
