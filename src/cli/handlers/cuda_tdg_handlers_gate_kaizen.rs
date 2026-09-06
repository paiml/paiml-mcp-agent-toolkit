/// Handle gate subcommand (CI/CD quality gate)
async fn handle_gate(
    path: &PathBuf,
    min_score: f64,
    fail_on_p0: bool,
    config: &CudaTdgCommandConfig,
) -> Result<()> {
    let analyzer_config = CudaSimdConfig {
        min_score,
        fail_on_p0,
        ..Default::default()
    };

    let analyzer = CudaSimdAnalyzer::with_config(analyzer_config);
    let result = analyzer.analyze(path)?;

    // GH-662: the gate used to print "Gateway (Falsifiability): PASSED" for a
    // path with zero files read. A gate that measured nothing has not passed.
    if report_if_unmeasured(&result, config)? {
        return Err(anyhow!(
            "Quality gate not evaluated: no analysable source files found under {}",
            path.display()
        ));
    }

    let passes = analyzer.passes_quality_gate(&result);

    let output = match config.format {
        CudaTdgOutputFormat::Json => serde_json::to_string_pretty(&serde_json::json!({
            "passes": passes,
            "score": result.score.total,
            "min_score": min_score,
            "grade": result.score.grade.to_string(),
            "gateway_passed": result.score.gateway_passed,
            "p0_defects": result.defects.iter()
                .filter(|d| d.defect_class.severity == DefectSeverity::P0Critical)
                .count(),
            "fail_on_p0": fail_on_p0,
        }))?,
        // Only the terminal format may carry colour; markdown and sarif stay
        // plain whatever `--color` says (PMAT-688 quorum).
        CudaTdgOutputFormat::Terminal => {
            format_gate_text(&result, passes, min_score, fail_on_p0, true)
        }
        _ => format_gate_text(&result, passes, min_score, fail_on_p0, false),
    };

    write_output(&output, config)?;

    if !passes {
        return Err(anyhow!("Quality gate failed"));
    }

    Ok(())
}

/// The P0 policy the gate applied, named so the report explains its own
/// verdict (PMAT-688): on a tree that already fails, `--fail-on-p0` changes
/// nothing else the reader can see.
fn p0_policy_line(fail_on_p0: bool) -> &'static str {
    if fail_on_p0 {
        "P0 policy: fail on any P0 defect (--fail-on-p0)"
    } else {
        "P0 policy: advisory (P0 defects are reported, not gated)"
    }
}

fn format_gate_text(
    result: &CudaSimdTdgResult,
    passes: bool,
    min_score: f64,
    fail_on_p0: bool,
    colour: bool,
) -> String {
    use crate::cli::colors as c;
    // `colour` is the format's say (terminal only); the helpers still defer
    // to `--color` / NO_COLOR / the tty through `colors_enabled()`.
    let paint = |sgr: c::Sgr, text: &str| {
        if colour {
            c::colored(sgr, text)
        } else {
            text.to_string()
        }
    };
    let mut output = String::new();
    output.push_str("CUDA-TDG Quality Gate\n");
    output.push_str("=====================\n\n");
    output.push_str(&format!(
        "Score: {:.1}/100 (Grade: {})\n",
        result.score.total, result.score.grade
    ));
    output.push_str(&format!("Minimum Required: {:.1}\n", min_score));
    output.push_str(p0_policy_line(fail_on_p0));
    output.push('\n');
    output.push_str(&format!(
        "Gateway (Falsifiability): {}\n",
        if result.score.gateway_passed {
            paint(c::GREEN, "PASSED")
        } else {
            paint(c::RED, "FAILED")
        }
    ));

    let p0_count = result
        .defects
        .iter()
        .filter(|d| d.defect_class.severity == DefectSeverity::P0Critical)
        .count();
    output.push_str(&format!("P0 Critical Defects: {}\n\n", p0_count));
    output.push_str(&format!(
        "Result: {}\n",
        if passes {
            paint(c::GREEN, "PASSED")
        } else {
            paint(c::RED, "FAILED")
        }
    ));
    output
}

