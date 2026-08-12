/// Format output based on format type
///
/// `evolution`/`days` are gone from this dispatcher: nothing measured debt over
/// time. Summary and SARIF ignored the flag outright, JSON emitted the literal
/// string "Evolution tracking would show SATD trends over time" and Markdown a
/// "## Evolution (Last N Days)" heading over the same sentence, so `--days`
/// only ever changed a heading. `handle_analyze_satd` now refuses `--evolution`
/// instead of rendering a placeholder that looks like a measurement.
fn format_output(
    result: &SatdAnalysisResult,
    format: SatdOutputFormat,
    metrics: bool,
    top_files: usize,
) -> String {
    match format {
        SatdOutputFormat::Summary => format_summary(result, top_files),
        SatdOutputFormat::Json => format_json(result, metrics, top_files),
        // SARIF alone lists every violation regardless of `--top-files`: it is
        // the format an IDE/code-scanner ingests, and a row limit there hides
        // real findings from the tool that is supposed to surface them.
        SatdOutputFormat::Sarif => format_sarif(result),
        SatdOutputFormat::Markdown => format_markdown(result, top_files),
    }
}

/// Format as summary
fn format_summary(result: &SatdAnalysisResult, top_files: usize) -> String {
    use crate::cli::colors as c;

    let mut output = String::new();
    output.push_str(&format!("{}\n\n", c::header("SATD Analysis Summary")));
    output.push_str(&result.summary);
    output.push_str(&format!(
        "\n\n{}  {}\n",
        c::label("Total violations:"),
        c::number(&result.violations.len().to_string())
    ));

    // Group by severity
    let critical_count = result
        .violations
        .iter()
        .filter(|v| {
            matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::Critical
            )
        })
        .count();
    let high_count = result
        .violations
        .iter()
        .filter(|v| {
            matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::High
            )
        })
        .count();
    let medium_count = result
        .violations
        .iter()
        .filter(|v| {
            matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::Medium
            )
        })
        .count();
    let low_count = result
        .violations
        .iter()
        .filter(|v| {
            matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::Low
            )
        })
        .count();

    // `--color never` / NO_COLOR reached these lines and changed nothing: the
    // raw `c::BOLD`/`c::RED`/`c::RESET` consts are unconditional, so
    // `analyze satd ... --color never > out.txt` still wrote five escape-bearing
    // lines. `c::seq` is the same sequence gated on `colors_enabled()`.
    output.push_str(&format!("\n{}\n", c::subheader("Severity Distribution")));
    output.push_str(&format!(
        "  {}{}Critical:{} {}\n",
        c::seq(c::BOLD),
        c::seq(c::RED),
        c::seq(c::RESET),
        c::number(&critical_count.to_string())
    ));
    output.push_str(&format!(
        "  {}{}High:{} {}\n",
        c::seq(c::BOLD),
        c::seq(c::RED),
        c::seq(c::RESET),
        c::number(&high_count.to_string())
    ));
    output.push_str(&format!(
        "  {}{}Medium:{} {}\n",
        c::seq(c::BOLD),
        c::seq(c::YELLOW),
        c::seq(c::RESET),
        c::number(&medium_count.to_string())
    ));
    output.push_str(&format!(
        "  {}{}Low:{} {}\n",
        c::seq(c::BOLD),
        c::seq(c::GREEN),
        c::seq(c::RESET),
        c::number(&low_count.to_string())
    ));

    if !result.violations.is_empty() {
        // `.take(10)` was hardcoded here: over a corpus of 63 violations,
        // `--top-files 1` and `--top-files 50` both printed ten rows.
        let shown = crate::cli::top_files_slice(&result.violations, top_files);
        output.push_str(&format!("\n{}\n", c::subheader("Top Violations")));
        for (i, violation) in shown.iter().enumerate() {
            let sev_color = match violation.severity {
                crate::services::facades::satd_facade::SatdSeverity::Critical
                | crate::services::facades::satd_facade::SatdSeverity::High => c::RED,
                crate::services::facades::satd_facade::SatdSeverity::Medium => c::YELLOW,
                crate::services::facades::satd_facade::SatdSeverity::Low => c::GREEN,
            };
            output.push_str(&format!(
                "  {}. {}:{} - {} {}{:?}{}\n",
                c::number(&(i + 1).to_string()),
                c::path(&violation.file_path),
                c::dim(&violation.line_number.to_string()),
                violation.violation_type,
                c::seq(sev_color),
                violation.severity,
                c::seq(c::RESET)
            ));
        }
        // A list that is secretly a cap tells the reader nothing about what it
        // hid; name both numbers, the way the big-O surfaces already do.
        if shown.len() < result.violations.len() {
            output.push_str(&format!(
                "  {}\n",
                c::dim(&format!(
                    "… {} more not shown (--top-files {}, 0 = all)",
                    result.violations.len() - shown.len(),
                    top_files
                ))
            ));
        }
    }

    output
}

