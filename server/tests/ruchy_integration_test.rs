//! TDD Tests for Ruchy Language Integration
//! 
//! Following Toyota Way TDD: RED -> GREEN -> REFACTOR
//! Tests written FIRST before implementation

use anyhow::Result;
use pmat::services::complexity::{ComplexityMetrics, FileComplexityMetrics, FunctionComplexity};
use pmat::services::languages::ruchy::analyze_ruchy_file_with_parser;
use std::io::Write;
use tempfile::NamedTempFile;

/// Helper to create temporary Ruchy file for testing
fn create_temp_ruchy_file(content: &str) -> Result<NamedTempFile> {
    let mut file = NamedTempFile::new()?;
    file.write_all(content.as_bytes())?;
    Ok(file)
}

#[cfg(feature = "ruchy-ast")]
mod ruchy_parser_tests {
    use super::*;

    #[tokio::test]
    async fn test_ruchy_parser_integration_simple_function() {
        // RED: Test should fail because analyze_ruchy_file_with_parser doesn't exist yet
        let ruchy_code = r#"
fun hello() -> String {
    "Hello, World!"
}
"#;
        
        let temp_file = create_temp_ruchy_file(ruchy_code).unwrap();
        let result = analyze_ruchy_file_with_parser(temp_file.path()).await;
        
        assert!(result.is_ok());
        let metrics = result.unwrap();
        
        // Should detect one function
        assert_eq!(metrics.functions.len(), 1);
        assert_eq!(metrics.functions[0].name, "hello");
        assert_eq!(metrics.functions[0].metrics.cyclomatic, 1); // Base complexity
        assert!(metrics.functions[0].line_start > 0);
        assert!(metrics.functions[0].line_end > metrics.functions[0].line_start);
    }

    #[tokio::test]
    async fn test_ruchy_parser_integration_complex_function() {
        // RED: Test should fail - testing if-else complexity analysis
        let ruchy_code = r#"
fun classify_number(x: i32) -> String {
    if x < 0 {
        "negative"
    } else if x == 0 {
        "zero"
    } else {
        "positive"
    }
}
"#;
        
        let temp_file = create_temp_ruchy_file(ruchy_code).unwrap();
        let result = analyze_ruchy_file_with_parser(temp_file.path()).await;
        
        assert!(result.is_ok());
        let metrics = result.unwrap();
        
        // Should detect one function with increased complexity
        assert_eq!(metrics.functions.len(), 1);
        assert_eq!(metrics.functions[0].name, "classify_number");
        // Base (1) + if (1) + else if (1) = 3
        assert_eq!(metrics.functions[0].metrics.cyclomatic, 3);
        assert!(metrics.functions[0].metrics.cognitive >= 3);
    }

    #[tokio::test]
    async fn test_ruchy_parser_integration_match_expression() {
        // RED: Test should fail - testing match complexity
        let ruchy_code = r#"
fun match_example(x: i32) -> String {
    match x {
        0 => "zero",
        1 | 2 => "small",
        n if n > 10 => "large", 
        _ => "other"
    }
}
"#;
        
        let temp_file = create_temp_ruchy_file(ruchy_code).unwrap();
        let result = analyze_ruchy_file_with_parser(temp_file.path()).await;
        
        assert!(result.is_ok());
        let metrics = result.unwrap();
        
        // Should detect match complexity correctly
        assert_eq!(metrics.functions.len(), 1);
        assert_eq!(metrics.functions[0].name, "match_example");
        // Base (1) + 4 match arms (4) = 5
        assert_eq!(metrics.functions[0].metrics.cyclomatic, 5);
        assert!(metrics.functions[0].metrics.cognitive >= 5);
    }

    #[tokio::test]
    async fn test_ruchy_parser_integration_loops() {
        // RED: Test should fail - testing loop complexity
        let ruchy_code = r#"
fun loop_example() -> i32 {
    let mut sum = 0;
    for i in 0..10 {
        if i % 2 == 0 {
            sum += i;
        }
    }
    while sum < 100 {
        sum *= 2;
    }
    sum
}
"#;
        
        let temp_file = create_temp_ruchy_file(ruchy_code).unwrap();
        let result = analyze_ruchy_file_with_parser(temp_file.path()).await;
        
        assert!(result.is_ok());
        let metrics = result.unwrap();
        
        // Should detect nested loop complexity
        assert_eq!(metrics.functions.len(), 1);
        assert_eq!(metrics.functions[0].name, "loop_example");
        // Base (1) + for (1) + if (1) + while (1) = 4
        assert_eq!(metrics.functions[0].metrics.cyclomatic, 4);
        assert!(metrics.functions[0].metrics.cognitive >= 6); // Nesting penalty
    }

