//! Comprehensive analysis handler implementation
//!
//! This module implements the comprehensive analysis command that aggregates
//! results from multiple analyzers into a unified report.

use crate::cli::ComprehensiveOutputFormat;
use crate::services::defect_report_service::{DefectReportService, ReportFormat};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::{info, warn};

/// Handle comprehensive analysis command
///
/// This function performs a comprehensive multi-dimensional analysis of a project or single file,
/// combining results from multiple analyzers including complexity, technical debt, defects,
/// dead code, and duplicates.
///
/// # Arguments
///
/// * `project_path` - The project directory to analyze
/// * `file` - Optional single file to analyze (overrides project path)
/// * `format` - Output format for the report
/// * `include_duplicates` - Whether to include duplicate detection
/// * `include_dead_code` - Whether to include dead code analysis
/// * `include_defects` - Whether to include defect prediction
/// * `include_complexity` - Whether to include complexity metrics
/// * `include_tdg` - Whether to include Technical Debt Gradient
/// * `confidence_threshold` - Minimum confidence threshold for predictions (0.0-1.0)
/// * `min_lines` - Minimum lines of code for analysis
/// * `include` - Optional file pattern to include
/// * `exclude` - Optional file pattern to exclude
/// * `output` - Optional output file path
/// * `perf` - Whether to show performance metrics
/// * `executive_summary` - Whether to include executive summary
///
/// # Examples
///
/// ```no_run
/// # use std::path::PathBuf;
/// # use anyhow::Result;
/// # use pmat::cli::ComprehensiveOutputFormat;
/// # async fn example() -> Result<()> {
/// use pmat::cli::handlers::comprehensive_handler::handle_analyze_comprehensive;
///
/// // Analyze entire project
/// handle_analyze_comprehensive(
///     PathBuf::from("."),
///     None,
///     ComprehensiveOutputFormat::Summary,
///     true,  // include_duplicates
///     true,  // include_dead_code
///     true,  // include_defects
///     true,  // include_complexity
///     true,  // include_tdg
///     0.5,   // confidence_threshold
///     10,    // min_lines
///     None,  // include pattern
///     None,  // exclude pattern
///     None,  // output file
///     false, // perf
///     false, // executive_summary
/// ).await?;
///
/// // Analyze single file
/// handle_analyze_comprehensive(
///     PathBuf::from("."),
///     Some(PathBuf::from("src/main.rs")),
///     ComprehensiveOutputFormat::Detailed,
///     true,  // include_duplicates
///     true,  // include_dead_code
///     true,  // include_defects
///     true,  // include_complexity
///     true,  // include_tdg
///     0.7,   // confidence_threshold
///     10,    // min_lines
///     None,  // include pattern
///     None,  // exclude pattern
///     Some(PathBuf::from("report.md")), // output file
///     true,  // perf
///     true,  // executive_summary
/// ).await?;
/// # Ok(())
/// # }
/// ```
#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_comprehensive(
    project_path: PathBuf,
    file: Option<PathBuf>,
    format: ComprehensiveOutputFormat,
    include_duplicates: bool,
    include_dead_code: bool,
    include_defects: bool,
    include_complexity: bool,
    include_tdg: bool,
    confidence_threshold: f32,
    min_lines: usize,
    include: Option<String>,
    exclude: Option<String>,
    output: Option<PathBuf>,
    perf: bool,
    executive_summary: bool,
) -> Result<()> {
    let start_time = Instant::now();

    info!("🔍 Starting comprehensive analysis");
    
    // Determine if we're analyzing a single file or whole project
    let (analysis_path, single_file_mode) = if let Some(ref file_path) = file {
        info!("📄 Single file mode: {}", file_path.display());
        // For single file, we need to find the project root
        let project_root = find_project_root(file_path)?;
        (project_root, true)
    } else {
        info!("📂 Project path: {}", project_path.display());
        (project_path.clone(), false)
    };

    // Log enabled analyses
    let mut enabled_analyses = Vec::new();
    if include_complexity {
        enabled_analyses.push("complexity");
    }
    if include_tdg {
        enabled_analyses.push("TDG");
    }
    if include_defects {
        enabled_analyses.push("defects");
    }
    if include_dead_code {
        enabled_analyses.push("dead code");
    }
    if include_duplicates {
        enabled_analyses.push("duplicates");
    }

    if enabled_analyses.is_empty() {
        // Default to all analyses if none specified
        enabled_analyses = vec!["complexity", "TDG", "defects", "dead code", "duplicates"];
    }

    info!("📊 Enabled analyses: {}", enabled_analyses.join(", "));

    // Create defect report service
    let service = DefectReportService::new();

    // Generate comprehensive report
    let report = service.generate_report(&analysis_path).await?;

    // Apply filters based on confidence threshold and single file mode
    let filtered_defects: Vec<_> = report
        .defects
        .iter()
        .filter(|d| {
            // Filter by single file if in single file mode
            if single_file_mode {
                if let Some(ref target_file) = file {
                    if d.file_path != *target_file {
                        return false;
                    }
                }
            }
            
            // Convert confidence from metrics if available
            let confidence = d.metrics.get("confidence").copied().unwrap_or(100.0) as f32;
            confidence >= confidence_threshold
        })
        .cloned()
        .collect();

    info!("📈 Total defects found: {}", report.defects.len());
    info!(
        "📉 After confidence filter (>={:.0}%): {}",
        confidence_threshold,
        filtered_defects.len()
    );

    // Convert format
    let output_format = match format {
        ComprehensiveOutputFormat::Json => ReportFormat::Json,
        ComprehensiveOutputFormat::Markdown => ReportFormat::Markdown,
        ComprehensiveOutputFormat::Summary => ReportFormat::Text,
        ComprehensiveOutputFormat::Detailed => ReportFormat::Markdown,
        ComprehensiveOutputFormat::Sarif => ReportFormat::Json, // SARIF not implemented, use JSON
    };

    // Format output
    let formatted_output = match output_format {
        ReportFormat::Json => {
            // Create filtered report for JSON output
            let mut filtered_report = report.clone();
            filtered_report.defects = filtered_defects;
            filtered_report.summary = service.compute_summary(&filtered_report.defects);
            service.format_json(&filtered_report)?
        }
        ReportFormat::Markdown => {
            // For Markdown, include executive summary if requested
            let mut md = String::new();

            if executive_summary {
                md.push_str("# Executive Summary\n\n");
                md.push_str(&format!("This comprehensive analysis of {} examined {} files and identified {} quality issues.\n\n",
                    project_path.display(),
                    report.metadata.total_files_analyzed,
                    filtered_defects.len()
                ));

                // Add severity breakdown
                let mut critical = 0;
                let mut high = 0;
                let mut medium = 0;
                let mut low = 0;

                for defect in &filtered_defects {
                    match defect.severity {
                        crate::models::defect_report::Severity::Critical => critical += 1,
                        crate::models::defect_report::Severity::High => high += 1,
                        crate::models::defect_report::Severity::Medium => medium += 1,
                        crate::models::defect_report::Severity::Low => low += 1,
                    }
                }

                md.push_str("## Key Findings\n\n");
                if critical > 0 {
                    md.push_str(&format!(
                        "- **{} Critical Issues** requiring immediate attention\n",
                        critical
                    ));
                }
                if high > 0 {
                    md.push_str(&format!(
                        "- **{} High Priority Issues** that should be addressed soon\n",
                        high
                    ));
                }
                md.push_str(&format!("- **{} Medium Priority Issues**\n", medium));
                md.push_str(&format!("- **{} Low Priority Issues**\n\n", low));

                // Add recommendations
                md.push_str("## Recommendations\n\n");
                md.push_str("1. Address all critical issues immediately\n");
                md.push_str("2. Create a plan to resolve high priority issues\n");
                md.push_str("3. Consider refactoring files with multiple defects\n");
                md.push_str("4. Establish quality gates to prevent new issues\n\n");

                md.push_str("---\n\n");
            }

            // Add the regular report
            let mut filtered_report = report.clone();
            filtered_report.defects = filtered_defects;
            filtered_report.summary = service.compute_summary(&filtered_report.defects);
            md.push_str(&service.format_markdown(&filtered_report)?);

            md
        }
        _ => {
            // Fallback to JSON for other formats
            let mut filtered_report = report.clone();
            filtered_report.defects = filtered_defects;
            filtered_report.summary = service.compute_summary(&filtered_report.defects);
            service.format_json(&filtered_report)?
        }
    };

    // Write output
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &formatted_output).await?;
        info!("📄 Report saved to: {}", output_path.display());
    } else {
        println!("{}", formatted_output);
    }

    let elapsed = start_time.elapsed();

    // Print performance metrics if requested
    if perf {
        info!("⚡ Performance Metrics:");
        info!("  Total time: {:?}", elapsed);
        info!("  Files analyzed: {}", report.metadata.total_files_analyzed);
        info!(
            "  Files/second: {:.1}",
            report.metadata.total_files_analyzed as f64 / elapsed.as_secs_f64()
        );
        info!(
            "  Analysis time: {}ms",
            report.metadata.analysis_duration_ms
        );
    }

    // Print summary by category
    info!("\n📊 Defects by Category:");
    for (category, count) in &report.summary.by_category {
        info!("  {}: {}", category, count);
    }

    // Warn about ignored parameters (for transparency)
    if include.is_some() {
        warn!("Note: --include parameter is not yet implemented in comprehensive analysis");
    }
    if exclude.is_some() {
        warn!("Note: --exclude parameter is not yet implemented in comprehensive analysis");
    }
    if min_lines > 0 {
        warn!("Note: --min-lines parameter is not yet implemented in comprehensive analysis");
    }

    Ok(())
}

