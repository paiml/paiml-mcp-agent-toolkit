//! Coverage tests part 2 for tdg handlers - Additional async handler tests
//! Extracted for file health compliance (CB-040)

use super::super::*;
use proptest::prelude::*;
use std::path::PathBuf;
use tempfile::TempDir;

// ========== Additional Coverage Tests for Async Handlers ==========

mod execute_tdg_analysis_tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_analysis_on_file() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("test.rs");
        std::fs::write(
            &rust_file,
            r#"
/// Simple function.
pub fn simple_function() {
    println!("hello");
}
"#,
        )
        .unwrap();

        let tdg_config = TdgConfig::default();
        let analyzer = TdgAnalyzer::with_storage(tdg_config).unwrap();
        let config = TdgCommandConfig {
            path: rust_file,
            command: None,
            format: TdgOutputFormat::Table,
            config: None,
            quiet: false,
            include_components: false,
            min_grade: None,
            output: None,
            with_git_context: false,
            explain: false,
            threshold: 10,
            baseline: None,
            viz: false,
            viz_theme: "default".to_string(),
        };

        let result = execute_tdg_analysis(&analyzer, &config).await;
        assert!(result.is_ok());
        let score = result.unwrap();
        assert!(score.total >= 0.0);
        assert!(score.total <= 100.0);
    }

    #[tokio::test]
    async fn test_execute_analysis_on_directory() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("lib.rs");
        std::fs::write(
            &rust_file,
            r#"
/// Hello.
pub fn hello() -> &'static str {
    "hello"
}
"#,
        )
        .unwrap();

        let tdg_config = TdgConfig::default();
        let analyzer = TdgAnalyzer::with_storage(tdg_config).unwrap();
        let config = TdgCommandConfig {
            path: temp_dir.path().to_path_buf(),
            command: None,
            format: TdgOutputFormat::Table,
            config: None,
            quiet: false,
            include_components: false,
            min_grade: None,
            output: None,
            with_git_context: false,
            explain: false,
            threshold: 10,
            baseline: None,
            viz: false,
            viz_theme: "default".to_string(),
        };

        let result = execute_tdg_analysis(&analyzer, &config).await;
        assert!(result.is_ok());
    }
}

mod handle_tdg_command_tests {
    use super::*;

    #[tokio::test]
    async fn test_handle_tdg_command_basic_file() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("main.rs");
        std::fs::write(
            &rust_file,
            r#"
fn main() {
    println!("Hello, world!");
}
"#,
        )
        .unwrap();

        let config = TdgCommandConfig {
            path: rust_file,
            command: None,
            format: TdgOutputFormat::Table,
            config: None,
            quiet: true,
            include_components: false,
            min_grade: None,
            output: None,
            with_git_context: false,
            explain: false,
            threshold: 10,
            baseline: None,
            viz: false,
            viz_theme: "default".to_string(),
        };

        let result = handle_tdg_command(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_tdg_command_with_output_file() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("lib.rs");
        std::fs::write(&rust_file, "pub fn foo() {}").unwrap();
        let output_file = temp_dir.path().join("output.txt");

        let config = TdgCommandConfig {
            path: rust_file,
            command: None,
            format: TdgOutputFormat::Table,
            config: None,
            quiet: false,
            include_components: true,
            min_grade: None,
            output: Some(output_file.clone()),
            with_git_context: false,
            explain: false,
            threshold: 10,
            baseline: None,
            viz: false,
            viz_theme: "default".to_string(),
        };

        let result = handle_tdg_command(config).await;
        assert!(result.is_ok());
        assert!(output_file.exists());
    }

    #[tokio::test]
    async fn test_handle_tdg_command_quiet_mode() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("quiet_test.rs");
        std::fs::write(&rust_file, "fn test() {}").unwrap();