/// Render a kaizen metric that the analyzer does not measure.
///
/// MTTD/MTTF/escape rate/regression rate have no defect-lifecycle source
/// behind them (see `build_kaizen_metrics`), so they arrive here as NaN and
/// must be shown as "not measured" rather than as a number.
fn kaizen_metric(value: f64, unit: &str) -> String {
    if value.is_finite() {
        format!("{:.1}{}", value, unit)
    } else {
        "not measured".to_string()
    }
}

fn kaizen_percentage(value: f64) -> String {
    kaizen_metric(value * 100.0, "%")
}

/// Handle kaizen subcommand
async fn handle_kaizen(
    path: &PathBuf,
    since: Option<&str>,
    config: &CudaTdgCommandConfig,
) -> Result<()> {
    // `--since` was bound to `_since` and dropped on the floor, so the flag
    // silently changed nothing. Nothing this report emits is derived from a
    // time window, so say so instead of pretending the filter applied.
    if let Some(since) = since {
        eprintln!(
            "Warning: --since {since} ignored — cuda-tdg kaizen reports no history-derived metrics"
        );
    }

    let analyzer = CudaSimdAnalyzer::new();
    let result = analyzer.analyze(path)?;

    let output = match config.format {
        CudaTdgOutputFormat::Json => serde_json::to_string_pretty(&result.kaizen)?,
        CudaTdgOutputFormat::Markdown => format_kaizen_markdown(&result),
        CudaTdgOutputFormat::Terminal => format_kaizen_text(&result, true),
        _ => format_kaizen_text(&result, false),
    };

    write_output(&output, config)?;

    Ok(())
}

fn format_kaizen_markdown(result: &CudaSimdTdgResult) -> String {
    let mut md = String::new();
    md.push_str("# Kaizen Continuous Improvement Report\n\n");
    md.push_str("## Metrics\n\n");
    md.push_str(&format!(
        "- **Tickets Resolved**: {}\n",
        result.kaizen.tickets_resolved
    ));
    md.push_str(&format!(
        "- **Mean Time to Detect**: {}\n",
        kaizen_metric(result.kaizen.mttd, " hours")
    ));
    md.push_str(&format!(
        "- **Mean Time to Fix**: {}\n",
        kaizen_metric(result.kaizen.mttf, " hours")
    ));
    md.push_str(&format!(
        "- **Escape Rate**: {}\n",
        kaizen_percentage(result.kaizen.escape_rate)
    ));
    md.push_str(&format!(
        "- **Regression Rate**: {}\n\n",
        kaizen_percentage(result.kaizen.regression_rate)
    ));

    if !result.kaizen.ticket_references.is_empty() {
        md.push_str("## Ticket References\n\n");
        for ticket in &result.kaizen.ticket_references {
            md.push_str(&format!("- {}\n", ticket));
        }
    }
    md
}

fn format_kaizen_text(result: &CudaSimdTdgResult, colour: bool) -> String {
    let mut output = String::new();
    let title = "Kaizen Continuous Improvement Report";
    if colour {
        output.push_str(&crate::cli::colors::header(title));
    } else {
        output.push_str(title);
    }
    output.push('\n');
    output.push_str("====================================\n\n");
    output.push_str(&format!(
        "Tickets Resolved: {}\n",
        result.kaizen.tickets_resolved
    ));
    output.push_str(&format!(
        "Mean Time to Detect: {}\n",
        kaizen_metric(result.kaizen.mttd, " hours")
    ));
    output.push_str(&format!(
        "Mean Time to Fix: {}\n",
        kaizen_metric(result.kaizen.mttf, " hours")
    ));
    output.push_str(&format!(
        "Escape Rate: {}\n",
        kaizen_percentage(result.kaizen.escape_rate)
    ));
    output.push_str(&format!(
        "Regression Rate: {}\n",
        kaizen_percentage(result.kaizen.regression_rate)
    ));
    output
}

#[cfg(test)]
mod kaizen_honesty_tests {
    use super::*;

    #[test]
    fn test_unmeasured_kaizen_metrics_render_as_not_measured() {
        // Regression: these printed 24.0 hours / 48.0 hours / 5.0% / 2.0% for
        // every input, from literals in build_kaizen_metrics.
        assert_eq!(kaizen_metric(f64::NAN, " hours"), "not measured");
        assert_eq!(kaizen_percentage(f64::NAN), "not measured");
        assert_eq!(kaizen_metric(12.5, " hours"), "12.5 hours");
        assert_eq!(kaizen_percentage(0.05), "5.0%");
    }

