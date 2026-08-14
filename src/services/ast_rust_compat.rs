#![cfg_attr(coverage_nightly, coverage(off))]
//! Compatibility shim for `ast_rust` module during migration to new AST architecture
//!
//! This module provides backward compatibility for services still using the old AST API.
//! It will be removed once all services are migrated to the new `ast::` module.

use anyhow::Result;
use std::path::Path;

use crate::models::error::TemplateError;
use crate::services::accurate_complexity_analyzer::AccurateComplexityAnalyzer;
use crate::services::complexity::{ComplexityMetrics, FileComplexityMetrics, FunctionComplexity};
use crate::services::context::FileContext;
use crate::services::file_classifier::FileClassifier;
use crate::services::source_line_index::LineSpan;

// Import the enhanced visitor for real AST extraction
use crate::services::enhanced_ast_visitor::EnhancedAstVisitor;

/// Analyze a Rust file and return complexity metrics (compatibility function)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_rust_file_with_complexity(
    path: &Path,
) -> Result<FileComplexityMetrics, TemplateError> {
    analyze_rust_file_with_complexity_and_classifier(path, None).await
}

/// Analyze a Rust file with optional classifier (compatibility function)
#[allow(clippy::cast_possible_truncation)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_rust_file_with_complexity_and_classifier(
    path: &Path,
    _classifier: Option<&FileClassifier>,
) -> Result<FileComplexityMetrics, TemplateError> {
    // Use the accurate complexity analyzer for real metrics
    let analyzer = AccurateComplexityAnalyzer::new();
    let accurate_result = analyzer
        .analyze_file(path)
        .await
        .map_err(|e| TemplateError::InvalidUtf8(e.to_string()))?;

    // Convert accurate metrics to old format.
    //
    // #652/#656: this loop used to fabricate the extent of every function.
    // Non-last functions were given `next.line_start - 1` (so a 3-line function
    // followed by a blank line was reported as longer than it is), and the LAST
    // function of every file was given `line_start + 50` with `lines: 50` — a
    // constant, routinely past EOF (a 1-line file reported `line_end: 51`).
    // The file's own `lines` was then copied from that invented end, reporting a
    // 13-line file as 61 lines. All extents now come from the measured spans in
    // `source_line_index`, which agree with `pmat extract --list`.
    let mut function_metrics = Vec::new();
    let mut total_cyclomatic = 0u32;
    let mut total_cognitive = 0u32;
    let mut max_nesting = 0u32;

    for func in &accurate_result.functions {
        total_cyclomatic += func.cyclomatic_complexity;
        total_cognitive += func.cognitive_complexity;
        max_nesting = max_nesting.max(func.max_nesting);

        let span = LineSpan {
            start: func.line_start,
            end: func.line_end,
        };

        function_metrics.push(FunctionComplexity {
            name: func.name.clone(),
            line_start: func.line_start,
            line_end: func.line_end,
            metrics: ComplexityMetrics {
                cyclomatic: clamp_u16(func.cyclomatic_complexity),
                cognitive: clamp_u16(func.cognitive_complexity),
                nesting_max: clamp_u8(func.max_nesting),
                // 0 when the definition could not be located — never a filler.
                lines: clamp_u16(span.line_count()),
                halstead: None,
            },
        });
    }

    // #931: `total_complexity` is the SUM of the file's function complexities.
    //
    // This used to divide by `function_metrics.len()` and store the integer
    // MEAN in the field literally named `total_complexity`, while the other
    // producer of the same field (`cli::language_analyzer::mod::
    // calculate_total_complexity`, which handles include!() fragments and
    // non-Rust files) stored the sum. One field carried two meanings in one
    // report — 2,084 files summed and 762 averaged — so no consumer could be
    // right, and "Top Files by Complexity", which sorts on this field, was
    // inverted: pmat's own build.rs (61 functions, true sum 157) reported 2
    // and ranked below 792 strictly simpler files. Under the mean, adding
    // trivial helpers to a file LOWERED its reported complexity.
    //
    // `nesting_max` stays a maximum (a depth does not add up) and `lines` is
    // the file length; only the two additive counters are summed.

    Ok(FileComplexityMetrics {
        path: path.display().to_string(),
        total_complexity: ComplexityMetrics {
            cyclomatic: clamp_u16(total_cyclomatic),
            cognitive: clamp_u16(total_cognitive),
            nesting_max: clamp_u8(max_nesting),
            // The real file length, not the last function's invented end.
            lines: clamp_u16(accurate_result.total_lines),
            halstead: None,
        },
        functions: function_metrics,
        classes: Vec::new(), // Rust doesn't have classes in the traditional sense
    })
}