        let config = TdgCommandConfig {
            path: rust_file,
            command: None,
            format: TdgOutputFormat::Table,
            config: None,
            quiet: true,
            include_components: false,
            min_grade: None,
            output: None,
            with_git_context: false,
            explain: false,
            threshold: 10,
            baseline: None,
            viz: false,
            viz_theme: "default".to_string(),
        };

        let result = handle_tdg_command(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_tdg_command_json_format() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("json_test.rs");
        std::fs::write(&rust_file, "pub fn json_fn() { let x = 1; }").unwrap();
        let output_file = temp_dir.path().join("output.json");

        let config = TdgCommandConfig {
            path: rust_file,
            command: None,
            format: TdgOutputFormat::Json,
            config: None,
            quiet: false,
            include_components: true,
            min_grade: None,
            output: Some(output_file.clone()),
            with_git_context: false,
            explain: false,
            threshold: 10,
            baseline: None,
            viz: false,
            viz_theme: "default".to_string(),
        };

        let result = handle_tdg_command(config).await;
        assert!(result.is_ok());
        assert!(output_file.exists());
        let content = std::fs::read_to_string(&output_file).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(parsed.get("score").is_some());
    }

    #[tokio::test]
    async fn test_handle_tdg_command_markdown_format() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("md_test.rs");
        std::fs::write(&rust_file, "pub fn md_fn() {}").unwrap();
        let output_file = temp_dir.path().join("output.md");

        let config = TdgCommandConfig {
            path: rust_file,
            command: None,
            format: TdgOutputFormat::Markdown,
            config: None,
            quiet: false,
            include_components: false,
            min_grade: None,
            output: Some(output_file.clone()),
            with_git_context: false,
            explain: false,
            threshold: 10,
            baseline: None,
            viz: false,
            viz_theme: "default".to_string(),
        };

        let result = handle_tdg_command(config).await;
        assert!(result.is_ok());
        assert!(output_file.exists());
        let content = std::fs::read_to_string(&output_file).unwrap();
        assert!(content.contains("# TDG Score Report"));
    }

    #[tokio::test]
    async fn test_handle_tdg_command_sarif_format() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("sarif_test.rs");
        std::fs::write(&rust_file, "pub fn sarif_fn() {}").unwrap();
        let output_file = temp_dir.path().join("output.sarif");

        let config = TdgCommandConfig {
            path: rust_file,
            command: None,
            format: TdgOutputFormat::Sarif,
            config: None,
            quiet: false,
            include_components: false,
            min_grade: None,
            output: Some(output_file.clone()),
            with_git_context: false,
            explain: false,
            threshold: 10,
            baseline: None,
            viz: false,
            viz_theme: "default".to_string(),
        };

        let result = handle_tdg_command(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_tdg_command_min_grade_passing() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("grade_pass.rs");
        // Simple file should get a good score
        std::fs::write(&rust_file, "pub fn simple() -> i32 { 42 }").unwrap();

        let config = TdgCommandConfig {
            path: rust_file,
            command: None,
            format: TdgOutputFormat::Table,
            config: None,
            quiet: true,
            include_components: false,
            min_grade: Some("F".to_string()), // Very low bar
            output: None,
            with_git_context: false,
            explain: false,
            threshold: 10,
            baseline: None,
            viz: false,
            viz_theme: "default".to_string(),
        };

        let result = handle_tdg_command(config).await;
        assert!(result.is_ok());
    }
}

mod format_explain_output_tests {
    use super::*;
    use crate::tdg::explain::{
        ActionableRecommendation, ComplexitySeverity, ExplainedTDGScore, FunctionComplexity,
        RecommendationType,
    };

