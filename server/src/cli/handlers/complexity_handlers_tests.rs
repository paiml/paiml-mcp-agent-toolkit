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
