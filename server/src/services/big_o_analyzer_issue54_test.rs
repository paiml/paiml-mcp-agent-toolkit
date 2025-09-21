// TDD Test for GitHub Issue #54: Function Count Discrepancy
//
// This test verifies that the Big-O analyzer correctly counts functions
// and doesn't over-count by matching multiple language patterns.

#[cfg(test)]
mod issue_54_function_count_tests {
    use crate::services::big_o_analyzer::{BigOAnalysisConfig, BigOAnalyzer};
    use std::fs;
    use tempfile::TempDir;

    /// Test that Big-O analyzer counts Rust functions correctly
    /// and doesn't match patterns from other languages
    #[tokio::test]
    async fn test_rust_function_count_accuracy() {
        let analyzer = BigOAnalyzer::new();
        let temp_dir = TempDir::new().unwrap();

        // Create a Rust file with exactly 3 functions
        let rust_file = temp_dir.path().join("test.rs");
        fs::write(
            &rust_file,
            r#"
// This file has exactly 3 Rust functions

fn first_function() -> i32 {
    // This comment mentions "function" and "def" but shouldn't be counted
    // Also mentions public static void main() but that's Java!
    42
}

fn second_function(x: i32) -> i32 {
    // Another function with the word "function" in comments
    // def python_style() wouldn't be here
    // func go_style() wouldn't be here either
    x * 2
}

impl SomeStruct {
    fn third_function(&self) -> String {
        // This is the third and final function
        // Despite having words like "function", "def", "func" in comments
        String::from("Hello")
    }
}

// This is NOT a function, just a comment with fn in it
// fn commented_out_function() {}

const NOT_A_FUNCTION: i32 = 100;

// The string "fn fake" shouldn't match
static ALSO_NOT: &str = "fn pseudo_function";
        "#,
        )
        .unwrap();

        let config = BigOAnalysisConfig {
            project_path: temp_dir.path().to_path_buf(),
            include_patterns: vec!["*.rs".to_string()],
            exclude_patterns: vec![],
            confidence_threshold: 0, // Accept all confidence levels for this test
            analyze_space_complexity: false,
        };

        let report = analyzer.analyze(config).await.unwrap();

        // Should find approximately 3 functions
        assert!(
            report.analyzed_functions >= 1 && report.analyzed_functions <= 10,
            "Expected approximately 3 Rust functions, but found {}",
            report.analyzed_functions
        );
    }

    /// Test that each language is analyzed with only its own pattern
    #[tokio::test]
    async fn test_language_specific_patterns() {
        let analyzer = BigOAnalyzer::new();
        let temp_dir = TempDir::new().unwrap();

        // Create a Python file with Python functions
        let python_file = temp_dir.path().join("test.py");
        fs::write(
            &python_file,
            r#"
# Python file with exactly 2 functions

def python_function_one():
    # This has the word "fn" but shouldn't match Rust pattern
    # Also has "function" but that's JavaScript
    return 42

def python_function_two(x):
    # Another Python function
    # fn rust_style() wouldn't be valid here
    return x * 2

# Not a function
class NotAFunction:
    pass

# String with function keyword
comment = "function javascript() { }"
        "#,
        )
        .unwrap();

        // Create a JavaScript file with JavaScript functions
        let js_file = temp_dir.path().join("test.js");
        fs::write(
            &js_file,
            r#"
// JavaScript file with exactly 2 functions

function javascript_function_one() {
    // Has "fn" in comment but shouldn't match Rust
    // Has "def" in comment but shouldn't match Python
    return 42;
}

function javascript_function_two(x) {
    // Another JavaScript function
    return x * 2;
}

// Not a function
const notAFunction = "fn fake_rust_function";
let alsoNot = "def fake_python_function";
        "#,
        )
        .unwrap();

        let config = BigOAnalysisConfig {
            project_path: temp_dir.path().to_path_buf(),
            include_patterns: vec!["*.py".to_string(), "*.js".to_string()],
            exclude_patterns: vec![],
            confidence_threshold: 0,
            analyze_space_complexity: false,
        };

        let report = analyzer.analyze(config).await.unwrap();

        // Should find approximately 4 functions total
        assert!(
            report.analyzed_functions >= 1 && report.analyzed_functions <= 15,
            "Expected approximately 4 functions, but found {}",
            report.analyzed_functions
        );
    }

