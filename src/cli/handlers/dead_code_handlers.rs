//! Dead Code Analysis Handler
//!
//! Extracted from complexity_handlers.rs for file health compliance (CB-040).
//! Contains dead code analysis handler and all related helper functions.

#![cfg_attr(coverage_nightly, coverage(off))]
use crate::cli::DeadCodeOutputFormat;
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Handle dead code analysis command - REFACTORED
/// Cognitive complexity reduced from 244 to ~10
#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_dead_code(
    path: PathBuf,
    format: DeadCodeOutputFormat,
    top_files: Option<usize>,
    include_unreachable: bool,
    min_dead_lines: usize,
    include_tests: bool,
    output: Option<PathBuf>,
    fail_on_violation: bool,
    max_percentage: f64,
    timeout: u64,
    include: Vec<String>,
    exclude: Vec<String>,
    max_depth: usize,
) -> Result<()> {
    eprintln!("☠️ Analyzing dead code in project...");
    eprintln!("⏰ Analysis timeout set to {timeout} seconds");

    // Apply include/exclude filters if specified
    if !include.is_empty() || !exclude.is_empty() {
        eprintln!("🔍 Applying file filters...");
        if !include.is_empty() {
            eprintln!("  Include patterns: {include:?}");
        }
        if !exclude.is_empty() {
            eprintln!("  Exclude patterns: {exclude:?}");
        }
    }

    // Run analysis with timeout
    let timeout_duration = tokio::time::Duration::from_secs(timeout);
    let result = tokio::time::timeout(timeout_duration, async {
        run_dead_code_analysis_with_filters(
            &path,
            DeadCodeAnalysisFilters {
                include_unreachable,
                include_tests,
                min_dead_lines,
                top_files,
                include,
                exclude,
                max_depth,
            },
        )
        .await
    })
    .await
    .map_err(|_| anyhow::anyhow!("Dead code analysis timed out after {timeout} seconds"))??;

    eprintln!(
        "📊 Analysis complete: {} files analyzed, {} with dead code",
        result.summary.total_files_analyzed, result.summary.files_with_dead_code
    );

    // Format output
    let formatted_output = format_dead_code_result(&result, &format)?;

    // Write output
    write_dead_code_output(formatted_output, output).await?;

    // Check for violations and exit with error code if requested
    if fail_on_violation {
        let dead_code_percentage = result.summary.dead_percentage;
        if dead_code_percentage > max_percentage as f32 {
            eprintln!(
                "\n❌ Dead code violations found: {dead_code_percentage:.1}% exceeds threshold of {max_percentage:.1}%"
            );
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Configuration for dead code analysis
#[allow(clippy::too_many_arguments)]
struct DeadCodeAnalysisFilters {
    include_unreachable: bool,
    include_tests: bool,
    min_dead_lines: usize,
    top_files: Option<usize>,
    include: Vec<String>,
    exclude: Vec<String>,
    max_depth: usize,
}

/// Run dead code analysis with include/exclude filters
async fn run_dead_code_analysis_with_filters(
    path: &Path,
    filters: DeadCodeAnalysisFilters,
) -> Result<crate::models::dead_code::DeadCodeResult> {
    use crate::models::dead_code::DeadCodeAnalysisConfig;
    use crate::utils::file_filter::FileFilter;

    // Detect project language to choose the right analyzer
    let detection =
        crate::services::enhanced_language_detection::detect_project_language_enhanced(path);

    // For non-Rust projects, use the multi-language analyzer
    if detection.language != "rust" {
        return run_multi_language_dead_code(path, &filters, &detection.language);
    }

    // Create file filter
    let filter = FileFilter::new(filters.include, filters.exclude)?;

    // Use the accurate cargo-based analyzer for Rust projects
    use crate::services::cargo_dead_code_analyzer::CargoDeadCodeAnalyzer;
    let cargo_analyzer = if filters.include_tests {
        CargoDeadCodeAnalyzer::new(path)
            .include_tests()
            .with_max_depth(filters.max_depth)
    } else {
        CargoDeadCodeAnalyzer::new(path).with_max_depth(filters.max_depth)
    };

    // Run cargo-based analysis for accurate results
    let accurate_report = cargo_analyzer.analyze().await?;

    // Create config for the result
    let config = DeadCodeAnalysisConfig {
        include_unreachable: filters.include_unreachable,
        include_tests: filters.include_tests,
        min_dead_lines: filters.min_dead_lines,
    };

    // Convert cargo report to ranking format for compatibility
    let files_with_dead_code_count = accurate_report.files_with_dead_code.len();
    let mut analysis_result = create_dead_code_ranking_result(
        accurate_report,
        files_with_dead_code_count,
        filters.min_dead_lines,
        config,
    );

    // Apply file filter to results if filters are active
    if filter.has_filters() {
        analysis_result.ranked_files.retain(|file| {
            let path = std::path::Path::new(&file.path);
            filter.should_include(path)
        });

        // Update summary counts
        analysis_result.summary.files_with_dead_code = analysis_result.ranked_files.len();
        analysis_result.summary.total_dead_lines = analysis_result
            .ranked_files
            .iter()
            .map(|f| f.dead_lines)
            .sum();
    }

    // Apply top_files limit if specified
    if let Some(limit) = filters.top_files {
        if limit > 0 && analysis_result.ranked_files.len() > limit {
            analysis_result.ranked_files.truncate(limit);
        }
    }

    // Convert to DeadCodeResult
    Ok(crate::models::dead_code::DeadCodeResult {
        summary: analysis_result.summary.clone(),
        files: analysis_result.ranked_files,
        total_files: analysis_result.summary.total_files_analyzed,
        analyzed_files: analysis_result.summary.total_files_analyzed,
    })
}

/// Run multi-language dead code analysis for non-Rust projects
fn run_multi_language_dead_code(
    path: &Path,
    filters: &DeadCodeAnalysisFilters,
    language: &str,
) -> Result<crate::models::dead_code::DeadCodeResult> {
    use crate::models::dead_code::{
        ConfidenceLevel, DeadCodeItem, DeadCodeSummary, DeadCodeType, FileDeadCodeMetrics,
    };
    use crate::services::dead_code_multi_language::analyze_dead_code_multi_language;

    eprintln!("🌐 Using multi-language analyzer for {language}");

    let ml_result = analyze_dead_code_multi_language(path)?;

    // Group dead functions by file for FileDeadCodeMetrics
    let mut file_map: std::collections::HashMap<
        String,
        Vec<&crate::services::dead_code_multi_language::DeadFunction>,
    > = std::collections::HashMap::new();
    for dead_fn in &ml_result.dead_functions {
        file_map
            .entry(dead_fn.file.clone())
            .or_default()
            .push(dead_fn);
    }

    let mut files: Vec<FileDeadCodeMetrics> = file_map
        .into_iter()
        .map(|(file_path, dead_fns)| {
            let mut metrics = FileDeadCodeMetrics::new(file_path);
            metrics.total_lines = 100; // Estimate
            for dead_fn in &dead_fns {
                metrics.add_item(DeadCodeItem {
                    item_type: DeadCodeType::Function,
                    name: dead_fn.name.clone(),
                    line: dead_fn.line as u32,
                    reason: dead_fn.reason.clone(),
                });
            }
            // Lua has dynamic dispatch, so Medium confidence for non-local functions
            metrics.confidence = ConfidenceLevel::Medium;
            metrics.update_percentage();
            metrics.calculate_score();
            metrics
        })
        .filter(|f| f.dead_lines >= filters.min_dead_lines || f.dead_functions > 0)
        .collect();

    // Sort by score descending
    files.sort_by(|a, b| {
        b.dead_score
            .partial_cmp(&a.dead_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if let Some(limit) = filters.top_files {
        if limit > 0 && files.len() > limit {
            files.truncate(limit);
        }
    }

    let summary = DeadCodeSummary::from_files(&files);

    Ok(crate::models::dead_code::DeadCodeResult {
        summary,
        total_files: ml_result.total_functions.max(1),
        analyzed_files: ml_result.total_functions.max(1),
        files,
    })
}

/// Create dead code ranking result from cargo analysis report
fn create_dead_code_ranking_result(
    accurate_report: crate::services::cargo_dead_code_analyzer::AccurateDeadCodeReport,
    files_with_dead_code_count: usize,
    min_dead_lines: usize,
    config: crate::models::dead_code::DeadCodeAnalysisConfig,
) -> crate::models::dead_code::DeadCodeRankingResult {
    use crate::models::dead_code::DeadCodeRankingResult;
    use chrono::Utc;

    DeadCodeRankingResult {
        ranked_files: convert_cargo_files_to_metrics(
            accurate_report.files_with_dead_code.clone(),
            min_dead_lines,
        ),
        summary: create_dead_code_summary(&accurate_report, files_with_dead_code_count),
        analysis_timestamp: Utc::now(),
        config,
    }
}

/// Convert cargo dead code files to metrics format
fn convert_cargo_files_to_metrics(
    cargo_files: Vec<crate::services::cargo_dead_code_analyzer::FileDeadCode>,
    min_dead_lines: usize,
) -> Vec<crate::models::dead_code::FileDeadCodeMetrics> {
    use crate::models::dead_code::{ConfidenceLevel, FileDeadCodeMetrics};

    cargo_files
        .into_iter()
        .map(|file| {
            let dead_functions_count = count_dead_items_by_kind(
                &file,
                &[
                    crate::services::cargo_dead_code_analyzer::DeadCodeKind::Function,
                    crate::services::cargo_dead_code_analyzer::DeadCodeKind::Method,
                ],
            );
            let dead_classes_count = count_dead_items_by_kind(
                &file,
                &[
                    crate::services::cargo_dead_code_analyzer::DeadCodeKind::Struct,
                    crate::services::cargo_dead_code_analyzer::DeadCodeKind::Enum,
                ],
            );

            FileDeadCodeMetrics {
                path: file.file_path.display().to_string(),
                dead_lines: file.dead_items.len() * 4, // Estimate lines per item
                total_lines: 100,                      // Will be updated later if needed
                dead_percentage: file.file_dead_percentage as f32,
                dead_functions: dead_functions_count,
                dead_classes: dead_classes_count,
                dead_modules: 0,
                unreachable_blocks: 0,
                dead_score: file.file_dead_percentage as f32,
                confidence: ConfidenceLevel::High, // Cargo-based detection is high confidence
                items: Vec::new(), // Will be populated if needed for detailed reporting
            }
        })
        .filter(|f| f.dead_lines >= min_dead_lines)
        .collect()
}

/// Count dead items of specific kinds
fn count_dead_items_by_kind(
    file: &crate::services::cargo_dead_code_analyzer::FileDeadCode,
    kinds: &[crate::services::cargo_dead_code_analyzer::DeadCodeKind],
) -> usize {
    file.dead_items
        .iter()
        .filter(|i| kinds.contains(&i.kind))
        .count()
}

/// Create dead code summary from cargo report
fn create_dead_code_summary(
    accurate_report: &crate::services::cargo_dead_code_analyzer::AccurateDeadCodeReport,
    files_with_dead_code_count: usize,
) -> crate::models::dead_code::DeadCodeSummary {
    use crate::models::dead_code::DeadCodeSummary;

    DeadCodeSummary {
        total_files_analyzed: accurate_report.total_lines / 100, // Rough estimate
        files_with_dead_code: files_with_dead_code_count,
        total_dead_lines: accurate_report.dead_lines,
        dead_percentage: accurate_report.dead_code_percentage as f32,
        dead_functions: get_dead_count_by_types(accurate_report, &["function", "method"]),
        dead_classes: get_dead_count_by_types(accurate_report, &["struct", "enum"]),
        dead_modules: get_dead_count_by_types(accurate_report, &["module"]),
        unreachable_blocks: 0, // Not tracked by cargo
    }
}

/// Get total dead count for specific types
fn get_dead_count_by_types(
    report: &crate::services::cargo_dead_code_analyzer::AccurateDeadCodeReport,
    types: &[&str],
) -> usize {
    types
        .iter()
        .map(|type_name| report.dead_by_type.get(*type_name).copied().unwrap_or(0))
        .sum()
}

/// Format dead code result based on output format
fn format_dead_code_result(
    result: &crate::models::dead_code::DeadCodeResult,
    format: &DeadCodeOutputFormat,
) -> Result<String> {
    match format {
        DeadCodeOutputFormat::Json => format_dead_code_as_json(result),
        DeadCodeOutputFormat::Sarif => format_dead_code_as_sarif(result),
        DeadCodeOutputFormat::Summary => format_dead_code_as_summary(result),
        DeadCodeOutputFormat::Markdown => format_dead_code_as_markdown(result),
    }
}

/// Format result as JSON
fn format_dead_code_as_json(result: &crate::models::dead_code::DeadCodeResult) -> Result<String> {
    Ok(serde_json::to_string_pretty(result)?)
}

/// Format result as SARIF
fn format_dead_code_as_sarif(result: &crate::models::dead_code::DeadCodeResult) -> Result<String> {
    use crate::models::dead_code::{ConfidenceLevel, DeadCodeType};
    use serde_json::json;

    let sarif = json!({
        "version": "2.1.0",
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                    "rules": [{
                        "id": "dead-code",
                        "name": "Dead Code Detection",
                        "shortDescription": {
                            "text": "Code that is never executed or referenced"
                        },
                        "fullDescription": {
                            "text": "Detects functions, classes, and code blocks that are not reachable from any entry point"
                        },
                        "defaultConfiguration": {
                            "level": "warning"
                        }
                    }]
                }
            },
            "results": result.files.iter().flat_map(|file| {
                file.items.iter().map(|item| {
                    let level = match file.confidence {
                        ConfidenceLevel::High => "error",
                        ConfidenceLevel::Medium => "warning",
                        ConfidenceLevel::Low => "note",
                    };
                    json!({
                        "ruleId": "dead-code",
                        "level": level,
                        "message": {
                            "text": format!("{}: {}",
                                match item.item_type {
                                    DeadCodeType::Function => "Dead function",
                                    DeadCodeType::Class => "Dead class",
                                    DeadCodeType::Variable => "Dead variable",
                                    DeadCodeType::UnreachableCode => "Unreachable code",
                                },
                                item.reason
                            )
                        },
                        "locations": [{
                            "physicalLocation": {
                                "artifactLocation": {
                                    "uri": &file.path
                                },
                                "region": {
                                    "startLine": item.line
                                }
                            }
                        }]
                    })
                }).collect::<Vec<_>>()
            }).collect::<Vec<_>>()
        }]
    });
    Ok(serde_json::to_string_pretty(&sarif)?)
}

/// Format result as summary
pub fn format_dead_code_as_summary(
    result: &crate::models::dead_code::DeadCodeResult,
) -> Result<String> {
    let mut output = String::new();

    write_dead_code_header(&mut output, result)?;

    if result.summary.dead_functions > 0 {
        write_dead_code_by_type_section(&mut output, &result.summary)?;
    }

    if !result.files.is_empty() {
        write_top_files_section(&mut output, &result.files)?;
    }

    Ok(output)
}

/// Write dead code analysis header section
fn write_dead_code_header(
    output: &mut String,
    result: &crate::models::dead_code::DeadCodeResult,
) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "# Dead Code Analysis Summary\n")?;
    writeln!(output, "📊 **Files analyzed**: {}", result.total_files)?;
    writeln!(
        output,
        "☠️  **Files with dead code**: {}",
        result.summary.files_with_dead_code
    )?;
    writeln!(
        output,
        "📏 **Total dead lines**: {}",
        result.summary.total_dead_lines
    )?;
    writeln!(
        output,
        "📈 **Dead code percentage**: {:.2}%\n",
        result.summary.dead_percentage
    )?;

    Ok(())
}

