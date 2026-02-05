//! Toyota Way: Churn Analysis Formatting Handler
//! Complexity: Reduced from 17 to individual functions ≤8
//! Purpose: Churn report formatting with clean separation of concerns

use anyhow::Result;
use std::fmt::Write;
use std::path::Path;

/// Toyota Way: Single Responsibility - Format churn analysis as markdown
/// Extracted from stubs.rs to reduce complexity and improve maintainability
///
/// # Parameters
///
/// * `summary` - Churn analysis summary data
///
/// # Returns
///
/// * `Ok(String)` - Formatted markdown output
/// * `Err(anyhow::Error)` - Formatting failed
pub fn format_churn_markdown(analysis: &crate::models::churn::CodeChurnAnalysis) -> Result<String> {
    let mut output = String::new();

    // Header
    writeln!(&mut output, "# Code Churn Analysis Report\n")?;
    writeln!(
        &mut output,
        "Generated: {}",
        analysis.generated_at.format("%Y-%m-%d %H:%M:%S UTC")
    )?;
    writeln!(
        &mut output,
        "Repository: {}",
        analysis.repository_root.display()
    )?;
    writeln!(
        &mut output,
        "Analysis Period: {} days\n",
        analysis.period_days
    )?;

    // Summary statistics table
    write_markdown_summary_table(&mut output, &analysis.summary)?;

    // File details if available
    write_markdown_file_details(&mut output, &analysis.files)?;

    // Author contributions if available
    if !analysis.summary.author_contributions.is_empty() {
        write_author_contributions(&mut output, &analysis.summary)?;
    }

    Ok(output)
}

/// Toyota Way: Extract Method - Write markdown summary table (complexity ≤8)
/// Creates a summary statistics table in markdown format
pub fn write_markdown_summary_table(
    output: &mut String,
    summary: &crate::models::churn::ChurnSummary,
) -> Result<()> {
    writeln!(output, "## Summary Statistics\n")?;
    writeln!(output, "| Metric | Value |")?;
    writeln!(output, "|--------|-------|")?;
    writeln!(output, "| Total Commits | {} |", summary.total_commits)?;
    writeln!(
        output,
        "| Files Changed | {} |",
        summary.total_files_changed
    )?;
    writeln!(
        output,
        "| Hotspot Files | {} |",
        summary.hotspot_files.len()
    )?;
    writeln!(output, "| Stable Files | {} |", summary.stable_files.len())?;
    writeln!(
        output,
        "| Contributing Authors | {} |",
        summary.author_contributions.len()
    )?;
    Ok(())
}

