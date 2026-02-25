    #[test]
    fn test_generate_metrics() {
        let detector = SATDDetector::new();
        let debts = vec![
            TechnicalDebt {
                category: DebtCategory::Security,
                severity: Severity::Critical,
                text: "Security issue".to_string(),
                file: PathBuf::from("file1.rs"),
                line: 10,
                column: 5,
                context_hash: [1; 16],
            },
            TechnicalDebt {
                category: DebtCategory::Design,
                severity: Severity::Medium,
                text: "Design issue".to_string(),
                file: PathBuf::from("file1.rs"),
                line: 20,
                column: 5,
                context_hash: [2; 16],
            },
            TechnicalDebt {
                category: DebtCategory::Design,
                severity: Severity::Low,
                text: "Another design issue".to_string(),
                file: PathBuf::from("file2.rs"),
                line: 30,
                column: 5,
                context_hash: [3; 16],
            },
        ];

        let metrics = detector.generate_metrics(&debts, 1000);

        assert_eq!(metrics.total_debts, 3);
        assert_eq!(metrics.debt_density_per_kloc, 3.0);
        assert_eq!(metrics.critical_debts.len(), 1);
        assert_eq!(metrics.by_category.len(), 2);

        let design_metrics = metrics.by_category.get("Design").expect("internal error");
        assert_eq!(design_metrics.count, 2);
        assert_eq!(design_metrics.files.len(), 2);

        // Test with zero LOC
        let metrics_zero = detector.generate_metrics(&debts, 0);
        assert_eq!(metrics_zero.debt_density_per_kloc, 0.0);
    }

    #[test]
    fn test_debt_category_variants() {
        let design = DebtCategory::Design;
        let defect = DebtCategory::Defect;
        let performance = DebtCategory::Performance;
        let requirement = DebtCategory::Requirement;
        let test_debt = DebtCategory::Test;
        let security = DebtCategory::Security;

        assert_eq!(design, DebtCategory::Design);
        assert_eq!(defect, DebtCategory::Defect);
        assert_eq!(performance, DebtCategory::Performance);
        assert_eq!(requirement, DebtCategory::Requirement);
        assert_eq!(test_debt, DebtCategory::Test);
        assert_eq!(security, DebtCategory::Security);
    }

    #[test]
    fn test_severity_variants() {
        let low = Severity::Low;
        let medium = Severity::Medium;
        let high = Severity::High;
        let critical = Severity::Critical;

        assert_eq!(low, Severity::Low);
        assert_eq!(medium, Severity::Medium);
        assert_eq!(high, Severity::High);
        assert_eq!(critical, Severity::Critical);
    }

    #[test]
    fn test_technical_debt_creation() {
        let debt = TechnicalDebt {
            category: DebtCategory::Design,
            severity: Severity::High,
            text: "Refactor this complex function".to_string(),
            file: PathBuf::from("src/complex.rs"),
            line: 100,
            column: 5,
            context_hash: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
        };

        assert_eq!(debt.category, DebtCategory::Design);
        assert_eq!(debt.severity, Severity::High);
        assert_eq!(debt.text, "Refactor this complex function");
        assert_eq!(debt.file, PathBuf::from("src/complex.rs"));
        assert_eq!(debt.line, 100);
        assert_eq!(debt.column, 5);
        assert_eq!(debt.context_hash.len(), 16);
    }

    #[test]
    fn test_debt_file_metrics_creation() {
        let file_metrics = DebtFileMetrics {
            file: PathBuf::from("test.rs"),
            count: 5,
            critical_count: 2,
            categories: vec!["Design".to_string(), "Defect".to_string()],
            lines: vec![10, 20, 30, 40, 50],
        };

        assert_eq!(file_metrics.file, PathBuf::from("test.rs"));
        assert_eq!(file_metrics.count, 5);
        assert_eq!(file_metrics.critical_count, 2);
        assert_eq!(file_metrics.categories.len(), 2);
        assert_eq!(file_metrics.lines.len(), 5);
    }

    #[test]
    fn test_debt_category_metrics_creation() {
        let category_metrics = DebtCategoryMetrics {
            count: 10,
            critical_count: 3,
            files: vec![PathBuf::from("file1.rs"), PathBuf::from("file2.rs")],
        };

        assert_eq!(category_metrics.count, 10);
        assert_eq!(category_metrics.critical_count, 3);
        assert_eq!(category_metrics.files.len(), 2);
    }

    #[test]
    fn test_satd_metrics_creation() {
        use std::collections::{BTreeMap, BTreeSet};

        let mut by_category = BTreeMap::new();
        let mut files = BTreeSet::new();
        files.insert("design.rs".to_string());

        by_category.insert(
            "Design".to_string(),
            CategoryMetrics {
                count: 5,
                files,
                avg_severity: 2.5,
            },
        );

        let metrics = SATDMetrics {
            total_debts: 15,
            critical_debts: vec![create_test_debt(DebtCategory::Defect, Severity::Critical)],
            debt_density_per_kloc: 7.5,
            by_category,
            debt_age_distribution: vec![1.0, 2.0, 3.0],
        };

        assert_eq!(metrics.total_debts, 15);
        assert_eq!(metrics.critical_debts.len(), 1);
        assert_eq!(metrics.debt_density_per_kloc, 7.5);
        assert_eq!(metrics.by_category.len(), 1);
        assert_eq!(metrics.debt_age_distribution.len(), 3);
    }

    #[test]
    fn test_satd_detector_creation() {
        let detector = SATDDetector::new();

        // Should be created successfully
        assert!(std::mem::size_of_val(&detector) > 0);
    }

    #[test]
    fn test_extract_from_content_empty_string() {
        let detector = SATDDetector::new();
        let empty_content = "";

        let result = detector.extract_from_content(empty_content, Path::new("empty.rs"));
        assert!(result.is_ok());
        assert_eq!(result.expect("internal error").len(), 0);
    }

    #[test]
    fn test_extract_from_content_no_debt() {
        let detector = SATDDetector::new();
        let clean_content = r#"
        fn main() {
            println!("Hello, world!");
        }
        
        struct MyStruct {
            field: i32,
        }
        "#;

        let result = detector.extract_from_content(clean_content, Path::new("clean.rs"));
        assert!(result.is_ok());
        assert_eq!(result.expect("internal error").len(), 0);
    }

    #[test]
    fn test_extract_from_content_single_todo() {
        let detector = SATDDetector::new();
        let content_with_todo = r#"
        fn main() {
            // TODO: Implement error handling
            println!("Hello, world!");
        }
        "#;

        let result = detector.extract_from_content(content_with_todo, Path::new("todo.rs"));
        assert!(result.is_ok());
        let debts = result.expect("internal error");
        assert_eq!(debts.len(), 1);
        assert_eq!(debts[0].category, DebtCategory::Requirement);
        assert!(debts[0].text.contains("Implement error handling"));
    }

    #[test]
    fn test_extract_from_content_multiple_debt_types() {
        let detector = SATDDetector::new();
        let mixed_content = r#"
        fn main() {
            // TODO: Add proper error handling
            // FIXME: This algorithm is inefficient
            // HACK: Temporary workaround for issue #123
            // XXX: This code is problematic
            println!("Hello, world!");
        }
        "#;

        let result = detector.extract_from_content(mixed_content, Path::new("mixed.rs"));
        assert!(result.is_ok());
        let debts = result.expect("internal error");
        assert_eq!(debts.len(), 4);

        // Check different debt types are detected
        let debt_texts: Vec<&str> = debts.iter().map(|d| d.text.as_str()).collect();
        assert!(debt_texts
            .iter()
            .any(|&text| text.contains("error handling")));
        assert!(debt_texts.iter().any(|&text| text.contains("inefficient")));
        assert!(debt_texts.iter().any(|&text| text.contains("workaround")));
        assert!(debt_texts.iter().any(|&text| text.contains("problematic")));
    }

    #[test]
    fn test_extract_from_content_case_insensitive() {
        let detector = SATDDetector::new();
        let case_content = r#"
        fn test() {
            // todo: lowercase todo
            // Todo: Capitalized todo
            // TODO: All caps todo
            // tOdO: Mixed case todo
        }
        "#;

        let result = detector.extract_from_content(case_content, Path::new("case.rs"));
        assert!(result.is_ok());
        let debts = result.expect("internal error");
        assert_eq!(debts.len(), 4); // All variations should be detected
    }
