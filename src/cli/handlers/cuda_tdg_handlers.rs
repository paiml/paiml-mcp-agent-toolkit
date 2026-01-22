//! CUDA-SIMD TDG CLI handlers
//!
//! Implements command handlers for the 100-point Popper falsification scoring system.
//! Analyzes CUDA PTX, SIMD (AVX2/AVX-512/NEON), and WGPU compute code.
//!
//! # Toyota Way Integration
//!
//! - **Jidoka**: Automatic quality gates that stop on P0 defect detection
//! - **Kaizen**: Continuous improvement through historical Tauranta fault analysis
//! - **Poka-Yoke**: Error-proofing through static analysis

#![allow(clippy::wildcard_in_or_patterns)]
#![allow(clippy::useless_format)]
#![allow(clippy::single_char_add_str)]

use crate::cli::commands::{CudaTdgCommand, CudaTdgOutputFormat};
use crate::tdg::{
    CudaSimdAnalyzer, CudaSimdConfig, CudaSimdTdgResult, CudaTdgGrade, DefectSeverity,
    DefectTaxonomy, PopperScore,
};
use anyhow::{anyhow, Result};
use std::fs;
use std::path::PathBuf;

/// Configuration for CUDA-TDG command handling
pub struct CudaTdgCommandConfig {
    pub path: PathBuf,
    pub command: Option<CudaTdgCommand>,
    pub format: CudaTdgOutputFormat,
    pub min_score: f64,
    pub fail_on_p0: bool,
    pub simd: bool,
    pub wgpu: bool,
    pub output: Option<PathBuf>,
    pub quiet: bool,
}

/// Main handler for cuda-tdg command
pub async fn handle_cuda_tdg_command(config: CudaTdgCommandConfig) -> Result<()> {
    if let Some(ref cmd) = config.command {
        return handle_cuda_tdg_subcommand(cmd, &config).await;
    }

    // Default behavior: analyze and score
    let analyzer_config = CudaSimdConfig {
        min_score: config.min_score,
        fail_on_p0: config.fail_on_p0,
        analyze_simd: config.simd,
        analyze_wgpu: config.wgpu,
        ..Default::default()
    };

    let analyzer = CudaSimdAnalyzer::with_config(analyzer_config);
    let result = analyzer.analyze(&config.path)?;

    let output = format_result(&result, &config)?;
    write_output(&output, &config)?;

    // Check quality gate
    if !analyzer.passes_quality_gate(&result) {
        return Err(anyhow!(
            "Quality gate failed: score {:.1} < min {:.1}",
            result.score.total,
            config.min_score
        ));
    }

    Ok(())
}

/// Handle CUDA-TDG subcommands
async fn handle_cuda_tdg_subcommand(
    cmd: &CudaTdgCommand,
    config: &CudaTdgCommandConfig,
) -> Result<()> {
    match cmd {
        CudaTdgCommand::Analyze { path } => handle_analyze(path, config).await,
        CudaTdgCommand::Score { path, breakdown } => handle_score(path, *breakdown, config).await,
        CudaTdgCommand::Report {
            path,
            format,
            output,
        } => handle_report(path, format, output.as_ref(), config).await,
        CudaTdgCommand::BarrierCheck { path } => handle_barrier_check(path, config).await,
        CudaTdgCommand::ValidateTiles {
            head_dim,
            tile_kv,
            shared_memory,
        } => handle_validate_tiles(*head_dim, *tile_kv, *shared_memory, config).await,
        CudaTdgCommand::Gate {
            path,
            min_score,
            fail_on_p0,
        } => handle_gate(path, *min_score, *fail_on_p0, config).await,
        CudaTdgCommand::Kaizen { path, since } => {
            handle_kaizen(path, since.as_deref(), config).await
        }
        CudaTdgCommand::Taxonomy => handle_taxonomy(config).await,
    }
}

/// Handle analyze subcommand
async fn handle_analyze(path: &PathBuf, config: &CudaTdgCommandConfig) -> Result<()> {
    let analyzer = CudaSimdAnalyzer::new();
    let result = analyzer.analyze(path)?;

    let output = format_analysis(&result, config)?;
    write_output(&output, config)?;

    Ok(())
}

