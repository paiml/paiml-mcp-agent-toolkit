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

    // Calculate average complexity for the file
    let avg_cyclomatic = if function_metrics.is_empty() {
        1
    } else {
        total_cyclomatic / function_metrics.len() as u32
    };

    let avg_cognitive = if function_metrics.is_empty() {
        0
    } else {
        total_cognitive / function_metrics.len() as u32
    };

    Ok(FileComplexityMetrics {
        path: path.display().to_string(),
        total_complexity: ComplexityMetrics {
            cyclomatic: clamp_u16(avg_cyclomatic),
            cognitive: clamp_u16(avg_cognitive),
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