    #[test]
    fn test_kaizen_report_does_not_quote_the_old_defaults() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::write(dir.path().join("lib.rs"), "pub fn f() {}\n").unwrap();

        let result = CudaSimdAnalyzer::new().analyze(dir.path()).unwrap();
        let text = format_kaizen_text(&result, false);

        assert!(
            text.contains("Mean Time to Detect: not measured"),
            "MTTD must not be a hardcoded estimate: {text}"
        );
        assert!(
            text.contains("Escape Rate: not measured"),
            "escape rate must not be a hardcoded estimate: {text}"
        );
        assert!(!text.contains("24.0 hours"), "{text}");
        assert!(!text.contains("5.0%"), "{text}");
    }
}

/// Handle taxonomy subcommand
async fn handle_taxonomy(config: &CudaTdgCommandConfig) -> Result<()> {
    let taxonomy = DefectTaxonomy::with_tauranta_patterns();

    let output = match config.format {
        CudaTdgOutputFormat::Json => {
            let patterns: Vec<_> = taxonomy.all().collect();
            serde_json::to_string_pretty(&patterns)?
        }
        CudaTdgOutputFormat::Markdown => format_taxonomy_markdown(&taxonomy),
        _ => format_taxonomy_text(&taxonomy),
    };

    write_output(&output, config)?;

    Ok(())
}

fn format_taxonomy_markdown(taxonomy: &DefectTaxonomy) -> String {
    let mut md = String::new();
    md.push_str("# Tauranta Fault Taxonomy\n\n");
    md.push_str("## P0 Critical Defects\n\n");
    md.push_str("| Ticket | Description | Detection | Status |\n");
    md.push_str("|--------|-------------|-----------|--------|\n");

    for defect in taxonomy.all() {
        if defect.severity == DefectSeverity::P0Critical {
            md.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                defect.ticket_id,
                defect.description,
                defect.detection_method,
                if defect.resolved { "Resolved" } else { "Open" }
            ));
        }
    }

    md.push_str("\n## P1 Performance Defects\n\n");
    md.push_str("| Ticket | Description | Detection | Status |\n");
    md.push_str("|--------|-------------|-----------|--------|\n");

    for defect in taxonomy.all() {
        if defect.severity == DefectSeverity::P1Performance {
            md.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                defect.ticket_id,
                defect.description,
                defect.detection_method,
                if defect.resolved { "Resolved" } else { "Open" }
            ));
        }
    }

    md.push_str("\n## P2 Efficiency Defects\n\n");
    md.push_str("| Ticket | Description | Detection | Status |\n");
    md.push_str("|--------|-------------|-----------|--------|\n");

    for defect in taxonomy.all() {
        if defect.severity == DefectSeverity::P2Efficiency {
            md.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                defect.ticket_id,
                defect.description,
                defect.detection_method,
                if defect.resolved { "Resolved" } else { "Open" }
            ));
        }
    }
    md
}

fn format_taxonomy_text(taxonomy: &DefectTaxonomy) -> String {
    let mut output = String::new();
    output.push_str("Tauranta Fault Taxonomy\n");
    output.push_str("=======================\n\n");

    output.push_str("P0 Critical Defects:\n");
    output.push_str("-------------------\n");
    for defect in taxonomy.all() {
        if defect.severity == DefectSeverity::P0Critical {
            output.push_str(&format!(
                "  {} - {}\n    Detection: {}\n",
                defect.ticket_id, defect.description, defect.detection_method
            ));
        }
    }

    output.push_str("\nP1 Performance Defects:\n");
    output.push_str("-----------------------\n");
    for defect in taxonomy.all() {
        if defect.severity == DefectSeverity::P1Performance {
            output.push_str(&format!(
                "  {} - {}\n    Detection: {}\n",
                defect.ticket_id, defect.description, defect.detection_method
            ));
        }
    }

    output.push_str("\nP2 Efficiency Defects:\n");
    output.push_str("----------------------\n");
    for defect in taxonomy.all() {
        if defect.severity == DefectSeverity::P2Efficiency {
            output.push_str(&format!(
                "  {} - {}\n    Detection: {}\n",
                defect.ticket_id, defect.description, defect.detection_method
            ));
        }
    }
    output
}
