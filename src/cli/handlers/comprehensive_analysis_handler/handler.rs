#![cfg_attr(coverage_nightly, coverage(off))]

use super::helpers::{
    determine_analysis_path, enhance_results_if_needed, init_timing,
    report_completion_and_performance, run_orchestrated_analysis,
};
use super::output::output_results;
use super::types::ComprehensiveAnalysisConfig;
use anyhow::Result;

/// Refactored handler for comprehensive analysis using the orchestrator facade.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn handle_analyze_comprehensive(config: ComprehensiveAnalysisConfig) -> Result<()> {
    crate::status_eprintln!("🔍 Running comprehensive analysis...");
    let start = init_timing(config.perf);

    let analysis_path = determine_analysis_path(&config);
    // GH-663: a nonexistent path analysed zero files and this printed
    // "✓ Comprehensive analysis completed / Quality Score: 100.0% /
    // Code quality looks good!" with exit 0 — a perfect score for a tree that
    // was never there, which turns a typo'd CI path green.
    crate::cli::ensure_analysis_path_exists(&analysis_path)?;
    let result = run_orchestrated_analysis(analysis_path.clone(), &config).await?;
    // #1015. The guard above closes "the tree is not there"; this closes the
    // one behind it. An EMPTY but perfectly real directory reached this point
    // with `total_files: 0, total_issues: 0` and printed
    // "Quality Score: 100.0% / Code quality looks good!" with exit 0 — the
    // congratulation being issued over a tree nothing was read from.
    //
    // Comprehensive is a composite of complexity + dead-code + SATD, and
    // `analyze satd` ALREADY refuses this case ("no source files were found …
    // This is not a clean result"). Comprehensive caught that refusal, printed
    // it as "Warning: satd analysis failed: …", and then scored the tree
    // anyway: a passing command wrapped around a refusing one. It refuses with
    // the same sentence its own sub-analyzers use.
    crate::cli::ensure_source_files_were_analyzed(
        "comprehensive quality-score",
        &analysis_path,
        result.summary.total_files,
    )?;
    let enhanced_result = enhance_results_if_needed(result, &config).await?;

    report_completion_and_performance(start, &config, &enhanced_result);
    output_results(
        enhanced_result,
        config.format,
        config.executive_summary,
        config.output,
        config.top_files,
    )
    .await?;

    Ok(())
}
