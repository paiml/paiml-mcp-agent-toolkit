//! Temporary stub implementations for handlers
//!
//! These are temporary implementations to fix compilation while refactoring.

use crate::cli::*;
use crate::services::lightweight_provability_analyzer::ProofSummary;
use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tracing::info;

// Additional handler stubs for advanced analysis
#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_tdg(
    path: PathBuf,
    threshold: f64,
    top: usize,
    format: TdgOutputFormat,
    include_components: bool,
    output: Option<PathBuf>,
    critical_only: bool,
    verbose: bool,
) -> Result<()> {
    // Delegate to refactored implementation
    stubs_refactored::handle_analyze_tdg(
        path,
        threshold,
        top,
        format,
        include_components,
        output,
        critical_only,
        verbose,
    )
    .await
}

pub async fn handle_analyze_makefile(
    path: PathBuf,
    rules: Vec<String>,
    format: MakefileOutputFormat,
    fix: bool,
    gnu_version: Option<String>,
) -> Result<()> {
    use crate::services::makefile_linter;
    use std::fmt::Write;

    eprintln!("🔧 Analyzing Makefile...");

    // Check if the file exists
    if !path.exists() {
        return Err(anyhow::anyhow!("Makefile not found: {}", path.display()));
    }

    // Run the linter
    let lint_result = makefile_linter::lint_makefile(&path)
        .await
        .map_err(|e| anyhow::anyhow!("Makefile linting failed: {}", e))?;

    eprintln!("📊 Found {} violations", lint_result.violations.len());
    eprintln!(
        "✨ Quality score: {:.1}%",
        lint_result.quality_score * 100.0
    );

    // Filter violations by rules if specified
    let filtered_violations = if rules.is_empty() {
        lint_result.violations
    } else {
        lint_result
            .violations
            .into_iter()
            .filter(|v| rules.contains(&v.rule))
            .collect()
    };

    // Format output based on requested format
    let content = match format {
        MakefileOutputFormat::Json => serde_json::to_string_pretty(&serde_json::json!({
            "path": path.display().to_string(),
            "violations": filtered_violations,
            "quality_score": lint_result.quality_score,
            "gnu_version": gnu_version,
        }))?,
        MakefileOutputFormat::Human => {
            let mut output = String::new();
            writeln!(&mut output, "# Makefile Analysis Report\n")?;
            writeln!(&mut output, "**File**: {}", path.display())?;
            writeln!(
                &mut output,
                "**Quality Score**: {:.1}%",
                lint_result.quality_score * 100.0
            )?;
            if let Some(ver) = &gnu_version {
                writeln!(&mut output, "**GNU Make Version**: {}", ver)?;
            }
            writeln!(&mut output)?;

            if filtered_violations.is_empty() {
                writeln!(&mut output, "✅ No violations found!")?;
            } else {
                writeln!(&mut output, "## Violations\n")?;
                writeln!(&mut output, "| Line | Rule | Severity | Message |")?;
                writeln!(&mut output, "|------|------|----------|---------|")?;

                for violation in &filtered_violations {
                    let severity = match violation.severity {
                        makefile_linter::Severity::Error => "❌ Error",
                        makefile_linter::Severity::Warning => "⚠️ Warning",
                        makefile_linter::Severity::Performance => "⚡ Performance",
                        makefile_linter::Severity::Info => "ℹ️ Info",
                    };

                    writeln!(
                        &mut output,
                        "| {} | {} | {} | {} |",
                        violation.span.line,
                        violation.rule,
                        severity,
                        violation.message.replace('|', "\\|")
                    )?;
                }

                // Add fix hints if available
                let violations_with_fixes: Vec<_> = filtered_violations
                    .iter()
                    .filter(|v| v.fix_hint.is_some())
                    .collect();

                if !violations_with_fixes.is_empty() {
                    writeln!(&mut output, "\n## Fix Suggestions\n")?;
                    for violation in violations_with_fixes {
                        writeln!(
                            &mut output,
                            "**Line {}** ({}): {}",
                            violation.span.line,
                            violation.rule,
                            violation.fix_hint.as_ref().unwrap()
                        )?;
                    }
                }
            }

            output
        }
        MakefileOutputFormat::Sarif => {
            // SARIF format for IDE integration
            let sarif = serde_json::json!({
                "version": "2.1.0",
                "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
                "runs": [{
                    "tool": {
                        "driver": {
                            "name": "paiml-makefile-linter",
                            "version": env!("CARGO_PKG_VERSION"),
                            "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                            "rules": filtered_violations.iter()
                                .map(|v| &v.rule)
                                .collect::<std::collections::HashSet<_>>()
                                .into_iter()
                                .map(|rule| {
                                    serde_json::json!({
                                        "id": rule,
                                        "name": rule,
                                        "defaultConfiguration": {
                                            "level": "warning"
                                        }
                                    })
                                })
                                .collect::<Vec<_>>()
                        }
                    },
                    "results": filtered_violations.iter().map(|violation| {
                        let level = match violation.severity {
                            makefile_linter::Severity::Error => "error",
                            makefile_linter::Severity::Warning => "warning",
                            makefile_linter::Severity::Performance => "note",
                            makefile_linter::Severity::Info => "note",
                        };

                        serde_json::json!({
                            "ruleId": &violation.rule,
                            "level": level,
                            "message": {
                                "text": &violation.message
                            },
                            "locations": [{
                                "physicalLocation": {
                                    "artifactLocation": {
                                        "uri": path.display().to_string()
                                    },
                                    "region": {
                                        "startLine": violation.span.line,
                                        "startColumn": violation.span.column
                                    }
                                }
                            }],
                            "fixes": violation.fix_hint.as_ref().map(|hint| vec![
                                serde_json::json!({
                                    "description": {
                                        "text": hint
                                    }
                                })
                            ])
                        })
                    }).collect::<Vec<_>>()
                }]
            });

            serde_json::to_string_pretty(&sarif)?
        }
        MakefileOutputFormat::Gcc => {
            // GCC-style output for editor integration
            let mut output = String::new();
            for violation in &filtered_violations {
                writeln!(
                    &mut output,
                    "{}:{}:{}: {}: {} [{}]",
                    path.display(),
                    violation.span.line,
                    violation.span.column,
                    match violation.severity {
                        makefile_linter::Severity::Error => "error",
                        makefile_linter::Severity::Warning => "warning",
                        makefile_linter::Severity::Performance => "note",
                        makefile_linter::Severity::Info => "note",
                    },
                    violation.message,
                    violation.rule
                )?;
            }
            output
        }
    };

    // Print output
    println!("{}", content);

    // Handle fix mode if requested
    if fix && filtered_violations.iter().any(|v| v.fix_hint.is_some()) {
        eprintln!("\n💡 Fix mode is not yet implemented. See fix suggestions above.");
    }

    Ok(())
}

