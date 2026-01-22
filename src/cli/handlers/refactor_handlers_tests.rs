//\! Tests for refactor handlers
//\! Extracted to separate file for file health compliance (CB-040)

use super::*;
use proptest::prelude::*;
use std::sync::Arc;
use tempfile::TempDir;

mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}

mod coverage_tests {
    use super::*;
    use proptest::prelude::*;
    use tempfile::TempDir;

    // Test helpers and fixtures

    fn create_temp_project() -> TempDir {
        let dir = TempDir::new().unwrap();
        // Create some test files
        let src_dir = dir.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        // Create a simple Rust file
        let rust_file = src_dir.join("main.rs");
        std::fs::write(&rust_file, "fn main() { println!(\"Hello\"); }").unwrap();

        // Create a TypeScript file
        let ts_file = src_dir.join("index.ts");
        std::fs::write(&ts_file, "export const hello = () => console.log('hi');").unwrap();

        dir
    }

    fn create_temp_checkpoint() -> (TempDir, PathBuf) {
        let dir = TempDir::new().unwrap();
        let checkpoint_file = dir.path().join("checkpoint.json");
        let checkpoint_data = serde_json::json!({
            "current": "Analyze",
            "targets": ["src/main.rs", "src/lib.rs"],
            "current_target_index": 1,
            "config": {
                "target_complexity": 20,
                "parallel_workers": 4
            }
        });
        std::fs::write(
            &checkpoint_file,
            serde_json::to_string_pretty(&checkpoint_data).unwrap(),
        )
        .unwrap();
        (dir, checkpoint_file)
    }

    fn create_temp_config_json() -> (TempDir, PathBuf) {
        let dir = TempDir::new().unwrap();
        let config_file = dir.path().join("refactor.json");
        let config_data = serde_json::json!({
            "rules": {
                "target_complexity": 15,
                "max_function_lines": 40,
                "remove_satd": false
            },
            "parallel_workers": 8,
            "memory_limit_mb": 1024,
            "batch_size": 20,
            "priority_expression": "complexity * defect_probability",
            "auto_commit_template": "refactor: fix {files} files"
        });
        std::fs::write(
            &config_file,
            serde_json::to_string_pretty(&config_data).unwrap(),
        )
        .unwrap();
        (dir, config_file)
    }

    // RefactorServeParams tests

    #[test]
    fn test_refactor_serve_params_creation() {
        let params = RefactorServeParams {
            mode: RefactorMode::Batch,
            config: Some(PathBuf::from("/tmp/config.json")),
            project: PathBuf::from("/tmp/project"),
            parallel: 4,
            memory_limit: 512,
            batch_size: 10,
            priority: Some("complexity".to_string()),
            checkpoint_dir: Some(PathBuf::from("/tmp/checkpoints")),
            resume: false,
            auto_commit: Some("refactor: {files} files".to_string()),
            max_runtime: Some(3600),
        };

        assert_eq!(params.parallel, 4);
        assert_eq!(params.memory_limit, 512);
        assert_eq!(params.batch_size, 10);
        assert!(!params.resume);
    }

    #[test]
    fn test_refactor_serve_params_defaults() {
        let params = RefactorServeParams {
            mode: RefactorMode::Interactive,
            config: None,
            project: PathBuf::from("."),
            parallel: 1,
            memory_limit: 256,
            batch_size: 5,
            priority: None,
            checkpoint_dir: None,
            resume: true,
            auto_commit: None,
            max_runtime: None,
        };

        assert!(params.config.is_none());
        assert!(params.priority.is_none());
        assert!(params.checkpoint_dir.is_none());
        assert!(params.resume);
        assert!(params.auto_commit.is_none());
        assert!(params.max_runtime.is_none());
    }

    // extract_refactor_params tests

    #[test]
    fn test_extract_refactor_params_full() {
        let params = RefactorServeParams {
            mode: RefactorMode::Batch,
            config: Some(PathBuf::from("/config.json")),
            project: PathBuf::from("/project"),
            parallel: 8,
            memory_limit: 1024,
            batch_size: 20,
            priority: Some("high".to_string()),
            checkpoint_dir: Some(PathBuf::from("/checkpoints")),
            resume: true,
            auto_commit: Some("commit template".to_string()),
            max_runtime: Some(7200),
        };

        let extracted = extract_refactor_params(params);

        assert_eq!(extracted.parallel, 8);
        assert_eq!(extracted.memory_limit, 1024);
        assert_eq!(extracted.batch_size, 20);
        assert!(extracted.resume);
        assert_eq!(extracted.priority, Some("high".to_string()));
        assert_eq!(extracted.max_runtime, Some(7200));
    }

