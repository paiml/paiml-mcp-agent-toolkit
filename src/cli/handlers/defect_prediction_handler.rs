//! Defect Prediction Analysis Handler
//!
//! Refactored handler using the service facade pattern to reduce complexity.

use crate::cli::DefectPredictionOutputFormat;
use crate::services::facades::defect_prediction_facade::{
    DefectPredictionFacade, DefectPredictionRequest, DefectPredictionResult,
};
use crate::services::service_registry::ServiceRegistry;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Configuration for defect prediction analysis
#[derive(Debug, Clone)]
pub struct DefectPredictionConfig {
    pub project_path: PathBuf,
    pub confidence_threshold: f32,
    pub min_lines: usize,
    pub include_low_confidence: bool,
    pub format: DefectPredictionOutputFormat,
    pub high_risk_only: bool,
    pub include_recommendations: bool,
    pub include: Option<String>,
    pub exclude: Option<String>,
    pub output: Option<PathBuf>,
    pub perf: bool,
    pub top_files: usize,
}

/// Refactored handler for defect prediction analysis using the facade pattern.
///
/// This reduces complexity from 23 to ~8 by delegating to the facade service.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn handle_analyze_defect_prediction(config: DefectPredictionConfig) -> Result<()> {
    // GH-666: a nonexistent path scanned zero files and this reported
    // "Analyzed 0 files: 0 high risk, 0 medium risk, 0 low risk /
    // ✓ Defect prediction analysis complete" with exit 0.
    crate::cli::ensure_analysis_path_exists(&config.project_path)?;

    // Print analysis header
    print_analysis_header(
        &config.project_path,
        config.high_risk_only,
        config.include_low_confidence,
    );

    // Create service registry and facade
    let registry = Arc::new(ServiceRegistry::new());
    let facade = DefectPredictionFacade::new(registry);

    // Build analysis request
    let request = DefectPredictionRequest {
        project_path: config.project_path.clone(),
        confidence_threshold: config.confidence_threshold,
        min_lines: config.min_lines,
        include_low_confidence: config.include_low_confidence,
        high_risk_only: config.high_risk_only,
        include_recommendations: config.include_recommendations,
        include: config.include.map(|s| vec![s]),
        exclude: config.exclude.map(|s| vec![s]),
        top_files: config.top_files,
    };

    // Perform analysis using facade
    let result = facade.analyze_project(request).await?;
    let files_analyzed = result.total_files_analyzed;
    let project_path = config.project_path.clone();

    // Format and output results
    output_results(result, config.format, config.output).await?;

    // Report first, then REFUSE. Over an empty directory this printed
    // "Analyzed 0 of 0 discovered files: 0 high risk, 0 medium risk, 0 low
    // risk" and exited 0 — a risk distribution over an empty population, which
    // is not a measurement, handed to CI as a pass. The eleven analyzers that
    // already call this helper refuse the same case; this one was written
    // before it existed.
    crate::cli::ensure_source_files_were_analyzed(
        "defect-prediction",
        &project_path,
        files_analyzed,
    )?;

    {
        use crate::cli::colors as c;
        crate::status_eprintln!("{}", c::pass("Defect prediction analysis complete"));
    }
    Ok(())
}

/// Print analysis header information
///
/// Every renderer in this file goes through `crate::cli::colors`' helpers, not
/// its raw `pub const` escape sequences. With the constants,
/// `analyze defect-prediction --color never` (and `NO_COLOR=1 TERM=dumb`, on a
/// pipe) emitted byte-identical output to `--color always`: 3 ESC lines in
/// `--format summary`, 35 in `--format detailed`, plus the stderr preamble —
/// while `analyze complexity` and `analyze dead-code` honoured the flag. The
/// helpers consult `colors_enabled()`; the constants cannot.
fn print_analysis_header(project_path: &Path, high_risk_only: bool, include_low_confidence: bool) {
    use crate::cli::colors as c;
    crate::status_eprintln!("{}", c::dim("Analyzing defect probability..."));
    crate::status_eprintln!(
        "  {} {}",
        c::label("Project path:"),
        c::path(&project_path.display().to_string())
    );
    crate::status_eprintln!(
        "  {} {}",
        c::label("High risk only:"),
        c::number(&high_risk_only.to_string())
    );
    crate::status_eprintln!(
        "  {} {}",
        c::label("Include low confidence:"),
        c::number(&include_low_confidence.to_string())
    );
}

