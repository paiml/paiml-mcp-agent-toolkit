//! Tests for utils helpers

use super::*;
use tempfile::TempDir;

#[test]
fn test_ensure_directory_exists() {
    let temp_dir = TempDir::new().unwrap();
    let new_dir = temp_dir.path().join("test_dir");
    
    let result = ensure_directory_exists(&new_dir);
    assert!(result.is_ok());
    assert!(new_dir.exists());
    
    // Test idempotency - should work even if directory exists
    let result2 = ensure_directory_exists(&new_dir);
    assert!(result2.is_ok());
}

#[test]
fn test_format_bytes() {
    assert_eq!(format_bytes(0), "0 B");
    assert_eq!(format_bytes(512), "512 B");
    assert_eq!(format_bytes(1024), "1.0 KB");
    assert_eq!(format_bytes(1536), "1.5 KB");
    assert_eq!(format_bytes(1048576), "1.0 MB");
    assert_eq!(format_bytes(1572864), "1.5 MB");
    assert_eq!(format_bytes(1073741824), "1.0 GB");
}

#[test]
fn test_truncate_string() {
    assert_eq!(truncate_string("hello", 10), "hello");
    assert_eq!(truncate_string("hello world", 5), "hello...");
    assert_eq!(truncate_string("hello world", 8), "hello...");
    assert_eq!(truncate_string("hello world", 11), "hello world");
    assert_eq!(truncate_string("", 5), "");
}

#[test]
fn test_sanitize_filename() {
    assert_eq!(sanitize_filename("hello.txt"), "hello.txt");
    assert_eq!(sanitize_filename("hello/world.txt"), "hello_world.txt");
    assert_eq!(sanitize_filename("hello\\world.txt"), "hello_world.txt");
    assert_eq!(sanitize_filename("hello:world.txt"), "hello_world.txt");
    assert_eq!(sanitize_filename("hello*world?.txt"), "hello_world_.txt");
    assert_eq!(sanitize_filename("hello|world<>.txt"), "hello_world__.txt");
}

#[test]
fn test_is_binary_file() {
    // Text files should return false
    assert!(!is_binary_file("test.txt"));
    assert!(!is_binary_file("test.rs"));
    assert!(!is_binary_file("test.json"));
    assert!(!is_binary_file("test.md"));
    
    // Binary files should return true
    assert!(is_binary_file("test.exe"));
    assert!(is_binary_file("test.dll"));
    assert!(is_binary_file("test.so"));
    assert!(is_binary_file("test.dylib"));
    assert!(is_binary_file("test.png"));
    assert!(is_binary_file("test.jpg"));
    assert!(is_binary_file("test.pdf"));
}

#[test]
fn test_get_file_extension() {
    assert_eq!(get_file_extension("test.txt"), Some("txt"));
    assert_eq!(get_file_extension("test.tar.gz"), Some("gz"));
    assert_eq!(get_file_extension("test"), None);
    assert_eq!(get_file_extension(".gitignore"), None);
    assert_eq!(get_file_extension("test."), None);
}

#[test]
fn test_normalize_path() {
    use std::path::Path;
    
    let path = Path::new("./test/../hello/./world.txt");
    let normalized = normalize_path(path);
    assert!(normalized.to_string_lossy().contains("hello"));
    assert!(normalized.to_string_lossy().contains("world.txt"));
    assert!(!normalized.to_string_lossy().contains(".."));
    assert!(!normalized.to_string_lossy().contains("./"));
}

#[tokio::test]
async fn test_timeout_future() {
    use std::time::Duration;
    use tokio::time::sleep;
    
    // Test successful completion
    let result = timeout_future(
        async { Ok::<_, anyhow::Error>("success".to_string()) },
        Duration::from_secs(1)
    ).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
    
    // Test timeout
    let result = timeout_future(
        async {
            sleep(Duration::from_secs(2)).await;
            Ok::<_, anyhow::Error>("never reached".to_string())
        },
        Duration::from_millis(100)
    ).await;
    assert!(result.is_err());
}