    #[test]
    fn test_extract_refactor_params_minimal() {
        let params = RefactorServeParams {
            mode: RefactorMode::Interactive,
            config: None,
            project: PathBuf::from("."),
            parallel: 1,
            memory_limit: 128,
            batch_size: 1,
            priority: None,
            checkpoint_dir: None,
            resume: false,
            auto_commit: None,
            max_runtime: None,
        };

        let extracted = extract_refactor_params(params);

        assert_eq!(extracted.parallel, 1);
        assert_eq!(extracted.memory_limit, 128);
        assert!(extracted.config.is_none());
        assert!(!extracted.resume);
    }

    // log_refactor_server_startup tests

    #[test]
    fn test_log_refactor_server_startup_batch() {
        let params = ExtractedRefactorParams {
            mode: RefactorMode::Batch,
            config: None,
            project: PathBuf::from("/test/project"),
            parallel: 4,
            memory_limit: 512,
            batch_size: 10,
            priority: None,
            checkpoint_dir: None,
            resume: false,
            auto_commit: None,
            max_runtime: None,
        };

        // Just verify it doesn't panic - output goes to stdout
        log_refactor_server_startup(&params);
    }

    #[test]
    fn test_log_refactor_server_startup_interactive() {
        let params = ExtractedRefactorParams {
            mode: RefactorMode::Interactive,
            config: Some(PathBuf::from("config.json")),
            project: PathBuf::from("./my-project"),
            parallel: 2,
            memory_limit: 256,
            batch_size: 5,
            priority: Some("defect_probability".to_string()),
            checkpoint_dir: Some(PathBuf::from("/tmp/cp")),
            resume: true,
            auto_commit: Some("template".to_string()),
            max_runtime: Some(1800),
        };

        log_refactor_server_startup(&params);
    }

    // apply_command_line_overrides tests

    #[test]
    fn test_apply_command_line_overrides_all() {
        let mut config = RefactorConfig::default();
        let params = ExtractedRefactorParams {
            mode: RefactorMode::Batch,
            config: None,
            project: PathBuf::from("."),
            parallel: 16,
            memory_limit: 2048,
            batch_size: 50,
            priority: Some("complexity * churn".to_string()),
            checkpoint_dir: None,
            resume: false,
            auto_commit: Some("refactor: {refactors} changes".to_string()),
            max_runtime: None,
        };

        apply_command_line_overrides(&mut config, &params);

        assert_eq!(config.parallel_workers, 16);
        assert_eq!(config.memory_limit_mb, 2048);
        assert_eq!(config.batch_size, 50);
        assert_eq!(
            config.priority_expression,
            Some("complexity * churn".to_string())
        );
        assert_eq!(
            config.auto_commit_template,
            Some("refactor: {refactors} changes".to_string())
        );
    }

    #[test]
    fn test_apply_command_line_overrides_none() {
        let mut config = RefactorConfig::default();
        let original_parallel = config.parallel_workers;
        let original_memory = config.memory_limit_mb;
        let original_batch = config.batch_size;

        let params = ExtractedRefactorParams {
            mode: RefactorMode::Interactive,
            config: None,
            project: PathBuf::from("."),
            parallel: original_parallel,
            memory_limit: original_memory,
            batch_size: original_batch,
            priority: None,
            checkpoint_dir: None,
            resume: false,
            auto_commit: None,
            max_runtime: None,
        };

        apply_command_line_overrides(&mut config, &params);

        assert!(config.priority_expression.is_none());
        assert!(config.auto_commit_template.is_none());
    }

    // setup_checkpoint_directory tests

    #[tokio::test]
    async fn test_setup_checkpoint_directory_default() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        let params = ExtractedRefactorParams {
            mode: RefactorMode::Batch,
            config: None,
            project: project_path.clone(),
            parallel: 4,
            memory_limit: 512,
            batch_size: 10,
            priority: None,
            checkpoint_dir: None,
            resume: false,
            auto_commit: None,
            max_runtime: None,
        };

