//! Technical Debt Gradient (TDG) analysis handler
//!
//! Provides full-featured TDG analysis with support for single file,
//! multiple files (MCP composition), and project-wide analysis.

use crate::cli::enums::TdgOutputFormat;
use crate::models::tdg::{TDGScore, TDGSummary, TDGHotspot, TDGSeverity};
use crate::services::tdg_calculator::TDGCalculator;
use anyhow::Result;
use std::path::PathBuf;

/// Handle TDG analysis command with full MCP tool composition support
///
/// This handler supports three distinct modes:
/// 1. **Single File Mode**: Deep TDG analysis of one specific file
/// 2. **Multi-File Mode**: Process specific file lists for MCP tool chaining
/// 3. **Project Mode**: Analyze entire project with pattern filtering
///
/// # MCP Tool Composition Examples
///
/// ```no_run
/// # async fn example() -> anyhow::Result<()> {
/// use std::path::PathBuf;
/// use pmat::cli::enums::TdgOutputFormat;
/// use pmat::cli::handlers::tdg_handler::handle_analyze_tdg;
///
/// // Example 1: Find high TDG files for targeted refactoring
/// handle_analyze_tdg(
///     PathBuf::from("."),
///     None,                           // file
///     vec![],                         // files (empty = project mode)
///     1.5,                            // threshold
///     5,                              // top_files
///     TdgOutputFormat::Json,          // format for parsing
///     false,                          // include_components
///     None,                           // output (stdout)
///     false,                          // critical_only
///     false,                          // verbose
///     vec![],                         // include patterns
///     false,                          // watch
/// ).await?;
///
/// // Example 2: Analyze specific hotspot files from another tool
/// let hotspot_files = vec![
///     PathBuf::from("src/complex_module.rs"),
///     PathBuf::from("src/legacy_code.rs"),
/// ];
///
/// handle_analyze_tdg(
///     PathBuf::from("."),
///     None,                           // file
///     hotspot_files,                  // files (MCP composition)
///     1.0,                            // lower threshold
///     0,                              // show all
///     TdgOutputFormat::Json,          // format
///     true,                           // include_components
///     None,                           // output
///     false,                          // critical_only
///     true,                           // verbose
///     vec![],                         // include patterns
///     false,                          // watch
/// ).await?;
/// # Ok(())
/// # }
/// ```
#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_tdg(
    project_path: PathBuf,
    file: Option<PathBuf>,
    files: Vec<PathBuf>,
    threshold: f64,
    top_files: usize,
    format: TdgOutputFormat,
    include_components: bool,
    output: Option<PathBuf>,
    critical_only: bool,
    verbose: bool,
    include: Vec<String>,
    watch: bool,
) -> Result<()> {
    if watch {
        eprintln!("❌ Watch mode not yet implemented for TDG analysis");
        return Ok(());
    }

    eprintln!("🔍 Analyzing Technical Debt Gradient...");

    // Create TDG calculator
    let calculator = TDGCalculator::new();

    // Determine analysis mode and generate output
    let output_content = if let Some(single_file) = file {
        // Single file mode
        analyze_single_file(
            &calculator,
            &project_path,
            single_file,
            threshold,
            format,
            include_components,
            critical_only,
            verbose,
        ).await?
    } else if !files.is_empty() {
        // Multiple files mode (MCP tool composition)
        analyze_multiple_files(
            &calculator,
            &project_path,
            files,
            threshold,
            top_files,
            format,
            include_components,
            critical_only,
            verbose,
        ).await?
    } else {
        // Project mode
        analyze_project(
            &calculator,
            &project_path,
            include,
            threshold,
            top_files,
            format,
            include_components,
            critical_only,
            verbose,
        ).await?
    };

    // Output results
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &output_content).await?;
        eprintln!("📝 Results written to {}", output_path.display());
    } else {
        println!("{}", output_content);
    }

    eprintln!("✅ TDG analysis complete");
    Ok(())
}

/// Analyze a single file and return formatted output
async fn analyze_single_file(
    calculator: &TDGCalculator,
    project_path: &PathBuf,
    file: PathBuf,
    threshold: f64,
    format: TdgOutputFormat,
    include_components: bool,
    critical_only: bool,
    verbose: bool,
) -> Result<String> {
    eprintln!("📄 Analyzing TDG for file: {}", file.display());

    // Resolve path
    let full_path = if file.is_absolute() {
        file
    } else {
        project_path.join(&file)
    };

    if !full_path.exists() {
        anyhow::bail!("File not found: {}", full_path.display());
    }

    // Analyze file
    let score = calculator.calculate_file(&full_path).await?;

    // Check if it meets criteria
    if critical_only && score.value <= 2.5 {
        return Ok(format_empty_results(format));
    }
    if score.value < threshold {
        return Ok(format_empty_results(format));
    }

    // Format single file results
    format_single_file_output(&score, &full_path, format, include_components, verbose)
}

