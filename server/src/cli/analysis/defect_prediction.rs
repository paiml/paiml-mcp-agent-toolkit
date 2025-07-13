//! Defect prediction analysis implementation using real ML-based service

use crate::cli::defect_helpers::discover_files_for_defect_analysis;
use crate::cli::defect_prediction_helpers::{collect_file_metrics, DefectPredictionConfig};
use crate::cli::DefectPredictionOutputFormat;
use crate::services::defect_probability::{DefectProbabilityCalculator, DefectScore};
use anyhow::Result;
use std::path::PathBuf;
use std::time::Instant;

/// Handle defect prediction analysis with real ML-based implementation
#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_defect_prediction(
    project_path: PathBuf,
    confidence_threshold: f32,
    min_lines: usize,
    include_low_confidence: bool,
    format: DefectPredictionOutputFormat,
    high_risk_only: bool,
    include_recommendations: bool,
    include: Option<String>,
    exclude: Option<String>,
    output: Option<PathBuf>,
    perf: bool,
    top_files: usize,
) -> Result<()> {
    let start_time = Instant::now();

    eprintln!("🔮 Analyzing defect probability using ML-based analysis...");
    eprintln!("📁 Project path: {}", project_path.display());
    eprintln!("🎯 Confidence threshold: {}", confidence_threshold);
    eprintln!("📊 High risk only: {}", high_risk_only);

    // Create configuration
    let config = DefectPredictionConfig {
        confidence_threshold,
        min_lines,
        include_low_confidence,
        high_risk_only,
        include_recommendations,
        include,
        exclude,
    };

    // Discover files
    let files = discover_files_for_defect_analysis(&project_path, &config).await?;
    eprintln!("📂 Found {} files matching criteria", files.len());

    if files.is_empty() {
        eprintln!("⚠️  No files found matching the criteria");
        return Ok(());
    }

    // Collect metrics for all files
    let file_metrics = collect_file_metrics(&files);

    // Calculate defect probabilities using real ML service
    let calculator = DefectProbabilityCalculator::new();
    let mut predictions: Vec<(String, DefectScore)> = file_metrics
        .into_iter()
        .map(|metrics| {
            let score = calculator.calculate(&metrics);
            (metrics.file_path, score)
        })
        .collect();

    // Apply filters
    if high_risk_only {
        predictions.retain(|(_, score)| score.probability > 0.7);
    }

    if !include_low_confidence {
        predictions.retain(|(_, score)| score.confidence > confidence_threshold);
    }

    // Sort by probability descending
    predictions.sort_by(|a, b| b.1.probability.partial_cmp(&a.1.probability).unwrap());

    // Limit to top files if specified
    if top_files > 0 && predictions.len() > top_files {
        predictions.truncate(top_files);
    }

    let elapsed = start_time.elapsed();

    // Format output
    let content = match format {
        DefectPredictionOutputFormat::Summary => format_defect_summary(&predictions, elapsed)?,
        DefectPredictionOutputFormat::Json => format_defect_json(&predictions, elapsed)?,
        DefectPredictionOutputFormat::Detailed => {
            format_defect_detailed(&predictions, elapsed, include_recommendations)?
        }
        DefectPredictionOutputFormat::Sarif => format_defect_sarif(&predictions)?,
        DefectPredictionOutputFormat::Csv => format_defect_csv(&predictions)?,
    };

    if perf {
        eprintln!("⏱️  Analysis completed in {:.2?}", elapsed);
    }

    eprintln!("✅ Defect prediction complete");

    // Output results
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        eprintln!("📝 Written to {}", output_path.display());
    } else {
        println!("{}", content);
    }

    Ok(())
}

/// Format predictions as summary
fn format_defect_summary(
    predictions: &[(String, DefectScore)],
    elapsed: std::time::Duration,
) -> Result<String> {
    use std::fmt::Write;
    let mut output = String::new();

    writeln!(&mut output, "🔮 Defect Prediction Summary")?;
    writeln!(&mut output, "==========================")?;
    writeln!(&mut output)?;

    // Statistics
    let high_risk = predictions
        .iter()
        .filter(|(_, s)| s.probability > 0.7)
        .count();
    let medium_risk = predictions
        .iter()
        .filter(|(_, s)| s.probability > 0.3 && s.probability <= 0.7)
        .count();
    let low_risk = predictions
        .iter()
        .filter(|(_, s)| s.probability <= 0.3)
        .count();

    writeln!(&mut output, "📊 Risk Distribution:")?;
    writeln!(&mut output, "  🔴 High risk:   {} files", high_risk)?;
    writeln!(&mut output, "  🟡 Medium risk: {} files", medium_risk)?;
    writeln!(&mut output, "  🟢 Low risk:    {} files", low_risk)?;
    writeln!(&mut output)?;

    if !predictions.is_empty() {
        writeln!(&mut output, "🎯 Top Risk Files:")?;
        for (file, score) in predictions.iter().take(10) {
            writeln!(
                &mut output,
                "  {} {:.1}% - {} (confidence: {:.1}%)",
                match score.risk_level {
                    crate::services::defect_probability::RiskLevel::High => "🔴",
                    crate::services::defect_probability::RiskLevel::Medium => "🟡",
                    crate::services::defect_probability::RiskLevel::Low => "🟢",
                },
                score.probability * 100.0,
                file,
                score.confidence * 100.0
            )?;
        }
    }

    writeln!(&mut output)?;
    writeln!(&mut output, "⏱️  Analysis time: {:.2?}", elapsed)?;

    Ok(output)
}

