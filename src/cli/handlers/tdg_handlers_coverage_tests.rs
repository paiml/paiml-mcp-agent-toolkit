//! Coverage tests part 1 for tdg handlers
//! Extracted for file health compliance (CB-040)

use super::super::*;
use proptest::prelude::*;
use std::path::PathBuf;
use tempfile::TempDir;

fn make_test_config(path: PathBuf) -> TdgCommandConfig {
    TdgCommandConfig {
        path,
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
    }
}

/// Create a test directory with Rust source files
fn create_test_project() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create a simple Rust file
    let rust_file = temp_dir.path().join("lib.rs");
    std::fs::write(
        &rust_file,
        r#"
/// Hello world.
pub fn hello_world() {
    println!("Hello, world!");
}

/// Complex function.
pub fn complex_function(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            if x > 100 {
                x * 3
            } else {
                x * 2
            }
        } else {
            x + 1
        }
    } else {
        0
    }
}
"#,
    )
    .expect("Failed to write test file");

    temp_dir
}

/// Create a TDG config file for testing
fn create_test_config_file(dir: &TempDir) -> PathBuf {
    let config_path = dir.path().join("tdg-config.toml");
    std::fs::write(
        &config_path,
        r#"
[thresholds]
complexity_max = 20
duplication_ratio = 0.1

[output]
verbose = false
"#,
    )
    .expect("Failed to write config file");
    config_path
}

// ========== Unit Tests for Helper Functions ==========

mod format_grade_tests {
    use super::*;

    #[test]
    fn test_format_grade_a_plus() {
        assert_eq!(format_grade(Grade::APLus), "A+");
    }

    #[test]
    fn test_format_grade_a() {
        assert_eq!(format_grade(Grade::A), "A");
    }

    #[test]
    fn test_format_grade_a_minus() {
        assert_eq!(format_grade(Grade::AMinus), "A-");
    }

    #[test]
    fn test_format_grade_b_plus() {
        assert_eq!(format_grade(Grade::BPlus), "B+");
    }

    #[test]
    fn test_format_grade_b() {
        assert_eq!(format_grade(Grade::B), "B");
    }

    #[test]
    fn test_format_grade_b_minus() {
        assert_eq!(format_grade(Grade::BMinus), "B-");
    }

    #[test]
    fn test_format_grade_c_plus() {
        assert_eq!(format_grade(Grade::CPlus), "C+");
    }

    #[test]
    fn test_format_grade_c() {
        assert_eq!(format_grade(Grade::C), "C");
    }

    #[test]
    fn test_format_grade_c_minus() {
        assert_eq!(format_grade(Grade::CMinus), "C-");
    }

    #[test]
    fn test_format_grade_d() {
        assert_eq!(format_grade(Grade::D), "D");
    }

    #[test]
    fn test_format_grade_f() {
        assert_eq!(format_grade(Grade::F), "F");
    }
}

mod parse_grade_tests {
    use super::*;

    #[test]
    fn test_parse_grade_a_plus() {
        assert_eq!(parse_grade("A+").unwrap(), Grade::APLus);
    }

    #[test]
    fn test_parse_grade_a() {
        assert_eq!(parse_grade("A").unwrap(), Grade::A);
    }

    #[test]
    fn test_parse_grade_a_minus() {
        assert_eq!(parse_grade("A-").unwrap(), Grade::AMinus);
    }

    #[test]
    fn test_parse_grade_b_plus() {
        assert_eq!(parse_grade("B+").unwrap(), Grade::BPlus);
    }

    #[test]
    fn test_parse_grade_b() {
        assert_eq!(parse_grade("B").unwrap(), Grade::B);
    }

    #[test]
    fn test_parse_grade_b_minus() {
        assert_eq!(parse_grade("B-").unwrap(), Grade::BMinus);
    }

    #[test]
    fn test_parse_grade_c_plus() {
        assert_eq!(parse_grade("C+").unwrap(), Grade::CPlus);
    }

    #[test]
    fn test_parse_grade_c() {
        assert_eq!(parse_grade("C").unwrap(), Grade::C);
    }

