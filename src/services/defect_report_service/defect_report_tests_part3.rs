
    #[test]
    fn test_filter_by_pattern_rebuilds_file_index() {
        let defects = vec![
            build_defect_for_file("IDX-001", "src/main.rs", Severity::High),
            build_defect_for_file("IDX-002", "tests/test.rs", Severity::High),
        ];
        let report = build_test_report(defects);

        let filtered =
            DefectReportService::filter_by_pattern(&report, Some("src/*.rs".to_string()), None, 0);

        assert_eq!(filtered.file_index.len(), 1);
        assert!(filtered
            .file_index
            .contains_key(&PathBuf::from("src/main.rs")));
    }

    #[test]
    fn test_filter_by_pattern_recomputes_summary() {
        let defects = vec![
            build_defect_for_file("SUM-001", "src/main.rs", Severity::Critical),
            build_defect_for_file("SUM-002", "tests/test.rs", Severity::Low),
        ];
        let report = build_test_report(defects);

        let filtered =
            DefectReportService::filter_by_pattern(&report, Some("src/*.rs".to_string()), None, 0);

        assert_eq!(filtered.summary.total_defects, 1);
        assert_eq!(filtered.summary.by_severity.get("critical"), Some(&1));
        assert!(filtered.summary.by_severity.get("low").is_none());
    }

    #[test]
    fn test_filter_by_pattern_preserves_metadata() {
        let defects = build_diverse_defects();
        let report = build_test_report(defects);

        let filtered = DefectReportService::filter_by_pattern(&report, None, None, 0);

        assert_eq!(filtered.metadata.tool, report.metadata.tool);
        assert_eq!(filtered.metadata.version, report.metadata.version);
        assert_eq!(filtered.metadata.project_root, report.metadata.project_root);
    }

    #[test]
    fn test_filter_by_pattern_invalid_glob_handled() {
        let defects = build_diverse_defects();
        let report = build_test_report(defects);

        // Invalid glob pattern should be handled gracefully (returns None matcher)
        let filtered = DefectReportService::filter_by_pattern(
            &report,
            Some("[[[invalid".to_string()),
            None,
            0,
        );

        // With invalid include pattern, nothing should be filtered
        assert_eq!(filtered.defects.len(), report.defects.len());
    }

    #[test]
    fn test_filter_by_pattern_empty_result() {
        let defects = vec![build_defect_for_file(
            "EMPTY-001",
            "src/main.rs",
            Severity::High,
        )];
        let report = build_test_report(defects);

        let filtered = DefectReportService::filter_by_pattern(
            &report,
            Some("nonexistent/*.rs".to_string()),
            None,
            0,
        );

        assert!(filtered.defects.is_empty());
        assert!(filtered.file_index.is_empty());
        assert_eq!(filtered.summary.total_defects, 0);
    }

    // ReportFormat Tests

    #[test]
    fn test_report_format_debug() {
        assert_eq!(format!("{:?}", ReportFormat::Json), "Json");
        assert_eq!(format!("{:?}", ReportFormat::Csv), "Csv");
        assert_eq!(format!("{:?}", ReportFormat::Markdown), "Markdown");
        assert_eq!(format!("{:?}", ReportFormat::Text), "Text");
    }

    #[test]
    fn test_report_format_clone() {
        let format = ReportFormat::Json;
        let cloned = format.clone();
        assert!(matches!(cloned, ReportFormat::Json));
    }

    #[test]
    fn test_report_format_copy() {
        let format = ReportFormat::Csv;
        let copied: ReportFormat = format;
        assert!(matches!(copied, ReportFormat::Csv));
        // format is still usable because it's Copy
        assert!(matches!(format, ReportFormat::Csv));
    }

    // Edge Cases and Error Handling

    #[test]
    fn test_format_with_unicode_content() {
        let service = DefectReportService::new();
        let mut defect = build_defect("UNICODE-001", Severity::High, DefectCategory::Complexity);
        defect.message = "Test with unicode: \u{1F600} \u{4E2D}\u{6587} \u{0394}".to_string();
        defect.fix_suggestion = Some("Use proper \u{03BB} function".to_string());
        let report = build_test_report(vec![defect]);

        // All formats should handle unicode
        let json = service.format_json(&report).unwrap();
        assert!(json.contains("\\u{1F600}") || json.contains("\u{1F600}"));

        let csv = service.format_csv(&report).unwrap();
        assert!(!csv.is_empty());

        let md = service.format_markdown(&report).unwrap();
        assert!(!md.is_empty());

        let text = service.format_text(&report).unwrap();
        assert!(!text.is_empty());
    }

    #[test]
    fn test_format_with_special_csv_characters() {
        let service = DefectReportService::new();
        let mut defect = build_defect("CSV-SPECIAL", Severity::High, DefectCategory::Complexity);
        defect.message = "Message with, comma and \"quotes\"".to_string();
        let report = build_test_report(vec![defect]);

        let csv = service.format_csv(&report).unwrap();
        // CSV should properly escape these characters
        assert!(csv.contains("CSV-SPECIAL"));
    }

    #[test]
    fn test_format_with_long_paths() {
        let service = DefectReportService::new();
        let long_path = "a/".repeat(50) + "very_long_file_name.rs";
        let defect = build_defect_for_file("LONG-001", &long_path, Severity::High);
        let report = build_test_report(vec![defect]);

        // All formats should handle long paths
        let json = service.format_json(&report).unwrap();
        assert!(json.contains("very_long_file_name.rs"));

        let csv = service.format_csv(&report).unwrap();
        assert!(csv.contains("very_long_file_name.rs"));

        let md = service.format_markdown(&report).unwrap();
        assert!(md.contains("very_long_file_name.rs"));

        let text = service.format_text(&report).unwrap();
        assert!(text.contains("very_long_file_name.rs"));
    }

    #[test]
    fn test_format_with_empty_message() {
        let service = DefectReportService::new();
        let mut defect = build_defect("EMPTY-MSG", Severity::High, DefectCategory::Complexity);
        defect.message = String::new();
        let report = build_test_report(vec![defect]);

        // Should not panic with empty message
        let json = service.format_json(&report).unwrap();
        assert!(json.contains("EMPTY-MSG"));

        let csv = service.format_csv(&report).unwrap();
        assert!(csv.contains("EMPTY-MSG"));
    }

    #[test]
    fn test_large_report_performance() {
        let service = DefectReportService::new();

        // Create a large number of defects
        let defects: Vec<Defect> = (0..1000)
            .map(|i| {
                let file = format!("src/file_{}.rs", i % 50);
                build_complete_defect(
                    &format!("PERF-{:04}", i),
                    Severity::High,
                    DefectCategory::Complexity,
                    &file,
                )
            })
            .collect();

        let report = build_test_report(defects);

        // All formats should complete without issues
        let json = service.format_json(&report).unwrap();
        assert!(json.len() > 10000);

        let csv = service.format_csv(&report).unwrap();
        assert!(csv.lines().count() > 1000);

        let md = service.format_markdown(&report).unwrap();
        assert!(md.len() > 1000);

        let text = service.format_text(&report).unwrap();
        assert!(text.len() > 1000);
    }

    // Property-Based Tests with Proptest

    proptest! {
        /// Property: compute_summary.total_defects always equals input defects count
        #[test]
        fn prop_summary_total_equals_input_count(count in 0usize..100) {
            let service = DefectReportService::new();
            let defects: Vec<Defect> = (0..count)
                .map(|i| build_defect(&format!("PROP-{}", i), Severity::High, DefectCategory::Complexity))
                .collect();

            let summary = service.compute_summary(&defects);
            prop_assert_eq!(summary.total_defects, count);
        }

        /// Property: hotspot files never exceed 10
        #[test]
        fn prop_hotspots_max_10(file_count in 1usize..50) {
            let service = DefectReportService::new();
            let defects: Vec<Defect> = (0..file_count)
                .map(|i| build_defect_for_file(&format!("PROP-{}", i), &format!("file_{}.rs", i), Severity::High))
                .collect();

            let summary = service.compute_summary(&defects);
            prop_assert!(summary.hotspot_files.len() <= 10);
        }

        /// Property: severity counts sum to total defects
        #[test]
        fn prop_severity_counts_sum_to_total(
            critical in 0usize..20,
            high in 0usize..20,
            medium in 0usize..20,
            low in 0usize..20,
        ) {
            let service = DefectReportService::new();
            let mut defects = Vec::new();

            for i in 0..critical {
                defects.push(build_defect(&format!("C-{}", i), Severity::Critical, DefectCategory::Complexity));
            }
            for i in 0..high {
                defects.push(build_defect(&format!("H-{}", i), Severity::High, DefectCategory::Complexity));
            }
            for i in 0..medium {
                defects.push(build_defect(&format!("M-{}", i), Severity::Medium, DefectCategory::Complexity));
            }
            for i in 0..low {
                defects.push(build_defect(&format!("L-{}", i), Severity::Low, DefectCategory::Complexity));
            }

            let summary = service.compute_summary(&defects);
            let severity_sum: usize = summary.by_severity.values().sum();
            prop_assert_eq!(severity_sum, summary.total_defects);
        }

        /// Property: category counts sum to total defects
        #[test]
        fn prop_category_counts_sum_to_total(count in 0usize..50) {
            let service = DefectReportService::new();
            let categories = DefectCategory::all();
            let defects: Vec<Defect> = (0..count)
                .map(|i| {
                    let category = categories[i % categories.len()];
                    build_defect(&format!("CAT-{}", i), Severity::High, category)
                })
                .collect();

            let summary = service.compute_summary(&defects);
            let category_sum: usize = summary.by_category.values().sum();
            prop_assert_eq!(category_sum, summary.total_defects);
        }

        /// Property: hotspots are sorted by severity_score descending
        #[test]
        fn prop_hotspots_sorted_descending(file_count in 2usize..20) {
            let service = DefectReportService::new();
            let severities = [Severity::Critical, Severity::High, Severity::Medium, Severity::Low];
            let defects: Vec<Defect> = (0..file_count)
                .map(|i| {
                    let severity = severities[i % severities.len()];
                    build_defect_for_file(&format!("SORT-{}", i), &format!("file_{}.rs", i), severity)
                })
                .collect();

            let summary = service.compute_summary(&defects);

            // Verify descending order
            for i in 1..summary.hotspot_files.len() {
                prop_assert!(summary.hotspot_files[i-1].severity_score >= summary.hotspot_files[i].severity_score);
            }
        }

        /// Property: filter_by_pattern preserves defect content
        #[test]
        fn prop_filter_preserves_defect_content(count in 1usize..20) {
            let defects: Vec<Defect> = (0..count)
                .map(|i| build_defect_for_file(&format!("PRESERVE-{}", i), "src/test.rs", Severity::High))
                .collect();
            let report = build_test_report(defects.clone());

            let filtered = DefectReportService::filter_by_pattern(&report, None, None, 0);

            for (original, filtered) in defects.iter().zip(filtered.defects.iter()) {
                prop_assert_eq!(&original.id, &filtered.id);
                prop_assert_eq!(&original.message, &filtered.message);
                prop_assert_eq!(original.severity, filtered.severity);
            }
        }

        /// Property: JSON format is always valid JSON
        #[test]
        fn prop_json_always_valid(count in 0usize..30) {
            let service = DefectReportService::new();
            let defects: Vec<Defect> = (0..count)
                .map(|i| build_defect(&format!("JSON-{}", i), Severity::High, DefectCategory::Complexity))
                .collect();
            let report = build_test_report(defects);

            let json = service.format_json(&report).unwrap();
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json);
            prop_assert!(parsed.is_ok());
        }

        /// Property: CSV has correct number of lines (header + defects)
        #[test]
        fn prop_csv_line_count(count in 0usize..50) {
            let service = DefectReportService::new();
            let defects: Vec<Defect> = (0..count)
                .map(|i| build_defect(&format!("CSV-{}", i), Severity::High, DefectCategory::Complexity))
                .collect();
            let report = build_test_report(defects);

            let csv = service.format_csv(&report).unwrap();
            let line_count = csv.lines().count();
            // Header + defect lines
            prop_assert_eq!(line_count, count + 1);
        }

        /// Property: filename always has correct extension
        #[test]
        fn prop_filename_extension(format_index in 0usize..4) {
            let service = DefectReportService::new();
            let formats = [ReportFormat::Json, ReportFormat::Csv, ReportFormat::Markdown, ReportFormat::Text];
            let extensions = [".json", ".csv", ".md", ".txt"];

            let format = formats[format_index];
            let expected_ext = extensions[format_index];

            let filename = service.generate_filename(format);
            prop_assert!(filename.ends_with(expected_ext));
        }

        /// Property: filter with exclude removes all matching files
        #[test]
        fn prop_filter_exclude_removes_matching(count in 1usize..20) {
            // Create defects in two directories
            let mut defects = Vec::new();
            for i in 0..(count/2).max(1) {
                defects.push(build_defect_for_file(&format!("SRC-{}", i), "src/file.rs", Severity::High));
            }
            for i in 0..(count/2).max(1) {
                defects.push(build_defect_for_file(&format!("TEST-{}", i), "tests/file.rs", Severity::High));
            }
            let report = build_test_report(defects);

            let filtered = DefectReportService::filter_by_pattern(
                &report,
                None,
                Some("tests/*".to_string()),
                0,
            );

            // No defects should be from tests/
            for defect in &filtered.defects {
                prop_assert!(!defect.file_path.starts_with("tests/"));
            }
        }
    }

    // Concurrency Tests (for semaphore behavior)

    #[test]
    fn test_service_can_be_used_multiple_times() {
        let service = DefectReportService::new();

        // Use the service multiple times to compute summaries
        for i in 0..10 {
            let defects = vec![build_defect(
                &format!("MULTI-{}", i),
                Severity::High,
                DefectCategory::Complexity,
            )];
            let summary = service.compute_summary(&defects);
            assert_eq!(summary.total_defects, 1);
        }
    }

    #[test]
    fn test_multiple_services_can_coexist() {
        let service1 = DefectReportService::new();
        let service2 = DefectReportService::new();

        let defects1 = vec![build_defect(
            "SVC1-001",
            Severity::High,
            DefectCategory::Complexity,
        )];
        let defects2 = vec![build_defect(
            "SVC2-001",
            Severity::Low,
            DefectCategory::DeadCode,
        )];

        let summary1 = service1.compute_summary(&defects1);
        let summary2 = service2.compute_summary(&defects2);

        assert_eq!(summary1.by_severity.get("high"), Some(&1));
        assert_eq!(summary2.by_severity.get("low"), Some(&1));
    }
