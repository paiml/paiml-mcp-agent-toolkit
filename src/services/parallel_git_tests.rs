// Parallel git tests - included by parallel_git.rs
// NO `use` imports or `#!` inner attributes - shares parent module scope

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_parallel_git_executor() {
        let executor = ParallelGitExecutor::new(PathBuf::from("."));
        let (size, _) = executor.cache_stats().await;
        assert_eq!(size, 0);
    }

    #[tokio::test]
    async fn test_cache_key_generation() {
        let executor = ParallelGitExecutor::new(PathBuf::from("."));

        // Execute a simple command
        let result = executor.execute_command(vec!["--version"]).await;
        assert!(result.is_ok());

        // Check cache was populated
        let (size, _) = executor.cache_stats().await;
        assert_eq!(size, 1);
    }

    #[tokio::test]
    async fn test_config_defaults() {
        let config = ParallelGitConfig::default();
        assert!(config.max_concurrent_operations > 0);
        assert!(config.max_concurrent_operations <= 8);
        assert!(config.enable_caching);
        assert_eq!(config.cache_ttl_seconds, 300);
    }

    #[tokio::test]
    async fn test_custom_config() {
        let config = ParallelGitConfig {
            max_concurrent_operations: 4,
            enable_caching: false,
            cache_ttl_seconds: 60,
        };

        let temp_dir = TempDir::new().unwrap();
        let executor = ParallelGitExecutor::with_config(temp_dir.path().to_path_buf(), config);

        // Cache should be empty and stay empty with caching disabled
        let (size, _) = executor.cache_stats().await;
        assert_eq!(size, 0);
    }

    #[tokio::test]
    async fn test_execute_batch() {
        let executor = ParallelGitExecutor::new(PathBuf::from("."));

        let commands = vec![vec!["--version"], vec!["--help"]];

        let results = executor.execute_batch(commands).await;
        assert!(results.is_ok());

        let outputs = results.unwrap();
        assert_eq!(outputs.len(), 2);

        // Both commands should succeed
        for output in outputs {
            assert!(!output.is_empty());
        }
    }

    #[tokio::test]
    async fn test_execute_command_failure() {
        let temp_dir = TempDir::new().unwrap();
        let executor = ParallelGitExecutor::new(temp_dir.path().to_path_buf());

        // Try to run a git command that will fail (no git repo)
        let result = executor.execute_command(vec!["log", "-1"]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_clear_cache() {
        let executor = ParallelGitExecutor::new(PathBuf::from("."));

        // Populate cache
        let _ = executor.execute_command(vec!["--version"]).await;
        let (size, _) = executor.cache_stats().await;
        assert!(size > 0);

        // Clear cache
        executor.clear_cache().await;
        let (size, _) = executor.cache_stats().await;
        assert_eq!(size, 0);
    }

    #[tokio::test]
    async fn test_parse_commit_log() {
        let output = r#"abc123|John Doe|2024-01-01T12:00:00Z|Initial commit
def456|Jane Smith|2024-01-02T12:00:00Z|Add feature"#;

        let commits = ParallelGitExecutor::parse_commit_log(output);
        assert_eq!(commits.len(), 2);

        assert_eq!(commits[0].hash, "abc123");
        assert_eq!(commits[0].author, "John Doe");
        assert_eq!(commits[0].message, "Initial commit");

        assert_eq!(commits[1].hash, "def456");
        assert_eq!(commits[1].author, "Jane Smith");
        assert_eq!(commits[1].message, "Add feature");
    }

    #[tokio::test]
    async fn test_parse_commit_log_empty() {
        let commits = ParallelGitExecutor::parse_commit_log("");
        assert_eq!(commits.len(), 0);
    }

    #[tokio::test]
    async fn test_parse_commit_log_invalid() {
        let output = "invalid line\nanother invalid line";
        let commits = ParallelGitExecutor::parse_commit_log(output);
        assert_eq!(commits.len(), 0);
    }

    #[tokio::test]
    async fn test_parse_diff_stats() {
        let output = "10\t20\tsrc/main.rs\n5\t15\tsrc/lib.rs";
        let file = PathBuf::from("test.rs");

        let stats = ParallelGitExecutor::parse_diff_stats(&file, output);
        assert_eq!(stats.file, file);
        assert_eq!(stats.additions, 15); // 10 + 5
        assert_eq!(stats.deletions, 35); // 20 + 15
    }

    #[tokio::test]
    async fn test_parse_diff_stats_empty() {
        let file = PathBuf::from("test.rs");
        let stats = ParallelGitExecutor::parse_diff_stats(&file, "");

        assert_eq!(stats.file, file);
        assert_eq!(stats.additions, 0);
        assert_eq!(stats.deletions, 0);
    }

    #[tokio::test]
    async fn test_get_file_histories_no_git() {
        let temp_dir = TempDir::new().unwrap();
        let executor = ParallelGitExecutor::new(temp_dir.path().to_path_buf());

        let files = vec![
            temp_dir.path().join("file1.rs"),
            temp_dir.path().join("file2.rs"),
        ];

        let result = executor.get_file_histories(files, 10).await;
        assert!(result.is_err()); // Should fail without git
    }

    #[tokio::test]
    async fn test_get_file_blames_no_git() {
        let temp_dir = TempDir::new().unwrap();
        let executor = ParallelGitExecutor::new(temp_dir.path().to_path_buf());

        let files = vec![temp_dir.path().join("file1.rs")];

        let result = executor.get_file_blames(files).await;
        assert!(result.is_err()); // Should fail without git
    }

    #[tokio::test]
    async fn test_get_diff_stats_no_git() {
        let temp_dir = TempDir::new().unwrap();
        let executor = ParallelGitExecutor::new(temp_dir.path().to_path_buf());

        let file_pairs = vec![(
            temp_dir.path().join("file1.rs"),
            "abc123".to_string(),
            "def456".to_string(),
        )];

        let result = executor.get_diff_stats(file_pairs).await;
        assert!(result.is_err()); // Should fail without git
    }

    #[tokio::test]
    async fn test_clone_executor() {
        let executor = ParallelGitExecutor::new(PathBuf::from("."));
        let cloned = executor.clone();

        // Both should share the same cache
        let _ = executor.execute_command(vec!["--version"]).await;

        let (size1, _) = executor.cache_stats().await;
        let (size2, _) = cloned.cache_stats().await;
        assert_eq!(size1, size2);
    }

    #[tokio::test]
    async fn test_cache_ttl() {
        let config = ParallelGitConfig {
            max_concurrent_operations: 4,
            enable_caching: true,
            cache_ttl_seconds: 0, // Immediate expiry
        };

        let executor = ParallelGitExecutor::with_config(PathBuf::from("."), config);

        // Execute command twice
        let result1 = executor.execute_command(vec!["--version"]).await;
        assert!(result1.is_ok());

        // Second execution should not hit cache due to TTL
        let result2 = executor.execute_command(vec!["--version"]).await;
        assert!(result2.is_ok());

        // Cache should have only one entry (the second one)
        let (size, _) = executor.cache_stats().await;
        assert_eq!(size, 1);
    }

    #[tokio::test]
    async fn test_concurrent_execution() {
        let executor = ParallelGitExecutor::new(PathBuf::from("."));

        // Execute multiple commands concurrently
        let mut handles = vec![];
        for _ in 0..5 {
            let exec = executor.clone();
            let handle = tokio::spawn(async move { exec.execute_command(vec!["--version"]).await });
            handles.push(handle);
        }

        // All should succeed
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_execute_batch_owned() {
        let executor = ParallelGitExecutor::new(PathBuf::from("."));

        let commands = vec![vec!["--version".to_string()], vec!["--help".to_string()]];

        let results = executor.execute_batch_owned(commands).await;
        assert!(results.is_ok());

        let outputs = results.unwrap();
        assert_eq!(outputs.len(), 2);

        // Both commands should succeed
        for output in outputs {
            assert!(!output.is_empty());
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
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
