// Provability and defect prediction handlers - extracted for file health (CB-040)
/// Analyzes provability of code assertions
///
/// # Errors
/// Returns an error if the analysis fails
#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_provability(
    project_path: PathBuf,
    functions: Vec<String>,
    _analysis_depth: usize,
    format: ProvabilityOutputFormat,
    high_confidence_only: bool,
    include_evidence: bool,
    output: Option<PathBuf>,
    top_files: usize,
) -> Result<()> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    use crate::services::lightweight_provability_analyzer::LightweightProvabilityAnalyzer;

    eprintln!("🔬 Analyzing function provability...");

    // Create the analyzer
    let analyzer = LightweightProvabilityAnalyzer::new();

    // Get function IDs based on input
    let function_ids = get_function_ids(&project_path, &functions).await?;

    // Analyze the functions
    let summaries = analyzer.analyze_incrementally(&function_ids).await;
    eprintln!("✅ Analyzed {} functions", summaries.len());

    // Filter and format the summaries
    let filtered_summaries_owned = prepare_summaries(&summaries, high_confidence_only);

    // Format output based on requested format
    let content = format_provability_output(
        format,
        &function_ids,
        &filtered_summaries_owned,
        include_evidence,
        top_files,
    )?;

    // Write output
    write_provability_output(output, &content).await?;

    Ok(())
}

/// Get function IDs based on input parameters
async fn get_function_ids(
    project_path: &Path,
    functions: &[String],
) -> Result<Vec<crate::services::lightweight_provability_analyzer::FunctionId>> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    use crate::cli::provability_helpers::{discover_project_functions, parse_function_spec};

    if functions.is_empty() {
        discover_project_functions(project_path).await
    } else {
        let mut ids = Vec::new();
        for spec in functions {
            ids.push(parse_function_spec(spec, project_path)?);
        }
        Ok(ids)
    }
}

/// Prepare summaries by filtering and converting
fn prepare_summaries(summaries: &[ProofSummary], high_confidence_only: bool) -> Vec<ProofSummary> {
    use crate::cli::provability_helpers::filter_summaries;

    let filtered_summaries = filter_summaries(summaries, high_confidence_only);
    filtered_summaries.into_iter().cloned().collect()
}

/// Format provability output based on the specified format
fn format_provability_output(
    format: ProvabilityOutputFormat,
    function_ids: &[crate::services::lightweight_provability_analyzer::FunctionId],
    summaries: &[ProofSummary],
    include_evidence: bool,
    top_files: usize,
) -> Result<String> {
    use crate::cli::provability_helpers::{
        format_provability_detailed, format_provability_json, format_provability_sarif,
        format_provability_summary,
    };

    match format {
        ProvabilityOutputFormat::Json => {
            format_provability_json(function_ids, summaries, include_evidence)
        }
        ProvabilityOutputFormat::Summary => {
            format_provability_summary(function_ids, summaries, top_files)
        }
        ProvabilityOutputFormat::Full | ProvabilityOutputFormat::Markdown => {
            format_provability_detailed(function_ids, summaries, include_evidence)
        }
        ProvabilityOutputFormat::Sarif => format_provability_sarif(function_ids, summaries),
    }
}

/// Write provability output to file or stdout
async fn write_provability_output(output: Option<PathBuf>, content: &str) -> Result<()> {
    debug_assert!(!content.is_empty(), "content must not be empty");
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, content).await?;
        eprintln!(
            "✅ Provability analysis written to: {}",
            output_path.display()
        );
    } else {
        println!("{content}");
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_defect_prediction(
    project_path: PathBuf,
    confidence_threshold: f32,
    _min_lines: usize,
    include_low_confidence: bool,
    format: DefectPredictionOutputFormat,
    high_risk_only: bool,
    _include_recommendations: bool,
    _include: Option<String>,
    _exclude: Option<String>,
    output: Option<PathBuf>,
    _perf: bool,
    top_files: usize,
) -> Result<()> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    print_defect_analysis_header(
        &project_path,
        high_risk_only,
        include_low_confidence,
        &format,
    );

    let config = create_defect_config(
        confidence_threshold,
        _min_lines,
        include_low_confidence,
        high_risk_only,
        _include_recommendations,
        _include,
        _exclude,
    );

    let predictions =
        compute_defect_predictions(&project_path, &config, confidence_threshold).await?;
    let top_predictions = filter_and_sort_predictions(predictions, top_files);

    // Convert to report format expected by existing formatting functions
    let report = create_defect_report_from_predictions(top_predictions)?;

    // Format and output
    let content = format_defect_report(&report, format)?;
    output_defect_result(content, output).await?;

    Ok(())
}

