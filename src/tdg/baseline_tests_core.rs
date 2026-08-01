    // ========== TdgBaseline::new() Tests ==========

    #[test]
    fn test_create_baseline_empty() {
        let baseline = TdgBaseline::new(None);
        assert_eq!(baseline.files.len(), 0);
        assert_eq!(baseline.summary.total_files, 0);
        assert_eq!(baseline.summary.avg_score, 0.0);
    }

    #[test]
    fn test_create_baseline_with_git_context() {
        let git_context = Some(GitContext {
            commit_sha: "abc123def456789012345678901234567890abcd".to_string(),
            commit_sha_short: "abc123d".to_string(),
            branch: "master".to_string(),
            author_name: "Test Author".to_string(),
            author_email: "test@example.com".to_string(),
            commit_timestamp: Utc::now(),
            commit_message: "Test commit".to_string(),
            tags: vec![],
            parent_commits: vec![],
            remote_url: None,
            is_clean: true,
            uncommitted_files: 0,
        });

        let baseline = TdgBaseline::new(git_context.clone());
        assert!(baseline.git_context.is_some());
        let ctx = baseline.git_context.unwrap();
        assert_eq!(ctx.commit_sha, "abc123def456789012345678901234567890abcd");
        assert_eq!(ctx.branch, "master");
        assert_eq!(ctx.author_name, "Test Author");
    }

    #[test]
    fn test_baseline_version_is_set() {
        let baseline = TdgBaseline::new(None);
        assert!(!baseline.version.is_empty());
        // Version should be a semantic version from Cargo.toml
        assert!(baseline.version.contains('.'));
    }

    #[test]
    fn test_baseline_created_at_is_recent() {
        let before = Utc::now();
        let baseline = TdgBaseline::new(None);
        let after = Utc::now();

        assert!(baseline.created_at >= before);
        assert!(baseline.created_at <= after);
    }

    // ========== TdgBaseline::add_entry() Tests ==========

    #[test]
    fn test_add_entry_updates_summary() {
        let mut baseline = TdgBaseline::new(None);
        let entry = create_test_entry(95.0, Grade::APlus);

        baseline.add_entry(PathBuf::from("test.rs"), entry);

        assert_eq!(baseline.summary.total_files, 1);
        assert!((baseline.summary.avg_score - 95.0).abs() < 0.01);
    }

    #[test]
    fn test_add_multiple_entries_updates_average() {
        let mut baseline = TdgBaseline::new(None);

        baseline.add_entry(PathBuf::from("a.rs"), create_test_entry(90.0, Grade::A));
        baseline.add_entry(PathBuf::from("b.rs"), create_test_entry(80.0, Grade::BPlus));
        baseline.add_entry(
            PathBuf::from("c.rs"),
            create_test_entry(70.0, Grade::BMinus),
        );

        assert_eq!(baseline.summary.total_files, 3);
        // Average: (90 + 80 + 70) / 3 = 80.0
        assert!((baseline.summary.avg_score - 80.0).abs() < 0.01);
    }

    #[test]
    fn test_add_entry_overwrites_existing_path() {
        let mut baseline = TdgBaseline::new(None);

        baseline.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry(80.0, Grade::BPlus),
        );
        baseline.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry(95.0, Grade::APlus),
        );

        assert_eq!(baseline.summary.total_files, 1);
        assert!((baseline.summary.avg_score - 95.0).abs() < 0.01);
    }

    #[test]
    fn test_add_entry_updates_grade_distribution() {
        let mut baseline = TdgBaseline::new(None);

        baseline.add_entry(PathBuf::from("a.rs"), create_test_entry(95.0, Grade::APlus));
        baseline.add_entry(PathBuf::from("b.rs"), create_test_entry(90.0, Grade::A));
        baseline.add_entry(
            PathBuf::from("c.rs"),
            create_test_entry(88.0, Grade::AMinus),
        );
        baseline.add_entry(PathBuf::from("d.rs"), create_test_entry(75.0, Grade::B));

        assert_eq!(
            *baseline
                .summary
                .grade_distribution
                .get(&Grade::APlus)
                .unwrap(),
            1
        );
        assert_eq!(
            *baseline.summary.grade_distribution.get(&Grade::A).unwrap(),
            1
        );
        assert_eq!(
            *baseline
                .summary
                .grade_distribution
                .get(&Grade::AMinus)
                .unwrap(),
            1
        );
        assert_eq!(
            *baseline.summary.grade_distribution.get(&Grade::B).unwrap(),
            1
        );
    }

    #[test]
    fn test_add_entry_updates_language_distribution() {
        let mut baseline = TdgBaseline::new(None);

        baseline.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry_with_lang(90.0, Grade::A, Language::Rust),
        );
        baseline.add_entry(
            PathBuf::from("test2.rs"),
            create_test_entry_with_lang(85.0, Grade::AMinus, Language::Rust),
        );
        baseline.add_entry(
            PathBuf::from("test.py"),
            create_test_entry_with_lang(80.0, Grade::BPlus, Language::Python),
        );

        assert_eq!(
            *baseline
                .summary
                .languages
                .get(&format!("{:?}", Language::Rust))
                .unwrap(),
            2
        );
        assert_eq!(
            *baseline
                .summary
                .languages
                .get(&format!("{:?}", Language::Python))
                .unwrap(),
            1
        );
    }

