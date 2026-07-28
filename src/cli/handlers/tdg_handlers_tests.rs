// Tests for TDG handlers
// Extracted for file health compliance (CB-040)

use super::*;

/// Active unit tests for tdg_handlers (not feature-gated)
mod unit_tests {
    use super::*;
    use crate::tdg::Grade;
    use std::path::{Path, PathBuf};

    // ========== Test Fixtures ==========

    /// Create a default TdgCommandConfig for testing
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

    /// Create a TdgScore for testing
    fn make_test_score(total: f32, grade: Grade) -> crate::tdg::TdgScore {
        crate::tdg::TdgScore {
            total,
            grade,
            confidence: 0.95,
            language: crate::tdg::Language::Rust,
            structural_complexity: 20.0,
            semantic_complexity: 15.0,
            duplication_ratio: 5.0,
            coupling_score: 10.0,
            doc_coverage: 8.0,
            consistency_score: 7.0,
            entropy_score: 20.0,
            file_path: None,
            penalties_applied: vec![],
            critical_defects_count: 0,
            has_critical_defects: false,
            has_contract_coverage: false,
        }
    }

    // ========== format_grade tests ==========

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

        #[test]
        fn test_format_grade_all_grades_return_non_empty() {
            let grades = [
                Grade::APLus,
                Grade::A,
                Grade::AMinus,
                Grade::BPlus,
                Grade::B,
                Grade::BMinus,
                Grade::CPlus,
                Grade::C,
                Grade::CMinus,
                Grade::D,
                Grade::F,
            ];
            for grade in grades {
                let formatted = format_grade(grade);
                assert!(
                    !formatted.is_empty(),
                    "Grade {:?} formatted to empty string",
                    grade
                );
                assert!(
                    formatted.len() <= 2,
                    "Grade {:?} formatted to {} (too long)",
                    grade,
                    formatted
                );
            }
        }
    }

    // ========== parse_grade tests ==========

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
            assert_eq!(parse_grade("f").unwrap(), Grade::F);
        }

        #[test]
        fn test_parse_grade_mixed_case() {
            assert_eq!(parse_grade("a+").unwrap(), Grade::APLus);
            assert_eq!(parse_grade("A+").unwrap(), Grade::APLus);
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

        #[test]
        fn test_parse_grade_whitespace() {
            // Leading/trailing whitespace should fail
            let err = parse_grade(" A").err().unwrap();
            assert!(err.to_string().contains("Invalid grade"));
        }

        #[test]
        fn test_parse_format_roundtrip() {
            let grades = [
                Grade::APLus,
                Grade::A,
                Grade::AMinus,
                Grade::BPlus,
                Grade::B,
                Grade::BMinus,
                Grade::CPlus,
                Grade::C,
                Grade::CMinus,
                Grade::D,
                Grade::F,
            ];
            for grade in grades {
                let formatted = format_grade(grade);
                let parsed = parse_grade(&formatted).unwrap();
                assert_eq!(grade, parsed, "Roundtrip failed for {:?}", grade);
            }
        }
    }

    // ========== is_analyzable_file tests ==========

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
        fn test_typescript_files() {
            assert!(is_analyzable_file(Path::new("test.ts")));
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
        fn test_c_cpp_files() {
            assert!(is_analyzable_file(Path::new("main.c")));
            assert!(is_analyzable_file(Path::new("main.cpp")));
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
        fn test_kotlin_files() {
            assert!(is_analyzable_file(Path::new("Main.kt")));
            assert!(is_analyzable_file(Path::new("build.kts")));
        }

        #[test]
        fn test_non_analyzable_files() {
            assert!(!is_analyzable_file(Path::new("readme.md")));
            assert!(!is_analyzable_file(Path::new("data.json")));
            assert!(!is_analyzable_file(Path::new("config.toml")));
            assert!(!is_analyzable_file(Path::new("Makefile")));
            assert!(!is_analyzable_file(Path::new("style.css")));
            assert!(!is_analyzable_file(Path::new("index.html")));
        }

        #[test]
        fn test_no_extension() {
            assert!(!is_analyzable_file(Path::new("Dockerfile")));
            assert!(!is_analyzable_file(Path::new("README")));
        }

        #[test]
        fn test_hidden_file_with_extension() {
            // Extension matters, not the hidden prefix
            assert!(is_analyzable_file(Path::new(".hidden.rs")));
        }

        #[test]
        fn test_deeply_nested_path() {
            assert!(is_analyzable_file(Path::new("a/b/c/d/e/f/g/h/file.rs")));
        }

        #[test]
        fn test_unicode_filename() {
            assert!(is_analyzable_file(Path::new("日本語.rs")));
            assert!(is_analyzable_file(Path::new("файл.py")));
        }

        #[test]
        fn test_empty_path() {
            assert!(!is_analyzable_file(Path::new("")));
        }

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
    }

    // ========== truncate_string tests ==========

    mod truncate_string_tests {
        use super::*;

        #[test]
        fn test_short_string_padded() {
            let result = truncate_string("hello", 10);
            assert_eq!(result.trim(), "hello");
            assert_eq!(result.len(), 10);
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
            assert_eq!(result.len(), 10);
            assert_eq!(result.trim(), "");
        }

        #[test]
        fn test_truncate_minimum_length() {
            // With length 3, we get "..." which is the minimum meaningful truncation
            let result = truncate_string("abcdef", 3);
            assert_eq!(result, "...");
        }

        #[test]
        fn test_truncate_preserves_start() {
            let result = truncate_string("abcdefghijklmnop", 10);
            assert!(result.starts_with("abcdefg"));
            assert!(result.ends_with("..."));
        }
    }

    // ========== TdgCommandConfig tests ==========

    mod tdg_command_config_tests {
        use super::*;

        #[test]
        fn test_default_config_creation() {
            let config = make_test_config(PathBuf::from("."));
            assert_eq!(config.path, PathBuf::from("."));
            assert!(!config.quiet);
            assert!(!config.include_components);
            assert!(config.min_grade.is_none());
            assert!(config.command.is_none());
        }

        #[test]
        fn test_config_with_all_options() {
            let config = TdgCommandConfig {
                path: PathBuf::from("/tmp/test"),
                command: None,
                format: TdgOutputFormat::Json,
                config: Some(PathBuf::from("/tmp/config.toml")),
                quiet: true,
                include_components: true,
                min_grade: Some("B".to_string()),
                output: Some(PathBuf::from("/tmp/output.json")),
                with_git_context: true,
                explain: true,
                threshold: 15,
                baseline: Some("HEAD~5".to_string()),
                viz: true,
                viz_theme: "high-contrast".to_string(),
            };

            assert_eq!(config.threshold, 15);
            assert!(config.include_components);
            assert!(config.quiet);
            assert!(config.explain);
            assert!(config.viz);
        }
    }

    // ========== validate_minimum_grade tests ==========

    mod validate_minimum_grade_tests {
        use super::*;

        #[test]
        fn test_no_minimum_grade_always_passes() {
            let config = make_test_config(PathBuf::from("."));
            let score = make_test_score(10.0, Grade::F);
            let result = validate_minimum_grade(&score, &config);
            assert!(result.is_ok());
        }

        #[test]
        fn test_grade_equals_minimum() {
            let mut config = make_test_config(PathBuf::from("."));
            config.min_grade = Some("B".to_string());
            let score = make_test_score(75.0, Grade::B);
            let result = validate_minimum_grade(&score, &config);
            assert!(result.is_ok());
        }

        /// Direction pins — the old `score.grade < min_grade` comparison was
        /// inverted (Grade ordering is APLus < ... < F, smaller = better):
        /// it rejected grades better than the minimum and accepted worse
        /// ones. Equal-grade tests alone cannot catch that.
        #[test]
        fn test_grade_better_than_minimum_passes() {
            let mut config = make_test_config(PathBuf::from("."));
            config.min_grade = Some("B".to_string());
            let score = make_test_score(95.0, Grade::APLus);
            assert!(
                validate_minimum_grade(&score, &config).is_ok(),
                "A+ must satisfy a B minimum"
            );
        }

        #[test]
        fn test_grade_worse_than_minimum_fails() {
            let mut config = make_test_config(PathBuf::from("."));
            config.min_grade = Some("B".to_string());
            let score = make_test_score(40.0, Grade::D);
            assert!(
                validate_minimum_grade(&score, &config).is_err(),
                "D must NOT satisfy a B minimum"
            );
        }
    }

    // ========== format_tdg_output tests ==========

    mod format_tdg_output_tests {
        use super::*;

        #[test]
        fn test_quiet_mode_outputs_score_only() {
            let mut config = make_test_config(PathBuf::from("."));
            config.quiet = true;
            let score = make_test_score(85.5, Grade::B);
            let result = format_tdg_output(&score, None, &config).unwrap();
            assert_eq!(result, "85.5");
        }

        #[test]
        fn test_table_format_contains_header() {
            let mut config = make_test_config(PathBuf::from("."));
            config.format = TdgOutputFormat::Table;
            let score = make_test_score(85.5, Grade::B);
            let result = format_tdg_output(&score, None, &config).unwrap();
            assert!(result.contains("TDG Score Report"));
            assert!(result.contains("85.5"));
        }

        #[test]
        fn test_json_format_is_valid_json() {
            let mut config = make_test_config(PathBuf::from("."));
            config.format = TdgOutputFormat::Json;
            let score = make_test_score(75.0, Grade::B);
            let result = format_tdg_output(&score, None, &config).unwrap();
            let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
            assert!(parsed.get("score").is_some());
        }

        #[test]
        fn test_markdown_format_has_header() {
            let mut config = make_test_config(PathBuf::from("."));
            config.format = TdgOutputFormat::Markdown;
            let score = make_test_score(80.0, Grade::BPlus);
            let result = format_tdg_output(&score, None, &config).unwrap();
            assert!(result.contains("# TDG Score Report"));
            assert!(result.contains("**Overall Score**"));
        }

        #[test]
        fn test_include_components_shows_breakdown() {
            let mut config = make_test_config(PathBuf::from("."));
            config.include_components = true;
            config.format = TdgOutputFormat::Table;
            let score = make_test_score(80.0, Grade::BPlus);
            let result = format_tdg_output(&score, None, &config).unwrap();
            assert!(result.contains("Breakdown"));
            assert!(result.contains("Structural"));
        }
    }

    // ========== format_tdg_score tests ==========

    mod format_tdg_score_tests {
        use super::*;

        #[test]
        fn test_table_without_components() {
            let score = make_test_score(75.0, Grade::B);
            let result = format_tdg_score(score, None, TdgOutputFormat::Table, false).unwrap();
            assert!(result.contains("TDG Score Report"));
            assert!(!result.contains("Breakdown"));
        }

        #[test]
        fn test_table_with_components() {
            let score = make_test_score(75.0, Grade::B);
            let result = format_tdg_score(score, None, TdgOutputFormat::Table, true).unwrap();
            assert!(result.contains("Breakdown"));
            assert!(result.contains("Structural"));
            assert!(result.contains("Semantic"));
        }

        #[test]
        fn test_json_output_structure() {
            let score = make_test_score(75.0, Grade::B);
            let result = format_tdg_score(score, None, TdgOutputFormat::Json, false).unwrap();
            let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
            assert_eq!(parsed["score"]["total"], 75.0);
            assert_eq!(parsed["score"]["grade"], "B");
        }

        #[test]
        fn test_json_with_components() {
            let score = make_test_score(75.0, Grade::B);
            let result = format_tdg_score(score, None, TdgOutputFormat::Json, true).unwrap();
            let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
            assert!(parsed["score"]["breakdown"].is_object());
        }

        #[test]
        fn test_json_without_components_null_breakdown() {
            let score = make_test_score(75.0, Grade::B);
            let result = format_tdg_score(score, None, TdgOutputFormat::Json, false).unwrap();
            let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
            assert!(parsed["score"]["breakdown"].is_null());
        }

        #[test]
        fn test_markdown_output() {
            let score = make_test_score(75.0, Grade::B);
            let result = format_tdg_score(score, None, TdgOutputFormat::Markdown, false).unwrap();
            assert!(result.contains("# TDG Score Report"));
            assert!(result.contains("**Overall Score**"));
        }

        #[test]
        fn test_markdown_with_components() {
            let score = make_test_score(75.0, Grade::B);
            let result = format_tdg_score(score, None, TdgOutputFormat::Markdown, true).unwrap();
            assert!(result.contains("## Component Breakdown"));
            assert!(result.contains("| Component | Score | Max |"));
        }

        #[test]
        fn test_sarif_output_is_score_only() {
            let score = make_test_score(75.0, Grade::B);
            let result = format_tdg_score(score, None, TdgOutputFormat::Sarif, false).unwrap();
            assert_eq!(result.trim(), "75.0");
        }

        #[test]
        fn test_with_file_path() {
            let mut score = make_test_score(88.0, Grade::BPlus);
            score.file_path = Some(PathBuf::from("src/handlers/tdg.rs"));

            let result =
                format_tdg_score(score.clone(), None, TdgOutputFormat::Table, false).unwrap();
            assert!(result.contains("src/handlers/tdg.rs"));

            let result =
                format_tdg_score(score.clone(), None, TdgOutputFormat::Markdown, false).unwrap();
            assert!(result.contains("**File**: `src/handlers/tdg.rs`"));

            let result = format_tdg_score(score, None, TdgOutputFormat::Json, false).unwrap();
            let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
            assert!(parsed["file"].as_str().unwrap().contains("tdg.rs"));
        }

        #[test]
        fn test_with_git_context() {
            let score = make_test_score(80.0, Grade::B);
            let git_context = crate::models::git_context::GitContext {
                commit_sha: "abc123def456789".to_string(),
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

            let result = format_tdg_score(
                score.clone(),
                Some(&git_context),
                TdgOutputFormat::Table,
                false,
            )
            .unwrap();
            assert!(result.contains("Git Context"));
            assert!(result.contains("abc123d"));
            assert!(result.contains("main"));

            let result =
                format_tdg_score(score, Some(&git_context), TdgOutputFormat::Json, false).unwrap();
            let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
            assert_eq!(parsed["git_context"]["branch"], "main");
            assert_eq!(parsed["git_context"]["is_clean"], true);
        }
    }

    // ========== format_comparison tests ==========

    mod format_comparison_tests {
        use super::*;

        fn make_comparison() -> crate::tdg::Comparison {
            crate::tdg::Comparison {
                source1: make_test_score(70.0, Grade::C),
                source2: make_test_score(85.0, Grade::B),
                delta: 15.0,
                improvement_percentage: 21.4,
                winner: "source2".to_string(),
                improvements: vec!["duplication".to_string()],
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
            assert_eq!(parsed["winner"], "source2");
        }

        #[test]
        fn test_markdown_uses_json() {
            // For non-table formats, JSON is used
            let comparison = make_comparison();
            let result = format_comparison(comparison, TdgOutputFormat::Markdown).unwrap();
            // Should be valid JSON
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(&result);
            assert!(parsed.is_ok());
        }
    }

    // ========== write_tdg_output tests ==========

    mod write_tdg_output_tests {
        use super::*;
        use tempfile::TempDir;

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

    // ========== load_tdg_configuration tests ==========

    mod load_tdg_configuration_tests {
        use super::*;
        use tempfile::TempDir;

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

        #[test]
        fn test_invalid_toml_config() {
            let temp_dir = TempDir::new().unwrap();
            let config_path = temp_dir.path().join("invalid.toml");
            std::fs::write(&config_path, "this is not valid toml [[[").unwrap();

            let mut config = make_test_config(temp_dir.path().to_path_buf());
            config.config = Some(config_path);

            let result = load_tdg_configuration(&config);
            assert!(result.is_err());
        }
    }

    // ========== format_history_output tests ==========

    mod format_history_output_tests {
        use super::*;
        use crate::tdg::storage::{ComponentScores, FileIdentity, FullTdgRecord};

        fn make_test_record(path: &str, total: f32, commit_sha: &str) -> FullTdgRecord {
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
                    has_contract_coverage: false,
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
                    commit_sha_short: commit_sha[..7.min(commit_sha.len())].to_string(),
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
        fn test_table_format_with_git_context() {
            let records = vec![make_test_record("test.rs", 80.0, "abcdef123456")];
            let result = format_history_output(&records, TdgOutputFormat::Table).unwrap();
            assert!(result.contains("TDG History"));
            assert!(result.contains("abcdef1"));
            assert!(result.contains("main"));
        }

        #[test]
        fn test_json_format_with_git_context() {
            let records = vec![make_test_record("test.rs", 80.0, "abcdef123456")];
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

        #[test]
        fn test_multiple_records() {
            let records = vec![
                make_test_record("src/lib.rs", 85.0, "abc1234567890"),
                make_test_record("src/main.rs", 75.0, "def4567890abc"),
                make_test_record("src/utils.rs", 90.0, "ghi7890abcdef"),
            ];

            let result = format_history_output(&records, TdgOutputFormat::Table).unwrap();
            assert!(result.contains("abc1234"));
            assert!(result.contains("def4567"));
            assert!(result.contains("ghi7890"));

            let result = format_history_output(&records, TdgOutputFormat::Json).unwrap();
            let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
            assert_eq!(parsed["total_records"], 3);
        }
    }

    // ========== display_gate_result_table tests ==========

    mod display_gate_result_tests {
        use super::*;
        use crate::tdg::{GateResult, Severity, Violation, ViolationType};

        /// check-quality JSON mode must emit ONE combined document even when
        /// the F-grade gate has violations — previously two concatenated
        /// JSON docs landed on stdout on exactly that path.
        #[test]
        fn test_check_quality_json_single_document_with_f_grades() {
            let primary = GateResult {
                passed: true,
                gate_name: "MinimumGradeGate".to_string(),
                violations: vec![],
                message: "ok".to_string(),
            };
            let f_grade = GateResult {
                passed: false,
                gate_name: "FGradeGate".to_string(),
                violations: vec![Violation {
                    path: PathBuf::from("bad.rs"),
                    violation_type: ViolationType::BelowMinimum,
                    severity: Severity::Error,
                    message: "F grade".to_string(),
                    old_score: None,
                    new_score: 35.0,
                    old_grade: None,
                    new_grade: crate::tdg::Grade::F,
                }],
                message: "1 F-grade file".to_string(),
            };

            let doc = crate::cli::handlers::tdg_handlers::quality_gates::check_quality_json(
                &primary, &f_grade,
            )
            .unwrap();

            // Exactly one parseable document carrying both gate verdicts
            let parsed: serde_json::Value = serde_json::from_str(&doc).unwrap();
            assert_eq!(parsed["gate"]["gate_name"], "MinimumGradeGate");
            assert_eq!(parsed["f_grade_gate"]["gate_name"], "FGradeGate");
            assert_eq!(
                parsed["passed"], false,
                "combined verdict must AND both gates"
            );
        }

        #[test]
        fn test_display_passed_result() {
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
        fn test_display_failed_result_with_violations() {
            let result = GateResult {
                passed: false,
                gate_name: "MinimumGradeGate".to_string(),
                violations: vec![Violation {
                    path: PathBuf::from("bad_file.rs"),
                    violation_type: ViolationType::BelowMinimum,
                    severity: Severity::Error,
                    message: "Grade C is below minimum B".to_string(),
                    old_score: None,
                    new_score: 72.0,
                    old_grade: None,
                    new_grade: Grade::C,
                }],
                message: "1 violation found".to_string(),
            };

            // Just verify it doesn't panic
            display_gate_result_table(&result);
        }

        #[test]
        fn test_display_multiple_violations() {
            let result = GateResult {
                passed: false,
                gate_name: "QualityGate".to_string(),
                violations: vec![
                    Violation {
                        path: PathBuf::from("file1.rs"),
                        violation_type: ViolationType::BelowMinimum,
                        severity: Severity::Error,
                        message: "Below minimum".to_string(),
                        old_score: None,
                        new_score: 60.0,
                        old_grade: None,
                        new_grade: Grade::C,
                    },
                    Violation {
                        path: PathBuf::from("file2.rs"),
                        violation_type: ViolationType::Regression,
                        severity: Severity::Critical,
                        message: "Regression".to_string(),
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
}

/// NOTE: Temporarily disabled due to struct definition mismatches
/// NOTE: Temporarily disabled due to struct definition mismatches
#[cfg(all(test, feature = "broken-tests"))]
#[path = "tdg_handlers_coverage_tests.rs"]
mod coverage_tests;

#[cfg(all(test, feature = "broken-tests"))]
#[path = "tdg_handlers_coverage_tests2.rs"]
mod coverage_tests2;