    fn make_explained_score() -> ExplainedTDGScore {
        let score = crate::tdg::TdgScore {
            total: 75.0,
            grade: Grade::C,
            confidence: 0.9,
            language: crate::tdg::Language::Rust,
            structural_complexity: 15.0,
            semantic_complexity: 12.0,
            duplication_ratio: 8.0,
            coupling_score: 10.0,
            doc_coverage: 5.0,
            consistency_score: 5.0,
            entropy_score: 20.0,
            file_path: Some(PathBuf::from("test.rs")),
            penalties_applied: vec![],
            critical_defects_count: 0,
            has_critical_defects: false,
        };

        let mut explained = ExplainedTDGScore::new(score);

        // Add some functions
        explained.add_function(FunctionComplexity {
            name: "complex_function".to_string(),
            line_number: 10,
            cyclomatic: 25,
            cognitive: 30,
            tdg_impact: 3.5,
            severity: ComplexitySeverity::Critical,
        });

        explained.add_function(FunctionComplexity {
            name: "medium_function".to_string(),
            line_number: 50,
            cyclomatic: 8,
            cognitive: 10,
            tdg_impact: 1.2,
            severity: ComplexitySeverity::Medium,
        });

        // Add a recommendation
        explained.add_recommendation(ActionableRecommendation {
            rec_type: RecommendationType::ExtractFunction,
            target_function: Some("complex_function".to_string()),
            action: "Extract nested loops into separate helper functions".to_string(),
            expected_impact: 5.0,
            effort_hours: 2.0,
            priority: 1,
        });

        explained
    }

    #[test]
    fn test_format_explain_json() {
        let explained = make_explained_score();
        let config = TdgCommandConfig {
            path: PathBuf::from("test.rs"),
            command: None,
            format: TdgOutputFormat::Json,
            config: None,
            quiet: false,
            include_components: false,
            min_grade: None,
            output: None,
            with_git_context: false,
            explain: true,
            threshold: 5,
            baseline: None,
            viz: false,
            viz_theme: "default".to_string(),
        };

        let result = format_explain_output(&explained, &config).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed.get("score").is_some());
        assert!(parsed.get("functions").is_some());
    }

    #[test]
    fn test_format_explain_markdown() {
        let explained = make_explained_score();
        let config = TdgCommandConfig {
            path: PathBuf::from("test.rs"),
            command: None,
            format: TdgOutputFormat::Markdown,
            config: None,
            quiet: false,
            include_components: false,
            min_grade: None,
            output: None,
            with_git_context: false,
            explain: true,
            threshold: 5,
            baseline: None,
            viz: false,
            viz_theme: "default".to_string(),
        };

        let result = format_explain_output(&explained, &config).unwrap();
        assert!(result.contains("```json"));
    }

    #[test]
    fn test_format_explain_table() {
        let explained = make_explained_score();
        let config = TdgCommandConfig {
            path: PathBuf::from("test.rs"),
            command: None,
            format: TdgOutputFormat::Table,
            config: None,
            quiet: false,
            include_components: false,
            min_grade: None,
            output: None,
            with_git_context: false,
            explain: true,
            threshold: 5,
            baseline: None,
            viz: false,
            viz_theme: "default".to_string(),
        };

        let result = format_explain_output(&explained, &config).unwrap();
        assert!(result.contains("TDG Explain Report"));
        assert!(result.contains("complex_function"));
        assert!(result.contains("Recommendations"));
    }

    #[test]
    fn test_format_explain_empty_functions() {
        let score = crate::tdg::TdgScore {
            total: 95.0,
            grade: Grade::A,
            confidence: 0.95,
            language: crate::tdg::Language::Rust,
            structural_complexity: 23.0,
            semantic_complexity: 18.0,
            duplication_ratio: 2.0,
            coupling_score: 5.0,
            doc_coverage: 9.0,
            consistency_score: 8.0,
            entropy_score: 30.0,
            file_path: None,
            penalties_applied: vec![],
            critical_defects_count: 0,
            has_critical_defects: false,
        };
        let explained = ExplainedTDGScore::new(score);
        let config = TdgCommandConfig {
            path: PathBuf::from("clean.rs"),
            command: None,
            format: TdgOutputFormat::Table,
            config: None,
            quiet: false,
            include_components: false,
            min_grade: None,
            output: None,
            with_git_context: false,
            explain: true,
            threshold: 10,
            baseline: None,
            viz: false,
            viz_theme: "default".to_string(),
        };

        let result = format_explain_output(&explained, &config).unwrap();
        assert!(result.contains("No functions above complexity threshold"));
    }
}

