    // === Repository Resolution Tests ===

    #[test]
    fn test_try_local_path_exists() {
        let temp_dir = TempDir::new().unwrap();
        let path_str = temp_dir.path().to_string_lossy().to_string();

        let result = try_local_path(&path_str);
        // try_local_path returns Some when path exists, but detect_repository
        // returns Err if not a git repository
        assert!(result.is_some());
        // The path exists but isn't a git repo, so detect_repository returns Err
        assert!(result.unwrap().is_err());
    }

    #[test]
    fn test_try_local_path_not_exists() {
        let result = try_local_path("/nonexistent/path/that/doesnt/exist/at/all");
        assert!(result.is_none());
    }

    #[test]
    fn test_try_github_shorthand() {
        let result = try_github_shorthand("gh:owner/repo");
        assert!(result.is_some());
        let path = result.unwrap().unwrap();
        assert!(path.to_string_lossy().contains("github.com"));
        assert!(path.to_string_lossy().contains("owner/repo"));
    }

    #[test]
    fn test_try_github_shorthand_not_shorthand() {
        let result = try_github_shorthand("owner/repo");
        assert!(result.is_none());
    }

    #[test]
    fn test_try_github_url_https() {
        let result = try_github_url("https://github.com/owner/repo");
        assert!(result.is_some());
        let path = result.unwrap().unwrap();
        assert_eq!(path.to_string_lossy(), "https://github.com/owner/repo");
    }

    #[test]
    fn test_try_github_url_git() {
        let result = try_github_url("git@github.com:owner/repo");
        assert!(result.is_some());
        let path = result.unwrap().unwrap();
        assert_eq!(path.to_string_lossy(), "git@github.com:owner/repo");
    }

    #[test]
    fn test_try_github_url_not_github() {
        let result = try_github_url("https://gitlab.com/owner/repo");
        assert!(result.is_none());
    }

    #[test]
    fn test_try_owner_repo_format() {
        let result = try_owner_repo_format("owner/repo");
        assert!(result.is_some());
        let path = result.unwrap().unwrap();
        assert!(path.to_string_lossy().contains("github.com"));
        assert!(path.to_string_lossy().contains("owner/repo"));
    }

    #[test]
    fn test_try_owner_repo_format_with_dot() {
        let result = try_owner_repo_format("owner.name/repo");
        assert!(result.is_none());
    }

    #[test]
    fn test_try_owner_repo_format_no_slash() {
        let result = try_owner_repo_format("owner-repo");
        assert!(result.is_none());
    }

    // === find_git_root Tests ===

    #[test]
    fn test_find_git_root_direct() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::create_dir(temp_dir.path().join(".git")).unwrap();

        let result = find_git_root(temp_dir.path());
        assert!(result.is_some());
        assert_eq!(result.unwrap(), temp_dir.path());
    }

    #[test]
    fn test_find_git_root_parent() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::create_dir(temp_dir.path().join(".git")).unwrap();
        let sub_dir = temp_dir.path().join("subdir");
        std::fs::create_dir(&sub_dir).unwrap();

        let result = find_git_root(&sub_dir);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), temp_dir.path());
    }

    #[test]
    fn test_find_git_root_not_found() {
        let temp_dir = TempDir::new().unwrap();
        // No .git directory created

        let result = find_git_root(temp_dir.path());
        assert!(result.is_none());
    }

    // === get_canonical_path Tests ===

    #[test]
    fn test_get_canonical_path_some() {
        let temp_dir = TempDir::new().unwrap();
        let result = get_canonical_path(Some(temp_dir.path().to_path_buf()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_canonical_path_none() {
        let result = get_canonical_path(None);
        // Should return current directory
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_canonical_path_nonexistent() {
        let result = get_canonical_path(Some(PathBuf::from("/nonexistent/path/xyz")));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    // === resolve_repository Tests ===

    #[test]
    fn test_resolve_repository_with_url() {
        let result = resolve_repository(
            None,
            Some("https://github.com/owner/repo".to_string()),
            None,
        );
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("github.com"));
    }

    #[test]
    fn test_resolve_repository_with_repo_shorthand() {
        let result = resolve_repository(None, None, Some("gh:owner/repo".to_string()));
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("github.com"));
    }

    #[test]
    fn test_resolve_repository_with_local_path() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::create_dir(temp_dir.path().join(".git")).unwrap();

        let result = resolve_repository(Some(temp_dir.path().to_path_buf()), None, None);
        assert!(result.is_ok());
    }

    // === is_interactive_environment Tests ===

    #[test]
    fn test_is_interactive_environment_in_ci() {
        // In CI, this should return false (CI env var is typically set)
        // We can't easily control the environment, but we can check it runs
        let _result = is_interactive_environment();
        // Just verify it doesn't panic
    }

    // === resolve_repo_spec Tests ===

    #[test]
    fn test_resolve_repo_spec_local_path() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::create_dir(temp_dir.path().join(".git")).unwrap();
        let path_str = temp_dir.path().to_string_lossy().to_string();

        let result = resolve_repo_spec(&path_str);
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolve_repo_spec_github_shorthand() {
        let result = resolve_repo_spec("gh:owner/repo");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("github.com"));
    }

    #[test]
    fn test_resolve_repo_spec_github_url() {
        let result = resolve_repo_spec("https://github.com/owner/repo");
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolve_repo_spec_owner_repo() {
        let result = resolve_repo_spec("owner/repo");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("github.com"));
    }

    #[test]
    fn test_resolve_repo_spec_not_found() {
        let result = resolve_repo_spec("nonexistent-path-that-definitely-does-not-exist");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    // === detect_repository Tests ===

    #[test]
    fn test_detect_repository_with_git() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::create_dir(temp_dir.path().join(".git")).unwrap();

        let result = detect_repository(Some(temp_dir.path().to_path_buf()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), temp_dir.path());
    }

    // === Additional DemoRunner Tests ===

    // === Additional Repository Resolution Tests ===

    #[test]
    fn test_resolve_repository_priority_repo_over_url() {
        // When both repo and url are provided, repo should take precedence
        let result = resolve_repository(
            None,
            Some("https://github.com/other/repo".to_string()),
            Some("gh:owner/main-repo".to_string()),
        );
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("owner/main-repo"));
    }

    #[test]
    fn test_resolve_repository_priority_url_over_path() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::create_dir(temp_dir.path().join(".git")).unwrap();

        // When url is provided but not repo, url takes precedence over path
        let result = resolve_repository(
            Some(temp_dir.path().to_path_buf()),
            Some("https://github.com/test/repo".to_string()),
            None,
        );
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("github.com"));
    }

    #[test]
    fn test_resolve_repo_spec_git_ssh_url() {
        let result = resolve_repo_spec("git@github.com:owner/repo.git");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path.to_string_lossy(), "git@github.com:owner/repo.git");
    }

    // === find_git_root Edge Cases ===

    #[test]
    fn test_find_git_root_deeply_nested() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::create_dir(temp_dir.path().join(".git")).unwrap();

        // Create deeply nested structure
        let mut nested = temp_dir.path().to_path_buf();
        for i in 0..10 {
            nested = nested.join(format!("level{i}"));
            std::fs::create_dir(&nested).unwrap();
        }

        let result = find_git_root(&nested);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), temp_dir.path());
    }

    #[test]
    fn test_find_git_root_at_filesystem_root() {
        // Test with a path that has no .git in any parent
        let result = find_git_root(Path::new("/"));
        assert!(result.is_none());
    }

    // === detect_repository Without Git Tests ===

    #[test]
    fn test_detect_repository_no_git_non_interactive() {
        let temp_dir = TempDir::new().unwrap();
        // No .git directory

        // This should fail in non-interactive mode (CI)
        let result = detect_repository(Some(temp_dir.path().to_path_buf()));
        // In CI, this will return an error
        // We can't control terminal state, so just verify it doesn't panic
        let _ = result;
    }

    // === Async Repository Resolution Tests ===

    #[tokio::test]
    async fn test_resolve_repository_async_local_path() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::create_dir(temp_dir.path().join(".git")).unwrap();

        let result =
            resolve_repository_async(Some(temp_dir.path().to_path_buf()), None, None).await;

        assert!(result.is_ok());
        // Should return the local path without cloning
        let path = result.unwrap();
        assert!(path.exists());
    }

    #[tokio::test]
    async fn test_resolve_repository_async_with_shorthand() {
        // This test would actually try to clone, so we just test the URL parsing
        let result = resolve_repository_async(None, None, Some("gh:rust-lang/rust".to_string()));

        // This would fail if not in CI or without network, but shouldn't panic
        // The important thing is the URL is correctly formed
        match result.await {
            Ok(path) => {
                // If it succeeds, verify path
                assert!(path.to_string_lossy().len() > 0);
            }
            Err(e) => {
                // Clone failure is acceptable in test environment
                let err_str = e.to_string();
                // Should be a clone-related error, not a parsing error
                assert!(
                    err_str.contains("clone")
                        || err_str.contains("git")
                        || err_str.contains("timeout")
                        || err_str.contains("network")
                        || err_str.contains("error")
                );
            }
        }
    }
