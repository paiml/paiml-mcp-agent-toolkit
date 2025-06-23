//! Tests for utility handlers

use super::*;
use crate::cli::{ContextFormat, OutputFormat as CliOutputFormat};
use crate::stateless_server::StatelessTemplateServer;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_handle_list_basic() {
    let server = Arc::new(StatelessTemplateServer::new().unwrap());

    // Test list all templates
    let result = handle_list(server.clone(), None, None, CliOutputFormat::Table).await;
    assert!(result.is_ok());

    // Test list specific category
    let result = handle_list(
        server.clone(),
        Some("rust".to_string()),
        Some("readme".to_string()),
        CliOutputFormat::Json,
    )
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_search() {
    let server = Arc::new(StatelessTemplateServer::new().unwrap());

    // Search for rust templates
    let result = handle_search(server.clone(), "rust".to_string(), None, 10).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_context() {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir(&src_dir).unwrap();

    // Create a test file
    std::fs::write(
        src_dir.join("main.rs"),
        r#"fn main() {
    println!("Hello, world!");
}
"#,
    )
    .unwrap();

    // Test normal context generation
    let result = handle_context(
        None,
        temp_dir.path().to_path_buf(),
        None,
        ContextFormat::Markdown,
        false, // include_large_files
        false, // skip_expensive_metrics
    )
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_detect_primary_language() {
    let temp_dir = TempDir::new().unwrap();

    // Test Rust detection
    std::fs::write(temp_dir.path().join("Cargo.toml"), "[package]").unwrap();
    let result = detect_primary_language(temp_dir.path());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "rust");
}
