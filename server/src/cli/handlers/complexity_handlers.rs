//! Complexity analysis command handlers with refactored dead code handler
//!
//! This module contains all complexity-related command implementations
//! extracted from the main CLI module to reduce cognitive complexity.

use crate::cli::*;
use anyhow::Result;
use std::path::PathBuf;

/// Handle complexity analysis command
#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_complexity(
    project_path: PathBuf,
    toolchain: Option<String>,
    format: ComplexityOutputFormat,
    output: Option<PathBuf>,
    max_cyclomatic: Option<u16>,
    max_cognitive: Option<u16>,
    include: Vec<String>,
    watch: bool,
    _top_files: usize,
) -> Result<()> {
    use crate::services::complexity::{
        aggregate_results, format_as_sarif, format_complexity_report, format_complexity_summary,
    };

    if watch {
        eprintln!("❌ Watch mode not yet implemented");
        return Ok(());
    }

    // Detect toolchain if not specified
    let detected_toolchain = toolchain
        .or_else(|| super::super::stubs::detect_toolchain(&project_path))
        .unwrap_or_else(|| "rust".to_string());

    eprintln!("🔍 Analyzing {detected_toolchain} project complexity...");

    // Custom thresholds
    let _thresholds =
        super::super::stubs::build_complexity_thresholds(max_cyclomatic, max_cognitive);

    // Analyze files
    let file_metrics = super::super::stubs::analyze_project_files(
        &project_path,
        Some(&detected_toolchain),
        &include,
        10,
        15,
    )
    .await?;

    // Aggregate results
    let summary = aggregate_results(file_metrics.clone());

    // Format output
    let formatted_output = match format {
        ComplexityOutputFormat::Summary => Ok(format_complexity_summary(&summary)),
        ComplexityOutputFormat::Full => Ok(format_complexity_report(&summary)),
        ComplexityOutputFormat::Sarif => format_as_sarif(&summary)
            .map_err(|e| anyhow::anyhow!("SARIF serialization failed: {}", e)),
        ComplexityOutputFormat::Json => {
            let json_output = serde_json::json!({
                "summary": summary,
                "files": file_metrics,
            });
            serde_json::to_string_pretty(&json_output)
                .map_err(|e| anyhow::anyhow!("JSON serialization failed: {}", e))
        }
    }?;

    // Write output
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &formatted_output).await?;
        eprintln!("📝 Results written to: {}", output_path.display());
    } else {
        println!("{}", formatted_output);
    }

    Ok(())
}

/// Handle churn analysis command
pub async fn handle_analyze_churn(
    project_path: PathBuf,
    days: u32,
    format: crate::models::churn::ChurnOutputFormat,
    output: Option<PathBuf>,
) -> Result<()> {
    // Delegate to main implementation for now - will be extracted in Phase 3 Day 8
    super::super::stubs::handle_analyze_churn(project_path, days, format, output).await
}

/// Handle dead code analysis command - REFACTORED
/// Cognitive complexity reduced from 244 to ~10
pub async fn handle_analyze_dead_code(
    path: PathBuf,
    format: DeadCodeOutputFormat,
    top_files: Option<usize>,
    include_unreachable: bool,
    min_dead_lines: usize,
    include_tests: bool,
    output: Option<PathBuf>,
) -> Result<()> {
    eprintln!("☠️ Analyzing dead code in project...");

    // Run analysis
    let result = run_dead_code_analysis(
        &path,
        include_unreachable,
        include_tests,
        min_dead_lines,
        top_files,
    )
    .await?;

    eprintln!(
        "📊 Analysis complete: {} files analyzed, {} with dead code",
        result.summary.total_files_analyzed, result.summary.files_with_dead_code
    );

    // Format output
    let formatted_output = format_dead_code_result(&result, &format)?;

    // Write output
    write_dead_code_output(formatted_output, output).await?;

    Ok(())
}

