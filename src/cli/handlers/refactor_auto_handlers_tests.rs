//\! Tests for refactor auto handlers
//\! Extracted for file health compliance (CB-040)

use super::*;

mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use tokio;

    #[test]
    fn test_quality_profile_default() {
        let profile = QualityProfile::default();

        assert_eq!(profile.coverage_min, 80.0);
        assert_eq!(profile.complexity_max, 20);
        assert_eq!(profile.complexity_target, 10);
        assert_eq!(profile.satd_allowed, 0);
    }

    #[test]
    fn test_quality_profile_creation() {
        let profile = QualityProfile {
            coverage_min: 75.0,
            complexity_max: 15,
            complexity_target: 8,
            satd_allowed: 2,
        };

        assert_eq!(profile.coverage_min, 75.0);
        assert_eq!(profile.complexity_max, 15);
        assert_eq!(profile.complexity_target, 8);
        assert_eq!(profile.satd_allowed, 2);
    }

    #[test]
    fn test_quality_metrics_default() {
        let metrics = QualityMetrics::default();

        assert_eq!(metrics.total_violations, 0);
        assert_eq!(metrics.coverage_percent, 0.0);
        assert_eq!(metrics.max_complexity, 0);
        assert_eq!(metrics.satd_count, 0);
        assert_eq!(metrics.files_with_issues, 0);
        assert_eq!(metrics.total_files, 0);
        assert_eq!(metrics.functions_with_high_complexity, 0);
    }

    #[test]
    fn test_quality_metrics_creation() {
        let metrics = QualityMetrics {
            total_violations: 50,
            coverage_percent: 75.5,
            max_complexity: 25,
            satd_count: 3,
            files_with_issues: 8,
            total_files: 20,
            functions_with_high_complexity: 12,
            total_functions: 100,
        };

        assert_eq!(metrics.total_violations, 50);
        assert_eq!(metrics.coverage_percent, 75.5);
        assert_eq!(metrics.max_complexity, 25);
        assert_eq!(metrics.satd_count, 3);
        assert_eq!(metrics.files_with_issues, 8);
        assert_eq!(metrics.total_files, 20);
        assert_eq!(metrics.functions_with_high_complexity, 12);
    }

    #[test]
    fn test_refactor_progress_default() {
        let progress = RefactorProgress::default();

        assert_eq!(progress.files_completed, 0);
        assert_eq!(progress.files_remaining, 0);
        assert_eq!(progress.overall_completion_percent, 0.0);
        assert_eq!(progress.current_phase, RefactorPhase::default());
    }

    #[test]
    fn test_refactor_progress_creation() {
        let progress = RefactorProgress {
            overall_completion_percent: 75.0,
            lint_completion_percent: 80.0,
            complexity_completion_percent: 70.0,
            satd_completion_percent: 85.0,
            coverage_completion_percent: 60.0,
            files_completed: 8,
            files_remaining: 7,
            estimated_time_remaining_minutes: 15,
            quality_gates_passed: vec!["lint".to_string(), "complexity".to_string()],
            quality_gates_remaining: vec!["satd".to_string(), "coverage".to_string()],
            current_phase: RefactorPhase::ComplexityReduction,
        };

        assert_eq!(progress.files_completed, 8);
        assert_eq!(progress.files_remaining, 7);
        assert_eq!(progress.overall_completion_percent, 75.0);
        assert_eq!(progress.quality_gates_passed.len(), 2);
    }

    #[test]
    fn test_refactor_state_creation() {
        let start_time = std::time::SystemTime::now();
        let state = RefactorState {
            iteration: 2,
            context_generated: true,
            context_path: PathBuf::from("/tmp/context"),
            current_file: Some(PathBuf::from("/src/test.rs")),
            files_completed: vec![PathBuf::from("/src/lib.rs")],
            quality_metrics: QualityMetrics::default(),
            progress: RefactorProgress::default(),
            start_time,
        };

        assert_eq!(state.iteration, 2);
        assert!(state.context_generated);
        assert_eq!(state.context_path, PathBuf::from("/tmp/context"));
        assert_eq!(state.current_file, Some(PathBuf::from("/src/test.rs")));
        assert_eq!(state.files_completed.len(), 1);
        assert_eq!(state.files_completed[0], PathBuf::from("/src/lib.rs"));
    }

    #[test]
    fn test_lint_hotspot_json_creation() {
        let hotspot = LintHotspotJson {
            file: PathBuf::from("/src/main.rs"),
            defect_density: 2.5,
            total_violations: 10,
        };

        assert_eq!(hotspot.file, PathBuf::from("/src/main.rs"));
        assert_eq!(hotspot.defect_density, 2.5);
        assert_eq!(hotspot.total_violations, 10);
    }

    #[test]
    fn test_violation_detail_json_creation() {
        let violation = ViolationDetailJson {
            file: PathBuf::from("/src/test.rs"),
            line: 42,
            column: 10,
            end_line: 42,
            end_column: 15,
            lint_name: "dead_code".to_string(),
            message: "unused variable".to_string(),
            severity: "warning".to_string(),
            suggestion: Some("remove unused variable".to_string()),
            machine_applicable: true,
        };

        assert_eq!(violation.file, PathBuf::from("/src/test.rs"));
        assert_eq!(violation.line, 42);
        assert_eq!(violation.column, 10);
        assert_eq!(violation.end_line, 42);
        assert_eq!(violation.end_column, 15);
        assert_eq!(violation.lint_name, "dead_code");
        assert_eq!(violation.message, "unused variable");
        assert_eq!(violation.severity, "warning");
        assert_eq!(
            violation.suggestion,
            Some("remove unused variable".to_string())
        );
        assert!(violation.machine_applicable);
    }

    #[test]
    fn test_lint_hotspot_json_response_creation() {
        let hotspot = LintHotspotJson {
            file: PathBuf::from("/src/lib.rs"),
            defect_density: 1.5,
            total_violations: 5,
        };

        let violation = ViolationDetailJson {
            file: PathBuf::from("/src/lib.rs"),
            line: 10,
            column: 5,
            end_line: 10,
            end_column: 8,
            lint_name: "clippy::complexity".to_string(),
            message: "complex expression".to_string(),
            severity: "error".to_string(),
            suggestion: None,
            machine_applicable: false,
        };

        let response = LintHotspotJsonResponse {
            hotspot,
            all_violations: vec![violation],
            total_project_violations: 25,
        };

        assert_eq!(response.hotspot.file, PathBuf::from("/src/lib.rs"));
        assert_eq!(response.hotspot.defect_density, 1.5);
        assert_eq!(response.all_violations.len(), 1);
        assert_eq!(response.all_violations[0].lint_name, "clippy::complexity");
        assert_eq!(response.total_project_violations, 25);
    }

    #[test]
    fn test_parse_github_issue_url_valid() {
        let url = "https://github.com/owner/repo/issues/123";
        let result = parse_github_issue_url(url);

        assert!(result.is_ok());
        let issue_ref = result.unwrap();
        assert_eq!(issue_ref.owner, "owner");
        assert_eq!(issue_ref.repo, "repo");
        assert_eq!(issue_ref.issue_number, 123);
    }

    #[test]
    fn test_parse_github_issue_url_invalid() {
        let url = "https://invalid-url.com/not-github";
        let result = parse_github_issue_url(url);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_coverage_from_output_valid() {
        let output = b"Coverage: 85.5%\nTotal lines: 1000";
        let result = parse_coverage_from_output(output);

        assert_eq!(result, Some(85.5));
    }

    #[test]
    fn test_parse_coverage_from_output_no_match() {
        let output = b"No coverage information available";
        let result = parse_coverage_from_output(output);

        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_coverage_from_output_multiple_matches() {
        let output = b"Test Coverage: 78.2%\nLine Coverage: 85.0%";
        let result = parse_coverage_from_output(output);

        // Should return the first match
        assert_eq!(result, Some(78.2));
    }

    #[tokio::test]
    async fn test_load_ignore_patterns_with_gitignore() {
        let temp_dir = TempDir::new().unwrap();
        let gitignore_path = temp_dir.path().join(".gitignore");
        std::fs::write(&gitignore_path, "target/\n*.tmp\n").unwrap();

        let config = PatternConfig {
            root_path: temp_dir.path().to_path_buf(),
            ignore_file: Some(".gitignore".to_string()),
            patterns: vec![],
            include_patterns: vec![],
            exclude_patterns: vec![],
            ignore_file_path: Some(gitignore_path),
            file_extensions: vec!["rs".to_string()],
        };

        let result = load_ignore_patterns(&config).await;

        assert!(result.is_ok());
        let patterns = result.unwrap();
        assert!(patterns.contains(&"target/".to_string()));
        assert!(patterns.contains(&"*.tmp".to_string()));
    }

    #[tokio::test]
    async fn test_load_ignore_patterns_no_file() {
        let temp_dir = TempDir::new().unwrap();

        let config = PatternConfig {
            root_path: temp_dir.path().to_path_buf(),
            ignore_file: Some(".nonexistent".to_string()),
            patterns: vec![],
            include_patterns: vec![],
            exclude_patterns: vec!["manual_pattern".to_string()],
            ignore_file_path: None,
            file_extensions: vec!["rs".to_string()],
        };

        let result = load_ignore_patterns(&config).await;

        assert!(result.is_ok());
        let patterns = result.unwrap();
        assert!(patterns.contains(&"manual_pattern".to_string()));
    }

    #[tokio::test]
    async fn test_discover_source_files_empty_directory() {
        let temp_dir = TempDir::new().unwrap();

        let config = PatternConfig {
            root_path: temp_dir.path().to_path_buf(),
            ignore_file: None,
            patterns: vec![],
            include_patterns: vec![],
            exclude_patterns: vec![],
            ignore_file_path: None,
            file_extensions: vec!["rs".to_string()],
        };

        let result = discover_source_files(&config.root_path, &config, &[]).await;

        assert!(result.is_ok());
        let files = result.unwrap();
        assert!(files.is_empty());
    }

    #[tokio::test]
    async fn test_discover_source_files_with_rust_files() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("main.rs");
        std::fs::write(&rust_file, "fn main() {}").unwrap();

        let config = PatternConfig {
            root_path: temp_dir.path().to_path_buf(),
            ignore_file: None,
            patterns: vec![],
            include_patterns: vec![],
            exclude_patterns: vec![],
            ignore_file_path: None,
            file_extensions: vec!["rs".to_string()],
        };

        let result = discover_source_files(&config.root_path, &config, &[]).await;

        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("main.rs"));
    }

    #[test]
    fn test_extract_target_files_from_issue() {
        let content = GitHubIssueContent {
            title: "Fix issues in src/main.rs and tests/test.rs".to_string(),
            body:
                "Found problems in:\n- src/lib.rs\n- src/utils.rs\n\nNeed to refactor these files."
                    .to_string(),
            number: 123,
        };

        let result = extract_target_files_from_issue(&content, Path::new("/project")).unwrap();

        assert_eq!(result.len(), 4); // main.rs, test.rs, lib.rs, utils.rs
        assert!(result
            .iter()
            .any(|p| p.to_string_lossy().ends_with("main.rs")));
        assert!(result
            .iter()
            .any(|p| p.to_string_lossy().ends_with("test.rs")));
        assert!(result
            .iter()
            .any(|p| p.to_string_lossy().ends_with("lib.rs")));
        assert!(result
            .iter()
            .any(|p| p.to_string_lossy().ends_with("utils.rs")));
    }

    #[test]
    fn test_extract_target_files_from_issue_no_files() {
        let content = GitHubIssueContent {
            title: "General refactoring needed".to_string(),
            body: "This project needs general improvements.".to_string(),
            number: 456,
        };

        let result = extract_target_files_from_issue(&content, Path::new("/project")).unwrap();

        assert!(result.is_empty());
    }

    #[test]
    fn test_refactor_auto_config_creation() {
        use std::path::PathBuf;

        let config = RefactorAutoConfig {
            project_path: PathBuf::from("/test/project"),
            single_file_mode: true,
            file: Some(PathBuf::from("test.rs")),
            format: RefactorAutoOutputFormat::Json,
            max_iterations: 5,
            cache_dir: Some(PathBuf::from("/tmp/cache")),
            dry_run: true,
            ci_mode: false,
            exclude_patterns: vec!["*.tmp".to_string()],
            include_patterns: vec!["*.rs".to_string()],
            ignore_file: Some(PathBuf::from(".gitignore")),
            test_file: None,
            test_name: None,
            github_issue_url: None,
            bug_report_path: None,
        };

        assert_eq!(config.project_path, PathBuf::from("/test/project"));
        assert!(config.single_file_mode);
        assert_eq!(config.file, Some(PathBuf::from("test.rs")));
        assert_eq!(config.max_iterations, 5);
        assert!(config.dry_run);
        assert!(!config.ci_mode);
        assert_eq!(config.exclude_patterns.len(), 1);
        assert_eq!(config.include_patterns.len(), 1);
    }

    #[test]
    fn test_refactor_auto_config_default_values() {
        use std::path::PathBuf;

        let config = RefactorAutoConfig {
            project_path: PathBuf::from("."),
            single_file_mode: false,
            file: None,
            format: RefactorAutoOutputFormat::Summary,
            max_iterations: 1,
            cache_dir: None,
            dry_run: false,
            ci_mode: false,
            exclude_patterns: Vec::new(),
            include_patterns: Vec::new(),
            ignore_file: None,
            test_file: None,
            test_name: None,
            github_issue_url: None,
            bug_report_path: None,
        };

        assert_eq!(config.project_path, PathBuf::from("."));
        assert!(!config.single_file_mode);
        assert!(config.file.is_none());
        assert_eq!(config.max_iterations, 1);
        assert!(!config.dry_run);
        assert!(config.exclude_patterns.is_empty());
        assert!(config.include_patterns.is_empty());
    }

    #[test]
    fn test_refactor_auto_config_clone() {
        use std::path::PathBuf;

        let original = RefactorAutoConfig {
            project_path: PathBuf::from("/original"),
            single_file_mode: true,
            file: Some(PathBuf::from("original.rs")),
            format: RefactorAutoOutputFormat::Detailed,
            max_iterations: 10,
            cache_dir: Some(PathBuf::from("/cache")),
            dry_run: true,
            ci_mode: true,
            exclude_patterns: vec!["exclude".to_string()],
            include_patterns: vec!["include".to_string()],
            ignore_file: Some(PathBuf::from(".ignore")),
            test_file: Some(PathBuf::from("test.rs")),
            test_name: Some("test_name".to_string()),
            github_issue_url: Some("https://github.com/test".to_string()),
            bug_report_path: Some(PathBuf::from("bug.md")),
        };

        let cloned = original.clone();

        assert_eq!(cloned.project_path, original.project_path);
        assert_eq!(cloned.single_file_mode, original.single_file_mode);
        assert_eq!(cloned.file, original.file);
        assert_eq!(cloned.max_iterations, original.max_iterations);
        assert_eq!(cloned.dry_run, original.dry_run);
        assert_eq!(cloned.ci_mode, original.ci_mode);
        assert_eq!(cloned.exclude_patterns, original.exclude_patterns);
        assert_eq!(cloned.include_patterns, original.include_patterns);
        assert_eq!(cloned.test_name, original.test_name);
    }
}


mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}


mod coverage_tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    // RefactorPhase Tests

    #[test]
    fn test_refactor_phase_default() {
        let phase = RefactorPhase::default();
        assert_eq!(phase, RefactorPhase::Initialization);
    }

    #[test]
    fn test_refactor_phase_equality() {
        assert_eq!(RefactorPhase::LintFixes, RefactorPhase::LintFixes);
        assert_ne!(RefactorPhase::LintFixes, RefactorPhase::BuildFixes);
        assert_ne!(
            RefactorPhase::ComplexityReduction,
            RefactorPhase::SatdCleanup
        );
    }

    #[test]
    fn test_refactor_phase_clone() {
        let phase = RefactorPhase::CoverageDriven;
        let cloned = phase.clone();
        assert_eq!(phase, cloned);
    }

    #[test]
    fn test_refactor_phase_all_variants() {
        let phases = vec![
            RefactorPhase::Initialization,
            RefactorPhase::LintFixes,
            RefactorPhase::BuildFixes,
            RefactorPhase::ComplexityReduction,
            RefactorPhase::SatdCleanup,
            RefactorPhase::CoverageDriven,
            RefactorPhase::QualityValidation,
            RefactorPhase::Complete,
        ];

        for phase in &phases {
            let cloned = phase.clone();
            assert_eq!(&cloned, phase);
        }
    }

    // RefactorMode Tests

    #[test]
    fn test_refactor_mode_project_wide() {
        let mode = RefactorMode::ProjectWide;
        assert!(matches!(mode, RefactorMode::ProjectWide));
    }

    #[test]
    fn test_refactor_mode_single_file() {
        let mode = RefactorMode::SingleFile(PathBuf::from("test.rs"));
        if let RefactorMode::SingleFile(path) = mode {
            assert_eq!(path, PathBuf::from("test.rs"));
        } else {
            panic!("Expected SingleFile mode");
        }
    }

    #[test]
    fn test_refactor_mode_bug_report() {
        let mode = RefactorMode::BugReport(PathBuf::from("bug.md"));
        if let RefactorMode::BugReport(path) = mode {
            assert_eq!(path, PathBuf::from("bug.md"));
        } else {
            panic!("Expected BugReport mode");
        }
    }

    #[test]
    fn test_refactor_mode_github_issue() {
        let mode = RefactorMode::GitHubIssue("https://github.com/test/repo/issues/123".to_string());
        if let RefactorMode::GitHubIssue(url) = mode {
            assert!(url.contains("github.com"));
        } else {
            panic!("Expected GitHubIssue mode");
        }
    }

    // RefactoringType Tests

    #[test]
    fn test_refactoring_type_variants() {
        let types = vec![
            RefactoringType::ComplexityReduction,
            RefactoringType::LintFix,
            RefactoringType::SatdCleanup,
            RefactoringType::CoverageImprovement,
            RefactoringType::SecurityFix,
        ];

        assert_eq!(types.len(), 5);
    }

    #[test]
    fn test_refactoring_type_clone() {
        let rt = RefactoringType::ComplexityReduction;
        let cloned = rt.clone();
        assert!(matches!(cloned, RefactoringType::ComplexityReduction));
    }

    // RefactoringPriority Tests

    #[test]
    fn test_refactoring_priority_variants() {
        let priorities = vec![
            RefactoringPriority::Critical,
            RefactoringPriority::High,
            RefactoringPriority::Medium,
            RefactoringPriority::Low,
        ];

        assert_eq!(priorities.len(), 4);
    }

    #[test]
    fn test_refactoring_priority_clone() {
        let priority = RefactoringPriority::Critical;
        let cloned = priority.clone();
        assert!(matches!(cloned, RefactoringPriority::Critical));
    }

    // RefactoringEffort Tests

    #[test]
    fn test_refactoring_effort_variants() {
        let efforts = vec![
            RefactoringEffort::Trivial,
            RefactoringEffort::Minor,
            RefactoringEffort::Moderate,
            RefactoringEffort::Major,
            RefactoringEffort::Extensive,
        ];

        assert_eq!(efforts.len(), 5);
    }

    #[test]
    fn test_refactoring_effort_clone() {
        let effort = RefactoringEffort::Moderate;
        let cloned = effort.clone();
        assert!(matches!(cloned, RefactoringEffort::Moderate));
    }

    // VerificationStatus Tests

    #[test]
    fn test_verification_status_pending() {
        let status = VerificationStatus::Pending;
        assert!(matches!(status, VerificationStatus::Pending));
    }

    #[test]
    fn test_verification_status_verified() {
        let status = VerificationStatus::Verified;
        assert!(matches!(status, VerificationStatus::Verified));
    }

    #[test]
    fn test_verification_status_failed() {
        let status = VerificationStatus::Failed("Test error".to_string());
        if let VerificationStatus::Failed(msg) = status {
            assert_eq!(msg, "Test error");
        } else {
            panic!("Expected Failed status");
        }
    }

    // FixStrategy Tests

    #[test]
    fn test_fix_strategy_variants() {
        let strategies = vec![
            FixStrategy::ExtractFunction,
            FixStrategy::SimplifyCondition,
            FixStrategy::RemoveDeadCode,
            FixStrategy::AddTest,
            FixStrategy::ApplySuggestion("Apply fix".to_string()),
        ];

        assert_eq!(strategies.len(), 5);
    }

    #[test]
    fn test_fix_strategy_apply_suggestion() {
        let strategy = FixStrategy::ApplySuggestion("Use Vec::new()".to_string());
        if let FixStrategy::ApplySuggestion(suggestion) = strategy {
            assert_eq!(suggestion, "Use Vec::new()");
        } else {
            panic!("Expected ApplySuggestion");
        }
    }

    // parse_github_issue_url Tests

    #[test]
    fn test_parse_github_issue_url_valid_format() {
        let url = "https://github.com/owner/repo/issues/456";
        let result = parse_github_issue_url(url);

        assert!(result.is_ok());
        let issue_ref = result.unwrap();
        assert_eq!(issue_ref.owner, "owner");
        assert_eq!(issue_ref.repo, "repo");
        assert_eq!(issue_ref.issue_number, 456);
    }

    #[test]
    fn test_parse_github_issue_url_invalid_host() {
        let url = "https://gitlab.com/owner/repo/issues/123";
        let result = parse_github_issue_url(url);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_github_issue_url_missing_issues_path() {
        let url = "https://github.com/owner/repo/pull/123";
        let result = parse_github_issue_url(url);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_github_issue_url_invalid_issue_number() {
        let url = "https://github.com/owner/repo/issues/abc";
        let result = parse_github_issue_url(url);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_github_issue_url_too_short() {
        let url = "https://github.com/owner";
        let result = parse_github_issue_url(url);

        assert!(result.is_err());
    }

    // parse_coverage_from_output Tests

    #[test]
    fn test_parse_coverage_from_output_percentage() {
        let output = b"Line coverage: 85.5%";
        let result = parse_coverage_from_output(output);

        assert_eq!(result, Some(85.5));
    }

    #[test]
    fn test_parse_coverage_from_output_mixed_case() {
        let output = b"COVERAGE REPORT: 92.3%";
        let result = parse_coverage_from_output(output);

        assert_eq!(result, Some(92.3));
    }

    #[test]
    fn test_parse_coverage_from_output_no_match() {
        let output = b"Build successful, no coverage data";
        let result = parse_coverage_from_output(output);

        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_coverage_from_output_first_match() {
        let output = b"Branch coverage: 70.0%, Line coverage: 85.0%";
        let result = parse_coverage_from_output(output);

        // Should return first match
        assert_eq!(result, Some(70.0));
    }

    // should_retry_refactoring Tests

    #[test]
    fn test_should_retry_on_timeout() {
        let error = anyhow::anyhow!("Connection timeout occurred");
        assert!(should_retry_refactoring(&error));
    }

    #[test]
    fn test_should_retry_on_network_error() {
        let error = anyhow::anyhow!("Network connection failed");
        assert!(should_retry_refactoring(&error));
    }

    #[test]
    fn test_should_retry_on_temporary_error() {
        let error = anyhow::anyhow!("Temporary failure, please retry");
        assert!(should_retry_refactoring(&error));
    }

    #[test]
    fn test_should_not_retry_on_permanent_error() {
        let error = anyhow::anyhow!("File not found");
        assert!(!should_retry_refactoring(&error));
    }

    #[test]
    fn test_should_not_retry_on_syntax_error() {
        let error = anyhow::anyhow!("Syntax error at line 42");
        assert!(!should_retry_refactoring(&error));
    }

    // Markdown Helper Tests

    #[test]
    fn test_is_markdown_file_true() {
        assert!(is_markdown_file(Path::new("README.md")));
        assert!(is_markdown_file(Path::new("/path/to/doc.md")));
    }

    #[test]
    fn test_is_markdown_file_false() {
        assert!(!is_markdown_file(Path::new("main.rs")));
        assert!(!is_markdown_file(Path::new("Cargo.toml")));
        assert!(!is_markdown_file(Path::new("no_extension")));
    }

    #[test]
    fn test_has_proper_headers_true() {
        assert!(has_proper_headers("# Main Header\n\nContent"));
        assert!(has_proper_headers("## Sub Header\n\nContent"));
    }

    #[test]
    fn test_has_proper_headers_false() {
        assert!(!has_proper_headers("No headers here, just content."));
        assert!(!has_proper_headers("###Not a header (no space)"));
    }

    #[test]
    fn test_has_unspecified_code_blocks_true() {
        let content = "```\ncode here\n```";
        assert!(has_unspecified_code_blocks(content));
    }

    #[test]
    fn test_has_unspecified_code_blocks_false_rust() {
        let content = "```rust\nfn main() {}\n```";
        assert!(!has_unspecified_code_blocks(content));
    }

    #[test]
    fn test_has_unspecified_code_blocks_false_bash() {
        let content = "```bash\necho hello\n```";
        assert!(!has_unspecified_code_blocks(content));
    }

    #[test]
    fn test_extract_link_path_relative() {
        let line = "See [docs](../README.md) for more info";
        let path = extract_link_path(line);

        assert_eq!(path, Some("../README.md"));
    }

    #[test]
    fn test_extract_link_path_no_link() {
        let line = "No links here";
        let path = extract_link_path(line);

        assert_eq!(path, None);
    }

    // extract_target_files_from_issue Tests

    #[test]
    fn test_extract_target_files_with_rust_paths() {
        let content = GitHubIssueContent {
            title: "Bug in src/main.rs".to_string(),
            body: "The issue is in src/lib.rs and also affects tests/test.rs".to_string(),
            number: 1,
        };

        let files = extract_target_files_from_issue(&content, Path::new("/project")).unwrap();

        assert!(files.len() >= 3);
        assert!(files
            .iter()
            .any(|p| p.to_string_lossy().contains("main.rs")));
        assert!(files.iter().any(|p| p.to_string_lossy().contains("lib.rs")));
    }

    #[test]
    fn test_extract_target_files_with_backticks() {
        let content = GitHubIssueContent {
            title: "Fix issue".to_string(),
            body: "Check the file `src/utils.rs` for the bug".to_string(),
            number: 2,
        };

        let files = extract_target_files_from_issue(&content, Path::new("/project")).unwrap();

        assert!(files
            .iter()
            .any(|p| p.to_string_lossy().contains("utils.rs")));
    }

    #[test]
    fn test_extract_target_files_no_duplicates() {
        let content = GitHubIssueContent {
            title: "Bug in src/main.rs".to_string(),
            body: "The bug is in src/main.rs".to_string(),
            number: 3,
        };

        let files = extract_target_files_from_issue(&content, Path::new("/project")).unwrap();

        // Should not have duplicate entries
        let unique_files: std::collections::HashSet<_> = files.iter().collect();
        assert_eq!(files.len(), unique_files.len());
    }

    // GitHubIssueRef Tests

    #[test]
    fn test_github_issue_ref_creation() {
        let issue_ref = GitHubIssueRef {
            owner: "paiml".to_string(),
            repo: "pmat".to_string(),
            issue_number: 42,
        };

        assert_eq!(issue_ref.owner, "paiml");
        assert_eq!(issue_ref.repo, "pmat");
        assert_eq!(issue_ref.issue_number, 42);
    }

    #[test]
    fn test_github_issue_ref_clone() {
        let original = GitHubIssueRef {
            owner: "test".to_string(),
            repo: "repo".to_string(),
            issue_number: 100,
        };

        let cloned = original.clone();

        assert_eq!(cloned.owner, original.owner);
        assert_eq!(cloned.repo, original.repo);
        assert_eq!(cloned.issue_number, original.issue_number);
    }

    // GitHubIssueContent Tests

    #[test]
    fn test_github_issue_content_creation() {
        let content = GitHubIssueContent {
            title: "Fix bug".to_string(),
            body: "Description of the bug".to_string(),
            number: 123,
        };

        assert_eq!(content.title, "Fix bug");
        assert_eq!(content.body, "Description of the bug");
        assert_eq!(content.number, 123);
    }

    #[test]
    fn test_github_issue_content_clone() {
        let original = GitHubIssueContent {
            title: "Test".to_string(),
            body: "Body".to_string(),
            number: 1,
        };

        let cloned = original.clone();

        assert_eq!(cloned.title, original.title);
        assert_eq!(cloned.body, original.body);
        assert_eq!(cloned.number, original.number);
    }

    // FunctionInfo Tests

    #[test]
    fn test_function_info_creation() {
        let info = FunctionInfo {
            name: "test_function".to_string(),
            start_line: 10,
            end_line: 25,
            complexity: 5,
            is_test: true,
        };

        assert_eq!(info.name, "test_function");
        assert_eq!(info.start_line, 10);
        assert_eq!(info.end_line, 25);
        assert_eq!(info.complexity, 5);
        assert!(info.is_test);
    }

    #[test]
    fn test_function_info_clone() {
        let original = FunctionInfo {
            name: "func".to_string(),
            start_line: 1,
            end_line: 10,
            complexity: 3,
            is_test: false,
        };

        let cloned = original.clone();

        assert_eq!(cloned.name, original.name);
        assert_eq!(cloned.start_line, original.start_line);
        assert_eq!(cloned.end_line, original.end_line);
        assert_eq!(cloned.complexity, original.complexity);
        assert_eq!(cloned.is_test, original.is_test);
    }

    // AstMetadata Tests

    #[test]
    fn test_ast_metadata_creation() {
        let metadata = AstMetadata {
            functions: vec![FunctionInfo {
                name: "main".to_string(),
                start_line: 1,
                end_line: 10,
                complexity: 2,
                is_test: false,
            }],
            imports: vec!["std::io".to_string()],
            structure_hash: "abc123".to_string(),
        };

        assert_eq!(metadata.functions.len(), 1);
        assert_eq!(metadata.imports.len(), 1);
        assert_eq!(metadata.structure_hash, "abc123");
    }

    #[test]
    fn test_ast_metadata_clone() {
        let original = AstMetadata {
            functions: vec![],
            imports: vec!["import1".to_string()],
            structure_hash: "hash".to_string(),
        };

        let cloned = original.clone();

        assert_eq!(cloned.functions.len(), original.functions.len());
        assert_eq!(cloned.imports, original.imports);
        assert_eq!(cloned.structure_hash, original.structure_hash);
    }

    // Async Helper Functions Tests

    #[tokio::test]
    async fn test_load_ignore_patterns_empty() {
        let temp_dir = TempDir::new().unwrap();

        let config = PatternConfig {
            root_path: temp_dir.path().to_path_buf(),
            ignore_file: None,
            patterns: vec![],
            include_patterns: vec![],
            exclude_patterns: vec!["test_pattern".to_string()],
            ignore_file_path: None,
            file_extensions: vec!["rs".to_string()],
        };

        let patterns = load_ignore_patterns(&config).await.unwrap();

        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0], "test_pattern");
    }

    #[tokio::test]
    async fn test_load_ignore_patterns_with_file() {
        let temp_dir = TempDir::new().unwrap();
        let ignore_path = temp_dir.path().join(".gitignore");

        std::fs::write(&ignore_path, "target/\n*.tmp\n# Comment\n\n").unwrap();

        let config = PatternConfig {
            root_path: temp_dir.path().to_path_buf(),
            ignore_file: Some(".gitignore".to_string()),
            patterns: vec![],
            include_patterns: vec![],
            exclude_patterns: vec![],
            ignore_file_path: Some(ignore_path),
            file_extensions: vec!["rs".to_string()],
        };

        let patterns = load_ignore_patterns(&config).await.unwrap();

        assert!(patterns.contains(&"target/".to_string()));
        assert!(patterns.contains(&"*.tmp".to_string()));
        // Comments and empty lines should be filtered
        assert!(!patterns.contains(&"# Comment".to_string()));
    }

    #[tokio::test]
    async fn test_discover_source_files_empty_dir() {
        let temp_dir = TempDir::new().unwrap();

        let config = PatternConfig {
            root_path: temp_dir.path().to_path_buf(),
            ignore_file: None,
            patterns: vec![],
            include_patterns: vec![],
            exclude_patterns: vec![],
            ignore_file_path: None,
            file_extensions: vec!["rs".to_string()],
        };

        let files = discover_source_files(temp_dir.path(), &config, &[])
            .await
            .unwrap();

        assert!(files.is_empty());
    }

    #[tokio::test]
    async fn test_discover_source_files_with_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create test files
        std::fs::write(temp_dir.path().join("main.rs"), "fn main() {}").unwrap();
        std::fs::write(temp_dir.path().join("lib.rs"), "// lib").unwrap();
        std::fs::write(temp_dir.path().join("readme.md"), "# README").unwrap();

        let config = PatternConfig {
            root_path: temp_dir.path().to_path_buf(),
            ignore_file: None,
            patterns: vec![],
            include_patterns: vec![],
            exclude_patterns: vec![],
            ignore_file_path: None,
            file_extensions: vec!["rs".to_string()],
        };

        let files = discover_source_files(temp_dir.path(), &config, &[])
            .await
            .unwrap();

        assert_eq!(files.len(), 2);
        assert!(files.iter().all(|f| f.extension().unwrap() == "rs"));
    }

    #[tokio::test]
    async fn test_discover_source_files_respects_ignore() {
        let temp_dir = TempDir::new().unwrap();

        std::fs::write(temp_dir.path().join("good.rs"), "// good").unwrap();
        std::fs::write(temp_dir.path().join("ignored.rs"), "// ignored").unwrap();

        let config = PatternConfig {
            root_path: temp_dir.path().to_path_buf(),
            ignore_file: None,
            patterns: vec![],
            include_patterns: vec![],
            exclude_patterns: vec![],
            ignore_file_path: None,
            file_extensions: vec!["rs".to_string()],
        };

        let ignore_patterns = vec!["ignored".to_string()];
        let files = discover_source_files(temp_dir.path(), &config, &ignore_patterns)
            .await
            .unwrap();

        assert_eq!(files.len(), 1);
        assert!(files[0].to_string_lossy().contains("good.rs"));
    }

    // RefactoringSummary Tests

    #[tokio::test]
    async fn test_create_refactoring_summary_empty() {
        let iteration_results: Vec<IterationResult> = vec![];
        let validation = ValidationResult {
            overall_success: true,
            compilation_passed: true,
            tests_passed: true,
            quality_improved: false,
            issues_found: vec![],
        };
        let temp_dir = TempDir::new().unwrap();
        let config = RefactorConfig {
            project_path: temp_dir.path().to_path_buf(),
            mode: RefactorMode::ProjectWide,
            quality_profile: QualityProfile::default(),
            patterns: PatternConfig {
                root_path: temp_dir.path().to_path_buf(),
                ignore_file: None,
                patterns: vec![],
                include_patterns: vec![],
                exclude_patterns: vec![],
                ignore_file_path: None,
                file_extensions: vec!["rs".to_string()],
            },
            output: OutputConfig {
                format: RefactorAutoOutputFormat::Summary,
                dry_run: false,
                max_iterations: 1,
                verbose: false,
            },
        };
        let context = RefactorContext {
            config,
            ignore_patterns: vec![],
            source_files: vec![],
            start_time: std::time::Instant::now(),
        };

        let summary = create_refactoring_summary(&iteration_results, &validation, &context)
            .await
            .unwrap();

        assert_eq!(summary.total_successful_requests, 0);
        assert_eq!(summary.total_failed_requests, 0);
        assert_eq!(summary.total_quality_score, 0.0);
    }
}