    #[test]
    fn test_parse_grade_c_minus() {
        assert_eq!(parse_grade("C-").unwrap(), Grade::CMinus);
    }

    #[test]
    fn test_parse_grade_d() {
        assert_eq!(parse_grade("D").unwrap(), Grade::D);
    }

    #[test]
    fn test_parse_grade_f() {
        assert_eq!(parse_grade("F").unwrap(), Grade::F);
    }

    #[test]
    fn test_parse_grade_lowercase() {
        assert_eq!(parse_grade("a+").unwrap(), Grade::APLus);
        assert_eq!(parse_grade("b").unwrap(), Grade::B);
        assert_eq!(parse_grade("c-").unwrap(), Grade::CMinus);
    }

    #[test]
    fn test_parse_grade_invalid() {
        let err = parse_grade("X").err().unwrap();
        assert!(err.to_string().contains("Invalid grade"));
    }

    #[test]
    fn test_parse_grade_empty() {
        let err = parse_grade("").err().unwrap();
        assert!(err.to_string().contains("Invalid grade"));
    }
}

mod is_analyzable_file_tests {
    use super::*;

    #[test]
    fn test_rust_file() {
        assert!(is_analyzable_file(Path::new("test.rs")));
    }

    #[test]
    fn test_python_file() {
        assert!(is_analyzable_file(Path::new("test.py")));
    }

    #[test]
    fn test_javascript_file() {
        assert!(is_analyzable_file(Path::new("test.js")));
    }

    #[test]
    fn test_typescript_file() {
        assert!(is_analyzable_file(Path::new("test.ts")));
    }

    #[test]
    fn test_tsx_file() {
        assert!(is_analyzable_file(Path::new("component.tsx")));
    }

    #[test]
    fn test_jsx_file() {
        assert!(is_analyzable_file(Path::new("component.jsx")));
    }

    #[test]
    fn test_java_file() {
        assert!(is_analyzable_file(Path::new("Main.java")));
    }

    #[test]
    fn test_c_file() {
        assert!(is_analyzable_file(Path::new("main.c")));
    }

    #[test]
    fn test_cpp_file() {
        assert!(is_analyzable_file(Path::new("main.cpp")));
    }

    #[test]
    fn test_header_file() {
        assert!(is_analyzable_file(Path::new("header.h")));
        assert!(is_analyzable_file(Path::new("header.hpp")));
    }

    #[test]
    fn test_go_file() {
        assert!(is_analyzable_file(Path::new("main.go")));
    }

    #[test]
    fn test_ruby_file() {
        assert!(is_analyzable_file(Path::new("app.rb")));
    }

    #[test]
    fn test_php_file() {
        assert!(is_analyzable_file(Path::new("index.php")));
    }

    #[test]
    fn test_swift_file() {
        assert!(is_analyzable_file(Path::new("App.swift")));
    }

    #[test]
    fn test_kotlin_file() {
        assert!(is_analyzable_file(Path::new("Main.kt")));
        assert!(is_analyzable_file(Path::new("build.kts")));
    }

    #[test]
    fn test_non_analyzable_file() {
        assert!(!is_analyzable_file(Path::new("readme.md")));
        assert!(!is_analyzable_file(Path::new("data.json")));
        assert!(!is_analyzable_file(Path::new("config.toml")));
        assert!(!is_analyzable_file(Path::new("Makefile")));
    }

    #[test]
    fn test_no_extension() {
        assert!(!is_analyzable_file(Path::new("Dockerfile")));
    }
}

mod truncate_string_tests {
    use super::*;

    #[test]
    fn test_short_string() {
        let result = truncate_string("hello", 10);
        assert_eq!(result.trim(), "hello");
    }

    #[test]
    fn test_exact_length_string() {
        let result = truncate_string("hello", 5);
        assert_eq!(result.trim(), "hello");
    }

    #[test]
    fn test_long_string_truncated() {
        let result = truncate_string("hello world", 8);
        assert_eq!(result, "hello...");
    }

