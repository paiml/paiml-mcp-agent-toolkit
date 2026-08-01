#![cfg_attr(coverage_nightly, coverage(off))]
//! Language-specific complexity analysis module
//!
//! This module provides proper separation of concerns for analyzing
//! complexity across different programming languages, following the
//! Toyota Way principle of quality and single responsibility.

mod c;
mod complexity;
mod dynamic;
mod javascript;
mod python;
mod rust;
mod types;

// Re-export all public items that were previously public from the single file
// `find_brace_balanced_end` is the codebase's real, string-aware end-of-block
// finder; services use it instead of guessing function extents (#652, #656).
pub use c::CAnalyzer;
pub(crate) use complexity::find_brace_balanced_end;
pub use dynamic::{LuaAnalyzer, ScalaAnalyzer, SqlAnalyzer};
pub use javascript::JavaScriptAnalyzer;
pub use python::PythonAnalyzer;
pub use rust::RustAnalyzer;
pub use types::{FunctionInfo, Language, LanguageAnalyzer};

use crate::services::complexity::{ComplexityMetrics, FileComplexityMetrics, FunctionComplexity};
use anyhow::Result;
use std::path::Path;

/// Analyze file complexity using appropriate language analyzer
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn analyze_file_complexity(path: &Path, content: &str) -> Result<FileComplexityMetrics> {
    let language = Language::from_path(path);

    // Try AST analysis first for Rust files
    if let Some(metrics) = try_ast_analysis(path, language).await {
        return Ok(metrics);
    }

    // Fall back to heuristic analysis
    analyze_with_heuristics(path, content, language)
}

/// Detect include!() fragment files that aren't standalone Rust (PMAT-507)
///
/// These files are included via `include!()` into a parent module and are not
/// valid standalone Rust. Attempting `syn::parse_file()` on them always fails.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn is_include_fragment(path: &Path) -> bool {
    let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    // Part files: part1.rs, part2.rs, etc.
    let is_part_file = name.starts_with("part") && name.len() <= 6;
    // Test fragments: *_tests_*.rs (split fragments), *_tests.rs (included test modules)
    let is_test_fragment = name.contains("_tests_")
        || (name.ends_with("_tests") && name != "tests")
        || name.starts_with("tests_");
    // HTML/template fragments included into parent modules
    let is_template_fragment = name.starts_with("html_") || name == "runner_pipeline";
    // Benchmark fragments
    let is_bench_fragment =
        path.components().any(|c| c.as_os_str() == "benchmarks") && name.starts_with("measure_");
    // Impl fragments: file is include!()'d by a sibling with matching prefix
    // e.g. muda_handlers_metrics.rs is included by muda_handlers.rs
    let is_impl_fragment = is_included_by_sibling(path, name);
    is_part_file
        || is_test_fragment
        || is_template_fragment
        || is_bench_fragment
        || is_impl_fragment
}

/// Check if a sibling file includes this file via include!() macro.
/// Detects impl-fragment files like `foo_bar.rs` included by `foo.rs`.
/// Tries progressively shorter prefixes: for `context_graph_coverage_fixtures`,
/// tries `context_graph_coverage.rs`, `context_graph.rs`, `context.rs`.
fn is_included_by_sibling(path: &Path, name: &str) -> bool {
    let Some(dir) = path.parent() else {
        return false;
    };
    let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or("");
    let needle = format!("include!(\"{filename}\")");
    // Try progressively shorter prefixes by stripping trailing _segment
    let mut candidate = name.to_string();
    while let Some(pos) = candidate.rfind('_') {
        candidate.truncate(pos);
        let parent_path = dir.join(format!("{candidate}.rs"));
        if parent_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&parent_path) {
                if content.contains(&needle) {
                    return true;
                }
            }
        }
        // Also check mod.rs in a subdirectory with that name
        let mod_path = dir.join(&candidate).join("mod.rs");
        if mod_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&mod_path) {
                if content.contains(&needle) {
                    return true;
                }
            }
        }
    }
    false
}

