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
    fn test_baseline_serialization_deterministic_across_insert_order() {
        // Regression: `files`, `grade_distribution`, and `languages` were
        // HashMaps, so two identical baselines could serialize with
        // different key orders, breaking byte-level comparison across runs.
        let mut a = TdgBaseline::new(None);
        let mut b = TdgBaseline::new(None);
        let entries = [
            ("zeta.rs", 95.0, Grade::APLus),
            ("alpha.rs", 85.0, Grade::AMinus),
            ("mid.rs", 75.0, Grade::B),
        ];
        for (p, s, g) in &entries {
            a.add_entry(PathBuf::from(p), create_test_entry(*s, *g));
        }
        for (p, s, g) in entries.iter().rev() {
            b.add_entry(PathBuf::from(p), create_test_entry(*s, *g));
        }
        b.created_at = a.created_at;

        let ja = serde_json::to_string_pretty(&a).expect("serialize a");
        let jb = serde_json::to_string_pretty(&b).expect("serialize b");
        assert_eq!(
            ja, jb,
            "serialized baseline must not depend on insertion order"
        );

        // Equality alone would pass ~3% of the time under HashMap iteration;
        // sorted key order fails a HashMap revert deterministically
        let ia = ja.find("alpha.rs").expect("alpha.rs present");
        let im = ja.find("mid.rs").expect("mid.rs present");
        let iz = ja.find("zeta.rs").expect("zeta.rs present");
        assert!(
            ia < im && im < iz,
            "files keys must serialize in sorted order"
        );
    }

    #[test]
    fn test_baseline_name_roundtrip_and_legacy_load() {
        let mut baseline = TdgBaseline::new(None);
        baseline.name = Some("pre-refactor".to_string());
        let temp_file = std::env::temp_dir().join(format!(
            "test_baseline_name_{}.json",
            std::process::id()
        ));
        baseline.save(&temp_file).expect("save named baseline");
        let loaded = TdgBaseline::load(&temp_file).expect("load named baseline");
        assert_eq!(loaded.name.as_deref(), Some("pre-refactor"));
        std::fs::remove_file(&temp_file).ok();

        // Baselines written before the `name` field existed must still load:
        // a populated baseline serialized without a name key is exactly the
        // shape 3.18.0 produced (modulo map key order, which serde accepts
        // in any order)
        let mut populated = TdgBaseline::new(None);
        populated.add_entry(
            PathBuf::from("a.rs"),
            create_test_entry(95.0, Grade::APLus),
        );
        populated.add_entry(PathBuf::from("b.rs"), create_test_entry(70.0, Grade::B));
        let json = serde_json::to_string(&populated).expect("serialize");
        assert!(
            !json.contains("\"name\""),
            "None name must not be serialized"
        );
        let legacy: TdgBaseline = serde_json::from_str(&json).expect("legacy load");
        assert!(legacy.name.is_none());
        assert_eq!(legacy.files.len(), 2);
        assert_eq!(legacy.summary.total_files, 2);
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

