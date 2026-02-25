    // Tests for format_report

    #[test]
    fn test_format_report_json() {
        let report = create_populated_report();
        let result = format_report(&report, DuplicateOutputFormat::Json);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("exact_duplicates"));
        assert!(output.contains("metrics"));
    }

    #[test]
    fn test_format_report_summary() {
        let report = create_populated_report();
        let result = format_report(&report, DuplicateOutputFormat::Summary);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Code Similarity Analysis Summary"));
        assert!(output.contains("Metrics"));
    }

    #[test]
    fn test_format_report_human() {
        let report = create_populated_report();
        let result = format_report(&report, DuplicateOutputFormat::Human);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Metrics"));
    }

    #[test]
    fn test_format_report_detailed() {
        let report = create_populated_report();
        let result = format_report(&report, DuplicateOutputFormat::Detailed);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Comprehensive Code Similarity Report"));
    }

    #[test]
    fn test_format_report_csv() {
        let report = create_populated_report();
        let result = format_report(&report, DuplicateOutputFormat::Csv);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Type,File1,Start1,End1,File2,Start2,End2,Similarity"));
    }

    #[test]
    fn test_format_report_sarif() {
        let report = create_populated_report();
        let result = format_report(&report, DuplicateOutputFormat::Sarif);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("$schema"));
        assert!(output.contains("pmat-similarity"));
    }

    // Tests for format_summary_report

    #[test]
    fn test_format_summary_report_empty() {
        let report = create_empty_report();
        let result = format_summary_report(&report);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("# Code Similarity Analysis Summary"));
        assert!(output.contains("Duplication: 0.0%"));
        assert!(output.contains("Total Clones: 0"));
    }

    #[test]
    fn test_format_summary_report_with_refactoring() {
        let report = create_populated_report();
        let result = format_summary_report(&report);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Top Refactoring Opportunities"));
    }

    // Tests for format_summary_metrics

    #[test]
    fn test_format_summary_metrics() {
        let metrics = create_test_metrics(12.5, 3.2, 42);
        let mut output = String::new();
        let result = format_summary_metrics(&mut output, &metrics);
        assert!(result.is_ok());
        assert!(output.contains("12.5%"));
        assert!(output.contains("3.20"));
        assert!(output.contains("42"));
    }

    // Tests for format_summary_clone_types

    #[test]
    fn test_format_summary_clone_types() {
        let report = create_populated_report();
        let mut output = String::new();
        let result = format_summary_clone_types(&mut output, &report);
        assert!(result.is_ok());
        assert!(output.contains("Clone Types"));
        assert!(output.contains("Exact Duplicates: 2"));
        assert!(output.contains("Structural Similarities: 1"));
        assert!(output.contains("Semantic Similarities: 1"));
    }

    // Tests for format_summary_refactoring_opportunities

    #[test]
    fn test_format_summary_refactoring_opportunities_empty() {
        let opportunities: Vec<RefactoringHint> = vec![];
        let mut output = String::new();
        let result = format_summary_refactoring_opportunities(&mut output, &opportunities);
        assert!(result.is_ok());
        assert!(!output.contains("Refactoring Opportunities"));
    }

    #[test]
    fn test_format_summary_refactoring_opportunities_nonempty() {
        let opportunities = vec![
            create_test_refactoring_hint("Pattern1", Priority::High),
            create_test_refactoring_hint("Pattern2", Priority::Medium),
        ];
        let mut output = String::new();
        let result = format_summary_refactoring_opportunities(&mut output, &opportunities);
        assert!(result.is_ok());
        assert!(output.contains("Top Refactoring Opportunities"));
        assert!(output.contains("Pattern1"));
        assert!(output.contains("Pattern2"));
    }

    #[test]
    fn test_format_summary_refactoring_opportunities_more_than_five() {
        let opportunities: Vec<_> = (0..10)
            .map(|i| create_test_refactoring_hint(&format!("Pattern{}", i), Priority::High))
            .collect();
        let mut output = String::new();
        let result = format_summary_refactoring_opportunities(&mut output, &opportunities);
        assert!(result.is_ok());
        // Should only show first 5
        assert!(output.contains("Pattern0"));
        assert!(output.contains("Pattern4"));
        assert!(!output.contains("Pattern5"));
    }

    // Tests for format_detailed_report

    #[test]
    fn test_format_detailed_report_empty() {
        let report = create_empty_report();
        let result = format_detailed_report(&report);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Comprehensive Code Similarity Report"));
        assert!(output.contains("Overall Metrics"));
    }

    #[test]
    fn test_format_detailed_report_populated() {
        let report = create_populated_report();
        let result = format_detailed_report(&report);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Exact Duplicates"));
        assert!(output.contains("Structural Similarities"));
        assert!(output.contains("Entropy Analysis"));
        assert!(output.contains("Refactoring Opportunities"));
    }

    // Tests for format_metrics_section

    #[test]
    fn test_format_metrics_section() {
        let metrics = create_test_metrics(25.0, 4.5, 100);
        let mut output = String::new();
        let result = format_metrics_section(&mut output, &metrics);
        assert!(result.is_ok());
        assert!(output.contains("Overall Metrics"));
        assert!(output.contains("25.0%"));
        assert!(output.contains("4.50"));
        assert!(output.contains("100"));
    }

    // Tests for format_exact_duplicates_section

    #[test]
    fn test_format_exact_duplicates_section_empty() {
        let duplicates: Vec<SimilarBlock> = vec![];
        let mut output = String::new();
        let result = format_exact_duplicates_section(&mut output, &duplicates);
        assert!(result.is_ok());
        assert!(output.is_empty());
    }

    #[test]
    fn test_format_exact_duplicates_section_nonempty() {
        let duplicates = vec![create_test_similar_block(
            "test_block",
            vec![
                create_test_location("file1.rs", 1, 10),
                create_test_location("file2.rs", 20, 30),
            ],
        )];
        let mut output = String::new();
        let result = format_exact_duplicates_section(&mut output, &duplicates);
        assert!(result.is_ok());
        assert!(output.contains("Exact Duplicates"));
        assert!(output.contains("test_block"));
    }

    // Tests for format_single_duplicate_block

    #[test]
    fn test_format_single_duplicate_block() {
        let block = create_test_similar_block(
            "dup123",
            vec![
                create_test_location("src/a.rs", 5, 15),
                create_test_location("src/b.rs", 25, 35),
            ],
        );
        let mut output = String::new();
        let result = format_single_duplicate_block(&mut output, &block);
        assert!(result.is_ok());
        assert!(output.contains("Block dup123"));
        assert!(output.contains("Lines: 10"));
        assert!(output.contains("Tokens: 50"));
        assert!(output.contains("src/a.rs:5-15"));
        assert!(output.contains("src/b.rs:25-35"));
        assert!(output.contains("Preview:"));
    }

    // Tests for format_structural_similarities_section

    #[test]
    fn test_format_structural_similarities_section_empty() {
        let similarities: Vec<SimilarBlock> = vec![];
        let mut output = String::new();
        let result = format_structural_similarities_section(&mut output, &similarities);
        assert!(result.is_ok());
        assert!(output.is_empty());
    }

    #[test]
    fn test_format_structural_similarities_section_nonempty() {
        let similarities = vec![SimilarBlock {
            id: "struct_sim".to_string(),
            locations: vec![
                create_test_location("a.rs", 1, 10),
                create_test_location("b.rs", 5, 15),
            ],
            similarity: 0.9,
            clone_type: CloneType::Type2,
            lines: 10,
            tokens: 50,
            content_preview: "fn test()".to_string(),
        }];
        let mut output = String::new();
        let result = format_structural_similarities_section(&mut output, &similarities);
        assert!(result.is_ok());
        assert!(output.contains("Structural Similarities"));
    }

    #[test]
    fn test_format_structural_similarities_section_limits_to_ten() {
        let similarities: Vec<_> = (0..20)
            .map(|i| SimilarBlock {
                id: format!("sim{}", i),
                locations: vec![create_test_location("a.rs", i, i + 10)],
                similarity: 0.8,
                clone_type: CloneType::Type2,
                lines: 10,
                tokens: 50,
                content_preview: "test".to_string(),
            })
            .collect();
        let mut output = String::new();
        let result = format_structural_similarities_section(&mut output, &similarities);
        assert!(result.is_ok());
        // Should only contain first 10
        assert!(output.contains("sim0"));
        assert!(output.contains("sim9"));
        assert!(!output.contains("sim10"));
    }

    // Tests for format_single_structural_block

    #[test]
    fn test_format_single_structural_block() {
        let block = SimilarBlock {
            id: "struct_block".to_string(),
            locations: vec![
                create_test_location("x.rs", 100, 120),
                create_test_location("y.rs", 200, 220),
            ],
            similarity: 0.875,
            clone_type: CloneType::Type3,
            lines: 20,
            tokens: 100,
            content_preview: "let x = 1;".to_string(),
        };
        let mut output = String::new();
        let result = format_single_structural_block(&mut output, &block);
        assert!(result.is_ok());
        assert!(output.contains("Similarity struct_block"));
        assert!(output.contains("87.5%"));
        assert!(output.contains("Type3"));
        assert!(output.contains("x.rs:100-120"));
        assert!(output.contains("y.rs:200-220"));
    }

    // Additional format report tests

    #[test]
    fn test_format_detailed_report_with_all_sections() {
        let report = create_populated_report();
        let result = format_detailed_report(&report);
        assert!(result.is_ok());
        let output = result.unwrap();

        // Verify all sections are present
        assert!(output.contains("Overall Metrics"));
        assert!(output.contains("Exact Duplicates"));
        assert!(output.contains("Structural Similarities"));
        assert!(output.contains("Entropy Analysis"));
        assert!(output.contains("Refactoring Opportunities"));
    }

    #[test]
    fn test_all_clone_types_in_detailed_report() {
        let mut report = create_empty_report();
        for clone_type in [
            CloneType::Type1,
            CloneType::Type2,
            CloneType::Type3,
            CloneType::Type4,
        ] {
            report.structural_similarities.push(SimilarBlock {
                id: format!("{:?}", clone_type),
                locations: vec![
                    create_test_location("a.rs", 1, 10),
                    create_test_location("b.rs", 5, 15),
                ],
                similarity: 0.9,
                clone_type,
                lines: 10,
                tokens: 50,
                content_preview: "test".to_string(),
            });
        }
        let result = format_detailed_report(&report);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Type1"));
        assert!(output.contains("Type2"));
        assert!(output.contains("Type3"));
        assert!(output.contains("Type4"));
    }

    #[test]
    fn test_format_report_handles_empty_content_preview() {
        let block = SimilarBlock {
            id: "empty_preview".to_string(),
            locations: vec![
                create_test_location("a.rs", 1, 10),
                create_test_location("b.rs", 5, 15),
            ],
            similarity: 0.95,
            clone_type: CloneType::Type1,
            lines: 10,
            tokens: 50,
            content_preview: "".to_string(),
        };
        let mut output = String::new();
        let result = format_single_duplicate_block(&mut output, &block);
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_structural_block_with_zero_similarity() {
        let block = SimilarBlock {
            id: "zero_sim".to_string(),
            locations: vec![create_test_location("a.rs", 1, 10)],
            similarity: 0.0,
            clone_type: CloneType::Type2,
            lines: 10,
            tokens: 50,
            content_preview: "test".to_string(),
        };
        let mut output = String::new();
        let result = format_single_structural_block(&mut output, &block);
        assert!(result.is_ok());
        assert!(output.contains("0.0%"));
    }

    #[test]
    fn test_report_json_roundtrip() {
        let report = create_populated_report();
        let json = format_report(&report, DuplicateOutputFormat::Json).unwrap();
        let parsed: ComprehensiveReport = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.metrics.total_clones, report.metrics.total_clones);
    }

    #[test]
    fn test_location_with_columns() {
        let location = Location {
            file: PathBuf::from("test.rs"),
            start_line: 10,
            end_line: 20,
            start_column: Some(5),
            end_column: Some(50),
        };
        let block = SimilarBlock {
            id: "col_test".to_string(),
            locations: vec![location],
            similarity: 1.0,
            clone_type: CloneType::Type1,
            lines: 10,
            tokens: 50,
            content_preview: "test".to_string(),
        };
        let mut output = String::new();
        let result = format_single_duplicate_block(&mut output, &block);
        assert!(result.is_ok());
    }