mod display_gate_result_tests {
    use super::*;
    use crate::tdg::{GateResult, Severity, Violation, ViolationType};

    #[test]
    fn test_display_gate_result_passed() {
        let result = GateResult {
            passed: true,
            gate_name: "RegressionGate".to_string(),
            violations: vec![],
            message: "All quality checks passed".to_string(),
        };

        // Just verify it doesn't panic
        display_gate_result_table(&result);
    }

    #[test]
    fn test_display_gate_result_with_violations() {
        let result = GateResult {
            passed: false,
            gate_name: "MinimumGradeGate".to_string(),
            violations: vec![
                Violation {
                    path: PathBuf::from("bad_file.rs"),
                    violation_type: ViolationType::BelowMinimum,
                    severity: Severity::Error,
                    message: "Grade C is below minimum B".to_string(),
                    old_score: None,
                    new_score: 72.0,
                    old_grade: None,
                    new_grade: Grade::C,
                },
                Violation {
                    path: PathBuf::from("regression.rs"),
                    violation_type: ViolationType::Regression,
                    severity: Severity::Critical,
                    message: "Score dropped by 15 points".to_string(),
                    old_score: Some(85.0),
                    new_score: 70.0,
                    old_grade: Some(Grade::B),
                    new_grade: Grade::C,
                },
            ],
            message: "2 violations found".to_string(),
        };

        // Just verify it doesn't panic
        display_gate_result_table(&result);
    }
}

mod handle_explain_mode_tests {
    use super::*;

    #[tokio::test]
    async fn test_handle_explain_mode_basic() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("explain_test.rs");
        std::fs::write(
            &rust_file,
            r#"
/// Simple function.
pub fn simple_function() -> i32 {
    let x = 1;
    let y = 2;
    x + y
}

/// Complex function.
pub fn complex_function(n: i32) -> i32 {
    if n > 0 {
        if n > 10 {
            if n > 100 {
                n * 3
            } else {
                n * 2
            }
        } else {
            n + 1
        }
    } else {
        match n {
            -1 => 0,
            -2 => 1,
            _ => n.abs(),
        }
    }
}
"#,
        )
        .unwrap();

        let tdg_config = TdgConfig::default();
        let analyzer = TdgAnalyzer::with_storage(tdg_config).unwrap();
        let config = TdgCommandConfig {
            path: rust_file,
            command: None,
            format: TdgOutputFormat::Table,
            config: None,
            quiet: false,
            include_components: false,
            min_grade: None,
            output: None,
            with_git_context: false,
            explain: true,
            threshold: 3,
            baseline: None,
            viz: false,
            viz_theme: "default".to_string(),
        };

        let result = handle_explain_mode(&analyzer, &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_explain_mode_json_output() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("explain_json.rs");
        std::fs::write(&rust_file, "pub fn test() { println!(\"test\"); }").unwrap();
        let output_file = temp_dir.path().join("explain.json");

        let tdg_config = TdgConfig::default();
        let analyzer = TdgAnalyzer::with_storage(tdg_config).unwrap();
        let config = TdgCommandConfig {
            path: rust_file,
            command: None,
            format: TdgOutputFormat::Json,
            config: None,
            quiet: false,
            include_components: false,
            min_grade: None,
            output: Some(output_file.clone()),
            with_git_context: false,
            explain: true,
            threshold: 1,
            baseline: None,
            viz: false,
            viz_theme: "default".to_string(),
        };

        let result = handle_explain_mode(&analyzer, &config).await;
        assert!(result.is_ok());
        assert!(output_file.exists());
    }

