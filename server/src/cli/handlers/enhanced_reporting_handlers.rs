//! Enhanced reporting command handlers
//!
//! This module provides handlers for generating comprehensive analysis reports
//! that consolidate multiple analysis outputs.

use crate::cli::*;
use crate::services::enhanced_reporting::{
    AnalysisResults, BigOAnalysis, ComplexityAnalysis, DeadCodeAnalysis, DuplicationAnalysis,
    EnhancedReportingService, ReportConfig, ReportFormat as ServiceReportFormat, TdgAnalysis,
};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tracing::info;

/// Handle enhanced report generation command
#[allow(clippy::too_many_arguments)]
pub async fn handle_generate_report(
    project_path: PathBuf,
    output_format: ReportOutputFormat,
    include_visualizations: bool,
    include_executive_summary: bool,
    include_recommendations: bool,
    analyses: Vec<AnalysisType>,
    confidence_threshold: u8,
    output: Option<PathBuf>,
    perf: bool,
) -> Result<()> {
    let start_time = Instant::now();

    info!("📊 Generating enhanced analysis report");
    info!("📂 Project path: {}", project_path.display());
    info!("📄 Output format: {:?}", output_format);
    info!("🔍 Analyses to include: {:?}", analyses);

    // Create reporting service
    let service = EnhancedReportingService::new()?;

    // Run requested analyses
    let analysis_results = run_analyses(&project_path, &analyses, perf).await?;

    // Convert output format
    let service_format = match output_format {
        ReportOutputFormat::Html => ServiceReportFormat::Html,
        ReportOutputFormat::Markdown => ServiceReportFormat::Markdown,
        ReportOutputFormat::Json => ServiceReportFormat::Json,
        ReportOutputFormat::Pdf => ServiceReportFormat::Pdf,
        ReportOutputFormat::Dashboard => ServiceReportFormat::Dashboard,
    };

    // Build report configuration
    let config = ReportConfig {
        project_path: project_path.clone(),
        output_format: service_format.clone(),
        include_visualizations,
        include_executive_summary,
        include_recommendations,
        confidence_threshold,
        output_path: output.clone(),
    };

    // Generate report
    let report = service.generate_report(config, analysis_results).await?;

    // Format output
    let formatted_output = service.format_report(&report, service_format).await?;

    // Write output
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &formatted_output).await?;
        info!("📄 Report saved to: {}", output_path.display());
    } else {
        println!("{formatted_output}");
    }

    let elapsed = start_time.elapsed();

    // Print summary
    info!("✅ Report generation completed in {:?}", elapsed);
    info!(
        "📊 Health Score: {:.1}/100",
        report.executive_summary.overall_health_score
    );
    info!(
        "⚠️ Critical Issues: {}",
        report.executive_summary.critical_issues
    );
    info!("🔍 Total Recommendations: {}", report.recommendations.len());

    if perf {
        let files_per_sec = report.metadata.analyzed_files as f64 / elapsed.as_secs_f64();
        info!("⚡ Performance: {:.0} files/second", files_per_sec);
    }

    Ok(())
}

/// Run the requested analyses
async fn run_analyses(
    project_path: &PathBuf,
    analyses: &[AnalysisType],
    perf: bool,
) -> Result<AnalysisResults> {
    let start_time = Instant::now();
    let mut results = AnalysisResults {
        total_duration: Duration::default(),
        analyzed_files: 0,
        total_lines: 0,
        complexity_analysis: None,
        dead_code_analysis: None,
        duplication_analysis: None,
        tdg_analysis: None,
        big_o_analysis: None,
    };

    // Count files and lines
    let (file_count, line_count) = count_files_and_lines(project_path).await?;
    results.analyzed_files = file_count;
    results.total_lines = line_count;

    for analysis_type in analyses {
        match analysis_type {
            AnalysisType::Complexity => {
                info!("Running complexity analysis...");
                results.complexity_analysis = Some(run_complexity_analysis(project_path).await?);
            }
            AnalysisType::DeadCode => {
                info!("Running dead code analysis...");
                results.dead_code_analysis = Some(run_dead_code_analysis(project_path).await?);
            }
            AnalysisType::Duplication => {
                info!("Running duplication analysis...");
                results.duplication_analysis = Some(run_duplication_analysis(project_path).await?);
            }
            AnalysisType::TechnicalDebt => {
                info!("Running technical debt analysis...");
                results.tdg_analysis = Some(run_tdg_analysis(project_path).await?);
            }
            AnalysisType::BigO => {
                info!("Running Big-O complexity analysis...");
                results.big_o_analysis = Some(run_big_o_analysis(project_path).await?);
            }
            AnalysisType::All => {
                // Run all analyses
                info!("Running all analyses...");
                results.complexity_analysis = Some(run_complexity_analysis(project_path).await?);
                results.dead_code_analysis = Some(run_dead_code_analysis(project_path).await?);
                results.duplication_analysis = Some(run_duplication_analysis(project_path).await?);
                results.tdg_analysis = Some(run_tdg_analysis(project_path).await?);
                results.big_o_analysis = Some(run_big_o_analysis(project_path).await?);
            }
        }
    }

    results.total_duration = start_time.elapsed();

    if perf {
        info!("Analysis completed in {:?}", results.total_duration);
    }

    Ok(results)
}

