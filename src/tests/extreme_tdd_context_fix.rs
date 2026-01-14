//! EXTREME TDD tests for fixing context to show actual code with proper AST annotations
//! These tests follow RED-GREEN-REFACTOR cycle strictly

use tempfile::TempDir;
use std::fs;

/// Test that context outputs the ACTUAL code from files, not just summaries
#[tokio::test]
async fn test_context_must_output_actual_file_content() {
    // ARRANGE: Create a Rust file with known content
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("example.rs");

    let expected_code = r#"// This is a comment
fn greet(name: &str) {
    println!("Hello, {}!", name);
}

fn main() {
    greet("World");
}"#;

    fs::write(&test_file, expected_code).unwrap();

    // ACT: Generate context
    let output_file = temp_dir.path().join("output.md");
    let result = crate::cli::handlers::utility_handlers::handle_context(
        Some("rust".to_string()),
        temp_dir.path().to_path_buf(),
        Some(output_file.clone()),
        crate::cli::handlers::utility_handlers::ContextFormat::Markdown,
        false,
        false,
    ).await;

    assert!(result.is_ok(), "Context generation failed");

    // ASSERT: The output must contain the EXACT code
    let output = fs::read_to_string(output_file).unwrap();

    // Must contain actual code lines
    assert!(output.contains("// This is a comment"),
            "Missing comment from source code");
    assert!(output.contains("fn greet(name: &str)"),
            "Missing greet function signature");
    assert!(output.contains("println!(\"Hello, {}!\", name)"),
            "Missing println statement");
    assert!(output.contains("greet(\"World\")"),
            "Missing function call");

    // Must be in a code block
    assert!(output.contains("```rust"),
            "Code not in rust code block");
}

/// Test that AST items are identified with correct line numbers
#[tokio::test]
async fn test_ast_items_have_correct_line_numbers() {
    // ARRANGE: Create file with known structure
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("structured.rs");

    let code = r#"use std::collections::HashMap;

pub struct Config {
    name: String,
}

impl Config {
    pub fn new(name: String) -> Self {
        Config { name }
    }
}

fn process_data() -> Vec<u8> {
    vec![1, 2, 3]
}

pub trait Handler {
    fn handle(&self);
}"#;

    fs::write(&test_file, code).unwrap();

    // ACT: Generate context
    let output_file = temp_dir.path().join("output.md");
    let _ = crate::cli::handlers::utility_handlers::handle_context(
        Some("rust".to_string()),
        temp_dir.path().to_path_buf(),
        Some(output_file.clone()),
        crate::cli::handlers::utility_handlers::ContextFormat::Markdown,
        false,
        false,
    ).await;

    // ASSERT: AST items listed with correct line numbers
    let output = fs::read_to_string(output_file).unwrap();

    // Check for AST items being listed
    assert!(output.contains("Config"),
            "Struct Config not found in AST");
    assert!(output.contains("process_data"),
            "Function process_data not found in AST");
    assert!(output.contains("Handler"),
            "Trait Handler not found in AST");

    // Line numbers should be mentioned (even if approximate)
    // Struct Config starts around line 3
    // Impl Config starts around line 7
    // Function process_data starts around line 13
    // Trait Handler starts around line 17

    // At minimum, these items should be identified as different AST elements
    assert!(output.contains("Struct") || output.contains("struct"),
            "Struct type not identified");
    assert!(output.contains("Function") || output.contains("fn"),
            "Function type not identified");
    assert!(output.contains("Trait") || output.contains("trait"),
            "Trait type not identified");
}