    #[tokio::test]
    async fn test_handle_explain_mode_high_threshold() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("simple.rs");
        std::fs::write(&rust_file, "pub fn simple() {}").unwrap();

        let tdg_config = TdgConfig::default();
        let analyzer = TdgAnalyzer::with_storage(tdg_config).unwrap();
        let config = TdgCommandConfig {
            path: rust_file,
            command: None,
            format: TdgOutputFormat::Table,
            config: None,
            quiet: false,
            include_components: false,
            min_grade: None,
            output: None,
            with_git_context: false,
            explain: true,
            threshold: 100, // Very high threshold
            baseline: None,
            viz: false,
            viz_theme: "default".to_string(),
        };

        let result = handle_explain_mode(&analyzer, &config).await;
        assert!(result.is_ok());
    }
}

mod is_analyzable_comprehensive_tests {
    use super::*;

    #[test]
    fn test_all_supported_extensions() {
        let extensions = [
            "rs", "py", "js", "ts", "tsx", "jsx", "java", "c", "cpp", "h", "hpp", "go", "rb",
            "php", "swift", "kt", "kts",
        ];

        for ext in extensions {
            let path = format!("file.{}", ext);
            assert!(
                is_analyzable_file(Path::new(&path)),
                "Expected {} to be analyzable",
                path
            );
        }
    }

    #[test]
    fn test_unsupported_extensions() {
        let extensions = [
            "txt", "md", "json", "yaml", "yml", "toml", "xml", "html", "css", "scss", "sql", "sh",
            "bash", "zsh", "fish", "ps1", "bat", "cmd",
        ];

        for ext in extensions {
            let path = format!("file.{}", ext);
            assert!(
                !is_analyzable_file(Path::new(&path)),
                "Expected {} to NOT be analyzable",
                path
            );
        }
    }
}

mod tdg_score_with_file_path_tests {
    use super::*;

    #[test]
    fn test_format_table_with_file_path() {
        let score = crate::tdg::TdgScore {
            total: 88.0,
            grade: Grade::BPlus,
            confidence: 0.92,
            language: crate::tdg::Language::Rust,
            structural_complexity: 22.0,
            semantic_complexity: 17.0,
            duplication_ratio: 3.0,
            coupling_score: 8.0,
            doc_coverage: 9.0,
            consistency_score: 9.0,
            entropy_score: 20.0,
            file_path: Some(PathBuf::from("src/handlers/tdg.rs")),
            penalties_applied: vec![],
            critical_defects_count: 0,
            has_critical_defects: false,
        };

        let result = format_tdg_score(score, None, TdgOutputFormat::Table, false).unwrap();
        assert!(result.contains("src/handlers/tdg.rs"));
    }

