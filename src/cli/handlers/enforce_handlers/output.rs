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
use anyhow::{Context, Result};
use std::path::Path;

/// Send a finished report to `-o FILE`, or to stdout when no path was given.
///
/// `--output` was accepted, bound to `_output` in the handler and never read:
/// `pmat enforce extreme -o report.json` created no file, printed the payload to
/// stdout and exited 0, so a CI step that reads the file it asked for got
/// nothing and a success code. Every report this module produces goes through
/// here, so the flag cannot be honoured on one format and dropped on another.
pub(crate) fn emit_report(text: &str, output: Option<&Path>) -> Result<()> {
    match output {
        // A path we cannot write is an error, not a silent fall back to stdout:
        // falling back is how the flag came to mean nothing in the first place.
        Some(path) => std::fs::write(path, text)
            .with_context(|| format!("failed to write report to {}", path.display())),
        None => {
            println!("{text}");
            Ok(())
        }
    }
}

/// Output enforcement result in requested format
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn output_result(
    result: &EnforcementResult,
    format: EnforceOutputFormat,
    show_progress: bool,
    output: Option<&Path>,
) -> Result<()> {
    // `--show-progress` was honoured by exactly one of the four formats, and the
    // default format is not that one, so the flag produced byte-identical output
    // to a run without it. The bar belongs to the run, not to the report: with a
    // machine-readable format it goes to stderr so `| jq` still parses stdout,
    // and with `-o` it never contaminates the file.
    if show_progress
        && matches!(
            format,
            EnforceOutputFormat::Json | EnforceOutputFormat::Sarif
        )
    {
        eprint!("{}", render_progress_bar(result));
    }
    let mut text = String::new();
    if show_progress
        && matches!(
            format,
            EnforceOutputFormat::Summary | EnforceOutputFormat::Progress
        )
    {
        text.push_str(&render_progress_bar(result));
    }
    match format {
        EnforceOutputFormat::Json => {
            text.push_str(&serde_json::to_string_pretty(result)?);
        }
        EnforceOutputFormat::Summary => {
            text.push_str(&format!("{} {:?}\n", c::label("State:"), result.state));
            text.push_str(&format!(
                "{} {}{:.2}{}/{}{:.2}{}\n",
                c::label("Score:"),
                c::BOLD_WHITE,
                result.score,
                c::RESET,
                c::DIM,
                result.target,
                c::RESET
            ));
            if let Some(file) = &result.current_file {
                text.push_str(&format!(
                    "{} {}\n",
                    c::label("Current File:"),
                    c::path(file)
                ));
            }
            text.push_str(&format!(
                "{} {}",
                c::label("Violations:"),
                c::number(&result.violations.len().to_string())
            ));
        }
        EnforceOutputFormat::Progress => {
            text.push_str(&format!("{} {:?}\n", c::label("State:"), result.state));
            text.push_str(&format!(
                "{} {}{:.2}{}/{}{:.2}{}",
                c::label("Score:"),
                c::BOLD_WHITE,
                result.score,
                c::RESET,
                c::DIM,
                result.target,
                c::RESET
            ));
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
            text.push_str(&serde_json::to_string_pretty(&sarif)?);
        }
    }
    emit_report(&text, output)
}

/// Render the visual progress bar.
///
/// One renderer, so the bar is identical whether it goes to stdout, to stderr
/// beside a JSON payload, or into an `-o` file.
#[must_use]
pub fn render_progress_bar(result: &EnforcementResult) -> String {
    let percentage = (result.score * 100.0) as u32;
    let filled = (percentage as f32 / 5.0) as usize;
    let empty = 20usize.saturating_sub(filled);

    let bar_color = if percentage >= 80 {
        c::GREEN
    } else if percentage >= 50 {
        c::YELLOW
    } else {
        c::RED
    };

    format!(
        "\n{}\n{}\n{} {}{:.2}{}/1.00 {}{}{}{}{}{} {}\n\n",
        c::header("Extreme Quality Enforcement Progress"),
        c::rule(),
        c::label("Overall Score:"),
        c::BOLD_WHITE,
        result.score,
        c::RESET,
        bar_color,
        "\u{2588}".repeat(filled),
        c::RESET,
        c::DIM,
        "\u{2591}".repeat(empty),
        c::RESET,
        c::pct(f64::from(percentage), 80.0, 50.0)
    )
}

/// Print visual progress bar
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn print_progress_bar(result: &EnforcementResult) {
    print!("{}", render_progress_bar(result));
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
