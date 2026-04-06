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
