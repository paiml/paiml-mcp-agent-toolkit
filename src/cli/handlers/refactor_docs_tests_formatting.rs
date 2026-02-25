    // ========== format_output Tests ==========

    #[test]
    fn test_format_output_summary() {
        let result = RefactorDocsResult {
            cruft_files: vec![],
            summary: CleanupSummary {
                total_files_scanned: 100,
                cruft_files_found: 0,
                total_size_bytes: 0,
                files_by_category: HashMap::new(),
                size_by_category: HashMap::new(),
                oldest_file_days: 0,
                newest_file_days: 0,
            },
            preserved_files: vec![],
            errors: vec![],
        };

        let output = format_output(
            &result,
            RefactorDocsOutputFormat::Summary,
            false,
            false,
            Duration::from_secs(1),
        )
        .unwrap();

        assert!(output.contains("# Documentation Refactoring Report"));
        assert!(output.contains("**Files Scanned**: 100"));
        assert!(output.contains("**Cruft Files Found**: 0"));
    }

    #[test]
    fn test_format_output_dry_run() {
        let result = RefactorDocsResult {
            cruft_files: vec![],
            summary: CleanupSummary::default(),
            preserved_files: vec![],
            errors: vec![],
        };

        let output = format_output(
            &result,
            RefactorDocsOutputFormat::Summary,
            true,
            false,
            Duration::from_secs(1),
        )
        .unwrap();

        assert!(output.contains("**Mode**: Dry Run"));
    }

    #[test]
    fn test_format_output_with_perf() {
        let result = RefactorDocsResult {
            cruft_files: vec![],
            summary: CleanupSummary::default(),
            preserved_files: vec![],
            errors: vec![],
        };

        let output = format_output(
            &result,
            RefactorDocsOutputFormat::Summary,
            false,
            true,
            Duration::from_millis(1500),
        )
        .unwrap();

        assert!(output.contains("Analysis completed in"));
    }

    #[test]
    fn test_format_output_json() {
        let result = RefactorDocsResult {
            cruft_files: vec![CruftFile {
                path: PathBuf::from("/tmp/test.txt"),
                category: FileCategory::TemporaryScript,
                size_bytes: 1024,
                modified: SystemTime::now(),
                age_days: 5,
                reason: "test reason".to_string(),
                pattern_matched: "*.txt".to_string(),
            }],
            summary: CleanupSummary::default(),
            preserved_files: vec![],
            errors: vec![],
        };

        let output = format_output(
            &result,
            RefactorDocsOutputFormat::Json,
            false,
            false,
            Duration::from_secs(1),
        )
        .unwrap();

        assert!(output.contains("\"cruft_files\""));
        assert!(output.contains("\"path\""));
        assert!(output.contains("TemporaryScript"));
    }

    #[test]
    fn test_format_output_detailed() {
        let result = RefactorDocsResult {
            cruft_files: vec![CruftFile {
                path: PathBuf::from("/tmp/test.txt"),
                category: FileCategory::BuildArtifact,
                size_bytes: 2048,
                modified: SystemTime::now(),
                age_days: 10,
                reason: "Matches pattern: *.txt".to_string(),
                pattern_matched: "*.txt".to_string(),
            }],
            summary: CleanupSummary {
                total_files_scanned: 50,
                cruft_files_found: 1,
                total_size_bytes: 2048,
                files_by_category: HashMap::new(),
                size_by_category: HashMap::new(),
                oldest_file_days: 10,
                newest_file_days: 10,
            },
            preserved_files: vec![PathBuf::from("/tmp/keep.txt")],
            errors: vec![],
        };

        let output = format_output(
            &result,
            RefactorDocsOutputFormat::Detailed,
            false,
            false,
            Duration::from_secs(1),
        )
        .unwrap();

        assert!(output.contains("## Cruft Files Details"));
        assert!(output.contains("/tmp/test.txt"));
        assert!(output.contains("**Category**: Build Artifact"));
        assert!(output.contains("**Age**: 10 days"));
        assert!(output.contains("## Preserved Files"));
        assert!(output.contains("/tmp/keep.txt"));
    }

    #[test]
    fn test_format_output_interactive_uses_summary() {
        let result = RefactorDocsResult {
            cruft_files: vec![],
            summary: CleanupSummary::default(),
            preserved_files: vec![],
            errors: vec![],
        };

        let output = format_output(
            &result,
            RefactorDocsOutputFormat::Interactive,
            false,
            false,
            Duration::from_secs(1),
        )
        .unwrap();

        // Interactive format uses summary format
        assert!(output.contains("# Documentation Refactoring Report"));
    }

    // ========== format_summary Tests ==========

    #[test]
    fn test_format_summary_with_errors() {
        let result = RefactorDocsResult {
            cruft_files: vec![],
            summary: CleanupSummary::default(),
            preserved_files: vec![],
            errors: vec![
                "Error reading file1".to_string(),
                "Permission denied for file2".to_string(),
            ],
        };

        let output = format_summary(&result, false, false, Duration::from_secs(1)).unwrap();

        assert!(output.contains("## ⚠️ Errors"));
        assert!(output.contains("Error reading file1"));
        assert!(output.contains("Permission denied for file2"));
    }

    #[test]
    fn test_format_summary_with_categories() {
        let mut files_by_category = HashMap::new();
        files_by_category.insert("Temporary Script".to_string(), 3);
        files_by_category.insert("Build Artifact".to_string(), 2);

        let mut size_by_category = HashMap::new();
        size_by_category.insert("Temporary Script".to_string(), 3 * 1024 * 1024);
        size_by_category.insert("Build Artifact".to_string(), 2 * 1024 * 1024);

        let result = RefactorDocsResult {
            cruft_files: vec![],
            summary: CleanupSummary {
                total_files_scanned: 100,
                cruft_files_found: 5,
                total_size_bytes: 5 * 1024 * 1024,
                files_by_category,
                size_by_category,
                oldest_file_days: 30,
                newest_file_days: 1,
            },
            preserved_files: vec![],
            errors: vec![],
        };

        let output = format_summary(&result, false, false, Duration::from_secs(1)).unwrap();

        assert!(output.contains("## Files by Category"));
        assert!(output.contains("Temporary Script"));
        assert!(output.contains("Build Artifact"));
    }

    // ========== format_detailed Tests ==========

    #[test]
    fn test_format_detailed_empty_cruft() {
        let result = RefactorDocsResult {
            cruft_files: vec![],
            summary: CleanupSummary::default(),
            preserved_files: vec![],
            errors: vec![],
        };

        let output = format_detailed(&result, false, false, Duration::from_secs(1)).unwrap();

        // Should not contain details section if no cruft files
        assert!(!output.contains("## Cruft Files Details"));
    }

    #[test]
    fn test_format_detailed_many_preserved_files() {
        let result = RefactorDocsResult {
            cruft_files: vec![],
            summary: CleanupSummary::default(),
            preserved_files: (0..25)
                .map(|i| PathBuf::from(format!("/tmp/keep{i}.txt")))
                .collect(),
            errors: vec![],
        };

        let output = format_detailed(&result, false, false, Duration::from_secs(1)).unwrap();

        // Should not show preserved files section if > 20 files
        assert!(!output.contains("## Preserved Files"));
    }

    // ========== format_json Tests ==========

    #[test]
    fn test_format_json_serialization() {
        let result = RefactorDocsResult {
            cruft_files: vec![],
            summary: CleanupSummary {
                total_files_scanned: 42,
                cruft_files_found: 0,
                total_size_bytes: 0,
                files_by_category: HashMap::new(),
                size_by_category: HashMap::new(),
                oldest_file_days: 0,
                newest_file_days: 0,
            },
            preserved_files: vec![PathBuf::from("/tmp/keep.txt")],
            errors: vec!["test error".to_string()],
        };

        let json_output = format_json(&result).unwrap();

        // Verify it's valid JSON by parsing
        let parsed: serde_json::Value = serde_json::from_str(&json_output).unwrap();
        assert_eq!(parsed["summary"]["total_files_scanned"], 42);
        assert_eq!(parsed["preserved_files"][0], "/tmp/keep.txt");
        assert_eq!(parsed["errors"][0], "test error");
    }

    // ========== create_cruft_file Tests ==========

    #[test]
    fn test_create_cruft_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "test content").unwrap();
        let metadata = std::fs::metadata(&file_path).unwrap();
        let now = SystemTime::now();

        let cruft = create_cruft_file(
            &file_path,
            &metadata,
            FileCategory::TemporaryScript,
            "*.txt",
            &now,
        );

        assert_eq!(cruft.path, file_path);
        assert_eq!(cruft.category, FileCategory::TemporaryScript);
        assert_eq!(cruft.size_bytes, metadata.len());
        assert_eq!(cruft.pattern_matched, "*.txt");
        assert!(cruft.reason.contains("Matches pattern"));
    }

    // ========== get_file_metadata Tests ==========

    #[test]
    fn test_get_file_metadata_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "test content").unwrap();

        let result = get_file_metadata(&file_path);
        assert!(result.is_ok());
        assert!(result.unwrap().len() > 0);
    }

    #[test]
    fn test_get_file_metadata_nonexistent() {
        let result = get_file_metadata(Path::new("/nonexistent/file.txt"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to read metadata"));
    }

