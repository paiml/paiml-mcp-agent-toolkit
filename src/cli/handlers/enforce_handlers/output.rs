#![cfg_attr(coverage_nightly, coverage(off))]
//! Output and formatting for enforcement results

use super::types::{EnforcementResult, EnforcementState, QualityProfile, QualityViolation};
use crate::cli::EnforceOutputFormat;
use anyhow::Result;

/// Output enforcement result in requested format
pub fn output_result(
    result: &EnforcementResult,
    format: EnforceOutputFormat,
    show_progress: bool,
) -> Result<()> {
    match format {
        EnforceOutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(result)?);
        }
        EnforceOutputFormat::Summary => {
            println!("State: {:?}", result.state);
            println!("Score: {:.2}/{:.2}", result.score, result.target);
            if let Some(file) = &result.current_file {
                println!("Current File: {file}");
            }
            println!("Violations: {}", result.violations.len());
        }
        EnforceOutputFormat::Progress => {
            if show_progress {
                print_progress_bar(result);
            }
            println!("State: {:?}", result.state);
            println!("Score: {:.2}/{:.2}", result.score, result.target);
        }
        EnforceOutputFormat::Sarif => {
            // Generate SARIF output
            let sarif = serde_json::json!({
                "version": "2.1.0",
                "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
                "runs": [{
                    "tool": {
                        "driver": {
                            "name": "pmat-enforce-extreme",
                            "version": env!("CARGO_PKG_VERSION"),
                            "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit"
                        }
                    },
                    "results": result.violations.iter().map(|v| {
                        serde_json::json!({
                            "ruleId": format!("quality.{}", v.violation_type),
                            "level": match v.severity.as_str() {
                                "high" => "error",
                                "medium" => "warning",
                                _ => "note"
                            },
                            "message": {
                                "text": format!("{} (current: {:.1}, target: {:.1})",
                                    v.suggestion, v.current, v.target)
                            },
                            "locations": [{
                                "physicalLocation": {
                                    "artifactLocation": {
                                        "uri": v.location.split(':').next().unwrap_or(&v.location)
                                    },
                                    "region": {
                                        "startLine": v.location.split(':').nth(1)
                                            .and_then(|s| s.parse::<i32>().ok())
                                            .unwrap_or(1)
                                    }
                                }
                            }]
                        })
                    }).collect::<Vec<_>>()
                }]
            });
            println!("{}", serde_json::to_string_pretty(&sarif)?);
        }
    }
    Ok(())
}

/// Print visual progress bar
pub fn print_progress_bar(result: &EnforcementResult) {
    let percentage = (result.score * 100.0) as u32;
    let filled = (percentage as f32 / 5.0) as usize;
    let empty = 20 - filled;

    println!("\n🎯 Extreme Quality Enforcement Progress");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    print!("Overall Score: {:.2}/1.00 ", result.score);
    print!("{}", "█".repeat(filled));
    print!("{}", "░".repeat(empty));
    println!(" {percentage}%");
    println!();
}

/// Print enforcement header
pub fn print_enforcement_header(project_path: &std::path::Path) {
    eprintln!("🎯 Starting Extreme Quality Enforcement");
    eprintln!("📁 Project: {}", project_path.display());
}

/// Print enforcement summary
pub fn print_enforcement_summary(
    current_score: f64,
    iteration: u32,
    duration: std::time::Duration,
) {
    eprintln!("\n🏁 Enforcement Complete");
    eprintln!("📊 Final Score: {current_score:.2}/1.00");
    eprintln!("🔄 Iterations: {iteration}");
    eprintln!("⏱️  Duration: {duration:?}");
}

/// Handle CI mode exit
pub fn handle_ci_mode_exit(ci_mode: bool, current_state: EnforcementState) {
    if ci_mode && current_state != EnforcementState::Complete {
        std::process::exit(1);
    }
}

/// Format violations output - extracted from `list_all_violations` (complexity: ≤10)
pub fn format_violations_output(
    violations: &[QualityViolation],
    profile: &QualityProfile,
    format: EnforceOutputFormat,
) -> Result<String> {
    if format == EnforceOutputFormat::Json {
        let json_output = serde_json::json!({
            "profile": profile.clone(),
            "violations": violations,
            "summary": {
                "total": violations.len(),
                "by_severity": {
                    "high": violations.iter().filter(|v| v.severity == "high").count(),
                    "medium": violations.iter().filter(|v| v.severity == "medium").count(),
                    "low": violations.iter().filter(|v| v.severity == "low").count(),
                },
                "by_type": {
                    "complexity": violations.iter().filter(|v| v.violation_type == "complexity").count(),
                    "satd": violations.iter().filter(|v| v.violation_type == "satd").count(),
                    "tdg": violations.iter().filter(|v| v.violation_type == "tdg").count(),
                }
            }
        });
        Ok(serde_json::to_string_pretty(&json_output)?)
    } else {
        // Simple text format
        let mut output = String::new();
        output.push_str(&format!("Found {} violations:\n\n", violations.len()));

        for violation in violations {
            output.push_str(&format!(
                "{} [{}]: {} (current: {}, target: {})\n  -> {}\n\n",
                violation.violation_type.to_uppercase(),
                violation.severity,
                violation.location,
                violation.current,
                violation.target,
                violation.suggestion
            ));
        }

        Ok(output)
    }
}
