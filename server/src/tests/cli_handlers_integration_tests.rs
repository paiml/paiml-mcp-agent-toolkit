#[cfg(test)]
mod cli_handlers_integration_tests {
    use crate::cli::handlers::*;
    use crate::stateless_server::StatelessTemplateServer;
    use crate::cli::{OutputFormat, ContextFormat};
    use std::sync::Arc;
    use tempfile::TempDir;
    use serde_json::json;
    use std::path::PathBuf;

    fn create_test_server() -> Arc<StatelessTemplateServer> {
        Arc::new(StatelessTemplateServer::new().unwrap())
    }

    fn create_test_project() -> TempDir {
        let test_dir = TempDir::new().unwrap();
        let src_dir = test_dir.path().join("src");
        std::fs::create_dir(&src_dir).unwrap();
        
        // Create main.rs
        std::fs::write(
            src_dir.join("main.rs"),
            r#"
            mod utils;
            use utils::helper;
            
            fn main() {
                println!("Hello, world!");
                let result = helper();
                complex_function(result);
            }
            
            fn complex_function(x: i32) -> i32 {
                match x {
                    0 => 0,
                    1..=10 => {
                        if x % 2 == 0 {
                            x * 2
                        } else {
                            x + 1
                        }
                    },
                    11..=100 => {
                        for i in 0..x {
                            if i % 3 == 0 {
                                println!("Fizz: {}", i);
                            }
                        }
                        x / 2
                    },
                    _ => -1
                }
            }
            "#
        ).unwrap();
        
        // Create utils.rs
        std::fs::write(
            src_dir.join("utils.rs"),
            r#"
            pub fn helper() -> i32 {
                42
            }
            
            pub fn duplicate_logic(x: i32) -> i32 {
                if x > 0 {
                    x * 2
                } else {
                    0
                }
            }
            
            pub fn similar_logic(x: i32) -> i32 {
                if x > 0 {
                    x * 2
                } else {
                    0
                }
            }
            "#
        ).unwrap();
        
        // Create Cargo.toml
        std::fs::write(
            test_dir.path().join("Cargo.toml"),
            r#"
            [package]
            name = "test-project"
            version = "0.1.0"
            edition = "2021"
            "#
        ).unwrap();
        
        // Create README.md
        std::fs::write(
            test_dir.path().join("README.md"),
            "# Test Project\nThis is a test project for CLI handlers."
        ).unwrap();
        
        test_dir
    }

