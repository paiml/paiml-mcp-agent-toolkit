    async fn test_setup_refactoring_context_project_wide() {
        let temp_dir = TempDir::new().unwrap();

        let result = setup_refactoring_context(
            temp_dir.path().to_path_buf(),
            false,
            None,
            RefactorAutoOutputFormat::Summary,
            5,
            false,
            vec![],
            vec![],
            None,
            None,
            None,
        )
        .await;

        assert!(result.is_ok());
        let context = result.unwrap();
        assert!(matches!(context.config.mode, RefactorMode::ProjectWide));
    }

    #[tokio::test]
    async fn test_setup_refactoring_context_single_file_mode() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn main() {}").unwrap();

        let result = setup_refactoring_context(
            temp_dir.path().to_path_buf(),
            true,
            Some(test_file.clone()),
            RefactorAutoOutputFormat::Json,
            3,
            true,
            vec!["target".to_string()],
            vec!["*.rs".to_string()],
            None,
            None,
            None,
        )
        .await;

        assert!(result.is_ok());
        let context = result.unwrap();
        if let RefactorMode::SingleFile(path) = &context.config.mode {
            assert_eq!(path, &test_file);
        } else {
            panic!("Expected SingleFile mode");
        }
    }

    #[tokio::test]
    async fn test_setup_refactoring_context_single_file_mode_no_file() {
        let temp_dir = TempDir::new().unwrap();

        let result = setup_refactoring_context(
            temp_dir.path().to_path_buf(),
            true,
            None, // No file provided
            RefactorAutoOutputFormat::Summary,
            1,
            false,
            vec![],
            vec![],
            None,
            None,
            None,
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Single file mode requires --file parameter"));
    }

    #[tokio::test]
    async fn test_setup_refactoring_context_bug_report_mode() {
        let temp_dir = TempDir::new().unwrap();
        let bug_report = temp_dir.path().join("bug.md");
        std::fs::write(&bug_report, "# Bug Report").unwrap();

        let result = setup_refactoring_context(
            temp_dir.path().to_path_buf(),
            false,
            None,
            RefactorAutoOutputFormat::Detailed,
            2,
            false,
            vec![],
            vec![],
            None,
            None,
            Some(bug_report.clone()),
        )
        .await;

        assert!(result.is_ok());
        let context = result.unwrap();
        if let RefactorMode::BugReport(path) = &context.config.mode {
            assert_eq!(path, &bug_report);
        } else {
            panic!("Expected BugReport mode");
        }
    }

    #[tokio::test]
    async fn test_setup_refactoring_context_github_issue_mode() {
        let temp_dir = TempDir::new().unwrap();
        let github_url = "https://github.com/owner/repo/issues/123".to_string();

        let result = setup_refactoring_context(
            temp_dir.path().to_path_buf(),
            false,
            None,
            RefactorAutoOutputFormat::Json,
            1,
            true,
            vec![],
            vec![],
            None,
            Some(github_url.clone()),
            None,
        )
        .await;

        assert!(result.is_ok());
        let context = result.unwrap();
        if let RefactorMode::GitHubIssue(url) = &context.config.mode {
            assert_eq!(url, &github_url);
        } else {
            panic!("Expected GitHubIssue mode");
        }
    }

    #[tokio::test]
    async fn test_setup_refactoring_context_with_ignore_file() {
        let temp_dir = TempDir::new().unwrap();
        let ignore_file = temp_dir.path().join(".pmatignore");
        std::fs::write(&ignore_file, "target/\n*.tmp").unwrap();

        let result = setup_refactoring_context(
            temp_dir.path().to_path_buf(),
            false,
            None,
            RefactorAutoOutputFormat::Summary,
            5,
            false,
            vec![],
            vec![],
            Some(ignore_file.clone()),
            None,
            None,
        )
        .await;

        assert!(result.is_ok());
        let context = result.unwrap();
        assert_eq!(
            context.config.patterns.ignore_file_path,
            Some(ignore_file)
        );