    #[tokio::test]
    async fn test_ruchy_parser_integration_multiple_functions() {
        // RED: Test should fail - testing multiple function detection
        let ruchy_code = r#"
fun add(x: i32, y: i32) -> i32 {
    x + y
}

fun fibonacci(n: i32) -> i32 {
    if n <= 1 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

fun main() {
    let result = fibonacci(10);
    println(f"Result: {result}");
}
"#;
        
        let temp_file = create_temp_ruchy_file(ruchy_code).unwrap();
        let result = analyze_ruchy_file_with_parser(temp_file.path()).await;
        
        assert!(result.is_ok());
        let metrics = result.unwrap();
        
        // Should detect all three functions
        assert_eq!(metrics.functions.len(), 3);
        
        // Verify function names are detected correctly
        let function_names: Vec<&str> = metrics.functions.iter().map(|f| f.name.as_str()).collect();
        assert!(function_names.contains(&"add"));
        assert!(function_names.contains(&"fibonacci"));
        assert!(function_names.contains(&"main"));
        
        // Verify complexity calculations
        let fibonacci_func = metrics.functions.iter().find(|f| f.name == "fibonacci").unwrap();
        assert!(fibonacci_func.metrics.cyclomatic >= 2); // if-else increases complexity
    }

    #[tokio::test]
    async fn test_ruchy_parser_integration_actor_model() {
        // RED: Test should fail - testing actor complexity
        let ruchy_code = r#"
actor Counter {
    count: i32,
    
    receive increment() {
        self.count += 1;
    }
    
    receive get() -> i32 {
        self.count
    }
    
    receive reset() {
        self.count = 0;
    }
}
"#;
        
        let temp_file = create_temp_ruchy_file(ruchy_code).unwrap();
        let result = analyze_ruchy_file_with_parser(temp_file.path()).await;
        
        assert!(result.is_ok());
        let metrics = result.unwrap();
        
        // Should detect actor handlers as functions
        assert!(metrics.functions.len() >= 3); // increment, get, reset
        
        // Check that actor patterns are recognized
        let handler_names: Vec<&str> = metrics.functions.iter().map(|f| f.name.as_str()).collect();
        assert!(handler_names.iter().any(|name| name.contains("increment")));
        assert!(handler_names.iter().any(|name| name.contains("get")));
        assert!(handler_names.iter().any(|name| name.contains("reset")));
    }

    #[tokio::test]
    async fn test_ruchy_parser_integration_syntax_error() {
        // RED: Test should fail - testing error handling
        let invalid_ruchy_code = r#"
fun broken_syntax( {
    // Missing closing parenthesis and function body
"#;
        
        let temp_file = create_temp_ruchy_file(invalid_ruchy_code).unwrap();
        let result = analyze_ruchy_file_with_parser(temp_file.path()).await;
        
        // Should return error for invalid syntax
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(!error_msg.is_empty());
        // Should contain parse error information
        assert!(error_msg.to_lowercase().contains("parse") || error_msg.to_lowercase().contains("syntax"));
    }

    #[tokio::test]
    async fn test_ruchy_parser_integration_empty_file() {
        // RED: Test should fail - testing empty file handling
        let empty_ruchy_code = "";
        
        let temp_file = create_temp_ruchy_file(empty_ruchy_code).unwrap();
        let result = analyze_ruchy_file_with_parser(temp_file.path()).await;
        
        // Empty file should return valid metrics with no functions
        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.functions.len(), 0);
        assert_eq!(metrics.total_complexity.cyclomatic, 0);
        assert_eq!(metrics.total_complexity.lines, 0);
    }

    #[tokio::test]
    async fn test_ruchy_parser_integration_pipeline_operators() {
        // RED: Test should fail - testing pipeline complexity
        let ruchy_code = r#"
fun process_data(data: [i32]) -> [i32] {
    data
        |> filter(|x| x > 0)
        |> map(|x| x * 2) 
        |> filter(|x| x < 100)
        |> sort()
}
"#;
        
        let temp_file = create_temp_ruchy_file(ruchy_code).unwrap();
        let result = analyze_ruchy_file_with_parser(temp_file.path()).await;
        
        assert!(result.is_ok());
        let metrics = result.unwrap();
        
        // Should detect pipeline function correctly
        assert_eq!(metrics.functions.len(), 1);
        assert_eq!(metrics.functions[0].name, "process_data");
        // Pipeline operations might contribute to cognitive complexity
        assert!(metrics.functions[0].metrics.cognitive >= 1);
    }

    #[tokio::test]
    async fn test_ruchy_parser_integration_generic_functions() {
        // RED: Test should fail - testing generic function complexity
        let ruchy_code = r#"
fun identity<T>(x: T) -> T {
    x
}

fun map_option<T, U>(opt: Option<T>, f: fun(T) -> U) -> Option<U> {
    match opt {
        Some(value) => Some(f(value)),
        None => None
    }
}
"#;
        
        let temp_file = create_temp_ruchy_file(ruchy_code).unwrap();
        let result = analyze_ruchy_file_with_parser(temp_file.path()).await;
        
        assert!(result.is_ok());
        let metrics = result.unwrap();
        
        // Should detect both generic functions
        assert_eq!(metrics.functions.len(), 2);
        
        let function_names: Vec<&str> = metrics.functions.iter().map(|f| f.name.as_str()).collect();
        assert!(function_names.contains(&"identity"));
        assert!(function_names.contains(&"map_option"));
        
        // map_option should have higher complexity due to match
        let map_option_func = metrics.functions.iter().find(|f| f.name == "map_option").unwrap();
        assert!(map_option_func.metrics.cyclomatic >= 2); // match arms
    }
}

#[cfg(not(feature = "ruchy-ast"))]
mod ruchy_fallback_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_ruchy_fallback_still_works() {
        // When ruchy-ast feature is disabled, should still work with existing implementation
        let ruchy_code = r#"
fun hello() -> String {
    "Hello, World!"
}
"#;
        
        let temp_file = create_temp_ruchy_file(ruchy_code).unwrap();
        // Use the existing heuristic-based analyzer
        let result = pmat::services::languages::ruchy::analyze_ruchy_file(temp_file.path()).await;
        
        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.functions.len() >= 1); // Should detect at least one function
    }
}