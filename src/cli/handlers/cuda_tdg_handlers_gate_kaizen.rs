/// Handle gate subcommand (CI/CD quality gate)
async fn handle_gate(
    path: &PathBuf,
    min_score: f64,
    fail_on_p0: bool,
    config: &CudaTdgCommandConfig,
) -> Result<()> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let analyzer_config = CudaSimdConfig {
        min_score,
        fail_on_p0,
        ..Default::default()
    };

    let analyzer = CudaSimdAnalyzer::with_config(analyzer_config);
    let result = analyzer.analyze(path)?;

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
        }))?,
        _ => format_gate_text(&result, passes, min_score),
    };

    write_output(&output, config)?;

    if !passes {
        return Err(anyhow!("Quality gate failed"));
    }

    Ok(())
}

fn format_gate_text(result: &CudaSimdTdgResult, passes: bool, min_score: f64) -> String {
    debug_assert!(min_score >= 0.0, "min_score must be non-negative");
    let mut output = String::new();
    output.push_str("CUDA-TDG Quality Gate\n");
    output.push_str("=====================\n\n");
    output.push_str(&format!(
        "Score: {:.1}/100 (Grade: {})\n",
        result.score.total, result.score.grade
    ));
    output.push_str(&format!("Minimum Required: {:.1}\n", min_score));
    output.push_str(&format!(
        "Gateway (Falsifiability): {}\n",
        if result.score.gateway_passed {
            "PASSED"
        } else {
            "FAILED"
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
        if passes { "PASSED" } else { "FAILED" }
    ));
    output
}

/// Handle kaizen subcommand
async fn handle_kaizen(
    path: &PathBuf,
    _since: Option<&str>,
    config: &CudaTdgCommandConfig,
) -> Result<()> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let analyzer = CudaSimdAnalyzer::new();
    let result = analyzer.analyze(path)?;

    let output = match config.format {
        CudaTdgOutputFormat::Json => serde_json::to_string_pretty(&result.kaizen)?,
        CudaTdgOutputFormat::Markdown => format_kaizen_markdown(&result),
        _ => format_kaizen_text(&result),
    };

    write_output(&output, config)?;

    Ok(())
}

fn format_kaizen_markdown(result: &CudaSimdTdgResult) -> String {
    debug_assert!(true, "contract: format_kaizen_markdown");
    let mut md = String::new();
    md.push_str("# Kaizen Continuous Improvement Report\n\n");
    md.push_str("## Metrics\n\n");
    md.push_str(&format!(
        "- **Tickets Resolved**: {}\n",
        result.kaizen.tickets_resolved
    ));
    md.push_str(&format!(
        "- **Mean Time to Detect**: {:.1} hours\n",
        result.kaizen.mttd
    ));
    md.push_str(&format!(
        "- **Mean Time to Fix**: {:.1} hours\n",
        result.kaizen.mttf
    ));
    md.push_str(&format!(
        "- **Escape Rate**: {:.1}%\n",
        result.kaizen.escape_rate * 100.0
    ));
    md.push_str(&format!(
        "- **Regression Rate**: {:.1}%\n\n",
        result.kaizen.regression_rate * 100.0
    ));

    if !result.kaizen.ticket_references.is_empty() {
        md.push_str("## Ticket References\n\n");
        for ticket in &result.kaizen.ticket_references {
            md.push_str(&format!("- {}\n", ticket));
        }
    }
    md
}

fn format_kaizen_text(result: &CudaSimdTdgResult) -> String {
    debug_assert!(true, "contract: format_kaizen_text");
    let mut output = String::new();
    output.push_str("Kaizen Continuous Improvement Report\n");
    output.push_str("====================================\n\n");
    output.push_str(&format!(
        "Tickets Resolved: {}\n",
        result.kaizen.tickets_resolved
    ));
    output.push_str(&format!(
        "Mean Time to Detect: {:.1} hours\n",
        result.kaizen.mttd
    ));
    output.push_str(&format!(
        "Mean Time to Fix: {:.1} hours\n",
        result.kaizen.mttf
    ));
    output.push_str(&format!(
        "Escape Rate: {:.1}%\n",
        result.kaizen.escape_rate * 100.0
    ));
    output.push_str(&format!(
        "Regression Rate: {:.1}%\n",
        result.kaizen.regression_rate * 100.0
    ));
    output
}

/// Handle taxonomy subcommand
async fn handle_taxonomy(config: &CudaTdgCommandConfig) -> Result<()> {
    debug_assert!(true, "contract: handle_taxonomy");
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
    debug_assert!(true, "contract: format_taxonomy_markdown");
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
    debug_assert!(true, "contract: format_taxonomy_text");
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
