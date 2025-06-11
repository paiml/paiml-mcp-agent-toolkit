//! Simple tests for refactor handlers

use super::*;
use tempfile::TempDir;

#[tokio::test]
async fn test_route_refactor_command_serve() {
    let temp_dir = TempDir::new().unwrap();

    let cmd = RefactorCommands::Serve {
        refactor_mode: RefactorMode::Interactive,
        config: None,
        project: temp_dir.path().to_path_buf(),
        parallel: 1,
        memory_limit: 100,
        batch_size: 10,
        priority: None,
        checkpoint_dir: None,
        resume: false,
        auto_commit: None,
        max_runtime: Some(1), // 1 second timeout
    };

    // This will timeout after 1 second
    let _ = route_refactor_command(cmd).await;
}

#[tokio::test]
async fn test_route_refactor_command_status() {
    let temp_dir = TempDir::new().unwrap();
    let checkpoint = temp_dir.path().join("checkpoint.json");

    // Create a dummy checkpoint file
    std::fs::write(&checkpoint, r#"{"current": "idle", "targets": []}"#).unwrap();

    let cmd = RefactorCommands::Status {
        checkpoint: checkpoint.clone(),
        format: RefactorOutputFormat::Json,
    };

    let result = route_refactor_command(cmd).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_discover_refactor_targets() {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir(&src_dir).unwrap();

    // Create test files
    std::fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();
    std::fs::write(src_dir.join("lib.rs"), "pub fn test() {}").unwrap();

    let targets = discover_refactor_targets(&temp_dir.path().to_path_buf())
        .await
        .unwrap();
    assert_eq!(targets.len(), 2);
}

#[tokio::test]
async fn test_load_refactor_config_json() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");

    std::fs::write(
        &config_path,
        r#"{
        "rules": {
            "target_complexity": 10,
            "max_function_lines": 50,
            "remove_satd": true
        },
        "parallel_workers": 4,
        "memory_limit_mb": 512,
        "batch_size": 20
    }"#,
    )
    .unwrap();

    let config = load_refactor_config_json(&config_path).await.unwrap();
    assert_eq!(config.target_complexity, 10);
    assert_eq!(config.max_function_lines, 50);
    assert!(config.remove_satd);
    assert_eq!(config.parallel_workers, 4);
    assert_eq!(config.memory_limit_mb, 512);
    assert_eq!(config.batch_size, 20);
}
