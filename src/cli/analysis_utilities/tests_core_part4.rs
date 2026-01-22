    #[test]
    fn test_get_ngrams() {
        let ngrams = get_ngrams("hello", 2);
        assert!(ngrams.contains("he"));
        assert!(ngrams.contains("el"));
        assert!(ngrams.contains("ll"));
        assert!(ngrams.contains("lo"));
        assert_eq!(ngrams.len(), 4);

        // Test with string shorter than n
        let short_ngrams = get_ngrams("hi", 3);
        assert_eq!(short_ngrams.len(), 1);
        assert!(short_ngrams.contains("hi"));
    }

    #[test]
    fn test_soundex_code() {
        assert_eq!(soundex_code('B'), '1');
        assert_eq!(soundex_code('C'), '2');
        assert_eq!(soundex_code('D'), '3');
        assert_eq!(soundex_code('L'), '4');
        assert_eq!(soundex_code('M'), '5');
        assert_eq!(soundex_code('R'), '6');
        assert_eq!(soundex_code('A'), '0');
        assert_eq!(soundex_code('E'), '0');
    }

    #[test]
    fn test_format_quality_gate_output_json() {
        let results = QualityGateResults {
            passed: false,
            total_violations: 10,
            complexity_violations: 3,
            dead_code_violations: 2,
            satd_violations: 1,
            entropy_violations: 1,
            security_violations: 2,
            duplicate_violations: 1,
            coverage_violations: 0,
            section_violations: 0,
            provability_violations: 0,
            provability_score: Some(85.5),
            violations: Vec::new(),
        };

        let violations = vec![
            QualityViolation {
                check_type: "complexity".to_string(),
                severity: "error".to_string(),
                message: "Function exceeds complexity threshold".to_string(),
                file: "src/main.rs".to_string(),
                line: Some(42),
            },
            QualityViolation {
                check_type: "dead_code".to_string(),
                severity: "warning".to_string(),
                message: "Unused function detected".to_string(),
                file: "src/utils.rs".to_string(),
                line: Some(100),
            },
        ];

        let output =
            format_quality_gate_output(&results, &violations, QualityGateOutputFormat::Json);
        assert!(output.is_ok());

        let json = output.unwrap();
        assert!(json.contains("\"passed\": false"));
        assert!(json.contains("\"total_violations\": 10"));
        assert!(json.contains("\"complexity_violations\": 3"));
        assert!(json.contains("src/main.rs"));
    }

    #[test]
    fn test_format_quality_gate_output_human() {
        let results = QualityGateResults {
            passed: true,
            total_violations: 0,
            complexity_violations: 0,
            dead_code_violations: 0,
            satd_violations: 0,
            entropy_violations: 0,
            security_violations: 0,
            duplicate_violations: 0,
            coverage_violations: 0,
            section_violations: 0,
            provability_violations: 0,
            provability_score: Some(95.0),
            violations: Vec::new(),
        };

        let violations = vec![];

        let output =
            format_quality_gate_output(&results, &violations, QualityGateOutputFormat::Human);
        assert!(output.is_ok());

        let text = output.unwrap();
        assert!(text.contains("✅ PASSED"));
        assert!(text.contains("Total violations: 0"));
        assert!(text.contains("Provability score: 95.00"));
    }

    #[test]
    fn test_format_quality_gate_output_junit() {
        let results = QualityGateResults {
            passed: false,
            total_violations: 2,
            complexity_violations: 1,
            dead_code_violations: 1,
            satd_violations: 0,
            entropy_violations: 0,
            security_violations: 0,
            duplicate_violations: 0,
            coverage_violations: 0,
            section_violations: 0,
            provability_violations: 0,
            provability_score: None,
            violations: Vec::new(),
        };

        let violations = vec![QualityViolation {
            check_type: "complexity".to_string(),
            severity: "error".to_string(),
            message: "Cyclomatic complexity 25 exceeds limit 20".to_string(),
            file: "src/complex.rs".to_string(),
            line: Some(50),
        }];

        let output =
            format_quality_gate_output(&results, &violations, QualityGateOutputFormat::Junit);
        assert!(output.is_ok());

        let xml = output.unwrap();
        assert!(xml.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml.contains("<testsuites name=\"Quality Gate\">"));
        assert!(xml.contains("<testcase name=\"Cyclomatic complexity 25 exceeds limit 20\""));
        assert!(xml.contains(
            "<failure message=\"Cyclomatic complexity 25 exceeds limit 20\" type=\"error\"/>"
        ));
    }

    #[test]
    fn test_format_quality_gate_output_summary() {
        let results = QualityGateResults {
            passed: true,
            total_violations: 0,
            complexity_violations: 0,
            dead_code_violations: 0,
            satd_violations: 0,
            entropy_violations: 0,
            security_violations: 0,
            duplicate_violations: 0,
            coverage_violations: 0,
            section_violations: 0,
            provability_violations: 0,
            provability_score: None,
            violations: Vec::new(),
        };

        let violations = vec![];

        let output =
            format_quality_gate_output(&results, &violations, QualityGateOutputFormat::Summary);
        assert!(output.is_ok());

        let text = output.unwrap();
        assert!(text.contains("Quality Gate: PASSED"));
        assert!(text.contains("Total violations: 0"));
        assert!(!text.contains("##")); // Summary should be minimal
    }

    #[test]
    fn test_format_quality_gate_output_detailed() {
        let results = QualityGateResults {
            passed: false,
            total_violations: 5,
            complexity_violations: 1,
            dead_code_violations: 1,
            satd_violations: 1,
            entropy_violations: 0,
            security_violations: 1,
            duplicate_violations: 1,
            coverage_violations: 0,
            section_violations: 0,
            provability_violations: 0,
            provability_score: Some(78.5),
            violations: Vec::new(),
        };

        let violations = vec![QualityViolation {
            check_type: "security".to_string(),
            severity: "error".to_string(),
            message: "Potential SQL injection vulnerability".to_string(),
            file: "src/db.rs".to_string(),
            line: Some(123),
        }];

        let output =
            format_quality_gate_output(&results, &violations, QualityGateOutputFormat::Detailed);
        assert!(output.is_ok());

        let text = output.unwrap();
        assert!(text.contains("❌ FAILED"));
        assert!(text.contains("## Violations by Type"));
        assert!(text.contains("- Complexity: 1"));
        assert!(text.contains("- Security: 1"));
        assert!(text.contains("Potential SQL injection vulnerability"));
        assert!(text.contains("src/db.rs:123"));
    }

    #[test]
    fn test_format_quality_gate_output_all_violation_types() {
        let results = QualityGateResults {
            passed: false,
            total_violations: 9,
            complexity_violations: 1,
            dead_code_violations: 1,
            satd_violations: 1,
            entropy_violations: 1,
            security_violations: 1,
            duplicate_violations: 1,
            coverage_violations: 1,
            section_violations: 1,
            provability_violations: 1,
            provability_score: Some(65.0),
            violations: Vec::new(),
        };

        let violations = vec![];

        let output =
            format_quality_gate_output(&results, &violations, QualityGateOutputFormat::Human);
        assert!(output.is_ok());

        let text = output.unwrap();
        assert!(text.contains("## Complexity violations: 1"));
        assert!(text.contains("## Dead code violations: 1"));
        assert!(text.contains("## Technical debt violations: 1"));
        assert!(text.contains("## Entropy violations: 1"));
        assert!(text.contains("## Security violations: 1"));
        assert!(text.contains("## Duplicate code violations: 1"));
    }

    // TDD Tests for extracted helper functions (Toyota Way)
    // Testing the functions we extracted to reduce complexity

    #[test]
    fn test_create_complexity_test_file() {
        use std::io::Read;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Test successful file creation
        let result = create_complexity_test_file(project_path);
        assert!(result.is_ok());

        // Verify file was created
        let src_dir = project_path.join("src");
        let test_file = src_dir.join("complex.rs");
        assert!(test_file.exists());

        // Verify file contents contain expected functions
        let mut contents = String::new();
        std::fs::File::open(&test_file)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();
        eprintln!("File contents:\n{}", contents);
        assert!(contents.contains("fn simple_function()"));
        assert!(contents.contains("fn moderate_function("));
        assert!(contents.contains("for i in 0..10"));
    }

    #[tokio::test]
    async fn test_validate_complexity_threshold_pass() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Create test file first
        create_complexity_test_file(project_path).unwrap();

        // This should not panic since threshold is high enough
        validate_complexity_threshold_pass(project_path, 25).await;

        // Test passes if no assertion fails
    }

    #[tokio::test]
    async fn test_validate_complexity_threshold_fail() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Create test file first
        create_complexity_test_file(project_path).unwrap();

        // This should not panic - it should find violations with low threshold
        validate_complexity_threshold_fail(project_path, 1).await;

        // Test passes if no assertion fails
    }

    #[test]
    fn test_apply_churn_file_filtering() {
        use crate::models::churn::{ChurnSummary, CodeChurnAnalysis, FileChurnMetrics};
        use chrono::Utc;
        use std::collections::HashMap;

        // Create test analysis with multiple files
        let mut analysis = CodeChurnAnalysis {
            generated_at: Utc::now(),
            period_days: 30,
            repository_root: std::path::PathBuf::from("."),
            files: vec![
                FileChurnMetrics {
                    path: std::path::PathBuf::from("file1.rs"),
                    relative_path: "file1.rs".to_string(),
                    commit_count: 10,
                    unique_authors: vec!["dev1".to_string()],
                    additions: 100,
                    deletions: 50,
                    churn_score: 0.8,
                    last_modified: Utc::now(),
                    first_seen: Utc::now(),
                },
                FileChurnMetrics {
                    path: std::path::PathBuf::from("file2.rs"),
                    relative_path: "file2.rs".to_string(),
                    commit_count: 15,
                    unique_authors: vec!["dev2".to_string()],
                    additions: 200,
                    deletions: 100,
                    churn_score: 0.9,
                    last_modified: Utc::now(),
                    first_seen: Utc::now(),
                },
                FileChurnMetrics {
                    path: std::path::PathBuf::from("file3.rs"),
                    relative_path: "file3.rs".to_string(),
                    commit_count: 5,
                    unique_authors: vec!["dev3".to_string()],
                    additions: 50,
                    deletions: 25,
                    churn_score: 0.3,
                    last_modified: Utc::now(),
                    first_seen: Utc::now(),
                },
            ],
            summary: ChurnSummary {
                total_commits: 30,
                total_files_changed: 3,
                author_contributions: HashMap::new(),
                hotspot_files: vec![],
                stable_files: vec![],
                mean_churn_score: 0.0,
                variance_churn_score: 0.0,
                stddev_churn_score: 0.0,
            },
        };

        // Test with no filtering (top_files = 0)
        let original_count = analysis.files.len();
        apply_churn_file_filtering(&mut analysis, 0);
        assert_eq!(analysis.files.len(), original_count);

        // Test with filtering (top_files = 2)
        apply_churn_file_filtering(&mut analysis, 2);
        assert_eq!(analysis.files.len(), 2);
        // Should be sorted by commit count desc, so file2 (15) and file1 (10)
        assert_eq!(analysis.files[0].commit_count, 15);
        assert_eq!(analysis.files[1].commit_count, 10);
    }

    #[test]
    fn test_format_churn_content() {
        use crate::models::churn::{ChurnOutputFormat, ChurnSummary, CodeChurnAnalysis};
        use chrono::Utc;
        use std::collections::HashMap;

        let analysis = CodeChurnAnalysis {
            generated_at: Utc::now(),
            period_days: 30,
            repository_root: std::path::PathBuf::from("."),
            files: vec![],
            summary: ChurnSummary {
                total_commits: 0,
                total_files_changed: 0,
                author_contributions: HashMap::new(),
                hotspot_files: vec![],
                stable_files: vec![],
                mean_churn_score: 0.0,
                variance_churn_score: 0.0,
                stddev_churn_score: 0.0,
            },
        };

        // Test JSON format
        let json_result = format_churn_content(&analysis, ChurnOutputFormat::Json);
        assert!(json_result.is_ok());
        let json_content = json_result.unwrap();
        assert!(json_content.contains("generated_at"));

        // Test Summary format
        let summary_result = format_churn_content(&analysis, ChurnOutputFormat::Summary);
        assert!(summary_result.is_ok());

        // Test Markdown format
        let markdown_result = format_churn_content(&analysis, ChurnOutputFormat::Markdown);
        assert!(markdown_result.is_ok());

        // Test CSV format
        let csv_result = format_churn_content(&analysis, ChurnOutputFormat::Csv);
        assert!(csv_result.is_ok());
    }

    #[test]
    #[ignore] // Five Whys: Process-global CWD modification causes race conditions under parallel execution
              // Root cause: std::env::set_current_dir() is process-wide, not thread-local
              // Fix attempted: RAII CwdGuard failed because current_dir() fails if CWD deleted
              // Decision: Mark as #[ignore] - unsuitable for parallel test execution
              // Run manually: cargo test test_run_comprehensive_analyses_basic -- --ignored --test-threads=1
    fn test_run_comprehensive_analyses_basic() {
        // This is a simple test to verify the function signature and basic structure
        // In a real scenario, we'd need to mock the analysis functions

        use std::path::PathBuf;

        // Create basic test data
        let mut report = ComprehensiveReport::default();
        let project_path = PathBuf::from(".");

        // Test with all options disabled (minimal execution path)
        let rt = tokio::runtime::Runtime::new().unwrap();
        let config = ComprehensiveAnalysisConfig::new(
            false, // include_complexity
            false, // include_tdg
            false, // include_dead_code
            false, // include_defects
            false, // include_duplicates
            &None, // include
            &None, // exclude
            0.5,   // confidence_threshold
            10,    // min_lines
        );
        let result = rt.block_on(async {
            run_comprehensive_analyses(&mut report, &project_path, &config).await
        });

        // Should succeed with minimal configuration
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_write_comprehensive_output() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let output_file = temp_dir.path().join("test_output.json");

        let report = ComprehensiveReport::default();

        // Test writing to file
        let result = write_comprehensive_output(
            &report,
            ComprehensiveOutputFormat::Json,
            false, // executive_summary
            Some(output_file.clone()),
        )
        .await;

        assert!(result.is_ok());
        assert!(output_file.exists());

        // Test writing to stdout (no file path)
        let stdout_result =
            write_comprehensive_output(&report, ComprehensiveOutputFormat::Json, false, None).await;

        assert!(stdout_result.is_ok());
    }