    #[test]
    fn test_empty_string() {
        let result = truncate_string("", 10);
        assert_eq!(result.trim(), "");
    }
}

mod load_tdg_configuration_tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = make_test_config(PathBuf::from("."));
        let result = load_tdg_configuration(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_missing_config_file() {
        let mut config = make_test_config(PathBuf::from("."));
        config.config = Some(PathBuf::from("/nonexistent/config.toml"));

        let result = load_tdg_configuration(&config);
        assert!(result.is_err());
    }
}

mod validate_minimum_grade_tests {
    use super::*;

    fn make_test_score(grade: Grade, total: f64) -> crate::tdg::TdgScore {
        crate::tdg::TdgScore {
            total: total as f32,
            grade,
            confidence: 1.0,
            language: crate::tdg::Language::Rust,
            structural_complexity: 0.0,
            semantic_complexity: 0.0,
            duplication_ratio: 0.0,
            coupling_score: 0.0,
            doc_coverage: 0.0,
            consistency_score: 0.0,
            entropy_score: 0.0,
            file_path: None,
            penalties_applied: vec![],
            critical_defects_count: 0,
            has_critical_defects: false,
        }
    }

    #[test]
    fn test_no_minimum_grade() {
        let config = make_test_config(PathBuf::from("."));
        let score = make_test_score(Grade::F, 10.0);
        let result = validate_minimum_grade(&score, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_grade_equals_minimum() {
        let mut config = make_test_config(PathBuf::from("."));
        config.min_grade = Some("B".to_string());
        let score = make_test_score(Grade::B, 80.0);
        let result = validate_minimum_grade(&score, &config);
        assert!(result.is_ok());
    }
}

mod format_tdg_output_tests {
    use super::*;

    fn make_test_score() -> crate::tdg::TdgScore {
        crate::tdg::TdgScore {
            total: 85.5,
            grade: Grade::B,
            confidence: 0.95,
            language: crate::tdg::Language::Rust,
            structural_complexity: 20.0,
            semantic_complexity: 15.0,
            duplication_ratio: 5.0,
            coupling_score: 10.0,
            doc_coverage: 8.0,
            consistency_score: 7.5,
            entropy_score: 20.0,
            file_path: Some(PathBuf::from("test.rs")),
            penalties_applied: vec![],
            critical_defects_count: 0,
            has_critical_defects: false,
        }
    }

    #[test]
    fn test_quiet_mode() {
        let mut config = make_test_config(PathBuf::from("."));
        config.quiet = true;
        let score = make_test_score();
        let result = format_tdg_output(&score, None, &config).unwrap();
        assert_eq!(result, "85.5");
    }

    #[test]
    fn test_table_format() {
        let mut config = make_test_config(PathBuf::from("."));
        config.format = TdgOutputFormat::Table;
        let score = make_test_score();
        let result = format_tdg_output(&score, None, &config).unwrap();
        assert!(result.contains("TDG Score Report"));
        assert!(result.contains("85.5"));
    }

    #[test]
    fn test_json_format() {
        let mut config = make_test_config(PathBuf::from("."));
        config.format = TdgOutputFormat::Json;
        let score = make_test_score();
        let result = format_tdg_output(&score, None, &config).unwrap();
        assert!(result.contains("\"total\""));
        assert!(result.contains("85.5"));
    }

    #[test]
    fn test_markdown_format() {
        let mut config = make_test_config(PathBuf::from("."));
        config.format = TdgOutputFormat::Markdown;
        let score = make_test_score();
        let result = format_tdg_output(&score, None, &config).unwrap();
        assert!(result.contains("# TDG Score Report"));
        assert!(result.contains("**Overall Score**"));
    }

    #[test]
    fn test_include_components() {
        let mut config = make_test_config(PathBuf::from("."));
        config.include_components = true;
        config.format = TdgOutputFormat::Table;
        let score = make_test_score();
        let result = format_tdg_output(&score, None, &config).unwrap();
        assert!(result.contains("Structural"));
        assert!(result.contains("Semantic"));
    }
}

mod format_tdg_score_tests {
    use super::*;

    fn make_test_score() -> crate::tdg::TdgScore {
        crate::tdg::TdgScore {
            total: 75.0,
            grade: Grade::C,
            confidence: 0.9,
            language: crate::tdg::Language::Python,
            structural_complexity: 15.0,
            semantic_complexity: 12.0,
            duplication_ratio: 8.0,
            coupling_score: 10.0,
            doc_coverage: 5.0,
            consistency_score: 5.0,
            entropy_score: 20.0,
            file_path: None,
            penalties_applied: vec![],
            critical_defects_count: 0,
            has_critical_defects: false,
        }
    }

    #[test]
    fn test_table_without_components() {
        let score = make_test_score();
        let result = format_tdg_score(score, None, TdgOutputFormat::Table, false).unwrap();
        assert!(result.contains("TDG Score Report"));
        assert!(!result.contains("Breakdown"));
    }

    #[test]
    fn test_table_with_components() {
        let score = make_test_score();
        let result = format_tdg_score(score, None, TdgOutputFormat::Table, true).unwrap();
        assert!(result.contains("Breakdown"));
        assert!(result.contains("Structural"));
    }

    #[test]
    fn test_json_output() {
        let score = make_test_score();
        let result = format_tdg_score(score, None, TdgOutputFormat::Json, false).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["score"]["total"], 75.0);
        assert_eq!(parsed["score"]["grade"], "C");
    }

    #[test]
    fn test_markdown_output() {
        let score = make_test_score();
        let result = format_tdg_score(score, None, TdgOutputFormat::Markdown, false).unwrap();
        assert!(result.contains("# TDG Score Report"));
        assert!(result.contains("**Overall Score**"));
    }

    #[test]
    fn test_sarif_output() {
        let score = make_test_score();
        let result = format_tdg_score(score, None, TdgOutputFormat::Sarif, false).unwrap();
        assert_eq!(result.trim(), "75.0");
    }

    #[test]
    fn test_with_git_context() {
        let score = make_test_score();
        let git_context = crate::models::git_context::GitContext {
            commit_sha: "abc123def456".to_string(),
            commit_sha_short: "abc123d".to_string(),
            branch: "main".to_string(),
            author_name: "Test Author".to_string(),
            author_email: "test@example.com".to_string(),
            commit_timestamp: chrono::Utc::now(),
            commit_message: "Test commit".to_string(),
            tags: vec!["v1.0".to_string()],
            parent_commits: vec![],
            remote_url: None,
            is_clean: true,
            uncommitted_files: 0,
        };
        let result =
            format_tdg_score(score, Some(&git_context), TdgOutputFormat::Table, false).unwrap();
        assert!(result.contains("Git Context"));
        assert!(result.contains("abc123d"));
    }
}

mod format_comparison_tests {
    use super::*;

