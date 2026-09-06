fn format_result(result: &CudaSimdTdgResult, config: &CudaTdgCommandConfig) -> Result<String> {
    match config.format {
        CudaTdgOutputFormat::Json => Ok(serde_json::to_string_pretty(result)?),
        CudaTdgOutputFormat::Sarif => Ok(format_sarif(result)?),
        CudaTdgOutputFormat::Markdown => Ok(format_markdown_report(result)?),
        CudaTdgOutputFormat::Terminal => {
            if config.quiet {
                Ok(format!("{:.1}", result.score.total))
            } else {
                Ok(format_terminal_output(result)?)
            }
        }
    }
}

fn format_analysis(result: &CudaSimdTdgResult, config: &CudaTdgCommandConfig) -> Result<String> {
    match config.format {
        CudaTdgOutputFormat::Json => Ok(serde_json::to_string_pretty(result)?),
        _ => {
            let mut output = String::new();
            output.push_str("CUDA-SIMD Analysis Results\n");
            output.push_str("==========================\n\n");
            output.push_str(&format!("Path: {}\n", result.path.display()));
            output.push_str(&format!("Files Analyzed: {}\n", result.files_analyzed));
            output.push_str(&format!(
                "  CUDA: {}, SIMD: {}, WGPU: {}\n\n",
                result.cuda_files, result.simd_files, result.wgpu_files
            ));
            output.push_str(&format!(
                "Score: {:.1}/100 (Grade: {})\n",
                result.score.total, result.score.grade
            ));
            output.push_str(&format!("Defects Found: {}\n", result.defects.len()));

            if !result.defects.is_empty() {
                output.push_str("\nDefects:\n");
                for defect in &result.defects {
                    output.push_str(&format!(
                        "  [{:?}] {} - {}\n",
                        defect.defect_class.severity,
                        defect.defect_class.ticket_id,
                        defect.defect_class.description
                    ));
                    if let Some(ref file) = defect.line {
                        output.push_str(&format!(
                            "    File: {}:{}\n",
                            defect.file_path.display(),
                            file
                        ));
                    }
                }
            }
            Ok(output)
        }
    }
}

fn format_score_summary(score: &PopperScore, config: &CudaTdgCommandConfig) -> Result<String> {
    match config.format {
        CudaTdgOutputFormat::Json => Ok(serde_json::to_string_pretty(score)?),
        // Only the terminal format may carry colour; markdown and sarif are
        // documents and stay plain whatever `--color` says (PMAT-688 quorum).
        CudaTdgOutputFormat::Terminal => {
            use crate::cli::colors as c;
            Ok(format!(
                "{:.1}/100 (Grade: {}, Gateway: {})",
                score.total,
                c::colored(grade_color(&score.grade), &score.grade.to_string()),
                if score.gateway_passed {
                    c::colored(c::GREEN, "PASSED")
                } else {
                    c::colored(c::RED, "FAILED")
                }
            ))
        }
        _ => Ok(format!(
            "{:.1}/100 (Grade: {}, Gateway: {})",
            score.total,
            score.grade,
            if score.gateway_passed {
                "PASSED"
            } else {
                "FAILED"
            }
        )),
    }
}

fn format_score_breakdown(score: &PopperScore, config: &CudaTdgCommandConfig) -> Result<String> {
    match config.format {
        CudaTdgOutputFormat::Json => Ok(serde_json::to_string_pretty(score)?),
        _ => Ok(build_score_breakdown_text(score)),
    }
}

fn build_score_breakdown_text(score: &PopperScore) -> String {
    let mut output = String::new();
    output.push_str("100-Point Popper Falsification Score\n");
    output.push_str("====================================\n\n");
    output.push_str(&format!(
        "Total: {:.1}/100 (Grade: {})\n",
        score.total, score.grade
    ));
    output.push_str(&format!(
        "Gateway: {}\n\n",
        if score.gateway_passed {
            "PASSED"
        } else {
            "FAILED"
        }
    ));

    output.push_str("Category Breakdown:\n");
    output.push_str("-------------------\n");
    output.push_str(&format!(
        "A. Falsifiability & Testability (GATEWAY): {:.1}/25\n",
        score.falsifiability.total()
    ));
    output.push_str(&format!(
        "   - Barrier Safety: {:.1}/5\n",
        score.falsifiability.barrier_safety
    ));
    output.push_str(&format!(
        "   - Bounds Verification: {:.1}/5\n",
        score.falsifiability.bounds_verification
    ));
    output.push_str(&format!(
        "   - Divergence Testing: {:.1}/5\n",
        score.falsifiability.divergence_testing
    ));
    output.push_str(&format!(
        "   - Memory Race Detection: {:.1}/5\n",
        score.falsifiability.memory_race_detection
    ));
    output.push_str(&format!(
        "   - Occupancy Bounds: {:.1}/5\n\n",
        score.falsifiability.occupancy_bounds
    ));

    output.push_str(&format!(
        "B. Reproducibility Infrastructure: {:.1}/25\n",
        score.reproducibility.total()
    ));
    output.push_str(&format!(
        "C. Transparency & Openness: {:.1}/20\n",
        score.transparency.total()
    ));
    output.push_str(&format!(
        "D. Statistical Rigor: {:.1}/15\n",
        score.statistical_rigor.total()
    ));
    output.push_str(&format!(
        "E. Historical Integrity: {:.1}/10\n",
        score.historical_integrity.total()
    ));
    output.push_str(&format!(
        "F. GPU/SIMD Specific: {:.1}/5\n",
        score.gpu_simd_specific.total()
    ));
    output
}

fn format_barrier_safety(
    result: &CudaSimdTdgResult,
    config: &CudaTdgCommandConfig,
) -> Result<String> {
    match config.format {
        CudaTdgOutputFormat::Json => Ok(serde_json::to_string_pretty(&result.barrier_safety)?),
        _ => Ok(build_barrier_safety_text(result)),
    }
}

fn build_barrier_safety_text(result: &CudaSimdTdgResult) -> String {
    let mut output = String::new();
    output.push_str("Barrier Safety Analysis (PARITY-114)\n");
    output.push_str("====================================\n\n");
    output.push_str(&format!(
        "Total Barriers: {}\n",
        result.barrier_safety.total_barriers
    ));
    output.push_str(&format!(
        "Safe Barriers: {}\n",
        result.barrier_safety.safe_barriers
    ));
    output.push_str(&format!(
        "Unsafe Barriers: {}\n",
        result.barrier_safety.unsafe_barriers.len()
    ));
    output.push_str(&format!(
        "Safety Score: {:.1}%\n\n",
        result.barrier_safety.safety_score * 100.0
    ));

    if !result.barrier_safety.unsafe_barriers.is_empty() {
        output.push_str("Unsafe Barriers Detected:\n");
        output.push_str("-------------------------\n");
        for issue in &result.barrier_safety.unsafe_barriers {
            output.push_str(&format!(
                "  Line {}: {} - {}\n",
                issue.line, issue.barrier_type, issue.issue
            ));
        }
    }
    output
}
