
    #[test]
    fn test_calculate_cyclomatic_complexity_simple() {
        let content = "fn main() {}";
        let complexity = calculate_cyclomatic_complexity(content);
        assert_eq!(complexity, 1); // Base complexity is 1
    }

    #[test]
    fn test_calculate_cyclomatic_complexity_with_if() {
        let content = "fn main() { if true {} }";
        let complexity = calculate_cyclomatic_complexity(content);
        assert!(complexity >= 2); // Base + if
    }

    #[test]
    fn test_calculate_cyclomatic_complexity_with_loops() {
        let content = "fn main() { for i in 0..10 {} while true {} }";
        let complexity = calculate_cyclomatic_complexity(content);
        assert!(complexity >= 3); // Base + for + while
    }

    #[test]
    fn test_calculate_cognitive_complexity() {
        // Cognitive is 1.5x cyclomatic
        assert_eq!(calculate_cognitive_complexity(10), 15);
        assert_eq!(calculate_cognitive_complexity(4), 6);
        assert_eq!(calculate_cognitive_complexity(1), 1);
    }

    #[test]
    fn test_calculate_duplicate_ratio_no_duplicates() {
        let lines = vec!["line1", "line2", "line3"];
        let ratio = calculate_duplicate_ratio(&lines);
        assert_eq!(ratio, 0.0);
    }

    #[test]
    fn test_calculate_duplicate_ratio_with_duplicates() {
        let lines = vec!["line1", "line1", "line2"];
        let ratio = calculate_duplicate_ratio(&lines);
        assert!(ratio > 0.0);
    }

    #[test]
    fn test_calculate_duplicate_ratio_empty() {
        let lines: Vec<&str> = vec![];
        let ratio = calculate_duplicate_ratio(&lines);
        assert_eq!(ratio, 0.0);
    }

    #[test]
    fn test_calculate_duplicate_ratio_skips_comments() {
        let lines = vec!["// comment", "// comment", "code"];
        let ratio = calculate_duplicate_ratio(&lines);
        // Comments should be skipped
        assert_eq!(ratio, 0.0);
    }

    #[test]
    fn test_calculate_efferent_coupling() {
        let content = "use std::io;\nuse std::path::Path;\nfn main() {}";
        let coupling = calculate_efferent_coupling(content);
        assert_eq!(coupling, 2.0);
    }

    #[test]
    fn test_calculate_efferent_coupling_none() {
        let content = "fn main() {}";
        let coupling = calculate_efferent_coupling(content);
        assert_eq!(coupling, 0.0);
    }

    #[test]
    fn test_calculate_afferent_coupling() {
        let content = "pub fn foo() {}\npub struct Bar {}\nfn private() {}";
        let coupling = calculate_afferent_coupling(content);
        assert_eq!(coupling, 2.0); // pub fn + pub struct
    }

    #[test]
    fn test_calculate_afferent_coupling_none() {
        let content = "fn foo() {}\nstruct Bar {}";
        let coupling = calculate_afferent_coupling(content);
        assert_eq!(coupling, 0.0);
    }

    #[test]
    fn test_is_public_declaration() {
        assert!(is_public_declaration("pub fn foo() {}"));
        assert!(is_public_declaration("pub struct Bar {}"));
        assert!(is_public_declaration("pub enum Baz {}"));
        assert!(is_public_declaration("pub trait Qux {}"));
        assert!(is_public_declaration("pub mod module;"));
        assert!(!is_public_declaration("fn foo() {}"));
        assert!(!is_public_declaration("struct Bar {}"));
    }

    #[test]
    fn test_get_churn_score_found() {
        let mut map = HashMap::new();
        map.insert("src/main.rs".to_string(), 0.75);
        let score = get_churn_score("src/main.rs", &map);
        assert_eq!(score, 0.75);
    }

    #[test]
    fn test_get_churn_score_not_found() {
        let map = HashMap::new();
        let score = get_churn_score("src/main.rs", &map);
        assert_eq!(score, 0.1); // Default
    }

    #[test]
    fn test_get_relative_path() {
        let path = Path::new("/project/src/main.rs");
        let project_path = Path::new("/project");
        let relative = get_relative_path(path, project_path);
        assert_eq!(relative, "src/main.rs");
    }

    #[test]
    fn test_get_relative_path_no_prefix() {
        let path = Path::new("/other/src/main.rs");
        let project_path = Path::new("/project");
        let relative = get_relative_path(path, project_path);
        // Should return the full path when not a prefix
        assert!(relative.contains("main.rs"));
    }

    // Tests for calculate_percentage()

    #[test]
    fn test_calculate_percentage_normal() {
        assert!((calculate_percentage(50, 100) - 50.0).abs() < f64::EPSILON);
        assert!((calculate_percentage(25, 100) - 25.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_percentage_zero_total() {
        assert_eq!(calculate_percentage(10, 0), 0.0);
    }

    #[test]
    fn test_calculate_percentage_all() {
        assert!((calculate_percentage(100, 100) - 100.0).abs() < f64::EPSILON);
    }

    // Tests for default_* functions

    #[test]
    fn test_default_project_path() {
        assert_eq!(default_project_path(), ".");
    }

    #[test]
    fn test_default_top_files() {
        assert_eq!(default_top_files(), 10);
    }

    #[test]
    fn test_default_min_violations() {
        assert_eq!(default_min_violations(), 1);
    }

    #[test]
    fn test_default_table_format() {
        assert_eq!(default_table_format(), "table");
    }

    #[test]
    fn test_default_true() {
        assert!(default_true());
    }

    #[test]
    fn test_default_summary_format() {
        assert_eq!(default_summary_format(), "summary");
    }

    // Tests for TDG formatting functions

    fn create_test_tdg_summary() -> TDGSummary {
        TDGSummary {
            total_files: 100,
            critical_files: 5,
            warning_files: 15,
            average_tdg: 1.5,
            p95_tdg: 2.8,
            p99_tdg: 3.5,
            estimated_debt_hours: 120.0,
            hotspots: vec![crate::models::tdg::TDGHotspot {
                path: "src/complex.rs".to_string(),
                tdg_score: 3.2,
                primary_factor: "High cyclomatic complexity".to_string(),
                estimated_hours: 8.0,
            }],
        }
    }

    #[test]
    fn test_format_tdg_summary_basic() {
        let summary = create_test_tdg_summary();
        let output = format_tdg_summary(&summary);

        assert!(output.contains("# Technical Debt Gradient Analysis"));
        assert!(output.contains("**Total files:** 100"));
    }

    #[test]
    fn test_format_tdg_summary_metrics() {
        let summary = create_test_tdg_summary();
        let output = format_tdg_summary(&summary);

        assert!(output.contains("**Average TDG:**"));
        assert!(output.contains("**95th percentile TDG:**"));
        assert!(output.contains("**99th percentile TDG:**"));
    }

    #[test]
    fn test_format_tdg_summary_hotspots() {
        let summary = create_test_tdg_summary();
        let output = format_tdg_summary(&summary);

        assert!(output.contains("## Top Hotspots"));
        assert!(output.contains("src/complex.rs"));
    }

    #[test]
    fn test_format_tdg_summary_severity() {
        let summary = create_test_tdg_summary();
        let output = format_tdg_summary(&summary);

        assert!(output.contains("## Severity Distribution"));
        assert!(output.contains("Critical"));
        assert!(output.contains("Warning"));
        assert!(output.contains("Normal"));
    }

    // Tests for dead code formatting functions

    fn create_test_dead_code_result() -> DeadCodeRankingResult {
        DeadCodeRankingResult {
            ranked_files: vec![FileDeadCodeMetrics {
                path: "src/unused.rs".to_string(),
                dead_lines: 50,
                total_lines: 200,
                dead_percentage: 25.0,
                dead_functions: 3,
                dead_classes: 1,
                dead_score: 75.0,
                confidence: ConfidenceLevel::High,
                items: vec![DeadCodeItem {
                    item_type: DeadCodeType::Function,
                    name: "unused_fn".to_string(),
                    line: 10,
                    end_line: 20,
                    reason: "Never called".to_string(),
                    confidence: ConfidenceLevel::High,
                }],
            }],
            summary: DeadCodeSummary {
                total_files_analyzed: 50,
                files_with_dead_code: 10,
                total_dead_lines: 200,
                dead_percentage: 4.0,
                dead_functions: 15,
                dead_classes: 3,
                dead_modules: 1,
                unreachable_blocks: 5,
            },
            analysis_timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_format_dead_code_summary_mcp() {
        let result = create_test_dead_code_result();
        let output = format_dead_code_summary_mcp(&result).unwrap();

        assert!(output.contains("# Dead Code Analysis Summary"));
        assert!(output.contains("**Total files analyzed:**"));
    }

    #[test]
    fn test_format_dead_code_as_sarif_mcp() {
        let result = create_test_dead_code_result();
        let output = format_dead_code_as_sarif_mcp(&result).unwrap();

        assert!(output.contains("$schema"));
        assert!(output.contains("sarif"));
        assert!(output.contains("pmat"));
    }

    #[test]
    fn test_format_dead_code_as_markdown_mcp() {
        let result = create_test_dead_code_result();
        let output = format_dead_code_as_markdown_mcp(&result).unwrap();

        assert!(output.contains("# Dead Code Analysis Report"));
        assert!(output.contains("## Summary"));
    }

    #[test]
    fn test_get_confidence_level_text() {
        assert_eq!(get_confidence_level_text(ConfidenceLevel::High), "HIGH ");
        assert_eq!(
            get_confidence_level_text(ConfidenceLevel::Medium),
            "MEDIUM "
        );
        assert_eq!(get_confidence_level_text(ConfidenceLevel::Low), "LOW ");
    }

    #[test]
    fn test_format_confidence_emoji() {
        assert!(format_confidence_emoji(ConfidenceLevel::High).contains("High"));
        assert!(format_confidence_emoji(ConfidenceLevel::Medium).contains("Medium"));
        assert!(format_confidence_emoji(ConfidenceLevel::Low).contains("Low"));
    }

    #[test]
    fn test_calculate_dead_files_percentage_normal() {
        let summary = DeadCodeSummary {
            total_files_analyzed: 100,
            files_with_dead_code: 25,
            total_dead_lines: 0,
            dead_percentage: 0.0,
            dead_functions: 0,
            dead_classes: 0,
            dead_modules: 0,
            unreachable_blocks: 0,
        };
        let pct = calculate_dead_files_percentage(&summary);
        assert!((pct - 25.0).abs() < f64::EPSILON as f32);
    }

    #[test]
    fn test_calculate_dead_files_percentage_zero() {
        let summary = DeadCodeSummary {
            total_files_analyzed: 0,
            files_with_dead_code: 0,
            total_dead_lines: 0,
            dead_percentage: 0.0,
            dead_functions: 0,
            dead_classes: 0,
            dead_modules: 0,
            unreachable_blocks: 0,
        };
        let pct = calculate_dead_files_percentage(&summary);
        assert_eq!(pct, 0.0);
    }