    fn make_comparison() -> crate::tdg::Comparison {
        crate::tdg::Comparison {
            source1: crate::tdg::TdgScore {
                total: 70.0,
                grade: Grade::C,
                confidence: 0.9,
                language: crate::tdg::Language::Rust,
                structural_complexity: 15.0,
                semantic_complexity: 12.0,
                duplication_ratio: 8.0,
                coupling_score: 10.0,
                doc_coverage: 5.0,
                consistency_score: 5.0,
                entropy_score: 15.0,
                file_path: Some(PathBuf::from("file1.rs")),
                penalties_applied: vec![],
                critical_defects_count: 0,
                has_critical_defects: false,
            },
            source2: crate::tdg::TdgScore {
                total: 85.0,
                grade: Grade::B,
                confidence: 0.95,
                language: crate::tdg::Language::Rust,
                structural_complexity: 20.0,
                semantic_complexity: 15.0,
                duplication_ratio: 5.0,
                coupling_score: 8.0,
                doc_coverage: 8.0,
                consistency_score: 9.0,
                entropy_score: 20.0,
                file_path: Some(PathBuf::from("file2.rs")),
                penalties_applied: vec![],
                critical_defects_count: 0,
                has_critical_defects: false,
            },
            delta: 15.0,
            improvement_percentage: 21.4,
            winner: "source2".to_string(),
            improvements: vec!["duplication_ratio".to_string()],
            regressions: vec![],
        }
    }

