//! Toyota Way: Extracted Provability Analysis Handler
//! Complexity: Reduced from 19 to individual functions ≤8
//! Purpose: Function formal provability analysis with confidence scoring

use crate::cli::enums::ProvabilityOutputFormat;
use crate::services::lightweight_provability_analyzer::ProofSummary;
use anyhow::Result;
use std::path::PathBuf;

/// Analyzes function provability using lightweight formal methods analysis.
///
/// This handler performs provability analysis on functions to determine their
/// formal verification potential using static analysis techniques.
///
/// # Toyota Way: Single Responsibility
/// - Dedicated handler for provability analysis only
/// - Clear separation from complexity analysis  
/// - Focused on formal methods and verification
///
/// # Parameters
///
/// * `project_path` - Root directory of the project to analyze
/// * `functions` - Specific functions to analyze (empty = all functions)
/// * `_analysis_depth` - Depth of analysis (currently unused)
/// * `format` - Output format for results
/// * `high_confidence_only` - Filter to high-confidence results only
/// * `include_evidence` - Include supporting evidence in output
/// * `output` - Optional output file path
/// * `top_files` - Number of top files to include in summary
///
/// # Returns
///
/// Configuration for provability analysis (SPRINT-22)
#[derive(Debug, Clone)]
pub struct ProvabilityConfig {
    pub project_path: PathBuf,
    pub functions: Vec<String>,
    pub analysis_depth: usize,
    pub format: ProvabilityOutputFormat,
    pub high_confidence_only: bool,
    pub include_evidence: bool,
    pub output: Option<PathBuf>,
    pub top_files: usize,
}

/// * `Ok(())` - Analysis completed successfully
/// * `Err(anyhow::Error)` - Analysis failed with detailed error context (cognitive complexity ≤8)
pub async fn handle_analyze_provability(config: ProvabilityConfig) -> Result<()> {
    use crate::services::lightweight_provability_analyzer::LightweightProvabilityAnalyzer;

    eprintln!("🔬 Analyzing function provability...");

    let analyzer = LightweightProvabilityAnalyzer::new();
    let function_ids = resolve_function_targets(&config).await?;
    let summaries = run_provability_analysis(&analyzer, &function_ids).await?;
    let filtered_summaries = prepare_filtered_summaries(&summaries, config.high_confidence_only);
    let content = format_provability_output(&function_ids, &filtered_summaries, &config)?;
    
    write_provability_output(&content, &config.output).await?;

    Ok(())
}

/// Resolve function targets for analysis (cognitive complexity ≤6)
async fn resolve_function_targets(config: &ProvabilityConfig) -> Result<Vec<crate::services::lightweight_provability_analyzer::FunctionId>> {
    use crate::cli::provability_helpers::*;
    
    if config.functions.is_empty() {
        discover_project_functions(&config.project_path).await
    } else {
        let mut ids = Vec::new();
        for spec in &config.functions {
            ids.push(parse_function_spec(spec, &config.project_path)?);
        }
        Ok(ids)
    }
}

/// Run provability analysis on function targets (cognitive complexity ≤3)
async fn run_provability_analysis(
    analyzer: &crate::services::lightweight_provability_analyzer::LightweightProvabilityAnalyzer,
    function_ids: &[crate::services::lightweight_provability_analyzer::FunctionId],
) -> Result<Vec<ProofSummary>> {
    let summaries = analyzer.analyze_incrementally(function_ids).await;
    eprintln!("✅ Analyzed {} functions", summaries.len());
    Ok(summaries)
}

/// Prepare filtered summaries for output (cognitive complexity ≤3)
fn prepare_filtered_summaries(
    summaries: &[ProofSummary],
    high_confidence_only: bool,
) -> Vec<ProofSummary> {
    use crate::cli::provability_helpers::filter_summaries;
    let filtered = filter_summaries(summaries, high_confidence_only);
    filtered.into_iter().cloned().collect()
}

/// Format provability output based on config (cognitive complexity ≤8)
fn format_provability_output(
    function_ids: &[crate::services::lightweight_provability_analyzer::FunctionId],
    summaries: &[ProofSummary],
    config: &ProvabilityConfig,
) -> Result<String> {
    use crate::cli::provability_helpers::*;
    
    match config.format {
        ProvabilityOutputFormat::Json => {
            format_provability_json(function_ids, summaries, config.include_evidence)
        }
        ProvabilityOutputFormat::Summary => {
            format_provability_summary(function_ids, summaries, config.top_files)
        }
        ProvabilityOutputFormat::Full => {
            format_provability_detailed(function_ids, summaries, config.include_evidence)
        }
        ProvabilityOutputFormat::Sarif => {
            format_provability_sarif(function_ids, summaries)
        }
        ProvabilityOutputFormat::Markdown => {
            format_provability_detailed(function_ids, summaries, config.include_evidence)
        }
    }
}

/// Write provability output to file or stdout (cognitive complexity ≤4)
async fn write_provability_output(content: &str, output_path: &Option<PathBuf>) -> Result<()> {
    if let Some(output_path) = output_path {
        tokio::fs::write(output_path, content).await?;
        eprintln!(
            "✅ Provability analysis written to: {}",
            output_path.display()
        );
    } else {
        println!("{}", content);
    }
    Ok(())
}