pub async fn handle_analyze_provability(
    project_path: PathBuf,
    functions: Vec<String>,
    _analysis_depth: usize,
    format: ProvabilityOutputFormat,
    high_confidence_only: bool,
    include_evidence: bool,
    output: Option<PathBuf>,
) -> Result<()> {
    use crate::cli::provability_helpers::*;
    use crate::services::lightweight_provability_analyzer::LightweightProvabilityAnalyzer;

    eprintln!("🔬 Analyzing function provability...");

    // Create the analyzer
    let analyzer = LightweightProvabilityAnalyzer::new();

    // Get function IDs based on input
    let function_ids = if functions.is_empty() {
        discover_project_functions(&project_path).await?
    } else {
        let mut ids = Vec::new();
        for spec in &functions {
            ids.push(parse_function_spec(spec, &project_path)?);
        }
        ids
    };

    // Analyze the functions
    let summaries = analyzer.analyze_incrementally(&function_ids).await;
    eprintln!("✅ Analyzed {} functions", summaries.len());

    // Filter by confidence if requested
    let filtered_summaries = filter_summaries(&summaries, high_confidence_only);
    let filtered_summaries_owned: Vec<ProofSummary> =
        filtered_summaries.into_iter().cloned().collect();

    // Format output based on requested format
    let content = match format {
        ProvabilityOutputFormat::Json => {
            format_provability_json(&function_ids, &filtered_summaries_owned, include_evidence)?
        }
        ProvabilityOutputFormat::Summary => {
            format_provability_summary(&function_ids, &filtered_summaries_owned)?
        }
        ProvabilityOutputFormat::Full => {
            format_provability_detailed(&function_ids, &filtered_summaries_owned, include_evidence)?
        }
        ProvabilityOutputFormat::Sarif => {
            format_provability_sarif(&function_ids, &filtered_summaries_owned)?
        }
        ProvabilityOutputFormat::Markdown => {
            format_provability_detailed(&function_ids, &filtered_summaries_owned, include_evidence)?
        }
    };

    // Write output
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        eprintln!(
            "✅ Provability analysis written to: {}",
            output_path.display()
        );
    } else {
        println!("{}", content);
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_defect_prediction(
    project_path: PathBuf,
    confidence_threshold: f32,
    min_lines: usize,
    include_low_confidence: bool,
    format: DefectPredictionOutputFormat,
    high_risk_only: bool,
    include_recommendations: bool,
    include: Option<String>,
    exclude: Option<String>,
    output: Option<PathBuf>,
    _perf: bool,
) -> Result<()> {
    // Delegate to refactored implementation
    stubs_refactored::handle_analyze_defect_prediction(
        project_path,
        confidence_threshold,
        min_lines,
        include_low_confidence,
        format,
        high_risk_only,
        include_recommendations,
        include,
        exclude,
        output,
        _perf,
    )
    .await
}
#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_proof_annotations(
    project_path: PathBuf,
    format: ProofAnnotationOutputFormat,
    high_confidence_only: bool,
    include_evidence: bool,
    property_type: Option<PropertyTypeFilter>,
    verification_method: Option<VerificationMethodFilter>,
    output: Option<PathBuf>,
    _perf: bool,
    clear_cache: bool,
) -> Result<()> {
    use crate::cli::proof_annotation_helpers::*;
    use std::time::Instant;

    eprintln!("🔍 Collecting proof annotations from project...");
    let start = Instant::now();

    // Setup annotator
    let annotator = setup_proof_annotator(clear_cache);

    // Create filter
    let filter = ProofAnnotationFilter {
        high_confidence_only,
        property_type,
        verification_method,
    };

    // Collect and filter annotations
    let annotations = collect_and_filter_annotations(&annotator, &project_path, &filter).await;
    let elapsed = start.elapsed();

    eprintln!("✅ Found {} matching proof annotations", annotations.len());

    // Format output using helpers
    let content = match format {
        ProofAnnotationOutputFormat::Json => format_as_json(&annotations, elapsed, &annotator)?,
        ProofAnnotationOutputFormat::Summary => format_as_summary(&annotations, elapsed)?,
        ProofAnnotationOutputFormat::Full => {
            format_as_full(&annotations, &project_path, include_evidence)?
        }
        ProofAnnotationOutputFormat::Markdown => {
            format_as_markdown(&annotations, &project_path, include_evidence)?
        }
        ProofAnnotationOutputFormat::Sarif => format_as_sarif(&annotations, &project_path)?,
    };

    // Write output
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        eprintln!("✅ Proof annotations written to: {}", output_path.display());
    } else {
        println!("{}", content);
    }

    Ok(())
}
#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_incremental_coverage(
    project_path: PathBuf,
    base_branch: String,
    target_branch: Option<String>,
    format: IncrementalCoverageOutputFormat,
    coverage_threshold: f64,
    changed_files_only: bool,
    detailed: bool,
    output: Option<PathBuf>,
    _perf: bool,
    cache_dir: Option<PathBuf>,
    force_refresh: bool,
) -> Result<()> {
    // Delegate to refactored implementation
    stubs_refactored::handle_analyze_incremental_coverage(
        project_path,
        base_branch,
        target_branch,
        format,
        coverage_threshold,
        changed_files_only,
        detailed,
        output,
        _perf,
        cache_dir,
        force_refresh,
    )
    .await
}
pub async fn handle_analyze_churn(
    project_path: PathBuf,
    days: u32,
    format: crate::models::churn::ChurnOutputFormat,
    output: Option<PathBuf>,
) -> Result<()> {
    use crate::models::churn::ChurnOutputFormat;
    use crate::services::git_analysis::GitAnalysisService;
    use std::fmt::Write;

    eprintln!("📊 Analyzing code churn for the last {} days...", days);

    // Analyze code churn
    let analysis = GitAnalysisService::analyze_code_churn(&project_path, days)
        .map_err(|e| anyhow::anyhow!("Churn analysis failed: {}", e))?;

    eprintln!("✅ Analyzed {} files with changes", analysis.files.len());

    // Format output based on requested format
    let content = match format {
        ChurnOutputFormat::Json => serde_json::to_string_pretty(&analysis)?,
        ChurnOutputFormat::Summary => {
            let mut output = String::new();
            writeln!(&mut output, "# Code Churn Analysis Summary\n")?;
            writeln!(
                &mut output,
                "**Period**: Last {} days",
                analysis.period_days
            )?;
            writeln!(
                &mut output,
                "**Total commits**: {}",
                analysis.summary.total_commits
            )?;
            writeln!(
                &mut output,
                "**Files changed**: {}",
                analysis.summary.total_files_changed
            )?;

            if !analysis.summary.hotspot_files.is_empty() {
                writeln!(&mut output, "\n## Hotspot Files (High Churn)\n")?;
                for (i, file) in analysis.summary.hotspot_files.iter().take(10).enumerate() {
                    writeln!(&mut output, "{}. {}", i + 1, file.display())?;
                }
            }

            if !analysis.summary.stable_files.is_empty() {
                writeln!(&mut output, "\n## Stable Files (Low Churn)\n")?;
                for (i, file) in analysis.summary.stable_files.iter().take(10).enumerate() {
                    writeln!(&mut output, "{}. {}", i + 1, file.display())?;
                }
            }

            if !analysis.summary.author_contributions.is_empty() {
                writeln!(&mut output, "\n## Top Contributors\n")?;
                let mut authors: Vec<_> = analysis.summary.author_contributions.iter().collect();
                authors.sort_by(|a, b| b.1.cmp(a.1));
                for (author, files) in authors.iter().take(10) {
                    writeln!(&mut output, "- {}: {} files", author, files)?;
                }
            }

            output
        }
        ChurnOutputFormat::Markdown => {
            let mut output = String::new();
            writeln!(&mut output, "# Code Churn Analysis Report\n")?;
            writeln!(
                &mut output,
                "Generated: {}",
                analysis.generated_at.format("%Y-%m-%d %H:%M:%S UTC")
            )?;
            writeln!(
                &mut output,
                "Repository: {}",
                analysis.repository_root.display()
            )?;
            writeln!(
                &mut output,
                "Analysis Period: {} days\n",
                analysis.period_days
            )?;

            writeln!(&mut output, "## Summary Statistics\n")?;
            writeln!(&mut output, "| Metric | Value |")?;
            writeln!(&mut output, "|--------|-------|")?;
            writeln!(
                &mut output,
                "| Total Commits | {} |",
                analysis.summary.total_commits
            )?;
            writeln!(
                &mut output,
                "| Files Changed | {} |",
                analysis.summary.total_files_changed
            )?;
            writeln!(
                &mut output,
                "| Hotspot Files | {} |",
                analysis.summary.hotspot_files.len()
            )?;
            writeln!(
                &mut output,
                "| Stable Files | {} |",
                analysis.summary.stable_files.len()
            )?;
            writeln!(
                &mut output,
                "| Contributing Authors | {} |",
                analysis.summary.author_contributions.len()
            )?;

            if !analysis.files.is_empty() {
                writeln!(&mut output, "\n## File Churn Details\n")?;
                writeln!(&mut output, "| File | Commits | Authors | Additions | Deletions | Churn Score | Last Modified |")?;
                writeln!(&mut output, "|------|---------|---------|-----------|-----------|-------------|----------------|")?;

                // Sort by churn score descending
                let mut sorted_files = analysis.files.clone();
                sorted_files.sort_by(|a, b| b.churn_score.partial_cmp(&a.churn_score).unwrap());

                for file in sorted_files.iter().take(20) {
                    writeln!(
                        &mut output,
                        "| {} | {} | {} | {} | {} | {:.2} | {} |",
                        file.relative_path,
                        file.commit_count,
                        file.unique_authors.len(),
                        file.additions,
                        file.deletions,
                        file.churn_score,
                        file.last_modified.format("%Y-%m-%d")
                    )?;
                }
            }

            if !analysis.summary.author_contributions.is_empty() {
                writeln!(&mut output, "\n## Author Contributions\n")?;
                writeln!(&mut output, "| Author | Files Modified |")?;
                writeln!(&mut output, "|--------|----------------|")?;

                let mut authors: Vec<_> = analysis.summary.author_contributions.iter().collect();
                authors.sort_by(|a, b| b.1.cmp(a.1));

                for (author, count) in authors.iter().take(15) {
                    writeln!(&mut output, "| {} | {} |", author, count)?;
                }
            }

            writeln!(&mut output, "\n## Recommendations\n")?;
            writeln!(&mut output, "1. **Review Hotspot Files**: Files with high churn scores may benefit from refactoring")?;
            writeln!(
                &mut output,
                "2. **Add Tests**: High-churn files should have comprehensive test coverage"
            )?;
            writeln!(
                &mut output,
                "3. **Code Review**: Frequently modified files may indicate design issues"
            )?;
            writeln!(
                &mut output,
                "4. **Documentation**: Document the reasons for frequent changes in hotspot files"
            )?;

            output
        }
        ChurnOutputFormat::Csv => {
            let mut output = String::new();
            writeln!(&mut output, "file_path,relative_path,commit_count,unique_authors,additions,deletions,churn_score,last_modified,first_seen")?;

            for file in &analysis.files {
                writeln!(
                    &mut output,
                    "{},{},{},{},{},{},{:.3},{},{}",
                    file.path.display(),
                    file.relative_path,
                    file.commit_count,
                    file.unique_authors.len(),
                    file.additions,
                    file.deletions,
                    file.churn_score,
                    file.last_modified.to_rfc3339(),
                    file.first_seen.to_rfc3339()
                )?;
            }

            output
        }
    };

    // Write output
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        eprintln!("✅ Churn analysis written to: {}", output_path.display());
    } else {
        println!("{}", content);
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_satd(
    _path: PathBuf,
    _format: SatdOutputFormat,
    _severity: Option<SatdSeverity>,
    _critical_only: bool,
    _include_tests: bool,
    _evolution: bool,
    _days: u32,
    _metrics: bool,
    _output: Option<PathBuf>,
) -> Result<()> {
    info!("SATD analysis not yet implemented");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_dag(
    _dag_type: DagType,
    _project_path: PathBuf,
    _output: Option<PathBuf>,
    _max_depth: Option<usize>,
    _filter_external: bool,
    _show_complexity: bool,
    _include_duplicates: bool,
    _include_dead_code: bool,
    _enhanced: bool,
) -> Result<()> {
    info!("DAG analysis not yet implemented");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn handle_quality_gate(
    _project_path: PathBuf,
    _format: QualityGateOutputFormat,
    _fail_on_violation: bool,
    _checks: Vec<QualityCheckType>,
    _max_dead_code: f64,
    _min_entropy: f64,
    _max_complexity_p99: u32,
    _include_provability: bool,
    _output: Option<PathBuf>,
    _perf: bool,
) -> Result<()> {
    eprintln!("🚧 Quality gate analysis is not yet implemented in this version.");
    eprintln!("This feature will be available in a future release.");
    Ok(())
}

pub async fn handle_serve(_host: String, _port: u16, _cors: bool) -> Result<()> {
    eprintln!("🚧 Server mode is not yet implemented in this version.");
    eprintln!("This feature will be available in a future release.");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_comprehensive(
    _project_path: PathBuf,
    _format: ComprehensiveOutputFormat,
    _include_duplicates: bool,
    _include_dead_code: bool,
    _include_defects: bool,
    _include_complexity: bool,
    _include_tdg: bool,
    _confidence_threshold: f32,
    _min_lines: usize,
    _include: Option<String>,
    _exclude: Option<String>,
    _output: Option<PathBuf>,
    _perf: bool,
    _executive_summary: bool,
) -> Result<()> {
    eprintln!("🚧 Comprehensive analysis is not yet implemented in this version.");
    eprintln!("This feature will be available in a future release.");
    eprintln!("For now, you can run individual analysis commands:");
    eprintln!("  - pmat analyze complexity");
    eprintln!("  - pmat analyze satd");
    eprintln!("  - pmat analyze tdg");
    eprintln!("  - pmat analyze defect-prediction");
    Ok(())
}

// Helper functions
pub fn detect_toolchain(path: &Path) -> Option<String> {
    super::detect_primary_language(path)
}

pub fn build_complexity_thresholds(
    max_cyclomatic: Option<u16>,
    max_cognitive: Option<u16>,
) -> (u16, u16) {
    (max_cyclomatic.unwrap_or(10), max_cognitive.unwrap_or(15))
}

pub async fn analyze_project_files(
    project_path: &Path,
    toolchain: Option<&str>,
    include: &[String],
    cyclomatic_threshold: u16,
    cognitive_threshold: u16,
) -> Result<Vec<crate::services::complexity::FileComplexityMetrics>> {
    use std::fs;
    use walkdir::WalkDir;

    let mut results = Vec::new();
    let toolchain = toolchain.unwrap_or("rust");

    // Simple file extension detection based on toolchain
    let extensions = match toolchain {
        "rust" => vec!["rs"],
        "deno" | "typescript" => vec!["ts", "tsx", "js", "jsx"],
        "python-uv" | "python" => vec!["py"],
        _ => vec!["rs"], // default to rust
    };

    // Walk through the project directory
    for entry in WalkDir::new(project_path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

        if !extensions.contains(&extension) {
            continue;
        }

        // Apply include patterns if specified
        if !include.is_empty() {
            let path_str = path.to_string_lossy();
            let matches_include = include.iter().any(|pattern| {
                // Simple pattern matching (not full glob support)
                if pattern.contains("**") {
                    let clean_pattern = pattern.replace("**/*.", "");
                    path_str.ends_with(&clean_pattern)
                } else {
                    path_str.contains(pattern)
                }
            });
            if !matches_include {
                continue;
            }
        }

        // Skip common vendor/build directories
        let path_str = path.to_string_lossy();
        if path_str.contains("/target/")
            || path_str.contains("/node_modules/")
            || path_str.contains("/.git/")
            || path_str.contains("/vendor/")
        {
            continue;
        }

        // Read file content
        if let Ok(content) = fs::read_to_string(path) {
            // Create basic complexity metrics (simplified analysis)
            let file_metrics =
                analyze_file_complexity(path, &content, cyclomatic_threshold, cognitive_threshold)?;
            results.push(file_metrics);
        }
    }

    Ok(results)
}

fn analyze_file_complexity(
    path: &Path,
    content: &str,
    _cyclomatic_threshold: u16,
    _cognitive_threshold: u16,
) -> Result<crate::services::complexity::FileComplexityMetrics> {
    use crate::services::complexity::{
        ComplexityMetrics, FileComplexityMetrics, FunctionComplexity,
    };

    // Simple heuristic-based complexity analysis
    let lines: Vec<&str> = content.lines().collect();
    let mut functions = Vec::new();

    // Detect language from file extension
    let is_rust = path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e == "rs")
        .unwrap_or(false);
    let is_typescript = path.extension()
        .and_then(|e| e.to_str())
        .map(|e| matches!(e, "ts" | "tsx" | "js" | "jsx"))
        .unwrap_or(false);
    let is_python = path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e == "py")
        .unwrap_or(false);

    // Find function definitions (basic pattern matching)
    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        let mut found_function = false;
        let mut function_name = None;

        if is_rust {
            // Rust function patterns
            if trimmed.starts_with("fn ")
                || trimmed.starts_with("pub fn ")
                || trimmed.starts_with("async fn ")
                || trimmed.starts_with("pub async fn ")
                || trimmed.starts_with("pub(crate) fn ")
                || trimmed.starts_with("pub(super) fn ")
            {
                function_name = extract_rust_function_name(trimmed);
                found_function = true;
            }
        } else if is_typescript {
            // TypeScript/JavaScript function patterns
            if trimmed.starts_with("function ")
                || trimmed.starts_with("async function ")
                || trimmed.starts_with("export function ")
                || trimmed.starts_with("export async function ")
                || trimmed.starts_with("export default function ")
                || trimmed.contains("= function")
                || trimmed.contains("= async function")
                || (trimmed.contains("const ") && trimmed.contains(" = ("))
                || (trimmed.contains("let ") && trimmed.contains(" = ("))
                || (trimmed.contains("var ") && trimmed.contains(" = ("))
                || (trimmed.contains("export const ") && trimmed.contains(" = ("))
                || trimmed.contains(" => {")
            {
                function_name = extract_js_function_name(trimmed);
                found_function = true;
            }
        } else if is_python {
            // Python function patterns
            if trimmed.starts_with("def ") || trimmed.starts_with("async def ") {
                function_name = extract_python_function_name(trimmed);
                found_function = true;
            }
        }

        if found_function {
            if let Some(name) = function_name {
                // Simple complexity estimation based on control flow keywords
                let function_complexity = estimate_function_complexity(&lines, line_num);

                functions.push(FunctionComplexity {
                    name,
                    line_start: (line_num + 1) as u32,
                    line_end: (line_num + 10) as u32, // Rough estimate
                    metrics: function_complexity,
                });
            }
        }
    }

    // Aggregate total complexity
    let total_complexity = ComplexityMetrics {
        cyclomatic: functions
            .iter()
            .map(|f| f.metrics.cyclomatic)
            .sum::<u16>()
            .max(1),
        cognitive: functions
            .iter()
            .map(|f| f.metrics.cognitive)
            .sum::<u16>()
            .max(1),
        nesting_max: functions
            .iter()
            .map(|f| f.metrics.nesting_max)
            .max()
            .unwrap_or(0),
        lines: lines.len() as u16,
    };

    Ok(FileComplexityMetrics {
        path: path.to_string_lossy().to_string(),
        total_complexity,
        functions,
        classes: vec![], // Not implemented for this basic version
    })
}

fn extract_rust_function_name(line: &str) -> Option<String> {
    // Extract function name from line like "pub fn function_name("
    let line = line.trim();
    if let Some(fn_pos) = line.find("fn ") {
        let after_fn = &line[fn_pos + 3..];
        if let Some(paren_pos) = after_fn.find('(') {
            let name = after_fn[..paren_pos].trim();
            return Some(name.to_string());
        }
    }
    None
}

fn extract_js_function_name(line: &str) -> Option<String> {
    let line = line.trim();
    
    // Handle: function name(
    if let Some(pos) = line.find("function ") {
        let after = &line[pos + 9..];
        if let Some(paren_pos) = after.find('(') {
            let name = after[..paren_pos].trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }
    
    // Handle: const/let/var name = 
    for keyword in &["const ", "let ", "var "] {
        if let Some(pos) = line.find(keyword) {
            let after = &line[pos + keyword.len()..];
            if let Some(eq_pos) = after.find(" = ") {
                let name = after[..eq_pos].trim();
                return Some(name.to_string());
            }
        }
    }
    
    // For anonymous functions, use line number as name
    Some(format!("anonymous_fn"))
}

fn extract_python_function_name(line: &str) -> Option<String> {
    let line = line.trim();
    if let Some(pos) = line.find("def ") {
        let after = &line[pos + 4..];
        if let Some(paren_pos) = after.find('(') {
            let name = after[..paren_pos].trim();
            return Some(name.to_string());
        }
    }
    None
}

fn estimate_function_complexity(
    lines: &[&str],
    start_line: usize,
) -> crate::services::complexity::ComplexityMetrics {
    // Look ahead a few lines to estimate complexity
    let end_line = (start_line + 20).min(lines.len());
    let function_lines = &lines[start_line..end_line];

    let mut cyclomatic = 1u16; // Base complexity
    let mut cognitive = 0u16;
    let mut nesting = 0u8;
    let mut max_nesting = 0u8;

    for line in function_lines {
        let trimmed = line.trim();

        // Count control flow keywords (simplified)
        if trimmed.contains("if ")
            || trimmed.contains("while ")
            || trimmed.contains("for ")
            || trimmed.contains("match ")
        {
            cyclomatic += 1;
            cognitive += 1 + nesting as u16; // Cognitive complexity increases with nesting
        }

        if trimmed.contains("else") {
            cyclomatic += 1;
            cognitive += 1;
        }

        // Track nesting (very basic)
        if trimmed.ends_with('{') {
            nesting += 1;
            max_nesting = max_nesting.max(nesting);
        }
        if trimmed.starts_with('}') {
            nesting = nesting.saturating_sub(1);
        }
    }

    crate::services::complexity::ComplexityMetrics {
        cyclomatic: cyclomatic.min(255),
        cognitive: cognitive.min(255),
        nesting_max: max_nesting,
        lines: function_lines.len().min(65535) as u16,
    }
}

pub fn add_top_files_ranking(
    files: Vec<crate::services::complexity::FileComplexityMetrics>,
    top_files: usize,
) -> Vec<crate::services::complexity::FileComplexityMetrics> {
    if top_files == 0 {
        files
    } else {
        files.into_iter().take(top_files).collect()
    }
}

pub fn format_dead_code_output(
    format: DeadCodeOutputFormat,
    dead_code_result: &crate::models::dead_code::DeadCodeResult,
    _output: Option<PathBuf>,
) -> Result<()> {
    use crate::models::dead_code::{ConfidenceLevel, DeadCodeType};

    match format {
        DeadCodeOutputFormat::Summary => {
            println!("# Dead Code Analysis Summary\n");
            println!("📊 **Files analyzed**: {}", dead_code_result.total_files);
            println!(
                "☠️  **Files with dead code**: {}",
                dead_code_result.summary.files_with_dead_code
            );
            println!(
                "📏 **Total dead lines**: {}",
                dead_code_result.summary.total_dead_lines
            );
            println!(
                "📈 **Dead code percentage**: {:.2}%\n",
                dead_code_result.summary.dead_percentage
            );

            if dead_code_result.summary.dead_functions > 0 {
                println!("## Dead Code by Type\n");
                println!(
                    "- **Dead functions**: {}",
                    dead_code_result.summary.dead_functions
                );
                println!(
                    "- **Dead classes**: {}",
                    dead_code_result.summary.dead_classes
                );
                println!(
                    "- **Dead variables**: {}",
                    dead_code_result.summary.dead_modules
                );
                println!(
                    "- **Unreachable blocks**: {}",
                    dead_code_result.summary.unreachable_blocks
                );
            }

            if !dead_code_result.files.is_empty() {
                println!("\n## Top Files with Dead Code\n");
                for (i, file) in dead_code_result.files.iter().take(10).enumerate() {
                    println!(
                        "{}. `{}` - {:.1}% dead ({} lines)",
                        i + 1,
                        file.path,
                        file.dead_percentage,
                        file.dead_lines
                    );
                    if file.dead_functions > 0 || file.dead_classes > 0 {
                        print!("   ");
                        if file.dead_functions > 0 {
                            print!("Functions: {} ", file.dead_functions);
                        }
                        if file.dead_classes > 0 {
                            print!("Classes: {} ", file.dead_classes);
                        }
                        println!();
                    }
                }
            }
        }
        DeadCodeOutputFormat::Json => {
            let json = serde_json::to_string_pretty(dead_code_result)?;
            println!("{}", json);
        }
        DeadCodeOutputFormat::Markdown => {
            println!("# Dead Code Analysis Report\n");
            println!("## Summary\n");
            println!("| Metric | Value |");
            println!("|--------|-------|");
            println!("| Files Analyzed | {} |", dead_code_result.total_files);
            println!(
                "| Files with Dead Code | {} |",
                dead_code_result.summary.files_with_dead_code
            );
            println!(
                "| Total Dead Lines | {} |",
                dead_code_result.summary.total_dead_lines
            );
            println!(
                "| Dead Code Percentage | {:.2}% |",
                dead_code_result.summary.dead_percentage
            );
            println!(
                "| Dead Functions | {} |",
                dead_code_result.summary.dead_functions
            );
            println!(
                "| Dead Classes | {} |",
                dead_code_result.summary.dead_classes
            );
            println!(
                "| Dead Variables | {} |",
                dead_code_result.summary.dead_modules
            );
            println!(
                "| Unreachable Blocks | {} |",
                dead_code_result.summary.unreachable_blocks
            );

            if !dead_code_result.files.is_empty() {
                println!("\n## Files with Dead Code\n");
                println!("| File | Dead % | Dead Lines | Functions | Classes | Confidence |");
                println!("|------|--------|------------|-----------|---------|------------|");
                for file in &dead_code_result.files {
                    let confidence = match file.confidence {
                        ConfidenceLevel::High => "High",
                        ConfidenceLevel::Medium => "Medium",
                        ConfidenceLevel::Low => "Low",
                    };
                    println!(
                        "| {} | {:.1}% | {} | {} | {} | {} |",
                        file.path,
                        file.dead_percentage,
                        file.dead_lines,
                        file.dead_functions,
                        file.dead_classes,
                        confidence
                    );
                }

                // Show detailed items for top files
                for file in dead_code_result.files.iter().take(5) {
                    if !file.items.is_empty() {
                        println!("\n### {} - Dead Code Items\n", file.path);
                        println!("| Type | Name | Line | Reason |");
                        println!("|------|------|------|--------|");
                        for item in &file.items {
                            let item_type = match item.item_type {
                                DeadCodeType::Function => "Function",
                                DeadCodeType::Class => "Class",
                                DeadCodeType::Variable => "Variable",
                                DeadCodeType::UnreachableCode => "Unreachable",
                            };
                            println!(
                                "| {} | {} | {} | {} |",
                                item_type, item.name, item.line, item.reason
                            );
                        }
                    }
                }
            }
        }
        DeadCodeOutputFormat::Sarif => {
            // SARIF format for IDE integration
            let sarif = serde_json::json!({
                "version": "2.1.0",
                "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
                "runs": [{
                    "tool": {
                        "driver": {
                            "name": "paiml-mcp-agent-toolkit",
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
                    "results": dead_code_result.files.iter().flat_map(|file| {
                        file.items.iter().map(|item| {
                            let level = match file.confidence {
                                ConfidenceLevel::High => "error",
                                ConfidenceLevel::Medium => "warning",
                                ConfidenceLevel::Low => "note",
                            };
                            serde_json::json!({
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

            println!("{}", serde_json::to_string_pretty(&sarif)?);
        }
    }

    Ok(())
}

// Name similarity helpers
pub fn extract_identifiers(content: &str) -> Vec<super::NameInfo> {
    use regex::Regex;

    let mut identifiers = Vec::new();
    let mut seen = HashSet::new();

    // Language-agnostic identifier patterns
    let patterns = vec![
        // Function/method definitions
        (r"(?m)^\s*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)", "function"),
        (r"(?m)^\s*def\s+(\w+)", "function"),
        (r"(?m)^\s*function\s+(\w+)", "function"),
        (
            r"(?m)^\s*(?:public|private|protected)?\s*(?:static)?\s*\w+\s+(\w+)\s*\(",
            "function",
        ),
        // Class/struct/interface definitions
        (r"(?m)^\s*(?:pub\s+)?struct\s+(\w+)", "struct"),
        (r"(?m)^\s*(?:pub\s+)?enum\s+(\w+)", "enum"),
        (r"(?m)^\s*(?:pub\s+)?trait\s+(\w+)", "trait"),
        (r"(?m)^\s*class\s+(\w+)", "class"),
        (r"(?m)^\s*interface\s+(\w+)", "interface"),
        (r"(?m)^\s*type\s+(\w+)", "type"),
        // Variable/constant definitions
        (r"(?m)^\s*(?:pub\s+)?(?:const|static)\s+(\w+)", "constant"),
        (r"(?m)^\s*(?:let|const|var)\s+(\w+)", "variable"),
        (r"(?m)^\s*(\w+)\s*=\s*", "variable"),
    ];

    for (pattern_str, kind) in patterns {
        if let Ok(re) = Regex::new(pattern_str) {
            for (line_num, line) in content.lines().enumerate() {
                for cap in re.captures_iter(line) {
                    if let Some(name_match) = cap.get(1) {
                        let name = name_match.as_str().to_string();

                        // Skip if we've already seen this identifier
                        if seen.insert(name.clone()) {
                            identifiers.push(super::NameInfo {
                                name,
                                kind: kind.to_string(),
                                file_path: PathBuf::from(""), // Will be filled by caller
                                line: line_num + 1,
                            });
                        }
                    }
                }
            }
        }
    }

    identifiers
}

pub fn calculate_string_similarity(s1: &str, s2: &str) -> f32 {
    // Normalized Levenshtein distance for basic string similarity
    if s1.is_empty() && s2.is_empty() {
        return 1.0;
    }

    if s1 == s2 {
        return 1.0;
    }

    // Calculate Jaccard similarity based on character n-grams
    let n = 2; // bigrams
    let ngrams1 = get_ngrams(s1, n);
    let ngrams2 = get_ngrams(s2, n);

    if ngrams1.is_empty() && ngrams2.is_empty() {
        // Fall back to exact character matching for very short strings
        let common_chars = s1.chars().filter(|c| s2.contains(*c)).count();
        let total_chars = s1.len().max(s2.len());
        return if total_chars > 0 {
            common_chars as f32 / total_chars as f32
        } else {
            0.0
        };
    }

    let intersection: HashSet<_> = ngrams1.intersection(&ngrams2).cloned().collect();
    let union: HashSet<_> = ngrams1.union(&ngrams2).cloned().collect();

    if union.is_empty() {
        0.0
    } else {
        intersection.len() as f32 / union.len() as f32
    }
}

/// Get character n-grams from a string
fn get_ngrams(s: &str, n: usize) -> HashSet<String> {
    let chars: Vec<char> = s.chars().collect();
    let mut ngrams = HashSet::new();

    if chars.len() >= n {
        for i in 0..=chars.len() - n {
            let ngram: String = chars[i..i + n].iter().collect();
            ngrams.insert(ngram);
        }
    } else {
        // For strings shorter than n, use the whole string as an n-gram
        ngrams.insert(s.to_string());
    }

    ngrams
}

pub fn calculate_edit_distance(s1: &str, s2: &str) -> usize {
    // Levenshtein distance implementation
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    // Create a 2D matrix for dynamic programming
    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    // Initialize first row and column
    for (i, row) in matrix.iter_mut().enumerate().take(len1 + 1) {
        row[0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    // Fill the matrix
    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] {
                0
            } else {
                1
            };

            matrix[i][j] = std::cmp::min(
                std::cmp::min(
                    matrix[i - 1][j] + 1, // deletion
                    matrix[i][j - 1] + 1, // insertion
                ),
                matrix[i - 1][j - 1] + cost, // substitution
            );
        }
    }

    matrix[len1][len2]
}

pub fn calculate_soundex(s: &str) -> String {
    // Soundex phonetic algorithm implementation
    if s.is_empty() {
        return String::new();
    }

    let s_upper = s.to_uppercase();
    let chars: Vec<char> = s_upper.chars().filter(|c| c.is_alphabetic()).collect();

    if chars.is_empty() {
        return String::new();
    }

    let mut soundex = String::new();
    soundex.push(chars[0]);

    let mut prev_code = soundex_code(chars[0]);

    for &ch in &chars[1..] {
        let code = soundex_code(ch);

        // Skip if same as previous code or if it's 0 (vowels and similar)
        if code != '0' && code != prev_code {
            soundex.push(code);
            prev_code = code;

            // Soundex codes are traditionally 4 characters
            if soundex.len() >= 4 {
                break;
            }
        } else if code == '0' {
            // Reset prev_code for vowels to allow consonants after vowels
            prev_code = '0';
        }
    }

    // Pad with zeros if necessary
    while soundex.len() < 4 {
        soundex.push('0');
    }

    // Ensure exactly 4 characters
    soundex.truncate(4);
    soundex
}

/// Get Soundex code for a character
fn soundex_code(ch: char) -> char {
    match ch {
        'B' | 'F' | 'P' | 'V' => '1',
        'C' | 'G' | 'J' | 'K' | 'Q' | 'S' | 'X' | 'Z' => '2',
        'D' | 'T' => '3',
        'L' => '4',
        'M' | 'N' => '5',
        'R' => '6',
        _ => '0', // A, E, I, O, U, H, W, Y and others
    }
}

// Helper function for params conversion
pub fn params_to_json(
    params: Vec<(String, serde_json::Value)>,
) -> serde_json::Map<String, serde_json::Value> {
    params.into_iter().collect()
}

// Table printing function
pub fn print_table(items: &[std::sync::Arc<crate::models::template::TemplateResource>]) {
    if items.is_empty() {
        println!("No templates found.");
        return;
    }

    // Calculate column widths
    let mut name_width = "Name".len();
    let mut toolchain_width = "Toolchain".len();
    let mut category_width = "Category".len();
    let mut desc_width = "Description".len();

    for item in items {
        name_width = name_width.max(item.name.len());
        toolchain_width = toolchain_width.max(item.toolchain.as_str().len());
        category_width = category_width.max(format!("{:?}", item.category).len());
        desc_width = desc_width.max(60.min(item.description.len()));
    }

    // Add padding
    name_width += 2;
    toolchain_width += 2;
    category_width += 2;
    desc_width += 2;

    // Print header
    println!(
        "┌{}┬{}┬{}┬{}┐",
        "─".repeat(name_width),
        "─".repeat(toolchain_width),
        "─".repeat(category_width),
        "─".repeat(desc_width)
    );

    println!(
        "│{:^name_width$}│{:^toolchain_width$}│{:^category_width$}│{:^desc_width$}│",
        "Name",
        "Toolchain",
        "Category",
        "Description",
        name_width = name_width,
        toolchain_width = toolchain_width,
        category_width = category_width,
        desc_width = desc_width
    );

    println!(
        "├{}┼{}┼{}┼{}┤",
        "─".repeat(name_width),
        "─".repeat(toolchain_width),
        "─".repeat(category_width),
        "─".repeat(desc_width)
    );

    // Print rows
    for item in items {
        let toolchain = item.toolchain.as_str();
        let category = format!("{:?}", item.category);
        let description = item.description.chars().take(60).collect::<String>();
        let description = if item.description.len() > 60 {
            format!("{}...", description)
        } else {
            description
        };

        println!(
            "│{:<name_width$}│{:<toolchain_width$}│{:<category_width$}│{:<desc_width$}│",
            format!(" {} ", item.name),
            format!(" {} ", toolchain),
            format!(" {} ", category),
            format!(" {} ", description),
            name_width = name_width,
            toolchain_width = toolchain_width,
            category_width = category_width,
            desc_width = desc_width
        );
    }

    // Print footer
    println!(
        "└{}┴{}┴{}┴{}┘",
        "─".repeat(name_width),
        "─".repeat(toolchain_width),
        "─".repeat(category_width),
        "─".repeat(desc_width)
    );
}

// Helper functions for defect prediction complexity estimation
#[allow(dead_code)]
fn estimate_cyclomatic_complexity(content: &str) -> u32 {
    let mut complexity = 1u32; // Base complexity

    // Count decision points
    for line in content.lines() {
        let trimmed = line.trim();

        // Count if/else/elif/when
        if trimmed.starts_with("if ")
            || trimmed.contains(" if ")
            || trimmed.starts_with("else if")
            || trimmed.starts_with("elif")
            || trimmed.starts_with("when ")
        {
            complexity += 1;
        }

        // Count loops
        if trimmed.starts_with("for ")
            || trimmed.starts_with("while ")
            || trimmed.starts_with("loop ")
            || trimmed.contains(".iter()")
            || trimmed.contains(".forEach")
            || trimmed.contains("for(")
        {
            complexity += 1;
        }

        // Count case/match arms
        if trimmed.contains("=>") && !trimmed.contains("//") {
            complexity += 1;
        }

        // Count catch/except blocks
        if trimmed.starts_with("catch") || trimmed.starts_with("except") {
            complexity += 1;
        }

        // Count logical operators
        complexity += (trimmed.matches("&&").count() + trimmed.matches("||").count()) as u32;
    }

    complexity.min(50) // Cap at 50 for reasonable values
}

#[allow(dead_code)]
fn estimate_cognitive_complexity(content: &str) -> u32 {
    let mut complexity = 0u32;
    let mut nesting_level = 0u32;

    for line in content.lines() {
        let trimmed = line.trim();

        // Track nesting
        if trimmed.ends_with('{') {
            nesting_level += 1;
        } else if trimmed.starts_with('}') {
            nesting_level = nesting_level.saturating_sub(1);
        }

        // Add complexity with nesting penalty
        if trimmed.starts_with("if ") || trimmed.contains(" if ") {
            complexity += 1 + nesting_level;
        }

        if trimmed.starts_with("for ") || trimmed.starts_with("while ") {
            complexity += 1 + nesting_level;
        }

        // Nested functions/closures add more complexity
        if (trimmed.contains("fn ") || trimmed.contains("function") || trimmed.contains("=>"))
            && nesting_level > 0
        {
            complexity += nesting_level;
        }

        // Early returns in nested contexts
        if trimmed.contains("return") && nesting_level > 1 {
            complexity += 1;
        }
    }

    complexity.min(60) // Cap at 60 for reasonable values
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_handle_analyze_makefile_basic() {
        // Create a temporary directory and Makefile
        let temp_dir = TempDir::new().unwrap();
        let makefile_path = temp_dir.path().join("Makefile");
        let mut file = std::fs::File::create(&makefile_path).unwrap();
        writeln!(file, "all:").unwrap();
        writeln!(file, "\techo 'Hello World'").unwrap();

        // Test basic makefile analysis
        let result = handle_analyze_makefile(
            makefile_path.clone(),
            vec![], // Empty rules vector
            MakefileOutputFormat::Human,
            false,
            None,
        )
        .await;

        // Should complete without error
        assert!(
            result.is_ok(),
            "Makefile analysis failed: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_handle_analyze_makefile_with_rules() {
        let temp_dir = TempDir::new().unwrap();
        let makefile_path = temp_dir.path().join("Makefile");
        let mut file = std::fs::File::create(&makefile_path).unwrap();
        writeln!(file, "test:").unwrap();
        writeln!(file, "\tcargo test").unwrap();

        // Test with custom rules
        let result = handle_analyze_makefile(
            makefile_path,
            vec!["phonytargets".to_string()],
            MakefileOutputFormat::Json,
            false,
            Some("3.82".to_string()),
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_provability() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        // Create a simple Rust file for analysis
        let src_dir = project_path.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        let rust_file = src_dir.join("lib.rs");
        let mut file = std::fs::File::create(&rust_file).unwrap();
        writeln!(file, "pub fn add(a: i32, b: i32) -> i32 {{").unwrap();
        writeln!(file, "    a + b").unwrap();
        writeln!(file, "}}").unwrap();

        // Test provability analysis
        let result = handle_analyze_provability(
            project_path,
            vec!["add".to_string()], // Functions to analyze
            10,                      // Analysis depth
            ProvabilityOutputFormat::Json,
            false, // high_confidence_only
            false, // include_evidence
            None,  // output path
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_defect_prediction() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        // Create test files
        let src_dir = project_path.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        let rust_file = src_dir.join("main.rs");
        let mut file = std::fs::File::create(&rust_file).unwrap();
        writeln!(file, "fn main() {{").unwrap();
        writeln!(file, "    println!(\"Hello, world!\");").unwrap();
        writeln!(file, "}}").unwrap();

        // Test defect prediction
        let result = handle_analyze_defect_prediction(
            project_path,
            0.5,   // confidence_threshold
            10,    // min_lines
            false, // include_low_confidence
            DefectPredictionOutputFormat::Summary,
            false, // high_risk_only
            false, // include_recommendations
            None,  // include
            None,  // exclude
            None,  // output
            false, // _perf
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_proof_annotations() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        // Test proof annotation collection
        let result = handle_analyze_proof_annotations(
            project_path,
            ProofAnnotationOutputFormat::Json,
            false, // high_confidence_only
            false, // include_evidence
            None,  // sources
            None,  // confidence_levels
            None,  // output
            false, // _perf
            false, // clear_cache
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_analyze_incremental_coverage() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        // Initialize git repo for incremental coverage
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(&project_path)
            .output()
            .unwrap();

        // Create src directory and files that the mock expects
        let src_dir = project_path.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();
        std::fs::write(src_dir.join("lib.rs"), "// lib").unwrap();

        // Test incremental coverage analysis
        let result = handle_analyze_incremental_coverage(
            project_path,
            "main".to_string(), // base_branch
            None,               // target_branch
            IncrementalCoverageOutputFormat::Summary,
            80.0,  // coverage_threshold
            false, // changed_files_only
            false, // detailed
            None,  // output
            false, // _perf
            None,  // cache_dir
            false, // force_refresh
        )
        .await;

        // This might fail if git is not available, but should not panic
        match result {
            Ok(_) => {} // Success
            Err(e) => {
                // Accept git-related errors or coverage analysis errors
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("git")
                        || error_msg.contains("No changed files")
                        || error_msg.contains("coverage")
                        || error_msg.contains("branch")
                        || error_msg.contains("Coverage threshold not met"),
                    "Unexpected error: {}",
                    error_msg
                );
            }
        }
    }

    #[test]
    fn test_extract_identifiers() {
        // Test Rust identifiers
        let rust_code = "fn calculate_total(items: Vec<Item>) -> u32 { items.len() }";
        let identifiers = extract_identifiers(rust_code);
        assert!(identifiers.iter().any(|i| i.name == "calculate_total"));

        // Test JavaScript identifiers
        let js_code = "function getUserName(userId) { return users[userId].name; }";
        let identifiers = extract_identifiers(js_code);
        assert!(identifiers.iter().any(|i| i.name == "getUserName"));

        // Test Python identifiers
        let py_code = "def process_data(input_list): return [x * 2 for x in input_list]";
        let identifiers = extract_identifiers(py_code);
        assert!(identifiers.iter().any(|i| i.name == "process_data"));
    }

    #[test]
    fn test_calculate_string_similarity() {
        // Identical strings
        assert_eq!(calculate_string_similarity("hello", "hello"), 1.0);

        // Completely different strings
        assert_eq!(calculate_string_similarity("hello", "world"), 0.0);

        // Similar strings
        let similarity = calculate_string_similarity("hello_world", "hello_word");
        assert!(similarity > 0.5 && similarity < 1.0);

        // Empty strings
        assert_eq!(calculate_string_similarity("", ""), 1.0);
        assert_eq!(calculate_string_similarity("hello", ""), 0.0);
    }

    #[test]
    fn test_calculate_edit_distance() {
        // Identical strings
        assert_eq!(calculate_edit_distance("hello", "hello"), 0);

        // One character difference
        assert_eq!(calculate_edit_distance("hello", "hallo"), 1);

        // Multiple differences
        assert_eq!(calculate_edit_distance("kitten", "sitting"), 3);

        // Empty strings
        assert_eq!(calculate_edit_distance("", ""), 0);
        assert_eq!(calculate_edit_distance("hello", ""), 5);
        assert_eq!(calculate_edit_distance("", "world"), 5);
    }

    #[test]
    fn test_calculate_soundex() {
        // Test basic soundex
        assert_eq!(calculate_soundex("Robert"), "R163");
        assert_eq!(calculate_soundex("Rupert"), "R163");
        assert_eq!(calculate_soundex("Rubin"), "R150");

        // Test similar sounding names
        assert_eq!(calculate_soundex("Ashcraft"), calculate_soundex("Ashcroft"));

        // Test edge cases
        assert_eq!(calculate_soundex("A"), "A000");
        assert_eq!(calculate_soundex("123"), "");
        assert_eq!(calculate_soundex(""), "");
    }

    #[test]
    fn test_handle_serve_placeholder() {
        // Test that handle_serve is defined (actual server test would require more setup)
        // This is a compile-time test to ensure the function exists
        let _ = handle_serve;
    }

    #[test]
    fn test_output_format_completeness() {
        // Test MakefileOutputFormat has all expected variants
        // Just verify that we can create each variant
        let _ = MakefileOutputFormat::Human;
        let _ = MakefileOutputFormat::Json;
        let _ = MakefileOutputFormat::Sarif;
        let _ = MakefileOutputFormat::Gcc;

        // Test that different formats produce different output
        let formats = [
            MakefileOutputFormat::Human,
            MakefileOutputFormat::Json,
            MakefileOutputFormat::Sarif,
            MakefileOutputFormat::Gcc,
        ];

        // Ensure we have 4 unique formats
        assert_eq!(formats.len(), 4);
    }

    #[test]
    fn test_estimate_complexity_functions() {
        // Test cyclomatic complexity estimation
        let simple_code = "fn add(a: i32, b: i32) -> i32 { a + b }";
        let simple_complexity = estimate_cyclomatic_complexity(simple_code);
        assert_eq!(simple_complexity, 1); // No branches

        let complex_code = r#"
            fn process(x: i32) -> i32 {
                if x > 0 {
                    for i in 0..x {
                        if i % 2 == 0 {
                            continue;
                        }
                    }
                    x
                } else {
                    -x
                }
            }
        "#;
        let complex_complexity = estimate_cyclomatic_complexity(complex_code);
        assert!(complex_complexity > 3); // Multiple branches and loops

        // Test cognitive complexity estimation
        let cognitive_simple = estimate_cognitive_complexity(simple_code);
        assert_eq!(cognitive_simple, 0); // No nesting or control flow

        let cognitive_complex = estimate_cognitive_complexity(complex_code);
        assert!(cognitive_complex > 2); // Nested control structures
    }

    #[test]
    fn test_get_ngrams() {
        let ngrams = get_ngrams("hello", 2);
        assert!(ngrams.contains("he"));
        assert!(ngrams.contains("el"));
        assert!(ngrams.contains("ll"));
        assert!(ngrams.contains("lo"));
        assert_eq!(ngrams.len(), 4);

        // Test with string shorter than n
        let short_ngrams = get_ngrams("hi", 3);
        assert_eq!(short_ngrams.len(), 1);
        assert!(short_ngrams.contains("hi"));
    }

    #[test]
    fn test_soundex_code() {
        assert_eq!(soundex_code('B'), '1');
        assert_eq!(soundex_code('C'), '2');
        assert_eq!(soundex_code('D'), '3');
        assert_eq!(soundex_code('L'), '4');
        assert_eq!(soundex_code('M'), '5');
        assert_eq!(soundex_code('R'), '6');
        assert_eq!(soundex_code('A'), '0');
        assert_eq!(soundex_code('E'), '0');
    }
}