    #[tokio::test]
    async fn test_handle_generate() {
        let server = create_test_server();
        let test_dir = TempDir::new().unwrap();
        let output_path = test_dir.path().join("test.gitignore");

        let result = handle_generate(
            server,
            "gitignore".to_string(),
            "rust/cli".to_string(),
            vec![("project_name".to_string(), json!("test-project"))],
            Some(output_path.clone()),
            true,
        ).await;

        assert!(result.is_ok(), "handle_generate failed: {:?}", result);
        assert!(output_path.exists(), "Output file was not created");
        
        let content = std::fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("target/") || content.contains("Cargo.lock"));
    }

    #[tokio::test]
    async fn test_handle_scaffold() {
        let server = create_test_server();

        let result = handle_scaffold(
            server,
            "rust".to_string(),
            vec!["gitignore".to_string()],
            vec![("project_name".to_string(), json!("test-scaffold"))],
            1,
        ).await;

        assert!(result.is_ok(), "handle_scaffold failed: {:?}", result);
    }

    #[tokio::test]
    async fn test_handle_scaffold_parallel() {
        let server = create_test_server();

        let result = handle_scaffold(
            server,
            "rust".to_string(),
            vec!["gitignore".to_string(), "makefile".to_string()],
            vec![("project_name".to_string(), json!("test-scaffold"))],
            2, // parallel = 2
        ).await;

        assert!(result.is_ok(), "handle_scaffold with parallel failed: {:?}", result);
    }

    #[tokio::test]
    async fn test_handle_list() {
        let server = create_test_server();

        let result = handle_list(server, None, None, OutputFormat::Table).await;
        assert!(result.is_ok(), "handle_list failed: {:?}", result);
    }

    #[tokio::test]
    async fn test_handle_list_with_toolchain() {
        let server = create_test_server();

        let result = handle_list(
            server, 
            Some("rust".to_string()), 
            None, 
            OutputFormat::Json
        ).await;
        assert!(result.is_ok(), "handle_list with toolchain failed: {:?}", result);
    }

    #[tokio::test]
    async fn test_handle_list_with_category() {
        let server = create_test_server();

        let result = handle_list(
            server, 
            None, 
            Some("gitignore".to_string()), 
            OutputFormat::Yaml
        ).await;
        assert!(result.is_ok(), "handle_list with category failed: {:?}", result);
    }

    #[tokio::test]
    async fn test_handle_search() {
        let server = create_test_server();

        let result = handle_search(
            server,
            "rust".to_string(),
            None,
            10,
        ).await;

        assert!(result.is_ok(), "handle_search failed: {:?}", result);
    }

    #[tokio::test]
    async fn test_handle_search_with_toolchain() {
        let server = create_test_server();

        let result = handle_search(
            server,
            "cli".to_string(),
            Some("rust".to_string()),
            5,
        ).await;

        assert!(result.is_ok(), "handle_search with toolchain failed: {:?}", result);
    }

    #[tokio::test]
    async fn test_handle_validate() {
        let server = create_test_server();

        let result = handle_validate(
            server,
            "rust/cli/gitignore".to_string(),
            vec![("project_name".to_string(), json!("test"))],
        ).await;

        assert!(result.is_ok(), "handle_validate failed: {:?}", result);
    }

    #[tokio::test]
    async fn test_handle_validate_invalid_uri() {
        let server = create_test_server();

        let result = handle_validate(
            server,
            "invalid/template/uri".to_string(),
            vec![],
        ).await;

        // Should handle invalid URI gracefully
        assert!(result.is_err(), "Expected error for invalid URI");
    }

    #[tokio::test]
    async fn test_handle_context() {
        let test_dir = create_test_project();

        let result = handle_context(
            None,
            test_dir.path().to_path_buf(),
            None,
            ContextFormat::Markdown,
        ).await;

        assert!(result.is_ok(), "handle_context failed: {:?}", result);
    }

    #[tokio::test]
    async fn test_handle_context_with_output() {
        let test_dir = create_test_project();
        let output_dir = TempDir::new().unwrap();
        let output_path = output_dir.path().join("context.md");

        let result = handle_context(
            None,
            test_dir.path().to_path_buf(),
            Some(output_path.clone()),
            ContextFormat::Markdown,
        ).await;

        assert!(result.is_ok(), "handle_context with output failed: {:?}", result);
        assert!(output_path.exists(), "Context output file was not created");
    }

    #[tokio::test]
    async fn test_handle_context_json_format() {
        let test_dir = create_test_project();

        let result = handle_context(
            None,
            test_dir.path().to_path_buf(),
            None,
            ContextFormat::Json,
        ).await;

        assert!(result.is_ok(), "handle_context JSON format failed: {:?}", result);
    }

    #[tokio::test]
    async fn test_handle_context_with_toolchain() {
        let test_dir = create_test_project();

        let result = handle_context(
            Some("rust".to_string()),
            test_dir.path().to_path_buf(),
            None,
            ContextFormat::Markdown,
        ).await;

        assert!(result.is_ok(), "handle_context with toolchain failed: {:?}", result);
    }

    #[tokio::test]
    async fn test_error_handling_nonexistent_path() {
        let result = handle_context(
            None,
            PathBuf::from("/non/existent/path"),
            None,
            ContextFormat::Markdown,
        ).await;

        assert!(result.is_err(), "Expected error for non-existent path");
    }

    #[tokio::test]
    async fn test_generate_to_stdout() {
        let server = create_test_server();

        let result = handle_generate(
            server,
            "gitignore".to_string(),
            "rust/cli".to_string(),
            vec![("project_name".to_string(), json!("test-project"))],
            None, // output to stdout
            false,
        ).await;

        assert!(result.is_ok(), "handle_generate to stdout failed: {:?}", result);
    }

    #[tokio::test]
    async fn test_generate_with_create_dirs() {
        let server = create_test_server();
        let test_dir = TempDir::new().unwrap();
        let nested_path = test_dir.path().join("deep").join("nested").join("test.gitignore");

        let result = handle_generate(
            server,
            "gitignore".to_string(),
            "rust/cli".to_string(),
            vec![("project_name".to_string(), json!("test-project"))],
            Some(nested_path.clone()),
            true, // create_dirs = true
        ).await;

        assert!(result.is_ok(), "handle_generate with create_dirs failed: {:?}", result);
        assert!(nested_path.exists(), "Nested output file was not created");
    }

    #[tokio::test]
    async fn test_generate_without_create_dirs() {
        let server = create_test_server();
        let test_dir = TempDir::new().unwrap();
        let nested_path = test_dir.path().join("deep").join("nested").join("test.gitignore");

        let result = handle_generate(
            server,
            "gitignore".to_string(),
            "rust/cli".to_string(),
            vec![("project_name".to_string(), json!("test-project"))],
            Some(nested_path.clone()),
            false, // create_dirs = false
        ).await;

        // Should fail because parent directory doesn't exist
        assert!(result.is_err(), "Expected error when parent directory doesn't exist");
    }

    #[tokio::test]
    async fn test_concurrent_handler_operations() {
        use tokio::task;
        
        let server = create_test_server();
        let test_dir = create_test_project();
        
        let handles: Vec<_> = (0..3).map(|i| {
            let server_clone = Arc::clone(&server);
            let path = test_dir.path().to_path_buf();
            
            task::spawn(async move {
                match i % 3 {
                    0 => handle_list(server_clone, None, None, OutputFormat::Json).await,
                    1 => handle_search(server_clone, "rust".to_string(), None, 5).await,
                    _ => handle_context(None, path, None, ContextFormat::Json).await,
                }
            })
        }).collect();

        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok(), "Concurrent handler operation failed");
        }
    }

    #[tokio::test]
    async fn test_error_propagation() {
        let server = create_test_server();

        // Test with invalid template that should cause an error
        let result = handle_generate(
            server,
            "invalid_category".to_string(),
            "invalid/template".to_string(),
            vec![],
            None,
            false,
        ).await;

        assert!(result.is_err(), "Expected error for invalid template");
    }
}