/// Write dead code by type breakdown section
fn write_dead_code_by_type_section(
    output: &mut String,
    summary: &crate::models::dead_code::DeadCodeSummary,
) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "## Dead Code by Type\n")?;
    writeln!(output, "- **Dead functions**: {}", summary.dead_functions)?;
    writeln!(output, "- **Dead classes**: {}", summary.dead_classes)?;
    writeln!(output, "- **Dead variables**: {}", summary.dead_modules)?;
    writeln!(
        output,
        "- **Unreachable blocks**: {}",
        summary.unreachable_blocks
    )?;

    Ok(())
}

/// Write top files with dead code section
fn write_top_files_section(
    output: &mut String,
    files: &[crate::models::dead_code::FileDeadCodeMetrics],
) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "\n## Top Files with Dead Code\n")?;
    for (i, file) in files.iter().take(10).enumerate() {
        writeln!(
            output,
            "{}. `{}` - {:.1}% dead ({} lines)",
            i + 1,
            file.path,
            file.dead_percentage,
            file.dead_lines
        )?;
    }

    Ok(())
}

/// Format result as markdown
fn format_dead_code_as_markdown(
    result: &crate::models::dead_code::DeadCodeResult,
) -> Result<String> {
    let mut sections = Vec::new();

    // Build summary section
    sections.push(format_dead_code_summary_section(result));

    // Build breakdown section if needed
    if result.summary.dead_functions > 0 {
        sections.push(format_dead_code_breakdown_section(&result.summary));
    }

    // Build file details section if needed
    if !result.files.is_empty() {
        sections.push(format_dead_code_file_details_section(&result.files));
    }

    // Build recommendations section
    sections.push(format_dead_code_recommendations_section());

    Ok(sections.join("\n"))
}

