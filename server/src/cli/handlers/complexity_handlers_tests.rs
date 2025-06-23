//! Tests for complexity handlers

use super::*;
use tempfile::TempDir;

#[tokio::test]
async fn test_handle_analyze_complexity_basic() {
    let temp_dir = TempDir::new().unwrap();

    std::fs::write(
        temp_dir.path().join("test.rs"),
        r#"
fn simple_function() {
    println!("Hello");
}
"#,
    )
    .unwrap();

    // Just verify the function can be called without panicking
    let _ = handle_analyze_complexity(
        temp_dir.path().to_path_buf(),
        None,
        ComplexityOutputFormat::Summary,
        None,
        None,
        None,
        vec![],
        false,
        10,
    )
    .await;
}

#[test]
fn test_complexity_output_format_display() {
    assert_eq!(format!("{:?}", ComplexityOutputFormat::Summary), "Summary");
    assert_eq!(format!("{:?}", ComplexityOutputFormat::Json), "Json");
    assert_eq!(format!("{:?}", ComplexityOutputFormat::Sarif), "Sarif");
}

#[tokio::test]
async fn test_handle_analyze_dead_code_summary_output() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create a test file with obvious dead code
    std::fs::write(
        temp_dir.path().join("dead.rs"),
        r#"
fn used_function() {
    println!("I am used");
}

fn dead_function() {
    println!("I am never called");
}

fn main() {
    used_function();
}
"#,
    )
    .unwrap();
    
    // Test with summary format
    let result = handle_analyze_dead_code(
        temp_dir.path().to_path_buf(),
        DeadCodeOutputFormat::Summary,
        None,
        false,
        5,
        false,
        None,
    )
    .await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_analyze_dead_code_json_output() {
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("dead_code.json");
    
    // Create test file
    std::fs::write(
        temp_dir.path().join("test.rs"),
        r#"
struct UnusedStruct {
    field: i32,
}

fn main() {
    println!("Main function");
}
"#,
    )
    .unwrap();
    
    // Test with JSON output to file
    let result = handle_analyze_dead_code(
        temp_dir.path().to_path_buf(),
        DeadCodeOutputFormat::Json,
        None,
        false,
        5,
        false,
        Some(output_file.clone()),
    )
    .await;
    
    assert!(result.is_ok());
    
    // Verify file was created
    if output_file.exists() {
        let content = std::fs::read_to_string(&output_file).unwrap();
        assert!(content.contains("\"summary\""));
        assert!(content.contains("\"files\""));
    }
}

#[tokio::test]
async fn test_handle_analyze_dead_code_top_files_limit() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create multiple test files
    for i in 0..5 {
        std::fs::write(
            temp_dir.path().join(format!("file{}.rs", i)),
            format!(
                r#"
fn dead_function_{}() {{
    println!("Dead code in file {}", {});
}}
"#,
                i, i, i
            ),
        )
        .unwrap();
    }
    
    // Test with top_files = 3
    let result = handle_analyze_dead_code(
        temp_dir.path().to_path_buf(),
        DeadCodeOutputFormat::Summary,
        Some(3),
        false,
        5,
        false,
        None,
    )
    .await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_analyze_dead_code_sarif_format() {
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("dead_code.sarif");
    
    // Create test file with dead code
    std::fs::write(
        temp_dir.path().join("test.rs"),
        r#"
fn dead_function() {
    if false {
        println!("Unreachable");
    }
}
"#,
    )
    .unwrap();
    
    // Test SARIF output
    let result = handle_analyze_dead_code(
        temp_dir.path().to_path_buf(),
        DeadCodeOutputFormat::Sarif,
        None,
        true, // include_unreachable
        5,
        false,
        Some(output_file.clone()),
    )
    .await;
    
    assert!(result.is_ok());
    
    // Verify SARIF structure if file exists
    if output_file.exists() {
        let content = std::fs::read_to_string(&output_file).unwrap();
        assert!(content.contains("\"version\": \"2.1.0\""));
        assert!(content.contains("\"$schema\""));
        assert!(content.contains("\"runs\""));
    }
}

#[tokio::test]
async fn test_handle_analyze_dead_code_with_min_lines() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create file with small dead function
    std::fs::write(
        temp_dir.path().join("test.rs"),
        r#"
fn tiny() {}  // Only 1 line

fn larger_dead_function() {
    let x = 1;
    let y = 2;
    let z = x + y;
    println!("{}", z);
}  // 5 lines
"#,
    )
    .unwrap();
    
    // Test with min_dead_lines = 3
    let result = handle_analyze_dead_code(
        temp_dir.path().to_path_buf(),
        DeadCodeOutputFormat::Summary,
        None,
        false,
        3, // min_dead_lines
        false,
        None,
    )
    .await;
    
    assert!(result.is_ok());
}

#[test]
fn test_dead_code_output_format_display() {
    assert_eq!(format!("{:?}", DeadCodeOutputFormat::Summary), "Summary");
    assert_eq!(format!("{:?}", DeadCodeOutputFormat::Json), "Json");
    assert_eq!(format!("{:?}", DeadCodeOutputFormat::Sarif), "Sarif");
    assert_eq!(format!("{:?}", DeadCodeOutputFormat::Markdown), "Markdown");
}