    #[test]
    fn test_format_json_with_file_path() {
        let score = crate::tdg::TdgScore {
            total: 88.0,
            grade: Grade::BPlus,
            confidence: 0.92,
            language: crate::tdg::Language::Rust,
            structural_complexity: 22.0,
            semantic_complexity: 17.0,
            duplication_ratio: 3.0,
            coupling_score: 8.0,
            doc_coverage: 9.0,
            consistency_score: 9.0,
            entropy_score: 20.0,
            file_path: Some(PathBuf::from("src/handlers/tdg.rs")),
            penalties_applied: vec![],
            critical_defects_count: 0,
            has_critical_defects: false,
        };

        let result = format_tdg_score(score, None, TdgOutputFormat::Json, false).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed["file"].as_str().unwrap().contains("tdg.rs"));
    }

    #[test]
    fn test_format_markdown_with_file_path() {
        let score = crate::tdg::TdgScore {
            total: 88.0,
            grade: Grade::BPlus,
            confidence: 0.92,
            language: crate::tdg::Language::Rust,
            structural_complexity: 22.0,
            semantic_complexity: 17.0,
            duplication_ratio: 3.0,
            coupling_score: 8.0,
            doc_coverage: 9.0,
            consistency_score: 9.0,
            entropy_score: 20.0,
            file_path: Some(PathBuf::from("src/handlers/tdg.rs")),
            penalties_applied: vec![],
            critical_defects_count: 0,
            has_critical_defects: false,
        };

        let result = format_tdg_score(score, None, TdgOutputFormat::Markdown, false).unwrap();
        assert!(result.contains("**File**: `src/handlers/tdg.rs`"));
    }

    #[test]
    fn test_format_markdown_with_components() {
        let score = crate::tdg::TdgScore {
            total: 70.0,
            grade: Grade::CMinus,
            confidence: 0.85,
            language: crate::tdg::Language::Python,
            structural_complexity: 12.0,
            semantic_complexity: 10.0,
            duplication_ratio: 10.0,
            coupling_score: 12.0,
            doc_coverage: 3.0,
            consistency_score: 3.0,
            entropy_score: 20.0,
            file_path: None,
            penalties_applied: vec![],
            critical_defects_count: 0,
            has_critical_defects: false,
        };

        let result = format_tdg_score(score, None, TdgOutputFormat::Markdown, true).unwrap();
        assert!(result.contains("## Component Breakdown"));
        assert!(result.contains("Structural Complexity"));
        assert!(result.contains("| Component | Score | Max |"));
    }

    #[test]
    fn test_format_json_with_components() {
        let score = crate::tdg::TdgScore {
            total: 70.0,
            grade: Grade::CMinus,
            confidence: 0.85,
            language: crate::tdg::Language::Python,
            structural_complexity: 12.0,
            semantic_complexity: 10.0,
            duplication_ratio: 10.0,
            coupling_score: 12.0,
            doc_coverage: 3.0,
            consistency_score: 3.0,
            entropy_score: 20.0,
            file_path: None,
            penalties_applied: vec![],
            critical_defects_count: 0,
            has_critical_defects: false,
        };

        let result = format_tdg_score(score, None, TdgOutputFormat::Json, true).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed["score"]["breakdown"].is_object());
        assert_eq!(parsed["score"]["breakdown"]["structural_complexity"], 12.0);
    }

    #[test]
    fn test_format_json_without_components() {
        let score = crate::tdg::TdgScore {
            total: 70.0,
            grade: Grade::CMinus,
            confidence: 0.85,
            language: crate::tdg::Language::Python,
            structural_complexity: 12.0,
            semantic_complexity: 10.0,
            duplication_ratio: 10.0,
            coupling_score: 12.0,
            doc_coverage: 3.0,
            consistency_score: 3.0,
            entropy_score: 20.0,
            file_path: None,
            penalties_applied: vec![],
            critical_defects_count: 0,
            has_critical_defects: false,
        };

        let result = format_tdg_score(score, None, TdgOutputFormat::Json, false).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed["score"]["breakdown"].is_null());
    }
}

mod git_context_output_tests {
    use super::*;

    fn make_git_context() -> crate::models::git_context::GitContext {
        crate::models::git_context::GitContext {
            commit_sha: "1234567890abcdef".to_string(),
            commit_sha_short: "1234567".to_string(),
            branch: "feature/test".to_string(),
            author_name: "Test Author".to_string(),
            author_email: "test@example.com".to_string(),
            commit_timestamp: chrono::Utc::now(),
            commit_message: "Test commit message".to_string(),
            tags: vec!["v1.0.0".to_string()],
            parent_commits: vec!["parent123".to_string()],
            remote_url: Some("https://github.com/test/repo".to_string()),
            is_clean: true,
            uncommitted_files: 0,
        }
    }

