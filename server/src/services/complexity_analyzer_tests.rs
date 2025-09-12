//! TDD Tests for Accurate Complexity Analysis
//!
//! Sprint 63: Fixes complexity false positives using industry-standard algorithms
//! Following Toyota Way TDD principles with comprehensive coverage

#[cfg(test)]
mod accurate_complexity_tests {
    use crate::services::accurate_complexity_analyzer::AccurateComplexityAnalyzer;
    use std::fs;
    use tempfile::TempDir;

    /// Test cyclomatic complexity calculation matches industry standard
    #[tokio::test]
    async fn test_cyclomatic_complexity_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        // Function with if, else, match, loop = complexity 5
        fs::write(
            &test_file,
            r#"
            fn complex_function(x: i32) -> i32 {
                if x > 0 {                      // +1
                    if x > 10 {                 // +1
                        20
                    } else {                    
                        10
                    }
                } else if x < -10 {            // +1
                    match x {                   // +1 for match
                        -20 => 1,
                        -30 => 2,
                        _ => 3,
                    }
                } else {
                    for i in 0..5 {             // +1
                        println!("{}", i);
                    }
                    0
                }
            }
        "#,
        )
        .unwrap();

        let analyzer = AccurateComplexityAnalyzer::new();
        let result = analyzer.analyze_file(&test_file).await.unwrap();

