
    // ========== passes_file_filters Tests ==========

    #[test]
    fn test_passes_file_filters_size_exceeded() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("large.txt");
        std::fs::write(&file_path, vec![0u8; 2000]).unwrap();
        let metadata = std::fs::metadata(&file_path).unwrap();
        let now = SystemTime::now();

        // Max size is 1000 bytes, file is 2000
        assert!(!passes_file_filters(&metadata, 0, 1000, &now));
    }

    #[test]
    fn test_passes_file_filters_size_within_limit() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("small.txt");
        std::fs::write(&file_path, vec![0u8; 500]).unwrap();
        let metadata = std::fs::metadata(&file_path).unwrap();
        let now = SystemTime::now();

        // Max size is 1000 bytes, file is 500
        assert!(passes_file_filters(&metadata, 0, 1000, &now));
    }

    #[test]
    fn test_passes_file_filters_too_new() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("new.txt");
        std::fs::write(&file_path, "test").unwrap();
        let metadata = std::fs::metadata(&file_path).unwrap();
        let now = SystemTime::now();

        // File is brand new (0 days old), but min age is 7 days
        assert!(!passes_file_filters(&metadata, 7, u64::MAX, &now));
    }

    #[test]
    fn test_passes_file_filters_old_enough() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("old.txt");
        std::fs::write(&file_path, "test").unwrap();
        let metadata = std::fs::metadata(&file_path).unwrap();
        let now = SystemTime::now();

        // Min age is 0 days
        assert!(passes_file_filters(&metadata, 0, u64::MAX, &now));
    }

    // ========== calculate_age_days Tests ==========

    #[test]
    fn test_calculate_age_days_recent() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("recent.txt");
        std::fs::write(&file_path, "test").unwrap();
        let metadata = std::fs::metadata(&file_path).unwrap();
        let now = SystemTime::now();

        let age = calculate_age_days(&metadata, &now);
        assert_eq!(age, 0);
    }

    #[test]
    fn test_calculate_age_days_with_offset() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "test").unwrap();
        let metadata = std::fs::metadata(&file_path).unwrap();

        // Simulate 3 days from now
        let now = SystemTime::now() + Duration::from_secs(3 * 86400);
        let age = calculate_age_days(&metadata, &now);
        assert_eq!(age, 3);
    }

    // ========== update_summary_for_cruft Tests ==========

    #[test]
    fn test_update_summary_for_cruft_first_file() {
        let mut summary = CleanupSummary::default();
        let cruft = CruftFile {
            path: PathBuf::from("/tmp/test.txt"),
            category: FileCategory::TemporaryScript,
            size_bytes: 1024,
            modified: SystemTime::now(),
            age_days: 5,
            reason: "test".to_string(),
            pattern_matched: "*.txt".to_string(),
        };

        update_summary_for_cruft(&mut summary, &cruft);

        assert_eq!(summary.files_by_category.get("Temporary Script"), Some(&1));
        assert_eq!(
            summary.size_by_category.get("Temporary Script"),
            Some(&1024)
        );
        assert_eq!(summary.oldest_file_days, 5);
        assert_eq!(summary.newest_file_days, 5);
    }

    #[test]
    fn test_update_summary_for_cruft_multiple_files() {
        let mut summary = CleanupSummary::default();

        let cruft1 = CruftFile {
            path: PathBuf::from("/tmp/test1.txt"),
            category: FileCategory::TemporaryScript,
            size_bytes: 1024,
            modified: SystemTime::now(),
            age_days: 10,
            reason: "test".to_string(),
            pattern_matched: "*.txt".to_string(),
        };
        update_summary_for_cruft(&mut summary, &cruft1);

        let cruft2 = CruftFile {
            path: PathBuf::from("/tmp/test2.txt"),
            category: FileCategory::TemporaryScript,
            size_bytes: 512,
            modified: SystemTime::now(),
            age_days: 3,
            reason: "test".to_string(),
            pattern_matched: "*.txt".to_string(),
        };
        update_summary_for_cruft(&mut summary, &cruft2);

        assert_eq!(summary.files_by_category.get("Temporary Script"), Some(&2));
        assert_eq!(
            summary.size_by_category.get("Temporary Script"),
            Some(&1536)
        );
        assert_eq!(summary.oldest_file_days, 10);
        assert_eq!(summary.newest_file_days, 3);
    }

    #[test]
    fn test_update_summary_for_cruft_different_categories() {
        let mut summary = CleanupSummary::default();

        let cruft1 = CruftFile {
            path: PathBuf::from("/tmp/script.sh"),
            category: FileCategory::TemporaryScript,
            size_bytes: 1024,
            modified: SystemTime::now(),
            age_days: 5,
            reason: "test".to_string(),
            pattern_matched: "*.sh".to_string(),
        };
        update_summary_for_cruft(&mut summary, &cruft1);

        let cruft2 = CruftFile {
            path: PathBuf::from("/tmp/build.o"),
            category: FileCategory::BuildArtifact,
            size_bytes: 2048,
            modified: SystemTime::now(),
            age_days: 7,
            reason: "test".to_string(),
            pattern_matched: "*.o".to_string(),
        };
        update_summary_for_cruft(&mut summary, &cruft2);

        assert_eq!(summary.files_by_category.get("Temporary Script"), Some(&1));
        assert_eq!(summary.files_by_category.get("Build Artifact"), Some(&1));
        assert_eq!(
            summary.size_by_category.get("Temporary Script"),
            Some(&1024)
        );
        assert_eq!(summary.size_by_category.get("Build Artifact"), Some(&2048));
    }

    // ========== merge_summary Tests ==========

    #[test]
    fn test_merge_summary_empty() {
        let mut main = CleanupSummary::default();
        let dir = CleanupSummary::default();
        merge_summary(&mut main, &dir);

        assert!(main.files_by_category.is_empty());
        assert_eq!(main.oldest_file_days, 0);
        assert_eq!(main.newest_file_days, 0);
    }

    #[test]
    fn test_merge_summary_into_empty() {
        let mut main = CleanupSummary::default();
        let mut dir = CleanupSummary::default();
        dir.files_by_category
            .insert("Temporary Script".to_string(), 5);
        dir.size_by_category
            .insert("Temporary Script".to_string(), 5000);
        dir.oldest_file_days = 30;
        dir.newest_file_days = 2;

        merge_summary(&mut main, &dir);

        assert_eq!(main.files_by_category.get("Temporary Script"), Some(&5));
        assert_eq!(main.size_by_category.get("Temporary Script"), Some(&5000));
        assert_eq!(main.oldest_file_days, 30);
        assert_eq!(main.newest_file_days, 2);
    }

    #[test]
    fn test_merge_summary_combine() {
        let mut main = CleanupSummary::default();
        main.files_by_category
            .insert("Temporary Script".to_string(), 3);
        main.size_by_category
            .insert("Temporary Script".to_string(), 3000);
        main.oldest_file_days = 20;
        main.newest_file_days = 5;

        let mut dir = CleanupSummary::default();
        dir.files_by_category
            .insert("Temporary Script".to_string(), 2);
        dir.size_by_category
            .insert("Temporary Script".to_string(), 2000);
        dir.oldest_file_days = 40;
        dir.newest_file_days = 1;

        merge_summary(&mut main, &dir);

        assert_eq!(main.files_by_category.get("Temporary Script"), Some(&5));
        assert_eq!(main.size_by_category.get("Temporary Script"), Some(&5000));
        assert_eq!(main.oldest_file_days, 40);
        assert_eq!(main.newest_file_days, 1);
    }

    // ========== finalize_summary Tests ==========

    #[test]
    fn test_finalize_summary_empty() {
        let mut summary = CleanupSummary::default();
        let cruft_files: Vec<CruftFile> = vec![];

        finalize_summary(&mut summary, 100, &cruft_files);

        assert_eq!(summary.total_files_scanned, 100);
        assert_eq!(summary.cruft_files_found, 0);
        assert_eq!(summary.total_size_bytes, 0);
    }

    #[test]
    fn test_finalize_summary_with_files() {
        let mut summary = CleanupSummary::default();
        let cruft_files = vec![
            CruftFile {
                path: PathBuf::from("/tmp/a.txt"),
                category: FileCategory::TemporaryScript,
                size_bytes: 1000,
                modified: SystemTime::now(),
                age_days: 1,
                reason: "test".to_string(),
                pattern_matched: "*.txt".to_string(),
            },
            CruftFile {
                path: PathBuf::from("/tmp/b.txt"),
                category: FileCategory::StatusReport,
                size_bytes: 500,
                modified: SystemTime::now(),
                age_days: 2,
                reason: "test".to_string(),
                pattern_matched: "*.txt".to_string(),
            },
        ];

        finalize_summary(&mut summary, 50, &cruft_files);

        assert_eq!(summary.total_files_scanned, 50);
        assert_eq!(summary.cruft_files_found, 2);
        assert_eq!(summary.total_size_bytes, 1500);
    }

