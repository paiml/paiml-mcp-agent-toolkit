    // ========== BaselineComparison Tests ==========

    #[test]
    fn test_comparison_has_regressions() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        old.add_entry(PathBuf::from("test.rs"), create_test_entry(90.0, Grade::A));
        new.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry(80.0, Grade::BPlus),
        );

        let comparison = old.compare(&new);
        assert!(comparison.has_regressions());
    }

    #[test]
    fn test_comparison_no_regressions() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        old.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry(80.0, Grade::BPlus),
        );
        new.add_entry(PathBuf::from("test.rs"), create_test_entry(90.0, Grade::A));

        let comparison = old.compare(&new);
        assert!(!comparison.has_regressions());
    }

    #[test]
    fn test_comparison_total_changes() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        old.add_entry(
            PathBuf::from("improved.rs"),
            create_test_entry(70.0, Grade::BMinus),
        );
        old.add_entry(
            PathBuf::from("regressed.rs"),
            create_test_entry(90.0, Grade::A),
        );
        old.add_entry(
            PathBuf::from("removed.rs"),
            create_test_entry(85.0, Grade::AMinus),
        );

        new.add_entry(
            PathBuf::from("improved.rs"),
            create_test_entry(85.0, Grade::AMinus),
        );
        new.add_entry(
            PathBuf::from("regressed.rs"),
            create_test_entry(75.0, Grade::B),
        );
        new.add_entry(
            PathBuf::from("added.rs"),
            create_test_entry(88.0, Grade::AMinus),
        );

        let comparison = old.compare(&new);
        // 1 improved + 1 regressed + 1 added + 1 removed = 4 changes
        assert_eq!(comparison.total_changes(), 4);
    }

    #[test]
    fn test_comparison_format_text_improvements() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        old.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry(70.0, Grade::BMinus),
        );
        new.add_entry(PathBuf::from("test.rs"), create_test_entry(90.0, Grade::A));

        let comparison = old.compare(&new);
        let text = comparison.format_text();

        assert!(text.contains("Improved: 1 files"));
        assert!(text.contains("test.rs"));
    }

    #[test]
    fn test_comparison_format_text_regressions() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        old.add_entry(PathBuf::from("test.rs"), create_test_entry(90.0, Grade::A));
        new.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry(70.0, Grade::BMinus),
        );

        let comparison = old.compare(&new);
        let text = comparison.format_text();

        assert!(text.contains("Regressed: 1 files"));
        assert!(text.contains("test.rs"));
    }

    #[test]
    fn test_comparison_format_text_added_removed() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        old.add_entry(
            PathBuf::from("removed.rs"),
            create_test_entry(85.0, Grade::AMinus),
        );
        new.add_entry(
            PathBuf::from("added.rs"),
            create_test_entry(88.0, Grade::AMinus),
        );

        let comparison = old.compare(&new);
        let text = comparison.format_text();

        assert!(text.contains("Added: 1 files"));
        assert!(text.contains("added.rs"));
        assert!(text.contains("Removed: 1 files"));
        assert!(text.contains("removed.rs"));
    }

    #[test]
    fn test_comparison_format_text_unchanged() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        old.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry(85.0, Grade::AMinus),
        );
        new.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry(85.0, Grade::AMinus),
        );

        let comparison = old.compare(&new);
        let text = comparison.format_text();

        assert!(text.contains("Unchanged: 1 files"));
    }

