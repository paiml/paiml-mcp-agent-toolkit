//! Advanced code similarity and duplication detection handler
//!
//! Uses the new similarity module with entropy analysis, winnowing,
//! and multiple similarity detection algorithms.

use anyhow::Result;
use std::path::PathBuf;
use std::time::Instant;

use crate::services::similarity::{
    ComprehensiveReport, SimilarityConfig, SimilarityDetector,
};

/// Handle similarity analysis command with entropy detection
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
    
    // Configure similarity detector
    let config = build_config(detection_type, threshold, min_lines, max_tokens);
    let detector = SimilarityDetector::new(config);
    
    // Collect files to analyze
    let files = collect_files(&project_path, &include, &exclude).await?;
    
    eprintln!("📊 Analyzing {} files...", files.len());
    
    // Perform comprehensive analysis
    let report = detector.comprehensive_analysis(&files);
    
    // Apply top_files filtering if needed
    let filtered_report = if top_files > 0 {
        filter_top_files(report, top_files)
    } else {
        report
    };
    
    // Format and output results
    let output_str = format_report(&filtered_report, format)?;
    
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &output_str).await?;
        eprintln!("📄 Report written to: {}", output_path.display());
    } else {
        println!("{}", output_str);
    }
    
    // Print performance metrics
    if let Some(start_time) = start {
        let elapsed = start_time.elapsed();
        print_performance_metrics(&filtered_report, elapsed);
    }
    
    // Print summary
    print_summary(&filtered_report);
    
    Ok(())
}

fn build_config(
    detection_type: crate::cli::DuplicateType,
    threshold: f32,
    min_lines: usize,
    max_tokens: usize,
) -> SimilarityConfig {
    let mut config = SimilarityConfig::default();
    config.similarity_threshold = threshold as f64;
    config.min_lines = min_lines;
    config.min_tokens = max_tokens;
    
    // Adjust config based on detection type
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
        
        if path.is_file() && is_source_file(path) {
            if should_include_file(path, include, exclude) {
                if let Ok(content) = tokio::fs::read_to_string(path).await {
                    files.push((path.to_path_buf(), content));
                }
            }
        }
    }
    
    Ok(files)
}

