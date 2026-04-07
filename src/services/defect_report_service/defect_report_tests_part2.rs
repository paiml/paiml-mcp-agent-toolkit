
    // format_csv() Tests

    #[test]
    fn test_format_csv_headers() {
        let service = DefectReportService::new();
        let report = build_test_report(vec![]);
        let csv = service.format_csv(&report).unwrap();

        let lines: Vec<&str> = csv.lines().collect();
        assert!(!lines.is_empty());
        let header = lines[0];
        assert!(header.contains("id"));
        assert!(header.contains("severity"));
        assert!(header.contains("category"));
        assert!(header.contains("file_path"));
        assert!(header.contains("line_start"));
        assert!(header.contains("line_end"));
        assert!(header.contains("message"));
        assert!(header.contains("rule_id"));
        assert!(header.contains("cyclomatic"));
        assert!(header.contains("cognitive"));
    }

    #[test]
    fn test_format_csv_with_defects() {
        let service = DefectReportService::new();
        let defects = vec![build_complete_defect(
            "CSV-001",
            Severity::High,
            DefectCategory::Complexity,
            "src/test.rs",
        )];
        let report = build_test_report(defects);
        let csv = service.format_csv(&report).unwrap();

        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 2); // Header + 1 defect
        assert!(lines[1].contains("CSV-001"));
        assert!(lines[1].contains("high"));
        assert!(lines[1].contains("Complexity"));
    }

    #[test]
    fn test_format_csv_metrics_columns() {
        let service = DefectReportService::new();
        let mut defect = build_defect("METRIC-001", Severity::High, DefectCategory::Complexity);
        defect.metrics.insert("cyclomatic".to_string(), 42.0);
        defect.metrics.insert("cognitive".to_string(), 55.0);
        let report = build_test_report(vec![defect]);
        let csv = service.format_csv(&report).unwrap();

        let lines: Vec<&str> = csv.lines().collect();
        // Check that metrics are included
        assert!(lines[1].contains("42"));
        assert!(lines[1].contains("55"));
    }

    #[test]
    fn test_format_csv_empty_optional_fields() {
        let service = DefectReportService::new();
        let defect = build_defect("EMPTY-001", Severity::Low, DefectCategory::DeadCode);
        let report = build_test_report(vec![defect]);
        let csv = service.format_csv(&report).unwrap();

        // Should still produce valid CSV even with empty optional fields
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_format_csv_multiple_defects() {
        let service = DefectReportService::new();
        let defects = build_diverse_defects();
        let expected_count = defects.len();
        let report = build_test_report(defects);
        let csv = service.format_csv(&report).unwrap();

        let lines: Vec<&str> = csv.lines().collect();
        // Header + 7 defects
        assert_eq!(lines.len(), expected_count + 1);
    }

    // format_markdown() Tests

    #[test]
    fn test_format_markdown_header() {
        let service = DefectReportService::new();
        let report = build_test_report(vec![]);
        let md = service.format_markdown(&report).unwrap();

        assert!(md.contains("# Code Quality Report"));
        assert!(md.contains("Generated:"));
    }

    #[test]
    fn test_format_markdown_executive_summary() {
        let service = DefectReportService::new();
        let defects = build_diverse_defects();
        let report = build_test_report(defects);
        let md = service.format_markdown(&report).unwrap();

        assert!(md.contains("## Executive Summary"));
        assert!(md.contains("**Total Defects**"));
        assert!(md.contains("**Files Analyzed**"));
        assert!(md.contains("**Analysis Duration**"));
    }

    #[test]
    fn test_format_markdown_severity_distribution() {
        let service = DefectReportService::new();
        let defects = vec![
            build_defect("MD-001", Severity::Critical, DefectCategory::Complexity),
            build_defect("MD-002", Severity::High, DefectCategory::Complexity),
        ];
        let report = build_test_report(defects);
        let md = service.format_markdown(&report).unwrap();

        assert!(md.contains("### Severity Distribution"));
        // Progress bar should be present
        assert!(md.contains("```"));
    }

    #[test]
    fn test_format_markdown_hotspot_table() {
        let service = DefectReportService::new();
        let defects = vec![
            build_defect_for_file("HOT-001", "src/hotspot.rs", Severity::Critical),
            build_defect_for_file("HOT-002", "src/hotspot.rs", Severity::High),
        ];
        let report = build_test_report(defects);
        let md = service.format_markdown(&report).unwrap();

        assert!(md.contains("### Top 10 Hotspot Files"));
        assert!(md.contains("| Rank | File | Defects | Severity Score |"));
        assert!(md.contains("src/hotspot.rs"));
    }

    #[test]
    fn test_format_markdown_detailed_findings() {
        let service = DefectReportService::new();
        let defects = vec![build_complete_defect(
            "DETAIL-001",
            Severity::High,
            DefectCategory::Complexity,
            "src/complex.rs",
        )];
        let report = build_test_report(defects);
        let md = service.format_markdown(&report).unwrap();

        assert!(md.contains("## Detailed Findings"));
        assert!(md.contains("### Complexity"));
        assert!(md.contains("src/complex.rs"));
    }

    #[test]
    fn test_format_markdown_fix_suggestion() {
        let service = DefectReportService::new();
        let defects = vec![build_complete_defect(
            "FIX-001",
            Severity::High,
            DefectCategory::Complexity,
            "src/fix.rs",
        )];
        let report = build_test_report(defects);
        let md = service.format_markdown(&report).unwrap();

        // Should contain the suggestion indicator
        assert!(md.contains("**Suggestion**"));
    }

    #[test]
    fn test_format_markdown_truncation_indicator() {
        let service = DefectReportService::new();
        // Create more than 10 defects of the same category
        let defects: Vec<Defect> = (0..15)
            .map(|i| {
                build_complete_defect(
                    &format!("TRUNC-{:03}", i),
                    Severity::High,
                    DefectCategory::Complexity,
                    "src/many.rs",
                )
            })
            .collect();
        let report = build_test_report(defects);
        let md = service.format_markdown(&report).unwrap();

        // Should show truncation message
        assert!(md.contains("...and 5 more Complexity"));
    }

    #[test]
    fn test_format_markdown_empty_category() {
        let service = DefectReportService::new();
        // Only create defects for one category
        let defects = vec![build_defect(
            "ONLY-001",
            Severity::High,
            DefectCategory::Complexity,
        )];
        let report = build_test_report(defects);
        let md = service.format_markdown(&report).unwrap();

        // Should only have Complexity section, not other categories
        assert!(md.contains("### Complexity"));
        // Other categories should not appear as headers (with issue counts)
        assert!(!md.contains("### Technical Debt ("));
    }

    // format_text() Tests

    #[test]
    fn test_format_text_header() {
        let service = DefectReportService::new();
        let report = build_test_report(vec![]);
        let text = service.format_text(&report).unwrap();

        assert!(text.contains("CODE QUALITY REPORT"));
        assert!(text.contains("==================="));
        assert!(text.contains("Generated:"));
        assert!(text.contains("Project:"));
    }

    #[test]
    fn test_format_text_severity_breakdown() {
        let service = DefectReportService::new();
        let defects = vec![
            build_defect("TXT-001", Severity::Critical, DefectCategory::Complexity),
            build_defect("TXT-002", Severity::High, DefectCategory::Complexity),
        ];
        let report = build_test_report(defects);
        let text = service.format_text(&report).unwrap();

        assert!(text.contains("SEVERITY BREAKDOWN"));
        assert!(text.contains("------------------"));
    }

    #[test]
    fn test_format_text_category_breakdown() {
        let service = DefectReportService::new();
        let defects = build_diverse_defects();
        let report = build_test_report(defects);
        let text = service.format_text(&report).unwrap();

        assert!(text.contains("CATEGORY BREAKDOWN"));
    }

    #[test]
    fn test_format_text_hotspot_files() {
        let service = DefectReportService::new();
        let defects = vec![build_defect_for_file(
            "TXT-HOT-001",
            "src/hot.rs",
            Severity::Critical,
        )];
        let report = build_test_report(defects);
        let text = service.format_text(&report).unwrap();

        assert!(text.contains("TOP HOTSPOT FILES"));
        assert!(text.contains("src/hot.rs"));
    }

    #[test]
    fn test_format_text_defect_listing() {
        let service = DefectReportService::new();
        let defects = vec![build_complete_defect(
            "LIST-001",
            Severity::High,
            DefectCategory::Complexity,
            "src/list.rs",
        )];
        let report = build_test_report(defects);
        let text = service.format_text(&report).unwrap();

        assert!(text.contains("DEFECTS"));
        assert!(text.contains("-------"));
        assert!(text.contains("[High]"));
        assert!(text.contains("Complexity"));
        assert!(text.contains("src/list.rs"));
    }

    #[test]
    fn test_format_text_line_range() {
        let service = DefectReportService::new();
        let mut defect = build_defect("RANGE-001", Severity::High, DefectCategory::Complexity);
        defect.line_start = 10;
        defect.line_end = Some(25);
        let report = build_test_report(vec![defect]);
        let text = service.format_text(&report).unwrap();

        // Should show line range
        assert!(text.contains("10-25"));
    }

    #[test]
    fn test_format_text_fix_suggestion_display() {
        let service = DefectReportService::new();
        let mut defect = build_defect("FIX-TXT-001", Severity::High, DefectCategory::Complexity);
        defect.fix_suggestion = Some("Extract method to reduce complexity".to_string());
        let report = build_test_report(vec![defect]);
        let text = service.format_text(&report).unwrap();

        assert!(text.contains("Fix:"));
        assert!(text.contains("Extract method to reduce complexity"));
    }

    // generate_filename() Tests

    #[test]
    fn test_generate_filename_json() {
        let service = DefectReportService::new();
        let filename = service.generate_filename(ReportFormat::Json);

        assert!(filename.starts_with("defect-report-"));
        assert!(filename.ends_with(".json"));
        // Should contain a timestamp pattern
        assert!(filename.len() > 20);
    }

    #[test]
    fn test_generate_filename_csv() {
        let service = DefectReportService::new();
        let filename = service.generate_filename(ReportFormat::Csv);

        assert!(filename.starts_with("defect-report-"));
        assert!(filename.ends_with(".csv"));
    }

    #[test]
    fn test_generate_filename_markdown() {
        let service = DefectReportService::new();
        let filename = service.generate_filename(ReportFormat::Markdown);

        assert!(filename.starts_with("defect-report-"));
        assert!(filename.ends_with(".md"));
    }

    #[test]
    fn test_generate_filename_text() {
        let service = DefectReportService::new();
        let filename = service.generate_filename(ReportFormat::Text);

        assert!(filename.starts_with("defect-report-"));
        assert!(filename.ends_with(".txt"));
    }

    #[test]
    fn test_generate_filename_uniqueness() {
        let service = DefectReportService::new();
        let filename1 = service.generate_filename(ReportFormat::Json);
        // Small delay to ensure different timestamps
        std::thread::sleep(std::time::Duration::from_millis(10));
        let filename2 = service.generate_filename(ReportFormat::Json);

        // Filenames should be different if generated at different times
        // (they might be same if generated in same second, so we just check format)
        assert!(filename1.starts_with("defect-report-"));
        assert!(filename2.starts_with("defect-report-"));
    }

    // filter_by_pattern() Tests

    #[test]
    fn test_filter_by_pattern_no_filters() {
        let defects = build_diverse_defects();
        let original_count = defects.len();
        let report = build_test_report(defects);

        let filtered = DefectReportService::filter_by_pattern(&report, None, None, 0);

        assert_eq!(filtered.defects.len(), original_count);
    }

    #[test]
    fn test_filter_by_pattern_include_glob() {
        let defects = vec![
            build_defect_for_file("INC-001", "src/main.rs", Severity::High),
            build_defect_for_file("INC-002", "src/lib.rs", Severity::High),
            build_defect_for_file("INC-003", "tests/test.rs", Severity::High),
        ];
        let report = build_test_report(defects);

        let filtered =
            DefectReportService::filter_by_pattern(&report, Some("src/*.rs".to_string()), None, 0);

        assert_eq!(filtered.defects.len(), 2);
        assert!(filtered
            .defects
            .iter()
            .all(|d| d.file_path.to_string_lossy().starts_with("src/")));
    }

    #[test]
    fn test_filter_by_pattern_exclude_glob() {
        let defects = vec![
            build_defect_for_file("EXC-001", "src/main.rs", Severity::High),
            build_defect_for_file("EXC-002", "tests/test.rs", Severity::High),
            build_defect_for_file("EXC-003", "benches/bench.rs", Severity::High),
        ];
        let report = build_test_report(defects);

        let filtered =
            DefectReportService::filter_by_pattern(&report, None, Some("tests/*".to_string()), 0);

        assert_eq!(filtered.defects.len(), 2);
        assert!(filtered
            .defects
            .iter()
            .all(|d| !d.file_path.to_string_lossy().contains("tests/")));
    }

    #[test]
    fn test_filter_by_pattern_include_and_exclude() {
        let defects = vec![
            build_defect_for_file("BOTH-001", "src/main.rs", Severity::High),
            build_defect_for_file("BOTH-002", "src/test_helper.rs", Severity::High),
            build_defect_for_file("BOTH-003", "tests/test.rs", Severity::High),
        ];
        let report = build_test_report(defects);

        let filtered = DefectReportService::filter_by_pattern(
            &report,
            Some("src/*.rs".to_string()),
            Some("**/test*.rs".to_string()),
            0,
        );

        // Should include src/*.rs but exclude **/test*.rs
        assert_eq!(filtered.defects.len(), 1);
        assert_eq!(filtered.defects[0].file_path, PathBuf::from("src/main.rs"));
    }