        assert_eq!(result.functions.len(), 1);
        assert_eq!(
            result.functions[0].cyclomatic_complexity, 5,
            "Expected cyclomatic complexity of 5 (1 base + 4 decision points)"
        );
    }

    /// Test cognitive complexity with nesting weights
    #[tokio::test]
    async fn test_cognitive_complexity_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        // Nested structures increase cognitive complexity
        fs::write(
            &test_file,
            r#"
            fn nested_function(x: i32) -> i32 {
                if x > 0 {                      // +1 (nesting 0)
                    for i in 0..x {             // +2 (nesting 1)
                        if i % 2 == 0 {         // +3 (nesting 2)
                            if i > 5 {          // +4 (nesting 3)
                                return i;
                            }
                        }
                    }
                }
                0
            }
        "#,
        )
        .unwrap();

        let analyzer = AccurateComplexityAnalyzer::new();
        let result = analyzer.analyze_file(&test_file).await.unwrap();

        assert_eq!(result.functions.len(), 1);
        assert!(
            result.functions[0].cognitive_complexity >= 10,
            "Cognitive complexity should be ≥10 for deeply nested function"
        );
    }

    /// Test that simple functions have low complexity
    #[tokio::test]
    async fn test_simple_function_low_complexity() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        fs::write(
            &test_file,
            r#"
            fn simple_add(a: i32, b: i32) -> i32 {
                a + b
            }
            
            fn simple_multiply(x: i32) -> i32 {
                x * 2
            }
        "#,
        )
        .unwrap();

        let analyzer = AccurateComplexityAnalyzer::new();
        let result = analyzer.analyze_file(&test_file).await.unwrap();

        assert_eq!(result.functions.len(), 2);
        for func in &result.functions {
            assert_eq!(
                func.cyclomatic_complexity, 1,
                "Simple functions should have complexity of 1"
            );
            assert_eq!(
                func.cognitive_complexity, 0,
                "Simple functions should have cognitive complexity of 0"
            );
        }
    }

    /// Test excluding test files
    #[tokio::test]
    async fn test_exclude_test_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create test file
        let test_file = temp_dir.path().join("lib_test.rs");
        fs::write(
            &test_file,
            r#"
            #[test]
            fn test_something() {
                // Complex test code
                for i in 0..10 {
                    if i > 5 {
                        assert!(true);
                    }
                }
            }
        "#,
        )
        .unwrap();

        // Create non-test file
        let src_file = temp_dir.path().join("lib.rs");
        fs::write(
            &src_file,
            r#"
            pub fn actual_function() -> i32 {
                42
            }
        "#,
        )
        .unwrap();

        let analyzer = AccurateComplexityAnalyzer::new().exclude_tests(true);

        let project_result = analyzer.analyze_project(temp_dir.path()).await.unwrap();

        // Should only analyze lib.rs, not lib_test.rs
        assert_eq!(project_result.files_analyzed, 1, "Should exclude test file");
        assert!(project_result
            .file_metrics
            .iter()
            .any(|f| f.file_path.ends_with("lib.rs")));
        assert!(!project_result
            .file_metrics
            .iter()
            .any(|f| f.file_path.ends_with("lib_test.rs")));
    }

    /// Test annotation support for suppressing complexity warnings
    #[tokio::test]
    async fn test_annotation_support() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        fs::write(
            &test_file,
            r#"
            #[allow(complex_function)]
            fn intentionally_complex(x: i32) -> i32 {
                // Very complex function that should be ignored
                let mut result = 0;
                for i in 0..x {
                    if i % 2 == 0 {
                        for j in 0..i {
                            if j % 3 == 0 {
                                result += j;
                            }
                        }
                    }
                }
                result
            }
            
            fn normal_complex(x: i32) -> i32 {
                // This one should be reported
                let mut result = 0;
                for i in 0..x {
                    if i % 2 == 0 {
                        result += i;
                    }
                }
                result
            }
        "#,
        )
        .unwrap();

        let analyzer = AccurateComplexityAnalyzer::new().respect_annotations(true);
        let result = analyzer.analyze_file(&test_file).await.unwrap();

        // Should have both functions
        assert_eq!(result.functions.len(), 2);

        // Find annotated function
        let annotated = result
            .functions
            .iter()
            .find(|f| f.name.contains("intentionally_complex"))
            .unwrap();
        assert!(
            annotated.suppressed,
            "Annotated function should be marked as suppressed"
        );

        // Find normal function
        let normal = result
            .functions
            .iter()
            .find(|f| f.name.contains("normal_complex"))
            .unwrap();
        assert!(
            !normal.suppressed,
            "Normal function should not be suppressed"
        );
    }

    /// Test match expressions with multiple arms
    #[tokio::test]
    async fn test_match_complexity() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        fs::write(
            &test_file,
            r#"
            fn match_function(x: Option<i32>) -> i32 {
                match x {                       // +1 for match
                    Some(n) if n > 0 => n * 2, // +1 for guard
                    Some(n) if n < 0 => -n,    // +1 for guard
                    Some(0) => 0,
                    None => -1,
                }
            }
        "#,
        )
        .unwrap();

        let analyzer = AccurateComplexityAnalyzer::new();
        let result = analyzer.analyze_file(&test_file).await.unwrap();

        assert_eq!(
            result.functions[0].cyclomatic_complexity, 3,
            "Match with 2 guards should have complexity 3"
        );
    }

    /// Test boolean operators (&&, ||) add to complexity
    #[tokio::test]
    async fn test_boolean_operator_complexity() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        fs::write(
            &test_file,
            r#"
            fn boolean_logic(a: bool, b: bool, c: bool) -> bool {
                if a && b || c {    // +3 (if + && + ||)
                    true
                } else {
                    false
                }
            }
        "#,
        )
        .unwrap();

        let analyzer = AccurateComplexityAnalyzer::new();
        let result = analyzer.analyze_file(&test_file).await.unwrap();

        assert_eq!(
            result.functions[0].cyclomatic_complexity, 3,
            "Boolean operators should add to complexity"
        );
    }

    /// Test ? operator adds to complexity
    #[tokio::test]
    async fn test_question_mark_operator() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        fs::write(
            &test_file,
            r#"
            fn try_function() -> Result<i32, String> {
                let x = some_operation()?;  // +1
                let y = another_op()?;       // +1
                Ok(x + y)
            }
            
            fn some_operation() -> Result<i32, String> {
                Ok(42)
            }
            
            fn another_op() -> Result<i32, String> {
                Ok(10)
            }
        "#,
        )
        .unwrap();

        let analyzer = AccurateComplexityAnalyzer::new();
        let result = analyzer.analyze_file(&test_file).await.unwrap();

        let try_fn = result
            .functions
            .iter()
            .find(|f| f.name.contains("try_function"))
            .unwrap();
        assert_eq!(
            try_fn.cyclomatic_complexity, 3,
            "? operator should add 1 to complexity per use"
        );
    }

    /// Test while and loop constructs
    #[tokio::test]
    async fn test_loop_complexity() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        fs::write(
            &test_file,
            r#"
            fn loop_function(mut x: i32) -> i32 {
                while x > 0 {           // +1
                    x -= 1;
                }
                
                loop {                  // +1
                    if x < -10 {        // +1
                        break;
                    }
                    x -= 1;
                }
                
                x
            }
        "#,
        )
        .unwrap();

        let analyzer = AccurateComplexityAnalyzer::new();
        let result = analyzer.analyze_file(&test_file).await.unwrap();

        assert_eq!(
            result.functions[0].cyclomatic_complexity, 4,
            "while and loop should each add 1 to complexity"
        );
    }

    /// Test recursion detection for cognitive complexity
    #[tokio::test]
    async fn test_recursion_cognitive_complexity() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        fs::write(
            &test_file,
            r#"
            fn factorial(n: u32) -> u32 {
                if n <= 1 {
                    1
                } else {
                    n * factorial(n - 1)  // Recursive call adds cognitive weight
                }
            }
        "#,
        )
        .unwrap();

        let analyzer = AccurateComplexityAnalyzer::new();
        let result = analyzer.analyze_file(&test_file).await.unwrap();

        assert!(
            result.functions[0].cognitive_complexity >= 3,
            "Recursive functions should have higher cognitive complexity"
        );
    }
}