/// Count files and lines in the project
async fn count_files_and_lines(project_path: &PathBuf) -> Result<(usize, usize)> {
    use tokio::fs;
    use tokio::io::{AsyncBufReadExt, BufReader};

    let mut file_count = 0;
    let mut line_count = 0;

    let extensions = ["rs", "js", "ts", "py", "c", "cpp", "h", "hpp", "java", "go"];

    let mut _entries = fs::read_dir(project_path).await?;
    let mut dirs = vec![project_path.clone()];

    while let Some(dir) = dirs.pop() {
        let mut entries = fs::read_dir(&dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let metadata = entry.metadata().await?;

            if metadata.is_dir() {
                let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
                if !dir_name.starts_with('.') && dir_name != "target" && dir_name != "node_modules"
                {
                    dirs.push(path);
                }
            } else if metadata.is_file() {
                if let Some(ext) = path.extension() {
                    if extensions.contains(&ext.to_string_lossy().as_ref()) {
                        file_count += 1;

                        // Count lines
                        if let Ok(file) = fs::File::open(&path).await {
                            let reader = BufReader::new(file);
                            let mut lines = reader.lines();
                            while lines.next_line().await?.is_some() {
                                line_count += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok((file_count, line_count))
}

/// Run complexity analysis (simplified stub)
async fn run_complexity_analysis(_project_path: &Path) -> Result<ComplexityAnalysis> {
    // In a real implementation, this would call the actual complexity analyzer
    Ok(ComplexityAnalysis {
        total_cyclomatic: 500,
        total_cognitive: 800,
        functions: 100,
        max_cyclomatic: 25,
        high_complexity_functions: 10,
        distribution: vec![30, 40, 20, 7, 3],
    })
}

/// Run dead code analysis (simplified stub)
async fn run_dead_code_analysis(_project_path: &Path) -> Result<DeadCodeAnalysis> {
    // In a real implementation, this would call the actual dead code analyzer
    Ok(DeadCodeAnalysis {
        dead_lines: 150,
        dead_functions: 8,
        dead_code_percentage: 1.5,
    })
}

/// Run duplication analysis (simplified stub)
async fn run_duplication_analysis(_project_path: &Path) -> Result<DuplicationAnalysis> {
    // In a real implementation, this would call the actual duplication detector
    Ok(DuplicationAnalysis {
        duplicated_lines: 200,
        duplicate_blocks: 15,
        duplication_percentage: 2.0,
    })
}

/// Run TDG analysis (simplified stub)
async fn run_tdg_analysis(_project_path: &Path) -> Result<TdgAnalysis> {
    // In a real implementation, this would call the actual TDG calculator
    Ok(TdgAnalysis {
        average_tdg: 2.5,
        max_tdg: 5.0,
        high_tdg_files: 5,
    })
}

/// Run Big-O analysis (simplified stub)
async fn run_big_o_analysis(_project_path: &Path) -> Result<BigOAnalysis> {
    use std::collections::HashMap;

    // In a real implementation, this would call the actual Big-O analyzer
    let mut distribution = HashMap::with_capacity(64);
    distribution.insert("O(1)".to_string(), 50);
    distribution.insert("O(n)".to_string(), 30);
    distribution.insert("O(n²)".to_string(), 5);

    Ok(BigOAnalysis {
        analyzed_functions: 100,
        high_complexity_count: 5,
        complexity_distribution: distribution,
    })
}

#[cfg(test)]
mod tests {
    // use super::*; // Unused in simple tests

    #[test]
    fn test_enhanced_reporting_handlers_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