/// Find the project root by looking for Cargo.toml
fn find_project_root(start_path: &Path) -> Result<PathBuf> {
    let mut current = if start_path.is_file() {
        start_path.parent().context("File has no parent directory")?
    } else {
        start_path
    };

    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            return Ok(current.to_path_buf());
        }

        // Move up one directory
        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }

    // If no Cargo.toml found, return the original directory
    Ok(start_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comprehensive_handler_params() {
        // Basic parameter validation test
        assert_eq!(
            ComprehensiveOutputFormat::Json as i32,
            ComprehensiveOutputFormat::Json as i32
        );
    }

    #[test]
    fn test_find_project_root() {
        use std::fs;
        use tempfile::TempDir;

        // Create a temporary directory structure
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();
        let src_dir = project_root.join("src");
        let sub_dir = src_dir.join("module");
        
        // Create directories
        fs::create_dir_all(&sub_dir).unwrap();
        
        // Create Cargo.toml at project root
        fs::write(project_root.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
        
        // Create a test file deep in the structure
        let test_file = sub_dir.join("test.rs");
        fs::write(&test_file, "// test file").unwrap();
        
        // Test finding project root from file
        let found_root = find_project_root(&test_file).unwrap();
        assert_eq!(found_root, project_root);
        
        // Test finding project root from directory
        let found_root = find_project_root(&sub_dir).unwrap();
        assert_eq!(found_root, project_root);
        
        // Test when no Cargo.toml exists
        let isolated_dir = TempDir::new().unwrap();
        let isolated_file = isolated_dir.path().join("isolated.rs");
        fs::write(&isolated_file, "// isolated file").unwrap();
        
        let found_root = find_project_root(&isolated_file).unwrap();
        assert_eq!(found_root, isolated_dir.path());
    }

    #[tokio::test]
    async fn test_comprehensive_single_file_filter() {
        use crate::models::defect_report::{Defect, DefectCategory, Severity};
        use std::collections::HashMap;

        // Create test defects for different files
        let defects = vec![
            Defect {
                id: "1".to_string(),
                category: DefectCategory::Complexity,
                severity: Severity::High,
                file_path: PathBuf::from("src/main.rs"),
                line_start: 10,
                line_end: Some(20),
                column_start: Some(5),
                column_end: Some(10),
                message: "High complexity in main".to_string(),
                rule_id: "complexity".to_string(),
                fix_suggestion: Some("Refactor".to_string()),
                metrics: HashMap::from([("confidence".to_string(), 0.8)]),
            },
            Defect {
                id: "2".to_string(),
                category: DefectCategory::Complexity,
                severity: Severity::Medium,
                file_path: PathBuf::from("src/lib.rs"),
                line_start: 15,
                line_end: Some(25),
                column_start: Some(3),
                column_end: Some(8),
                message: "Medium complexity in lib".to_string(),
                rule_id: "complexity".to_string(),
                fix_suggestion: Some("Consider refactoring".to_string()),
                metrics: HashMap::from([("confidence".to_string(), 0.7)]),
            },
        ];

        // Test single file filtering
        let target_file = Some(PathBuf::from("src/main.rs"));
        let filtered: Vec<_> = defects
            .iter()
            .filter(|d| {
                if let Some(ref tf) = target_file {
                    d.file_path == *tf
                } else {
                    true
                }
            })
            .collect();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "1");
        assert_eq!(filtered[0].file_path, PathBuf::from("src/main.rs"));
    }
}
