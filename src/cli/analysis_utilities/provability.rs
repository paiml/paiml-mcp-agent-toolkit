// Provability and defect prediction handlers - extracted for file health (CB-040)
/// Analyzes provability of code assertions
///
/// # Errors
/// Returns an error if the analysis fails
#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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
///
/// `--functions` used to go through `parse_function_spec`, which manufactured a
/// `FunctionId { file_path: String::new(), line_number: 0 }` out of a bare name
/// — its "will search all files" comment described a search nobody wrote. The
/// analyzer was then handed an empty path, read no source, and returned the
/// 20% no-evidence baseline for a function that scores 100% when the same tree
/// is analyzed unfiltered; a name that exists nowhere scored 20% just the same.
/// A filter must narrow the functions actually discovered, never invent one.
async fn get_function_ids(
    project_path: &Path,
    functions: &[String],
) -> Result<Vec<crate::services::lightweight_provability_analyzer::FunctionId>> {
    use crate::cli::provability_helpers::{discover_project_functions, match_function_spec};

    let discovered = discover_project_functions(project_path).await?;

    if functions.is_empty() {
        return Ok(discovered);
    }

    let mut ids = Vec::new();
    let mut unmatched = Vec::new();
    for spec in functions {
        let matches = match_function_spec(spec, &discovered);
        if matches.is_empty() {
            unmatched.push(spec.clone());
        }
        ids.extend(matches);
    }

    if !unmatched.is_empty() {
        anyhow::bail!(
            "no function matching {} was found under {}",
            unmatched.join(", "),
            project_path.display()
        );
    }

    Ok(ids)
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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

#[cfg(test)]
mod provability_tests {
    //! Covers the pure-compute helpers in provability.rs (158 uncov on broad, 0% cov).
    //! The async handlers + pmat-query-spawning paths are skipped.
    use super::*;
    use crate::services::defect_probability::{DefectScore, RiskLevel};

    fn score_with(probability: f32, risk: RiskLevel, confidence: f32) -> DefectScore {
        DefectScore {
            probability,
            contributing_factors: vec![("churn".to_string(), 0.5)],
            confidence,
            risk_level: risk,
            recommendations: vec!["refactor".to_string()],
        }
    }

    #[test]
    fn test_should_include_prediction_high_risk_only_rejects_low_and_medium() {
        let low = score_with(0.1, RiskLevel::Low, 0.9);
        assert!(!should_include_prediction(&low, true, true, 0.0));

        let medium = score_with(0.5, RiskLevel::Medium, 0.9);
        assert!(!should_include_prediction(&medium, true, true, 0.0));
    }

    #[test]
    fn test_should_include_prediction_high_risk_only_accepts_high() {
        let high = score_with(0.9, RiskLevel::High, 0.9);
        assert!(should_include_prediction(&high, true, true, 0.0));
    }

    #[test]
    fn test_should_include_prediction_low_confidence_filter_rejects_below_threshold() {
        let low_prob = score_with(0.2, RiskLevel::Low, 0.9);
        assert!(!should_include_prediction(&low_prob, false, false, 0.5));
    }

    #[test]
    fn test_should_include_prediction_low_confidence_filter_accepts_at_or_above_threshold() {
        let at_thresh = score_with(0.5, RiskLevel::Medium, 0.9);
        assert!(should_include_prediction(&at_thresh, false, false, 0.5));
    }

    #[test]
    fn test_should_include_prediction_include_low_confidence_bypasses_threshold() {
        let low_prob = score_with(0.1, RiskLevel::Low, 0.9);
        assert!(should_include_prediction(&low_prob, false, true, 0.99));
    }

    #[test]
    fn test_filter_and_sort_predictions_orders_by_probability_desc() {
        let preds = vec![
            ("a.rs".to_string(), score_with(0.2, RiskLevel::Low, 0.9)),
            ("b.rs".to_string(), score_with(0.8, RiskLevel::High, 0.9)),
            ("c.rs".to_string(), score_with(0.5, RiskLevel::Medium, 0.9)),
        ];
        let sorted = filter_and_sort_predictions(preds, 10);
        assert_eq!(sorted[0].0, "b.rs");
        assert_eq!(sorted[1].0, "c.rs");
        assert_eq!(sorted[2].0, "a.rs");
    }

    #[test]
    fn test_filter_and_sort_predictions_truncates_to_top_n() {
        let preds = vec![
            ("a.rs".to_string(), score_with(0.2, RiskLevel::Low, 0.9)),
            ("b.rs".to_string(), score_with(0.8, RiskLevel::High, 0.9)),
            ("c.rs".to_string(), score_with(0.5, RiskLevel::Medium, 0.9)),
        ];
        let sorted = filter_and_sort_predictions(preds, 2);
        assert_eq!(sorted.len(), 2);
        assert_eq!(sorted[0].0, "b.rs");
        assert_eq!(sorted[1].0, "c.rs");
    }

    #[test]
    fn test_filter_and_sort_predictions_empty_is_empty() {
        let sorted = filter_and_sort_predictions(vec![], 5);
        assert!(sorted.is_empty());
    }

    #[test]
    fn test_create_defect_config_copies_all_fields() {
        let cfg = create_defect_config(
            0.7,
            100,
            true,
            false,
            true,
            Some("src/*".into()),
            Some("tests/*".into()),
        );
        assert!((cfg.confidence_threshold - 0.7).abs() < 1e-6);
        assert_eq!(cfg.min_lines, 100);
        assert!(cfg.include_low_confidence);
        assert!(!cfg.high_risk_only);
        assert!(cfg.include_recommendations);
        assert_eq!(cfg.include.as_deref(), Some("src/*"));
        assert_eq!(cfg.exclude.as_deref(), Some("tests/*"));
    }

    #[test]
    fn test_create_file_metrics_derives_from_line_count() {
        let m = create_file_metrics(std::path::Path::new("src/a.rs"), 200);
        assert_eq!(m.lines_of_code, 200);
        assert_eq!(m.cyclomatic_complexity, 10);
        assert_eq!(m.cognitive_complexity, 13);
        assert_eq!(m.file_path, "src/a.rs");
        assert!((m.churn_score - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_create_file_metrics_small_file_floors_complexity() {
        let m = create_file_metrics(std::path::Path::new("t.rs"), 5);
        assert_eq!(m.cyclomatic_complexity, 0);
        assert_eq!(m.cognitive_complexity, 0);
    }

    /// `--functions <name>` must select from the functions actually discovered.
    /// It used to synthesise `FunctionId { file_path: "", line_number: 0 }`, so
    /// the analyzer read no source and returned the 20% no-evidence baseline
    /// for a function that scores 100% when the same tree is analyzed whole.
    #[tokio::test]
    async fn test_function_filter_keeps_the_real_source_location() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("lib.rs"),
            "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
        )
        .unwrap();

        let ids = get_function_ids(dir.path(), &["add".to_string()])
            .await
            .unwrap();

        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0].function_name, "add");
        assert!(
            !ids[0].file_path.is_empty(),
            "filtering must not erase the file path"
        );
        assert_eq!(ids[0].line_number, 1, "line must survive filtering");
    }

    /// A name that exists nowhere used to be scored anyway.
    #[tokio::test]
    async fn test_unknown_function_name_is_an_error_not_a_score() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("lib.rs"), "pub fn add() {}\n").unwrap();

        let err = get_function_ids(dir.path(), &["ghost".to_string()])
            .await
            .expect_err("a function that does not exist cannot be analyzed");
        assert!(err.to_string().contains("ghost"), "{err}");
    }

    #[test]
    fn test_print_defect_analysis_header_runs() {
        print_defect_analysis_header(
            std::path::Path::new("/tmp/x"),
            true,
            false,
            &DefectPredictionOutputFormat::Summary,
        );
    }
}