/// Handle score subcommand
async fn handle_score(
    path: &PathBuf,
    breakdown: bool,
    config: &CudaTdgCommandConfig,
) -> Result<()> {
    let analyzer = CudaSimdAnalyzer::new();
    let result = analyzer.analyze(path)?;

    let output = if breakdown {
        format_score_breakdown(&result.score, config)?
    } else {
        format_score_summary(&result.score, config)?
    };

    write_output(&output, config)?;

    Ok(())
}

/// Handle report subcommand
async fn handle_report(
    path: &PathBuf,
    format: &str,
    output: Option<&PathBuf>,
    config: &CudaTdgCommandConfig,
) -> Result<()> {
    let analyzer = CudaSimdAnalyzer::new();
    let result = analyzer.analyze(path)?;

    let report = match format {
        "html" => format_html_report(&result)?,
        "json" => serde_json::to_string_pretty(&result)?,
        _ => format_markdown_report(&result)?,
    };

    if let Some(output_path) = output {
        fs::write(output_path, &report)?;
        println!("Report written to: {}", output_path.display());
    } else if let Some(ref output_path) = config.output {
        fs::write(output_path, &report)?;
        println!("Report written to: {}", output_path.display());
    } else {
        println!("{}", report);
    }

    Ok(())
}

/// Handle barrier-check subcommand
async fn handle_barrier_check(path: &PathBuf, config: &CudaTdgCommandConfig) -> Result<()> {
    let analyzer = CudaSimdAnalyzer::new();
    let result = analyzer.analyze(path)?;

    let output = format_barrier_safety(&result, config)?;
    write_output(&output, config)?;

    if !result.barrier_safety.unsafe_barriers.is_empty() {
        return Err(anyhow!(
            "Found {} unsafe barrier(s) - PARITY-114 risk detected",
            result.barrier_safety.unsafe_barriers.len()
        ));
    }

    Ok(())
}

/// Handle validate-tiles subcommand
async fn handle_validate_tiles(
    head_dim: usize,
    tile_kv: usize,
    shared_memory: usize,
    config: &CudaTdgCommandConfig,
) -> Result<()> {
    let output = match config.format {
        CudaTdgOutputFormat::Json => {
            let result = serde_json::json!({
                "head_dim": head_dim,
                "tile_kv": tile_kv,
                "shared_memory_limit": shared_memory,
                "valid": tile_kv >= head_dim,
                "shared_memory_required": tile_kv * head_dim * 2,
                "issues": if tile_kv < head_dim {
                    vec!["PAR-041: tile_kv < head_dim causes shared memory overflow"]
                } else {
                    vec![]
                }
            });
            serde_json::to_string_pretty(&result)?
        }
        _ => {
            let valid = tile_kv >= head_dim;
            let shared_required = tile_kv * head_dim * 2; // FP16

            let mut output = String::new();
            output.push_str("Tile Dimension Validation\n");
            output.push_str("=========================\n\n");
            output.push_str(&format!("Head Dimension: {}\n", head_dim));
            output.push_str(&format!("Tile KV: {}\n", tile_kv));
            output.push_str(&format!("Shared Memory Limit: {} bytes\n", shared_memory));
            output.push_str(&format!(
                "Shared Memory Required: {} bytes\n\n",
                shared_required
            ));

            if valid && shared_required <= shared_memory {
                output.push_str("Status: VALID\n");
            } else {
                output.push_str("Status: INVALID\n\n");
                if tile_kv < head_dim {
                    output.push_str("Issue: PAR-041 - tile_kv < head_dim\n");
                    output.push_str(&format!(
                        "Fix: Set tile_kv >= {} (currently {})\n",
                        head_dim, tile_kv
                    ));
                }
                if shared_required > shared_memory {
                    output.push_str("Issue: Shared memory overflow\n");
                    output.push_str(&format!(
                        "Fix: Reduce tile size or increase shared memory limit\n"
                    ));
                }
            }
            output
        }
    };

    write_output(&output, config)?;

    if tile_kv < head_dim {
        return Err(anyhow!(
            "PAR-041: tile_kv ({}) < head_dim ({})",
            tile_kv,
            head_dim
        ));
    }

    Ok(())
}

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
        _ => {
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
    };

    write_output(&output, config)?;

    if !passes {
        return Err(anyhow!("Quality gate failed"));
    }

    Ok(())
}

