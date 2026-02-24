    // ========== Edge Cases and Boundary Tests ==========

    #[test]
    fn test_baseline_empty_project() {
        let baseline = TdgBaseline::new(None);

        assert_eq!(baseline.summary.total_files, 0);
        assert_eq!(baseline.summary.avg_score, 0.0);
        assert!(baseline.summary.grade_distribution.is_empty());
        assert!(baseline.summary.languages.is_empty());
    }

    #[test]
    fn test_baseline_single_file() {
        let mut baseline = TdgBaseline::new(None);
        baseline.add_entry(
            PathBuf::from("single.rs"),
            create_test_entry(92.5, Grade::A),
        );

        assert_eq!(baseline.summary.total_files, 1);
        assert!((baseline.summary.avg_score - 92.5).abs() < 0.01);
        assert_eq!(
            *baseline.summary.grade_distribution.get(&Grade::A).unwrap(),
            1
        );
    }

    #[test]
    fn test_baseline_large_project() {
        let mut baseline = TdgBaseline::new(None);

        for i in 0..500 {
            let score = 70.0 + (i % 30) as f32;
            let grade = Grade::from_score(score);
            baseline.add_entry(
                PathBuf::from(format!("src/module{}/file.rs", i)),
                BaselineEntry {
                    content_hash: blake3::hash(format!("content{}", i).as_bytes()),
                    score: TdgScore {
                        total: score,
                        grade,
                        ..Default::default()
                    },
                    components: ComponentScores::default(),
                    git_context: None,
                },
            );
        }

        assert_eq!(baseline.summary.total_files, 500);
        // Average should be around 84.5 ((70+99)/2 roughly, accounting for modulo distribution)
        assert!(baseline.summary.avg_score > 80.0 && baseline.summary.avg_score < 90.0);
    }

    #[test]
    fn test_compare_empty_baselines() {
        let old = TdgBaseline::new(None);
        let new = TdgBaseline::new(None);

        let comparison = old.compare(&new);

        assert!(comparison.improved.is_empty());
        assert!(comparison.regressed.is_empty());
        assert!(comparison.unchanged.is_empty());
        assert!(comparison.added.is_empty());
        assert!(comparison.removed.is_empty());
        assert_eq!(comparison.total_changes(), 0);
        assert!(!comparison.has_regressions());
    }

    #[test]
    fn test_compare_with_floating_point_tolerance() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        // Delta of 0.005 should be considered unchanged (within 0.01 tolerance)
        old.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry(85.0, Grade::AMinus),
        );
        new.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry(85.005, Grade::AMinus),
        );

        let comparison = old.compare(&new);
        assert_eq!(comparison.unchanged.len(), 1);
        assert!(comparison.improved.is_empty());
        assert!(comparison.regressed.is_empty());
    }

    #[test]
    fn test_compare_at_tolerance_boundary() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        // Delta of 0.02 should be detected as improvement (above 0.01 tolerance)
        old.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry(85.0, Grade::AMinus),
        );
        new.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry(85.02, Grade::AMinus),
        );

        let comparison = old.compare(&new);
        assert_eq!(comparison.improved.len(), 1);
        assert!(comparison.unchanged.is_empty());
    }

    #[test]
    fn test_file_comparison_delta_calculation() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        old.add_entry(PathBuf::from("test.rs"), create_test_entry(75.5, Grade::B));
        new.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry(88.3, Grade::AMinus),
        );

        let comparison = old.compare(&new);
        assert_eq!(comparison.improved.len(), 1);

        let file_cmp = &comparison.improved[0];
        assert_eq!(file_cmp.path, PathBuf::from("test.rs"));
        assert!((file_cmp.delta - 12.8).abs() < 0.01);
        assert!((file_cmp.old_score.total - 75.5).abs() < 0.01);
        assert!((file_cmp.new_score.total - 88.3).abs() < 0.01);
    }

    // ========== Summary Recomputation Tests ==========

    #[test]
    fn test_summary_recomputes_on_each_add() {
        let mut baseline = TdgBaseline::new(None);

        baseline.add_entry(PathBuf::from("a.rs"), create_test_entry(80.0, Grade::BPlus));
        assert_eq!(baseline.summary.total_files, 1);
        assert!((baseline.summary.avg_score - 80.0).abs() < 0.01);

        baseline.add_entry(PathBuf::from("b.rs"), create_test_entry(90.0, Grade::A));
        assert_eq!(baseline.summary.total_files, 2);
        assert!((baseline.summary.avg_score - 85.0).abs() < 0.01);

        baseline.add_entry(
            PathBuf::from("c.rs"),
            create_test_entry(70.0, Grade::BMinus),
        );
        assert_eq!(baseline.summary.total_files, 3);
        assert!((baseline.summary.avg_score - 80.0).abs() < 0.01);
    }

    #[test]
    fn test_summary_grade_distribution_clears_on_recompute() {
        let mut baseline = TdgBaseline::new(None);

        baseline.add_entry(PathBuf::from("a.rs"), create_test_entry(95.0, Grade::APLus));
        assert_eq!(
            *baseline
                .summary
                .grade_distribution
                .get(&Grade::APLus)
                .unwrap(),
            1
        );

        // Overwrite with different grade
        baseline.add_entry(PathBuf::from("a.rs"), create_test_entry(75.0, Grade::B));
        assert!(baseline
            .summary
            .grade_distribution
            .get(&Grade::APLus)
            .is_none());
        assert_eq!(
            *baseline.summary.grade_distribution.get(&Grade::B).unwrap(),
            1
        );
    }

    #[test]
    fn test_summary_language_distribution_clears_on_recompute() {
        let mut baseline = TdgBaseline::new(None);

        baseline.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry_with_lang(90.0, Grade::A, Language::Rust),
        );
        assert_eq!(
            *baseline
                .summary
                .languages
                .get(&format!("{:?}", Language::Rust))
                .unwrap(),
            1
        );

        // Overwrite with different language
        baseline.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry_with_lang(90.0, Grade::A, Language::Python),
        );
        assert!(baseline
            .summary
            .languages
            .get(&format!("{:?}", Language::Rust))
            .is_none());
        assert_eq!(
            *baseline
                .summary
                .languages
                .get(&format!("{:?}", Language::Python))
                .unwrap(),
            1
        );
    }

    // ========== recompute_summary empty files branch ==========

    #[test]
    fn test_recompute_summary_empty_files() {
        let mut baseline = TdgBaseline::new(None);
        // Add an entry so summary has non-zero values
        baseline.add_entry(PathBuf::from("test.rs"), create_test_entry(90.0, Grade::A));
        assert_eq!(baseline.summary.total_files, 1);
        assert!((baseline.summary.avg_score - 90.0).abs() < 0.01);

        // Clear files directly (pub field) and call recompute
        baseline.files.clear();
        baseline.recompute_summary();

        assert_eq!(baseline.summary.total_files, 0);
        assert_eq!(baseline.summary.avg_score, 0.0);
    }

    // ========== All Grade Distribution Tests ==========

    #[test]
    fn test_all_grades_in_distribution() {
        let mut baseline = TdgBaseline::new(None);

        let grades_and_scores = [
            ("a.rs", Grade::APLus, 96.0),
            ("b.rs", Grade::A, 92.0),
            ("c.rs", Grade::AMinus, 87.0),
            ("d.rs", Grade::BPlus, 82.0),
            ("e.rs", Grade::B, 77.0),
            ("f.rs", Grade::BMinus, 72.0),
            ("g.rs", Grade::CPlus, 67.0),
            ("h.rs", Grade::C, 62.0),
            ("i.rs", Grade::CMinus, 57.0),
            ("j.rs", Grade::D, 52.0),
            ("k.rs", Grade::F, 40.0),
        ];

        for (path, grade, score) in grades_and_scores {
            baseline.add_entry(PathBuf::from(path), create_test_entry(score, grade));
        }

        assert_eq!(baseline.summary.total_files, 11);
        for (_, grade, _) in grades_and_scores {
            assert_eq!(
                *baseline
                    .summary
                    .grade_distribution
                    .get(&grade)
                    .unwrap_or(&0),
                1,
                "Grade {:?} should have count 1",
                grade
            );
        }
    }
