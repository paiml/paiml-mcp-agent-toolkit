    // ========== TdgBaseline::save() and load() Tests ==========

    #[test]
    fn test_baseline_serialization_roundtrip() {
        let mut baseline = TdgBaseline::new(None);
        baseline.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry(95.0, Grade::APLus),
        );
        baseline.add_entry(
            PathBuf::from("test2.rs"),
            create_test_entry(85.0, Grade::AMinus),
        );

        let temp_file = std::env::temp_dir().join(format!(
            "test_baseline_roundtrip_{}.json",
            std::process::id()
        ));
        baseline.save(&temp_file).expect("Failed to save baseline");

        let loaded = TdgBaseline::load(&temp_file).expect("Failed to load baseline");

        assert_eq!(loaded.files.len(), baseline.files.len());
        assert!((loaded.summary.avg_score - baseline.summary.avg_score).abs() < 0.01);
        assert_eq!(loaded.summary.total_files, baseline.summary.total_files);

        std::fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_baseline_load_invalid_path() {
        let result = TdgBaseline::load("/nonexistent/path/baseline.json");
        assert!(result.is_err());
    }

    #[test]
    fn test_baseline_save_to_nested_path() {
        let temp_dir = std::env::temp_dir().join(format!("baseline_test_{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

        let baseline = TdgBaseline::new(None);
        let temp_file = temp_dir.join("nested/deep/baseline.json");

        // This should fail because parent directories don't exist
        let result = baseline.save(&temp_file);
        assert!(result.is_err());

        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_baseline_serialization_with_git_context() {
        let git_context = Some(GitContext {
            commit_sha: "1234567890abcdef1234567890abcdef12345678".to_string(),
            commit_sha_short: "1234567".to_string(),
            branch: "feature/test".to_string(),
            author_name: "Test Developer".to_string(),
            author_email: "dev@test.com".to_string(),
            commit_timestamp: Utc::now(),
            commit_message: "Test serialization".to_string(),
            tags: vec!["v1.0.0".to_string()],
            parent_commits: vec!["abcdef1234567890abcdef1234567890abcdef12".to_string()],
            remote_url: Some("https://github.com/test/repo.git".to_string()),
            is_clean: false,
            uncommitted_files: 3,
        });

        let mut baseline = TdgBaseline::new(git_context);
        baseline.add_entry(PathBuf::from("test.rs"), create_test_entry(90.0, Grade::A));

        let temp_file =
            std::env::temp_dir().join(format!("test_baseline_git_{}.json", std::process::id()));
        baseline.save(&temp_file).expect("Failed to save baseline");

        let loaded = TdgBaseline::load(&temp_file).expect("Failed to load baseline");

        assert!(loaded.git_context.is_some());
        let ctx = loaded.git_context.unwrap();
        assert_eq!(ctx.branch, "feature/test");
        assert_eq!(ctx.tags, vec!["v1.0.0"]);
        assert!(!ctx.is_clean);
        assert_eq!(ctx.uncommitted_files, 3);

        std::fs::remove_file(temp_file).ok();
    }

    // ========== BaselineEntry Tests ==========

    #[test]
    fn test_baseline_entry_content_hash_deduplication() {
        let mut baseline = TdgBaseline::new(None);
        let content_hash = blake3::hash(b"identical content");

        let entry1 = BaselineEntry {
            content_hash,
            score: TdgScore {
                total: 90.0,
                grade: Grade::A,
                ..Default::default()
            },
            components: ComponentScores::default(),
            git_context: None,
        };

        let entry2 = BaselineEntry {
            content_hash,
            score: TdgScore {
                total: 90.0,
                grade: Grade::A,
                ..Default::default()
            },
            components: ComponentScores::default(),
            git_context: None,
        };

        baseline.add_entry(PathBuf::from("file1.rs"), entry1);
        baseline.add_entry(PathBuf::from("file2.rs"), entry2);

        assert_eq!(baseline.files.len(), 2);
        assert_eq!(
            baseline
                .files
                .get(&PathBuf::from("file1.rs"))
                .unwrap()
                .content_hash,
            baseline
                .files
                .get(&PathBuf::from("file2.rs"))
                .unwrap()
                .content_hash
        );
    }

    #[test]
    fn test_baseline_entry_with_components() {
        let mut components = ComponentScores::default();
        components
            .complexity_breakdown
            .insert("cyclomatic".to_string(), 5.0);
        components
            .duplication_sources
            .push("line 10-20".to_string());
        components
            .coupling_dependencies
            .push("module_a".to_string());
        components
            .doc_missing_items
            .push("fn process()".to_string());
        components
            .consistency_violations
            .push("naming_style".to_string());

        let entry = BaselineEntry {
            content_hash: blake3::hash(b"test"),
            score: TdgScore {
                total: 85.0,
                grade: Grade::AMinus,
                ..Default::default()
            },
            components,
            git_context: None,
        };

        let mut baseline = TdgBaseline::new(None);
        baseline.add_entry(PathBuf::from("test.rs"), entry);

        let retrieved = baseline.files.get(&PathBuf::from("test.rs")).unwrap();
        assert_eq!(retrieved.components.complexity_breakdown.len(), 1);
        assert_eq!(retrieved.components.duplication_sources.len(), 1);
        assert_eq!(retrieved.components.coupling_dependencies.len(), 1);
        assert_eq!(retrieved.components.doc_missing_items.len(), 1);
        assert_eq!(retrieved.components.consistency_violations.len(), 1);
    }