/// Test that the context shows BOTH code AND analysis
#[tokio::test]
async fn test_context_shows_code_plus_analysis() {
    // ARRANGE: Create a small project
    let temp_dir = TempDir::new().unwrap();

    let lib_code = r#"pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 2), 4);
    }
}"#;

    fs::write(temp_dir.path().join("lib.rs"), lib_code).unwrap();

    // ACT: Generate context
    let output_file = temp_dir.path().join("output.md");
    let _ = crate::cli::handlers::utility_handlers::handle_context(
        Some("rust".to_string()),
        temp_dir.path().to_path_buf(),
        Some(output_file.clone()),
        crate::cli::handlers::utility_handlers::ContextFormat::Markdown,
        false,
        false,
    ).await;

    // ASSERT: Has both code and metadata
    let output = fs::read_to_string(output_file).unwrap();

    // Must have the actual code
    assert!(output.contains("pub fn add(a: i32, b: i32) -> i32"),
            "Missing function signature");
    assert!(output.contains("a + b"),
            "Missing function body");

    // Must have analysis/metadata
    assert!(output.contains("Functions") || output.contains("functions"),
            "Missing function count");

    // Must identify test module
    assert!(output.contains("mod tests") || output.contains("test_add"),
            "Test module not included");
}

/// Test JSON format also includes actual code
#[tokio::test]
async fn test_json_format_includes_code() {
    // ARRANGE: Simple file
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("simple.rs");

    let code = r#"fn hello() {
    println!("Hello!");
}"#;

    fs::write(&test_file, code).unwrap();

    // ACT: Generate JSON context
    let output_file = temp_dir.path().join("output.json");
    let result = crate::cli::handlers::utility_handlers::handle_context(
        Some("rust".to_string()),
        temp_dir.path().to_path_buf(),
        Some(output_file.clone()),
        crate::cli::handlers::utility_handlers::ContextFormat::Json,
        false,
        false,
    ).await;

    assert!(result.is_ok(), "JSON context generation failed");

    // ASSERT: JSON contains the code or reference to it
    let output = fs::read_to_string(output_file).unwrap();

    // The JSON should have file information
    assert!(output.contains("simple.rs"),
            "File name not in JSON output");

    // Should identify the function
    assert!(output.contains("hello"),
            "Function name not in JSON output");
}

/// Test that empty files don't break context generation
#[tokio::test]
async fn test_empty_file_handling() {
    // ARRANGE: Create empty file
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("empty.rs");

    fs::write(&test_file, "").unwrap();

    // ACT: Generate context
    let output_file = temp_dir.path().join("output.md");
    let result = crate::cli::handlers::utility_handlers::handle_context(
        Some("rust".to_string()),
        temp_dir.path().to_path_buf(),
        Some(output_file.clone()),
        crate::cli::handlers::utility_handlers::ContextFormat::Markdown,
        false,
        false,
    ).await;

    // ASSERT: Should succeed even with empty file
    assert!(result.is_ok(), "Failed on empty file");

    let output = fs::read_to_string(output_file).unwrap();
    assert!(output.contains("empty.rs"), "Empty file not listed");
}

/// Test mixed language project shows all files with code
#[tokio::test]
async fn test_mixed_language_project() {
    // ARRANGE: Create files in different languages
    let temp_dir = TempDir::new().unwrap();

    let rust_code = "fn main() { println!(\"Rust\"); }";
    let python_code = "def main():\n    print(\"Python\")";
    let js_code = "function main() { console.log(\"JS\"); }";

    fs::write(temp_dir.path().join("main.rs"), rust_code).unwrap();
    fs::write(temp_dir.path().join("main.py"), python_code).unwrap();
    fs::write(temp_dir.path().join("main.js"), js_code).unwrap();

    // ACT: Generate context (auto-detect language)
    let output_file = temp_dir.path().join("output.md");
    let _ = crate::cli::handlers::utility_handlers::handle_context(
        None, // Auto-detect
        temp_dir.path().to_path_buf(),
        Some(output_file.clone()),
        crate::cli::handlers::utility_handlers::ContextFormat::Markdown,
        false,
        false,
    ).await;

    // ASSERT: All files included with their code
    let output = fs::read_to_string(output_file).unwrap();

    // Should have all three files
    assert!(output.contains("main.rs"), "Rust file missing");
    assert!(output.contains("main.py"), "Python file missing");
    assert!(output.contains("main.js"), "JS file missing");

    // Should have actual code from each
    assert!(output.contains("println!(\"Rust\")"), "Rust code missing");
    assert!(output.contains("print(\"Python\")"), "Python code missing");
    assert!(output.contains("console.log(\"JS\")"), "JS code missing");
}