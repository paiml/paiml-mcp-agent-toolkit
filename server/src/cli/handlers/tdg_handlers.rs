use crate::cli::commands::TdgCommand;
use crate::cli::TdgOutputFormat;
use crate::tdg::{Grade, TdgAnalyzer, TdgConfig};
use anyhow::{anyhow, Result};
use std::fs;
use std::path::PathBuf;

/// Handle TDG command execution
pub async fn handle_tdg_command(
    path: PathBuf,
    command: Option<TdgCommand>,
    format: TdgOutputFormat,
    config: Option<PathBuf>,
    quiet: bool,
    include_components: bool,
    min_grade: Option<String>,
    output: Option<PathBuf>,
) -> Result<()> {
    // Load configuration if provided
    let tdg_config = if let Some(config_path) = config {
        let config_content = fs::read_to_string(&config_path)?;
        toml::from_str(&config_content)?
    } else {
        TdgConfig::default()
    };

    // Create analyzer with configuration and storage
    let analyzer = TdgAnalyzer::with_storage(tdg_config)?;

    // Handle subcommands
    if let Some(cmd) = command {
        match cmd {
            TdgCommand::Compare { source1, source2 } => {
                let comparison = analyzer.compare(&source1, &source2).await?;
                let output_str = format_comparison(comparison, format)?;

                if let Some(output_path) = output {
                    fs::write(output_path, output_str)?;
                } else {
                    println!("{}", output_str);
                }
                return Ok(());
            }
            TdgCommand::Diagnostics { .. }
            | TdgCommand::Storage { .. }
            | TdgCommand::Dashboard { .. } => {
                // Handle diagnostic and dashboard commands
                return super::tdg_diagnostic_handler::handle_tdg_diagnostics(&cmd, &path).await;
            }
        }
    }

    // Analyze single file or directory
    let score = if path.is_dir() {
        analyzer.analyze_project(&path).await?.average()
    } else {
        analyzer.analyze_file(&path).await?
    };

    // Check minimum grade if specified (for CI/CD)
    if let Some(min_grade_str) = min_grade {
        let min_grade = parse_grade(&min_grade_str)?;
        if score.grade < min_grade {
            return Err(anyhow!(
                "Grade {} is below minimum required grade {}",
                format_grade(score.grade),
                format_grade(min_grade)
            ));
        }
    }

    // Format output
    let output_str = if quiet {
        format!("{:.1}", score.total)
    } else {
        format_tdg_score(score, format, include_components)?
    };

    // Write output
    if let Some(output_path) = output {
        fs::write(output_path, output_str)?;
    } else {
        println!("{}", output_str);
    }

    Ok(())
}

fn format_tdg_score(
    score: crate::tdg::TdgScore,
    format: TdgOutputFormat,
    include_components: bool,
) -> Result<String> {
    match format {
        TdgOutputFormat::Table => {
            let mut output = String::new();

            // Header
            output.push_str("╭─────────────────────────────────────────────────╮\n");
            if let Some(file_path) = &score.file_path {
                output.push_str(&format!(
                    "│  TDG Score Report: {}              │\n",
                    file_path.display()
                ));
            } else {
                output.push_str("│  TDG Score Report                              │\n");
            }
            output.push_str("├─────────────────────────────────────────────────┤\n");

            // Overall score
            output.push_str(&format!(
                "│  Overall Score: {:.1}/100 ({})                  │\n",
                score.total,
                format_grade(score.grade)
            ));
            output.push_str(&format!(
                "│  Language: {:?} (confidence: {:.0}%)             │\n",
                score.language,
                score.confidence * 100.0
            ));

            if include_components {
                output.push_str("│                                                 │\n");
                output.push_str("│  📊 Breakdown:                                  │\n");
                output.push_str(&format!(
                    "│  ├─ Structural:     {:.1}/25                    │\n",
                    score.structural_complexity
                ));
                output.push_str(&format!(
                    "│  ├─ Semantic:       {:.1}/20                    │\n",
                    score.semantic_complexity
                ));
                output.push_str(&format!(
                    "│  ├─ Duplication:    {:.1}/20                    │\n",
                    score.duplication_ratio
                ));
                output.push_str(&format!(
                    "│  ├─ Coupling:       {:.1}/15                    │\n",
                    score.coupling_score
                ));
                output.push_str(&format!(
                    "│  ├─ Documentation:  {:.1}/10                    │\n",
                    score.doc_coverage
                ));
                output.push_str(&format!(
                    "│  └─ Consistency:    {:.1}/10                    │\n",
                    score.consistency_score
                ));
            }

            output.push_str("╰─────────────────────────────────────────────────╯\n");
            Ok(output)
        }
        TdgOutputFormat::Json => {
            let json_value = serde_json::json!({
                "file": score.file_path.map(|p| p.to_string_lossy().to_string()),
                "language": format!("{:?}", score.language),
                "confidence": score.confidence,
                "score": {
                    "total": score.total,
                    "grade": format_grade(score.grade),
                    "breakdown": if include_components {
                        Some(serde_json::json!({
                            "structural_complexity": score.structural_complexity,
                            "semantic_complexity": score.semantic_complexity,
                            "duplication": score.duplication_ratio,
                            "coupling": score.coupling_score,
                            "documentation": score.doc_coverage,
                            "consistency": score.consistency_score,
                        }))
                    } else {
                        None
                    }
                }
            });
            Ok(serde_json::to_string_pretty(&json_value)?)
        }
        TdgOutputFormat::Markdown => {
            let mut output = String::new();

            output.push_str("# TDG Score Report\n\n");
            if let Some(file_path) = &score.file_path {
                output.push_str(&format!("**File**: `{}`\n\n", file_path.display()));
            }

            output.push_str(&format!(
                "**Overall Score**: {:.1}/100 ({})\n",
                score.total,
                format_grade(score.grade)
            ));
            output.push_str(&format!(
                "**Language**: {:?} (confidence: {:.0}%)\n\n",
                score.language,
                score.confidence * 100.0
            ));

            if include_components {
                output.push_str("## Component Breakdown\n\n");
                output.push_str("| Component | Score | Max |\n");
                output.push_str("|-----------|-------|-----|\n");
                output.push_str(&format!(
                    "| Structural Complexity | {:.1} | 25 |\n",
                    score.structural_complexity
                ));
                output.push_str(&format!(
                    "| Semantic Complexity | {:.1} | 20 |\n",
                    score.semantic_complexity
                ));
                output.push_str(&format!(
                    "| Duplication | {:.1} | 20 |\n",
                    score.duplication_ratio
                ));
                output.push_str(&format!(
                    "| Coupling | {:.1} | 15 |\n",
                    score.coupling_score
                ));
                output.push_str(&format!(
                    "| Documentation | {:.1} | 10 |\n",
                    score.doc_coverage
                ));
                output.push_str(&format!(
                    "| Consistency | {:.1} | 10 |\n",
                    score.consistency_score
                ));
            }

            Ok(output)
        }
        TdgOutputFormat::Sarif => {
            // For SARIF format, return simplified score
            Ok(format!("{:.1}", score.total))
        }
    }
}

