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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn handle_analyze_provability(config: ProvabilityConfig) -> Result<()> {
    use crate::services::lightweight_provability_analyzer::LightweightProvabilityAnalyzer;

    // GH-666: a nonexistent path discovered zero functions and this reported
    // "📊 Found 0 source files / ✓ Analyzed 0 functions" with exit 0 — a
    // successful empty analysis of a tree that was never there.
    crate::cli::ensure_analysis_path_exists(&config.project_path)?;

    use crate::cli::colors as c;
    eprintln!("{}", c::dim("🔬 Analyzing function provability..."));

    let analyzer = LightweightProvabilityAnalyzer::new();
    let function_ids = resolve_function_targets(&config).await?;
    let summaries = run_provability_analysis(&analyzer, &function_ids).await?;
    let filtered_summaries = prepare_filtered_summaries(&summaries, config.high_confidence_only);
    let content = format_provability_output(&function_ids, &filtered_summaries, &config)?;

    write_provability_output(&content, &config.output).await?;

    Ok(())
}

/// Resolve function targets for analysis (cognitive complexity ≤6)
async fn resolve_function_targets(
    config: &ProvabilityConfig,
) -> Result<Vec<crate::services::lightweight_provability_analyzer::FunctionId>> {
    use crate::cli::provability_helpers::{discover_project_functions, match_function_spec};

    let discovered = discover_project_functions(&config.project_path).await?;

    if config.functions.is_empty() {
        return Ok(discovered);
    }

    // Every --functions spec used to be handed to parse_function_spec, which
    // built a FunctionId out of the argument string with no lookup at all — so
    // `--functions ghost` against an EMPTY directory still printed "✓ Analyzed
    // 1 functions" and scored it 20.0%. A name that is not in the tree has to
    // come back as an error, not as a result row.
    let mut ids = Vec::new();
    for spec in &config.functions {
        let matches = match_function_spec(spec, &discovered);
        if matches.is_empty() {
            anyhow::bail!(
                "no function named `{spec}` found under {}",
                config.project_path.display()
            );
        }
        ids.extend(matches);
    }
    Ok(ids)
}

/// Run provability analysis on function targets (cognitive complexity ≤3)
async fn run_provability_analysis(
    analyzer: &crate::services::lightweight_provability_analyzer::LightweightProvabilityAnalyzer,
    function_ids: &[crate::services::lightweight_provability_analyzer::FunctionId],
) -> Result<Vec<ProofSummary>> {
    let summaries = analyzer.analyze_incrementally(function_ids).await;
    use crate::cli::colors as c;
    eprintln!("{} Analyzed {} functions", c::pass(""), c::number(&summaries.len().to_string()));
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
    use crate::cli::provability_helpers::{
        format_provability_detailed, format_provability_json, format_provability_sarif,
        format_provability_summary,
    };

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
        ProvabilityOutputFormat::Sarif => format_provability_sarif(function_ids, summaries),
        ProvabilityOutputFormat::Markdown => {
            format_provability_detailed(function_ids, summaries, config.include_evidence)
        }
    }
}

/// Write provability output to file or stdout (cognitive complexity ≤4)
async fn write_provability_output(content: &str, output_path: &Option<PathBuf>) -> Result<()> {
    if let Some(output_path) = output_path {
        tokio::fs::write(output_path, content).await?;
        use crate::cli::colors as c;
        eprintln!(
            "{} Provability analysis written to: {}",
            c::pass(""), c::path(&output_path.display().to_string())
        );
    } else {
        println!("{content}");
    }
    Ok(())
}