/// Format predictions as JSON
fn format_defect_json(
    predictions: &[(String, DefectScore)],
    elapsed: std::time::Duration,
) -> Result<String> {
    let report = serde_json::json!({
        "analysis_type": "defect_prediction",
        "summary": {
            "total_files_analyzed": predictions.len(),
            "high_risk_files": predictions.iter().filter(|(_, s)| s.probability > 0.7).count(),
            "medium_risk_files": predictions.iter().filter(|(_, s)| s.probability > 0.3 && s.probability <= 0.7).count(),
            "low_risk_files": predictions.iter().filter(|(_, s)| s.probability <= 0.3).count(),
            "analysis_time_ms": elapsed.as_millis(),
        },
        "predictions": predictions.iter().map(|(file, score)| {
            serde_json::json!({
                "file": file,
                "probability": score.probability,
                "confidence": score.confidence,
                "risk_level": format!("{:?}", score.risk_level),
                "contributing_factors": score.contributing_factors,
                "recommendations": score.recommendations,
            })
        }).collect::<Vec<_>>(),
    });

    Ok(serde_json::to_string_pretty(&report)?)
}

/// Format predictions as detailed report
fn format_defect_detailed(
    predictions: &[(String, DefectScore)],
    elapsed: std::time::Duration,
    include_recommendations: bool,
) -> Result<String> {
    use std::fmt::Write;
    let mut output = String::new();

    writeln!(&mut output, "🔮 Defect Prediction Detailed Report")?;
    writeln!(&mut output, "===================================")?;
    writeln!(&mut output)?;

    for (file, score) in predictions {
        writeln!(&mut output, "📄 File: {}", file)?;
        writeln!(
            &mut output,
            "   Risk Level: {} ({:.1}%)",
            match score.risk_level {
                crate::services::defect_probability::RiskLevel::High => "🔴 HIGH",
                crate::services::defect_probability::RiskLevel::Medium => "🟡 MEDIUM",
                crate::services::defect_probability::RiskLevel::Low => "🟢 LOW",
            },
            score.probability * 100.0
        )?;
        writeln!(&mut output, "   Confidence: {:.1}%", score.confidence * 100.0)?;

        if !score.contributing_factors.is_empty() {
            writeln!(&mut output, "   Contributing Factors:")?;
            for (factor, weight) in &score.contributing_factors {
                writeln!(&mut output, "     - {}: {:.1}%", factor, weight * 100.0)?;
            }
        }

        if include_recommendations && !score.recommendations.is_empty() {
            writeln!(&mut output, "   Recommendations:")?;
            for rec in &score.recommendations {
                writeln!(&mut output, "     • {}", rec)?;
            }
        }
        writeln!(&mut output)?;
    }

    writeln!(&mut output, "⏱️  Analysis time: {:.2?}", elapsed)?;

    Ok(output)
}

/// Format predictions as SARIF
fn format_defect_sarif(predictions: &[(String, DefectScore)]) -> Result<String> {
    let sarif = serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat-defect-prediction",
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                    "version": env!("CARGO_PKG_VERSION"),
                    "rules": [{
                        "id": "DEFECT-RISK",
                        "name": "DefectRisk",
                        "shortDescription": {
                            "text": "ML-based defect probability prediction"
                        },
                        "fullDescription": {
                            "text": "Predicts defect probability using ensemble ML model based on churn, complexity, duplication, and coupling metrics"
                        },
                        "help": {
                            "text": "Files with high defect probability should be reviewed carefully and refactored if necessary"
                        }
                    }]
                }
            },
            "results": predictions.iter().map(|(file, score)| {
                serde_json::json!({
                    "ruleId": "DEFECT-RISK",
                    "level": match score.risk_level {
                        crate::services::defect_probability::RiskLevel::High => "error",
                        crate::services::defect_probability::RiskLevel::Medium => "warning",
                        crate::services::defect_probability::RiskLevel::Low => "note",
                    },
                    "message": {
                        "text": format!("Defect probability: {:.1}% (confidence: {:.1}%)", 
                            score.probability * 100.0, score.confidence * 100.0)
                    },
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": {
                                "uri": file,
                                "uriBaseId": "%SRCROOT%"
                            }
                        }
                    }],
                    "properties": {
                        "probability": score.probability,
                        "confidence": score.confidence,
                        "contributing_factors": score.contributing_factors,
                        "recommendations": score.recommendations
                    }
                })
            }).collect::<Vec<_>>()
        }]
    });

    Ok(serde_json::to_string_pretty(&sarif)?)
}

/// Format predictions as CSV
fn format_defect_csv(predictions: &[(String, DefectScore)]) -> Result<String> {
    let mut csv = String::new();

    // Header
    csv.push_str("file,probability,confidence,risk_level,top_factor,top_factor_weight\n");

    // Data rows
    for (file, score) in predictions {
        let (top_factor, top_weight) = score
            .contributing_factors
            .first()
            .map(|(f, w)| (f.as_str(), *w))
            .unwrap_or(("", 0.0));

        csv.push_str(&format!(
            "{},{:.3},{:.3},{:?},{},{:.3}\n",
            file, score.probability, score.confidence, score.risk_level, top_factor, top_weight
        ));
    }

    Ok(csv)
}