/// Format as JSON
fn format_json(result: &SatdAnalysisResult, metrics: bool, top_files: usize) -> String {
    let listed = crate::cli::top_files_slice(&result.violations, top_files);
    let mut json_data = serde_json::json!({
        "total_files": result.total_files,
        "total_violations": result.violations.len(),
        "violations_listed": listed.len(),
        "violations_truncated": listed.len() < result.violations.len(),
        "summary": result.summary,
        "violations": listed.iter().map(|v| {
            serde_json::json!({
                "file": v.file_path,
                "line": v.line_number,
                "type": v.violation_type,
                "message": v.message,
                "severity": format!("{:?}", v.severity)
            })
        }).collect::<Vec<_>>()
    });

    if metrics {
        json_data["metrics"] = serde_json::json!({
            "critical_count": result.violations.iter()
                .filter(|v| matches!(v.severity, crate::services::facades::satd_facade::SatdSeverity::Critical))
                .count(),
            "high_count": result.violations.iter()
                .filter(|v| matches!(v.severity, crate::services::facades::satd_facade::SatdSeverity::High))
                .count(),
            "medium_count": result.violations.iter()
                .filter(|v| matches!(v.severity, crate::services::facades::satd_facade::SatdSeverity::Medium))
                .count(),
            "low_count": result.violations.iter()
                .filter(|v| matches!(v.severity, crate::services::facades::satd_facade::SatdSeverity::Low))
                .count(),
        });
    }

    serde_json::to_string_pretty(&json_data).unwrap_or_else(|_| "{}".to_string())
}

/// Format as SARIF
fn format_sarif(result: &SatdAnalysisResult) -> String {
    let rules = vec![serde_json::json!({
        "id": "satd-violation",
        "shortDescription": {
            "text": "Self-Admitted Technical Debt"
        },
        "fullDescription": {
            "text": "Code contains self-admitted technical debt that should be addressed"
        }
    })];

    let results: Vec<_> = result
        .violations
        .iter()
        .map(|violation| {
            let level = match violation.severity {
                crate::services::facades::satd_facade::SatdSeverity::Critical => "error",
                crate::services::facades::satd_facade::SatdSeverity::High => "error",
                crate::services::facades::satd_facade::SatdSeverity::Medium => "warning",
                crate::services::facades::satd_facade::SatdSeverity::Low => "note",
            };

            serde_json::json!({
                "ruleId": "satd-violation",
                "level": level,
                "message": {
                    "text": format!("{}: {}", violation.violation_type, violation.message)
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": violation.file_path.clone()
                        },
                        "region": {
                            "startLine": violation.line_number
                        }
                    }
                }]
            })
        })
        .collect();

    serde_json::json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat-satd-detector",
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

/// Format as Markdown
fn format_markdown(result: &SatdAnalysisResult, top_files: usize) -> String {
    let mut output = String::new();
    output.push_str("# SATD Analysis Report\n\n");
    output.push_str(&format!("**Summary:** {}\n\n", result.summary));

    output.push_str("## Metrics\n\n");
    output.push_str("| Metric | Value |\n");
    output.push_str("|--------|-------|\n");
    output.push_str(&format!("| Total Files | {} |\n", result.total_files));
    output.push_str(&format!(
        "| Total Violations | {} |\n",
        result.violations.len()
    ));

    let critical_count = result
        .violations
        .iter()
        .filter(|v| {
            matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::Critical
            )
        })
        .count();
    let high_count = result
        .violations
        .iter()
        .filter(|v| {
            matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::High
            )
        })
        .count();

    output.push_str(&format!("| Critical Violations | {critical_count} |\n"));
    output.push_str(&format!("| High Violations | {high_count} |\n\n"));

    if !result.violations.is_empty() {
        output.push_str("## Violations\n\n");
        output.push_str("| File | Line | Type | Severity | Message |\n");
        output.push_str("|------|------|------|----------|----------|\n");

        let shown = crate::cli::top_files_slice(&result.violations, top_files);
        for violation in shown {
            output.push_str(&format!(
                "| {} | {} | {} | {:?} | {} |\n",
                violation.file_path,
                violation.line_number,
                violation.violation_type,
                violation.severity,
                violation.message
            ));
        }
        if shown.len() < result.violations.len() {
            output.push_str(&format!(
                "\n_{} of {} violations shown (`--top-files {}`, 0 = all)._\n",
                shown.len(),
                result.violations.len(),
                top_files
            ));
        }
    }

    output
}

/// Print metrics to stderr
fn print_metrics(result: &SatdAnalysisResult) {
    use crate::cli::colors as c;

    eprintln!("\n{} SATD Metrics:", c::subheader("📊"));
    eprintln!(
        "  {} {}",
        c::label("Total files analyzed:"),
        c::number(&result.total_files.to_string())
    );
    eprintln!(
        "  {} {}",
        c::label("Total violations:"),
        c::number(&result.violations.len().to_string())
    );

    let critical_count = result
        .violations
        .iter()
        .filter(|v| {
            matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::Critical
            )
        })
        .count();
    let high_count = result
        .violations
        .iter()
        .filter(|v| {
            matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::High
            )
        })
        .count();

    // Same unconditional-const defect as the severity block above.
    eprintln!(
        "  {}Critical violations:{} {}",
        c::seq(c::BOLD_RED),
        c::seq(c::RESET),
        c::number(&critical_count.to_string())
    );
    eprintln!(
        "  {}High violations:{} {}",
        c::seq(c::BOLD_RED),
        c::seq(c::RESET),
        c::number(&high_count.to_string())
    );

    if !result.violations.is_empty() {
        eprintln!("\n  {}:", c::label("Top violation types"));
        use std::collections::HashMap;
        let mut type_counts: HashMap<&str, usize> = HashMap::new();
        for violation in &result.violations {
            *type_counts.entry(&violation.violation_type).or_insert(0) += 1;
        }

        let mut sorted_types: Vec<_> = type_counts.iter().collect();
        sorted_types.sort_by(|a, b| b.1.cmp(a.1));

        for (violation_type, count) in sorted_types.iter().take(5) {
            eprintln!(
                "    - {}: {}",
                violation_type,
                c::number(&count.to_string())
            );
        }
    }
}
