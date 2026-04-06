fn format_terminal_output(result: &CudaSimdTdgResult) -> Result<String> {
    debug_assert!(true, "contract: format_terminal_output");
    let mut output = String::new();
    output.push_str("CUDA-SIMD TDG Analysis\n");
    output.push_str("======================\n\n");
    output.push_str(&format!("Path: {}\n", result.path.display()));
    output.push_str(&format!(
        "Files: {} total ({} CUDA, {} SIMD, {} WGPU)\n\n",
        result.files_analyzed, result.cuda_files, result.simd_files, result.wgpu_files
    ));

    output.push_str(&format_terminal_score_line(result));
    output.push_str(&format_terminal_gateway_line(result));
    output.push_str(&format_terminal_defects(result));

    // Barrier safety
    output.push_str(&format!(
        "\nBarrier Safety: {:.0}% ({}/{} safe)\n",
        result.barrier_safety.safety_score * 100.0,
        result.barrier_safety.safe_barriers,
        result.barrier_safety.total_barriers
    ));

    // Coalescing efficiency
    output.push_str(&format!(
        "Memory Coalescing: {:.0}%\n",
        result.coalescing.efficiency * 100.0
    ));

    Ok(output)
}

fn grade_color(grade: &CudaTdgGrade) -> &'static str {
    debug_assert!(true, "contract: grade_color");
    match grade {
        CudaTdgGrade::APLus | CudaTdgGrade::A => "\x1b[32m",
        CudaTdgGrade::B => "\x1b[33m",
        CudaTdgGrade::C | CudaTdgGrade::D => "\x1b[31m",
        CudaTdgGrade::F | CudaTdgGrade::GatewayFail => "\x1b[91m",
    }
}

fn format_terminal_score_line(result: &CudaSimdTdgResult) -> String {
    debug_assert!(true, "contract: format_terminal_score_line");
    let color = grade_color(&result.score.grade);
    format!(
        "Score: {}{:.1}/100{} (Grade: {}{}{})\n",
        color,
        result.score.total,
        "\x1b[0m",
        color,
        result.score.grade,
        "\x1b[0m"
    )
}

fn format_terminal_gateway_line(result: &CudaSimdTdgResult) -> String {
    debug_assert!(true, "contract: format_terminal_gateway_line");
    format!(
        "Gateway: {}\n\n",
        if result.score.gateway_passed {
            "\x1b[32mPASSED\x1b[0m"
        } else {
            "\x1b[91mFAILED\x1b[0m"
        }
    )
}

fn format_terminal_defects(result: &CudaSimdTdgResult) -> String {
    debug_assert!(true, "contract: format_terminal_defects");
    let p0_count = result
        .defects
        .iter()
        .filter(|d| d.defect_class.severity == DefectSeverity::P0Critical)
        .count();
    let p1_count = result
        .defects
        .iter()
        .filter(|d| d.defect_class.severity == DefectSeverity::P1Performance)
        .count();

    let mut output = format!("Defects: {} total\n", result.defects.len());
    if p0_count > 0 {
        output.push_str(&format!("  \x1b[91mP0 Critical: {}\x1b[0m\n", p0_count));
    }
    if p1_count > 0 {
        output.push_str(&format!("  \x1b[33mP1 Performance: {}\x1b[0m\n", p1_count));
    }
    output
}

