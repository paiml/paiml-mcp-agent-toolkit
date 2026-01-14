//! EXTREME TDD tests for fixing P0 context bug
//! These tests ensure that context outputs actual code with AST annotations
//! not just analysis metrics

use tempfile::TempDir;
use std::fs;
use std::path::PathBuf;

/// RED TEST 1: Context must output actual code content
#[tokio::test]
async fn test_context_outputs_actual_code() {
    // ARRANGE: Create test project with known code
    let temp_dir = TempDir::new().unwrap();
    let test_file_path = temp_dir.path().join("test.rs");

    let test_code = r#"
fn hello_world() {
    println!("Hello, World!");
}

struct Person {
    name: String,
    age: u32,
}

impl Person {
    fn new(name: String, age: u32) -> Self {
        Person { name, age }
    }

    fn greet(&self) {
        println!("Hi, I'm {}", self.name);
    }
}
"#;

    fs::write(&test_file_path, test_code).unwrap();

    // ACT: Generate context
    let output_path = temp_dir.path().join("context.md");
    let result = crate::cli::handlers::utility_handlers::handle_context(
        Some("rust".to_string()),
        temp_dir.path().to_path_buf(),
        Some(output_path.clone()),
        crate::cli::handlers::utility_handlers::ContextFormat::Markdown,
        false,
        false,
    ).await;

    // ASSERT: Context generation succeeded
    assert!(result.is_ok(), "Context generation failed: {:?}", result);

    // ASSERT: Output file exists
    assert!(output_path.exists(), "Context output file was not created");

    // Read the generated context
    let context_content = fs::read_to_string(&output_path).unwrap();

    // CRITICAL ASSERTIONS: Must contain actual code, not just metrics
    assert!(
        context_content.contains("fn hello_world()"),
        "Context missing function definition 'hello_world'"
    );

    assert!(
        context_content.contains("struct Person"),
        "Context missing struct definition 'Person'"
    );

    assert!(
        context_content.contains("impl Person"),
        "Context missing impl block for Person"
    );

    assert!(
        context_content.contains("println!(\"Hello, World!\")"),
        "Context missing actual code content"
    );

    // Should NOT be just empty files or metrics only
    assert!(
        !context_content.contains("**Total Files**: 0"),
        "Context shows 0 files when it should have files"
    );

    assert!(
        !context_content.contains("**Total Functions**: 0"),
        "Context shows 0 functions when it should have functions"
    );
}

/// RED TEST 2: Context must show AST annotations for functions
#[tokio::test]
async fn test_context_shows_ast_annotations() {
    // ARRANGE: Create test project
    let temp_dir = TempDir::new().unwrap();
    let test_file_path = temp_dir.path().join("lib.rs");

    let test_code = r#"
pub fn calculate_sum(a: i32, b: i32) -> i32 {
    a + b
}

pub async fn fetch_data() -> String {
    "data".to_string()
}

fn private_helper(x: u64) -> u64 {
    x * 2
}
"#;

    fs::write(&test_file_path, test_code).unwrap();

    // ACT: Generate context
    let output_path = temp_dir.path().join("context.md");
    let _ = crate::cli::handlers::utility_handlers::handle_context(
        Some("rust".to_string()),
        temp_dir.path().to_path_buf(),
        Some(output_path.clone()),
        crate::cli::handlers::utility_handlers::ContextFormat::Markdown,
        false,
        false,
    ).await;

    // Read the generated context
    let context_content = fs::read_to_string(&output_path).unwrap();

    // ASSERT: Must have AST annotations showing function signatures
    assert!(
        context_content.contains("calculate_sum") &&
        (context_content.contains("Function") || context_content.contains("fn")),
        "Context missing AST annotation for calculate_sum function"
    );

    assert!(
        context_content.contains("fetch_data") &&
        (context_content.contains("async") || context_content.contains("Function")),
        "Context missing AST annotation for async function"
    );

    assert!(
        context_content.contains("private_helper"),
        "Context missing private function in AST"
    );
}

/// RED TEST 3: Context must list all function names in summary
#[tokio::test]
async fn test_context_lists_function_names() {
    // ARRANGE: Create test project with multiple functions
    let temp_dir = TempDir::new().unwrap();
    let test_file_path = temp_dir.path().join("main.rs");

    let test_code = r#"
fn process_data(input: &str) -> Vec<u8> {
    input.bytes().collect()
}

fn validate_input(data: &[u8]) -> bool {
    !data.is_empty()
}

fn main() {
    let data = process_data("test");
    if validate_input(&data) {
        println!("Valid!");
    }
}
"#;

    fs::write(&test_file_path, test_code).unwrap();

    // ACT: Generate context
    let output_path = temp_dir.path().join("context.md");
    let _ = crate::cli::handlers::utility_handlers::handle_context(
        Some("rust".to_string()),
        temp_dir.path().to_path_buf(),
        Some(output_path.clone()),
        crate::cli::handlers::utility_handlers::ContextFormat::Markdown,
        false,
        false,
    ).await;

    // Read the generated context
    let context_content = fs::read_to_string(&output_path).unwrap();

    // ASSERT: All function names must be listed
    let function_names = ["process_data", "validate_input", "main"];

    for func_name in &function_names {
        assert!(
            context_content.contains(func_name),
            "Context missing function name: {}", func_name
        );
    }

    // Should show correct function count
    assert!(
        !context_content.contains("**Total Functions**: 0"),
        "Context incorrectly shows 0 functions"
    );

    // Should have at least 3 functions mentioned
    assert!(
        context_content.contains("3") ||
        context_content.contains("process_data") &&
        context_content.contains("validate_input") &&
        context_content.contains("main"),
        "Context doesn't show all 3 functions"
    );
}

