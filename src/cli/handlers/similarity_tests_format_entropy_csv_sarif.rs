    // Tests for format_entropy_analysis_section

    #[test]
    fn test_format_entropy_analysis_section_none() {
        let entropy_analysis: Option<EntropyReport> = None;
        let mut output = String::new();
        let result = format_entropy_analysis_section(&mut output, &entropy_analysis);
        assert!(result.is_ok());
        assert!(output.is_empty());
    }

    #[test]
    fn test_format_entropy_analysis_section_some() {
        let entropy_analysis = Some(EntropyReport {
            average_entropy: 3.5,
            high_entropy_blocks: vec![create_test_entropy_block("high.rs", 4.8)],
            low_entropy_patterns: vec![create_test_entropy_block("low.rs", 1.2)],
            recommendations: vec!["Refactor complex code".to_string()],
        });
        let mut output = String::new();
        let result = format_entropy_analysis_section(&mut output, &entropy_analysis);
        assert!(result.is_ok());
        assert!(output.contains("Entropy Analysis"));
        assert!(output.contains("3.50"));
    }

    #[test]
    fn test_format_entropy_analysis_section_empty_blocks() {
        let entropy_analysis = Some(EntropyReport {
            average_entropy: 2.5,
            high_entropy_blocks: vec![],
            low_entropy_patterns: vec![],
            recommendations: vec![],
        });
        let mut output = String::new();
        let result = format_entropy_analysis_section(&mut output, &entropy_analysis);
        assert!(result.is_ok());
        assert!(output.contains("Entropy Analysis"));
        assert!(!output.contains("High Complexity"));
        assert!(!output.contains("Repetitive Patterns"));
    }

    // Tests for format_high_entropy_blocks

    #[test]
    fn test_format_high_entropy_blocks_empty() {
        let entropy = EntropyReport {
            average_entropy: 3.0,
            high_entropy_blocks: vec![],
            low_entropy_patterns: vec![],
            recommendations: vec![],
        };
        let mut output = String::new();
        let result = format_high_entropy_blocks(&mut output, &entropy);
        assert!(result.is_ok());
        assert!(output.is_empty());
    }

    #[test]
    fn test_format_high_entropy_blocks_nonempty() {
        let entropy = EntropyReport {
            average_entropy: 4.0,
            high_entropy_blocks: vec![
                create_test_entropy_block("complex1.rs", 4.5),
                create_test_entropy_block("complex2.rs", 4.8),
            ],
            low_entropy_patterns: vec![],
            recommendations: vec![],
        };
        let mut output = String::new();
        let result = format_high_entropy_blocks(&mut output, &entropy);
        assert!(result.is_ok());
        assert!(output.contains("High Complexity Code"));
        assert!(output.contains("complex1.rs"));
        assert!(output.contains("4.50"));
    }

    #[test]
    fn test_format_high_entropy_blocks_limits_to_five() {
        let entropy = EntropyReport {
            average_entropy: 4.0,
            high_entropy_blocks: (0..10)
                .map(|i| create_test_entropy_block(&format!("file{}.rs", i), 4.0 + i as f64 * 0.1))
                .collect(),
            low_entropy_patterns: vec![],
            recommendations: vec![],
        };
        let mut output = String::new();
        let result = format_high_entropy_blocks(&mut output, &entropy);
        assert!(result.is_ok());
        assert!(output.contains("file0.rs"));
        assert!(output.contains("file4.rs"));
        assert!(!output.contains("file5.rs"));
    }

    // Tests for format_low_entropy_patterns

    #[test]
    fn test_format_low_entropy_patterns_empty() {
        let entropy = EntropyReport {
            average_entropy: 3.0,
            high_entropy_blocks: vec![],
            low_entropy_patterns: vec![],
            recommendations: vec![],
        };
        let mut output = String::new();
        let result = format_low_entropy_patterns(&mut output, &entropy);
        assert!(result.is_ok());
        assert!(output.is_empty());
    }

    #[test]
    fn test_format_low_entropy_patterns_nonempty() {
        let entropy = EntropyReport {
            average_entropy: 2.0,
            high_entropy_blocks: vec![],
            low_entropy_patterns: vec![
                create_test_entropy_block("repetitive1.rs", 1.2),
                create_test_entropy_block("repetitive2.rs", 1.5),
            ],
            recommendations: vec![],
        };
        let mut output = String::new();
        let result = format_low_entropy_patterns(&mut output, &entropy);
        assert!(result.is_ok());
        assert!(output.contains("Repetitive Patterns"));
        assert!(output.contains("repetitive1.rs"));
    }

    // Tests for format_entropy_block_item

    #[test]
    fn test_format_entropy_block_item() {
        let block = create_test_entropy_block("entropy_test.rs", 3.75);
        let mut output = String::new();
        let result = format_entropy_block_item(&mut output, &block);
        assert!(result.is_ok());
        assert!(output.contains("entropy_test.rs:1"));
        assert!(output.contains("3.75"));
        assert!(output.contains("Suggestion:"));
    }

    // Tests for format_refactoring_opportunities_section

    #[test]
    fn test_format_refactoring_opportunities_section_empty() {
        let opportunities: Vec<RefactoringHint> = vec![];
        let mut output = String::new();
        let result = format_refactoring_opportunities_section(&mut output, &opportunities);
        assert!(result.is_ok());
        assert!(output.is_empty());
    }

    #[test]
    fn test_format_refactoring_opportunities_section_nonempty() {
        let opportunities = vec![
            create_test_refactoring_hint("Extract method", Priority::High),
            create_test_refactoring_hint("Consolidate", Priority::Low),
        ];
        let mut output = String::new();
        let result = format_refactoring_opportunities_section(&mut output, &opportunities);
        assert!(result.is_ok());
        assert!(output.contains("Refactoring Opportunities"));
        assert!(output.contains("Extract method"));
        assert!(output.contains("High"));
    }

    // Tests for format_single_refactoring_hint

    #[test]
    fn test_format_single_refactoring_hint() {
        let hint = create_test_refactoring_hint("DRY violation", Priority::Medium);
        let mut output = String::new();
        let result = format_single_refactoring_hint(&mut output, &hint);
        assert!(result.is_ok());
        assert!(output.contains("### DRY violation"));
        assert!(output.contains("Priority: Medium"));
        assert!(output.contains("Suggestion:"));
        assert!(output.contains("test1.rs:1-10"));
        assert!(output.contains("test2.rs:5-15"));
    }

    // Tests for print_performance_metrics

    #[test]
    fn test_print_performance_metrics() {
        let report = create_populated_report();
        let elapsed = std::time::Duration::from_millis(500);
        // This prints to stderr, so we just ensure it doesn't panic
        print_performance_metrics(&report, elapsed);
    }

    #[test]
    fn test_print_performance_metrics_empty_report() {
        let report = create_empty_report();
        let elapsed = std::time::Duration::from_millis(100);
        print_performance_metrics(&report, elapsed);
    }

    // Tests for format_csv_report

    #[test]
    fn test_format_csv_report_empty() {
        let report = create_empty_report();
        let result = format_csv_report(&report);
        assert!(result.is_ok());
        let output = result.unwrap();
        // Should contain header only
        assert!(output.starts_with("Type,File1,Start1,End1,File2,Start2,End2,Similarity"));
        // Should have exactly one line (header)
        assert_eq!(output.lines().count(), 1);
    }

    #[test]
    fn test_format_csv_report_with_duplicates() {
        let report = create_populated_report();
        let result = format_csv_report(&report);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Exact,"));
        assert!(output.contains(",100.0"));
        assert!(output.contains("Structural,"));
    }

    #[test]
    fn test_format_csv_report_single_location_blocks_skipped() {
        let mut report = create_empty_report();
        report.exact_duplicates.push(SimilarBlock {
            id: "single".to_string(),
            locations: vec![create_test_location("only.rs", 1, 10)],
            similarity: 1.0,
            clone_type: CloneType::Type1,
            lines: 10,
            tokens: 50,
            content_preview: "test".to_string(),
        });
        let result = format_csv_report(&report);
        assert!(result.is_ok());
        let output = result.unwrap();
        // Should only have header, single-location blocks are skipped
        assert_eq!(output.lines().count(), 1);
    }

    // Tests for format_sarif_report

    #[test]
    fn test_format_sarif_report_empty() {
        let report = create_empty_report();
        let result = format_sarif_report(&report);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("\"$schema\""));
        assert!(output.contains("\"version\": \"2.1.0\""));
        assert!(output.contains("\"results\": []"));
    }

    #[test]
    fn test_format_sarif_report_with_duplicates() {
        let report = create_populated_report();
        let result = format_sarif_report(&report);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("duplicate-code"));
        assert!(output.contains("warning"));
        assert!(output.contains("physicalLocation"));
        assert!(output.contains("startLine"));
    }

    #[test]
    fn test_format_sarif_report_contains_tool_info() {
        let report = create_empty_report();
        let result = format_sarif_report(&report);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("pmat-similarity"));
        assert!(output.contains("paiml-mcp-agent-toolkit"));
    }

    // Tests for print_summary

    #[test]
    fn test_print_summary() {
        let report = create_populated_report();
        // This prints to stderr, just verify it doesn't panic
        print_summary(&report);
    }

    #[test]
    fn test_print_summary_empty() {
        let report = create_empty_report();
        print_summary(&report);
    }

    #[test]
    fn test_print_summary_no_refactoring() {
        let mut report = create_populated_report();
        report.refactoring_opportunities.clear();
        print_summary(&report);
    }

    // Additional entropy/refactoring/csv/sarif edge case tests

    #[test]
    fn test_low_entropy_patterns_limits_to_five() {
        let entropy = EntropyReport {
            average_entropy: 1.5,
            high_entropy_blocks: vec![],
            low_entropy_patterns: (0..10)
                .map(|i| create_test_entropy_block(&format!("rep{}.rs", i), 1.0 + i as f64 * 0.1))
                .collect(),
            recommendations: vec![],
        };
        let mut output = String::new();
        let result = format_low_entropy_patterns(&mut output, &entropy);
        assert!(result.is_ok());
        assert!(output.contains("rep0.rs"));
        assert!(output.contains("rep4.rs"));
        assert!(!output.contains("rep5.rs"));
    }

    #[test]
    fn test_format_csv_escaping() {
        // Test that CSV formatting handles special characters
        let mut report = create_empty_report();
        report.exact_duplicates.push(SimilarBlock {
            id: "test".to_string(),
            locations: vec![
                create_test_location("file,with,commas.rs", 1, 10),
                create_test_location("normal.rs", 5, 15),
            ],
            similarity: 1.0,
            clone_type: CloneType::Type1,
            lines: 10,
            tokens: 50,
            content_preview: "test".to_string(),
        });
        let result = format_csv_report(&report);
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_csv_with_multiple_blocks() {
        let mut report = create_empty_report();
        for i in 0..5 {
            report.exact_duplicates.push(SimilarBlock {
                id: format!("block{}", i),
                locations: vec![
                    create_test_location(&format!("file{}.rs", i), i * 10, i * 10 + 5),
                    create_test_location(&format!("file{}_dup.rs", i), i * 20, i * 20 + 5),
                ],
                similarity: 1.0,
                clone_type: CloneType::Type1,
                lines: 5,
                tokens: 25,
                content_preview: format!("preview {}", i),
            });
        }

        let result = format_csv_report(&report);
        assert!(result.is_ok());
        let output = result.unwrap();

        // Count lines (header + 5 data rows)
        assert_eq!(output.lines().count(), 6);
    }

    #[test]
    fn test_format_sarif_with_many_locations() {
        let mut report = create_empty_report();
        let locations: Vec<_> = (0..100)
            .map(|i| create_test_location(&format!("file{}.rs", i), i, i + 10))
            .collect();
        report.exact_duplicates.push(SimilarBlock {
            id: "many_locs".to_string(),
            locations,
            similarity: 1.0,
            clone_type: CloneType::Type1,
            lines: 10,
            tokens: 50,
            content_preview: "test".to_string(),
        });
        let result = format_sarif_report(&report);
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_sarif_structure() {
        let report = create_populated_report();
        let result = format_sarif_report(&report);
        assert!(result.is_ok());
        let output = result.unwrap();

        // Parse as JSON to verify structure
        let sarif: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(sarif.get("$schema").is_some());
        assert!(sarif.get("version").is_some());
        assert!(sarif.get("runs").is_some());
    }

    #[test]
    fn test_metrics_with_extreme_values() {
        let metrics = create_test_metrics(100.0, 10.0, 1_000_000);
        let mut output = String::new();
        let result = format_summary_metrics(&mut output, &metrics);
        assert!(result.is_ok());
        assert!(output.contains("100.0%"));
        assert!(output.contains("10.00"));
        assert!(output.contains("1000000"));
    }

    #[test]
    fn test_metrics_with_zero_values() {
        let metrics = create_test_metrics(0.0, 0.0, 0);
        let mut output = String::new();
        let result = format_summary_metrics(&mut output, &metrics);
        assert!(result.is_ok());
        assert!(output.contains("0.0%"));
        assert!(output.contains("0.00"));
        assert!(output.contains(": 0"));
    }

    #[test]
    fn test_all_priority_levels() {
        let mut output = String::new();
        for priority in [Priority::High, Priority::Medium, Priority::Low] {
            let hint = create_test_refactoring_hint(&format!("{:?}_pattern", priority), priority);
            format_single_refactoring_hint(&mut output, &hint).unwrap();
        }
        assert!(output.contains("High"));
        assert!(output.contains("Medium"));
        assert!(output.contains("Low"));
    }

    #[test]
    fn test_format_refactoring_hint_empty_locations() {
        let hint = RefactoringHint {
            locations: vec![],
            pattern: "No locations".to_string(),
            suggestion: "N/A".to_string(),
            priority: Priority::Low,
        };
        let mut output = String::new();
        let result = format_single_refactoring_hint(&mut output, &hint);
        assert!(result.is_ok());
        assert!(output.contains("No locations"));
    }