/// Analyze multiple files and return formatted output
async fn analyze_multiple_files(
    calculator: &TDGCalculator,
    project_path: &PathBuf,
    files: Vec<PathBuf>,
    threshold: f64,
    top_files: usize,
    format: TdgOutputFormat,
    include_components: bool,
    critical_only: bool,
    verbose: bool,
) -> Result<String> {
    eprintln!("📄 Analyzing TDG for {} files...", files.len());

    let mut results = Vec::new();

    for file_path in files {
        let full_path = if file_path.is_absolute() {
            file_path
        } else {
            project_path.join(&file_path)
        };

        if !full_path.exists() {
            eprintln!("⚠️  Skipping missing file: {}", full_path.display());
            continue;
        }

        match calculator.calculate_file(&full_path).await {
            Ok(score) => {
                // Apply filters
                if critical_only && score.value <= 2.5 {
                    continue;
                }
                if score.value < threshold {
                    continue;
                }
                results.push((score, full_path));
            }
            Err(e) => {
                eprintln!("⚠️  Error analyzing {}: {}", full_path.display(), e);
            }
        }
    }

    // Sort by TDG score descending
    results.sort_by(|a, b| b.0.value.partial_cmp(&a.0.value).unwrap());

    // Apply top_files limit
    if top_files > 0 && results.len() > top_files {
        results.truncate(top_files);
    }

    // Create synthetic summary
    let summary = create_summary_from_file_results(&results);

    // Format output
    format_output_from_summary(&summary, format, include_components, verbose)
}

/// Analyze entire project and return formatted output
async fn analyze_project(
    calculator: &TDGCalculator,
    project_path: &PathBuf,
    _include: Vec<String>,
    threshold: f64,
    top_files: usize,
    format: TdgOutputFormat,
    include_components: bool,
    critical_only: bool,
    verbose: bool,
) -> Result<String> {
    eprintln!("📁 Project path: {}", project_path.display());

    // Analyze directory
    let mut summary = calculator.analyze_directory(project_path).await?;

    // Filter hotspots based on criteria
    summary.hotspots = summary.hotspots
        .into_iter()
        .filter(|h| {
            if critical_only {
                h.tdg_score > 2.5
            } else {
                h.tdg_score >= threshold
            }
        })
        .take(if top_files > 0 { top_files } else { usize::MAX })
        .collect();

    // Format output
    format_output_from_summary(&summary, format, include_components, verbose)
}

/// Create a summary from individual file results
fn create_summary_from_file_results(results: &[(TDGScore, PathBuf)]) -> TDGSummary {
    let total_files = results.len();
    let critical_files = results.iter().filter(|(s, _)| matches!(s.severity, TDGSeverity::Critical)).count();
    let warning_files = results.iter().filter(|(s, _)| matches!(s.severity, TDGSeverity::Warning)).count();
    
    let tdg_values: Vec<f64> = results.iter().map(|(s, _)| s.value).collect();
    let average_tdg = if tdg_values.is_empty() {
        0.0
    } else {
        tdg_values.iter().sum::<f64>() / tdg_values.len() as f64
    };

    // Calculate percentiles
    let mut sorted_values = tdg_values.clone();
    sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    let p95_tdg = percentile(&sorted_values, 0.95);
    let p99_tdg = percentile(&sorted_values, 0.99);

    // Create hotspots
    let hotspots = results
        .iter()
        .map(|(score, path)| TDGHotspot {
            path: path.display().to_string(),
            tdg_score: score.value,
            primary_factor: identify_primary_factor(&score.components),
            estimated_hours: estimate_refactoring_hours(score.value),
        })
        .collect();

    let estimated_debt_hours = results
        .iter()
        .map(|(s, _)| estimate_refactoring_hours(s.value))
        .sum();

    TDGSummary {
        total_files,
        critical_files,
        warning_files,
        average_tdg,
        p95_tdg,
        p99_tdg,
        estimated_debt_hours,
        hotspots,
    }
}

/// Format output from a TDG summary
fn format_output_from_summary(
    summary: &TDGSummary,
    format: TdgOutputFormat,
    include_components: bool,
    verbose: bool,
) -> Result<String> {
    match format {
        TdgOutputFormat::Table => Ok(format_table_output(summary, include_components, verbose)),
        TdgOutputFormat::Json => Ok(format_json_output(summary, include_components)),
        TdgOutputFormat::Markdown => Ok(format_markdown_output(summary, include_components)),
        TdgOutputFormat::Sarif => Ok(format_sarif_output(summary)),
    }
}