    #[test]
    fn test_json_output_with_full_git_context() {
        let score = crate::tdg::TdgScore {
            total: 80.0,
            grade: Grade::B,
            confidence: 0.9,
            language: crate::tdg::Language::Rust,
            structural_complexity: 18.0,
            semantic_complexity: 14.0,
            duplication_ratio: 5.0,
            coupling_score: 8.0,
            doc_coverage: 7.0,
            consistency_score: 8.0,
            entropy_score: 20.0,
            file_path: None,
            penalties_applied: vec![],
            critical_defects_count: 0,
            has_critical_defects: false,
        };
        let git = make_git_context();

        let result = format_tdg_score(score, Some(&git), TdgOutputFormat::Json, false).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["git_context"]["commit_sha"], "1234567890abcdef");
        assert_eq!(parsed["git_context"]["branch"], "feature/test");
        assert_eq!(parsed["git_context"]["is_clean"], true);
        assert!(parsed["git_context"]["tags"].is_array());
    }

    #[test]
    fn test_table_output_with_git_context() {
        let score = crate::tdg::TdgScore {
            total: 80.0,
            grade: Grade::B,
            confidence: 0.9,
            language: crate::tdg::Language::Rust,
            structural_complexity: 18.0,
            semantic_complexity: 14.0,
            duplication_ratio: 5.0,
            coupling_score: 8.0,
            doc_coverage: 7.0,
            consistency_score: 8.0,
            entropy_score: 20.0,
            file_path: None,
            penalties_applied: vec![],
            critical_defects_count: 0,
            has_critical_defects: false,
        };
        let git = make_git_context();

        let result = format_tdg_score(score, Some(&git), TdgOutputFormat::Table, false).unwrap();
        assert!(result.contains("Git Context"));
        assert!(result.contains("1234567"));
        assert!(result.contains("feature/test"));
    }
}

mod multiple_records_history_tests {
    use super::*;
    use crate::tdg::storage::{ComponentScores, FileIdentity, FullTdgRecord};

    fn make_record(path: &str, total: f32, commit_sha: &str) -> FullTdgRecord {
        FullTdgRecord {
            identity: FileIdentity {
                path: PathBuf::from(path),
                content_hash: blake3::hash(path.as_bytes()),
                size_bytes: 1024,
                modified_time: std::time::SystemTime::now(),
            },
            score: crate::tdg::TdgScore {
                total,
                grade: if total >= 80.0 { Grade::B } else { Grade::C },
                confidence: 0.9,
                language: crate::tdg::Language::Rust,
                structural_complexity: 15.0,
                semantic_complexity: 12.0,
                duplication_ratio: 8.0,
                coupling_score: 10.0,
                doc_coverage: 5.0,
                consistency_score: 5.0,
                entropy_score: total - 55.0,
                file_path: Some(PathBuf::from(path)),
                penalties_applied: vec![],
                critical_defects_count: 0,
                has_critical_defects: false,
            },
            components: ComponentScores::default(),
            semantic_sig: crate::tdg::storage::SemanticSignature {
                ast_structure_hash: 12345,
                identifier_pattern: "test".to_string(),
                control_flow_pattern: "linear".to_string(),
                import_dependencies: vec![],
            },
            metadata: crate::tdg::storage::AnalysisMetadata {
                analyzer_version: "1.0.0".to_string(),
                analysis_duration_ms: 100,
                language_confidence: 0.95,
                analysis_timestamp: std::time::SystemTime::now(),
                cache_hit: false,
            },
            git_context: Some(crate::models::git_context::GitContext {
                commit_sha: commit_sha.to_string(),
                commit_sha_short: commit_sha[..7].to_string(),
                branch: "main".to_string(),
                author_name: "Developer".to_string(),
                author_email: "dev@test.com".to_string(),
                commit_timestamp: chrono::Utc::now(),
                commit_message: "Update".to_string(),
                tags: vec![],
                parent_commits: vec![],
                remote_url: None,
                is_clean: true,
                uncommitted_files: 0,
            }),
        }
    }