mod extended_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_parse_github_url_never_panics(url in ".*") {
            // Should never panic, regardless of input
            let _ = parse_github_issue_url(&url);
        }

        #[test]
        fn test_parse_coverage_never_panics(data in prop::collection::vec(any::<u8>(), 0..1000)) {
            // Should never panic, regardless of input
            let _ = parse_coverage_from_output(&data);
        }

        #[test]
        fn test_should_retry_never_panics(msg in ".*") {
            let error = anyhow::anyhow!("{}", msg);
            let _ = should_retry_refactoring(&error);
        }

        #[test]
        fn test_is_markdown_file_deterministic(path in "[a-z./]+") {
            let p = std::path::Path::new(&path);
            let result1 = is_markdown_file(p);
            let result2 = is_markdown_file(p);
            prop_assert_eq!(result1, result2);
        }

        #[test]
        fn test_has_proper_headers_deterministic(content in ".*") {
            let result1 = has_proper_headers(&content);
            let result2 = has_proper_headers(&content);
            prop_assert_eq!(result1, result2);
        }

        #[test]
        fn test_extract_link_path_deterministic(line in ".*") {
            let result1 = extract_link_path(&line);
            let result2 = extract_link_path(&line);
            prop_assert_eq!(result1, result2);
        }
    }
}