/// Format single file output
fn format_single_file_output(
    score: &TDGScore,
    path: &PathBuf,
    format: TdgOutputFormat,
    include_components: bool,
    verbose: bool,
) -> Result<String> {
    // Create a single-file summary
    let hotspot = TDGHotspot {
        path: path.display().to_string(),
        tdg_score: score.value,
        primary_factor: identify_primary_factor(&score.components),
        estimated_hours: estimate_refactoring_hours(score.value),
    };

    let summary = TDGSummary {
        total_files: 1,
        critical_files: if matches!(score.severity, TDGSeverity::Critical) { 1 } else { 0 },
        warning_files: if matches!(score.severity, TDGSeverity::Warning) { 1 } else { 0 },
        average_tdg: score.value,
        p95_tdg: score.value,
        p99_tdg: score.value,
        estimated_debt_hours: estimate_refactoring_hours(score.value),
        hotspots: vec![hotspot],
    };

    format_output_from_summary(&summary, format, include_components, verbose)
}

/// Format empty results when no files meet criteria
fn format_empty_results(format: TdgOutputFormat) -> String {
    match format {
        TdgOutputFormat::Table => "No files found matching the specified criteria.\n".to_string(),
        TdgOutputFormat::Json => r#"{"summary": {"total_files": 0}, "hotspots": []}"#.to_string(),
        TdgOutputFormat::Markdown => "# Technical Debt Gradient Analysis\n\nNo files found matching the specified criteria.\n".to_string(),
        TdgOutputFormat::Sarif => r#"{"version": "2.1.0", "runs": [{"tool": {"driver": {"name": "pmat-tdg"}}, "results": []}]}"#.to_string(),
    }
}

// Format implementations...

fn format_table_output(summary: &TDGSummary, include_components: bool, verbose: bool) -> String {
    let mut table = String::new();
    table.push_str("\n# Technical Debt Gradient Analysis\n\n");
    table.push_str(&format!("📊 **Total Files Analyzed**: {}\n", summary.total_files));
    
    if summary.total_files > 0 {
        table.push_str(&format!("🔴 **Critical Files**: {} ({:.1}%)\n", 
            summary.critical_files, 
            (summary.critical_files as f64 / summary.total_files as f64) * 100.0
        ));
        table.push_str(&format!("🟡 **Warning Files**: {} ({:.1}%)\n", 
            summary.warning_files,
            (summary.warning_files as f64 / summary.total_files as f64) * 100.0
        ));
    }
    
    table.push_str(&format!("📈 **Average TDG**: {:.2}\n", summary.average_tdg));
    table.push_str(&format!("📊 **95th Percentile**: {:.2}\n", summary.p95_tdg));
    table.push_str(&format!("📊 **99th Percentile**: {:.2}\n", summary.p99_tdg));
    table.push_str(&format!("⏱️  **Estimated Debt**: {:.1} hours\n\n", summary.estimated_debt_hours));
    
    if !summary.hotspots.is_empty() {
        table.push_str("## Top Hotspots\n\n");
        table.push_str("| File | TDG Score | Primary Factor | Est. Hours |\n");
        table.push_str("|------|-----------|----------------|------------|\n");
        
        for hotspot in &summary.hotspots {
            table.push_str(&format!(
                "| {} | {:.2} | {} | {:.1} |\n",
                hotspot.path,
                hotspot.tdg_score,
                hotspot.primary_factor,
                hotspot.estimated_hours
            ));
        }
    }
    
    if include_components && verbose {
        table.push_str("\n## Component Weights\n\n");
        table.push_str("| Component | Weight |\n");
        table.push_str("|-----------|--------|\n");
        table.push_str("| Complexity | 30% |\n");
        table.push_str("| Code Churn | 35% |\n");
        table.push_str("| Coupling | 15% |\n");
        table.push_str("| Domain Risk | 10% |\n");
        table.push_str("| Duplication | 10% |\n");
    }
    
    table
}

fn format_json_output(summary: &TDGSummary, include_components: bool) -> String {
    let json_output = serde_json::json!({
        "summary": {
            "total_files": summary.total_files,
            "critical_files": summary.critical_files,
            "warning_files": summary.warning_files,
            "average_tdg": summary.average_tdg,
            "p95_tdg": summary.p95_tdg,
            "p99_tdg": summary.p99_tdg,
            "estimated_debt_hours": summary.estimated_debt_hours,
        },
        "hotspots": summary.hotspots,
        "components": if include_components {
            Some(serde_json::json!({
                "complexity_weight": 0.30,
                "churn_weight": 0.35,
                "coupling_weight": 0.15,
                "domain_risk_weight": 0.10,
                "duplication_weight": 0.10,
            }))
        } else {
            None
        }
    });
    
    serde_json::to_string_pretty(&json_output).unwrap_or_else(|_| "{}".to_string())
}