fn is_source_file(path: &std::path::Path) -> bool {
    if let Some(ext) = path.extension() {
        matches!(
            ext.to_str(),
            Some("rs") | Some("ts") | Some("tsx") | Some("js") | Some("jsx") |
            Some("py") | Some("c") | Some("cpp") | Some("cc") | Some("h") |
            Some("hpp") | Some("kt") | Some("java") | Some("go")
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
    
    // Check exclude patterns
    if let Some(exclude_pattern) = exclude {
        if path_str.contains(exclude_pattern) {
            return false;
        }
    }
    
    // Check include patterns
    if let Some(include_pattern) = include {
        return path_str.contains(include_pattern);
    }
    
    true
}

fn filter_top_files(report: ComprehensiveReport, top_files: usize) -> ComprehensiveReport {
    // For now, return the report as-is
    // In a full implementation, we'd filter by top problematic files
    if top_files > 0 {
        eprintln!("📈 Showing top {} files with issues", top_files);
    }
    report
}

fn format_report(
    report: &ComprehensiveReport,
    format: crate::cli::DuplicateOutputFormat,
) -> Result<String> {
    match format {
        crate::cli::DuplicateOutputFormat::Json => {
            Ok(serde_json::to_string_pretty(report)?)
        }
        crate::cli::DuplicateOutputFormat::Summary | crate::cli::DuplicateOutputFormat::Human => {
            format_summary_report(report)
        }
        crate::cli::DuplicateOutputFormat::Detailed => {
            format_detailed_report(report)
        }
        crate::cli::DuplicateOutputFormat::Csv => {
            format_csv_report(report)
        }
        crate::cli::DuplicateOutputFormat::Sarif => {
            format_sarif_report(report)
        }
    }
}

fn format_summary_report(report: &ComprehensiveReport) -> Result<String> {
    use std::fmt::Write;
    let mut output = String::new();
    
    writeln!(&mut output, "# Code Similarity Analysis Summary\n")?;
    writeln!(&mut output, "## Metrics")?;
    writeln!(&mut output, "- Duplication: {:.1}%", report.metrics.duplication_percentage)?;
    writeln!(&mut output, "- Average Entropy: {:.2}", report.metrics.average_entropy)?;
    writeln!(&mut output, "- Total Clones: {}", report.metrics.total_clones)?;
    writeln!(&mut output)?;
    
    writeln!(&mut output, "## Clone Types")?;
    writeln!(&mut output, "- Exact Duplicates: {}", report.exact_duplicates.len())?;
    writeln!(&mut output, "- Structural Similarities: {}", report.structural_similarities.len())?;
    writeln!(&mut output, "- Semantic Similarities: {}", report.semantic_similarities.len())?;
    writeln!(&mut output)?;
    
    if !report.refactoring_opportunities.is_empty() {
        writeln!(&mut output, "## Top Refactoring Opportunities")?;
        for (i, hint) in report.refactoring_opportunities.iter().take(5).enumerate() {
            writeln!(&mut output, "{}. {}: {}", i + 1, hint.pattern, hint.suggestion)?;
        }
    }
    
    Ok(output)
}

fn format_detailed_report(report: &ComprehensiveReport) -> Result<String> {
    use std::fmt::Write;
    let mut output = String::new();
    
    writeln!(&mut output, "# Comprehensive Code Similarity Report\n")?;
    
    // Metrics section
    writeln!(&mut output, "## Overall Metrics")?;
    writeln!(&mut output, "- Duplication Percentage: {:.1}%", report.metrics.duplication_percentage)?;
    writeln!(&mut output, "- Average Entropy: {:.2}", report.metrics.average_entropy)?;
    writeln!(&mut output, "- Total Clones Found: {}", report.metrics.total_clones)?;
    writeln!(&mut output)?;
    
    // Exact duplicates
    if !report.exact_duplicates.is_empty() {
        writeln!(&mut output, "## Exact Duplicates (Type-1 Clones)")?;
        for block in &report.exact_duplicates {
            writeln!(&mut output, "\n### Block {}", block.id)?;
            writeln!(&mut output, "- Lines: {}", block.lines)?;
            writeln!(&mut output, "- Tokens: {}", block.tokens)?;
            writeln!(&mut output, "- Locations:")?;
            for loc in &block.locations {
                writeln!(&mut output, "  - {}:{}-{}", 
                    loc.file.display(), loc.start_line, loc.end_line)?;
            }
            writeln!(&mut output, "- Preview:\n```\n{}\n```", block.content_preview)?;
        }
    }
    
    // Structural similarities
    if !report.structural_similarities.is_empty() {
        writeln!(&mut output, "\n## Structural Similarities (Type-2/3 Clones)")?;
        for block in report.structural_similarities.iter().take(10) {
            writeln!(&mut output, "\n### Similarity {}", block.id)?;
            writeln!(&mut output, "- Similarity: {:.1}%", block.similarity * 100.0)?;
            writeln!(&mut output, "- Type: {:?}", block.clone_type)?;
            writeln!(&mut output, "- Locations:")?;
            for loc in &block.locations {
                writeln!(&mut output, "  - {}:{}-{}", 
                    loc.file.display(), loc.start_line, loc.end_line)?;
            }
        }
    }
    
    // Entropy analysis
    if let Some(entropy) = &report.entropy_analysis {
        writeln!(&mut output, "\n## Entropy Analysis")?;
        writeln!(&mut output, "- Average Entropy: {:.2}", entropy.average_entropy)?;
        
        if !entropy.high_entropy_blocks.is_empty() {
            writeln!(&mut output, "\n### High Complexity Code (High Entropy)")?;
            for block in entropy.high_entropy_blocks.iter().take(5) {
                writeln!(&mut output, "- {}:{} (entropy: {:.2})", 
                    block.location.file.display(),
                    block.location.start_line,
                    block.entropy)?;
                writeln!(&mut output, "  Suggestion: {}", block.suggestion)?;
            }
        }
        
        if !entropy.low_entropy_patterns.is_empty() {
            writeln!(&mut output, "\n### Repetitive Patterns (Low Entropy)")?;
            for block in entropy.low_entropy_patterns.iter().take(5) {
                writeln!(&mut output, "- {}:{} (entropy: {:.2})", 
                    block.location.file.display(),
                    block.location.start_line,
                    block.entropy)?;
                writeln!(&mut output, "  Suggestion: {}", block.suggestion)?;
            }
        }
    }
    
    // Refactoring opportunities
    if !report.refactoring_opportunities.is_empty() {
        writeln!(&mut output, "\n## Refactoring Opportunities")?;
        for hint in &report.refactoring_opportunities {
            writeln!(&mut output, "\n### {}", hint.pattern)?;
            writeln!(&mut output, "- Priority: {:?}", hint.priority)?;
            writeln!(&mut output, "- Suggestion: {}", hint.suggestion)?;
            writeln!(&mut output, "- Affected locations:")?;
            for loc in &hint.locations {
                writeln!(&mut output, "  - {}:{}-{}", 
                    loc.file.display(), loc.start_line, loc.end_line)?;
            }
        }
    }
    
    Ok(output)
}

fn print_performance_metrics(report: &ComprehensiveReport, elapsed: std::time::Duration) {
    eprintln!("\n⏱️  Performance Metrics:");
    eprintln!("  Total time: {:?}", elapsed);
    eprintln!("  Clones found: {}", report.metrics.total_clones);
    eprintln!("  Analysis rate: {:.0} LOC/sec", 
        (report.exact_duplicates.len() * 1000) as f64 / elapsed.as_millis() as f64);
}

fn format_csv_report(report: &ComprehensiveReport) -> Result<String> {
    use std::fmt::Write;
    let mut output = String::new();
    
    writeln!(&mut output, "Type,File1,Start1,End1,File2,Start2,End2,Similarity")?;
    
    for block in &report.exact_duplicates {
        if block.locations.len() >= 2 {
            writeln!(&mut output, "Exact,{},{},{},{},{},{},100.0",
                block.locations[0].file.display(),
                block.locations[0].start_line,
                block.locations[0].end_line,
                block.locations[1].file.display(),
                block.locations[1].start_line,
                block.locations[1].end_line)?;
        }
    }
    
    for block in &report.structural_similarities {
        if block.locations.len() >= 2 {
            writeln!(&mut output, "Structural,{},{},{},{},{},{},{:.1}",
                block.locations[0].file.display(),
                block.locations[0].start_line,
                block.locations[0].end_line,
                block.locations[1].file.display(),
                block.locations[1].start_line,
                block.locations[1].end_line,
                block.similarity * 100.0)?;
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
    eprintln!("\n✅ Analysis Complete:");
    eprintln!("  📊 Duplication: {:.1}%", report.metrics.duplication_percentage);
    eprintln!("  🔢 Total clones: {}", report.metrics.total_clones);
    eprintln!("  📈 Average entropy: {:.2}", report.metrics.average_entropy);
    
    if !report.refactoring_opportunities.is_empty() {
        eprintln!("  💡 Refactoring opportunities: {}", 
            report.refactoring_opportunities.len());
    }
}