    #[test]
    fn test_multiple_records_table_format() {
        let records = vec![
            make_record("src/lib.rs", 85.0, "abc1234567890"),
            make_record("src/main.rs", 75.0, "def4567890abc"),
            make_record("src/utils.rs", 90.0, "ghi7890abcdef"),
        ];

        let result = format_history_output(&records, TdgOutputFormat::Table).unwrap();
        assert!(result.contains("abc1234"));
        assert!(result.contains("def4567"));
        assert!(result.contains("ghi7890"));
    }

    #[test]
    fn test_multiple_records_json_format() {
        let records = vec![
            make_record("src/lib.rs", 85.0, "abc1234567890"),
            make_record("src/main.rs", 75.0, "def4567890abc"),
        ];

        let result = format_history_output(&records, TdgOutputFormat::Json).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["total_records"], 2);
        assert_eq!(parsed["history"].as_array().unwrap().len(), 2);
    }
}

mod comparison_json_detailed_tests {
    use super::*;

    #[test]
    fn test_comparison_json_all_fields() {
        let comparison = crate::tdg::Comparison {
            source1: crate::tdg::TdgScore {
                total: 60.0,
                grade: Grade::D,
                confidence: 0.8,
                language: crate::tdg::Language::Rust,
                structural_complexity: 10.0,
                semantic_complexity: 8.0,
                duplication_ratio: 12.0,
                coupling_score: 10.0,
                doc_coverage: 2.0,
                consistency_score: 3.0,
                entropy_score: 15.0,
                file_path: None,
                penalties_applied: vec![],
                critical_defects_count: 0,
                has_critical_defects: false,
            },
            source2: crate::tdg::TdgScore {
                total: 90.0,
                grade: Grade::A,
                confidence: 0.98,
                language: crate::tdg::Language::Rust,
                structural_complexity: 23.0,
                semantic_complexity: 18.0,
                duplication_ratio: 2.0,
                coupling_score: 5.0,
                doc_coverage: 10.0,
                consistency_score: 10.0,
                entropy_score: 22.0,
                file_path: None,
                penalties_applied: vec![],
                critical_defects_count: 0,
                has_critical_defects: false,
            },
            delta: 30.0,
            improvement_percentage: 50.0,
            winner: "source2".to_string(),
            improvements: vec![
                "duplication".to_string(),
                "coupling".to_string(),
                "documentation".to_string(),
            ],
            regressions: vec![],
        };

        let result = format_comparison(comparison, TdgOutputFormat::Json).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["source1"]["total"], 60.0);
        assert_eq!(parsed["source2"]["total"], 90.0);
        assert_eq!(parsed["difference"], 30.0);
        assert_eq!(parsed["winner"], "source2");
    }
}

mod config_loading_edge_cases {
    use super::*;

    #[test]
    fn test_config_with_invalid_toml() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("invalid.toml");
        std::fs::write(&config_path, "this is not valid toml [[[").unwrap();

        let config = TdgCommandConfig {
            path: temp_dir.path().to_path_buf(),
            command: None,
            format: TdgOutputFormat::Table,
            config: Some(config_path),
            quiet: false,
            include_components: false,
            min_grade: None,
            output: None,
            with_git_context: false,
            explain: false,
            threshold: 10,
            baseline: None,
            viz: false,
            viz_theme: "default".to_string(),
        };

        let result = load_tdg_configuration(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_with_empty_toml() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("empty.toml");
        std::fs::write(&config_path, "").unwrap();

        let config = TdgCommandConfig {
            path: temp_dir.path().to_path_buf(),
            command: None,
            format: TdgOutputFormat::Table,
            config: Some(config_path),
            quiet: false,
            include_components: false,
            min_grade: None,
            output: None,
            with_git_context: false,
            explain: false,
            threshold: 10,
            baseline: None,
            viz: false,
            viz_theme: "default".to_string(),
        };

        let result = load_tdg_configuration(&config);
        // Empty TOML should be valid and use defaults
        assert!(result.is_ok());
    }
}

mod grade_validation_tests {
    use super::*;
}