/// Output results in the requested format
async fn output_results(
    result: DefectPredictionResult,
    format: DefectPredictionOutputFormat,
    output: Option<PathBuf>,
) -> Result<()> {
    let content = format_result(result, format)?;

    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        crate::status_eprintln!("📝 Written to {}", output_path.display());
    } else {
        println!("{content}");
    }

    Ok(())
}

/// Format the analysis result based on the requested format
fn format_result(
    result: DefectPredictionResult,
    format: DefectPredictionOutputFormat,
) -> Result<String> {
    match format {
        DefectPredictionOutputFormat::Summary => Ok(format_summary(&result)),
        DefectPredictionOutputFormat::Detailed => Ok(format_detailed(&result)),
        DefectPredictionOutputFormat::Json => {
            serde_json::to_string_pretty(&result).map_err(Into::into)
        }
        DefectPredictionOutputFormat::Csv => Ok(format_csv(&result)),
        DefectPredictionOutputFormat::Sarif => Ok(format_sarif(&result)),
    }
}

/// Format as summary
fn format_summary(result: &DefectPredictionResult) -> String {
    use crate::cli::colors as c;
    use std::fmt::Write;

    let mut output = String::new();
    let _ = writeln!(output, "{}\n", c::header("Defect Prediction Summary"));
    let _ = writeln!(output, "  {}", result.summary);
    let _ = writeln!(output, "\n{}", c::subheader("Top Risk Files"));

    for (i, prediction) in result.predictions.iter().take(10).enumerate() {
        let risk_color = match prediction.risk_level {
            crate::services::facades::defect_prediction_facade::RiskLevel::Critical => c::RED,
            crate::services::facades::defect_prediction_facade::RiskLevel::High => c::RED,
            crate::services::facades::defect_prediction_facade::RiskLevel::Medium => c::YELLOW,
            crate::services::facades::defect_prediction_facade::RiskLevel::Low => c::GREEN,
        };
        let _ = writeln!(
            output,
            "  {}. {} - {} ({})",
            c::number(&(i + 1).to_string()),
            c::path(&prediction.file_path),
            c::colored(
                risk_color,
                &format!("{:.1}% risk", prediction.defect_probability * 100.0)
            ),
            c::colored(risk_color, &format!("{:?}", prediction.risk_level)),
        );
    }

    if !result.recommendations.is_empty() {
        let _ = writeln!(output, "\n{}", c::subheader("Recommendations"));
        for rec in &result.recommendations {
            let _ = writeln!(output, "  - {rec}");
        }
    }

    output
}

