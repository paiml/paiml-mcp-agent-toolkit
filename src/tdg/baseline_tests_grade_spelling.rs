    // ========== format_text() grade spelling ==========
    //
    // `BaselineComparison::format_text` rendered grades with `{:?}`, so
    // `pmat tdg baseline compare` printed the Rust variant name:
    //     - ./src/lib.rs: APlus (100.0) → F (25.2) [-74.8]
    // Every sibling printer (check-regression, tdg, analyze tdg, check-quality,
    // and the JSON/SARIF payloads) goes through `Grade`'s `Display`, which
    // emits the symbolic form. One spelling, one impl.

    #[test]
    fn format_text_prints_symbolic_grades_not_debug_variant_names() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        // Regressed: A+ → F
        old.add_entry(
            PathBuf::from("regressed.rs"),
            create_test_entry(100.0, Grade::APlus),
        );
        new.add_entry(
            PathBuf::from("regressed.rs"),
            create_test_entry(25.2, Grade::F),
        );
        // Improved: B- → A+
        old.add_entry(
            PathBuf::from("improved.rs"),
            create_test_entry(80.0, Grade::BMinus),
        );
        new.add_entry(
            PathBuf::from("improved.rs"),
            create_test_entry(100.0, Grade::APlus),
        );

        let text = old.compare(&new).format_text();

        for debug_spelling in [
            "APlus", "AMinus", "BPlus", "BMinus", "CPlus", "CMinus",
        ] {
            assert!(
                !text.contains(debug_spelling),
                "format_text leaked the Debug variant name {debug_spelling:?}:\n{text}"
            );
        }
        assert!(
            text.contains("A+ (100.0) → F (25.2)"),
            "expected the symbolic spelling Display emits, got:\n{text}"
        );
        assert!(
            text.contains("B- (80.0) → A+ (100.0)"),
            "expected the symbolic spelling Display emits, got:\n{text}"
        );
    }