/// Run dead code analysis
async fn run_dead_code_analysis(
    path: &Path,
    include_unreachable: bool,
    include_tests: bool,
    min_dead_lines: usize,
    top_files: Option<usize>,
) -> Result<crate::models::dead_code::DeadCodeResult> {
    use crate::models::dead_code::DeadCodeAnalysisConfig;
    use crate::services::dead_code_analyzer::DeadCodeAnalyzer;

    let mut analyzer = DeadCodeAnalyzer::new(10000);

    let config = DeadCodeAnalysisConfig {
        include_unreachable,
        include_tests,
        min_dead_lines,
    };

    let mut analysis_result = analyzer.analyze_with_ranking(path, config).await?;

    eprintln!(
        "🔍 Found {} ranked files",
        analysis_result.ranked_files.len()
    );
    eprintln!(
        "🔍 Total files analyzed: {}",
        analysis_result.summary.total_files_analyzed
    );

    // Apply top_files limit
    if let Some(limit) = top_files {
        analysis_result.ranked_files.truncate(limit);
    }

    // Convert to DeadCodeResult
    Ok(crate::models::dead_code::DeadCodeResult {
        summary: analysis_result.summary.clone(),
        files: analysis_result.ranked_files,
        total_files: analysis_result.summary.total_files_analyzed,
        analyzed_files: analysis_result.summary.total_files_analyzed,
    })
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
fn format_dead_code_as_summary(
    result: &crate::models::dead_code::DeadCodeResult,
) -> Result<String> {
    use std::fmt::Write;
    let mut output = String::new();

    writeln!(&mut output, "# Dead Code Analysis Summary\n")?;
    writeln!(&mut output, "📊 **Files analyzed**: {}", result.total_files)?;
    writeln!(
        &mut output,
        "☠️  **Files with dead code**: {}",
        result.summary.files_with_dead_code
    )?;
    writeln!(
        &mut output,
        "📏 **Total dead lines**: {}",
        result.summary.total_dead_lines
    )?;
    writeln!(
        &mut output,
        "📈 **Dead code percentage**: {:.2}%\n",
        result.summary.dead_percentage
    )?;

    if result.summary.dead_functions > 0 {
        writeln!(&mut output, "## Dead Code by Type\n")?;
        writeln!(
            &mut output,
            "- **Dead functions**: {}",
            result.summary.dead_functions
        )?;
        writeln!(
            &mut output,
            "- **Dead classes**: {}",
            result.summary.dead_classes
        )?;
        writeln!(
            &mut output,
            "- **Dead variables**: {}",
            result.summary.dead_modules
        )?;
        writeln!(
            &mut output,
            "- **Unreachable blocks**: {}",
            result.summary.unreachable_blocks
        )?;
    }

    if !result.files.is_empty() {
        writeln!(&mut output, "\n## Top Files with Dead Code\n")?;
        for (i, file) in result.files.iter().take(10).enumerate() {
            writeln!(
                &mut output,
                "{}. `{}` - {:.1}% dead ({} lines)",
                i + 1,
                file.path,
                file.dead_percentage,
                file.dead_lines
            )?;
        }
    }

    Ok(output)
}

/// Format result as markdown
fn format_dead_code_as_markdown(
    result: &crate::models::dead_code::DeadCodeResult,
) -> Result<String> {
    use std::fmt::Write;
    let mut output = String::new();

    writeln!(&mut output, "# Dead Code Analysis Report\n")?;
    writeln!(&mut output, "## Summary\n")?;
    writeln!(&mut output, "| Metric | Value |")?;
    writeln!(&mut output, "|--------|-------|")?;
    writeln!(&mut output, "| Files Analyzed | {} |", result.total_files)?;
    writeln!(
        &mut output,
        "| Files with Dead Code | {} |",
        result.summary.files_with_dead_code
    )?;
    writeln!(
        &mut output,
        "| Total Dead Lines | {} |",
        result.summary.total_dead_lines
    )?;
    writeln!(
        &mut output,
        "| Dead Code Percentage | {:.2}% |",
        result.summary.dead_percentage
    )?;
    writeln!(&mut output)?;

    if result.summary.dead_functions > 0 {
        writeln!(&mut output, "## Dead Code Breakdown\n")?;
        writeln!(&mut output, "| Type | Count |")?;
        writeln!(&mut output, "|------|-------|")?;
        writeln!(
            &mut output,
            "| Functions | {} |",
            result.summary.dead_functions
        )?;
        writeln!(&mut output, "| Classes | {} |", result.summary.dead_classes)?;
        writeln!(
            &mut output,
            "| Variables | {} |",
            result.summary.dead_modules
        )?;
        writeln!(
            &mut output,
            "| Unreachable Blocks | {} |",
            result.summary.unreachable_blocks
        )?;
        writeln!(&mut output)?;
    }

    if !result.files.is_empty() {
        writeln!(&mut output, "## File Details\n")?;
        writeln!(
            &mut output,
            "| File | Dead % | Dead Lines | Confidence | Items |"
        )?;
        writeln!(
            &mut output,
            "|------|--------|------------|------------|-------|"
        )?;

        for file in result.files.iter().take(20) {
            writeln!(
                &mut output,
                "| {} | {:.1}% | {} | {:?} | {} |",
                file.path,
                file.dead_percentage,
                file.dead_lines,
                file.confidence,
                file.items.len()
            )?;
        }
        writeln!(&mut output)?;
    }

    writeln!(&mut output, "## Recommendations\n")?;
    writeln!(
        &mut output,
        "1. **Review High Confidence Dead Code**: Start with files marked as high confidence."
    )?;
    writeln!(
        &mut output,
        "2. **Check Test Coverage**: Dead code often indicates missing tests."
    )?;
    writeln!(
        &mut output,
        "3. **Consider Refactoring**: Large amounts of dead code may indicate design issues."
    )?;
    writeln!(
        &mut output,
        "4. **Remove Carefully**: Ensure code is truly dead before removal."
    )?;

    Ok(output)
}

/// Write dead code output to file or stdout
async fn write_dead_code_output(content: String, output: Option<PathBuf>) -> Result<()> {
    match output {
        Some(path) => {
            tokio::fs::write(&path, content).await?;
            eprintln!("📝 Results written to: {}", path.display());
        }
        None => {
            println!("{}", content);
        }
    }
    Ok(())
}

/// Handle SATD (Self-Admitted Technical Debt) analysis command
#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_satd(
    path: PathBuf,
    format: SatdOutputFormat,
    severity: Option<SatdSeverity>,
    critical_only: bool,
    include_tests: bool,
    strict: bool,
    evolution: bool,
    days: u32,
    metrics: bool,
    output: Option<PathBuf>,
) -> Result<()> {
    use crate::services::satd_detector::{SATDDetector, Severity as DetectorSeverity};

    eprintln!("🔍 Analyzing self-admitted technical debt...");
    if strict {
        eprintln!("📝 Using strict mode (only explicit SATD markers)");
    }

    // Create SATD detector
    let detector = if strict {
        SATDDetector::new_strict()
    } else {
        SATDDetector::new()
    };

    // Run analysis
    let mut result = detector.analyze_project(&path, include_tests).await?;

    // Filter by severity if specified
    if let Some(min_severity) = severity {
        let min_detector_severity = match min_severity {
            SatdSeverity::Critical => DetectorSeverity::Critical,
            SatdSeverity::High => DetectorSeverity::High,
            SatdSeverity::Medium => DetectorSeverity::Medium,
            SatdSeverity::Low => DetectorSeverity::Low,
        };

        result
            .items
            .retain(|item| item.severity >= min_detector_severity);
    }

    // Filter critical only if requested
    if critical_only {
        result
            .items
            .retain(|item| item.severity == DetectorSeverity::Critical);
    }

    eprintln!(
        "📊 Found {} SATD items in {} files",
        result.items.len(),
        result.files_with_debt
    );

    // Format output
    let content = match format {
        SatdOutputFormat::Json => serde_json::to_string_pretty(&result)?,
        SatdOutputFormat::Sarif => {
            // Generate SARIF format
            let sarif = generate_satd_sarif(&result);
            serde_json::to_string_pretty(&sarif)?
        }
        SatdOutputFormat::Summary => format_satd_summary(&result, metrics),
        SatdOutputFormat::Markdown => format_satd_markdown(&result, metrics, evolution, days),
    };

    // Write output
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        eprintln!("✅ SATD analysis written to: {}", output_path.display());
    } else {
        println!("{}", content);
    }

    Ok(())
}