/// Format as detailed report
fn format_detailed(result: &DefectPredictionResult) -> String {
    use crate::cli::colors as c;
    use std::fmt::Write;

    let mut output = String::new();
    let _ = writeln!(
        output,
        "{}\n",
        c::header("Defect Prediction Detailed Report")
    );
    // #657: report the real counts. `Total files analyzed` used to be the
    // post-truncation prediction count, so it read "10" for a 3863-file repo.
    let _ = writeln!(
        output,
        "  {} {}",
        c::label("Files discovered:"),
        c::number(&result.total_files_discovered.to_string())
    );
    let _ = writeln!(
        output,
        "  {} {}",
        c::label("Files analyzed:"),
        c::number(&result.total_files_analyzed.to_string())
    );
    let _ = writeln!(
        output,
        "  {} {}",
        c::label("Predictions shown:"),
        c::number(&format!(
            "{} of {}{}",
            result.predictions_reported,
            result.files_matching_filters,
            if result.predictions_truncated {
                " (truncated by --top-files)"
            } else {
                ""
            }
        ))
    );
    // Every factor states where it came from, or why it is absent (#657).
    for (label, description) in [
        ("Churn source:", result.churn_source.describe()),
        ("Coupling source:", result.coupling_source.describe()),
        ("Duplication source:", result.duplication_source.describe()),
    ] {
        let _ = writeln!(
            output,
            "  {}{label}{} {}{description}{}",
            c::BOLD,
            c::RESET,
            c::BOLD_WHITE,
            c::RESET
        );
    }
    let _ = writeln!(
        output,
        "  {} {}",
        c::label("High risk files:"),
        c::colored(c::RED, &result.high_risk_files.to_string())
    );
    let _ = writeln!(
        output,
        "  {} {}",
        c::label("Medium risk files:"),
        c::colored(c::YELLOW, &result.medium_risk_files.to_string())
    );
    let _ = writeln!(
        output,
        "  {} {}\n",
        c::label("Low risk files:"),
        c::colored(c::GREEN, &result.low_risk_files.to_string())
    );

    let _ = writeln!(output, "{}", c::subheader("File Analysis"));
    for prediction in &result.predictions {
        let risk_color = match prediction.risk_level {
            crate::services::facades::defect_prediction_facade::RiskLevel::Critical => c::RED,
            crate::services::facades::defect_prediction_facade::RiskLevel::High => c::RED,
            crate::services::facades::defect_prediction_facade::RiskLevel::Medium => c::YELLOW,
            crate::services::facades::defect_prediction_facade::RiskLevel::Low => c::GREEN,
        };
        let _ = writeln!(output, "\n  {}", c::path(&prediction.file_path));
        let _ = writeln!(
            output,
            "    {} {}",
            c::label("Risk Level:"),
            c::colored(risk_color, &format!("{:?}", prediction.risk_level))
        );
        let _ = writeln!(
            output,
            "    {} {}%",
            c::label("Defect Probability:"),
            c::number(&format!("{:.1}", prediction.defect_probability * 100.0))
        );
        let _ = writeln!(
            output,
            "    {} {}%",
            c::label("Confidence:"),
            c::number(&format!("{:.1}", prediction.confidence * 100.0))
        );

        let _ = writeln!(output, "    {}", c::label("Risk Metrics:"));
        // #723: complexity was `lines / 100`, a length proxy. It is now a
        // measured cyclomatic figure and renders as "not measured" when the
        // language has no analyzer, never as a number nobody computed.
        let _ = writeln!(
            output,
            "      {} {}",
            c::label("Complexity:"),
            c::number(&format_optional_score(prediction.metrics.complexity_score))
        );
        // #657: an unmeasured churn renders as "not measured", never as a
        // number — 0.0 would read as "this file never changed".
        let _ = writeln!(
            output,
            "      {} {}",
            c::label("Churn:"),
            c::number(&format_optional_score(prediction.metrics.churn_score))
        );
        // Same rule for coupling and duplication (#657): both were constants
        // (0.2 and 0.1) presented with the authority of a measurement.
        let _ = writeln!(
            output,
            "      {}Coupling:{} {}{}{}",
            c::BOLD,
            c::RESET,
            c::BOLD_WHITE,
            format_optional_score(prediction.metrics.coupling_score),
            c::RESET
        );
        let _ = writeln!(
            output,
            "      {} {}",
            c::label("Size:"),
            c::number(&format!("{:.1}", prediction.metrics.size_score))
        );
        let _ = writeln!(
            output,
            "      {}Duplication:{} {}{}{}",
            c::BOLD,
            c::RESET,
            c::BOLD_WHITE,
            format_optional_score(prediction.metrics.duplication_score),
            c::RESET
        );

        if !prediction.contributing_factors.is_empty() {
            let _ = writeln!(output, "    {}", c::label("Contributing Factors:"));
            for factor in &prediction.contributing_factors {
                let _ = writeln!(output, "      - {factor}");
            }
        }
    }

    output
}

/// Render a possibly-unmeasured 0-1 score.
///
/// #657: `churn_score` used to be the constant 0.3 for every file. When it
/// cannot be measured it must render as "not measured" — never as a plausible
/// default and never as 0, which looks like a finding.
fn format_optional_score(score: Option<f32>) -> String {
    score.map_or_else(|| "not measured".to_string(), |v| format!("{v:.3}"))
}

/// Format as CSV
fn format_csv(result: &DefectPredictionResult) -> String {
    let mut output = String::new();
    output.push_str("File,Risk Level,Defect Probability,Confidence,Complexity,Churn,Coupling,Size,Duplication\n");

    for prediction in &result.predictions {
        output.push_str(&format!(
            "{},{:?},{:.3},{:.3},{},{},{},{:.3},{}\n",
            prediction.file_path,
            prediction.risk_level,
            prediction.defect_probability,
            prediction.confidence,
            // #723: measured cyclomatic, or "not measured" — same rule as the
            // other optional scores in this row.
            format_optional_score(prediction.metrics.complexity_score),
            format_optional_score(prediction.metrics.churn_score),
            format_optional_score(prediction.metrics.coupling_score),
            prediction.metrics.size_score,
            format_optional_score(prediction.metrics.duplication_score)
        ));
    }

    output
}

