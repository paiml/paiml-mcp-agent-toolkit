#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complexity_ordering() {
        assert!(Complexity::O1 < Complexity::OLogN);
        assert!(Complexity::OLogN < Complexity::ON);
        assert!(Complexity::ON < Complexity::ONLogN);
        assert!(Complexity::ONLogN < Complexity::ON2);
        assert!(Complexity::ON2 < Complexity::ON3);
        assert!(Complexity::ON3 < Complexity::OExp);
    }

    #[test]
    fn test_complexity_combination() {
        assert_eq!(Complexity::ON.combine(&Complexity::ON), Complexity::ON2);
        assert_eq!(Complexity::O1.combine(&Complexity::ON), Complexity::ON);
        assert_eq!(
            Complexity::ON.combine(&Complexity::OLogN),
            Complexity::ONLogN
        );
    }

    #[test]
    fn test_symbolic_execution() {
        let code = r#"
            fn bubble_sort(arr: &mut [i32]) {
                for i in 0..arr.len() {
                    for j in 0..arr.len() - 1 {
                        if arr[j] > arr[j + 1] {
                            arr.swap(j, j + 1);
                        }
                    }
                }
            }
        "#;

        let ast = syn::parse_file(code).unwrap();
        let mut executor = SymbolicExecutor::new();

        for item in &ast.items {
            if let syn::Item::Fn(func) = item {
                let complexity = executor.analyze_function(func);
                assert_eq!(complexity, Complexity::ON2);
            }
        }
    }

    // === Complexity Enum Tests ===

    #[test]
    fn test_complexity_display_o1() {
        assert_eq!(format!("{}", Complexity::O1), "O(1)");
    }

    #[test]
    fn test_complexity_display_ologn() {
        assert_eq!(format!("{}", Complexity::OLogN), "O(log n)");
    }

    #[test]
    fn test_complexity_display_on() {
        assert_eq!(format!("{}", Complexity::ON), "O(n)");
    }

    #[test]
    fn test_complexity_display_onlogn() {
        assert_eq!(format!("{}", Complexity::ONLogN), "O(n log n)");
    }

    #[test]
    fn test_complexity_display_on2() {
        assert_eq!(format!("{}", Complexity::ON2), "O(n^2)");
    }

    #[test]
    fn test_complexity_display_on3() {
        assert_eq!(format!("{}", Complexity::ON3), "O(n^3)");
    }

    #[test]
    fn test_complexity_display_oexp() {
        assert_eq!(format!("{}", Complexity::OExp), "O(2^n)");
    }

    #[test]
    fn test_complexity_display_ofactorial() {
        assert_eq!(format!("{}", Complexity::OFactorial), "O(n!)");
    }

    #[test]
    fn test_complexity_combine_with_o1() {
        assert_eq!(Complexity::O1.combine(&Complexity::ON), Complexity::ON);
        assert_eq!(Complexity::ON.combine(&Complexity::O1), Complexity::ON);
    }

    #[test]
    fn test_complexity_combine_logn() {
        assert_eq!(
            Complexity::OLogN.combine(&Complexity::OLogN),
            Complexity::ON
        );
    }

    #[test]
    fn test_complexity_combine_on_on2() {
        assert_eq!(Complexity::ON.combine(&Complexity::ON2), Complexity::ON3);
        assert_eq!(Complexity::ON2.combine(&Complexity::ON), Complexity::ON3);
    }

    #[test]
    fn test_complexity_combine_on2_on2() {
        assert_eq!(Complexity::ON2.combine(&Complexity::ON2), Complexity::ON3);
    }

    #[test]
    fn test_complexity_max() {
        // Using Ord::max which takes values by value
        assert_eq!(Complexity::O1.max(Complexity::ON), Complexity::ON);
        assert_eq!(Complexity::ON.max(Complexity::O1), Complexity::ON);
        assert_eq!(Complexity::ON2.max(Complexity::ON), Complexity::ON2);
    }

    #[test]
    fn test_complexity_clone() {
        let c = Complexity::ONLogN;
        let cloned = c.clone();
        assert_eq!(c, cloned);
    }

    #[test]
    fn test_complexity_eq() {
        assert_eq!(Complexity::O1, Complexity::O1);
        assert_ne!(Complexity::O1, Complexity::ON);
    }

    #[test]
    fn test_complexity_ord() {
        assert!(Complexity::O1 < Complexity::OExp);
        assert!(Complexity::OExp > Complexity::O1);
    }

    // === SymbolicExecutor Tests ===

    #[test]
    fn test_symbolic_executor_new() {
        let executor = SymbolicExecutor::new();
        assert!(executor.loop_depths.is_empty());
        assert_eq!(executor.recursive_depth, 0);
    }

    #[test]
    fn test_symbolic_executor_default() {
        let executor = SymbolicExecutor::default();
        assert!(executor.loop_depths.is_empty());
    }

    #[test]
    fn test_symbolic_executor_simple_function() {
        let code = r#"
            fn simple() -> i32 {
                42
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

    #[test]
    fn test_symbolic_executor_single_loop() {
        let code = r#"
            fn single_loop(arr: &[i32]) -> i32 {
                let mut sum = 0;
                for x in arr.iter() {
                    sum += x;
                }
                sum
            }
        "#;
        let ast = syn::parse_file(code).unwrap();
        let mut executor = SymbolicExecutor::new();

        for item in &ast.items {
            if let syn::Item::Fn(func) = item {
                let complexity = executor.analyze_function(func);
                assert_eq!(complexity, Complexity::ON);
            }
        }
    }

    #[test]
    fn test_symbolic_executor_while_loop() {
        let code = r#"
            fn with_while(n: i32) -> i32 {
                let mut i = 0;
                while i < n {
                    i += 1;
                }
                i
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

    // === AlgorithmPattern Tests ===

    #[test]
    fn test_algorithm_pattern_debug() {
        let patterns = vec![
            AlgorithmPattern::Sorting,
            AlgorithmPattern::Search,
            AlgorithmPattern::Graph,
            AlgorithmPattern::DynamicProgramming,
            AlgorithmPattern::Greedy,
            AlgorithmPattern::DivideAndConquer,
            AlgorithmPattern::Backtracking,
        ];
        for pattern in patterns {
            let debug = format!("{:?}", pattern);
            assert!(!debug.is_empty());
        }
    }

    #[test]
    fn test_algorithm_pattern_clone() {
        let pattern = AlgorithmPattern::Sorting;
        let cloned = pattern.clone();
        assert_eq!(format!("{:?}", pattern), format!("{:?}", cloned));
    }
}
