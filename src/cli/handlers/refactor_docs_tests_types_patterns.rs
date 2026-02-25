
    // ========== FileCategory Tests ==========

    #[test]
    fn test_file_category_display() {
        assert_eq!(
            FileCategory::TemporaryScript.to_string(),
            "Temporary Script"
        );
        assert_eq!(FileCategory::StatusReport.to_string(), "Status Report");
        assert_eq!(FileCategory::BuildArtifact.to_string(), "Build Artifact");
    }

    #[test]
    fn test_file_category_display_all_variants() {
        assert_eq!(FileCategory::TestFixture.to_string(), "Test Fixture");
        assert_eq!(FileCategory::CustomPattern.to_string(), "Custom Pattern");
        assert_eq!(FileCategory::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn test_file_category_clone_and_copy() {
        let category = FileCategory::TemporaryScript;
        let cloned = category.clone();
        let copied = category;
        assert_eq!(category, cloned);
        assert_eq!(category, copied);
    }

    #[test]
    fn test_file_category_equality() {
        assert_eq!(FileCategory::BuildArtifact, FileCategory::BuildArtifact);
        assert_ne!(FileCategory::BuildArtifact, FileCategory::StatusReport);
    }

    // ========== CleanupSummary Tests ==========

    #[test]
    fn test_cleanup_summary_default() {
        let summary = CleanupSummary::default();
        assert_eq!(summary.total_files_scanned, 0);
        assert_eq!(summary.cruft_files_found, 0);
        assert_eq!(summary.total_size_bytes, 0);
        assert!(summary.files_by_category.is_empty());
        assert!(summary.size_by_category.is_empty());
        assert_eq!(summary.oldest_file_days, 0);
        assert_eq!(summary.newest_file_days, 0);
    }

    // ========== CruftFile Tests ==========

    #[test]
    fn test_cruft_file_creation() {
        let cruft = CruftFile {
            path: PathBuf::from("/tmp/test.txt"),
            category: FileCategory::TemporaryScript,
            size_bytes: 1024,
            modified: SystemTime::now(),
            age_days: 5,
            reason: "Test reason".to_string(),
            pattern_matched: "*.txt".to_string(),
        };
        assert_eq!(cruft.path, PathBuf::from("/tmp/test.txt"));
        assert_eq!(cruft.category, FileCategory::TemporaryScript);
        assert_eq!(cruft.size_bytes, 1024);
        assert_eq!(cruft.age_days, 5);
    }

    #[test]
    fn test_cruft_file_clone() {
        let cruft = CruftFile {
            path: PathBuf::from("/tmp/test.txt"),
            category: FileCategory::BuildArtifact,
            size_bytes: 2048,
            modified: SystemTime::now(),
            age_days: 10,
            reason: "Matches artifact pattern".to_string(),
            pattern_matched: "*.o".to_string(),
        };
        let cloned = cruft.clone();
        assert_eq!(cloned.path, cruft.path);
        assert_eq!(cloned.category, cruft.category);
        assert_eq!(cloned.size_bytes, cruft.size_bytes);
    }

    // ========== RefactorDocsResult Tests ==========

    #[test]
    fn test_refactor_docs_result_creation() {
        let result = RefactorDocsResult {
            cruft_files: vec![],
            summary: CleanupSummary::default(),
            preserved_files: vec![],
            errors: vec![],
        };
        assert!(result.cruft_files.is_empty());
        assert!(result.preserved_files.is_empty());
        assert!(result.errors.is_empty());
    }

    // ========== should_preserve Tests ==========

    #[test]
    fn test_should_preserve() {
        let patterns = vec!["README.md".to_string(), "LICENSE*".to_string()];

        assert!(should_preserve(Path::new("README.md"), &patterns));
        assert!(should_preserve(Path::new("LICENSE"), &patterns));
        assert!(should_preserve(Path::new("LICENSE.txt"), &patterns));
        assert!(!should_preserve(Path::new("test.md"), &patterns));
    }

    #[test]
    fn test_should_preserve_empty_patterns() {
        let patterns: Vec<String> = vec![];
        assert!(!should_preserve(Path::new("README.md"), &patterns));
        assert!(!should_preserve(Path::new("anything.txt"), &patterns));
    }

    #[test]
    fn test_should_preserve_complex_patterns() {
        let patterns = vec![
            "*.keep".to_string(),
            "important-*".to_string(),
            "config.*.json".to_string(),
        ];
        assert!(should_preserve(Path::new("file.keep"), &patterns));
        assert!(should_preserve(Path::new("important-data.txt"), &patterns));
        assert!(should_preserve(Path::new("config.prod.json"), &patterns));
        assert!(!should_preserve(Path::new("config.json"), &patterns));
    }

    #[test]
    fn test_should_preserve_path_with_directories() {
        let patterns = vec!["README.md".to_string()];
        // Only matches file name, not full path
        assert!(should_preserve(
            Path::new("/some/path/README.md"),
            &patterns
        ));
    }

    #[test]
    fn test_should_preserve_invalid_pattern() {
        // Invalid glob pattern - should not crash
        let patterns = vec!["[invalid".to_string()];
        assert!(!should_preserve(Path::new("test.txt"), &patterns));
    }

    // ========== matches_pattern Tests ==========

    #[test]
    fn test_matches_pattern() {
        let patterns = vec![
            ("fix-*.sh".to_string(), FileCategory::TemporaryScript),
            ("*_STATUS.md".to_string(), FileCategory::StatusReport),
        ];

        assert_eq!(
            matches_pattern(Path::new("fix-test.sh"), &patterns),
            Some(("fix-*.sh".to_string(), FileCategory::TemporaryScript))
        );

        assert_eq!(
            matches_pattern(Path::new("BUILD_STATUS.md"), &patterns),
            Some(("*_STATUS.md".to_string(), FileCategory::StatusReport))
        );

        assert_eq!(matches_pattern(Path::new("normal.txt"), &patterns), None);
    }

    #[test]
    fn test_matches_pattern_empty_patterns() {
        let patterns: Vec<(String, FileCategory)> = vec![];
        assert_eq!(matches_pattern(Path::new("anything.txt"), &patterns), None);
    }

    #[test]
    fn test_matches_pattern_first_match_wins() {
        let patterns = vec![
            ("*.txt".to_string(), FileCategory::TemporaryScript),
            ("test*.txt".to_string(), FileCategory::StatusReport),
        ];
        // First matching pattern wins
        let result = matches_pattern(Path::new("test.txt"), &patterns);
        assert_eq!(
            result,
            Some(("*.txt".to_string(), FileCategory::TemporaryScript))
        );
    }

    #[test]
    fn test_matches_pattern_with_invalid_glob() {
        let patterns = vec![
            ("[invalid".to_string(), FileCategory::TemporaryScript),
            ("*.txt".to_string(), FileCategory::BuildArtifact),
        ];
        // Should skip invalid pattern and match valid one
        let result = matches_pattern(Path::new("test.txt"), &patterns);
        assert_eq!(
            result,
            Some(("*.txt".to_string(), FileCategory::BuildArtifact))
        );
    }

    // ========== collect_scan_directories Tests ==========

    #[test]
    fn test_collect_scan_directories_include_root_only() {
        let project_path = Path::new("/project");
        let dirs = collect_scan_directories(project_path, true, false, vec![]);
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0], PathBuf::from("/project"));
    }

    #[test]
    fn test_collect_scan_directories_include_docs_nonexistent() {
        let project_path = Path::new("/nonexistent/project");
        let dirs = collect_scan_directories(project_path, false, true, vec![]);
        // docs dir doesn't exist, so not included
        assert!(dirs.is_empty());
    }

    #[test]
    fn test_collect_scan_directories_with_additional() {
        let project_path = Path::new("/project");
        let additional = vec![PathBuf::from("/extra1"), PathBuf::from("/extra2")];
        let dirs = collect_scan_directories(project_path, false, false, additional);
        assert_eq!(dirs.len(), 2);
        assert!(dirs.contains(&PathBuf::from("/extra1")));
        assert!(dirs.contains(&PathBuf::from("/extra2")));
    }

    #[test]
    fn test_collect_scan_directories_all_options() {
        let project_path = Path::new("/project");
        let additional = vec![PathBuf::from("/extra")];
        let dirs = collect_scan_directories(project_path, true, false, additional);
        assert_eq!(dirs.len(), 2);
        assert!(dirs.contains(&PathBuf::from("/project")));
        assert!(dirs.contains(&PathBuf::from("/extra")));
    }

    // ========== combine_patterns Tests ==========

    #[test]
    fn test_combine_patterns_empty() {
        let result = combine_patterns(vec![], vec![], vec![], vec![]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_combine_patterns_temp_only() {
        let result = combine_patterns(vec!["fix-*.sh".to_string()], vec![], vec![], vec![]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "fix-*.sh");
        assert_eq!(result[0].1, FileCategory::TemporaryScript);
    }

    #[test]
    fn test_combine_patterns_status_only() {
        let result = combine_patterns(vec![], vec!["*_STATUS.md".to_string()], vec![], vec![]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "*_STATUS.md");
        assert_eq!(result[0].1, FileCategory::StatusReport);
    }

    #[test]
    fn test_combine_patterns_artifact_only() {
        let result = combine_patterns(vec![], vec![], vec!["*.o".to_string()], vec![]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "*.o");
        assert_eq!(result[0].1, FileCategory::BuildArtifact);
    }

    #[test]
    fn test_combine_patterns_custom_only() {
        let result = combine_patterns(vec![], vec![], vec![], vec!["custom-*".to_string()]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "custom-*");
        assert_eq!(result[0].1, FileCategory::CustomPattern);
    }

    #[test]
    fn test_combine_patterns_all_types() {
        let result = combine_patterns(
            vec!["temp-*.sh".to_string()],
            vec!["*_STATUS.md".to_string()],
            vec!["*.mmd".to_string()],
            vec!["custom.txt".to_string()],
        );
        assert_eq!(result.len(), 4);

        // Verify order: temp, status, artifact, custom
        assert_eq!(result[0].1, FileCategory::TemporaryScript);
        assert_eq!(result[1].1, FileCategory::StatusReport);
        assert_eq!(result[2].1, FileCategory::BuildArtifact);
        assert_eq!(result[3].1, FileCategory::CustomPattern);
    }

    // ========== should_use_interactive_mode Tests ==========

    #[test]
    fn test_should_use_interactive_mode_true() {
        assert!(should_use_interactive_mode(
            RefactorDocsOutputFormat::Interactive,
            false,
            false
        ));
    }

    #[test]
    fn test_should_use_interactive_mode_dry_run() {
        assert!(!should_use_interactive_mode(
            RefactorDocsOutputFormat::Interactive,
            true,
            false
        ));
    }

    #[test]
    fn test_should_use_interactive_mode_auto_remove() {
        assert!(!should_use_interactive_mode(
            RefactorDocsOutputFormat::Interactive,
            false,
            true
        ));
    }

    #[test]
    fn test_should_use_interactive_mode_non_interactive() {
        assert!(!should_use_interactive_mode(
            RefactorDocsOutputFormat::Summary,
            false,
            false
        ));
        assert!(!should_use_interactive_mode(
            RefactorDocsOutputFormat::Json,
            false,
            false
        ));
        assert!(!should_use_interactive_mode(
            RefactorDocsOutputFormat::Detailed,
            false,
            false
        ));
    }

    // ========== should_create_backup Tests ==========

    #[test]
    fn test_should_create_backup_true() {
        let files = vec![CruftFile {
            path: PathBuf::from("/tmp/test.txt"),
            category: FileCategory::TemporaryScript,
            size_bytes: 100,
            modified: SystemTime::now(),
            age_days: 1,
            reason: "test".to_string(),
            pattern_matched: "*.txt".to_string(),
        }];
        assert!(should_create_backup(true, false, &files, false));
    }

    #[test]
    fn test_should_create_backup_dry_run() {
        let files = vec![CruftFile {
            path: PathBuf::from("/tmp/test.txt"),
            category: FileCategory::TemporaryScript,
            size_bytes: 100,
            modified: SystemTime::now(),
            age_days: 1,
            reason: "test".to_string(),
            pattern_matched: "*.txt".to_string(),
        }];
        assert!(!should_create_backup(true, true, &files, false));
    }

    #[test]
    fn test_should_create_backup_no_backup_flag() {
        let files = vec![CruftFile {
            path: PathBuf::from("/tmp/test.txt"),
            category: FileCategory::TemporaryScript,
            size_bytes: 100,
            modified: SystemTime::now(),
            age_days: 1,
            reason: "test".to_string(),
            pattern_matched: "*.txt".to_string(),
        }];
        assert!(!should_create_backup(false, false, &files, false));
    }

    #[test]
    fn test_should_create_backup_empty_files_no_auto_remove() {
        let files: Vec<CruftFile> = vec![];
        assert!(!should_create_backup(true, false, &files, false));
    }

    #[test]
    fn test_should_create_backup_empty_files_with_auto_remove() {
        let files: Vec<CruftFile> = vec![];
        assert!(should_create_backup(true, false, &files, true));
    }

    // ========== should_remove_files Tests ==========

    #[test]
    fn test_should_remove_files_auto_remove() {
        assert!(should_remove_files(
            false,
            true,
            RefactorDocsOutputFormat::Summary
        ));
    }

    #[test]
    fn test_should_remove_files_interactive() {
        assert!(should_remove_files(
            false,
            false,
            RefactorDocsOutputFormat::Interactive
        ));
    }

    #[test]
    fn test_should_remove_files_dry_run() {
        assert!(!should_remove_files(
            true,
            true,
            RefactorDocsOutputFormat::Summary
        ));
        assert!(!should_remove_files(
            true,
            false,
            RefactorDocsOutputFormat::Interactive
        ));
    }

    #[test]
    fn test_should_remove_files_no_auto_not_interactive() {
        assert!(!should_remove_files(
            false,
            false,
            RefactorDocsOutputFormat::Summary
        ));
        assert!(!should_remove_files(
            false,
            false,
            RefactorDocsOutputFormat::Json
        ));
    }
