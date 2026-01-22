// Defect report formatting - extracted for file health (CB-040)
fn create_defect_report_from_predictions(
    predictions: Vec<(String, crate::services::defect_probability::DefectScore)>,
) -> Result<DefectPredictionReport> {
    use crate::services::defect_probability::RiskLevel;
    let mut high_risk_files = 0;
    let mut medium_risk_files = 0;
    let mut low_risk_files = 0;

    let file_predictions: Vec<FilePrediction> = predictions
        .iter()
        .map(|(file_path, score)| {
            match score.risk_level {
                RiskLevel::High => high_risk_files += 1,
                RiskLevel::Medium => medium_risk_files += 1,
                RiskLevel::Low => low_risk_files += 1,
            }

            let factors: Vec<String> = score
                .contributing_factors
                .iter()
                .map(|(factor, contribution)| format!("{}: {:.1}%", factor, contribution * 100.0))
                .collect();

            FilePrediction {
                file_path: file_path.clone(),
                risk_score: score.probability,
                risk_level: format!("{:?}", score.risk_level),
                factors,
            }
        })
        .collect();

    Ok(DefectPredictionReport {
        total_files: predictions.len(),
        high_risk_files,
        medium_risk_files,
        low_risk_files,
        file_predictions,
    })
}

#[derive(Debug, Serialize)]
pub struct DefectPredictionReport {
    pub total_files: usize,
    pub high_risk_files: usize,
    pub medium_risk_files: usize,
    pub low_risk_files: usize,
    pub file_predictions: Vec<FilePrediction>,
}

#[derive(Debug, Serialize)]
pub struct FilePrediction {
    pub file_path: String,
    pub risk_score: f32,
    pub risk_level: String,
    pub factors: Vec<String>,
}

/// Format defect prediction summary with top files
///
/// # Example
///
/// ```no_run
/// use pmat::cli::analysis_utilities::{format_defect_summary, DefectPredictionReport, FilePrediction};
///
/// let report = DefectPredictionReport {
///     total_files: 100,
///     high_risk_files: 5,
///     medium_risk_files: 20,
///     low_risk_files: 75,
///     file_predictions: vec![
///         FilePrediction {
///             file_path: "src/main.rs".to_string(),
///             risk_score: 0.9,
///             risk_level: "high".to_string(),
///             factors: vec!["High complexity".to_string()],
///         },
///         FilePrediction {
///             file_path: "src/lib.rs".to_string(),
///             risk_score: 0.6,
///             risk_level: "medium".to_string(),
///             factors: vec!["Recent churn".to_string()],
///         },
///     ],
/// };
///
/// let output = format_defect_summary(&report, 5).unwrap();
///
/// assert!(output.contains("# Defect Prediction Analysis"));
/// assert!(output.contains("Total files analyzed: 100"));
/// assert!(output.contains("## Top Files by Defect Risk"));
/// assert!(output.contains("1. `main.rs` - 90.0% risk (high)"));
/// ```ignore
pub fn format_defect_summary(report: &DefectPredictionReport, top_files: usize) -> Result<String> {
    use std::fmt::Write;
    let mut output = String::new();

    writeln!(&mut output, "# Defect Prediction Analysis\n")?;
    format_defect_summary_stats(&mut output, report)?;

    if !report.file_predictions.is_empty() {
        format_defect_top_files(&mut output, report, top_files)?;
    }

    Ok(output)
}

/// Format the defect prediction summary statistics
fn format_defect_summary_stats(output: &mut String, report: &DefectPredictionReport) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "## Summary")?;
    writeln!(output, "- Total files analyzed: {}", report.total_files)?;
    writeln!(output, "- High risk files: {}", report.high_risk_files)?;
    writeln!(output, "- Medium risk files: {}", report.medium_risk_files)?;
    writeln!(output, "- Low risk files: {}\n", report.low_risk_files)?;

    Ok(())
}

/// Format the top files by defect risk section
fn format_defect_top_files(
    output: &mut String,
    report: &DefectPredictionReport,
    top_files: usize,
) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "## Top Files by Defect Risk\n")?;

    let files_to_show = if top_files == 0 { 10 } else { top_files };
    for (i, prediction) in report
        .file_predictions
        .iter()
        .take(files_to_show)
        .enumerate()
    {
        format_defect_prediction_entry(output, i + 1, prediction)?;
    }

    Ok(())
}

/// Format a single defect prediction entry
fn format_defect_prediction_entry(
    output: &mut String,
    index: usize,
    prediction: &FilePrediction,
) -> Result<()> {
    use std::fmt::Write;

    let filename = extract_filename_from_prediction(prediction);
    writeln!(
        output,
        "{}. `{}` - {:.1}% risk ({})",
        index,
        filename,
        prediction.risk_score * 100.0,
        prediction.risk_level
    )?;

    Ok(())
}

/// Extract display filename from prediction
fn extract_filename_from_prediction(prediction: &FilePrediction) -> &str {
    std::path::Path::new(&prediction.file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&prediction.file_path)
}

fn format_defect_full(report: &DefectPredictionReport, top_files: usize) -> Result<String> {
    crate::cli::defect_formatter::format_defect_report(report, "full", top_files)
}

fn format_defect_sarif(report: &DefectPredictionReport) -> Result<String> {
    crate::cli::defect_formatter::format_defect_report(report, "sarif", 0)
}

fn format_defect_csv(report: &DefectPredictionReport) -> Result<String> {
    crate::cli::defect_formatter::format_defect_report(report, "csv", 0)
}

// Single file quality gate check functions