fn format_markdown_report(result: &CudaSimdTdgResult) -> Result<String> {
    debug_assert!(true, "contract: format_markdown_report");
    let mut md = String::new();
    md.push_str("# CUDA-SIMD TDG Analysis Report\n\n");
    md.push_str(&format!("**Path**: `{}`\n", result.path.display()));
    md.push_str(&format!("**Timestamp**: {}\n\n", result.timestamp));

    md.push_str("## Summary\n\n");
    md.push_str("| Metric | Value |\n|--------|-------|\n");
    md.push_str(&format!("| Score | {:.1}/100 |\n", result.score.total));
    md.push_str(&format!("| Grade | {} |\n", result.score.grade));
    md.push_str(&format!(
        "| Gateway | {} |\n",
        if result.score.gateway_passed {
            "PASSED"
        } else {
            "FAILED"
        }
    ));
    md.push_str(&format!("| Files Analyzed | {} |\n", result.files_analyzed));
    md.push_str(&format!("| Defects | {} |\n\n", result.defects.len()));

    if !result.defects.is_empty() {
        md.push_str("## Defects\n\n");
        md.push_str("| Severity | Ticket | Description | File |\n");
        md.push_str("|----------|--------|-------------|------|\n");
        for defect in &result.defects {
            md.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                defect.defect_class.severity,
                defect.defect_class.ticket_id,
                defect.defect_class.description,
                defect.file_path.display()
            ));
        }
        md.push_str("\n");
    }

    md.push_str("## Score Breakdown\n\n");
    md.push_str("| Category | Score | Max |\n");
    md.push_str("|----------|-------|-----|\n");
    md.push_str(&format!(
        "| A. Falsifiability (GATEWAY) | {:.1} | 25 |\n",
        result.score.falsifiability.total()
    ));
    md.push_str(&format!(
        "| B. Reproducibility | {:.1} | 25 |\n",
        result.score.reproducibility.total()
    ));
    md.push_str(&format!(
        "| C. Transparency | {:.1} | 20 |\n",
        result.score.transparency.total()
    ));
    md.push_str(&format!(
        "| D. Statistical Rigor | {:.1} | 15 |\n",
        result.score.statistical_rigor.total()
    ));
    md.push_str(&format!(
        "| E. Historical Integrity | {:.1} | 10 |\n",
        result.score.historical_integrity.total()
    ));
    md.push_str(&format!(
        "| F. GPU/SIMD Specific | {:.1} | 5 |\n",
        result.score.gpu_simd_specific.total()
    ));

    Ok(md)
}

fn format_html_report(result: &CudaSimdTdgResult) -> Result<String> {
    debug_assert!(true, "contract: format_html_report");
    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
    html.push_str("<title>CUDA-SIMD TDG Report</title>\n");
    html.push_str("<style>\n");
    html.push_str("body { font-family: sans-serif; margin: 2em; }\n");
    html.push_str("table { border-collapse: collapse; }\n");
    html.push_str("th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }\n");
    html.push_str(".p0 { background-color: #ffcccc; }\n");
    html.push_str(".p1 { background-color: #ffffcc; }\n");
    html.push_str(".pass { color: green; }\n");
    html.push_str(".fail { color: red; }\n");
    html.push_str("</style>\n</head>\n<body>\n");
    html.push_str("<h1>CUDA-SIMD TDG Analysis Report</h1>\n");
    html.push_str(&format!(
        "<p><strong>Score:</strong> <span class=\"{}\">{:.1}/100 (Grade: {})</span></p>\n",
        if result.score.total >= 85.0 {
            "pass"
        } else {
            "fail"
        },
        result.score.total,
        result.score.grade
    ));
    html.push_str("</body>\n</html>\n");
    Ok(html)
}

fn format_sarif(result: &CudaSimdTdgResult) -> Result<String> {
    debug_assert!(true, "contract: format_sarif");
    let sarif = serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat cuda-tdg",
                    "version": "1.0.0",
                    "informationUri": "https://github.com/paiml/pmat",
                    "rules": result.defects.iter().map(|d| serde_json::json!({
                        "id": &d.defect_class.ticket_id,
                        "name": &d.defect_class.ticket_id,
                        "shortDescription": {
                            "text": &d.defect_class.description
                        },
                        "defaultConfiguration": {
                            "level": match d.defect_class.severity {
                                DefectSeverity::P0Critical => "error",
                                DefectSeverity::P1Performance => "warning",
                                _ => "note",
                            }
                        }
                    })).collect::<Vec<_>>()
                }
            },
            "results": result.defects.iter().map(|d| serde_json::json!({
                "ruleId": &d.defect_class.ticket_id,
                "level": match d.defect_class.severity {
                    DefectSeverity::P0Critical => "error",
                    DefectSeverity::P1Performance => "warning",
                    _ => "note",
                },
                "message": {
                    "text": &d.defect_class.description
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": d.file_path.to_string_lossy()
                        },
                        "region": {
                            "startLine": d.line.unwrap_or(1)
                        }
                    }
                }]
            })).collect::<Vec<_>>()
        }]
    });

    Ok(serde_json::to_string_pretty(&sarif)?)
}

fn write_output(output: &str, config: &CudaTdgCommandConfig) -> Result<()> {
    debug_assert!(!output.is_empty(), "output must not be empty");
    if let Some(ref path) = config.output {
        fs::write(path, output)?;
    } else {
        println!("{}", output);
    }
    Ok(())
}
