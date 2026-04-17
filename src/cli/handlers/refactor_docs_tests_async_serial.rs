    // ========== Async Function Tests ==========

    #[tokio::test]
    async fn test_collect_files_flat() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
        std::fs::write(temp_dir.path().join("file2.txt"), "content2").unwrap();
        std::fs::create_dir(temp_dir.path().join("subdir")).unwrap();
        std::fs::write(temp_dir.path().join("subdir/file3.txt"), "content3").unwrap();

        let files = collect_files_flat(temp_dir.path()).await.unwrap();

        // Should only get files in root, not in subdir
        assert_eq!(files.len(), 2);
    }

    #[tokio::test]
    async fn test_collect_files_recursive() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
        std::fs::create_dir(temp_dir.path().join("subdir")).unwrap();
        std::fs::write(temp_dir.path().join("subdir/file2.txt"), "content2").unwrap();
        std::fs::create_dir(temp_dir.path().join("subdir/nested")).unwrap();
        std::fs::write(temp_dir.path().join("subdir/nested/file3.txt"), "content3").unwrap();

        let files = collect_files_recursive(temp_dir.path()).await.unwrap();

        // Should get all files including nested
        assert_eq!(files.len(), 3);
    }

    #[tokio::test]
    async fn test_process_directory_nonexistent() {
        let result = process_directory(
            Path::new("/nonexistent/directory"),
            &[],
            &[],
            0,
            u64::MAX,
            false,
            &SystemTime::now(),
        )
        .await
        .unwrap();

        assert!(result.cruft_files.is_empty());
        assert_eq!(result.files_scanned, 0);
        assert!(!result.errors.is_empty());
        assert!(result.errors[0].contains("does not exist"));
    }

    #[tokio::test]
    async fn test_process_directory_with_matches() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("fix-bug.sh"), "#!/bin/bash").unwrap();
        std::fs::write(temp_dir.path().join("normal.txt"), "content").unwrap();

        let patterns = vec![("fix-*.sh".to_string(), FileCategory::TemporaryScript)];

        let result = process_directory(
            temp_dir.path(),
            &patterns,
            &[],
            0,
            u64::MAX,
            false,
            &SystemTime::now(),
        )
        .await
        .unwrap();

        assert_eq!(result.cruft_files.len(), 1);
        assert!(result.cruft_files[0]
            .path
            .to_string_lossy()
            .contains("fix-bug.sh"));
    }

    #[tokio::test]
    async fn test_process_directory_with_preservation() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("fix-important.sh"), "#!/bin/bash").unwrap();

        let patterns = vec![("fix-*.sh".to_string(), FileCategory::TemporaryScript)];
        let preserve = vec!["*-important.sh".to_string()];

        let result = process_directory(
            temp_dir.path(),
            &patterns,
            &preserve,
            0,
            u64::MAX,
            false,
            &SystemTime::now(),
        )
        .await
        .unwrap();

        assert!(result.cruft_files.is_empty());
        assert_eq!(result.preserved_files.len(), 1);
    }

    #[tokio::test]
    async fn test_scan_for_cruft_multiple_dirs() {
        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();

        std::fs::write(temp_dir1.path().join("fix-1.sh"), "#!/bin/bash").unwrap();
        std::fs::write(temp_dir2.path().join("fix-2.sh"), "#!/bin/bash").unwrap();

        let patterns = vec![("fix-*.sh".to_string(), FileCategory::TemporaryScript)];

        let result = scan_for_cruft(
            &[
                temp_dir1.path().to_path_buf(),
                temp_dir2.path().to_path_buf(),
            ],
            &patterns,
            &[],
            0,
            u64::MAX,
            false,
        )
        .await
        .unwrap();

        assert_eq!(result.cruft_files.len(), 2);
        assert_eq!(result.summary.cruft_files_found, 2);
    }

    #[tokio::test]
    async fn test_perform_cruft_scan_sorting() {
        let temp_dir = TempDir::new().unwrap();

        // Create files of different sizes
        std::fs::write(temp_dir.path().join("small.sh"), "x").unwrap();
        std::fs::write(temp_dir.path().join("medium.sh"), "xxxxx").unwrap();
        std::fs::write(temp_dir.path().join("large.sh"), "xxxxxxxxxx").unwrap();

        let patterns = vec![("*.sh".to_string(), FileCategory::TemporaryScript)];

        let result = perform_cruft_scan(
            &[temp_dir.path().to_path_buf()],
            &patterns,
            &[],
            0,
            100,
            false,
        )
        .await
        .unwrap();

        // Files should be sorted by size (largest first)
        assert_eq!(result.cruft_files.len(), 3);
        assert!(result.cruft_files[0].size_bytes >= result.cruft_files[1].size_bytes);
        assert!(result.cruft_files[1].size_bytes >= result.cruft_files[2].size_bytes);
    }

    #[tokio::test]
    async fn test_remove_files_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("to_remove.txt");
        std::fs::write(&file_path, "content").unwrap();

        let files = vec![CruftFile {
            path: file_path.clone(),
            category: FileCategory::TemporaryScript,
            size_bytes: 7,
            modified: SystemTime::now(),
            age_days: 0,
            reason: "test".to_string(),
            pattern_matched: "*.txt".to_string(),
        }];

        remove_files(&files).await.unwrap();

        assert!(!file_path.exists());
    }

    #[tokio::test]
    async fn test_remove_files_nonexistent() {
        let files = vec![CruftFile {
            path: PathBuf::from("/nonexistent/file.txt"),
            category: FileCategory::TemporaryScript,
            size_bytes: 0,
            modified: SystemTime::now(),
            age_days: 0,
            reason: "test".to_string(),
            pattern_matched: "*.txt".to_string(),
        }];

        // Should not panic, just log errors
        let result = remove_files(&files).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_backup() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = TempDir::new().unwrap();

        let file_path = temp_dir.path().join("to_backup.txt");
        std::fs::write(&file_path, "backup content").unwrap();

        let files = vec![CruftFile {
            path: file_path.clone(),
            category: FileCategory::TemporaryScript,
            size_bytes: 14,
            modified: SystemTime::now(),
            age_days: 0,
            reason: "test".to_string(),
            pattern_matched: "*.txt".to_string(),
        }];

        create_backup(&files, backup_dir.path()).await.unwrap();

        // Verify backup directory was created
        let backup_entries: Vec<_> = std::fs::read_dir(backup_dir.path()).unwrap().collect();
        assert!(!backup_entries.is_empty());
    }

    #[tokio::test]
    async fn test_output_results_to_file() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("output.md");

        let result = RefactorDocsResult {
            cruft_files: vec![],
            summary: CleanupSummary::default(),
            preserved_files: vec![],
            errors: vec![],
        };

        output_results(
            &result,
            RefactorDocsOutputFormat::Summary,
            false,
            false,
            Duration::from_secs(1),
            Some(output_path.clone()),
        )
        .await
        .unwrap();

        assert!(output_path.exists());
        let content = std::fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("# Documentation Refactoring Report"));
    }

    #[tokio::test]
    async fn test_handle_backup_processing_skipped() {
        let result = RefactorDocsResult {
            cruft_files: vec![],
            summary: CleanupSummary::default(),
            preserved_files: vec![],
            errors: vec![],
        };

        // Dry run should skip backup
        let backup_result =
            handle_backup_processing(&result, true, true, false, Path::new("/tmp")).await;
        assert!(backup_result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_file_removal_processing_skipped() {
        let result = RefactorDocsResult {
            cruft_files: vec![],
            summary: CleanupSummary::default(),
            preserved_files: vec![],
            errors: vec![],
        };

        // Dry run should skip removal
        let removal_result =
            handle_file_removal_processing(&result, true, false, RefactorDocsOutputFormat::Summary)
                .await;
        assert!(removal_result.is_ok());
    }

    // ========== Edge Case Tests ==========

    #[test]
    fn test_should_preserve_empty_filename() {
        let patterns = vec!["*.txt".to_string()];
        // Path with no filename
        assert!(!should_preserve(Path::new("/"), &patterns));
    }

    #[test]
    fn test_matches_pattern_empty_filename() {
        let patterns = vec![("*.txt".to_string(), FileCategory::TemporaryScript)];
        assert_eq!(matches_pattern(Path::new("/"), &patterns), None);
    }

    #[test]
    fn test_calculate_age_days_future_modification() {
        // Create a file with current metadata
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "test").unwrap();
        let metadata = std::fs::metadata(&file_path).unwrap();

        // Use a "now" that's in the past relative to the file
        let past_now = SystemTime::UNIX_EPOCH;
        let age = calculate_age_days(&metadata, &past_now);

        // When now is before modified, duration_since returns error, so age is 0
        assert_eq!(age, 0);
    }

    #[test]
    fn test_update_summary_oldest_newest_tracking() {
        let mut summary = CleanupSummary::default();

        // First file: 10 days old
        let cruft1 = CruftFile {
            path: PathBuf::from("/tmp/a.txt"),
            category: FileCategory::TemporaryScript,
            size_bytes: 100,
            modified: SystemTime::now(),
            age_days: 10,
            reason: "test".to_string(),
            pattern_matched: "*.txt".to_string(),
        };
        update_summary_for_cruft(&mut summary, &cruft1);
        assert_eq!(summary.oldest_file_days, 10);
        assert_eq!(summary.newest_file_days, 10);

        // Second file: 5 days old (newer)
        let cruft2 = CruftFile {
            path: PathBuf::from("/tmp/b.txt"),
            category: FileCategory::TemporaryScript,
            size_bytes: 100,
            modified: SystemTime::now(),
            age_days: 5,
            reason: "test".to_string(),
            pattern_matched: "*.txt".to_string(),
        };
        update_summary_for_cruft(&mut summary, &cruft2);
        assert_eq!(summary.oldest_file_days, 10);
        assert_eq!(summary.newest_file_days, 5);

        // Third file: 20 days old (older)
        let cruft3 = CruftFile {
            path: PathBuf::from("/tmp/c.txt"),
            category: FileCategory::TemporaryScript,
            size_bytes: 100,
            modified: SystemTime::now(),
            age_days: 20,
            reason: "test".to_string(),
            pattern_matched: "*.txt".to_string(),
        };
        update_summary_for_cruft(&mut summary, &cruft3);
        assert_eq!(summary.oldest_file_days, 20);
        assert_eq!(summary.newest_file_days, 5);
    }

    // ========== Serialization Tests ==========

    #[test]
    fn test_file_category_serialization() {
        let category = FileCategory::BuildArtifact;
        let json = serde_json::to_string(&category).unwrap();
        assert_eq!(json, "\"BuildArtifact\"");

        let deserialized: FileCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, FileCategory::BuildArtifact);
    }

    #[test]
    fn test_cruft_file_serialization() {
        let cruft = CruftFile {
            path: PathBuf::from("/tmp/test.txt"),
            category: FileCategory::StatusReport,
            size_bytes: 256,
            modified: SystemTime::UNIX_EPOCH,
            age_days: 42,
            reason: "Test reason".to_string(),
            pattern_matched: "*_STATUS.md".to_string(),
        };

        let json = serde_json::to_string(&cruft).unwrap();
        let deserialized: CruftFile = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.path, cruft.path);
        assert_eq!(deserialized.category, cruft.category);
        assert_eq!(deserialized.size_bytes, cruft.size_bytes);
        assert_eq!(deserialized.age_days, cruft.age_days);
    }

    #[test]
    fn test_cleanup_summary_serialization() {
        let mut summary = CleanupSummary {
            total_files_scanned: 100,
            cruft_files_found: 5,
            ..Default::default()
        };
        summary.files_by_category.insert("Test".to_string(), 3);

        let json = serde_json::to_string(&summary).unwrap();
        let deserialized: CleanupSummary = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.total_files_scanned, 100);
        assert_eq!(deserialized.cruft_files_found, 5);
        assert_eq!(deserialized.files_by_category.get("Test"), Some(&3));
    }