fn format_markdown_output(summary: &TDGSummary, include_components: bool) -> String {
    let mut md = String::new();
    md.push_str("# Technical Debt Gradient Analysis\n\n");
    md.push_str("## Summary\n\n");
    md.push_str(&format!("- **Total Files**: {}\n", summary.total_files));
    
    if summary.total_files > 0 {
        md.push_str(&format!("- **Critical Files**: {} ({:.1}%)\n", 
            summary.critical_files, 
            (summary.critical_files as f64 / summary.total_files as f64) * 100.0
        ));
        md.push_str(&format!("- **Warning Files**: {} ({:.1}%)\n", 
            summary.warning_files,
            (summary.warning_files as f64 / summary.total_files as f64) * 100.0
        ));
    }
    
    md.push_str(&format!("- **Average TDG**: {:.2}\n", summary.average_tdg));
    md.push_str(&format!("- **95th Percentile**: {:.2}\n", summary.p95_tdg));
    md.push_str(&format!("- **99th Percentile**: {:.2}\n", summary.p99_tdg));
    md.push_str(&format!("- **Estimated Technical Debt**: {:.1} hours\n\n", summary.estimated_debt_hours));
    
    if !summary.hotspots.is_empty() {
        md.push_str("## Hotspots\n\n");
        for (i, hotspot) in summary.hotspots.iter().enumerate() {
            md.push_str(&format!("### {}. {}\n\n", i + 1, hotspot.path));
            md.push_str(&format!("- **TDG Score**: {:.2}\n", hotspot.tdg_score));
            md.push_str(&format!("- **Primary Factor**: {}\n", hotspot.primary_factor));
            md.push_str(&format!("- **Estimated Refactoring Time**: {:.1} hours\n\n", hotspot.estimated_hours));
        }
    }
    
    if include_components {
        md.push_str("## TDG Components\n\n");
        md.push_str("The Technical Debt Gradient is calculated using the following weighted components:\n\n");
        md.push_str("- **Complexity** (30%): Cyclomatic and cognitive complexity\n");
        md.push_str("- **Code Churn** (35%): Frequency of changes over time\n");
        md.push_str("- **Coupling** (15%): Dependencies between modules\n");
        md.push_str("- **Domain Risk** (10%): Critical domain areas (auth, crypto, etc.)\n");
        md.push_str("- **Duplication** (10%): Code duplication percentage\n");
    }
    
    md
}

fn format_sarif_output(summary: &TDGSummary) -> String {
    let sarif = serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat-tdg",
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                    "version": env!("CARGO_PKG_VERSION"),
                    "rules": [{
                        "id": "TDG001",
                        "name": "HighTechnicalDebtGradient",
                        "shortDescription": {
                            "text": "File has high technical debt gradient"
                        },
                        "fullDescription": {
                            "text": "Technical Debt Gradient exceeds threshold, indicating accumulated technical debt"
                        },
                        "help": {
                            "text": "Consider refactoring to reduce complexity, stabilize churn, or reduce coupling"
                        }
                    }]
                }
            },
            "results": summary.hotspots.iter().map(|hotspot| {
                serde_json::json!({
                    "ruleId": "TDG001",
                    "level": if hotspot.tdg_score > 2.5 { "error" } else { "warning" },
                    "message": {
                        "text": format!("TDG score {:.2} - Primary factor: {}", 
                            hotspot.tdg_score, hotspot.primary_factor)
                    },
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": {
                                "uri": hotspot.path.clone()
                            }
                        }
                    }],
                    "properties": {
                        "tdg_score": hotspot.tdg_score,
                        "primary_factor": &hotspot.primary_factor,
                        "estimated_hours": hotspot.estimated_hours
                    }
                })
            }).collect::<Vec<_>>()
        }]
    });
    
    serde_json::to_string_pretty(&sarif).unwrap_or_else(|_| "{}".to_string())
}

// Helper functions

fn percentile(sorted_values: &[f64], p: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }
    
    let index = (sorted_values.len() as f64 * p) as usize;
    let index = index.min(sorted_values.len() - 1);
    sorted_values[index]
}

fn identify_primary_factor(components: &crate::models::tdg::TDGComponents) -> String {
    let mut factors = [
        (components.complexity * 0.30, "High Complexity"),
        (components.churn * 0.35, "Frequent Changes"),
        (components.coupling * 0.15, "High Coupling"),
        (components.domain_risk * 0.10, "Domain Risk"),
        (components.duplication * 0.10, "Code Duplication"),
    ];
    
    factors.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    factors[0].1.to_string()
}

fn estimate_refactoring_hours(tdg_score: f64) -> f64 {
    // Empirical formula: hours = base * multiplier^tdg
    let base_hours = 2.0;
    let multiplier = 1.8;
    base_hours * multiplier.powf(tdg_score)
}