/// Toyota Way: Extract Method - Write markdown file details (complexity ≤8)
fn write_markdown_file_details(
    output: &mut String,
    files: &[crate::models::churn::FileChurnMetrics],
) -> Result<()> {
    if files.is_empty() {
        return Ok(());
    }

    writeln!(output, "\n## File Churn Details\n")?;
    writeln!(
        output,
        "| File | Commits | Authors | Additions | Deletions | Churn Score | Last Modified |"
    )?;
    writeln!(
        output,
        "|------|---------|---------|-----------|-----------|-------------|----------------|"
    )?;

    // Sort by churn score descending
    let mut sorted_files = files.to_vec();
    sorted_files.sort_by(|a, b| {
        b.churn_score
            .partial_cmp(&a.churn_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Write top 20 files
    for file in sorted_files.iter().take(20) {
        write_file_row(output, file)?;
    }

    Ok(())
}

/// Toyota Way: Extract Method - Write single file row (complexity ≤3)
fn write_file_row(
    output: &mut String,
    file: &crate::models::churn::FileChurnMetrics,
) -> Result<()> {
    writeln!(
        output,
        "| {} | {} | {} | {} | {} | {:.2} | {} |",
        file.relative_path,
        file.commit_count,
        file.unique_authors.len(),
        file.additions,
        file.deletions,
        file.churn_score,
        file.last_modified.format("%Y-%m-%d"),
    )?;
    Ok(())
}

/// Toyota Way: Extract Method - Write author contributions (complexity ≤8)
fn write_author_contributions(
    output: &mut String,
    summary: &crate::models::churn::ChurnSummary,
) -> Result<()> {
    writeln!(output, "\n## Author Contributions\n")?;
    writeln!(output, "| Author | Files Modified |")?;
    writeln!(output, "|--------|----------------|")?;

    // Sort by file count descending
    let mut sorted_authors: Vec<_> = summary.author_contributions.iter().collect();
    sorted_authors.sort_by(|a, b| b.1.cmp(a.1));

    // Write top 15 authors
    for (author, count) in sorted_authors.iter().take(15) {
        writeln!(output, "| {author} | {count} |")?;
    }

    Ok(())
}

/// Toyota Way: Extract Method - Check if path is source file (complexity ≤8)
/// Determines if a path represents a source code file
#[must_use]
pub fn is_source_file(path: &Path) -> bool {
    // Check if it has a source code extension
    if !has_source_extension(path) {
        return false;
    }

    // Exclude test and example files by path
    if is_test_path(path) {
        return false;
    }

    // Exclude test files by name pattern
    if is_test_filename(path) {
        return false;
    }

    true
}

/// Toyota Way: Extract Method - Check source extension (complexity ≤3)
fn has_source_extension(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|s| s.to_str()),
        Some(
            "rs" | "js"
                | "ts"
                | "py"
                | "java"
                | "cpp"
                | "c"
                | "go"
                | "kt"
                | "swift"
                | "php"
                | "rb"
                | "scala"
        )
    )
}

/// Toyota Way: Extract Method - Check if path contains test directory (complexity ≤5)
fn is_test_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    let test_patterns = [
        "/tests/",
        "/test/",
        "/examples/",
        "/benches/",
        "/fixtures/",
        "/testdata/",
        "/test_data/",
        "/debug_test/",
        "/test-",
        "/__tests__/",
    ];

    test_patterns
        .iter()
        .any(|pattern| path_str.contains(pattern))
}

