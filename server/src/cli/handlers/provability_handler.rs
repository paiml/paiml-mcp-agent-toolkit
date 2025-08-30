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
/// * `Err(anyhow::Error)` - Analysis failed with detailed error context
pub async fn handle_analyze_provability(config: ProvabilityConfig) -> Result<()> {
    use crate::cli::provability_helpers::*;
    use crate::services::lightweight_provability_analyzer::LightweightProvabilityAnalyzer;

    eprintln!("🔬 Analyzing function provability...");

    // Create the analyzer
    let analyzer = LightweightProvabilityAnalyzer::new();

    // Get function IDs based on input
    let function_ids = if config.functions.is_empty() {
        discover_project_functions(&config.project_path).await?
    } else {
        let mut ids = Vec::new();
        for spec in &config.functions {
            ids.push(parse_function_spec(spec, &config.project_path)?);
        }
        ids
    };

    // Analyze the functions
    let summaries = analyzer.analyze_incrementally(&function_ids).await;
    eprintln!("✅ Analyzed {} functions", summaries.len());

    // Filter by confidence if requested
    let filtered_summaries = filter_summaries(&summaries, config.high_confidence_only);
    let filtered_summaries_owned: Vec<ProofSummary> =
        filtered_summaries.into_iter().cloned().collect();

    // Format output based on requested format
    let content = match config.format {
        ProvabilityOutputFormat::Json => {
            format_provability_json(&function_ids, &filtered_summaries_owned, config.include_evidence)?
        }
        ProvabilityOutputFormat::Summary => {
            format_provability_summary(&function_ids, &filtered_summaries_owned, config.top_files)?
        }
        ProvabilityOutputFormat::Full => {
            format_provability_detailed(&function_ids, &filtered_summaries_owned, config.include_evidence)?
        }
        ProvabilityOutputFormat::Sarif => {
            format_provability_sarif(&function_ids, &filtered_summaries_owned)?
        }
        ProvabilityOutputFormat::Markdown => {
            format_provability_detailed(&function_ids, &filtered_summaries_owned, config.include_evidence)?
        }
    };

    // Write output
    if let Some(output_path) = config.output {
        tokio::fs::write(&output_path, &content).await?;
        eprintln!(
            "✅ Provability analysis written to: {}",
            output_path.display()
        );
    } else {
        println!("{}", content);
    }

    Ok(())
}