/// Format as SARIF
fn format_sarif(result: &DefectPredictionResult) -> String {
    let rules = vec![serde_json::json!({
        "id": "high-defect-risk",
        "shortDescription": {
            "text": "High defect probability detected"
        },
        "fullDescription": {
            "text": "Files with high defect probability require additional testing and review"
        }
    })];

    let results: Vec<_> = result
        .predictions
        .iter()
        .filter(|p| {
            matches!(
                p.risk_level,
                crate::services::facades::defect_prediction_facade::RiskLevel::High
                    | crate::services::facades::defect_prediction_facade::RiskLevel::Critical
            )
        })
        .map(|prediction| {
            serde_json::json!({
                "ruleId": "high-defect-risk",
                "level": if matches!(prediction.risk_level,
                    crate::services::facades::defect_prediction_facade::RiskLevel::Critical) {
                    "error"
                } else {
                    "warning"
                },
                "message": {
                    "text": format!(
                        "File has {:.1}% defect probability. Contributing factors: {}",
                        prediction.defect_probability * 100.0,
                        prediction.contributing_factors.join(", ")
                    )
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": prediction.file_path.clone()
                        }
                    }
                }],
                "properties": {
                    "defectProbability": prediction.defect_probability,
                    "confidence": prediction.confidence,
                    "riskLevel": format!("{:?}", prediction.risk_level)
                }
            })
        })
        .collect();

    serde_json::json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat-defect-prediction",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                    "rules": rules
                }
            },
            "results": results
        }]
    })
    .to_string()
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::facades::defect_prediction_facade::{
        ChurnSource, CouplingSource, DuplicationSource, FilePrediction, FileRiskMetrics, RiskLevel,
    };

    fn sample_result(
        churn_score: Option<f32>,
        churn_source: ChurnSource,
    ) -> DefectPredictionResult {
        DefectPredictionResult {
            total_files_discovered: 12,
            total_files_analyzed: 10,
            files_matching_filters: 10,
            high_risk_files: 3,
            medium_risk_files: 4,
            low_risk_files: 3,
            predictions_reported: 1,
            predictions_truncated: true,
            churn_source,
            coupling_source: CouplingSource::EfferentImports {
                imports_at_full_scale: 20,
            },
            duplication_source: DuplicationSource::not_run(),
            predictions: vec![FilePrediction {
                file_path: "test.rs".to_string(),
                defect_probability: 0.8,
                risk_level: RiskLevel::High,
                confidence: 0.9,
                metrics: FileRiskMetrics {
                    complexity_score: Some(0.8),
                    max_cyclomatic: Some(24),
                    churn_score,
                    coupling_score: Some(0.6),
                    size_score: 0.5,
                    duplication_score: None,
                    lines: 800,
                    imports: Some(12),
                    commits_in_window: churn_score.map(|_| 3),
                    changed_lines_in_window: churn_score.map(|_| 40),
                },
                contributing_factors: vec!["Long file (800 lines)".to_string()],
            }],
            summary: "Test summary".to_string(),
            recommendations: vec!["Test recommendation".to_string()],
        }
    }

    #[test]
    fn test_format_summary() {
        let result = sample_result(
            Some(0.7),
            ChurnSource::GitHistory {
                window_days: 90,
                files_with_churn: 4,
                commits_at_full_scale: 20,
                changed_lines_at_full_scale: 1000,
            },
        );

        let output = format_summary(&result);
        assert!(output.contains("Test summary"));
        assert!(output.contains("test.rs"));
        assert!(output.contains("80.0%"));
        assert!(output.contains("Test recommendation"));
    }

    /// #657: an unmeasured churn must never render as a number.
    #[test]
    fn test_format_optional_score_absent_says_not_measured() {
        assert_eq!(format_optional_score(None), "not measured");
        assert_eq!(format_optional_score(Some(0.0)), "0.000");
    }

    #[test]
    fn test_format_detailed_renders_absent_churn_as_not_measured() {
        let result = sample_result(
            None,
            ChurnSource::NotMeasured {
                reason: "No git repository found".to_string(),
            },
        );
        let output = format_detailed(&result);
        assert!(
            output.contains("not measured"),
            "detailed output must not invent a churn number: {output}"
        );
        // The cap must be visible as a cap, not as the total.
        assert!(output.contains("Files discovered:"));
        assert!(output.contains("truncated by --top-files"));
    }

    #[test]
    fn test_format_csv_renders_absent_churn_as_not_measured() {
        let result = sample_result(
            None,
            ChurnSource::NotMeasured {
                reason: "No git repository found".to_string(),
            },
        );
        let csv = format_csv(&result);
        assert!(csv.contains("not measured"), "csv churn column: {csv}");
    }

    /// Round 3: the disclosure round 2 added for churn must exist for the two
    /// other fields #657 named. Pre-fix, `coupling_score` (0.2) and
    /// `duplication_score` (0.1) were rendered as plain numbers with no
    /// provenance in any renderer.
    #[test]
    fn test_detailed_and_csv_disclose_coupling_and_duplication_sources() {
        let result = sample_result(
            Some(0.4),
            ChurnSource::GitHistory {
                window_days: 90,
                files_with_churn: 4,
                commits_at_full_scale: 20,
                changed_lines_at_full_scale: 1000,
            },
        );

        let detailed = format_detailed(&result);
        assert!(
            detailed.contains("Coupling source:"),
            "coupling must state where it came from: {detailed}"
        );
        assert!(
            detailed.contains("Duplication source:"),
            "duplication must state why it is absent: {detailed}"
        );
        // duplication_score is None in the sample → never a number. (The
        // renderer interleaves colour codes, so match the two halves.)
        let duplication_line = detailed
            .lines()
            .find(|l| l.contains("Duplication:") && !l.contains("Duplication source:"))
            .expect("a Duplication metric row");
        assert!(
            duplication_line.contains("not measured"),
            "duplication must not render as a number: {duplication_line}"
        );

        let csv = format_csv(&result);
        let data_line = csv.lines().nth(1).expect("one data row");
        assert!(
            data_line.ends_with("not measured"),
            "csv duplication column must be 'not measured', got: {data_line}"
        );
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod empty_population_tests {
    use super::*;

    fn predict_over(path: &std::path::Path) -> DefectPredictionConfig {
        DefectPredictionConfig {
            project_path: path.to_path_buf(),
            confidence_threshold: 0.5,
            min_lines: 1,
            include_low_confidence: true,
            format: DefectPredictionOutputFormat::Summary,
            high_risk_only: false,
            include_recommendations: false,
            include: None,
            exclude: None,
            output: None,
            perf: false,
            top_files: 10,
        }
    }

    /// A risk distribution over an empty population is not a measurement.
    ///
    /// Over a directory with nothing it can read, this printed "Analyzed 0 of 0
    /// discovered files: 0 high risk, 0 medium risk, 0 low risk" and exited 0 —
    /// handed to CI as a pass. Eleven analyzers already refuse this case
    /// through `ensure_source_files_were_analyzed`; this one predates it.
    #[tokio::test]
    async fn a_prediction_over_zero_files_does_not_exit_zero() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        std::fs::write(dir.path().join("legacy.cbl"), "IDENTIFICATION DIVISION.\n")
            .expect("write cobol");

        let err = handle_analyze_defect_prediction(predict_over(dir.path()))
            .await
            .expect_err("zero files analysed must not report success");
        assert!(
            err.to_string().to_lowercase().contains("defect-prediction"),
            "the refusal must name the measurement: {err}"
        );
    }

    /// The counter-test: a tree it CAN read still succeeds, so refusing
    /// unconditionally cannot pass the test above.
    #[tokio::test]
    async fn a_prediction_over_real_files_still_succeeds() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        std::fs::create_dir_all(dir.path().join("src")).expect("mkdir");
        std::fs::write(
            dir.path().join("src/lib.rs"),
            "//! Tiny.\n\n/// Adds.\npub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
        )
        .expect("write rs");

        handle_analyze_defect_prediction(predict_over(dir.path()))
            .await
            .expect("a readable tree must still succeed");
    }
}
