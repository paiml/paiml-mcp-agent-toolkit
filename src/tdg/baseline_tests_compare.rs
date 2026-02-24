    // ========== TdgBaseline::compare() Tests ==========

    #[test]
    fn test_compare_detects_improvements() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        old.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry(80.0, Grade::BPlus),
        );
        new.add_entry(PathBuf::from("test.rs"), create_test_entry(90.0, Grade::A));

        let comparison = old.compare(&new);
        assert_eq!(comparison.improved.len(), 1);
        assert_eq!(comparison.regressed.len(), 0);
        assert!((comparison.improved[0].delta - 10.0).abs() < 0.01);
        assert_eq!(
            comparison.improved[0].grade_change,
            (Grade::BPlus, Grade::A)
        );
    }

    #[test]
    fn test_compare_detects_regressions() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        old.add_entry(PathBuf::from("test.rs"), create_test_entry(90.0, Grade::A));
        new.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry(80.0, Grade::BPlus),
        );

        let comparison = old.compare(&new);
        assert_eq!(comparison.improved.len(), 0);
        assert_eq!(comparison.regressed.len(), 1);
        assert!((comparison.regressed[0].delta + 10.0).abs() < 0.01);
        assert_eq!(
            comparison.regressed[0].grade_change,
            (Grade::A, Grade::BPlus)
        );
    }

    #[test]
    fn test_compare_detects_unchanged_files() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        old.add_entry(PathBuf::from("test.rs"), create_test_entry(90.0, Grade::A));
        new.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry(90.005, Grade::A),
        ); // Within tolerance

        let comparison = old.compare(&new);
        assert_eq!(comparison.unchanged.len(), 1);
        assert_eq!(comparison.improved.len(), 0);
        assert_eq!(comparison.regressed.len(), 0);
    }

    #[test]
    fn test_compare_detects_added_files() {
        let old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        new.add_entry(
            PathBuf::from("new_file.rs"),
            create_test_entry(85.0, Grade::AMinus),
        );

        let comparison = old.compare(&new);
        assert_eq!(comparison.added.len(), 1);
        assert_eq!(comparison.added[0], PathBuf::from("new_file.rs"));
    }

    #[test]
    fn test_compare_detects_removed_files() {
        let mut old = TdgBaseline::new(None);
        let new = TdgBaseline::new(None);

        old.add_entry(
            PathBuf::from("removed_file.rs"),
            create_test_entry(85.0, Grade::AMinus),
        );

        let comparison = old.compare(&new);
        assert_eq!(comparison.removed.len(), 1);
        assert_eq!(comparison.removed[0], PathBuf::from("removed_file.rs"));
    }

    #[test]
    fn test_compare_sorts_improvements_by_delta_descending() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        // Add files with different improvement magnitudes
        old.add_entry(
            PathBuf::from("small.rs"),
            create_test_entry(80.0, Grade::BPlus),
        );
        old.add_entry(
            PathBuf::from("large.rs"),
            create_test_entry(70.0, Grade::BMinus),
        );
        old.add_entry(
            PathBuf::from("medium.rs"),
            create_test_entry(75.0, Grade::B),
        );

        new.add_entry(
            PathBuf::from("small.rs"),
            create_test_entry(81.0, Grade::BPlus),
        ); // +1.0
        new.add_entry(
            PathBuf::from("large.rs"),
            create_test_entry(85.0, Grade::AMinus),
        ); // +15.0
        new.add_entry(
            PathBuf::from("medium.rs"),
            create_test_entry(80.0, Grade::BPlus),
        ); // +5.0

        let comparison = old.compare(&new);

        assert_eq!(comparison.improved.len(), 3);
        assert_eq!(comparison.improved[0].path, PathBuf::from("large.rs"));
        assert_eq!(comparison.improved[1].path, PathBuf::from("medium.rs"));
        assert_eq!(comparison.improved[2].path, PathBuf::from("small.rs"));
    }

    #[test]
    fn test_compare_sorts_regressions_by_delta_ascending() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        old.add_entry(
            PathBuf::from("small.rs"),
            create_test_entry(80.0, Grade::BPlus),
        );
        old.add_entry(PathBuf::from("large.rs"), create_test_entry(90.0, Grade::A));
        old.add_entry(
            PathBuf::from("medium.rs"),
            create_test_entry(85.0, Grade::AMinus),
        );

        new.add_entry(PathBuf::from("small.rs"), create_test_entry(79.0, Grade::B)); // -1.0
        new.add_entry(PathBuf::from("large.rs"), create_test_entry(75.0, Grade::B)); // -15.0
        new.add_entry(
            PathBuf::from("medium.rs"),
            create_test_entry(80.0, Grade::BPlus),
        ); // -5.0

        let comparison = old.compare(&new);

        assert_eq!(comparison.regressed.len(), 3);
        assert_eq!(comparison.regressed[0].path, PathBuf::from("large.rs"));
        assert_eq!(comparison.regressed[1].path, PathBuf::from("medium.rs"));
        assert_eq!(comparison.regressed[2].path, PathBuf::from("small.rs"));
    }

    #[test]
    fn test_compare_mixed_changes() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        // File that improves
        old.add_entry(
            PathBuf::from("improved.rs"),
            create_test_entry(70.0, Grade::BMinus),
        );
        new.add_entry(
            PathBuf::from("improved.rs"),
            create_test_entry(85.0, Grade::AMinus),
        );

        // File that regresses
        old.add_entry(
            PathBuf::from("regressed.rs"),
            create_test_entry(90.0, Grade::A),
        );
        new.add_entry(
            PathBuf::from("regressed.rs"),
            create_test_entry(75.0, Grade::B),
        );

        // File that stays the same
        old.add_entry(
            PathBuf::from("unchanged.rs"),
            create_test_entry(80.0, Grade::BPlus),
        );
        new.add_entry(
            PathBuf::from("unchanged.rs"),
            create_test_entry(80.0, Grade::BPlus),
        );

        // File that was removed
        old.add_entry(
            PathBuf::from("removed.rs"),
            create_test_entry(85.0, Grade::AMinus),
        );

        // File that was added
        new.add_entry(
            PathBuf::from("added.rs"),
            create_test_entry(88.0, Grade::AMinus),
        );

        let comparison = old.compare(&new);

        assert_eq!(comparison.improved.len(), 1);
        assert_eq!(comparison.regressed.len(), 1);
        assert_eq!(comparison.unchanged.len(), 1);
        assert_eq!(comparison.added.len(), 1);
        assert_eq!(comparison.removed.len(), 1);
    }

