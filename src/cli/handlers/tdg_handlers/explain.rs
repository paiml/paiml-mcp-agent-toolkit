#![cfg_attr(coverage_nightly, coverage(off))]
//! TDG explain mode: function-level complexity breakdown
//!
//! Issue #78: Detailed explanation mode with per-function analysis
//! and actionable recommendations.

use super::display::format_explain_output;
use super::formatting::write_tdg_output;
use super::quality_gates::execute_tdg_analysis;
use super::TdgCommandConfig;
use crate::tdg::TdgAnalyzer;
use anyhow::Result;

/// Handle TDG explain mode with function-level complexity breakdown (Issue #78)
pub(super) async fn handle_explain_mode(
    analyzer: &TdgAnalyzer,
    config: &TdgCommandConfig,
) -> Result<()> {
    use crate::tdg::explain::ExplainedTDGScore;

    // First get the base TDG score
    let score = execute_tdg_analysis(analyzer, config).await?;

    // Create explained score
    let mut explained = ExplainedTDGScore::new(score.clone());

    // Analyze function-level complexity (only for single files, requires rust-ast)
    #[cfg(feature = "rust-ast")]
    if config.path.is_file() && config.path.extension().is_some_and(|e| e == "rs") {
        use crate::tdg::function_analyzer::FunctionAnalyzer;
        use crate::tdg::recommendation_engine::generate_recommendations;
        let mut func_analyzer = FunctionAnalyzer::new()?;
        let functions = func_analyzer.analyze_file(&config.path)?;

        for func in functions {
            explained.add_function(func);
        }

        // Apply threshold filter
        explained.filter_functions_by_threshold(config.threshold);
        explained.sort_functions_by_impact();

        // Generate recommendations
        let recommendations = generate_recommendations(&explained);
        for rec in recommendations {
            explained.add_recommendation(rec);
        }
        explained.sort_recommendations();
    }

    // Format and output
    let output_str = format_explain_output(&explained, config)?;
    write_tdg_output(&output_str, config)?;

    Ok(())
}
