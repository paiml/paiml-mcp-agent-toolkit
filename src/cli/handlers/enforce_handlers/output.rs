#![cfg_attr(coverage_nightly, coverage(off))]
//! Output and formatting for enforcement results

use super::types::{EnforcementResult, EnforcementState, QualityProfile, QualityViolation};
use crate::cli::colors as c;

fn parse_line_num(location: &str) -> i32 {
    location
        .split(':')
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1)
}
use crate::cli::EnforceOutputFormat;
use anyhow::Result;

/// Output enforcement result in requested format
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
            println!("{} {:?}", c::label("State:"), result.state);
            println!(
                "{} {}{:.2}{}/{}{:.2}{}",
                c::label("Score:"),
                c::BOLD_WHITE,
                result.score,
                c::RESET,
                c::DIM,
                result.target,
                c::RESET
            );
            if let Some(file) = &result.current_file {
                println!("{} {}", c::label("Current File:"), c::path(file));
            }
            println!(
                "{} {}",
                c::label("Violations:"),
                c::number(&result.violations.len().to_string())
            );
        }
        EnforceOutputFormat::Progress => {
            if show_progress {
                print_progress_bar(result);
            }
            println!("{} {:?}", c::label("State:"), result.state);
            println!(
                "{} {}{:.2}{}/{}{:.2}{}",
                c::label("Score:"),
                c::BOLD_WHITE,
                result.score,
                c::RESET,
                c::DIM,
                result.target,
                c::RESET
            );
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
                                        "startLine": parse_line_num(&v.location)
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn print_progress_bar(result: &EnforcementResult) {
    let percentage = (result.score * 100.0) as u32;
    let filled = (percentage as f32 / 5.0) as usize;
    let empty = 20 - filled;

    println!("\n{}", c::header("Extreme Quality Enforcement Progress"));
    println!("{}", c::rule());
    print!(
        "{} {}{:.2}{}/1.00 ",
        c::label("Overall Score:"),
        c::BOLD_WHITE,
        result.score,
        c::RESET
    );
    let bar_color = if percentage >= 80 {
        c::GREEN
    } else if percentage >= 50 {
        c::YELLOW
    } else {
        c::RED
    };
    print!("{}{}{}", bar_color, "\u{2588}".repeat(filled), c::RESET);
    print!("{}{}{}", c::DIM, "\u{2591}".repeat(empty), c::RESET);
    println!(" {}", c::pct(f64::from(percentage), 80.0, 50.0));
    println!();
}

/// Print enforcement header
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn print_enforcement_header(project_path: &std::path::Path) {
    eprintln!("{}", c::header("Starting Extreme Quality Enforcement"));
    eprintln!(
        "{} {}",
        c::label("Project:"),
        c::path(&project_path.display().to_string())
    );
}

/// Print enforcement summary
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn print_enforcement_summary(
    current_score: f64,
    iteration: u32,
    duration: std::time::Duration,
) {
    eprintln!("\n{}", c::header("Enforcement Complete"));
    eprintln!(
        "{} {}{current_score:.2}{}/1.00",
        c::label("Final Score:"),
        c::BOLD_WHITE,
        c::RESET
    );
    eprintln!(
        "{} {}",
        c::label("Iterations:"),
        c::number(&iteration.to_string())
    );
    eprintln!("{} {duration:?}", c::label("Duration:"));
}

/// Handle CI mode exit
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn handle_ci_mode_exit(ci_mode: bool, current_state: EnforcementState) {
    if ci_mode && current_state != EnforcementState::Complete {
        std::process::exit(1);
    }
}

/// Format violations output - extracted from `list_all_violations` (complexity: ≤10)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
        output.push_str(&format!(
            "{} {} violations:\n\n",
            c::label("Found"),
            c::number(&violations.len().to_string())
        ));

        for violation in violations {
            let sev_color = match violation.severity.as_str() {
                "high" => c::BOLD_RED,
                "medium" => c::BOLD_YELLOW,
                _ => c::DIM_WHITE,
            };
            output.push_str(&format!(
                "{}{}{} [{}{}{}]: {} (current: {}, target: {})\n  -> {}\n\n",
                c::BOLD,
                violation.violation_type.to_uppercase(),
                c::RESET,
                sev_color,
                violation.severity,
                c::RESET,
                c::path(&violation.location),
                c::number(&format!("{}", violation.current)),
                c::number(&format!("{}", violation.target)),
                violation.suggestion
            ));
        }

        Ok(output)
    }
}
