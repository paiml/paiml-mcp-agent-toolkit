    #[test]
    fn test_is_template_tool_generate() {
        assert!(is_template_tool("generate_template"));
    }

    #[test]
    fn test_is_template_tool_list() {
        assert!(is_template_tool("list_templates"));
    }

    #[test]
    fn test_is_template_tool_validate() {
        assert!(is_template_tool("validate_template"));
    }

    #[test]
    fn test_is_template_tool_scaffold() {
        assert!(is_template_tool("scaffold_project"));
    }

    #[test]
    fn test_is_template_tool_search() {
        assert!(is_template_tool("search_templates"));
    }

    #[test]
    fn test_is_template_tool_false() {
        assert!(!is_template_tool("analyze_complexity"));
        assert!(!is_template_tool("unknown_tool"));
        assert!(!is_template_tool(""));
    }

    // Tests for is_analysis_tool()

    #[test]
    fn test_is_analysis_tool_churn() {
        assert!(is_analysis_tool("analyze_code_churn"));
    }

    #[test]
    fn test_is_analysis_tool_complexity() {
        assert!(is_analysis_tool("analyze_complexity"));
    }

    #[test]
    fn test_is_analysis_tool_dag() {
        assert!(is_analysis_tool("analyze_dag"));
    }

    #[test]
    fn test_is_analysis_tool_context() {
        assert!(is_analysis_tool("generate_context"));
    }

    #[test]
    fn test_is_analysis_tool_architecture() {
        assert!(is_analysis_tool("analyze_system_architecture"));
    }

    #[test]
    fn test_is_analysis_tool_defect() {
        assert!(is_analysis_tool("analyze_defect_probability"));
    }

    #[test]
    fn test_is_analysis_tool_dead_code() {
        assert!(is_analysis_tool("analyze_dead_code"));
    }

    #[test]
    fn test_is_analysis_tool_deep_context() {
        assert!(is_analysis_tool("analyze_deep_context"));
    }

    #[test]
    fn test_is_analysis_tool_tdg() {
        assert!(is_analysis_tool("analyze_tdg"));
    }

    #[test]
    fn test_is_analysis_tool_makefile() {
        assert!(is_analysis_tool("analyze_makefile_lint"));
    }

    #[test]
    fn test_is_analysis_tool_provability() {
        assert!(is_analysis_tool("analyze_provability"));
    }

    #[test]
    fn test_is_analysis_tool_satd() {
        assert!(is_analysis_tool("analyze_satd"));
    }

    #[test]
    fn test_is_analysis_tool_qdd() {
        assert!(is_analysis_tool("quality_driven_development"));
    }

    #[test]
    fn test_is_analysis_tool_lint_hotspot() {
        assert!(is_analysis_tool("analyze_lint_hotspot"));
    }

    #[test]
    fn test_is_analysis_tool_false() {
        assert!(!is_analysis_tool("generate_template"));
        assert!(!is_analysis_tool("unknown_tool"));
        assert!(!is_analysis_tool(""));
    }

    // Tests for format_churn_summary()

    fn create_test_churn_analysis() -> CodeChurnAnalysis {
        CodeChurnAnalysis {
            generated_at: Utc::now(),
            period_days: 30,
            repository_root: PathBuf::from("/test/repo"),
            files: vec![FileChurnMetrics {
                path: PathBuf::from("src/main.rs"),
                relative_path: "src/main.rs".to_string(),
                commit_count: 15,
                unique_authors: vec!["alice".to_string(), "bob".to_string()],
                additions: 200,
                deletions: 50,
                churn_score: 0.8,
                last_modified: Utc::now(),
                first_seen: Utc::now(),
            }],
            summary: ChurnSummary {
                total_commits: 50,
                total_files_changed: 25,
                hotspot_files: vec![PathBuf::from("src/hot.rs")],
                stable_files: vec![PathBuf::from("src/stable.rs")],
                author_contributions: std::collections::BTreeMap::from([
                    ("alice".to_string(), 30),
                    ("bob".to_string(), 20),
                ]),
                mean_churn_score: 0.5,
                variance_churn_score: 0.1,
                stddev_churn_score: 0.316,
            },
        }
    }

    #[test]
    fn test_format_churn_summary_basic() {
        let analysis = create_test_churn_analysis();
        let summary = format_churn_summary(&analysis);

        assert!(summary.contains("# Code Churn Analysis"));
        assert!(summary.contains("Period: 30 days"));
        assert!(summary.contains("Total files changed: 25"));
        assert!(summary.contains("Total commits: 50"));
    }

    #[test]
    fn test_format_churn_summary_hotspots() {
        let analysis = create_test_churn_analysis();
        let summary = format_churn_summary(&analysis);

        assert!(summary.contains("## Hotspot Files"));
        assert!(summary.contains("src/hot.rs"));
    }

    #[test]
    fn test_format_churn_summary_stable() {
        let analysis = create_test_churn_analysis();
        let summary = format_churn_summary(&analysis);

        assert!(summary.contains("## Stable Files"));
        assert!(summary.contains("src/stable.rs"));
    }

    #[test]
    fn test_format_churn_summary_empty() {
        let analysis = CodeChurnAnalysis {
            generated_at: Utc::now(),
            period_days: 7,
            repository_root: PathBuf::from("/test"),
            files: vec![],
            summary: ChurnSummary {
                total_commits: 0,
                total_files_changed: 0,
                hotspot_files: vec![],
                stable_files: vec![],
                author_contributions: std::collections::BTreeMap::new(),
                mean_churn_score: 0.0,
                variance_churn_score: 0.0,
                stddev_churn_score: 0.0,
            },
        };

        let summary = format_churn_summary(&analysis);
        assert!(summary.contains("# Code Churn Analysis"));
        assert!(summary.contains("Period: 7 days"));
        // Should not contain hotspot/stable sections when empty
        assert!(!summary.contains("## Hotspot Files"));
        assert!(!summary.contains("## Stable Files"));
    }

    // Tests for format_churn_as_markdown()

    #[test]
    fn test_format_churn_as_markdown_basic() {
        let analysis = create_test_churn_analysis();
        let markdown = format_churn_as_markdown(&analysis);

        assert!(markdown.contains("# Code Churn Analysis Report"));
        assert!(markdown.contains("**Period:** 30 days"));
        assert!(markdown.contains("**Repository:**"));
    }

    #[test]
    fn test_format_churn_as_markdown_summary_section() {
        let analysis = create_test_churn_analysis();
        let markdown = format_churn_as_markdown(&analysis);

        assert!(markdown.contains("## Summary"));
        assert!(markdown.contains("Total files changed: 25"));
        assert!(markdown.contains("Total commits: 50"));
    }

    // Tests for format_churn_as_csv()

    #[test]
    fn test_format_churn_as_csv_headers() {
        let analysis = create_test_churn_analysis();
        let csv = format_churn_as_csv(&analysis);

        // Check that there's a header line
        assert!(csv.lines().next().is_some());
    }

    #[test]
    fn test_format_churn_as_csv_data() {
        let analysis = create_test_churn_analysis();
        let csv = format_churn_as_csv(&analysis);

        // Check that it contains the file path
        assert!(csv.contains("src/main.rs") || csv.contains("main.rs"));
    }

    #[test]
    fn test_format_churn_as_csv_empty() {
        let analysis = CodeChurnAnalysis {
            generated_at: Utc::now(),
            period_days: 7,
            repository_root: PathBuf::from("/test"),
            files: vec![],
            summary: ChurnSummary {
                total_commits: 0,
                total_files_changed: 0,
                hotspot_files: vec![],
                stable_files: vec![],
                author_contributions: std::collections::BTreeMap::new(),
                mean_churn_score: 0.0,
                variance_churn_score: 0.0,
                stddev_churn_score: 0.0,
            },
        };

        let csv = format_churn_as_csv(&analysis);
        // Should have header but no data rows
        let lines: Vec<_> = csv.lines().collect();
        assert_eq!(lines.len(), 1); // Only header
    }

    // Tests for tool name categorization (both functions together)

    #[test]
    fn test_tools_are_mutually_exclusive() {
        // Template tools should not be analysis tools and vice versa
        let template_tools = [
            "generate_template",
            "list_templates",
            "validate_template",
            "scaffold_project",
            "search_templates",
        ];
        let analysis_tools = [
            "analyze_code_churn",
            "analyze_complexity",
            "analyze_dag",
            "generate_context",
            "analyze_system_architecture",
            "analyze_defect_probability",
            "analyze_dead_code",
            "analyze_deep_context",
            "analyze_tdg",
            "analyze_makefile_lint",
            "analyze_provability",
            "analyze_satd",
            "quality_driven_development",
            "analyze_lint_hotspot",
        ];

        for tool in template_tools {
            assert!(is_template_tool(tool), "{} should be template tool", tool);
            assert!(
                !is_analysis_tool(tool),
                "{} should NOT be analysis tool",
                tool
            );
        }

        for tool in analysis_tools {
            assert!(is_analysis_tool(tool), "{} should be analysis tool", tool);
            assert!(
                !is_template_tool(tool),
                "{} should NOT be template tool",
                tool
            );
        }
    }