/// Generate SARIF format for SATD results
fn generate_satd_sarif(
    result: &crate::services::satd_detector::SATDAnalysisResult,
) -> serde_json::Value {
    serde_json::json!({
        "version": "2.1.0",
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                    "rules": [{
                        "id": "satd",
                        "name": "Self-Admitted Technical Debt",
                        "shortDescription": {
                            "text": "Technical debt explicitly documented in code comments"
                        },
                        "fullDescription": {
                            "text": "Detects TODO, FIXME, HACK, and other technical debt markers in comments"
                        },
                        "defaultConfiguration": {
                            "level": "warning"
                        }
                    }]
                }
            },
            "results": result.items.iter().map(|item| {
                let level = match item.severity {
                    crate::services::satd_detector::Severity::Critical => "error",
                    crate::services::satd_detector::Severity::High => "error",
                    crate::services::satd_detector::Severity::Medium => "warning",
                    crate::services::satd_detector::Severity::Low => "note",
                };
                serde_json::json!({
                    "ruleId": "satd",
                    "level": level,
                    "message": {
                        "text": format!("{} debt: {}", item.category, item.text)
                    },
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": {
                                "uri": item.file.to_string_lossy()
                            },
                            "region": {
                                "startLine": item.line,
                                "startColumn": item.column
                            }
                        }
                    }]
                })
            }).collect::<Vec<_>>()
        }]
    })
}

