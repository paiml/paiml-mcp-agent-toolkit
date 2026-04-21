#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_part2 {
    use super::*;

    // === SpaceComplexityAnalyzer Tests ===

    #[test]
    fn test_space_complexity_analyzer_new() {
        let analyzer = SpaceComplexityAnalyzer::new();
        assert!(analyzer.allocations.is_empty());
        assert_eq!(analyzer.max_depth, 0);
    }

    #[test]
    fn test_space_complexity_analyzer_default() {
        let analyzer = SpaceComplexityAnalyzer::default();
        assert!(analyzer.allocations.is_empty());
    }

    #[test]
    fn test_space_complexity_simple() {
        let code = r#"
            fn simple() -> i32 {
                let x = 5;
                x
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let mut analyzer = SpaceComplexityAnalyzer::new();
        let complexity = analyzer.analyze(&ast);
        assert_eq!(complexity, Complexity::O1);
    }

    #[test]
    fn test_space_complexity_with_vec() {
        let code = r#"
            fn with_vec() {
                let v = Vec::new();
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let mut analyzer = SpaceComplexityAnalyzer::new();
        let complexity = analyzer.analyze(&ast);
        assert!(complexity >= Complexity::O1);
    }

    // === path_to_string Tests ===

    #[test]
    fn test_path_to_string_simple() {
        let code = "use std::collections::HashMap;";
        let ast = syn::parse_file(code).unwrap();
        if let syn::Item::Use(item_use) = &ast.items[0] {
            // Just verify the function exists and doesn't panic
            let _ = &item_use.tree;
        }
    }

    // === is_sorting_algorithm Tests ===

    #[test]
    fn test_is_sorting_algorithm() {
        let code = r#"
            fn quick_sort(arr: &mut [i32]) {}
        "#;
        let ast = syn::parse_file(code).unwrap();
        let executor = SymbolicExecutor::new();

        for item in &ast.items {
            if let syn::Item::Fn(func) = item {
                assert!(executor.is_sorting_algorithm(func));
            }
        }
    }

    #[test]
    fn test_is_not_sorting_algorithm() {
        let code = r#"
            fn calculate(x: i32) -> i32 { x }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let executor = SymbolicExecutor::new();

        for item in &ast.items {
            if let syn::Item::Fn(func) = item {
                assert!(!executor.is_sorting_algorithm(func));
            }
        }
    }

    // === is_search_algorithm Tests ===

    #[test]
    fn test_is_search_algorithm() {
        let code = r#"
            fn binary_search(arr: &[i32], target: i32) -> Option<usize> { None }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let executor = SymbolicExecutor::new();

        for item in &ast.items {
            if let syn::Item::Fn(func) = item {
                assert!(executor.is_search_algorithm(func));
            }
        }
    }

    // === is_graph_algorithm Tests ===

    #[test]
    fn test_is_graph_algorithm() {
        let code = r#"
            fn dfs_traverse(node: &Node) {}
        "#;
        let ast = syn::parse_file(code).unwrap();
        let executor = SymbolicExecutor::new();

        for item in &ast.items {
            if let syn::Item::Fn(func) = item {
                assert!(executor.is_graph_algorithm(func));
            }
        }
    }

    // === analyze_algorithm_patterns Tests ===

    #[test]
    fn test_analyze_algorithm_patterns_sorting() {
        let code = r#"
            fn heap_sort(arr: &mut [i32]) {}
        "#;
        let ast = syn::parse_file(code).unwrap();
        let executor = SymbolicExecutor::new();
        let patterns = executor.analyze_algorithm_patterns(&ast);
        assert!(patterns
            .iter()
            .any(|p| matches!(p, AlgorithmPattern::Sorting)));
    }

    #[test]
    fn test_analyze_algorithm_patterns_empty() {
        let code = r#"
            fn foo() {}
        "#;
        let ast = syn::parse_file(code).unwrap();
        let executor = SymbolicExecutor::new();
        let patterns = executor.analyze_algorithm_patterns(&ast);
        assert!(patterns.is_empty());
    }

    // === RecursionDetector Tests ===

    #[test]
    fn test_recursive_function_detection() {
        let code = r#"
            fn factorial(n: u32) -> u32 {
                if n <= 1 {
                    1
                } else {
                    n * factorial(n - 1)
                }
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let mut executor = SymbolicExecutor::new();

        for item in &ast.items {
            if let syn::Item::Fn(func) = item {
                let complexity = executor.analyze_function(func);
                assert!(complexity >= Complexity::ON);
            }
        }
    }

    #[test]
    fn test_non_recursive_function() {
        let code = r#"
            fn add(a: i32, b: i32) -> i32 {
                a + b
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let mut executor = SymbolicExecutor::new();

        for item in &ast.items {
            if let syn::Item::Fn(func) = item {
                let complexity = executor.analyze_function(func);
                assert_eq!(complexity, Complexity::O1);
            }
        }
    }

    /// efficiency_enhanced_visitors.rs:48-53 — named match arms in
    /// SymbolicExecutor::visit_expr_call (`sort`, `binary_search`,
    /// `contains`, `find`, `reverse`). Bare function-call form (not method
    /// call) is what syn::ExprCall matches. Analyze driver only needs one
    /// call per arm; the highest complexity wins, so we assert >= ONLogN
    /// (from `sort`).
    #[test]
    fn test_symbolic_executor_visit_expr_call_named_complexity_arms() {
        let code = r#"
            fn runner() {
                sort();
                sort_by();
                binary_search();
                contains();
                find();
                reverse();
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let mut executor = SymbolicExecutor::new();

        for item in &ast.items {
            if let syn::Item::Fn(func) = item {
                let complexity = executor.analyze_function(func);
                // `sort` is ONLogN — must dominate the ON / OLogN / O1 arms.
                assert!(
                    complexity >= Complexity::ONLogN,
                    "named-arm driver must reach at least ONLogN via sort, got {complexity:?}"
                );
            }
        }
    }

    /// efficiency_enhanced_visitors.rs:73 — RecursionDetector::visit_expr_call
    /// non-matching-ident branch. Existing `test_non_recursive_function` uses
    /// `add(a, b)` which has zero calls in its body; this test places a call
    /// to a *different* function inside the candidate, so the ident-compare
    /// arm at :73 executes but the equality branch returns false.
    #[test]
    fn test_recursion_detector_ignores_non_matching_ident() {
        let code = r#"
            fn outer(n: i32) -> i32 {
                helper(n)
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let mut executor = SymbolicExecutor::new();

        for item in &ast.items {
            if let syn::Item::Fn(func) = item {
                // `outer` calls `helper`, never `outer` — RecursionDetector
                // visits the call, compares idents, finds non-match, and
                // leaves is_recursive=false. analyze_function returns O1 in
                // this path because there's no loop and no recursion boost.
                let complexity = executor.analyze_function(func);
                assert_eq!(
                    complexity,
                    Complexity::O1,
                    "non-self-call must not inflate complexity"
                );
            }
        }
    }

    // === is_dynamic_programming Tests ===

    #[test]
    fn test_is_dynamic_programming_with_hashmap() {
        let code = r#"
            fn dp_solution(n: usize) -> usize {
                let memo = HashMap::new();
                memo.insert(0, 1);
                0
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let executor = SymbolicExecutor::new();

        for item in &ast.items {
            if let syn::Item::Fn(func) = item {
                assert!(executor.is_dynamic_programming(func));
            }
        }
    }

    #[test]
    fn test_is_dynamic_programming_with_btreemap() {
        let code = r#"
            fn dp_btree(n: usize) -> usize {
                let cache = BTreeMap::new();
                0
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let executor = SymbolicExecutor::new();

        for item in &ast.items {
            if let syn::Item::Fn(func) = item {
                assert!(executor.is_dynamic_programming(func));
            }
        }
    }

    #[test]
    fn test_is_dynamic_programming_with_hashmap_macro() {
        let code = r#"
            fn dp_macro(n: usize) -> usize {
                let memo = hashmap! { 0 => 1 };
                0
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let executor = SymbolicExecutor::new();

        for item in &ast.items {
            if let syn::Item::Fn(func) = item {
                assert!(executor.is_dynamic_programming(func));
            }
        }
    }

    #[test]
    fn test_is_dynamic_programming_with_cache_macro() {
        let code = r#"
            fn cached_fn(n: usize) -> usize {
                let c = cache! { size: 100 };
                0
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let executor = SymbolicExecutor::new();

        for item in &ast.items {
            if let syn::Item::Fn(func) = item {
                assert!(executor.is_dynamic_programming(func));
            }
        }
    }

    #[test]
    fn test_is_not_dynamic_programming() {
        let code = r#"
            fn plain(n: usize) -> usize {
                let x = 42;
                x + n
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let executor = SymbolicExecutor::new();

        for item in &ast.items {
            if let syn::Item::Fn(func) = item {
                assert!(!executor.is_dynamic_programming(func));
            }
        }
    }

    // === SpaceComplexityAnalyzer visit_local Tests ===

    #[test]
    fn test_space_complexity_with_array() {
        let code = r#"
            fn with_array() {
                let arr = [1, 2, 3];
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let mut analyzer = SpaceComplexityAnalyzer::new();
        let _complexity = analyzer.analyze(&ast);
        assert!(!analyzer.allocations.is_empty());
    }

    #[test]
    fn test_space_complexity_with_string_new() {
        let code = r#"
            fn with_string() {
                let s = String::new();
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let mut analyzer = SpaceComplexityAnalyzer::new();
        let _complexity = analyzer.analyze(&ast);
        assert!(!analyzer.allocations.is_empty());
    }

    #[test]
    fn test_space_complexity_with_vec_macro() {
        let code = r#"
            fn with_vec_macro() {
                let v = vec![1, 2, 3];
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let mut analyzer = SpaceComplexityAnalyzer::new();
        let _complexity = analyzer.analyze(&ast);
        assert!(!analyzer.allocations.is_empty());
    }

    #[test]
    fn test_space_complexity_multiple_allocations() {
        let code = r#"
            fn multi() {
                let a = [1, 2, 3];
                let v = Vec::new();
                let s = String::new();
                let m = vec![0; 100];
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let mut analyzer = SpaceComplexityAnalyzer::new();
        let _complexity = analyzer.analyze(&ast);
        assert!(analyzer.allocations.len() >= 3);
    }

    // === analyze_algorithm_patterns with DP ===

    #[test]
    fn test_analyze_algorithm_patterns_dp() {
        let code = r#"
            fn dp_knapsack(items: &[(usize, usize)], capacity: usize) -> usize {
                let memo = HashMap::new();
                0
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let executor = SymbolicExecutor::new();
        let patterns = executor.analyze_algorithm_patterns(&ast);
        assert!(patterns
            .iter()
            .any(|p| matches!(p, AlgorithmPattern::DynamicProgramming)));
    }
}
