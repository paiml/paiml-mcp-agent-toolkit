use crate::cli::{
    command_dispatcher::CommandDispatcher,
    commands::{AnalyzeCommands, Commands},
    detect_primary_language, ComplexityOutputFormat, OutputFormat,
};
use crate::stateless_server::StatelessTemplateServer;
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;

fn create_test_server() -> Arc<StatelessTemplateServer> {
    Arc::new(StatelessTemplateServer::new().unwrap())
}

fn create_test_dir_with_rust_file() -> TempDir {
    let test_dir = TempDir::new().unwrap();
    let src_dir = test_dir.path().join("src");
    std::fs::create_dir(&src_dir).unwrap();

    std::fs::write(
        src_dir.join("lib.rs"),
        r#"
        pub fn test_function() {
            println!("test");
        }
        "#,
    )
    .unwrap();

    test_dir
}

#[tokio::test]
async fn test_execute_generate_command_basic() {
    let server = create_test_server();
    let test_dir = TempDir::new().unwrap();
    let output_path = test_dir.path().join("test.gitignore");

    let command = Commands::Generate {
        category: "gitignore".to_string(),
        template: "rust/cli".to_string(),
        params: vec![("project_name".to_string(), json!("test-project"))],
        output: Some(output_path.clone()),
        create_dirs: true,
    };

    let result = CommandDispatcher::execute_command(command, server).await;
    assert!(result.is_ok(), "Generate command failed: {:?}", result);
}

#[tokio::test]
async fn test_execute_list_command_basic() {
    let server = create_test_server();

    let command = Commands::List {
        toolchain: None,
        category: None,
        format: OutputFormat::Table,
    };

    let result = CommandDispatcher::execute_command(command, server).await;
    assert!(result.is_ok(), "List command failed: {:?}", result);
}

#[tokio::test]
async fn test_execute_search_command_basic() {
    let server = create_test_server();

    let command = Commands::Search {
        query: "rust".to_string(),
        toolchain: None,
        limit: 10,
    };

    let result = CommandDispatcher::execute_command(command, server).await;
    assert!(result.is_ok(), "Search command failed: {:?}", result);
}

#[tokio::test]
async fn test_execute_analyze_complexity_basic() {
    let test_dir = create_test_dir_with_rust_file();

    let analyze_cmd = AnalyzeCommands::Complexity {
        project_path: test_dir.path().to_path_buf(),
        toolchain: None,
        format: ComplexityOutputFormat::Summary,
        output: None,
        max_cyclomatic: None,
        max_cognitive: None,
        include: vec![],
        watch: false,
        top_files: 10,
    };

    let command = Commands::Analyze(analyze_cmd);
    let result = CommandDispatcher::execute_command(command, create_test_server()).await;
    assert!(
        result.is_ok(),
        "Analyze complexity command failed: {:?}",
        result
    );
}

#[test]
fn test_detect_primary_language_rust() {
    let test_dir = TempDir::new().unwrap();
    let src_dir = test_dir.path().join("src");
    std::fs::create_dir(&src_dir).unwrap();

    // Create multiple Rust files
    for i in 0..5 {
        std::fs::write(
            src_dir.join(format!("mod{}.rs", i)),
            format!("pub fn function{}() {{}}", i),
        )
        .unwrap();
    }

    let language = detect_primary_language(test_dir.path());
    assert_eq!(language, Some("rust".to_string()));
}

#[test]
fn test_detect_primary_language_empty() {
    let test_dir = TempDir::new().unwrap();
    let language = detect_primary_language(test_dir.path());
    assert_eq!(language, None);
}

#[test]
fn test_command_dispatcher_trait_bounds() {
    // Test that the dispatcher compiles and has proper trait bounds
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<crate::cli::command_dispatcher::CommandDispatcher>();
}

#[test]
fn test_output_format_display() {
    assert_eq!(format!("{}", OutputFormat::Table), "table");
    assert_eq!(format!("{}", OutputFormat::Json), "json");
    assert_eq!(format!("{}", OutputFormat::Yaml), "yaml");
}

#[test]
fn test_complexity_output_format_display() {
    assert_eq!(format!("{}", ComplexityOutputFormat::Summary), "summary");
    assert_eq!(format!("{}", ComplexityOutputFormat::Full), "full");
    assert_eq!(format!("{}", ComplexityOutputFormat::Json), "json");
    assert_eq!(format!("{}", ComplexityOutputFormat::Sarif), "sarif");
}

#[tokio::test]
async fn test_error_handling_invalid_generate() {
    let server = create_test_server();

    // Test with invalid template that should cause an error
    let command = Commands::Generate {
        category: "invalid_category".to_string(),
        template: "invalid/template".to_string(),
        params: vec![],
        output: None,
        create_dirs: false,
    };

    let result = CommandDispatcher::execute_command(command, server).await;
    assert!(result.is_err(), "Expected error for invalid template");
}

#[tokio::test]
async fn test_multiple_commands_sequential() {
    let server = create_test_server();

    // Test multiple commands in sequence
    let commands = vec![
        Commands::List {
            toolchain: None,
            category: None,
            format: OutputFormat::Json,
        },
        Commands::Search {
            query: "test".to_string(),
            toolchain: None,
            limit: 5,
        },
    ];

    for command in commands {
        let result = CommandDispatcher::execute_command(command, Arc::clone(&server)).await;
        assert!(result.is_ok(), "Sequential command execution failed");
    }
}

#[test]
fn test_tempdir_creation() {
    // Basic test to ensure our test infrastructure works
    let test_dir = create_test_dir_with_rust_file();
    assert!(test_dir.path().exists());

    let src_dir = test_dir.path().join("src");
    assert!(src_dir.exists());

    let lib_file = src_dir.join("lib.rs");
    assert!(lib_file.exists());
}

#[test]
fn test_server_creation() {
    // Test that we can create a server instance
    let server = create_test_server();
    assert_eq!(Arc::strong_count(&server), 1);
}