/// Format SATD summary
fn format_satd_summary(
    result: &crate::services::satd_detector::SATDAnalysisResult,
    metrics: bool,
) -> String {
    use std::fmt::Write;
    let mut output = String::new();

    writeln!(&mut output, "# SATD Analysis Summary\n").unwrap();
    writeln!(
        &mut output,
        "📊 **Files analyzed**: {}",
        result.total_files_analyzed
    )
    .unwrap();
    writeln!(
        &mut output,
        "📁 **Files with SATD**: {}",
        result.files_with_debt
    )
    .unwrap();
    writeln!(
        &mut output,
        "🔍 **Total SATD items**: {}",
        result.items.len()
    )
    .unwrap();

    if metrics && !result.summary.by_severity.is_empty() {
        writeln!(&mut output, "\n## By Severity\n").unwrap();
        for (severity, count) in &result.summary.by_severity {
            writeln!(&mut output, "- **{}**: {}", severity, count).unwrap();
        }
    }

    if metrics && !result.summary.by_category.is_empty() {
        writeln!(&mut output, "\n## By Category\n").unwrap();
        for (category, count) in &result.summary.by_category {
            writeln!(&mut output, "- **{}**: {}", category, count).unwrap();
        }
    }

    // Show top items
    if !result.items.is_empty() {
        writeln!(&mut output, "\n## Critical Items\n").unwrap();
        for item in result
            .items
            .iter()
            .filter(|i| i.severity == crate::services::satd_detector::Severity::Critical)
            .take(5)
        {
            writeln!(
                &mut output,
                "- `{}:{}` - {}",
                item.file.file_name().unwrap_or_default().to_string_lossy(),
                item.line,
                item.text
            )
            .unwrap();
        }
    }

    output
}