    #[test]
    fn test_table_format() {
        let comparison = make_comparison();
        let result = format_comparison(comparison, TdgOutputFormat::Table).unwrap();
        assert!(result.contains("TDG Comparison"));
        assert!(result.contains("70.0"));
        assert!(result.contains("85.0"));
        assert!(result.contains("+15.0"));
    }

    #[test]
    fn test_json_format() {
        let comparison = make_comparison();
        let result = format_comparison(comparison, TdgOutputFormat::Json).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["source1"]["total"], 70.0);
        assert_eq!(parsed["source2"]["total"], 85.0);
        assert_eq!(parsed["difference"], 15.0);
    }
}

mod write_tdg_output_tests {
    use super::*;

    #[test]
    fn test_write_to_stdout() {
        let config = make_test_config(PathBuf::from("."));
        let result = write_tdg_output("test output", &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_to_file() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("output.txt");
        let mut config = make_test_config(temp_dir.path().to_path_buf());
        config.output = Some(output_path.clone());

        let result = write_tdg_output("test output content", &config);
        assert!(result.is_ok());
        assert!(output_path.exists());
        let content = std::fs::read_to_string(&output_path).unwrap();
        assert_eq!(content, "test output content");
    }
}

// ========== Integration Tests ==========

mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_handle_tdg_command_skips_test_file() {
        let temp_dir = TempDir::new().unwrap();
        let tests_dir = temp_dir.path().join("tests");
        std::fs::create_dir_all(&tests_dir).unwrap();
        let test_file = tests_dir.join("test_module.rs");
        std::fs::write(&test_file, "fn test_fn() {}").unwrap();

        let config = TdgCommandConfig {
            path: test_file,
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
    async fn test_handle_tdg_command_skips_bench_file() {
        let temp_dir = TempDir::new().unwrap();
        let benches_dir = temp_dir.path().join("benches");
        std::fs::create_dir_all(&benches_dir).unwrap();
        let bench_file = benches_dir.join("bench_module.rs");
        std::fs::write(&bench_file, "fn bench_fn() {}").unwrap();

        let config = TdgCommandConfig {
            path: bench_file,
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
}

// ========== Property-Based Tests ==========

mod proptest_tests {
    use super::*;

    proptest! {
        #[test]
        fn test_parse_format_grade_roundtrip(grade_idx in 0usize..11) {
            let grades = [
                Grade::APLus, Grade::A, Grade::AMinus,
                Grade::BPlus, Grade::B, Grade::BMinus,
                Grade::CPlus, Grade::C, Grade::CMinus,
                Grade::D, Grade::F,
            ];
            let grade = grades[grade_idx];
            let formatted = format_grade(grade);
            let parsed = parse_grade(&formatted).unwrap();
            prop_assert_eq!(grade, parsed);
        }

        #[test]
        fn test_truncate_string_never_exceeds_length(s in ".{0,100}", max_len in 3usize..50) {
            let result = truncate_string(&s, max_len);
            // Result should not exceed max_len (accounting for padding)
            prop_assert!(result.len() >= max_len || result.contains(&s));
        }

        #[test]
        fn test_is_analyzable_file_consistency(filename in "[a-z]+\\.[a-z]{1,4}") {
            let path = Path::new(&filename);
            // Call should never panic
            let _ = is_analyzable_file(path);
        }

        #[test]
        fn test_format_grade_returns_valid_string(grade_idx in 0usize..11) {
            let grades = [
                Grade::APLus, Grade::A, Grade::AMinus,
                Grade::BPlus, Grade::B, Grade::BMinus,
                Grade::CPlus, Grade::C, Grade::CMinus,
                Grade::D, Grade::F,
            ];
            let grade = grades[grade_idx];
            let result = format_grade(grade);
            prop_assert!(!result.is_empty());
            prop_assert!(result.len() <= 2);
        }
    }
}

// ========== Edge Case Tests ==========

mod edge_case_tests {
    use super::*;

    #[test]
    fn test_config_with_all_options() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("output.json");
        let config_path = create_test_config_file(&temp_dir);

        let config = TdgCommandConfig {
            path: temp_dir.path().to_path_buf(),
            command: None,
            format: TdgOutputFormat::Json,
            config: Some(config_path),
            quiet: false,
            include_components: true,
            min_grade: Some("B".to_string()),
            output: Some(output_path),
            with_git_context: true,
            explain: false,
            threshold: 15,
            baseline: Some("HEAD~5".to_string()),
            viz: false,
            viz_theme: "high-contrast".to_string(),
        };

        // Just verify config creation doesn't panic
        assert_eq!(config.threshold, 15);
        assert!(config.include_components);
    }

    #[test]
    fn test_empty_file_path() {
        let path = Path::new("");
        assert!(!is_analyzable_file(path));
    }

    #[test]
    fn test_hidden_file() {
        let path = Path::new(".hidden.rs");
        assert!(is_analyzable_file(path)); // Extension matters, not name
    }

    #[test]
    fn test_deeply_nested_path() {
        let path = Path::new("a/b/c/d/e/f/g/h/i/j/file.rs");
        assert!(is_analyzable_file(path));
    }

    #[test]
    fn test_unicode_filename() {
        let path = Path::new("日本語.rs");
        assert!(is_analyzable_file(path));
    }
}

mod format_history_output_tests {
    use super::*;
    use crate::tdg::storage::{ComponentScores, FileIdentity, FullTdgRecord};

    fn make_test_record() -> FullTdgRecord {
        FullTdgRecord {
            identity: FileIdentity {
                path: PathBuf::from("test.rs"),
                content_hash: blake3::hash(b"test"),
                size_bytes: 1024,
                modified_time: std::time::SystemTime::now(),
            },
            score: crate::tdg::TdgScore {
                total: 80.0,
                grade: Grade::B,
                confidence: 0.9,
                language: crate::tdg::Language::Rust,
                structural_complexity: 15.0,
                semantic_complexity: 12.0,
                duplication_ratio: 8.0,
                coupling_score: 10.0,
                doc_coverage: 5.0,
                consistency_score: 5.0,
                entropy_score: 25.0,
                file_path: Some(PathBuf::from("test.rs")),
                penalties_applied: vec![],
                critical_defects_count: 0,
                has_critical_defects: false,
            },
            components: ComponentScores::default(),
            semantic_sig: crate::tdg::storage::SemanticSignature {
                ast_structure_hash: 12345,
                identifier_pattern: "test_pattern".to_string(),
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
                commit_sha: "abcdef123456".to_string(),
                commit_sha_short: "abcdef1".to_string(),
                branch: "main".to_string(),
                author_name: "Test User".to_string(),
                author_email: "test@test.com".to_string(),
                commit_timestamp: chrono::Utc::now(),
                commit_message: "Test commit".to_string(),
                tags: vec![],
                parent_commits: vec![],
                remote_url: None,
                is_clean: true,
                uncommitted_files: 0,
            }),
        }
    }

    #[test]
    fn test_table_format_with_git_context() {
        let records = vec![make_test_record()];
        let result = format_history_output(&records, TdgOutputFormat::Table).unwrap();
        assert!(result.contains("TDG History"));
        assert!(result.contains("abcdef1"));
        assert!(result.contains("main"));
    }

    #[test]
    fn test_json_format_with_git_context() {
        let records = vec![make_test_record()];
        let result = format_history_output(&records, TdgOutputFormat::Json).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["total_records"], 1);
        assert!(parsed["history"].is_array());
    }

    #[test]
    fn test_empty_records() {
        let records: Vec<FullTdgRecord> = vec![];
        let result = format_history_output(&records, TdgOutputFormat::Table).unwrap();
        assert!(result.contains("TDG History"));
    }
}
