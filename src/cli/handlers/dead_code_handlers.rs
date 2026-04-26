//! Dead Code Analysis Handler
//!
//! Extracted from complexity_handlers.rs for file health compliance (CB-040).
//! Contains dead code analysis handler and all related helper functions.
//!
//! Submodule layout (include! pattern):
//! - dead_code_handlers_analysis.rs: Core analysis logic and cargo integration
//! - dead_code_handlers_output.rs: Output formatting (JSON, SARIF, summary, markdown)

#![cfg_attr(coverage_nightly, coverage(off))]
use crate::cli::DeadCodeOutputFormat;
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Configuration for dead code analysis
#[allow(clippy::too_many_arguments)]
struct DeadCodeAnalysisFilters {
    include_unreachable: bool,
    include_tests: bool,
    min_dead_lines: usize,
    top_files: Option<usize>,
    include: Vec<String>,
    exclude: Vec<String>,
    max_depth: usize,
}

/// Handle dead code analysis command - REFACTORED
/// Cognitive complexity reduced from 244 to ~10
#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_analyze_dead_code(
    path: PathBuf,
    format: DeadCodeOutputFormat,
    top_files: Option<usize>,
    include_unreachable: bool,
    min_dead_lines: usize,
    include_tests: bool,
    output: Option<PathBuf>,
    fail_on_violation: bool,
    max_percentage: f64,
    timeout: u64,
    include: Vec<String>,
    exclude: Vec<String>,
    max_depth: usize,
) -> Result<()> {
    eprintln!("☠️ Analyzing dead code in project...");
    eprintln!("⏰ Analysis timeout set to {timeout} seconds");

    // Apply include/exclude filters if specified
    if !include.is_empty() || !exclude.is_empty() {
        eprintln!("🔍 Applying file filters...");
        if !include.is_empty() {
            eprintln!("  Include patterns: {include:?}");
        }
        if !exclude.is_empty() {
            eprintln!("  Exclude patterns: {exclude:?}");
        }
    }

    // Run analysis with timeout
    let timeout_duration = tokio::time::Duration::from_secs(timeout);
    let result = tokio::time::timeout(timeout_duration, async {
        run_dead_code_analysis_with_filters(
            &path,
            DeadCodeAnalysisFilters {
                include_unreachable,
                include_tests,
                min_dead_lines,
                top_files,
                include,
                exclude,
                max_depth,
            },
        )
        .await
    })
    .await
    .map_err(|_| anyhow::anyhow!("Dead code analysis timed out after {timeout} seconds"))??;

    eprintln!(
        "📊 Analysis complete: {} files analyzed, {} with dead code",
        result.summary.total_files_analyzed, result.summary.files_with_dead_code
    );

    // Format output
    let formatted_output = format_dead_code_result(&result, &format)?;

    // Write output
    write_dead_code_output(formatted_output, output).await?;

    // Check for violations and exit with error code if requested
    if fail_on_violation {
        let dead_code_percentage = result.summary.dead_percentage;
        if dead_code_percentage > max_percentage as f32 {
            eprintln!(
                "\n❌ Dead code violations found: {dead_code_percentage:.1}% exceeds threshold of {max_percentage:.1}%"
            );
            std::process::exit(1);
        }
    }

    Ok(())
}

// --- Submodule includes ---