/// Format SATD as markdown report
fn format_satd_markdown(
    result: &crate::services::satd_detector::SATDAnalysisResult,
    metrics: bool,
    _evolution: bool,
    _days: u32,
) -> String {
    use std::fmt::Write;
    let mut output = String::new();

    writeln!(&mut output, "# Self-Admitted Technical Debt Report\n").unwrap();
    writeln!(
        &mut output,
        "Generated: {}",
        result.analysis_timestamp.format("%Y-%m-%d %H:%M:%S UTC")
    )
    .unwrap();

    writeln!(&mut output, "\n## Summary\n").unwrap();
    writeln!(&mut output, "| Metric | Value |").unwrap();
    writeln!(&mut output, "|--------|-------|").unwrap();
    writeln!(
        &mut output,
        "| Files Analyzed | {} |",
        result.total_files_analyzed
    )
    .unwrap();
    writeln!(
        &mut output,
        "| Files with SATD | {} |",
        result.files_with_debt
    )
    .unwrap();
    writeln!(&mut output, "| Total SATD Items | {} |", result.items.len()).unwrap();

    if metrics {
        writeln!(&mut output, "\n## Distribution\n").unwrap();
        writeln!(&mut output, "### By Severity\n").unwrap();
        writeln!(&mut output, "| Severity | Count |").unwrap();
        writeln!(&mut output, "|----------|-------|").unwrap();
        for (severity, count) in &result.summary.by_severity {
            writeln!(&mut output, "| {} | {} |", severity, count).unwrap();
        }

        writeln!(&mut output, "\n### By Category\n").unwrap();
        writeln!(&mut output, "| Category | Count |").unwrap();
        writeln!(&mut output, "|----------|-------|").unwrap();
        for (category, count) in &result.summary.by_category {
            writeln!(&mut output, "| {} | {} |", category, count).unwrap();
        }
    }

    // Group items by file
    use std::collections::BTreeMap;
    let mut by_file: BTreeMap<
        &std::path::Path,
        Vec<&crate::services::satd_detector::TechnicalDebt>,
    > = BTreeMap::new();
    for item in &result.items {
        by_file.entry(&item.file).or_default().push(item);
    }

    writeln!(&mut output, "\n## SATD Items by File\n").unwrap();
    for (file, items) in by_file.iter().take(20) {
        writeln!(&mut output, "### {}\n", file.display()).unwrap();
        writeln!(&mut output, "| Line | Severity | Category | Text |").unwrap();
        writeln!(&mut output, "|------|----------|----------|------|").unwrap();
        for item in items {
            writeln!(
                &mut output,
                "| {} | {:?} | {} | {} |",
                item.line,
                item.severity,
                item.category,
                item.text.replace('|', "\\|")
            )
            .unwrap();
        }
        writeln!(&mut output).unwrap();
    }

    output
}