/// RED TEST 4: Context must include struct and impl definitions
#[tokio::test]
async fn test_context_includes_structs_and_impls() {
    // ARRANGE: Create test with structs
    let temp_dir = TempDir::new().unwrap();
    let test_file_path = temp_dir.path().join("types.rs");

    let test_code = r#"
#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub timeout: Duration,
}

impl Config {
    pub fn new(host: String, port: u16) -> Self {
        Config {
            host,
            port,
            timeout: Duration::from_secs(30),
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.host.is_empty() && self.port > 0
    }
}

pub enum Status {
    Active,
    Inactive,
    Error(String),
}
"#;

    fs::write(&test_file_path, test_code).unwrap();

    // ACT: Generate context
    let output_path = temp_dir.path().join("context.md");
    let _ = crate::cli::handlers::utility_handlers::handle_context(
        Some("rust".to_string()),
        temp_dir.path().to_path_buf(),
        Some(output_path.clone()),
        crate::cli::handlers::utility_handlers::ContextFormat::Markdown,
        false,
        false,
    ).await;

    // Read the generated context
    let context_content = fs::read_to_string(&output_path).unwrap();

    // ASSERT: Must include struct definitions
    assert!(
        context_content.contains("struct Config"),
        "Context missing struct Config definition"
    );

    assert!(
        context_content.contains("impl Config"),
        "Context missing impl block for Config"
    );

    assert!(
        context_content.contains("enum Status"),
        "Context missing enum Status definition"
    );

    // Should show struct fields
    assert!(
        context_content.contains("host") && context_content.contains("port"),
        "Context missing struct fields"
    );

    // Should NOT show empty structs count
    assert!(
        !context_content.contains("**Total Structs**: 0"),
        "Context incorrectly shows 0 structs"
    );
}

/// RED TEST 5: Context must not be just a report with empty files
#[tokio::test]
async fn test_context_not_just_empty_report() {
    // ARRANGE: Create realistic project
    let temp_dir = TempDir::new().unwrap();

    // Create multiple files
    let lib_code = r#"
pub mod utils {
    pub fn helper() -> i32 { 42 }
}

pub struct Application {
    name: String,
}

impl Application {
    pub fn run(&self) {
        println!("Running {}", self.name);
    }
}
"#;

    let main_code = r#"
use crate::Application;

fn main() {
    let app = Application { name: "Test".into() };
    app.run();
}
"#;

    fs::write(temp_dir.path().join("lib.rs"), lib_code).unwrap();
    fs::write(temp_dir.path().join("main.rs"), main_code).unwrap();

    // ACT: Generate context
    let output_path = temp_dir.path().join("context.md");
    let _ = crate::cli::handlers::utility_handlers::handle_context(
        Some("rust".to_string()),
        temp_dir.path().to_path_buf(),
        Some(output_path.clone()),
        crate::cli::handlers::utility_handlers::ContextFormat::Markdown,
        false,
        false,
    ).await;

    // Read the generated context
    let context_content = fs::read_to_string(&output_path).unwrap();

    // ASSERT: Must NOT be just metrics
    assert!(
        !context_content.contains("files: vec![]"),
        "Context has empty files vector"
    );

    // Must contain actual file content
    assert!(
        context_content.contains("lib.rs") || context_content.contains("main.rs"),
        "Context missing file names"
    );

    assert!(
        context_content.contains("pub mod utils") ||
        context_content.contains("pub struct Application"),
        "Context missing actual code content from lib.rs"
    );

    assert!(
        context_content.contains("fn main()") ||
        context_content.contains("app.run()"),
        "Context missing actual code content from main.rs"
    );

    // Must NOT be dominated by analysis metrics
    let metrics_keywords = ["Entropy Analysis", "Big-O Complexity", "Provability Analysis",
                            "Graph Metrics", "Technical Debt Gradient"];
    let code_keywords = ["fn ", "struct ", "impl ", "pub ", "mod "];

    let metrics_count: usize = metrics_keywords.iter()
        .filter(|k| context_content.contains(*k))
        .count();

    let code_count: usize = code_keywords.iter()
        .filter(|k| context_content.contains(*k))
        .count();

    assert!(
        code_count > 0,
        "Context contains no code keywords"
    );

    // If it has metrics, it must ALSO have code
    if metrics_count > 0 {
        assert!(
            code_count > 0,
            "Context has metrics but no actual code"
        );
    }
}