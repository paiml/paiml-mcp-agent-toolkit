//! EXTREME TDD: Context should output ANNOTATED AST, not raw code
//! The context should show the structure with function signatures, types, etc.
//! but NOT the implementation details

use tempfile::TempDir;
use std::fs;

/// Test that context shows AST structure, not raw implementation
#[tokio::test]
async fn test_context_shows_ast_not_raw_code() {
    // ARRANGE: Create file with implementation details
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("example.rs");

    let code = r#"
/// Calculate the sum of two numbers
fn add(a: i32, b: i32) -> i32 {
    // This is implementation detail
    let result = a + b;
    println!("Adding {} + {} = {}", a, b, result);
    result
}

struct Calculator {
    value: i32,
}

impl Calculator {
    fn new(initial: i32) -> Self {
        // Implementation details we don't want
        println!("Creating calculator with {}", initial);
        Calculator { value: initial }
    }

    fn compute(&mut self, x: i32) -> i32 {
        // More implementation
        self.value += x;
        self.value * 2
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

    // ASSERT: Should have AST structure
    assert!(output.contains("fn add") || output.contains("add(a: i32, b: i32) -> i32"),
            "Should show function signature");
    assert!(output.contains("struct Calculator"),
            "Should show struct definition");
    assert!(output.contains("impl Calculator"),
            "Should show impl block");

    // ASSERT: Should NOT have implementation details
    assert!(!output.contains("let result = a + b"),
            "Should NOT contain implementation details");
    assert!(!output.contains("println!(\"Adding"),
            "Should NOT contain println statements");
    assert!(!output.contains("self.value * 2"),
            "Should NOT contain internal computations");
    assert!(!output.contains("// This is implementation detail"),
            "Should NOT contain implementation comments");
}

/// Test that AST shows proper annotations with line numbers
#[tokio::test]
async fn test_ast_has_proper_annotations() {
    // ARRANGE
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("lib.rs");

    let code = r#"use std::collections::HashMap;

pub struct Config {
    name: String,
    port: u16,
}

impl Config {
    pub fn new(name: String, port: u16) -> Self {
        Config { name, port }
    }
}

pub trait Handler {
    fn handle(&self);
}

pub fn process() -> Result<(), Error> {
    Ok(())
}
"#;

    fs::write(&test_file, code).unwrap();

    // ACT
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

    // ASSERT: Should show AST structure with types
    assert!(output.contains("Config") &&
            (output.contains("struct") || output.contains("Struct")),
            "Should annotate struct Config");

    assert!(output.contains("Handler") &&
            (output.contains("trait") || output.contains("Trait")),
            "Should annotate trait Handler");

    assert!(output.contains("process") &&
            (output.contains("fn") || output.contains("Function")),
            "Should annotate function process");

    // Should show visibility
    assert!(output.contains("pub") || output.contains("public"),
            "Should indicate public visibility");

    // Should NOT have the actual implementation
    assert!(!output.contains("Config { name, port }"),
            "Should not have constructor implementation");
}

/// Test that context shows function signatures but not bodies
#[tokio::test]
async fn test_shows_signatures_not_bodies() {
    // ARRANGE
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("main.rs");

    let code = r#"
fn fibonacci(n: u32) -> u64 {
    if n <= 1 {
        return n as u64;
    }
    fibonacci(n - 1) + fibonacci(n - 2)
}

async fn fetch_data(url: &str) -> Result<String, Error> {
    let client = Client::new();
    let response = client.get(url).send().await?;
    Ok(response.text().await?)
}

fn main() {
    println!("Result: {}", fibonacci(10));
}
"#;

    fs::write(&test_file, code).unwrap();

    // ACT
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

    // ASSERT: Should show signatures
    assert!(output.contains("fibonacci") &&
            (output.contains("u32") || output.contains("u64")),
            "Should show fibonacci signature with types");

    assert!(output.contains("fetch_data") &&
            output.contains("async"),
            "Should show async function");

    // Should NOT show implementation
    assert!(!output.contains("fibonacci(n - 1)"),
            "Should not show recursive call");
    assert!(!output.contains("client.get(url)"),
            "Should not show HTTP client code");
    assert!(!output.contains("println!(\"Result:"),
            "Should not show println in main");
}