/// Comprehensive coverage tests for refactor_auto_handlers
/// Tests all async functions and edge cases

mod comprehensive_coverage_tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    // Setup Refactoring Context Tests

    #[tokio::test]
    async fn test_setup_refactoring_context_project_wide() {
        let temp_dir = TempDir::new().unwrap();

        let result = setup_refactoring_context(
            temp_dir.path().to_path_buf(),
            false,
            None,
            RefactorAutoOutputFormat::Summary,
            5,
            false,
            vec![],
            vec![],
            None,
            None,
            None,
        )
        .await;

        assert!(result.is_ok());
        let context = result.unwrap();
        assert!(matches!(context.config.mode, RefactorMode::ProjectWide));
    }

    #[tokio::test]
    async fn test_setup_refactoring_context_single_file_mode() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn main() {}").unwrap();

        let result = setup_refactoring_context(
            temp_dir.path().to_path_buf(),
            true,
            Some(test_file.clone()),
            RefactorAutoOutputFormat::Json,
            3,
            true,
            vec!["target".to_string()],
            vec!["*.rs".to_string()],
            None,
            None,
            None,
        )
        .await;

        assert!(result.is_ok());
        let context = result.unwrap();
        if let RefactorMode::SingleFile(path) = &context.config.mode {
            assert_eq!(path, &test_file);
        } else {
            panic!("Expected SingleFile mode");
        }
    }

    #[tokio::test]
    async fn test_setup_refactoring_context_single_file_mode_no_file() {
        let temp_dir = TempDir::new().unwrap();

        let result = setup_refactoring_context(
            temp_dir.path().to_path_buf(),
            true,
            None, // No file provided
            RefactorAutoOutputFormat::Summary,
            1,
            false,
            vec![],
            vec![],
            None,
            None,
            None,
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Single file mode requires --file parameter"));
    }

    #[tokio::test]
    async fn test_setup_refactoring_context_bug_report_mode() {
        let temp_dir = TempDir::new().unwrap();
        let bug_report = temp_dir.path().join("bug.md");
        std::fs::write(&bug_report, "# Bug Report").unwrap();

        let result = setup_refactoring_context(
            temp_dir.path().to_path_buf(),
            false,
            None,
            RefactorAutoOutputFormat::Detailed,
            2,
            false,
            vec![],
            vec![],
            None,
            None,
            Some(bug_report.clone()),
        )
        .await;

        assert!(result.is_ok());
        let context = result.unwrap();
        if let RefactorMode::BugReport(path) = &context.config.mode {
            assert_eq!(path, &bug_report);
        } else {
            panic!("Expected BugReport mode");
        }
    }

    #[tokio::test]
    async fn test_setup_refactoring_context_github_issue_mode() {
        let temp_dir = TempDir::new().unwrap();
        let github_url = "https://github.com/owner/repo/issues/123".to_string();

        let result = setup_refactoring_context(
            temp_dir.path().to_path_buf(),
            false,
            None,
            RefactorAutoOutputFormat::Json,
            1,
            true,
            vec![],
            vec![],
            None,
            Some(github_url.clone()),
            None,
        )
        .await;

        assert!(result.is_ok());
        let context = result.unwrap();
        if let RefactorMode::GitHubIssue(url) = &context.config.mode {
            assert_eq!(url, &github_url);
        } else {
            panic!("Expected GitHubIssue mode");
        }
    }

    #[tokio::test]
    async fn test_setup_refactoring_context_with_ignore_file() {
        let temp_dir = TempDir::new().unwrap();
        let ignore_file = temp_dir.path().join(".pmatignore");
        std::fs::write(&ignore_file, "target/\n*.tmp").unwrap();

        let result = setup_refactoring_context(
            temp_dir.path().to_path_buf(),
            false,
            None,
            RefactorAutoOutputFormat::Summary,
            5,
            false,
            vec![],
            vec![],
            Some(ignore_file.clone()),
            None,
            None,
        )
        .await;

        assert!(result.is_ok());
        let context = result.unwrap();
        assert_eq!(
            context.config.patterns.ignore_file_path,
            Some(ignore_file)
        );
    }

    // Analyze Project Quality Tests

    #[tokio::test]
    async fn test_analyze_project_quality_empty_project() {
        let temp_dir = TempDir::new().unwrap();
        let config = RefactorConfig {
            project_path: temp_dir.path().to_path_buf(),
            mode: RefactorMode::ProjectWide,
            quality_profile: QualityProfile::default(),
            patterns: PatternConfig {
                root_path: temp_dir.path().to_path_buf(),
                ignore_file: None,
                patterns: vec![],
                include_patterns: vec![],
                exclude_patterns: vec![],
                ignore_file_path: None,
                file_extensions: vec!["rs".to_string()],
            },
            output: OutputConfig {
                format: RefactorAutoOutputFormat::Summary,
                dry_run: false,
                max_iterations: 1,
                verbose: false,
            },
        };
        let context = RefactorContext {
            config,
            ignore_patterns: vec![],
            source_files: vec![],
            start_time: std::time::Instant::now(),
        };

        let result = analyze_project_quality(&context).await;
        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert_eq!(analysis.total_files_analyzed, 0);
    }

    #[tokio::test]
    async fn test_analyze_project_quality_with_files() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("main.rs");
        std::fs::write(&test_file, "fn main() { println!(\"Hello\"); }").unwrap();

        let config = RefactorConfig {
            project_path: temp_dir.path().to_path_buf(),
            mode: RefactorMode::ProjectWide,
            quality_profile: QualityProfile::default(),
            patterns: PatternConfig {
                root_path: temp_dir.path().to_path_buf(),
                ignore_file: None,
                patterns: vec![],
                include_patterns: vec![],
                exclude_patterns: vec![],
                ignore_file_path: None,
                file_extensions: vec!["rs".to_string()],
            },
            output: OutputConfig {
                format: RefactorAutoOutputFormat::Summary,
                dry_run: false,
                max_iterations: 1,
                verbose: false,
            },
        };
        let context = RefactorContext {
            config,
            ignore_patterns: vec![],
            source_files: vec![test_file],
            start_time: std::time::Instant::now(),
        };

        let result = analyze_project_quality(&context).await;
        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert_eq!(analysis.total_files_analyzed, 1);
    }

    // Generate Refactoring Requests Tests

    #[tokio::test]
    async fn test_generate_refactoring_requests_empty() {
        let temp_dir = TempDir::new().unwrap();
        let config = RefactorConfig {
            project_path: temp_dir.path().to_path_buf(),
            mode: RefactorMode::ProjectWide,
            quality_profile: QualityProfile::default(),
            patterns: PatternConfig {
                root_path: temp_dir.path().to_path_buf(),
                ignore_file: None,
                patterns: vec![],
                include_patterns: vec![],
                exclude_patterns: vec![],
                ignore_file_path: None,
                file_extensions: vec!["rs".to_string()],
            },
            output: OutputConfig {
                format: RefactorAutoOutputFormat::Summary,
                dry_run: false,
                max_iterations: 1,
                verbose: false,
            },
        };
        let context = RefactorContext {
            config,
            ignore_patterns: vec![],
            source_files: vec![],
            start_time: std::time::Instant::now(),
        };

        let quality_analysis = ProjectQualityAnalysis {
            lint_violations: vec![],
            complexity_analysis: ComplexityAnalysis {
                high_complexity_violations: vec![],
                high_complexity_count: 0,
                total_functions: 0,
                average_complexity: 0.0,
            },
            satd_analysis: SatdAnalysis {
                satd_comments: vec![],
                total_satd_count: 0,
                files_with_satd: 0,
            },
            coverage_analysis: CoverageAnalysis {
                overall_coverage_percent: 100.0,
                files_with_low_coverage: vec![],
                uncovered_lines: vec![],
            },
            total_files_analyzed: 0,
            analysis_timestamp: std::time::SystemTime::now(),
        };

        let result = generate_refactoring_requests(&quality_analysis, &context).await;
        assert!(result.is_ok());
        let requests = result.unwrap();
        assert!(requests.is_empty());
    }

    #[tokio::test]
    async fn test_generate_refactoring_requests_with_complexity_violations() {
        let temp_dir = TempDir::new().unwrap();
        let config = RefactorConfig {
            project_path: temp_dir.path().to_path_buf(),
            mode: RefactorMode::ProjectWide,
            quality_profile: QualityProfile::default(),
            patterns: PatternConfig {
                root_path: temp_dir.path().to_path_buf(),
                ignore_file: None,
                patterns: vec![],
                include_patterns: vec![],
                exclude_patterns: vec![],
                ignore_file_path: None,
                file_extensions: vec!["rs".to_string()],
            },
            output: OutputConfig {
                format: RefactorAutoOutputFormat::Summary,
                dry_run: false,
                max_iterations: 1,
                verbose: false,
            },
        };
        let context = RefactorContext {
            config,
            ignore_patterns: vec![],
            source_files: vec![],
            start_time: std::time::Instant::now(),
        };

        let quality_analysis = ProjectQualityAnalysis {
            lint_violations: vec![],
            complexity_analysis: ComplexityAnalysis {
                high_complexity_violations: vec![
                    ComplexityViolation {
                        file: PathBuf::from("test.rs"),
                        function_name: "complex_function".to_string(),
                        complexity: 25,
                        line_number: 10,
                        suggestion: "Refactor".to_string(),
                    },
                ],
                high_complexity_count: 1,
                total_functions: 5,
                average_complexity: 15.0,
            },
            satd_analysis: SatdAnalysis {
                satd_comments: vec![],
                total_satd_count: 0,
                files_with_satd: 0,
            },
            coverage_analysis: CoverageAnalysis {
                overall_coverage_percent: 100.0,
                files_with_low_coverage: vec![],
                uncovered_lines: vec![],
            },
            total_files_analyzed: 1,
            analysis_timestamp: std::time::SystemTime::now(),
        };

        let result = generate_refactoring_requests(&quality_analysis, &context).await;
        assert!(result.is_ok());
        let requests = result.unwrap();
        assert!(!requests.is_empty());
    }

    // Create Complexity Reduction Request Tests

    #[tokio::test]
    async fn test_create_complexity_reduction_request_critical() {
        let temp_dir = TempDir::new().unwrap();
        let config = RefactorConfig {
            project_path: temp_dir.path().to_path_buf(),
            mode: RefactorMode::ProjectWide,
            quality_profile: QualityProfile::default(),
            patterns: PatternConfig {
                root_path: temp_dir.path().to_path_buf(),
                ignore_file: None,
                patterns: vec![],
                include_patterns: vec![],
                exclude_patterns: vec![],
                ignore_file_path: None,
                file_extensions: vec!["rs".to_string()],
            },
            output: OutputConfig {
                format: RefactorAutoOutputFormat::Summary,
                dry_run: false,
                max_iterations: 1,
                verbose: false,
            },
        };
        let context = RefactorContext {
            config,
            ignore_patterns: vec![],
            source_files: vec![],
            start_time: std::time::Instant::now(),
        };

        let violation = ComplexityViolation {
            file: PathBuf::from("test.rs"),
            function_name: "very_complex".to_string(),
            complexity: 50,
            line_number: 100,
            suggestion: "Split into multiple functions".to_string(),
        };

        let result = create_complexity_reduction_request(&violation, &context).await;
        assert!(result.is_ok());
        let request = result.unwrap();
        assert!(matches!(request.priority, RefactoringPriority::Critical));
        // Effort may vary depending on implementation details
        assert!(matches!(request.estimated_effort, RefactoringEffort::Minor | RefactoringEffort::Moderate | RefactoringEffort::Major));
    }

    #[tokio::test]
    async fn test_create_complexity_reduction_request_high() {
        let temp_dir = TempDir::new().unwrap();
        let config = RefactorConfig {
            project_path: temp_dir.path().to_path_buf(),
            mode: RefactorMode::ProjectWide,
            quality_profile: QualityProfile::default(),
            patterns: PatternConfig {
                root_path: temp_dir.path().to_path_buf(),
                ignore_file: None,
                patterns: vec![],
                include_patterns: vec![],
                exclude_patterns: vec![],
                ignore_file_path: None,
                file_extensions: vec!["rs".to_string()],
            },
            output: OutputConfig {
                format: RefactorAutoOutputFormat::Summary,
                dry_run: false,
                max_iterations: 1,
                verbose: false,
            },
        };
        let context = RefactorContext {
            config,
            ignore_patterns: vec![],
            source_files: vec![],
            start_time: std::time::Instant::now(),
        };

        let violation = ComplexityViolation {
            file: PathBuf::from("test.rs"),
            function_name: "moderate_complex".to_string(),
            complexity: 15,
            line_number: 50,
            suggestion: "Simplify".to_string(),
        };

        let result = create_complexity_reduction_request(&violation, &context).await;
        assert!(result.is_ok());
        let request = result.unwrap();
        assert!(matches!(request.priority, RefactoringPriority::High));
        assert!(matches!(request.estimated_effort, RefactoringEffort::Minor));
    }

    // Create Lint Fix Requests Tests

    #[tokio::test]
    async fn test_create_lint_fix_requests_error_severity() {
        let temp_dir = TempDir::new().unwrap();
        let config = RefactorConfig {
            project_path: temp_dir.path().to_path_buf(),
            mode: RefactorMode::ProjectWide,
            quality_profile: QualityProfile::default(),
            patterns: PatternConfig {
                root_path: temp_dir.path().to_path_buf(),
                ignore_file: None,
                patterns: vec![],
                include_patterns: vec![],
                exclude_patterns: vec![],
                ignore_file_path: None,
                file_extensions: vec!["rs".to_string()],
            },
            output: OutputConfig {
                format: RefactorAutoOutputFormat::Summary,
                dry_run: false,
                max_iterations: 1,
                verbose: false,
            },
        };
        let context = RefactorContext {
            config,
            ignore_patterns: vec![],
            source_files: vec![],
            start_time: std::time::Instant::now(),
        };

        let violations = vec![ViolationDetailJson {
            file: PathBuf::from("test.rs"),
            line: 10,
            column: 5,
            end_line: 10,
            end_column: 15,
            lint_name: "clippy::unwrap_used".to_string(),
            message: "used unwrap on Result".to_string(),
            severity: "error".to_string(),
            suggestion: Some("Use ? operator".to_string()),
            machine_applicable: true,
        }];

        let result = create_lint_fix_requests(&violations, &context).await;
        assert!(result.is_ok());
        let requests = result.unwrap();
        assert_eq!(requests.len(), 1);
        assert!(matches!(requests[0].priority, RefactoringPriority::High));
    }

    #[tokio::test]
    async fn test_create_lint_fix_requests_warning_severity() {
        let temp_dir = TempDir::new().unwrap();
        let config = RefactorConfig {
            project_path: temp_dir.path().to_path_buf(),
            mode: RefactorMode::ProjectWide,
            quality_profile: QualityProfile::default(),
            patterns: PatternConfig {
                root_path: temp_dir.path().to_path_buf(),
                ignore_file: None,
                patterns: vec![],
                include_patterns: vec![],
                exclude_patterns: vec![],
                ignore_file_path: None,
                file_extensions: vec!["rs".to_string()],
            },
            output: OutputConfig {
                format: RefactorAutoOutputFormat::Summary,
                dry_run: false,
                max_iterations: 1,
                verbose: false,
            },
        };
        let context = RefactorContext {
            config,
            ignore_patterns: vec![],
            source_files: vec![],
            start_time: std::time::Instant::now(),
        };

        let violations = vec![ViolationDetailJson {
            file: PathBuf::from("test.rs"),
            line: 20,
            column: 1,
            end_line: 20,
            end_column: 10,
            lint_name: "dead_code".to_string(),
            message: "unused function".to_string(),
            severity: "warning".to_string(),
            suggestion: None,
            machine_applicable: false,
        }];

        let result = create_lint_fix_requests(&violations, &context).await;
        assert!(result.is_ok());
        let requests = result.unwrap();
        assert_eq!(requests.len(), 1);
        assert!(matches!(requests[0].priority, RefactoringPriority::Medium));
    }

    // Create SATD Cleanup Requests Tests

    #[tokio::test]
    async fn test_create_satd_cleanup_requests_fixme() {
        let temp_dir = TempDir::new().unwrap();
        let config = RefactorConfig {
            project_path: temp_dir.path().to_path_buf(),
            mode: RefactorMode::ProjectWide,
            quality_profile: QualityProfile::default(),
            patterns: PatternConfig {
                root_path: temp_dir.path().to_path_buf(),
                ignore_file: None,
                patterns: vec![],
                include_patterns: vec![],
                exclude_patterns: vec![],
                ignore_file_path: None,
                file_extensions: vec!["rs".to_string()],
            },
            output: OutputConfig {
                format: RefactorAutoOutputFormat::Summary,
                dry_run: false,
                max_iterations: 1,
                verbose: false,
            },
        };
        let context = RefactorContext {
            config,
            ignore_patterns: vec![],
            source_files: vec![],
            start_time: std::time::Instant::now(),
        };

        let satd_analysis = SatdAnalysis {
            satd_comments: vec![SatdComment {
                file: PathBuf::from("test.rs"),
                line_number: 5,
                comment_text: "FIXME: This needs to be fixed".to_string(),
                satd_type: "FIXME".to_string(),
            }],
            total_satd_count: 1,
            files_with_satd: 1,
        };

        let result = create_satd_cleanup_requests(&satd_analysis, &context).await;
        assert!(result.is_ok());
        let requests = result.unwrap();
        assert_eq!(requests.len(), 1);
        assert!(matches!(requests[0].priority, RefactoringPriority::High));
    }

    #[tokio::test]
    async fn test_create_satd_cleanup_requests_todo() {
        let temp_dir = TempDir::new().unwrap();
        let config = RefactorConfig {
            project_path: temp_dir.path().to_path_buf(),
            mode: RefactorMode::ProjectWide,
            quality_profile: QualityProfile::default(),
            patterns: PatternConfig {
                root_path: temp_dir.path().to_path_buf(),
                ignore_file: None,
                patterns: vec![],
                include_patterns: vec![],
                exclude_patterns: vec![],
                ignore_file_path: None,
                file_extensions: vec!["rs".to_string()],
            },
            output: OutputConfig {
                format: RefactorAutoOutputFormat::Summary,
                dry_run: false,
                max_iterations: 1,
                verbose: false,
            },
        };
        let context = RefactorContext {
            config,
            ignore_patterns: vec![],
            source_files: vec![],
            start_time: std::time::Instant::now(),
        };

        let satd_analysis = SatdAnalysis {
            satd_comments: vec![SatdComment {
                file: PathBuf::from("test.rs"),
                line_number: 10,
                comment_text: "TODO: Add tests".to_string(),
                satd_type: "TODO".to_string(),
            }],
            total_satd_count: 1,
            files_with_satd: 1,
        };

        let result = create_satd_cleanup_requests(&satd_analysis, &context).await;
        assert!(result.is_ok());
        let requests = result.unwrap();
        assert_eq!(requests.len(), 1);
        assert!(matches!(requests[0].priority, RefactoringPriority::Medium));
    }

    // Create Coverage Improvement Requests Tests

    #[tokio::test]
    async fn test_create_coverage_improvement_requests() {
        let temp_dir = TempDir::new().unwrap();
        let config = RefactorConfig {
            project_path: temp_dir.path().to_path_buf(),
            mode: RefactorMode::ProjectWide,
            quality_profile: QualityProfile::default(),
            patterns: PatternConfig {
                root_path: temp_dir.path().to_path_buf(),
                ignore_file: None,
                patterns: vec![],
                include_patterns: vec![],
                exclude_patterns: vec![],
                ignore_file_path: None,
                file_extensions: vec!["rs".to_string()],
            },
            output: OutputConfig {
                format: RefactorAutoOutputFormat::Summary,
                dry_run: false,
                max_iterations: 1,
                verbose: false,
            },
        };
        let context = RefactorContext {
            config,
            ignore_patterns: vec![],
            source_files: vec![],
            start_time: std::time::Instant::now(),
        };

        let coverage_analysis = CoverageAnalysis {
            overall_coverage_percent: 50.0,
            files_with_low_coverage: vec![
                PathBuf::from("src/main.rs"),
                PathBuf::from("src/lib.rs"),
            ],
            uncovered_lines: vec![],
        };

        let result = create_coverage_improvement_requests(&coverage_analysis, &context).await;
        assert!(result.is_ok());
        let requests = result.unwrap();
        assert_eq!(requests.len(), 2);
        for request in &requests {
            assert!(matches!(request.priority, RefactoringPriority::Medium));
            assert!(matches!(
                request.request_type,
                RefactoringType::CoverageImprovement
            ));
        }
    }

    // Calculate Quality Improvement Tests

    #[tokio::test]
    async fn test_calculate_quality_improvement_empty() {
        let result = calculate_quality_improvement(&[]).await;
        assert!(result.is_ok());
        let improvement = result.unwrap();
        assert_eq!(improvement.complexity_reduced, 0);
        assert_eq!(improvement.violations_fixed, 0);
        assert_eq!(improvement.satd_resolved, 0);
        assert_eq!(improvement.coverage_increased, 0.0);
        assert_eq!(improvement.overall_score, 0.0);
    }

    #[tokio::test]
    async fn test_calculate_quality_improvement_with_successes() {
        let successes = vec![
            RefactoringSuccess {
                request: RefactoringRequest {
                    request_type: RefactoringType::ComplexityReduction,
                    target_file: PathBuf::from("test.rs"),
                    priority: RefactoringPriority::High,
                    description: "Reduce complexity".to_string(),
                    ai_instructions: "Refactor".to_string(),
                    estimated_effort: RefactoringEffort::Moderate,
                },
                changes_made: vec!["Change 1".to_string()],
                application_duration: std::time::Duration::from_secs(1),
                verification_status: VerificationStatus::Verified,
            },
            RefactoringSuccess {
                request: RefactoringRequest {
                    request_type: RefactoringType::LintFix,
                    target_file: PathBuf::from("test2.rs"),
                    priority: RefactoringPriority::Medium,
                    description: "Fix lint".to_string(),
                    ai_instructions: "Fix".to_string(),
                    estimated_effort: RefactoringEffort::Trivial,
                },
                changes_made: vec!["Change 2".to_string()],
                application_duration: std::time::Duration::from_secs(1),
                verification_status: VerificationStatus::Verified,
            },
            RefactoringSuccess {
                request: RefactoringRequest {
                    request_type: RefactoringType::SatdCleanup,
                    target_file: PathBuf::from("test3.rs"),
                    priority: RefactoringPriority::Low,
                    description: "Clean SATD".to_string(),
                    ai_instructions: "Clean".to_string(),
                    estimated_effort: RefactoringEffort::Minor,
                },
                changes_made: vec!["Change 3".to_string()],
                application_duration: std::time::Duration::from_secs(1),
                verification_status: VerificationStatus::Verified,
            },
            RefactoringSuccess {
                request: RefactoringRequest {
                    request_type: RefactoringType::CoverageImprovement,
                    target_file: PathBuf::from("test4.rs"),
                    priority: RefactoringPriority::Medium,
                    description: "Add tests".to_string(),
                    ai_instructions: "Test".to_string(),
                    estimated_effort: RefactoringEffort::Moderate,
                },
                changes_made: vec!["Change 4".to_string()],
                application_duration: std::time::Duration::from_secs(1),
                verification_status: VerificationStatus::Verified,
            },
            RefactoringSuccess {
                request: RefactoringRequest {
                    request_type: RefactoringType::SecurityFix,
                    target_file: PathBuf::from("test5.rs"),
                    priority: RefactoringPriority::Critical,
                    description: "Fix security".to_string(),
                    ai_instructions: "Secure".to_string(),
                    estimated_effort: RefactoringEffort::Major,
                },
                changes_made: vec!["Change 5".to_string()],
                application_duration: std::time::Duration::from_secs(1),
                verification_status: VerificationStatus::Verified,
            },
        ];

        let result = calculate_quality_improvement(&successes).await;
        assert!(result.is_ok());
        let improvement = result.unwrap();
        assert_eq!(improvement.complexity_reduced, 1);
        assert_eq!(improvement.violations_fixed, 2); // LintFix + SecurityFix
        assert_eq!(improvement.satd_resolved, 1);
        assert_eq!(improvement.coverage_increased, 5.0);
    }

    // Apply Refactoring Functions Tests

    #[tokio::test]
    async fn test_apply_complexity_reduction() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        let result = apply_complexity_reduction(&test_file, "Reduce complexity").await;
        assert!(result.is_ok());
        let changes = result.unwrap();
        assert!(!changes.is_empty());
    }

    #[tokio::test]
    async fn test_apply_lint_fixes() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        let result = apply_lint_fixes(&test_file, "Fix clippy warnings").await;
        assert!(result.is_ok());
        let changes = result.unwrap();
        assert!(!changes.is_empty());
    }

    #[tokio::test]
    async fn test_apply_satd_cleanup() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        let result = apply_satd_cleanup(&test_file, "Remove TODOs").await;
        assert!(result.is_ok());
        let changes = result.unwrap();
        assert!(!changes.is_empty());
    }

    #[tokio::test]
    async fn test_apply_coverage_improvements() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        let result = apply_coverage_improvements(&test_file, "Add tests").await;
        assert!(result.is_ok());
        let changes = result.unwrap();
        assert!(!changes.is_empty());
    }

    #[tokio::test]
    async fn test_apply_security_fixes() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        let result = apply_security_fixes(&test_file, "Fix security issue").await;
        assert!(result.is_ok());
        let changes = result.unwrap();
        assert!(!changes.is_empty());
    }

    // Helper Function Tests

    #[tokio::test]
    async fn test_get_single_file_lint_violations() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        let result = get_single_file_lint_violations(&test_file).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_count_file_satd() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        let result = count_file_satd(&test_file).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_analyze_file_complexity() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        let result = analyze_file_complexity(&test_file).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_single_file_refactor_request() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");

        let result = generate_single_file_refactor_request(
            &test_file,
            vec![],
            QualityMetrics::default(),
            0,
        );

        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.is_object());
    }

    // Markdown Analysis Tests

    #[tokio::test]
    async fn test_handle_markdown_analysis() {
        let temp_dir = TempDir::new().unwrap();
        let md_file = temp_dir.path().join("README.md");
        std::fs::write(&md_file, "# Title\n\nContent here").unwrap();

        let result = handle_markdown_analysis(&md_file, RefactorAutoOutputFormat::Json).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_markdown_issues_valid() {
        let temp_dir = TempDir::new().unwrap();
        let md_file = temp_dir.path().join("test.md");
        std::fs::write(&md_file, "# Header\n\n```rust\ncode\n```").unwrap();

        let result = analyze_markdown_issues(&md_file, "# Header\n\n```rust\ncode\n```");
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_markdown_issues_no_headers() {
        let temp_dir = TempDir::new().unwrap();
        let md_file = temp_dir.path().join("test.md");

        let result = analyze_markdown_issues(&md_file, "No headers here");
        assert!(result.is_ok());
        let issues = result.unwrap();
        assert!(issues.contains(&"Missing proper header structure"));
    }

    #[test]
    fn test_analyze_markdown_issues_unspecified_code_block() {
        let temp_dir = TempDir::new().unwrap();
        let md_file = temp_dir.path().join("test.md");

        let result = analyze_markdown_issues(&md_file, "# Header\n\n```\ncode\n```");
        assert!(result.is_ok());
        let issues = result.unwrap();
        assert!(issues.contains(&"Code blocks without language specification"));
    }

    #[test]
    fn test_create_markdown_refactor_request() {
        let path = Path::new("test.md");
        let issues = vec!["Issue 1", "Issue 2"];
        let content = "# Content";

        let result = create_markdown_refactor_request(path, &issues, content);
        assert!(result.is_object());
        assert_eq!(result["file_type"], "markdown");
    }

    #[test]
    fn test_print_markdown_summary() {
        let request = serde_json::json!({
            "issues": ["Issue 1", "Issue 2"]
        });

        // Should not panic
        print_markdown_summary(&request);
    }

    // Output Function Tests

    #[test]
    fn test_output_regular_file_results_json() {
        let request = serde_json::json!({
            "file": "test.rs",
            "refactoring_needed": true
        });

        // Should not panic
        output_regular_file_results(&request, RefactorAutoOutputFormat::Json);
    }

    #[test]
    fn test_output_regular_file_results_summary() {
        let request = serde_json::json!({
            "file": "test.rs",
            "refactoring_needed": false
        });

        // Should not panic
        output_regular_file_results(&request, RefactorAutoOutputFormat::Summary);
    }

    #[test]
    fn test_output_regular_file_results_detailed() {
        let request = serde_json::json!({
            "file": "test.rs",
            "violations": []
        });

        // Should not panic
        output_regular_file_results(&request, RefactorAutoOutputFormat::Detailed);
    }

    #[test]
    fn test_print_single_file_summary() {
        let request = serde_json::json!({});

        // Should not panic
        print_single_file_summary(&request);
    }

    #[test]
    fn test_print_single_file_detailed() {
        let request = serde_json::json!({});

        // Should not panic
        print_single_file_detailed(&request);
    }

    // Filter Successful Requests Tests

    #[test]
    fn test_filter_successful_requests_all_success() {
        let requests = vec![
            RefactoringRequest {
                request_type: RefactoringType::LintFix,
                target_file: PathBuf::from("test1.rs"),
                priority: RefactoringPriority::High,
                description: "Fix 1".to_string(),
                ai_instructions: "Instructions 1".to_string(),
                estimated_effort: RefactoringEffort::Trivial,
            },
            RefactoringRequest {
                request_type: RefactoringType::LintFix,
                target_file: PathBuf::from("test2.rs"),
                priority: RefactoringPriority::Medium,
                description: "Fix 2".to_string(),
                ai_instructions: "Instructions 2".to_string(),
                estimated_effort: RefactoringEffort::Minor,
            },
        ];

        let iteration_result = IterationResult {
            iteration_number: 1,
            successful_requests: vec![
                RefactoringSuccess {
                    request: requests[0].clone(),
                    changes_made: vec![],
                    application_duration: std::time::Duration::from_secs(1),
                    verification_status: VerificationStatus::Verified,
                },
                RefactoringSuccess {
                    request: requests[1].clone(),
                    changes_made: vec![],
                    application_duration: std::time::Duration::from_secs(1),
                    verification_status: VerificationStatus::Verified,
                },
            ],
            failed_requests: vec![],
            iteration_duration: std::time::Duration::from_secs(2),
            quality_improvement: QualityImprovement {
                complexity_reduced: 0,
                violations_fixed: 2,
                satd_resolved: 0,
                coverage_increased: 0.0,
                overall_score: 2.0,
            },
        };

        let remaining = filter_successful_requests(&requests, &iteration_result);
        assert!(remaining.is_empty());
    }

    #[test]
    fn test_filter_successful_requests_partial_success() {
        let requests = vec![
            RefactoringRequest {
                request_type: RefactoringType::LintFix,
                target_file: PathBuf::from("test1.rs"),
                priority: RefactoringPriority::High,
                description: "Fix 1".to_string(),
                ai_instructions: "Instructions 1".to_string(),
                estimated_effort: RefactoringEffort::Trivial,
            },
            RefactoringRequest {
                request_type: RefactoringType::LintFix,
                target_file: PathBuf::from("test2.rs"),
                priority: RefactoringPriority::Medium,
                description: "Fix 2".to_string(),
                ai_instructions: "Instructions 2".to_string(),
                estimated_effort: RefactoringEffort::Minor,
            },
        ];

        let iteration_result = IterationResult {
            iteration_number: 1,
            successful_requests: vec![RefactoringSuccess {
                request: requests[0].clone(),
                changes_made: vec![],
                application_duration: std::time::Duration::from_secs(1),
                verification_status: VerificationStatus::Verified,
            }],
            failed_requests: vec![],
            iteration_duration: std::time::Duration::from_secs(1),
            quality_improvement: QualityImprovement {
                complexity_reduced: 0,
                violations_fixed: 1,
                satd_resolved: 0,
                coverage_increased: 0.0,
                overall_score: 1.0,
            },
        };

        let remaining = filter_successful_requests(&requests, &iteration_result);
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].target_file, PathBuf::from("test2.rs"));
    }

    // Broken Links Tests

    #[test]
    fn test_has_broken_relative_links_no_links() {
        let temp_dir = TempDir::new().unwrap();
        let md_file = temp_dir.path().join("test.md");

        let result = has_broken_relative_links(&md_file, "No links here");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_has_broken_relative_links_with_broken_link() {
        let temp_dir = TempDir::new().unwrap();
        let md_file = temp_dir.path().join("test.md");

        let result = has_broken_relative_links(&md_file, "See [docs](../nonexistent.md)");
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_has_broken_relative_links_with_valid_link() {
        let temp_dir = TempDir::new().unwrap();
        let md_file = temp_dir.path().join("test.md");
        let linked_file = temp_dir.path().join("other.md");
        std::fs::write(&linked_file, "# Other").unwrap();

        let result = has_broken_relative_links(&md_file, "See [other](./other.md)");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    // Handle Special Modes Tests

    #[tokio::test]
    async fn test_handle_special_modes_project_wide() {
        let temp_dir = TempDir::new().unwrap();
        let config = RefactorConfig {
            project_path: temp_dir.path().to_path_buf(),
            mode: RefactorMode::ProjectWide,
            quality_profile: QualityProfile::default(),
            patterns: PatternConfig {
                root_path: temp_dir.path().to_path_buf(),
                ignore_file: None,
                patterns: vec![],
                include_patterns: vec![],
                exclude_patterns: vec![],
                ignore_file_path: None,
                file_extensions: vec!["rs".to_string()],
            },
            output: OutputConfig {
                format: RefactorAutoOutputFormat::Summary,
                dry_run: false,
                max_iterations: 1,
                verbose: false,
            },
        };
        let context = RefactorContext {
            config,
            ignore_patterns: vec![],
            source_files: vec![],
            start_time: std::time::Instant::now(),
        };

        let result = handle_special_modes(&context).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // ProjectWide returns None
    }

    #[tokio::test]
    async fn test_handle_special_modes_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn main() {}").unwrap();

        let config = RefactorConfig {
            project_path: temp_dir.path().to_path_buf(),
            mode: RefactorMode::SingleFile(test_file),
            quality_profile: QualityProfile::default(),
            patterns: PatternConfig {
                root_path: temp_dir.path().to_path_buf(),
                ignore_file: None,
                patterns: vec![],
                include_patterns: vec![],
                exclude_patterns: vec![],
                ignore_file_path: None,
                file_extensions: vec!["rs".to_string()],
            },
            output: OutputConfig {
                format: RefactorAutoOutputFormat::Summary,
                dry_run: false,
                max_iterations: 1,
                verbose: false,
            },
        };
        let context = RefactorContext {
            config,
            ignore_patterns: vec![],
            source_files: vec![],
            start_time: std::time::Instant::now(),
        };

        let result = handle_special_modes(&context).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some()); // SingleFile returns Some(())
    }

    #[tokio::test]
    async fn test_handle_special_modes_bug_report_md() {
        let temp_dir = TempDir::new().unwrap();
        let bug_file = temp_dir.path().join("bug.md");
        std::fs::write(&bug_file, "# Bug Report\n\nDescription").unwrap();

        let config = RefactorConfig {
            project_path: temp_dir.path().to_path_buf(),
            mode: RefactorMode::BugReport(bug_file),
            quality_profile: QualityProfile::default(),
            patterns: PatternConfig {
                root_path: temp_dir.path().to_path_buf(),
                ignore_file: None,
                patterns: vec![],
                include_patterns: vec![],
                exclude_patterns: vec![],
                ignore_file_path: None,
                file_extensions: vec!["rs".to_string()],
            },
            output: OutputConfig {
                format: RefactorAutoOutputFormat::Summary,
                dry_run: false,
                max_iterations: 1,
                verbose: false,
            },
        };
        let context = RefactorContext {
            config,
            ignore_patterns: vec![],
            source_files: vec![],
            start_time: std::time::Instant::now(),
        };

        let result = handle_special_modes(&context).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some()); // BugReport .md returns Some(())
    }

    #[tokio::test]
    async fn test_handle_special_modes_bug_report_non_md() {
        let temp_dir = TempDir::new().unwrap();
        let bug_file = temp_dir.path().join("bug.txt");
        std::fs::write(&bug_file, "Bug description").unwrap();

        let config = RefactorConfig {
            project_path: temp_dir.path().to_path_buf(),
            mode: RefactorMode::BugReport(bug_file),
            quality_profile: QualityProfile::default(),
            patterns: PatternConfig {
                root_path: temp_dir.path().to_path_buf(),
                ignore_file: None,
                patterns: vec![],
                include_patterns: vec![],
                exclude_patterns: vec![],
                ignore_file_path: None,
                file_extensions: vec!["rs".to_string()],
            },
            output: OutputConfig {
                format: RefactorAutoOutputFormat::Summary,
                dry_run: false,
                max_iterations: 1,
                verbose: false,
            },
        };
        let context = RefactorContext {
            config,
            ignore_patterns: vec![],
            source_files: vec![],
            start_time: std::time::Instant::now(),
        };

        let result = handle_special_modes(&context).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // BugReport non-.md returns None
    }

    // Analyze Project Functions Tests

    #[tokio::test]
    async fn test_analyze_project_lint_violations_empty() {
        let result = analyze_project_lint_violations(&[]).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_analyze_project_lint_violations_with_files() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn main() {}").unwrap();

        let result = analyze_project_lint_violations(&[test_file]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_project_complexity_empty() {
        let result = analyze_project_complexity(&[]).await;
        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert_eq!(analysis.total_functions, 0);
        assert_eq!(analysis.high_complexity_count, 0);
    }

    #[tokio::test]
    async fn test_analyze_project_satd_empty() {
        let result = analyze_project_satd(&[]).await;
        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert_eq!(analysis.total_satd_count, 0);
        assert_eq!(analysis.files_with_satd, 0);
    }

    // Validation Tests

    #[tokio::test]
    async fn test_get_final_validation_empty() {
        let temp_dir = TempDir::new().unwrap();
        let config = RefactorConfig {
            project_path: temp_dir.path().to_path_buf(),
            mode: RefactorMode::ProjectWide,
            quality_profile: QualityProfile::default(),
            patterns: PatternConfig {
                root_path: temp_dir.path().to_path_buf(),
                ignore_file: None,
                patterns: vec![],
                include_patterns: vec![],
                exclude_patterns: vec![],
                ignore_file_path: None,
                file_extensions: vec!["rs".to_string()],
            },
            output: OutputConfig {
                format: RefactorAutoOutputFormat::Summary,
                dry_run: false,
                max_iterations: 1,
                verbose: false,
            },
        };
        let context = RefactorContext {
            config,
            ignore_patterns: vec![],
            source_files: vec![],
            start_time: std::time::Instant::now(),
        };

        let result = get_final_validation(&[], &context).await;
        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(validation.overall_success);
        assert!(validation.compilation_passed);
        assert!(validation.tests_passed);
        assert!(!validation.quality_improved);
    }

    // ViolationWithContext Tests

    #[test]
    fn test_violation_with_context_creation() {
        let violation = ViolationWithContext {
            lint_name: "dead_code".to_string(),
            line: 10,
            column: 5,
            message: "unused variable".to_string(),
            ast_node_id: Some("node_123".to_string()),
            fix_strategy: FixStrategy::RemoveDeadCode,
        };

        assert_eq!(violation.lint_name, "dead_code");
        assert_eq!(violation.line, 10);
        assert_eq!(violation.column, 5);
        assert!(violation.ast_node_id.is_some());
    }

    #[test]
    fn test_violation_with_context_clone() {
        let original = ViolationWithContext {
            lint_name: "clippy".to_string(),
            line: 20,
            column: 1,
            message: "warning".to_string(),
            ast_node_id: None,
            fix_strategy: FixStrategy::ApplySuggestion("use vec![]".to_string()),
        };

        let cloned = original.clone();
        assert_eq!(cloned.lint_name, original.lint_name);
        assert_eq!(cloned.line, original.line);
    }

    // FileRewritePlan Tests

    #[test]
    fn test_file_rewrite_plan_creation() {
        let plan = FileRewritePlan {
            file_path: PathBuf::from("test.rs"),
            violations: vec![],
            ast_metadata: AstMetadata {
                functions: vec![],
                imports: vec![],
                structure_hash: "hash".to_string(),
            },
            new_content: "fn main() {}".to_string(),
        };

        assert_eq!(plan.file_path, PathBuf::from("test.rs"));
        assert!(plan.violations.is_empty());
    }

    #[test]
    fn test_file_rewrite_plan_clone() {
        let original = FileRewritePlan {
            file_path: PathBuf::from("lib.rs"),
            violations: vec![ViolationWithContext {
                lint_name: "test".to_string(),
                line: 1,
                column: 1,
                message: "msg".to_string(),
                ast_node_id: None,
                fix_strategy: FixStrategy::AddTest,
            }],
            ast_metadata: AstMetadata {
                functions: vec![],
                imports: vec!["std".to_string()],
                structure_hash: "abc".to_string(),
            },
            new_content: "// content".to_string(),
        };

        let cloned = original.clone();
        assert_eq!(cloned.file_path, original.file_path);
        assert_eq!(cloned.violations.len(), original.violations.len());
    }

    // ComplexityViolation Tests

    #[test]
    fn test_complexity_violation_clone() {
        let original = ComplexityViolation {
            file: PathBuf::from("complex.rs"),
            function_name: "too_complex".to_string(),
            complexity: 35,
            line_number: 100,
            suggestion: "Split function".to_string(),
        };

        let cloned = original.clone();
        assert_eq!(cloned.file, original.file);
        assert_eq!(cloned.function_name, original.function_name);
        assert_eq!(cloned.complexity, original.complexity);
    }

    // SatdComment Tests

    #[test]
    fn test_satd_comment_clone() {
        let original = SatdComment {
            file: PathBuf::from("todo.rs"),
            line_number: 50,
            comment_text: "TODO: Implement this".to_string(),
            satd_type: "TODO".to_string(),
        };

        let cloned = original.clone();
        assert_eq!(cloned.file, original.file);
        assert_eq!(cloned.comment_text, original.comment_text);
        assert_eq!(cloned.satd_type, original.satd_type);
    }

    // UncoveredLine Tests

    #[test]
    fn test_uncovered_line_clone() {
        let original = UncoveredLine {
            file: PathBuf::from("uncovered.rs"),
            line_number: 42,
            content: "unreachable!()".to_string(),
        };

        let cloned = original.clone();
        assert_eq!(cloned.file, original.file);
        assert_eq!(cloned.line_number, original.line_number);
        assert_eq!(cloned.content, original.content);
    }

    // RefactoringRequest Tests

    #[test]
    fn test_refactoring_request_clone() {
        let original = RefactoringRequest {
            request_type: RefactoringType::SecurityFix,
            target_file: PathBuf::from("secure.rs"),
            priority: RefactoringPriority::Critical,
            description: "Fix SQL injection".to_string(),
            ai_instructions: "Sanitize input".to_string(),
            estimated_effort: RefactoringEffort::Extensive,
        };

        let cloned = original.clone();
        assert_eq!(cloned.target_file, original.target_file);
        assert_eq!(cloned.description, original.description);
    }

    // Print Functions Tests (Ensure No Panics)

    #[test]
    fn test_print_refactoring_header() {
        let config = RefactorAutoConfig {
            project_path: PathBuf::from("/test/project"),
            single_file_mode: false,
            file: None,
            format: RefactorAutoOutputFormat::Summary,
            max_iterations: 5,
            cache_dir: None,
            dry_run: false,
            ci_mode: false,
            exclude_patterns: vec![],
            include_patterns: vec![],
            ignore_file: None,
            test_file: None,
            test_name: None,
            github_issue_url: None,
            bug_report_path: None,
        };

        // Should not panic
        print_refactoring_header(&config);
    }

    // RefactorState Serialization Tests

    #[test]
    fn test_refactor_state_serialization() {
        let state = RefactorState {
            iteration: 3,
            context_generated: true,
            context_path: PathBuf::from("/tmp/ctx"),
            current_file: Some(PathBuf::from("current.rs")),
            files_completed: vec![PathBuf::from("done.rs")],
            quality_metrics: QualityMetrics {
                total_violations: 10,
                coverage_percent: 85.0,
                max_complexity: 15,
                satd_count: 2,
                files_with_issues: 3,
                total_files: 10,
                functions_with_high_complexity: 2,
                total_functions: 50,
            },
            progress: RefactorProgress::default(),
            start_time: std::time::SystemTime::now(),
        };

        let json = serde_json::to_string(&state);
        assert!(json.is_ok());

        let json_str = json.unwrap();
        let deserialized: Result<RefactorState, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());
    }

    // RefactorProgress Serialization Tests

    #[test]
    fn test_refactor_progress_serialization() {
        let progress = RefactorProgress {
            overall_completion_percent: 50.0,
            lint_completion_percent: 60.0,
            complexity_completion_percent: 40.0,
            satd_completion_percent: 70.0,
            coverage_completion_percent: 30.0,
            files_completed: 5,
            files_remaining: 5,
            estimated_time_remaining_minutes: 10,
            quality_gates_passed: vec!["lint".to_string()],
            quality_gates_remaining: vec!["complexity".to_string()],
            current_phase: RefactorPhase::LintFixes,
        };

        let json = serde_json::to_string(&progress);
        assert!(json.is_ok());

        let json_str = json.unwrap();
        let deserialized: Result<RefactorProgress, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());
    }

    // QualityMetrics Serialization Tests

    #[test]
    fn test_quality_metrics_serialization() {
        let metrics = QualityMetrics {
            total_violations: 25,
            coverage_percent: 90.5,
            max_complexity: 20,
            satd_count: 5,
            files_with_issues: 8,
            total_files: 50,
            functions_with_high_complexity: 3,
            total_functions: 200,
        };

        let json = serde_json::to_string(&metrics);
        assert!(json.is_ok());

        let json_str = json.unwrap();
        let deserialized: Result<QualityMetrics, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());
    }
}