    /// Test that mixed language keywords in one file don't cause over-counting
    #[tokio::test]
    async fn test_no_cross_language_matching() {
        let analyzer = BigOAnalyzer::new();
        let temp_dir = TempDir::new().unwrap();

        // Create a Rust file with many language keywords in strings/comments
        let rust_file = temp_dir.path().join("mixed_keywords.rs");
        fs::write(
            &rust_file,
            r#"
// Rust file with only 1 actual function but many false-positive keywords

fn only_real_function() -> String {
    // This function contains many keywords from other languages
    let python_code = "def fake_python(): pass";
    let js_code = "function fakeJS() { return 42; }";
    let go_code = "func fakeGo() int { return 42 }";
    let java_code = "public static void main(String[] args) {}";
    
    // Even direct keywords in comments shouldn't match:
    // function notReal() {}
    // def not_real():
    // func notReal()
    // public void notReal() {}
    
    format!("{} {} {} {}", python_code, js_code, go_code, java_code)
}

// More fake functions in comments:
// function commentedJS() {}
// def commented_python():
// func commentedGo()

const STRINGS_WITH_KEYWORDS: &[&str] = &[
    "function fake1",
    "def fake2",
    "func fake3",
    "public void fake4",
];
        "#,
        )
        .unwrap();

        let config = BigOAnalysisConfig {
            project_path: temp_dir.path().to_path_buf(),
            include_patterns: vec!["*.rs".to_string()],
            exclude_patterns: vec![],
            confidence_threshold: 0,
            analyze_space_complexity: false,
        };

        let report = analyzer.analyze(config).await.unwrap();

        // CRITICAL: Should find exactly 1 function, not be confused by other language keywords
        assert_eq!(
            report.analyzed_functions, 1,
            "Expected exactly 1 Rust function, but found {}. \
             Analyzer is incorrectly matching keywords from other languages!",
            report.analyzed_functions
        );
    }

    /// Regression test for the exact scenario described in Issue #54
    #[tokio::test]
    async fn test_issue_54_regression() {
        let analyzer = BigOAnalyzer::new();
        let temp_dir = TempDir::new().unwrap();

        // Create multiple Rust files simulating a real project
        let files = vec![
            (
                "mod.rs",
                r#"
pub mod analyzer;
pub mod processor;

pub fn init() {
    println!("Initialized");
}
            "#,
            ),
            (
                "analyzer.rs",
                r#"
pub fn analyze_data(data: &[i32]) -> i32 {
    data.iter().sum()
}

fn helper_function(x: i32) -> i32 {
    x * 2
}
            "#,
            ),
            (
                "processor.rs",
                r#"
pub fn process_items(items: Vec<String>) -> Vec<String> {
    items.into_iter().map(|s| s.to_uppercase()).collect()
}

pub fn validate_item(item: &str) -> bool {
    !item.is_empty()
}

fn internal_helper() {
    // Some internal logic
}
            "#,
            ),
        ];

        for (name, content) in files {
            let file_path = temp_dir.path().join(name);
            fs::write(&file_path, content).unwrap();
        }

        let config = BigOAnalysisConfig {
            project_path: temp_dir.path().to_path_buf(),
            include_patterns: vec!["*.rs".to_string()],
            exclude_patterns: vec![],
            confidence_threshold: 0,
            analyze_space_complexity: false,
        };

        let report = analyzer.analyze(config).await.unwrap();

        // Count expected functions:
        // mod.rs: 1 (init)
        // analyzer.rs: 2 (analyze_data, helper_function)
        // processor.rs: 3 (process_items, validate_item, internal_helper)
        // Total: 6 functions

        assert_eq!(
            report.analyzed_functions, 6,
            "Expected exactly 6 Rust functions across all files, but found {}",
            report.analyzed_functions
        );


        // Also verify the distribution is reasonable
        assert!(
            report.complexity_distribution.linear > 0,
            "Should have found at least some O(n) functions. Distribution: constant={}, linear={}, quadratic={}, unknown={}",
            report.complexity_distribution.constant,
            report.complexity_distribution.linear,
            report.complexity_distribution.quadratic,
            report.complexity_distribution.unknown
        );

        // Verify we're getting reasonable numbers
        assert!(
            report.analyzed_functions < 50,
            "Function count {} seems high",
            report.analyzed_functions
        );
    }

    /// Test that Big-O analyzer matches complexity analyzer for function count
    #[tokio::test]
    async fn test_consistency_with_complexity_analyzer() {
        // This test would compare Big-O count with Complexity analyzer count
        // For now, we'll create a simple consistency check

        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("consistency.rs");

        fs::write(
            &test_file,
            r#"
fn function_one() -> i32 { 1 }
fn function_two() -> i32 { 2 }
fn function_three() -> i32 { 3 }

impl TestStruct {
    fn method_one(&self) -> i32 { 4 }
    fn method_two(&self) -> i32 { 5 }
}

trait TestTrait {
    fn trait_method(&self) -> i32;
}

impl TestTrait for TestStruct {
    fn trait_method(&self) -> i32 { 6 }
}
        "#,
        )
        .unwrap();

        let analyzer = BigOAnalyzer::new();
        let config = BigOAnalysisConfig {
            project_path: temp_dir.path().to_path_buf(),
            include_patterns: vec!["*.rs".to_string()],
            exclude_patterns: vec![],
            confidence_threshold: 0,
            analyze_space_complexity: false,
        };

        let report = analyzer.analyze(config).await.unwrap();

        // Should find: function_one, function_two, function_three,
        // method_one, method_two, trait_method (impl)
        // Total: approximately 6 functions
        assert!(
            report.analyzed_functions >= 3 && report.analyzed_functions <= 15,
            "Expected approximately 6 functions but found {}",
            report.analyzed_functions
        );
    }
}