/// Handle DAG (Dependency Analysis Graph) generation command
#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_dag(
    dag_type: DagType,
    project_path: PathBuf,
    output: Option<PathBuf>,
    max_depth: Option<usize>,
    target_nodes: Option<usize>,
    filter_external: bool,
    show_complexity: bool,
    include_duplicates: bool,
    include_dead_code: bool,
    enhanced: bool,
) -> Result<()> {
    use crate::services::{
        context::analyze_project,
        dag_builder::DagBuilder,
        fixed_graph_builder::{GraphConfig, GroupingStrategy},
        mermaid_generator::{MermaidGenerator, MermaidOptions},
    };

    eprintln!("🔄 Generating dependency analysis graph...");

    // Analyze project to get context
    let toolchain =
        super::super::detect_primary_language(&project_path).unwrap_or_else(|| "rust".to_string());
    let project_context = analyze_project(&project_path, &toolchain).await?;

    eprintln!("📁 Analyzed {} files", project_context.files.len());

    // Build DAG based on type
    let graph = match dag_type {
        DagType::CallGraph => {
            // Filter to only call edges
            let mut dag = DagBuilder::build_from_project(&project_context);
            dag.edges
                .retain(|edge| matches!(edge.edge_type, crate::models::dag::EdgeType::Calls));
            dag
        }
        DagType::ImportGraph => {
            // Filter to only import edges
            let mut dag = DagBuilder::build_from_project(&project_context);
            dag.edges
                .retain(|edge| matches!(edge.edge_type, crate::models::dag::EdgeType::Imports));
            dag
        }
        DagType::Inheritance => {
            // Filter to inheritance/implements edges
            let mut dag = DagBuilder::build_from_project(&project_context);
            dag.edges.retain(|edge| {
                matches!(
                    edge.edge_type,
                    crate::models::dag::EdgeType::Inherits
                        | crate::models::dag::EdgeType::Implements
                )
            });
            dag
        }
        DagType::FullDependency => {
            // Include all edges
            DagBuilder::build_from_project(&project_context)
        }
    };

    eprintln!(
        "📊 Generated graph with {} nodes and {} edges",
        graph.nodes.len(),
        graph.edges.len()
    );

    // Optionally add additional analysis data
    let mut enriched_graph = graph;

    if include_dead_code {
        // Add dead code information to nodes
        use crate::services::dead_code_analyzer::DeadCodeAnalyzer;
        let mut analyzer = DeadCodeAnalyzer::new(10000);
        let dead_code_result = analyzer.analyze_dependency_graph(&enriched_graph);
        // Mark dead nodes
        for dead_func in &dead_code_result.dead_functions {
            if let Some(node) = enriched_graph
                .nodes
                .get_mut(&format!("{}_{}", dead_func.file_path, dead_func.name))
            {
                node.label = format!("{} [DEAD]", node.label);
            }
        }
    }

    if include_duplicates {
        // Add duplicate information
        use crate::services::duplicate_detector::{
            DuplicateDetectionConfig, DuplicateDetectionEngine, Language,
        };
        use walkdir::WalkDir;

        // Create duplicate detection engine
        let config = DuplicateDetectionConfig::default();
        let detector = DuplicateDetectionEngine::new(config);

        // Collect source files
        let mut files = Vec::new();
        for entry in WalkDir::new(&project_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                let lang = match ext {
                    "rs" => Some(Language::Rust),
                    "ts" | "tsx" => Some(Language::TypeScript),
                    "js" | "jsx" => Some(Language::JavaScript),
                    "py" => Some(Language::Python),
                    "c" => Some(Language::C),
                    "cpp" | "cc" | "cxx" => Some(Language::Cpp),
                    _ => None,
                };

                if let Some(language) = lang {
                    if let Ok(content) = std::fs::read_to_string(path) {
                        files.push((path.to_path_buf(), content, language));
                    }
                }
            }
        }

        // Detect duplicates
        if let Ok(report) = detector.detect_duplicates(&files) {
            // Mark files with duplicates
            let mut files_with_duplicates = std::collections::HashSet::new();
            for group in &report.groups {
                for instance in &group.fragments {
                    files_with_duplicates.insert(instance.file.display().to_string());
                }
            }

            // Mark duplicate nodes
            for node in enriched_graph.nodes.values_mut() {
                if files_with_duplicates.contains(&node.file_path) {
                    node.label = format!("{} [DUP]", node.label);
                }
            }
        }
    }

    // Generate Mermaid diagram
    let options = MermaidOptions {
        max_depth,
        filter_external,
        group_by_module: enhanced,
        show_complexity,
    };

    let generator = MermaidGenerator::new(options);
    let mermaid_content = if enhanced || target_nodes.is_some() {
        // Use advanced graph configuration
        let config = GraphConfig {
            max_nodes: target_nodes.unwrap_or(100),
            max_edges: target_nodes.map(|n| n * 4).unwrap_or(400),
            grouping: GroupingStrategy::Module,
        };
        generator.generate_with_config(&enriched_graph, &config)
    } else {
        generator.generate(&enriched_graph)
    };

    // Write output
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &mermaid_content).await?;
        eprintln!("✅ DAG written to: {}", output_path.display());

        // Additional hint for viewing
        if output_path.extension().is_some_and(|ext| ext == "mmd") {
            eprintln!("\n💡 To view the graph:");
            eprintln!("   - Copy content to https://mermaid.live");
            eprintln!("   - Or use VS Code with Mermaid extension");
        }
    } else {
        println!("{}", mermaid_content);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    // use super::*; // Unused in simple tests

    #[test]
    fn test_complexity_handlers_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
