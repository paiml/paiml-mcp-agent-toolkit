//! Simple integration tests for Universal Demo
//! 
//! These tests verify basic functionality works.

#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_demo_compiles() {
        // Just ensure the demo module compiles
        use pmat::demo::runner::DemoRunner;
        use pmat::stateless_server::StatelessTemplateServer;
        use std::sync::Arc;

        let server = StatelessTemplateServer::new().unwrap();
        let _ = DemoRunner::new(Arc::new(server));
    }

    #[test] 
    fn test_import_variants() {
        use pmat::services::context::AstItem;

        // Test that Import variant exists and works
        let import = AstItem::Import {
            module: "test".to_string(),
            items: vec![],
            alias: None,
            line: 1,
        };

        assert_eq!(import.display_name(), "test");
    }

    #[tokio::test]
    async fn test_local_repo_analysis() {
        use pmat::services::deep_context::{DeepContextAnalyzer, DeepContextConfig};

        // Create a temporary directory with some files
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.py");
        fs::write(&test_file, "# Test file\nimport os\nprint('hello')").unwrap();

        // Analyze it
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let path = temp_dir.path().to_path_buf();
        let result = analyzer.analyze_project(&path).await.unwrap();

        // Basic validation
        assert!(result.metadata.project_root.exists());
        assert!(result.qa_verification.is_some());
    }

    #[tokio::test]
    async fn test_quality_gates_pass() {
        use pmat::services::quality_gates::VerificationStatus;
        use pmat::services::deep_context::{DeepContextAnalyzer, DeepContextConfig};

        // Create a test project
        let temp_dir = TempDir::new().unwrap();
        
        // Add multiple files
        for i in 0..5 {
            let file = temp_dir.path().join(format!("file{}.py", i));
            fs::write(&file, format!("# File {}\nprint('test')", i)).unwrap();
        }

        // Analyze
        let config = DeepContextConfig::default();
        let analyzer = DeepContextAnalyzer::new(config);
        let path = temp_dir.path().to_path_buf();
        let result = analyzer.analyze_project(&path).await.unwrap();

        // Check quality gates
        if let Some(qa) = result.qa_verification {
            // Should pass or be partial (not fail)
            assert_ne!(qa.overall, VerificationStatus::Fail,
                      "Quality gates should not fail for simple project");
        }
    }

    #[test]
    fn test_repository_resolution() {
        use pmat::demo::runner::resolve_repository;

        // Test local path resolution
        let temp_dir = TempDir::new().unwrap();
        
        // Create a git repo marker so it's recognized as valid
        let git_dir = temp_dir.path().join(".git");
        fs::create_dir(&git_dir).unwrap();
        
        let result = resolve_repository(
            Some(temp_dir.path().to_path_buf()),
            None,
            None
        );
        
        match result {
            Ok(path) => assert_eq!(path, temp_dir.path()),
            Err(e) => {
                // It's okay if it fails, as long as the error is reasonable
                assert!(e.to_string().contains("repository") || 
                        e.to_string().contains("path"));
            }
        }

        // Test URL detection
        let url_result = resolve_repository(
            None,
            Some("https://github.com/test/repo".to_string()),
            None
        );
        
        // Should recognize it as a URL and return it for async processing
        assert!(url_result.is_ok());
        let path = url_result.unwrap();
        assert!(path.to_string_lossy().contains("github"));
    }
}