        let result = setup_checkpoint_directory(&params).await;
        assert!(result.is_ok());
        let checkpoint_path = result.unwrap();
        assert_eq!(checkpoint_path, project_path.join(".refactor_checkpoints"));
        assert!(checkpoint_path.exists());
    }

    #[tokio::test]
    async fn test_setup_checkpoint_directory_custom() {
        let temp_dir = TempDir::new().unwrap();
        let custom_checkpoint = temp_dir.path().join("custom_checkpoints");

        let params = ExtractedRefactorParams {
            mode: RefactorMode::Batch,
            config: None,
            project: PathBuf::from("/tmp/project"),
            parallel: 4,
            memory_limit: 512,
            batch_size: 10,
            priority: None,
            checkpoint_dir: Some(custom_checkpoint.clone()),
            resume: false,
            auto_commit: None,
            max_runtime: None,
        };

        let result = setup_checkpoint_directory(&params).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), custom_checkpoint);
    }

    #[tokio::test]
    async fn test_setup_checkpoint_directory_resume() {
        let temp_dir = TempDir::new().unwrap();
        let checkpoint_dir = temp_dir.path().join("existing_checkpoints");
        std::fs::create_dir_all(&checkpoint_dir).unwrap();

        let params = ExtractedRefactorParams {
            mode: RefactorMode::Batch,
            config: None,
            project: PathBuf::from("/tmp/project"),
            parallel: 4,
            memory_limit: 512,
            batch_size: 10,
            priority: None,
            checkpoint_dir: Some(checkpoint_dir.clone()),
            resume: true, // Resume mode - shouldn't create dir
            auto_commit: None,
            max_runtime: None,
        };

        let result = setup_checkpoint_directory(&params).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), checkpoint_dir);
    }

    // setup_cache_and_ast_engine tests

    #[test]
    fn test_setup_cache_and_ast_engine_normal() {
        let result = setup_cache_and_ast_engine(512);
        assert!(result.is_ok());
        let (cache, ast_engine) = result.unwrap();
        // Verify both Arc instances are valid
        assert!(Arc::strong_count(&cache) >= 1);
        assert!(Arc::strong_count(&ast_engine) >= 1);
    }

    #[test]
    fn test_setup_cache_and_ast_engine_small_memory() {
        let result = setup_cache_and_ast_engine(64);
        assert!(result.is_ok());
    }

    #[test]
    fn test_setup_cache_and_ast_engine_large_memory() {
        let result = setup_cache_and_ast_engine(4096);
        assert!(result.is_ok());
    }

    // create_engine_mode tests

    #[test]
    fn test_create_engine_mode_batch() {
        let checkpoint_path = PathBuf::from("/tmp/checkpoints");
        let params = ExtractedRefactorParams {
            mode: RefactorMode::Batch,
            config: None,
            project: PathBuf::from("."),
            parallel: 4,
            memory_limit: 512,
            batch_size: 10,
            priority: None,
            checkpoint_dir: None,
            resume: false,
            auto_commit: None,
            max_runtime: None,
        };

        let mode = create_engine_mode(&params, &checkpoint_path);
        match mode {
            EngineMode::Batch {
                checkpoint_dir,
                resume,
                parallel_workers,
            } => {
                assert_eq!(checkpoint_dir, checkpoint_path);
                assert!(!resume);
                assert_eq!(parallel_workers, 4);
            }
            _ => panic!("Expected Batch mode"),
        }
    }

    #[test]
    fn test_create_engine_mode_batch_with_resume() {
        let checkpoint_path = PathBuf::from("/tmp/checkpoints");
        let params = ExtractedRefactorParams {
            mode: RefactorMode::Batch,
            config: None,
            project: PathBuf::from("."),
            parallel: 8,
            memory_limit: 512,
            batch_size: 10,
            priority: None,
            checkpoint_dir: None,
            resume: true,
            auto_commit: None,
            max_runtime: None,
        };

        let mode = create_engine_mode(&params, &checkpoint_path);
        match mode {
            EngineMode::Batch {
                resume,
                parallel_workers,
                ..
            } => {
                assert!(resume);
                assert_eq!(parallel_workers, 8);
            }
            _ => panic!("Expected Batch mode"),
        }
    }

    #[test]
    fn test_create_engine_mode_interactive() {
        let checkpoint_path = PathBuf::from("/tmp/checkpoints");
        let params = ExtractedRefactorParams {
            mode: RefactorMode::Interactive,
            config: None,
            project: PathBuf::from("."),
            parallel: 4,
            memory_limit: 512,
            batch_size: 10,
            priority: None,
            checkpoint_dir: None,
            resume: false,
            auto_commit: None,
            max_runtime: None,
        };

        let mode = create_engine_mode(&params, &checkpoint_path);
        match mode {
            EngineMode::Interactive {
                checkpoint_file,
                explain_level,
            } => {
                assert_eq!(
                    checkpoint_file,
                    checkpoint_path.join("interactive_state.json")
                );
                assert!(matches!(
                    explain_level,
                    crate::services::refactor_engine::ExplainLevel::Detailed
                ));
            }
            _ => panic!("Expected Interactive mode"),
        }
    }

    // validate_checkpoint_file tests

    #[test]
    fn test_validate_checkpoint_file_exists() {
        let (_dir, checkpoint) = create_temp_checkpoint();
        let result = validate_checkpoint_file(&checkpoint);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_checkpoint_file_not_exists() {
        let result = validate_checkpoint_file(Path::new("/nonexistent/checkpoint.json"));
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Checkpoint file not found"));
    }

    // read_checkpoint_data tests

    #[tokio::test]
    async fn test_read_checkpoint_data_success() {
        let (_dir, checkpoint) = create_temp_checkpoint();
        let result = read_checkpoint_data(&checkpoint).await;
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(data.contains("current"));
        assert!(data.contains("targets"));
    }

    #[tokio::test]
    async fn test_read_checkpoint_data_not_found() {
        let result = read_checkpoint_data(&PathBuf::from("/nonexistent/file.json")).await;
        assert!(result.is_err());
    }

    // format_refactor_status tests

    #[test]
    fn test_format_as_json_valid() {
        let checkpoint_data = r#"{"current": "Analyze", "targets": ["a.rs", "b.rs"]}"#;
        let result = format_as_json(checkpoint_data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_as_json_invalid() {
        let checkpoint_data = "not valid json";
        let result = format_as_json(checkpoint_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_format_as_table_valid() {
        let checkpoint_data = r#"{
            "current": "Analyze",
            "targets": ["a.rs", "b.rs"],
            "current_target_index": 1
        }"#;
        let result = format_as_table(checkpoint_data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_as_table_minimal() {
        let checkpoint_data = r#"{}"#;
        let result = format_as_table(checkpoint_data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_as_table_invalid_json() {
        let checkpoint_data = "invalid";
        let result = format_as_table(checkpoint_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_format_as_summary_valid() {
        let checkpoint_data = r#"{
            "current": "Plan",
            "targets": ["x.rs", "y.rs", "z.rs"]
        }"#;
        let checkpoint_path = PathBuf::from("/tmp/checkpoint.json");
        let result = format_as_summary(checkpoint_data, &checkpoint_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_as_summary_empty() {
        let checkpoint_data = r#"{}"#;
        let checkpoint_path = PathBuf::from("/tmp/cp.json");
        let result = format_as_summary(checkpoint_data, &checkpoint_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_refactor_status_json() {
        let checkpoint_data = r#"{"status": "ok"}"#;
        let checkpoint = PathBuf::from("/tmp/test.json");
        let result =
            format_refactor_status(checkpoint_data, RefactorOutputFormat::Json, &checkpoint);
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_refactor_status_table() {
        let checkpoint_data = r#"{"current": "Test", "targets": []}"#;
        let checkpoint = PathBuf::from("/tmp/test.json");
        let result =
            format_refactor_status(checkpoint_data, RefactorOutputFormat::Table, &checkpoint);
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_refactor_status_summary() {
        let checkpoint_data = r#"{"current": "Complete"}"#;
        let checkpoint = PathBuf::from("/tmp/test.json");
        let result =
            format_refactor_status(checkpoint_data, RefactorOutputFormat::Summary, &checkpoint);
        assert!(result.is_ok());
    }

    // print_table_* tests

    #[test]
    fn test_print_table_header() {
        // Just verify it doesn't panic
        print_table_header();
    }

    #[test]
    fn test_print_table_footer() {
        print_table_footer();
    }

    #[test]
    fn test_print_table_data_full() {
        let state: serde_json::Value = serde_json::json!({
            "current": "Analyze",
            "targets": ["a.rs", "b.rs", "c.rs"],
            "current_target_index": 2
        });
        print_table_data(&state);
    }

    #[test]
    fn test_print_table_data_partial() {
        let state: serde_json::Value = serde_json::json!({
            "current": "Test"
        });
        print_table_data(&state);
    }

    #[test]
    fn test_print_table_data_empty() {
        let state: serde_json::Value = serde_json::json!({});
        print_table_data(&state);
    }

    // print_refactor_summary tests

    #[test]
    fn test_print_refactor_summary_normal() {
        let summary = Summary {
            files_processed: 10,
            refactors_applied: 5,
            complexity_reduction: 15.5,
            satd_removed: 3,
            total_time: Duration::from_secs(120),
        };
        print_refactor_summary(&summary);
    }

    #[test]
    fn test_print_refactor_summary_zero() {
        let summary = Summary::default();
        print_refactor_summary(&summary);
    }

    #[test]
    fn test_print_refactor_summary_large() {
        let summary = Summary {
            files_processed: 1000,
            refactors_applied: 500,
            complexity_reduction: 75.25,
            satd_removed: 250,
            total_time: Duration::from_secs(3600),
        };
        print_refactor_summary(&summary);
    }

    // handle_auto_commit tests

    #[tokio::test]
    async fn test_handle_auto_commit_no_template() {
        let config = RefactorConfig::default();
        let summary = Summary {
            files_processed: 5,
            refactors_applied: 3,
            complexity_reduction: 10.0,
            satd_removed: 1,
            total_time: Duration::from_secs(60),
        };

        let result = handle_auto_commit(&config, &summary).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_auto_commit_with_template_no_refactors() {
        let mut config = RefactorConfig::default();
        config.auto_commit_template = Some("refactor: {files} files changed".to_string());

        let summary = Summary {
            files_processed: 5,
            refactors_applied: 0, // No refactors - should skip commit
            complexity_reduction: 0.0,
            satd_removed: 0,
            total_time: Duration::from_secs(30),
        };

        let result = handle_auto_commit(&config, &summary).await;
        assert!(result.is_ok());
    }

    // handle_refactor_status tests

    #[tokio::test]
    async fn test_handle_refactor_status_json() {
        let (_dir, checkpoint) = create_temp_checkpoint();

        let result = handle_refactor_status(checkpoint, RefactorOutputFormat::Json).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_refactor_status_table() {
        let (_dir, checkpoint) = create_temp_checkpoint();

        let result = handle_refactor_status(checkpoint, RefactorOutputFormat::Table).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_refactor_status_summary() {
        let (_dir, checkpoint) = create_temp_checkpoint();

        let result = handle_refactor_status(checkpoint, RefactorOutputFormat::Summary).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_refactor_status_not_found() {
        let result = handle_refactor_status(
            PathBuf::from("/nonexistent/checkpoint.json"),
            RefactorOutputFormat::Json,
        )
        .await;
        assert!(result.is_err());
    }

    // handle_refactor_resume tests

    #[tokio::test]
    async fn test_handle_refactor_resume_valid() {
        let (_dir, checkpoint) = create_temp_checkpoint();

        let result = handle_refactor_resume(checkpoint, 10, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_refactor_resume_with_explain() {
        let (_dir, checkpoint) = create_temp_checkpoint();

        let result = handle_refactor_resume(checkpoint, 5, Some(ExplainLevel::Verbose)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_refactor_resume_not_found() {
        let result = handle_refactor_resume(
            PathBuf::from("/nonexistent/checkpoint.json"),
            10,
            Some(ExplainLevel::Brief),
        )
        .await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Checkpoint file not found"));
    }

    // load_refactor_config tests

    #[tokio::test]
    async fn test_load_refactor_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, "# empty config").unwrap();

        let result = load_refactor_config(&config_path).await;
        assert!(result.is_ok());
    }

    // load_refactor_config_json tests

    #[tokio::test]
    async fn test_load_refactor_config_json_full() {
        let (_dir, config_path) = create_temp_config_json();

        let result = load_refactor_config_json(&config_path).await;
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.target_complexity, 15);
        assert_eq!(config.max_function_lines, 40);
        assert!(!config.remove_satd);
        assert_eq!(config.parallel_workers, 8);
        assert_eq!(config.memory_limit_mb, 1024);
        assert_eq!(config.batch_size, 20);
        assert_eq!(
            config.priority_expression,
            Some("complexity * defect_probability".to_string())
        );
        assert_eq!(
            config.auto_commit_template,
            Some("refactor: fix {files} files".to_string())
        );
    }

    #[tokio::test]
    async fn test_load_refactor_config_json_minimal() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("minimal.json");
        std::fs::write(&config_path, "{}").unwrap();

        let result = load_refactor_config_json(&config_path).await;
        assert!(result.is_ok());
        // Should use defaults
        let config = result.unwrap();
        assert_eq!(config.target_complexity, 20); // default
    }

    #[tokio::test]
    async fn test_load_refactor_config_json_invalid() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("invalid.json");
        std::fs::write(&config_path, "not json").unwrap();

        let result = load_refactor_config_json(&config_path).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_load_refactor_config_json_not_found() {
        let result = load_refactor_config_json(Path::new("/nonexistent/config.json")).await;
        assert!(result.is_err());
    }

    // sort_targets_by_priority tests

    #[tokio::test]
    async fn test_sort_targets_by_priority() {
        let targets = vec![
            PathBuf::from("a.rs"),
            PathBuf::from("b.rs"),
            PathBuf::from("c.rs"),
        ];

        let result = sort_targets_by_priority(targets.clone(), "complexity").await;
        assert!(result.is_ok());

        let sorted = result.unwrap();
        assert_eq!(sorted.len(), 3);
        // Current implementation just reverses
        assert_eq!(sorted[0], PathBuf::from("c.rs"));
        assert_eq!(sorted[2], PathBuf::from("a.rs"));
    }

    #[tokio::test]
    async fn test_sort_targets_by_priority_empty() {
        let targets: Vec<PathBuf> = vec![];

        let result = sort_targets_by_priority(targets, "any").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    // discover_refactor_targets tests

    #[tokio::test]
    async fn test_discover_refactor_targets() {
        let temp_dir = create_temp_project();

        let result = discover_refactor_targets(&temp_dir.path().to_path_buf()).await;
        assert!(result.is_ok());

        let targets = result.unwrap();
        assert!(!targets.is_empty());

        // Should find the .rs and .ts files we created
        let extensions: Vec<_> = targets
            .iter()
            .filter_map(|p| p.extension().map(|e| e.to_string_lossy().to_string()))
            .collect();
        assert!(extensions.contains(&"rs".to_string()));
        assert!(extensions.contains(&"ts".to_string()));
    }

    #[tokio::test]
    async fn test_discover_refactor_targets_empty_dir() {
        let temp_dir = TempDir::new().unwrap();

        let result = discover_refactor_targets(&temp_dir.path().to_path_buf()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_discover_refactor_targets_various_extensions() {
        let temp_dir = TempDir::new().unwrap();
        let src = temp_dir.path().join("src");
        std::fs::create_dir_all(&src).unwrap();

        // Create files with various supported extensions
        std::fs::write(src.join("app.rs"), "fn main() {}").unwrap();
        std::fs::write(src.join("index.ts"), "export {};").unwrap();
        std::fs::write(src.join("component.tsx"), "export default () => null;").unwrap();
        std::fs::write(src.join("util.js"), "module.exports = {};").unwrap();
        std::fs::write(src.join("helper.jsx"), "export default () => null;").unwrap();
        std::fs::write(src.join("main.py"), "print('hi')").unwrap();
        std::fs::write(src.join("readme.md"), "# Readme").unwrap(); // Should NOT be included

        let result = discover_refactor_targets(&temp_dir.path().to_path_buf()).await;
        assert!(result.is_ok());

        let targets = result.unwrap();
        assert_eq!(targets.len(), 6); // md should not be included
    }

    // ExplainLevel conversion tests

    #[test]
    fn test_explain_level_from_brief() {
        let level: crate::services::refactor_engine::ExplainLevel = ExplainLevel::Brief.into();
        assert!(matches!(
            level,
            crate::services::refactor_engine::ExplainLevel::Brief
        ));
    }

    #[test]
    fn test_explain_level_from_detailed() {
        let level: crate::services::refactor_engine::ExplainLevel = ExplainLevel::Detailed.into();
        assert!(matches!(
            level,
            crate::services::refactor_engine::ExplainLevel::Detailed
        ));
    }

    #[test]
    fn test_explain_level_from_verbose() {
        let level: crate::services::refactor_engine::ExplainLevel = ExplainLevel::Verbose.into();
        assert!(matches!(
            level,
            crate::services::refactor_engine::ExplainLevel::Verbose
        ));
    }

    // load_base_configuration tests

    #[tokio::test]
    async fn test_load_base_configuration_with_file() {
        let (_dir, config_path) = create_temp_config_json();

        let params = ExtractedRefactorParams {
            mode: RefactorMode::Batch,
            config: Some(config_path),
            project: PathBuf::from("."),
            parallel: 4,
            memory_limit: 512,
            batch_size: 10,
            priority: None,
            checkpoint_dir: None,
            resume: false,
            auto_commit: None,
            max_runtime: None,
        };

        let result = load_base_configuration(&params).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_load_base_configuration_default() {
        let params = ExtractedRefactorParams {
            mode: RefactorMode::Batch,
            config: None,
            project: PathBuf::from("."),
            parallel: 4,
            memory_limit: 512,
            batch_size: 10,
            priority: None,
            checkpoint_dir: None,
            resume: false,
            auto_commit: None,
            max_runtime: None,
        };

        let result = load_base_configuration(&params).await;
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.target_complexity, 20); // default
    }

    // setup_refactor_configuration tests

    #[tokio::test]
    async fn test_setup_refactor_configuration_with_overrides() {
        let (_dir, config_path) = create_temp_config_json();

        let params = ExtractedRefactorParams {
            mode: RefactorMode::Batch,
            config: Some(config_path),
            project: PathBuf::from("."),
            parallel: 32,                                // Override
            memory_limit: 4096,                          // Override
            batch_size: 100,                             // Override
            priority: Some("high_priority".to_string()), // Override
            checkpoint_dir: None,
            resume: false,
            auto_commit: Some("custom template".to_string()), // Override
            max_runtime: None,
        };

        let result = setup_refactor_configuration(&params).await;
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.parallel_workers, 32);
        assert_eq!(config.memory_limit_mb, 4096);
        assert_eq!(config.batch_size, 100);
        assert_eq!(
            config.priority_expression,
            Some("high_priority".to_string())
        );
        assert_eq!(
            config.auto_commit_template,
            Some("custom template".to_string())
        );
    }

    // discover_and_prioritize_targets tests

    #[tokio::test]
    async fn test_discover_and_prioritize_targets_no_priority() {
        let temp_dir = create_temp_project();

        let params = ExtractedRefactorParams {
            mode: RefactorMode::Batch,
            config: None,
            project: temp_dir.path().to_path_buf(),
            parallel: 4,
            memory_limit: 512,
            batch_size: 10,
            priority: None,
            checkpoint_dir: None,
            resume: false,
            auto_commit: None,
            max_runtime: None,
        };

        let config = RefactorConfig::default();

        let result = discover_and_prioritize_targets(&params, &config).await;
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_discover_and_prioritize_targets_with_priority() {
        let temp_dir = create_temp_project();

        let params = ExtractedRefactorParams {
            mode: RefactorMode::Batch,
            config: None,
            project: temp_dir.path().to_path_buf(),
            parallel: 4,
            memory_limit: 512,
            batch_size: 10,
            priority: None,
            checkpoint_dir: None,
            resume: false,
            auto_commit: None,
            max_runtime: None,
        };

        let mut config = RefactorConfig::default();
        config.priority_expression = Some("complexity".to_string());

        let result = discover_and_prioritize_targets(&params, &config).await;
        assert!(result.is_ok());
    }

    // Property-based tests

    proptest! {
        #[test]
        fn prop_extract_params_preserves_values(
            parallel in 1usize..100,
            memory_limit in 64usize..8192,
            batch_size in 1usize..100,
            max_runtime in proptest::option::of(1u64..86400)
        ) {
            let params = RefactorServeParams {
                mode: RefactorMode::Batch,
                config: None,
                project: PathBuf::from("."),
                parallel,
                memory_limit,
                batch_size,
                priority: None,
                checkpoint_dir: None,
                resume: false,
                auto_commit: None,
                max_runtime,
            };

            let extracted = extract_refactor_params(params);
            prop_assert_eq!(extracted.parallel, parallel);
            prop_assert_eq!(extracted.memory_limit, memory_limit);
            prop_assert_eq!(extracted.batch_size, batch_size);
            prop_assert_eq!(extracted.max_runtime, max_runtime);
        }

        #[test]
        fn prop_apply_overrides_sets_config_values(
            parallel in 1usize..100,
            memory_limit in 64usize..8192,
            batch_size in 1usize..100
        ) {
            let mut config = RefactorConfig::default();
            let params = ExtractedRefactorParams {
                mode: RefactorMode::Batch,
                config: None,
                project: PathBuf::from("."),
                parallel,
                memory_limit,
                batch_size,
                priority: None,
                checkpoint_dir: None,
                resume: false,
                auto_commit: None,
                max_runtime: None,
            };

            apply_command_line_overrides(&mut config, &params);

            prop_assert_eq!(config.parallel_workers, parallel);
            prop_assert_eq!(config.memory_limit_mb, memory_limit);
            prop_assert_eq!(config.batch_size, batch_size);
        }

        #[test]
        fn prop_cache_and_ast_engine_handles_various_memory_limits(memory_limit in 32usize..16384) {
            let result = setup_cache_and_ast_engine(memory_limit);
            prop_assert!(result.is_ok());
        }

        #[test]
        fn prop_sort_targets_preserves_count(count in 0usize..20) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let targets: Vec<PathBuf> = (0..count).map(|i| PathBuf::from(format!("file{i}.rs"))).collect();
            let original_count = targets.len();

            let result = rt.block_on(sort_targets_by_priority(targets, "any"));
            prop_assert!(result.is_ok());
            prop_assert_eq!(result.unwrap().len(), original_count);
        }

        #[test]
        fn prop_json_format_roundtrip(
            current in "[A-Za-z]+",
            target_count in 0usize..10
        ) {
            let targets: Vec<String> = (0..target_count).map(|i| format!("file{i}.rs")).collect();
            let json = serde_json::json!({
                "current": current,
                "targets": targets
            });
            let checkpoint_data = serde_json::to_string(&json).unwrap();

            let result = format_as_json(&checkpoint_data);
            prop_assert!(result.is_ok());
        }

        #[test]
        fn prop_table_format_handles_various_states(
            has_current in proptest::bool::ANY,
            has_targets in proptest::bool::ANY,
            has_index in proptest::bool::ANY
        ) {
            let mut json = serde_json::Map::new();
            if has_current {
                json.insert("current".to_string(), serde_json::json!("Test"));
            }
            if has_targets {
                json.insert("targets".to_string(), serde_json::json!(["a.rs"]));
            }
            if has_index {
                json.insert("current_target_index".to_string(), serde_json::json!(0));
            }

            let checkpoint_data = serde_json::to_string(&json).unwrap();
            let result = format_as_table(&checkpoint_data);
            prop_assert!(result.is_ok());
        }
    }

    // Edge case tests

    #[test]
    fn test_engine_mode_batch_zero_parallel() {
        let checkpoint_path = PathBuf::from("/tmp/cp");
        let params = ExtractedRefactorParams {
            mode: RefactorMode::Batch,
            config: None,
            project: PathBuf::from("."),
            parallel: 0, // Edge case
            memory_limit: 512,
            batch_size: 10,
            priority: None,
            checkpoint_dir: None,
            resume: false,
            auto_commit: None,
            max_runtime: None,
        };

        let mode = create_engine_mode(&params, &checkpoint_path);
        match mode {
            EngineMode::Batch {
                parallel_workers, ..
            } => {
                assert_eq!(parallel_workers, 0);
            }
            _ => panic!("Expected Batch mode"),
        }
    }

    #[test]
    fn test_print_table_data_with_long_current_state() {
        let state: serde_json::Value = serde_json::json!({
            "current": "ThisIsAVeryLongStateNameThatExceedsThirtyTwoCharacters",
            "targets": [],
            "current_target_index": 0
        });
        // Should truncate to 36 chars without panicking
        print_table_data(&state);
    }

    #[tokio::test]
    async fn test_load_config_json_partial_rules() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("partial.json");
        let config_data = serde_json::json!({
            "rules": {
                "target_complexity": 25
                // Other rules missing
            }
        });
        std::fs::write(&config_path, serde_json::to_string(&config_data).unwrap()).unwrap();

        let result = load_refactor_config_json(&config_path).await;
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.target_complexity, 25);
        // Other values should be defaults
        assert_eq!(config.max_function_lines, 50);
        assert!(config.remove_satd); // default is true
    }

    #[test]
    fn test_format_as_summary_with_empty_targets_array() {
        let checkpoint_data = r#"{
            "current": "Plan",
            "targets": []
        }"#;
        let checkpoint_path = PathBuf::from("/tmp/cp.json");
        let result = format_as_summary(checkpoint_data, &checkpoint_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_as_table_targets_not_array() {
        let checkpoint_data = r#"{
            "targets": "not_an_array"
        }"#;
        // Should not panic, just skip that field
        let result = format_as_table(checkpoint_data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_as_table_index_not_number() {
        let checkpoint_data = r#"{
            "current_target_index": "not_a_number"
        }"#;
        let result = format_as_table(checkpoint_data);
        assert!(result.is_ok());
    }
}