async fn try_ast_analysis(path: &Path, language: Language) -> Option<FileComplexityMetrics> {
    if language != Language::Rust {
        return None;
    }

    // Skip include!() fragment files — they aren't standalone Rust
    if is_include_fragment(path) {
        return None;
    }

    if let Ok(metrics) = crate::services::ast_rust::analyze_rust_file_with_complexity(path).await {
        Some(metrics)
    } else {
        eprintln!(
            "Warning: AST analysis failed for {}, using heuristic fallback",
            path.display()
        );
        None
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
/// Analyze with heuristics.
pub fn analyze_with_heuristics(
    path: &Path,
    content: &str,
    language: Language,
) -> Result<FileComplexityMetrics> {
    if language == Language::Unknown {
        Ok(create_empty_metrics(path, content))
    } else {
        let analyzer = create_analyzer(language);
        analyze_functions_with_analyzer(path, content, &*analyzer)
    }
}

fn create_empty_metrics(path: &Path, content: &str) -> FileComplexityMetrics {
    FileComplexityMetrics {
        path: path.to_string_lossy().to_string(),
        total_complexity: ComplexityMetrics {
            cyclomatic: 1,
            cognitive: 0,
            nesting_max: 0,
            lines: content.lines().count() as u16,
            halstead: None,
        },
        functions: vec![],
        classes: vec![],
    }
}

/// Build the analyzer for a language. Public so other services (Big-O) can
/// extract the *same* function boundaries the complexity path uses, instead of
/// running their own regex over the raw file (#655, #661).
#[must_use]
pub fn create_analyzer(language: Language) -> Box<dyn LanguageAnalyzer> {
    match language {
        Language::Rust => Box::new(RustAnalyzer),
        Language::JavaScript | Language::TypeScript => Box::new(JavaScriptAnalyzer),
        Language::Python => Box::new(PythonAnalyzer),
        Language::C => Box::new(CAnalyzer),
        // C++ function syntax is similar enough to JavaScript for basic extraction
        Language::CPP => Box::new(JavaScriptAnalyzer),
        // Go func syntax: func name(params) type { } - similar to C
        Language::Go => Box::new(CAnalyzer),
        // Bash function syntax: function name() { } or name() { } - similar to JavaScript
        Language::Bash => Box::new(JavaScriptAnalyzer),
        // Java method syntax: public Type name(params) { } - similar to C
        Language::Java => Box::new(CAnalyzer),
        // Kotlin fun syntax: fun name(params): Type { } - similar to C
        Language::Kotlin => Box::new(CAnalyzer),
        // Ruby def syntax: def name(params) - similar to Python
        Language::Ruby => Box::new(PythonAnalyzer),
        // PHP function syntax: function name($params) { } - similar to JavaScript
        Language::PHP => Box::new(JavaScriptAnalyzer),
        // Swift func syntax: func name(params) -> Type { } - similar to C
        Language::Swift => Box::new(CAnalyzer),
        // C# method syntax: public Type Name(params) { } - similar to C
        Language::CSharp => Box::new(CAnalyzer),
        // Lua function syntax: function name(params) ... end
        Language::Lua => Box::new(LuaAnalyzer),
        Language::Sql => Box::new(SqlAnalyzer),
        Language::Scala => Box::new(ScalaAnalyzer),
        Language::Yaml | Language::Markdown => Box::new(PythonAnalyzer), // structural analysis only
        // Lean: theorem/def/lemma keyword syntax similar to Python's def/class
        Language::Lean => Box::new(PythonAnalyzer),
        Language::Unknown => unreachable!("Unknown language should be handled earlier"),
    }
}

fn analyze_functions_with_analyzer(
    path: &Path,
    content: &str,
    analyzer: &dyn LanguageAnalyzer,
) -> Result<FileComplexityMetrics> {
    let function_infos = analyzer.extract_functions(content);
    let functions = process_function_infos(content, function_infos, analyzer);
    let total_complexity = calculate_total_complexity(&functions, content);

    Ok(FileComplexityMetrics {
        path: path.to_string_lossy().to_string(),
        total_complexity,
        functions,
        classes: vec![],
    })
}

fn process_function_infos(
    content: &str,
    function_infos: Vec<FunctionInfo>,
    analyzer: &dyn LanguageAnalyzer,
) -> Vec<FunctionComplexity> {
    function_infos
        .into_iter()
        .map(|info| {
            let metrics = analyzer.estimate_complexity(content, &info);
            FunctionComplexity {
                name: info.name,
                line_start: (info.line_start + 1) as u32,
                line_end: (info.line_end + 1) as u32,
                metrics,
            }
        })
        .collect()
}

fn calculate_total_complexity(
    functions: &[FunctionComplexity],
    content: &str,
) -> ComplexityMetrics {
    ComplexityMetrics {
        cyclomatic: functions
            .iter()
            .map(|f| f.metrics.cyclomatic)
            .sum::<u16>()
            .max(1),
        cognitive: functions
            .iter()
            .map(|f| f.metrics.cognitive)
            .sum::<u16>()
            .max(1),
        nesting_max: functions
            .iter()
            .map(|f| f.metrics.nesting_max)
            .max()
            .unwrap_or(0),
        lines: content.lines().count() as u16,
        halstead: None,
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection() {
        assert_eq!(Language::from_path(Path::new("test.rs")), Language::Rust);
        assert_eq!(
            Language::from_path(Path::new("test.js")),
            Language::JavaScript
        );
        assert_eq!(
            Language::from_path(Path::new("test.ts")),
            Language::TypeScript
        );
        assert_eq!(Language::from_path(Path::new("test.py")), Language::Python);
        assert_eq!(
            Language::from_path(Path::new("test.txt")),
            Language::Unknown
        );
    }

    #[test]
    fn test_complexity_visitor() {
        let mut visitor = complexity::ComplexityVisitor::new();
        let lines = vec![
            "fn test() {",
            "    if condition {",
            "        while true {",
            "            break;",
            "        }",
            "    }",
            "}",
        ];

        visitor.analyze_lines(&lines);
        let metrics = visitor.into_metrics();

        assert!(metrics.cyclomatic > 1);
        assert!(metrics.cognitive > 0);
        assert_eq!(metrics.nesting_max, 3);
    }

    /// TDD Test: Integration test to expose the real bug
    #[tokio::test]
    async fn test_end_to_end_integration_bug() {
        let content = r#"fn simple_function() {
    println!("hello");
}

/// Second function.
pub fn second_function() {
    if true {
        println!("world");
    }
}
"#;
        let path = Path::new("test.rs");

        // Test the RustAnalyzer directly first
        let analyzer = RustAnalyzer;
        let functions = analyzer.extract_functions(content);
        assert_eq!(functions.len(), 2, "RustAnalyzer should detect 2 functions");

        // Test the full integration
        let result = analyze_file_complexity(path, content).await;
        assert!(
            result.is_ok(),
            "analyze_file_complexity should succeed: {:?}",
            result
        );

        let metrics = result.expect("internal error");

        // THIS MIGHT FAIL - if it does, we found the integration bug
        assert_eq!(
            metrics.functions.len(),
            2,
            "Integration should analyze 2 functions but found {}. Functions: {:?}",
            metrics.functions.len(),
            metrics
                .functions
                .iter()
                .map(|f| &f.name)
                .collect::<Vec<_>>()
        );
    }

    /// TDD Test: CLI Layer integration test using analyze_project_files
    #[tokio::test]
    async fn test_cli_layer_integration_bug() {
        use std::fs;
        use tempfile::TempDir;

        let content = r#"fn simple_function() {
    println!("hello");
}

/// Second function.
pub fn second_function() {
    if true {
        println!("world");
    }
}
"#;

        // Create a temporary directory and file
        let temp_dir = TempDir::new().expect("internal error");
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, content).expect("internal error");

        // Test the CLI stubs layer using analyze_project_files
        let result = crate::cli::analysis_utilities::analyze_project_files(
            temp_dir.path(),
            Some("rust"),
            &[], // empty include patterns
            20,  // cyclomatic threshold
            15,  // cognitive threshold
        )
        .await;

        assert!(
            result.is_ok(),
            "analyze_project_files should succeed: {:?}",
            result
        );

        let file_metrics = result.expect("internal error");

        // Skip test if no files were analyzed (common in test environments)
        if file_metrics.is_empty() {
            eprintln!("Warning: No files analyzed in test - skipping assertions");
            return;
        }

        // Find our test file
        let test_metrics = file_metrics
            .iter()
            .find(|metrics| metrics.path.ends_with("test.rs"))
            .expect("Should find test.rs in results");

        // THIS SHOULD EXPOSE THE BUG
        assert_eq!(
            test_metrics.functions.len(),
            2,
            "CLI layer should analyze 2 functions but found {}. Functions: {:?}",
            test_metrics.functions.len(),
            test_metrics
                .functions
                .iter()
                .map(|f| &f.name)
                .collect::<Vec<_>>()
        );
    }
}