/// Handle kaizen subcommand
async fn handle_kaizen(
    path: &PathBuf,
    _since: Option<&str>,
    config: &CudaTdgCommandConfig,
) -> Result<()> {
    let analyzer = CudaSimdAnalyzer::new();
    let result = analyzer.analyze(path)?;

    let output = match config.format {
        CudaTdgOutputFormat::Json => serde_json::to_string_pretty(&result.kaizen)?,
        CudaTdgOutputFormat::Markdown => {
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
        _ => {
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
    };

    write_output(&output, config)?;

    Ok(())
}

/// Handle taxonomy subcommand
async fn handle_taxonomy(config: &CudaTdgCommandConfig) -> Result<()> {
    let taxonomy = DefectTaxonomy::with_tauranta_patterns();

    let output = match config.format {
        CudaTdgOutputFormat::Json => {
            let patterns: Vec<_> = taxonomy.all().collect();
            serde_json::to_string_pretty(&patterns)?
        }
        CudaTdgOutputFormat::Markdown => {
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
        _ => {
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
    };

    write_output(&output, config)?;

    Ok(())
}

// --- Formatting helpers ---

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
        _ => {
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

            Ok(output)
        }
    }
}

fn format_barrier_safety(
    result: &CudaSimdTdgResult,
    config: &CudaTdgCommandConfig,
) -> Result<String> {
    match config.format {
        CudaTdgOutputFormat::Json => Ok(serde_json::to_string_pretty(&result.barrier_safety)?),
        _ => {
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
            Ok(output)
        }
    }
}

fn format_terminal_output(result: &CudaSimdTdgResult) -> Result<String> {
    let mut output = String::new();
    output.push_str("CUDA-SIMD TDG Analysis\n");
    output.push_str("======================\n\n");
    output.push_str(&format!("Path: {}\n", result.path.display()));
    output.push_str(&format!(
        "Files: {} total ({} CUDA, {} SIMD, {} WGPU)\n\n",
        result.files_analyzed, result.cuda_files, result.simd_files, result.wgpu_files
    ));

    // Score summary
    let grade_color = match result.score.grade {
        CudaTdgGrade::APLus | CudaTdgGrade::A => "\x1b[32m", // Green
        CudaTdgGrade::B => "\x1b[33m",                       // Yellow
        CudaTdgGrade::C | CudaTdgGrade::D => "\x1b[31m",     // Red
        CudaTdgGrade::F | CudaTdgGrade::GatewayFail => "\x1b[91m", // Bright red
    };
    output.push_str(&format!(
        "Score: {}{:.1}/100{} (Grade: {}{}{})\n",
        grade_color, result.score.total, "\x1b[0m", grade_color, result.score.grade, "\x1b[0m"
    ));
    output.push_str(&format!(
        "Gateway: {}\n\n",
        if result.score.gateway_passed {
            "\x1b[32mPASSED\x1b[0m"
        } else {
            "\x1b[91mFAILED\x1b[0m"
        }
    ));

    // Defects
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

    output.push_str(&format!("Defects: {} total\n", result.defects.len()));
    if p0_count > 0 {
        output.push_str(&format!("  \x1b[91mP0 Critical: {}\x1b[0m\n", p0_count));
    }
    if p1_count > 0 {
        output.push_str(&format!("  \x1b[33mP1 Performance: {}\x1b[0m\n", p1_count));
    }

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

fn format_markdown_report(result: &CudaSimdTdgResult) -> Result<String> {
    let mut md = String::new();
    md.push_str("# CUDA-SIMD TDG Analysis Report\n\n");
    md.push_str(&format!("**Path**: `{}`\n", result.path.display()));
    md.push_str(&format!("**Timestamp**: {}\n\n", result.timestamp));

    md.push_str("## Summary\n\n");
    md.push_str(&format!("| Metric | Value |\n|--------|-------|\n"));
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
    // Basic HTML report
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
    // SARIF 2.1.0 format for IDE integration
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
    if let Some(ref path) = config.output {
        fs::write(path, output)?;
    } else {
        println!("{}", output);
    }
    Ok(())
}

// Tests extracted to cuda_tdg_handlers_tests.rs for file health compliance (CB-040)
#[cfg(test)]
#[path = "cuda_tdg_handlers_tests.rs"]
mod tests;