fn format_dead_code_summary_section(result: &crate::models::dead_code::DeadCodeResult) -> String {
    format!(
        "# Dead Code Analysis Report\n\n\
         ## Summary\n\n\
         | Metric | Value |\n\
         |--------|-------|\n\
         | Files Analyzed | {} |\n\
         | Files with Dead Code | {} |\n\
         | Total Dead Lines | {} |\n\
         | Dead Code Percentage | {:.2}% |\n",
        result.total_files,
        result.summary.files_with_dead_code,
        result.summary.total_dead_lines,
        result.summary.dead_percentage
    )
}

fn format_dead_code_breakdown_section(
    summary: &crate::models::dead_code::DeadCodeSummary,
) -> String {
    format!(
        "## Dead Code Breakdown\n\n\
         | Type | Count |\n\
         |------|-------|\n\
         | Functions | {} |\n\
         | Classes | {} |\n\
         | Variables | {} |\n\
         | Unreachable Blocks | {} |\n",
        summary.dead_functions,
        summary.dead_classes,
        summary.dead_modules,
        summary.unreachable_blocks
    )
}

fn format_dead_code_file_details_section(
    files: &[crate::models::dead_code::FileDeadCodeMetrics],
) -> String {
    let mut output = String::from(
        "## File Details\n\n\
         | File | Dead % | Dead Lines | Confidence | Items |\n\
         |------|--------|------------|------------|-------|\n",
    );

    for file in files.iter().take(20) {
        output.push_str(&format!(
            "| {} | {:.1}% | {} | {:?} | {} |\n",
            file.path,
            file.dead_percentage,
            file.dead_lines,
            file.confidence,
            file.items.len()
        ));
    }

    output
}

fn format_dead_code_recommendations_section() -> String {
    "## Recommendations\n\n\
     1. **Review High Confidence Dead Code**: Start with files marked as high confidence.\n\
     2. **Check Test Coverage**: Dead code often indicates missing tests.\n\
     3. **Consider Refactoring**: Large amounts of dead code may indicate design issues.\n\
     4. **Remove Carefully**: Ensure code is truly dead before removal.\n"
        .to_string()
}

/// Write dead code output to file or stdout
async fn write_dead_code_output(content: String, output: Option<PathBuf>) -> Result<()> {
    match output {
        Some(path) => {
            tokio::fs::write(&path, content).await?;
            eprintln!("📝 Results written to: {}", path.display());
        }
        None => {
            println!("{content}");
        }
    }
    Ok(())
}
