//! Advanced code similarity and duplication detection handler.
//! Submodules: similarity_handler_formatting.rs, similarity_handler_output.rs

#![cfg_attr(coverage_nightly, coverage(off))]
use anyhow::Result;
use std::path::PathBuf;
use std::time::Instant;

use crate::services::similarity::{
    ComprehensiveReport, EntropyBlock, EntropyReport, Metrics, RefactoringHint, SimilarBlock,
    SimilarityConfig, SimilarityDetector,
};

/// Handle similarity analysis command with entropy detection
#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_analyze_similarity(
    project_path: PathBuf,
    detection_type: crate::cli::DuplicateType,
    threshold: f32,
    min_lines: usize,
    max_tokens: usize,
    format: crate::cli::DuplicateOutputFormat,
    perf: bool,
    include: Option<String>,
    exclude: Option<String>,
    output: Option<PathBuf>,
    top_files: usize,
) -> Result<()> {
    let start = if perf { Some(Instant::now()) } else { None };
    eprintln!("🔍 Advanced similarity analysis starting...");

    let config = build_config(detection_type, threshold, min_lines, max_tokens);
    let detector = SimilarityDetector::new(config);
    let files = collect_files(&project_path, &include, &exclude).await?;
    eprintln!("📊 Analyzing {} files...", files.len());

    let report = detector.comprehensive_analysis(&files);
    let filtered_report = if top_files > 0 {
        filter_top_files(report, top_files)
    } else {
        report
    };

    let output_str = format_report(&filtered_report, format)?;
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &output_str).await?;
        eprintln!("📄 Report written to: {}", output_path.display());
    } else {
        println!("{output_str}");
    }

    if let Some(start_time) = start {
        print_performance_metrics(&filtered_report, start_time.elapsed());
    }
    print_summary(&filtered_report);

    Ok(())
}

fn build_config(
    detection_type: crate::cli::DuplicateType,
    threshold: f32,
    min_lines: usize,
    max_tokens: usize,
) -> SimilarityConfig {
    let mut config = SimilarityConfig {
        similarity_threshold: f64::from(threshold),
        min_lines,
        min_tokens: max_tokens,
        ..Default::default()
    };
    match detection_type {
        crate::cli::DuplicateType::Exact => {
            config.enable_ast = false;
            config.enable_semantic = false;
        }
        crate::cli::DuplicateType::Fuzzy | crate::cli::DuplicateType::Renamed => {
            config.enable_ast = true;
            config.enable_semantic = false;
        }
        crate::cli::DuplicateType::Semantic | crate::cli::DuplicateType::Gapped => {
            config.enable_ast = true;
            config.enable_semantic = true;
        }
        crate::cli::DuplicateType::All => {
            config.enable_ast = true;
            config.enable_semantic = true;
            config.enable_entropy = true;
        }
    }
    config
}

async fn collect_files(
    project_path: &PathBuf,
    include: &Option<String>,
    exclude: &Option<String>,
) -> Result<Vec<(PathBuf, String)>> {
    use walkdir::WalkDir;
    let mut files = Vec::new();
    for entry in WalkDir::new(project_path) {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && is_source_file(path) && should_include_file(path, include, exclude) {
            if let Ok(content) = tokio::fs::read_to_string(path).await {
                files.push((path.to_path_buf(), content));
            }
        }
    }
    Ok(files)
}

fn is_source_file(path: &std::path::Path) -> bool {
    if let Some(ext) = path.extension() {
        matches!(
            ext.to_str(),
            Some(
                "rs" | "ts"
                    | "tsx"
                    | "js"
                    | "jsx"
                    | "py"
                    | "c"
                    | "cpp"
                    | "cc"
                    | "h"
                    | "hpp"
                    | "kt"
                    | "java"
                    | "go"
            )
        )
    } else {
        false
    }
}

fn should_include_file(
    path: &std::path::Path,
    include: &Option<String>,
    exclude: &Option<String>,
) -> bool {
    let path_str = path.to_string_lossy();
    if let Some(exclude_pattern) = exclude {
        if path_str.contains(exclude_pattern) {
            return false;
        }
    }
    if let Some(include_pattern) = include {
        return path_str.contains(include_pattern);
    }
    true
}

fn filter_top_files(report: ComprehensiveReport, top_files: usize) -> ComprehensiveReport {
    if top_files > 0 {
        eprintln!("📈 Showing top {top_files} files with issues");
    }
    report
}

fn format_report(
    report: &ComprehensiveReport,
    format: crate::cli::DuplicateOutputFormat,
) -> Result<String> {
    match format {
        crate::cli::DuplicateOutputFormat::Json => Ok(serde_json::to_string_pretty(report)?),
        crate::cli::DuplicateOutputFormat::Summary | crate::cli::DuplicateOutputFormat::Human => {
            format_summary_report(report)
        }
        crate::cli::DuplicateOutputFormat::Detailed => format_detailed_report(report),
        crate::cli::DuplicateOutputFormat::Csv => format_csv_report(report),
        crate::cli::DuplicateOutputFormat::Sarif => format_sarif_report(report),
    }
}

// --- Submodule includes ---
include!("similarity_handler_formatting.rs");
include!("similarity_handler_output.rs");

#[cfg(test)]
#[path = "similarity_handler_tests.rs"]
mod tests;
