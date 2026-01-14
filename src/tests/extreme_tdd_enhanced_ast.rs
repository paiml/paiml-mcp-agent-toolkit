//! EXTREME TDD: Enhanced Annotated AST Tests
//! Goal: Show AST structure with rich annotations, not raw code

use tempfile::TempDir;
use std::fs;

/// Test that context shows annotated AST structure, NOT raw code
#[tokio::test]
async fn test_context_shows_annotated_ast_not_code() {
    // ARRANGE: Create test file
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.rs");

    let code = r#"
fn calculate_sum(a: i32, b: i32) -> i32 {
    // Complex implementation details
    let mut result = 0;
    for i in 0..a {
        result += b;
    }
    result
}

struct Calculator {
    value: i32,
}

impl Calculator {
    fn new(v: i32) -> Self {
        // Implementation details
        Calculator { value: v }
    }
}
"#;

    fs::write(&test_file, code).unwrap();

    // ACT: Generate context
    let output_file = temp_dir.path().join("context.md");
    let _ = crate::cli::handlers::utility_handlers::handle_context(
        Some("rust".to_string()),
        temp_dir.path().to_path_buf(),
        Some(output_file.clone()),
        crate::cli::handlers::utility_handlers::ContextFormat::Markdown,
        false,
        false,
    ).await;

    let output = fs::read_to_string(output_file).unwrap();

    // ASSERT: Should show AST structure
    assert!(output.contains("Function") || output.contains("function"),
            "Should identify functions in AST");
    assert!(output.contains("Struct") || output.contains("struct"),
            "Should identify structs in AST");

    // ASSERT: Should NOT contain raw implementation
    assert!(!output.contains("let mut result = 0"),
            "Should NOT contain implementation details");
    assert!(!output.contains("for i in 0..a"),
            "Should NOT contain loop implementation");
    assert!(!output.contains("// Complex implementation"),
            "Should NOT contain implementation comments");
}

/// Test that AST shows rich annotations
#[tokio::test]
async fn test_ast_has_rich_annotations() {
    // ARRANGE: Create test file
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("complex.rs");

    let code = r#"
fn complex_function(data: Vec<i32>) -> i32 {
    // High complexity function
    let mut sum = 0;
    for item in data {
        if item > 0 {
            for j in 0..item {
                if j % 2 == 0 {
                    sum += j;
                }
            }
        }
    }
    sum
}

fn simple_function() -> i32 {
    42
}
"#;

    fs::write(&test_file, code).unwrap();

    // ACT: Generate context
    let output_file = temp_dir.path().join("context.md");
    let _ = crate::cli::handlers::utility_handlers::handle_context(
        Some("rust".to_string()),
        temp_dir.path().to_path_buf(),
        Some(output_file.clone()),
        crate::cli::handlers::utility_handlers::ContextFormat::Markdown,
        false,
        false,
    ).await;

    let output = fs::read_to_string(output_file).unwrap();

    // ASSERT: Should have annotations (at least some of these)
    let has_annotations =
        output.contains("complexity") ||
        output.contains("cognitive") ||
        output.contains("O(") ||  // Big-O notation
        output.contains("Score") ||
        output.contains("Metrics") ||
        output.contains("Functions:");

    assert!(has_annotations,
            "Should have rich annotations like complexity, cognitive, big-o, etc.");
}

/// Test that AST shows hierarchical structure
#[tokio::test]
async fn test_ast_shows_hierarchical_structure() {
    // ARRANGE: Create test with nested items
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("hierarchical.rs");

    let code = r#"
mod utils {
    pub fn helper() -> i32 { 1 }
}

pub struct Server {
    port: u16,
}

impl Server {
    pub fn new(port: u16) -> Self {
        Server { port }
    }

    pub fn start(&self) {
        // start server
    }
}

pub trait Handler {
    fn handle(&self);
}
"#;

    fs::write(&test_file, code).unwrap();

    // ACT: Generate context
    let output_file = temp_dir.path().join("context.md");
    let _ = crate::cli::handlers::utility_handlers::handle_context(
        Some("rust".to_string()),
        temp_dir.path().to_path_buf(),
        Some(output_file.clone()),
        crate::cli::handlers::utility_handlers::ContextFormat::Markdown,
        false,
        false,
    ).await;

    let output = fs::read_to_string(output_file).unwrap();

    // ASSERT: Should show structure
    assert!(output.contains("Server") || output.contains("struct"),
            "Should show Server struct");
    assert!(output.contains("Handler") || output.contains("trait"),
            "Should show Handler trait");
    assert!(output.contains("impl") || output.contains("Impl"),
            "Should show impl blocks");

    // Should list functions/methods
    let has_function_list =
        output.contains("new") ||
        output.contains("start") ||
        output.contains("helper");

    assert!(has_function_list,
            "Should list function/method names");
}

/// Test that enhanced annotations include all metadata
#[tokio::test]
async fn test_enhanced_annotations_complete() {
    // ARRANGE: Create test file
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("annotated.rs");

    let code = r#"
// TODO: Refactor this function
fn process_data(items: &[u32]) -> u32 {
    items.iter().sum()
}

#[test]
fn test_process() {
    assert_eq!(process_data(&[1, 2, 3]), 6);
}
"#;

    fs::write(&test_file, code).unwrap();

    // ACT: Generate context
    let output_file = temp_dir.path().join("context.md");
    let _ = crate::cli::handlers::utility_handlers::handle_context(
        Some("rust".to_string()),
        temp_dir.path().to_path_buf(),
        Some(output_file.clone()),
        crate::cli::handlers::utility_handlers::ContextFormat::Markdown,
        false,
        false,
    ).await;

    let output = fs::read_to_string(output_file).unwrap();

    // ASSERT: Should identify different types of functions
    assert!(output.contains("process_data") || output.contains("Function"),
            "Should identify regular functions");

    // Enhanced annotations should be present (at least in headers/summary)
    let has_enhanced_info =
        output.contains("Total Functions") ||
        output.contains("Quality") ||
        output.contains("Scorecard") ||
        output.contains("Metrics");

    assert!(has_enhanced_info,
            "Should have enhanced project-level annotations");
}