include!("dead_code_handlers_analysis.rs");
include!("dead_code_handlers_output.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod output_tests {
    //! Covers format_dead_code_* + write_*_section helpers in
    //! dead_code_handlers_output.rs (315 uncov on broad, 0% cov).
    //! The async write_dead_code_output is skipped (requires fs/IO setup).
    use super::*;
    use crate::models::dead_code::{
        ConfidenceLevel, DeadCodeItem, DeadCodeResult, DeadCodeSummary, DeadCodeType,
        FileDeadCodeMetrics,
    };

    fn item(ty: DeadCodeType, line: u32, name: &str, reason: &str) -> DeadCodeItem {
        DeadCodeItem {
            item_type: ty,
            name: name.to_string(),
            line,
            reason: reason.to_string(),
        }
    }

    fn file(path: &str, conf: ConfidenceLevel, items: Vec<DeadCodeItem>) -> FileDeadCodeMetrics {
        FileDeadCodeMetrics {
            path: path.to_string(),
            dead_lines: 10,
            total_lines: 100,
            dead_percentage: 10.0,
            dead_functions: 1,
            dead_classes: 0,
            dead_modules: 0,
            unreachable_blocks: 0,
            dead_score: 0.0,
            confidence: conf,
            items,
        }
    }

    fn empty_summary() -> DeadCodeSummary {
        DeadCodeSummary {
            total_files_analyzed: 5,
            files_with_dead_code: 0,
            total_dead_lines: 0,
            dead_percentage: 0.0,
            dead_functions: 0,
            dead_classes: 0,
            dead_modules: 0,
            unreachable_blocks: 0,
        }
    }

    fn full_summary() -> DeadCodeSummary {
        DeadCodeSummary {
            total_files_analyzed: 5,
            files_with_dead_code: 2,
            total_dead_lines: 20,
            dead_percentage: 12.0,
            dead_functions: 3,
            dead_classes: 2,
            dead_modules: 1,
            unreachable_blocks: 4,
        }
    }

    fn populated_result() -> DeadCodeResult {
        DeadCodeResult {
            summary: full_summary(),
            files: vec![
                file(
                    "src/a.rs",
                    ConfidenceLevel::High,
                    vec![
                        item(DeadCodeType::Function, 10, "f", "no callers"),
                        item(DeadCodeType::Class, 20, "C", "unused"),
                    ],
                ),
                file(
                    "src/b.rs",
                    ConfidenceLevel::Medium,
                    vec![item(DeadCodeType::Variable, 5, "x", "never read")],
                ),
                file(
                    "src/c.rs",
                    ConfidenceLevel::Low,
                    vec![item(
                        DeadCodeType::UnreachableCode,
                        99,
                        "block",
                        "after panic",
                    )],
                ),
            ],
            total_files: 5,
            analyzed_files: 5,
        }
    }

    fn empty_result() -> DeadCodeResult {
        DeadCodeResult {
            summary: empty_summary(),
            files: vec![],
            total_files: 5,
            analyzed_files: 5,
        }
    }

    // ── format_dead_code_result dispatcher ──

    #[test]
    fn test_format_dispatcher_json_arm() {
        let r = format_dead_code_result(&empty_result(), &DeadCodeOutputFormat::Json).unwrap();
        // serde_json output is non-empty even for empty data.
        assert!(r.contains("summary") || r.contains("files"));
    }

    #[test]
    fn test_format_dispatcher_sarif_arm() {
        let r = format_dead_code_result(&populated_result(), &DeadCodeOutputFormat::Sarif).unwrap();
        assert!(r.contains("\"version\": \"2.1.0\""));
        assert!(r.contains("dead-code"));
    }

    #[test]
    fn test_format_dispatcher_summary_arm() {
        let r =
            format_dead_code_result(&populated_result(), &DeadCodeOutputFormat::Summary).unwrap();
        assert!(!r.is_empty());
    }

    #[test]
    fn test_format_dispatcher_markdown_arm() {
        let r =
            format_dead_code_result(&populated_result(), &DeadCodeOutputFormat::Markdown).unwrap();
        assert!(r.contains("# Dead Code Analysis Report"));
    }

    // ── format_dead_code_as_sarif: confidence + item type arms ──

    #[test]
    fn test_sarif_levels_for_each_confidence() {
        // High → "error", Medium → "warning", Low → "note".
        let r = format_dead_code_as_sarif(&populated_result()).unwrap();
        assert!(r.contains("\"error\""));
        assert!(r.contains("\"warning\""));
        assert!(r.contains("\"note\""));
    }

    #[test]
    fn test_sarif_message_for_each_dead_code_type() {
        // Function/Class/Variable/UnreachableCode label arms.
        let r = format_dead_code_as_sarif(&populated_result()).unwrap();
        assert!(r.contains("Dead function"));
        assert!(r.contains("Dead class"));
        assert!(r.contains("Dead variable"));
        assert!(r.contains("Unreachable code"));
    }

    #[test]
    fn test_sarif_empty_files_yields_empty_results_array() {
        let r = format_dead_code_as_sarif(&empty_result()).unwrap();
        assert!(r.contains("\"results\": []"));
    }

    // ── format_dead_code_as_summary: branch arms ──

    #[test]
    fn test_summary_with_dead_functions_emits_breakdown_section() {
        let r = format_dead_code_as_summary(&populated_result()).unwrap();
        assert!(r.contains("Dead Code by Type"));
        assert!(r.contains("Top Files"));
    }

    #[test]
    fn test_summary_no_dead_functions_skips_breakdown() {
        let mut res = populated_result();
        res.summary.dead_functions = 0;
        let r = format_dead_code_as_summary(&res).unwrap();
        // Breakdown skipped when dead_functions == 0.
        assert!(!r.contains("Dead Code by Type"));
        // But Top Files still emitted (files non-empty).
        assert!(r.contains("Top Files"));
    }

    #[test]
    fn test_summary_empty_files_skips_top_files_section() {
        let r = format_dead_code_as_summary(&empty_result()).unwrap();
        assert!(!r.contains("Top Files"));
    }

    // ── format_dead_code_as_markdown: section gating ──

    #[test]
    fn test_markdown_with_full_data_emits_all_sections() {
        let r = format_dead_code_as_markdown(&populated_result()).unwrap();
        assert!(r.contains("# Dead Code Analysis Report"));
        assert!(r.contains("## Summary"));
        assert!(r.contains("## Dead Code Breakdown"));
        assert!(r.contains("## File Details"));
        assert!(r.contains("## Recommendations"));
    }

    #[test]
    fn test_markdown_empty_skips_breakdown_and_files() {
        let r = format_dead_code_as_markdown(&empty_result()).unwrap();
        // Always includes summary + recommendations.
        assert!(r.contains("## Summary"));
        assert!(r.contains("## Recommendations"));
        // Skipped when dead_functions == 0 and files empty.
        assert!(!r.contains("## Dead Code Breakdown"));
        assert!(!r.contains("## File Details"));
    }

    #[test]
    fn test_markdown_file_details_section_takes_first_20_files() {
        let mut res = populated_result();
        // Bloat to 30 files; details section caps at 20.
        for i in 0..30 {
            res.files.push(file(
                &format!("src/extra-{i}.rs"),
                ConfidenceLevel::High,
                vec![],
            ));
        }
        let r = format_dead_code_as_markdown(&res).unwrap();
        // First file always included.
        assert!(r.contains("src/a.rs"));
        // 20-cap means "src/extra-29.rs" must NOT appear.
        assert!(!r.contains("src/extra-29.rs"));
    }

    #[test]
    fn test_summary_top_files_section_takes_first_10_files() {
        let mut res = populated_result();
        for i in 0..15 {
            res.files.push(file(
                &format!("src/extra-{i}.rs"),
                ConfidenceLevel::High,
                vec![],
            ));
        }
        let r = format_dead_code_as_summary(&res).unwrap();
        // 10-file cap → extra-14 must NOT appear.
        assert!(!r.contains("src/extra-14.rs"));
    }

    #[test]
    fn test_recommendations_section_is_static_text() {
        // Pure static-text helper; no inputs.
        let r = format_dead_code_recommendations_section();
        assert!(r.contains("## Recommendations"));
        assert!(r.contains("High Confidence Dead Code"));
        assert!(r.contains("Test Coverage"));
    }
}