fn format_comparison(
    comparison: crate::tdg::Comparison,
    format: TdgOutputFormat,
) -> Result<String> {
    match format {
        TdgOutputFormat::Table => {
            let mut output = String::new();
            output.push_str("╭─────────────────────────────────────────────────╮\n");
            output.push_str("│  TDG Comparison                                 │\n");
            output.push_str("├─────────────────────────────────────────────────┤\n");
            output.push_str(&format!(
                "│  Source 1: {:.1} ({})                           │\n",
                comparison.source1.total,
                format_grade(comparison.source1.grade)
            ));
            output.push_str(&format!(
                "│  Source 2: {:.1} ({})                           │\n",
                comparison.source2.total,
                format_grade(comparison.source2.grade)
            ));
            output.push_str(&format!(
                "│  Difference: {:+.1}                             │\n",
                comparison.delta
            ));

            output.push_str(&format!(
                "│  Winner: {}                                      │\n",
                comparison.winner
            ));

            output.push_str("╰─────────────────────────────────────────────────╯\n");
            Ok(output)
        }
        _ => {
            // For other formats, output as JSON
            let json_value = serde_json::json!({
                "source1": {
                    "total": comparison.source1.total,
                    "grade": format_grade(comparison.source1.grade),
                },
                "source2": {
                    "total": comparison.source2.total,
                    "grade": format_grade(comparison.source2.grade),
                },
                "difference": comparison.delta,
                "winner": comparison.winner
            });
            Ok(serde_json::to_string_pretty(&json_value)?)
        }
    }
}

fn format_grade(grade: Grade) -> String {
    match grade {
        Grade::APLus => "A+",
        Grade::A => "A",
        Grade::AMinus => "A-",
        Grade::BPlus => "B+",
        Grade::B => "B",
        Grade::BMinus => "B-",
        Grade::CPlus => "C+",
        Grade::C => "C",
        Grade::CMinus => "C-",
        Grade::D => "D",
        Grade::F => "F",
    }
    .to_string()
}

fn parse_grade(grade_str: &str) -> Result<Grade> {
    match grade_str.to_uppercase().as_str() {
        "A+" | "APLUS" => Ok(Grade::APLus),
        "A" => Ok(Grade::A),
        "A-" | "AMINUS" => Ok(Grade::AMinus),
        "B+" | "BPLUS" => Ok(Grade::BPlus),
        "B" => Ok(Grade::B),
        "B-" | "BMINUS" => Ok(Grade::BMinus),
        "C+" | "CPLUS" => Ok(Grade::CPlus),
        "C" => Ok(Grade::C),
        "C-" | "CMINUS" => Ok(Grade::CMinus),
        "D" => Ok(Grade::D),
        "F" => Ok(Grade::F),
        _ => Err(anyhow!(
            "Invalid grade: {}. Valid grades are: A+, A, A-, B+, B, B-, C+, C, C-, D, F",
            grade_str
        )),
    }
}