/// Toyota Way: Extract Method - Check if filename is test file (complexity ≤4)
fn is_test_filename(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|fname| {
            fname.ends_with("_test.rs")
                || fname.ends_with("_tests.rs")
                || fname.starts_with("test_")
                || fname.contains("_test_")
                || fname.ends_with(".test.js")
                || fname.ends_with(".spec.js")
                || fname.ends_with("_test.py")
                || fname.ends_with("Test.java")
        })
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    //! EXTREME TDD coverage tests for churn formatter
    //!
    //! These tests exercise the markdown formatting, path detection,
    //! and all helper functions with comprehensive edge cases.

    use super::*;
    use crate::models::churn::{ChurnSummary, CodeChurnAnalysis, FileChurnMetrics};
    use chrono::Utc;
    use std::collections::HashMap;
    use std::path::PathBuf;

    // ============================================================================
    // Test Fixtures
    // ============================================================================

    fn create_empty_summary() -> ChurnSummary {
        ChurnSummary {
            total_commits: 0,
            total_files_changed: 0,
            hotspot_files: vec![],
            stable_files: vec![],
            author_contributions: HashMap::new(),
            mean_churn_score: 0.0,
            variance_churn_score: 0.0,
            stddev_churn_score: 0.0,
        }
    }

    fn create_populated_summary() -> ChurnSummary {
        let mut author_contributions = HashMap::new();
        author_contributions.insert("alice".to_string(), 50);
        author_contributions.insert("bob".to_string(), 30);
        author_contributions.insert("charlie".to_string(), 20);

        ChurnSummary {
            total_commits: 100,
            total_files_changed: 25,
            hotspot_files: vec![PathBuf::from("src/main.rs"), PathBuf::from("src/lib.rs")],
            stable_files: vec![PathBuf::from("src/utils.rs")],
            author_contributions,
            mean_churn_score: 0.45,
            variance_churn_score: 0.04,
            stddev_churn_score: 0.2,
        }
    }

    fn create_test_file_metrics(
        path: &str,
        commit_count: usize,
        churn_score: f32,
    ) -> FileChurnMetrics {
        let now = Utc::now();
        FileChurnMetrics {
            path: PathBuf::from(path),
            relative_path: path.to_string(),
            commit_count,
            unique_authors: vec!["dev1".to_string(), "dev2".to_string()],
            additions: 100,
            deletions: 50,
            churn_score,
            last_modified: now,
            first_seen: now,
        }
    }

    fn create_test_analysis(files: Vec<FileChurnMetrics>) -> CodeChurnAnalysis {
        let summary = if files.is_empty() {
            create_empty_summary()
        } else {
            create_populated_summary()
        };

        CodeChurnAnalysis {
            generated_at: Utc::now(),
            period_days: 30,
            repository_root: PathBuf::from("/test/repo"),
            files,
            summary,
        }
    }

    // ============================================================================
    // format_churn_markdown Tests
    // ============================================================================

    mod format_churn_markdown_tests {
        use super::*;

        #[test]
        fn test_format_empty_analysis() {
            let analysis = create_test_analysis(vec![]);
            let result = format_churn_markdown(&analysis).unwrap();

            assert!(result.contains("# Code Churn Analysis Report"));
            assert!(result.contains("Analysis Period: 30 days"));
            assert!(result.contains("## Summary Statistics"));
        }

        #[test]
        fn test_format_with_files() {
            let files = vec![
                create_test_file_metrics("src/main.rs", 20, 0.85),
                create_test_file_metrics("src/lib.rs", 15, 0.65),
            ];
            let analysis = create_test_analysis(files);
            let result = format_churn_markdown(&analysis).unwrap();

            assert!(result.contains("# Code Churn Analysis Report"));
            assert!(result.contains("## File Churn Details"));
            assert!(result.contains("src/main.rs"));
            assert!(result.contains("src/lib.rs"));
        }

        #[test]
        fn test_format_includes_header() {
            let analysis = create_test_analysis(vec![]);
            let result = format_churn_markdown(&analysis).unwrap();

            assert!(result.contains("Generated:"));
            assert!(result.contains("Repository:"));
        }

        #[test]
        fn test_format_with_author_contributions() {
            // Need to pass non-empty files to get populated summary with authors
            let files = vec![create_test_file_metrics("src/main.rs", 20, 0.85)];
            let analysis = create_test_analysis(files);
            let result = format_churn_markdown(&analysis).unwrap();

            assert!(result.contains("## Author Contributions"));
            assert!(result.contains("alice"));
            assert!(result.contains("bob"));
            assert!(result.contains("charlie"));
        }

        #[test]
        fn test_format_sorts_files_by_churn_score() {
            let files = vec![
                create_test_file_metrics("low.rs", 5, 0.2),
                create_test_file_metrics("high.rs", 30, 0.9),
                create_test_file_metrics("medium.rs", 15, 0.5),
            ];
            let analysis = create_test_analysis(files);
            let result = format_churn_markdown(&analysis).unwrap();

            // high.rs should appear before medium.rs should appear before low.rs
            let high_pos = result.find("high.rs").unwrap();
            let medium_pos = result.find("medium.rs").unwrap();
            let low_pos = result.find("low.rs").unwrap();

            assert!(
                high_pos < medium_pos,
                "high.rs should appear before medium.rs"
            );
            assert!(
                medium_pos < low_pos,
                "medium.rs should appear before low.rs"
            );
        }

        #[test]
        fn test_format_limits_to_top_20_files() {
            // Create 25 files
            let files: Vec<FileChurnMetrics> = (0..25)
                .map(|i| {
                    create_test_file_metrics(&format!("file{}.rs", i), i + 1, (i as f32) / 25.0)
                })
                .collect();

            let analysis = create_test_analysis(files);
            let result = format_churn_markdown(&analysis).unwrap();

            // Count table rows (excluding header rows)
            let file_rows: Vec<&str> = result
                .lines()
                .filter(|line| line.starts_with("| file"))
                .collect();

            // Should be limited to 20 files
            assert!(file_rows.len() <= 20, "Should have at most 20 file rows");
        }
    }

    // ============================================================================
    // write_markdown_summary_table Tests
    // ============================================================================

    mod write_markdown_summary_table_tests {
        use super::*;

        #[test]
        fn test_empty_summary_table() {
            let summary = create_empty_summary();
            let mut output = String::new();

            write_markdown_summary_table(&mut output, &summary).unwrap();

            assert!(output.contains("## Summary Statistics"));
            assert!(output.contains("| Total Commits | 0 |"));
            assert!(output.contains("| Files Changed | 0 |"));
        }

        #[test]
        fn test_populated_summary_table() {
            let summary = create_populated_summary();
            let mut output = String::new();

            write_markdown_summary_table(&mut output, &summary).unwrap();

            assert!(output.contains("| Total Commits | 100 |"));
            assert!(output.contains("| Files Changed | 25 |"));
            assert!(output.contains("| Hotspot Files | 2 |"));
            assert!(output.contains("| Stable Files | 1 |"));
            assert!(output.contains("| Contributing Authors | 3 |"));
        }

        #[test]
        fn test_table_has_headers() {
            let summary = create_empty_summary();
            let mut output = String::new();

            write_markdown_summary_table(&mut output, &summary).unwrap();

            assert!(output.contains("| Metric | Value |"));
            assert!(output.contains("|--------|-------|"));
        }
    }

    // ============================================================================
    // is_source_file Tests
    // ============================================================================

    mod is_source_file_tests {
        use super::*;

        #[test]
        fn test_rust_source_file() {
            assert!(is_source_file(Path::new("src/main.rs")));
            assert!(is_source_file(Path::new("lib.rs")));
        }

        #[test]
        fn test_javascript_source_file() {
            assert!(is_source_file(Path::new("src/index.js")));
            assert!(is_source_file(Path::new("app.js")));
        }

        #[test]
        fn test_typescript_source_file() {
            assert!(is_source_file(Path::new("src/app.ts")));
            assert!(is_source_file(Path::new("component.ts")));
        }

        #[test]
        fn test_python_source_file() {
            assert!(is_source_file(Path::new("src/main.py")));
            assert!(is_source_file(Path::new("app.py")));
        }

        #[test]
        fn test_java_source_file() {
            assert!(is_source_file(Path::new("src/Main.java")));
            assert!(is_source_file(Path::new("App.java")));
        }

        #[test]
        fn test_cpp_source_files() {
            assert!(is_source_file(Path::new("src/main.cpp")));
            assert!(is_source_file(Path::new("lib.c")));
        }

        #[test]
        fn test_go_source_file() {
            assert!(is_source_file(Path::new("main.go")));
            assert!(is_source_file(Path::new("pkg/server.go")));
        }

        #[test]
        fn test_kotlin_source_file() {
            assert!(is_source_file(Path::new("Main.kt")));
        }

        #[test]
        fn test_swift_source_file() {
            assert!(is_source_file(Path::new("App.swift")));
        }

        #[test]
        fn test_php_source_file() {
            assert!(is_source_file(Path::new("index.php")));
        }

        #[test]
        fn test_ruby_source_file() {
            assert!(is_source_file(Path::new("app.rb")));
        }

        #[test]
        fn test_scala_source_file() {
            assert!(is_source_file(Path::new("Main.scala")));
        }

        #[test]
        fn test_non_source_files() {
            assert!(!is_source_file(Path::new("README.md")));
            assert!(!is_source_file(Path::new("Cargo.toml")));
            assert!(!is_source_file(Path::new("package.json")));
            assert!(!is_source_file(Path::new("data.csv")));
            assert!(!is_source_file(Path::new("image.png")));
        }

        #[test]
        fn test_file_without_extension() {
            assert!(!is_source_file(Path::new("Makefile")));
            assert!(!is_source_file(Path::new("Dockerfile")));
        }

        #[test]
        fn test_hidden_files() {
            assert!(!is_source_file(Path::new(".gitignore")));
        }
    }

    // ============================================================================
    // Test Path Detection (is_test_path and is_test_filename)
    // ============================================================================

    mod test_path_detection_tests {
        use super::*;

        #[test]
        fn test_tests_directory() {
            assert!(!is_source_file(Path::new("/project/tests/test_main.rs")));
            assert!(!is_source_file(Path::new("/project/test/unit.rs")));
        }

        #[test]
        fn test_examples_directory() {
            assert!(!is_source_file(Path::new("/project/examples/demo.rs")));
        }

        #[test]
        fn test_benches_directory() {
            assert!(!is_source_file(Path::new("/project/benches/benchmark.rs")));
        }

        #[test]
        fn test_fixtures_directory() {
            assert!(!is_source_file(Path::new("/project/fixtures/data.rs")));
        }

        #[test]
        fn test_testdata_directory() {
            assert!(!is_source_file(Path::new("/project/testdata/mock.rs")));
            assert!(!is_source_file(Path::new("/project/test_data/mock.rs")));
        }

        #[test]
        fn test_debug_test_directory() {
            assert!(!is_source_file(Path::new("/project/debug_test/helper.rs")));
        }

        #[test]
        fn test_jest_tests_directory() {
            assert!(!is_source_file(Path::new("/project/__tests__/app.js")));
        }

        #[test]
        fn test_rust_test_suffix() {
            assert!(!is_source_file(Path::new("main_test.rs")));
            assert!(!is_source_file(Path::new("main_tests.rs")));
        }

        #[test]
        fn test_rust_test_prefix() {
            assert!(!is_source_file(Path::new("test_main.rs")));
        }

        #[test]
        fn test_rust_test_infix() {
            assert!(!is_source_file(Path::new("module_test_helper.rs")));
        }

        #[test]
        fn test_js_test_suffix() {
            assert!(!is_source_file(Path::new("app.test.js")));
            assert!(!is_source_file(Path::new("app.spec.js")));
        }

        #[test]
        fn test_python_test_suffix() {
            assert!(!is_source_file(Path::new("main_test.py")));
        }

        #[test]
        fn test_java_test_suffix() {
            assert!(!is_source_file(Path::new("MainTest.java")));
        }

        #[test]
        fn test_regular_source_in_src_directory() {
            // Regular source files in src/ should be valid
            assert!(is_source_file(Path::new("/project/src/main.rs")));
            assert!(is_source_file(Path::new("/project/src/lib/core.rs")));
        }
    }

    // ============================================================================
    // has_source_extension Tests (Private function tested via is_source_file)
    // ============================================================================

    mod extension_tests {
        use super::*;

        #[test]
        fn test_all_supported_extensions() {
            let extensions = [
                "rs", "js", "ts", "py", "java", "cpp", "c", "go", "kt", "swift", "php", "rb",
                "scala",
            ];

            for ext in extensions.iter() {
                let path = format!("file.{}", ext);
                assert!(
                    is_source_file(Path::new(&path)),
                    "Extension .{} should be recognized as source",
                    ext
                );
            }
        }

        #[test]
        fn test_uppercase_extensions() {
            // Extensions are case-sensitive in the current implementation
            // These should NOT match since we check lowercase
            let path = Path::new("FILE.RS");
            // The extension .RS won't match "rs"
            assert!(!is_source_file(path));
        }
    }

    // ============================================================================
    // Edge Cases and Error Paths
    // ============================================================================

    mod edge_cases {
        use super::*;

        #[test]
        fn test_path_with_no_components() {
            assert!(!is_source_file(Path::new("")));
        }

        #[test]
        fn test_path_with_dots_only() {
            assert!(!is_source_file(Path::new("...")));
            assert!(!is_source_file(Path::new("..")));
        }

        #[test]
        fn test_deeply_nested_path() {
            assert!(is_source_file(Path::new(
                "/a/very/deeply/nested/path/to/source.rs"
            )));
        }

        #[test]
        fn test_path_with_unicode() {
            // Unicode path should work if extension matches
            assert!(is_source_file(Path::new("/src/模块.rs")));
        }

        #[test]
        fn test_format_with_unicode_in_file_names() {
            let files = vec![create_test_file_metrics("src/文件.rs", 10, 0.5)];
            let analysis = create_test_analysis(files);

            let result = format_churn_markdown(&analysis);
            assert!(result.is_ok());
        }

        #[test]
        fn test_format_with_special_characters_in_paths() {
            let files = vec![create_test_file_metrics("src/file-name_123.rs", 10, 0.5)];
            let analysis = create_test_analysis(files);

            let result = format_churn_markdown(&analysis);
            assert!(result.is_ok());
            assert!(result.unwrap().contains("file-name_123.rs"));
        }

        #[test]
        fn test_empty_author_contributions_skips_section() {
            let mut analysis = create_test_analysis(vec![]);
            analysis.summary.author_contributions = HashMap::new();

            let result = format_churn_markdown(&analysis).unwrap();

            // Should not have author contributions section when empty
            assert!(!result.contains("## Author Contributions"));
        }

        #[test]
        fn test_sorts_authors_by_contribution() {
            let mut contributions = HashMap::new();
            contributions.insert("low_contributor".to_string(), 5);
            contributions.insert("high_contributor".to_string(), 100);
            contributions.insert("mid_contributor".to_string(), 50);

            let mut analysis = create_test_analysis(vec![]);
            analysis.summary.author_contributions = contributions;

            let result = format_churn_markdown(&analysis).unwrap();

            // high_contributor should appear before mid_contributor should appear before low_contributor
            let high_pos = result.find("high_contributor").unwrap();
            let mid_pos = result.find("mid_contributor").unwrap();
            let low_pos = result.find("low_contributor").unwrap();

            assert!(
                high_pos < mid_pos,
                "high_contributor should appear before mid_contributor"
            );
            assert!(
                mid_pos < low_pos,
                "mid_contributor should appear before low_contributor"
            );
        }

        #[test]
        fn test_limits_author_list_to_15() {
            let mut contributions = HashMap::new();
            for i in 0..20 {
                contributions.insert(format!("author_{:02}", i), 100 - i);
            }

            let mut analysis = create_test_analysis(vec![]);
            analysis.summary.author_contributions = contributions;

            let result = format_churn_markdown(&analysis).unwrap();

            // Count author rows (excluding header)
            let author_rows: Vec<&str> = result
                .lines()
                .filter(|line| line.starts_with("| author_"))
                .collect();

            assert!(
                author_rows.len() <= 15,
                "Should have at most 15 author rows"
            );
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::models::churn::{ChurnSummary, CodeChurnAnalysis, FileChurnMetrics};
    use chrono::Utc;
    use proptest::prelude::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    proptest! {
        #[test]
        fn prop_is_source_file_deterministic(path in "[a-z]{1,20}\\.[a-z]{1,5}") {
            let path = Path::new(&path);
            let result1 = is_source_file(path);
            let result2 = is_source_file(path);
            prop_assert_eq!(result1, result2, "is_source_file should be deterministic");
        }

        #[test]
        fn prop_rust_files_always_source(name in "[a-z]{1,20}") {
            let path_str = format!("src/{}.rs", name);
            let path = Path::new(&path_str);
            prop_assert!(
                is_source_file(path),
                "Any .rs file in src/ should be a source file"
            );
        }

        #[test]
        fn prop_test_directories_never_source(
            dir in prop_oneof![
                Just("tests"),
                Just("test"),
                Just("examples"),
                Just("benches"),
                Just("fixtures"),
                Just("__tests__")
            ],
            _name in "[a-z]{1,10}"
        ) {
            let path_str = format!("/{}/source.rs", dir);
            let path = Path::new(&path_str);
            prop_assert!(
                !is_source_file(path),
                "Files in {} directory should not be source files",
                dir
            );
        }

        #[test]
        fn prop_format_churn_markdown_always_returns_string(
            period_days in 1u32..365,
            total_commits in 0usize..10000
        ) {
            let summary = ChurnSummary {
                total_commits,
                total_files_changed: 0,
                hotspot_files: vec![],
                stable_files: vec![],
                author_contributions: HashMap::new(),
                mean_churn_score: 0.0,
                variance_churn_score: 0.0,
                stddev_churn_score: 0.0,
            };

            let analysis = CodeChurnAnalysis {
                generated_at: Utc::now(),
                period_days,
                repository_root: PathBuf::from("/test"),
                files: vec![],
                summary,
            };

            let result = format_churn_markdown(&analysis);
            prop_assert!(result.is_ok(), "format_churn_markdown should always succeed");

            let output = result.unwrap();
            prop_assert!(!output.is_empty(), "Output should not be empty");
            prop_assert!(
                output.contains("# Code Churn Analysis Report"),
                "Output should contain header"
            );
        }

        #[test]
        fn prop_summary_table_contains_accurate_counts(
            commits in 0usize..10000,
            files in 0usize..1000,
            hotspots in 0usize..50,
            stable in 0usize..50,
            authors in 0usize..100
        ) {
            let mut author_contributions = HashMap::new();
            for i in 0..authors {
                author_contributions.insert(format!("author_{}", i), 10);
            }

            let summary = ChurnSummary {
                total_commits: commits,
                total_files_changed: files,
                hotspot_files: (0..hotspots).map(|i| PathBuf::from(format!("hot{}.rs", i))).collect(),
                stable_files: (0..stable).map(|i| PathBuf::from(format!("stable{}.rs", i))).collect(),
                author_contributions,
                mean_churn_score: 0.5,
                variance_churn_score: 0.1,
                stddev_churn_score: 0.316,
            };

            let mut output = String::new();
            write_markdown_summary_table(&mut output, &summary).unwrap();

            prop_assert!(
                output.contains(&format!("| Total Commits | {} |", commits)),
                "Output should contain correct commit count"
            );
            prop_assert!(
                output.contains(&format!("| Files Changed | {} |", files)),
                "Output should contain correct files count"
            );
            prop_assert!(
                output.contains(&format!("| Hotspot Files | {} |", hotspots)),
                "Output should contain correct hotspot count"
            );
            prop_assert!(
                output.contains(&format!("| Stable Files | {} |", stable)),
                "Output should contain correct stable count"
            );
            prop_assert!(
                output.contains(&format!("| Contributing Authors | {} |", authors)),
                "Output should contain correct author count"
            );
        }

        #[test]
        fn prop_file_extensions_preserved_in_output(
            ext in prop_oneof![
                Just("rs"),
                Just("js"),
                Just("ts"),
                Just("py"),
                Just("java"),
                Just("go")
            ]
        ) {
            let file_path = format!("src/file.{}", ext);
            let metrics = FileChurnMetrics {
                path: PathBuf::from(&file_path),
                relative_path: file_path.clone(),
                commit_count: 10,
                unique_authors: vec!["dev".to_string()],
                additions: 100,
                deletions: 50,
                churn_score: 0.5,
                last_modified: Utc::now(),
                first_seen: Utc::now(),
            };

            let summary = ChurnSummary {
                total_commits: 10,
                total_files_changed: 1,
                hotspot_files: vec![],
                stable_files: vec![],
                author_contributions: HashMap::new(),
                mean_churn_score: 0.5,
                variance_churn_score: 0.0,
                stddev_churn_score: 0.0,
            };

            let analysis = CodeChurnAnalysis {
                generated_at: Utc::now(),
                period_days: 30,
                repository_root: PathBuf::from("/test"),
                files: vec![metrics],
                summary,
            };

            let result = format_churn_markdown(&analysis).unwrap();
            prop_assert!(
                result.contains(&format!(".{}", ext)),
                "Output should contain the file extension .{}",
                ext
            );
        }
    }
}