/// Saturating narrow — a truncating `as u16` would wrap a 70,000-line file to a
/// small number, which reads as a measurement rather than an overflow.
fn clamp_u16(value: u32) -> u16 {
    u16::try_from(value).unwrap_or(u16::MAX)
}

fn clamp_u8(value: u32) -> u8 {
    u8::try_from(value).unwrap_or(u8::MAX)
}

/// Analyze a Rust file and return context (compatibility function)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_rust_file(path: &Path) -> Result<FileContext, TemplateError> {
    analyze_rust_file_with_classifier(path, None).await
}

/// Analyze a Rust file with optional classifier and return context (compatibility function)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_rust_file_with_classifier(
    path: &Path,
    _classifier: Option<&FileClassifier>,
) -> Result<FileContext, TemplateError> {
    // Read the file content
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(TemplateError::Io)?;

    // Parse the Rust code with syn
    let syntax_tree =
        syn::parse_file(&content).map_err(|e| TemplateError::InvalidUtf8(e.to_string()))?;

    // Use enhanced visitor to extract real AST information
    let visitor = EnhancedAstVisitor::new(path, &content);
    let items = visitor.extract_items(&syntax_tree);

    Ok(FileContext {
        path: path.display().to_string(),
        language: "rust".to_string(),
        items,
        complexity_metrics: None,
    })
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod fabrication_tests {
    use super::*;

    async fn metrics_for(source: &str) -> FileComplexityMetrics {
        let dir = tempfile::TempDir::new().unwrap();
        let file = dir.path().join("lib.rs");
        std::fs::write(&file, source).unwrap();
        analyze_rust_file_with_complexity(&file).await.unwrap()
    }

    /// #652 exact fixture: a 13-line file with two functions.
    #[tokio::test]
    async fn test_extents_match_the_source_not_a_50_line_guess() {
        let source = concat!(
            "pub fn cc_six(a: i32) -> i32 {\n",
            "    let mut t = 0;\n",
            "    if a > 1 { t += 1; }\n",
            "    if a > 2 { t += 1; }\n",
            "    if a > 3 { t += 1; }\n",
            "    if a > 4 { t += 1; }\n",
            "    if a > 5 { t += 1; }\n",
            "    t\n",
            "}\n",
            "\n",
            "pub fn cc_one(a: i32) -> i32 {\n",
            "    a + 1\n",
            "}\n",
        );

        let metrics = metrics_for(source).await;

        assert_eq!(
            metrics.total_complexity.lines, 13,
            "file is 13 lines; used to be reported as 61"
        );

        let cc_six = &metrics.functions[0];
        assert_eq!((cc_six.line_start, cc_six.line_end), (1, 9));
        assert_eq!(cc_six.metrics.lines, 9);

        let cc_one = &metrics.functions[1];
        assert_eq!(
            (cc_one.line_start, cc_one.line_end),
            (11, 13),
            "last function used to be reported as 11-61"
        );
        assert_eq!(cc_one.metrics.lines, 3, "used to be the constant 50");
    }

    /// #656 exact fixture: a single-line file.
    #[tokio::test]
    async fn test_line_end_never_exceeds_the_file() {
        let metrics = metrics_for("fn only_one() -> i32 { 42 }\n").await;

        assert_eq!(metrics.total_complexity.lines, 1);
        let only = &metrics.functions[0];
        assert_eq!(only.line_start, 1);
        assert_eq!(only.line_end, 1, "used to be 51, i.e. 50 lines past EOF");
        assert_eq!(only.metrics.lines, 1);
    }

    /// Every function's end must be inside the file, for any shape of file.
    #[tokio::test]
    async fn test_all_extents_stay_within_the_file() {
        let source = concat!(
            "fn a() {}\n",
            "\n",
            "fn b(x: i32) -> i32 {\n",
            "    if x > 0 { 1 } else { 0 }\n",
            "}\n",
            "\n",
            "fn another_orphan() -> u8 {\n",
            "    3\n",
            "}\n",
        );
        let metrics = metrics_for(source).await;
        let total = metrics.total_complexity.lines;
        assert_eq!(total, 9);

        for func in &metrics.functions {
            assert!(
                func.line_start >= 1 && func.line_end <= u32::from(total),
                "{} spans {}-{} in a {}-line file",
                func.name,
                func.line_start,
                func.line_end,
                total
            );
            assert!(func.line_end >= func.line_start);
        }
    }

    /// output_derived_from_input: no functions in, no functions out.
    #[tokio::test]
    async fn test_empty_file_reports_zero_lines_and_no_functions() {
        let metrics = metrics_for("").await;
        assert!(metrics.functions.is_empty());
        assert_eq!(metrics.total_complexity.lines, 0);
    }
}

/// #931 — `total_complexity` was the integer MEAN on this (AST) path and the
/// SUM on the include!()-fragment path, in the same report.
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod total_complexity_is_a_total_tests {
    use super::*;

    async fn metrics_for(source: &str) -> FileComplexityMetrics {
        let dir = tempfile::TempDir::new().unwrap();
        let file = dir.path().join("lib.rs");
        std::fs::write(&file, source).unwrap();
        analyze_rust_file_with_complexity(&file).await.unwrap()
    }

    fn sum_of_functions(metrics: &FileComplexityMetrics) -> u16 {
        metrics.functions.iter().map(|f| f.metrics.cyclomatic).sum()
    }

    /// The exact shape called out with #931: one branchy function plus a
    /// trivial free function. Sum 5; the mean floor was 2, which is neither
    /// the sum nor the maximum.
    #[tokio::test]
    async fn test_file_total_is_the_sum_not_the_mean() {
        let source = concat!(
            "pub fn branchy(a: i32) -> i32 {\n",
            "    if a > 1 { return 1; }\n",
            "    if a > 2 { return 2; }\n",
            "    0\n",
            "}\n",
            "\n",
            "pub fn trivial() -> i32 {\n",
            "    7\n",
            "}\n",
        );
        let metrics = metrics_for(source).await;

        assert_eq!(metrics.functions.len(), 2);
        let sum = sum_of_functions(&metrics);
        assert_eq!(
            metrics.total_complexity.cyclomatic, sum,
            "total_complexity must equal the sum of the file's functions; \
             the integer mean used to be reported here"
        );
        assert!(
            metrics.total_complexity.cyclomatic > 2,
            "the mean floor of this fixture was 2 — a value that is neither \
             the sum ({sum}) nor the maximum"
        );
    }

    /// #931's inverted ranking, reduced: a file holding one 60-branch function
    /// plus 60 trivial helpers must not report LESS than a file whose worst
    /// function is a 3.
    #[tokio::test]
    async fn test_trivial_helpers_cannot_lower_a_file_total() {
        let mut worst = String::from("pub fn beast(k: i64) -> i64 {\n    let mut r = 0i64;\n");
        for i in 0..60 {
            worst.push_str(&format!("    if k > {i} {{ r += {i}; }}\n"));
        }
        worst.push_str("    r\n}\n");
        let beast_only = metrics_for(&worst).await;

        for i in 0..60 {
            worst.push_str(&format!("pub fn t{i}() -> i64 {{ {i} }}\n"));
        }
        let with_helpers = metrics_for(&worst).await;

        assert_eq!(
            with_helpers.total_complexity.cyclomatic,
            sum_of_functions(&with_helpers)
        );
        assert!(
            with_helpers.total_complexity.cyclomatic >= beast_only.total_complexity.cyclomatic,
            "adding 60 trivial helpers dropped the reported total from {} to {} \
             (the mean collapsed a true sum of 122 to 2)",
            beast_only.total_complexity.cyclomatic,
            with_helpers.total_complexity.cyclomatic
        );

        let mild: String = (0..8)
            .map(|i| {
                format!(
                    "pub fn m{i}(a:i32,b:i32)->i32{{ if a>b {{ a }} else if a<b {{ b }} else {{ 0 }} }}\n"
                )
            })
            .collect();
        let mild = metrics_for(&mild).await;
        assert!(
            with_helpers.total_complexity.cyclomatic > mild.total_complexity.cyclomatic,
            "the file holding a 61-branch function reported {} while a file whose \
             worst function is a 3 reported {} — this is the inverted 'Top Files \
             by Complexity' ranking",
            with_helpers.total_complexity.cyclomatic,
            mild.total_complexity.cyclomatic
        );
    }

    /// Both producers of `total_complexity` must mean the same thing. The
    /// heuristic path (include!() fragments, non-Rust) already summed; the AST
    /// path averaged.
    #[tokio::test]
    async fn test_ast_and_heuristic_producers_agree_on_the_meaning() {
        let source = concat!(
            "pub fn a(x: i32) -> i32 {\n",
            "    if x > 0 { 1 } else { 0 }\n",
            "}\n",
            "pub fn b(x: i32) -> i32 {\n",
            "    if x > 0 { if x > 1 { 2 } else { 1 } } else { 0 }\n",
            "}\n",
            "pub fn c() -> i32 { 3 }\n",
        );

        let ast = metrics_for(source).await;
        assert_eq!(ast.total_complexity.cyclomatic, sum_of_functions(&ast));

        let heuristic = crate::cli::language_analyzer::analyze_with_heuristics(
            Path::new("fragment.rs"),
            source,
            crate::cli::language_analyzer::Language::Rust,
        )
        .unwrap();
        assert_eq!(
            heuristic.total_complexity.cyclomatic,
            sum_of_functions(&heuristic),
            "the include!()-fragment producer must also report a sum"
        );
    }
}
