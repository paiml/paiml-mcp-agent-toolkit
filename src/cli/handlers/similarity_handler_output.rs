fn print_performance_metrics(report: &ComprehensiveReport, elapsed: std::time::Duration) {
    use crate::cli::colors as c;
    eprintln!("\n{}", c::subheader("⏱️  Performance Metrics:"));
    eprintln!("  Total time: {}{elapsed:?}{}", c::BOLD_WHITE, c::RESET);
    eprintln!("  Clones found: {}", c::number(&report.metrics.total_clones.to_string()));
    eprintln!(
        "  Analysis rate: {}{:.0}{} LOC/sec",
        c::BOLD_WHITE,
        (report.exact_duplicates.len() * 1000) as f64 / elapsed.as_millis() as f64,
        c::RESET,
    );
}

fn format_csv_report(report: &ComprehensiveReport) -> Result<String> {
    use std::fmt::Write;
    let mut output = String::new();

    writeln!(
        &mut output,
        "Type,File1,Start1,End1,File2,Start2,End2,Similarity"
    )?;

    for block in &report.exact_duplicates {
        if block.locations.len() >= 2 {
            writeln!(
                &mut output,
                "Exact,{},{},{},{},{},{},100.0",
                block.locations[0].file.display(),
                block.locations[0].start_line,
                block.locations[0].end_line,
                block.locations[1].file.display(),
                block.locations[1].start_line,
                block.locations[1].end_line
            )?;
        }
    }

    for block in &report.structural_similarities {
        if block.locations.len() >= 2 {
            writeln!(
                &mut output,
                "Structural,{},{},{},{},{},{},{:.1}",
                block.locations[0].file.display(),
                block.locations[0].start_line,
                block.locations[0].end_line,
                block.locations[1].file.display(),
                block.locations[1].start_line,
                block.locations[1].end_line,
                block.similarity * 100.0
            )?;
        }
    }

    Ok(output)
}

fn format_sarif_report(report: &ComprehensiveReport) -> Result<String> {
    let mut results = Vec::new();

    for block in &report.exact_duplicates {
        for location in &block.locations {
            results.push(serde_json::json!({
                "ruleId": "duplicate-code",
                "level": "warning",
                "message": {
                    "text": format!("Exact duplicate found ({} lines)", block.lines)
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": location.file.display().to_string()
                        },
                        "region": {
                            "startLine": location.start_line,
                            "endLine": location.end_line
                        }
                    }
                }]
            }));
        }
    }

    let sarif = serde_json::json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat-similarity",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit"
                }
            },
            "results": results
        }]
    });

    Ok(serde_json::to_string_pretty(&sarif)?)
}

fn print_summary(report: &ComprehensiveReport) {
    use crate::cli::colors as c;
    eprintln!("\n{}", c::pass("Analysis Complete:"));
    let dup_color = if report.metrics.duplication_percentage < 5.0 { c::GREEN } else if report.metrics.duplication_percentage < 15.0 { c::YELLOW } else { c::RED };
    eprintln!(
        "  📊 Duplication: {}{:.1}%{}",
        dup_color, report.metrics.duplication_percentage, c::RESET,
    );
    eprintln!("  🔢 Total clones: {}", c::number(&report.metrics.total_clones.to_string()));
    eprintln!(
        "  📈 Average entropy: {}",
        c::number(&format!("{:.2}", report.metrics.average_entropy)),
    );

    if !report.refactoring_opportunities.is_empty() {
        eprintln!(
            "  💡 Refactoring opportunities: {}",
            c::number(&report.refactoring_opportunities.len().to_string()),
        );
    }
}