fn print_defect_analysis_header(
    project_path: &Path,
    high_risk_only: bool,
    include_low_confidence: bool,
    format: &DefectPredictionOutputFormat,
) {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    eprintln!("🔮 Analyzing defect probability...");
    eprintln!("📁 Project path: {}", project_path.display());
    eprintln!("🎯 High risk only: {high_risk_only}");
    eprintln!("📊 Include low confidence: {include_low_confidence}");
    eprintln!("📄 Format: {format:?}");
}

fn create_defect_config(
    confidence_threshold: f32,
    min_lines: usize,
    include_low_confidence: bool,
    high_risk_only: bool,
    include_recommendations: bool,
    include: Option<String>,
    exclude: Option<String>,
) -> crate::cli::defect_prediction_helpers::DefectPredictionConfig {
    debug_assert!(min_lines > 0, "min_lines must be positive");
    crate::cli::defect_prediction_helpers::DefectPredictionConfig {
        confidence_threshold,
        min_lines,
        include_low_confidence,
        high_risk_only,
        include_recommendations,
        include,
        exclude,
    }
}

async fn compute_defect_predictions(
    project_path: &Path,
    config: &crate::cli::defect_prediction_helpers::DefectPredictionConfig,
    confidence_threshold: f32,
) -> Result<Vec<(String, crate::services::defect_probability::DefectScore)>> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    use crate::cli::defect_prediction_helpers::discover_source_files_for_defect_analysis;
    use crate::services::defect_probability::DefectProbabilityCalculator;

    let calculator = DefectProbabilityCalculator::new();
    let files = discover_source_files_for_defect_analysis(project_path, config).await?;

    let mut predictions = Vec::new();
    for (file_path, _content, lines) in files {
        let metrics = create_file_metrics(&file_path, lines);
        let score = calculator.calculate(&metrics);

        if should_include_prediction(
            &score,
            config.high_risk_only,
            config.include_low_confidence,
            confidence_threshold,
        ) {
            predictions.push((file_path.to_string_lossy().to_string(), score));
        }
    }

    Ok(predictions)
}

fn create_file_metrics(
    file_path: &Path,
    lines: usize,
) -> crate::services::defect_probability::FileMetrics {
    debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
    crate::services::defect_probability::FileMetrics {
        file_path: file_path.to_string_lossy().to_string(),
        churn_score: 0.5,                 // Would be calculated from git history
        complexity: (lines as f32) * 0.1, // Rough estimate
        duplicate_ratio: 0.1,             // Would be calculated from duplicate analysis
        afferent_coupling: 1.0,
        efferent_coupling: 1.0,
        lines_of_code: lines,
        cyclomatic_complexity: (lines / 20) as u32, // Rough estimate
        cognitive_complexity: (lines / 15) as u32,  // Rough estimate
    }
}

fn should_include_prediction(
    score: &crate::services::defect_probability::DefectScore,
    high_risk_only: bool,
    include_low_confidence: bool,
    confidence_threshold: f32,
) -> bool {
    use crate::services::defect_probability::RiskLevel;

    if high_risk_only && matches!(score.risk_level, RiskLevel::Low | RiskLevel::Medium) {
        return false;
    }

    if !include_low_confidence && score.probability < confidence_threshold {
        return false;
    }

    true
}

fn filter_and_sort_predictions(
    mut predictions: Vec<(String, crate::services::defect_probability::DefectScore)>,
    top_files: usize,
) -> Vec<(String, crate::services::defect_probability::DefectScore)> {
    predictions.sort_unstable_by(|a, b| {
        b.1.probability
            .partial_cmp(&a.1.probability)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    predictions.truncate(top_files);
    predictions
}

fn format_defect_report(
    report: &DefectPredictionReport,
    format: DefectPredictionOutputFormat,
) -> Result<String> {
    use DefectPredictionOutputFormat::{Csv, Detailed, Json, Sarif, Summary};
    match format {
        Summary => format_defect_summary(report, 10),
        Json => serde_json::to_string_pretty(report).map_err(Into::into),
        Detailed => format_defect_full(report, 10),
        Sarif => format_defect_sarif(report),
        Csv => format_defect_csv(report),
    }
}

async fn output_defect_result(content: String, output: Option<PathBuf>) -> Result<()> {
    eprintln!("✅ Defect prediction complete");

    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        eprintln!("📝 Written to {}", output_path.display());
    } else {
        println!("{content}");
    }
    Ok(())
}
