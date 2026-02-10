// TDG handlers - extracted for file health (CB-040)
/// Analyzes Technical Debt Gradient (TDG) for a project.
///
/// Technical Debt Gradient measures the rate of technical debt accumulation
/// relative to code complexity and change frequency. Critical for identifying
/// files that are both complex and frequently modified, indicating high
/// maintenance burden and defect risk.
///
/// # Parameters
///
/// * `path` - Root directory of the project to analyze
/// * `threshold` - TDG threshold above which files are considered problematic
/// * `top` - Number of top TDG violating files to report
/// * `format` - Output format for the TDG analysis results
/// * `include_components` - Whether to include component-level TDG breakdown
/// * `output` - Optional output file path
/// * `critical_only` - Only report files above critical TDG threshold
/// * `verbose` - Include detailed TDG calculation methodology
///
/// # Returns
///
/// * `Ok(())` - TDG analysis completed successfully
/// * `Err(anyhow::Error)` - Analysis failed (file access, calculation, or output)
///
/// # TDG Calculation
///
/// TDG = (Complexity Score × Churn Frequency) / Code Size
///
/// Where:
/// - **Complexity Score**: Cyclomatic complexity + cognitive complexity
/// - **Churn Frequency**: Git commits per file over analysis period
/// - **Code Size**: Lines of code normalization factor
///
/// # Interpretation
///
/// - **TDG < 0.5**: Well-maintained, low-risk files
/// - **0.5 ≤ TDG < 1.0**: Moderate technical debt, monitor
/// - **1.0 ≤ TDG < 2.0**: High technical debt, prioritize refactoring
/// - **TDG ≥ 2.0**: Critical technical debt, immediate attention required
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::cli::analysis_utilities::handle_analyze_tdg;
/// use pmat::cli::TdgOutputFormat;
/// use std::path::{Path, PathBuf};
/// use tempfile::tempdir;
/// use std::fs;
///
/// # tokio_test::block_on(async {
/// // Create a temporary project
/// let dir = tempdir().unwrap();
/// let main_rs = dir.path().join("main.rs");
/// fs::write(&main_rs, "fn complex_function() { /* complex code */ }").unwrap();
///
/// // Standard TDG analysis
/// let result = handle_analyze_tdg(
///     dir.path().to_path_buf(),
///     None,  // file - project mode
///     vec![], // files - project mode
///     1.0,  // threshold
///     10,   // top files
///     TdgOutputFormat::Table,
///     false, // no component breakdown
///     None,  // stdout output
///     false, // all files
///     false, // normal verbosity
///     vec![], // include patterns
///     false, // watch mode
/// ).await;
///
/// assert!(result.is_ok());
///
/// // Critical TDG analysis with detailed output
/// let critical_result = handle_analyze_tdg(
///     dir.path().to_path_buf(),
///     None,  // file - project mode
///     vec![], // files - project mode
///     2.0,  // critical threshold
///     5,    // top 5 files
///     TdgOutputFormat::Json,
///     true,  // include components
///     Some(dir.path().join("tdg-report.txt")),
///     true,  // critical only
///     true,  // verbose
///     vec![], // include patterns
///     false, // watch mode
/// ).await;
///
/// assert!(critical_result.is_ok());
/// # });
/// ```
///
/// # CLI Usage Examples
///
/// ```bash
/// # Standard TDG analysis
/// pmat analyze tdg /path/to/project --threshold 1.0 --top-files 10
///
/// # Critical debt identification
/// pmat analyze tdg /path/to/project --threshold 2.0 --critical-only \
///   --format full --output critical-debt.txt
///
/// # Component-level TDG analysis
/// pmat analyze tdg /path/to/project --include-components --verbose \
///   --format json --output tdg-detailed.json
/// ```ignore
#[cfg(feature = "watch")]
/// Helper function to perform TDG analysis without watch mode
#[allow(clippy::too_many_arguments)]
async fn perform_tdg_analysis(
    calculator: &crate::services::tdg_calculator::TDGCalculator,
    path: &Path,
    threshold: f64,
    top: usize,
    format: &TdgOutputFormat,
    include_components: bool,
    output: &Option<PathBuf>,
    critical_only: bool,
    verbose: bool,
) -> Result<()> {
    // Reuse the main analysis logic
    let output_content = analyze_multiple_files(
        calculator,
        path,
        vec![], // Empty files list for project mode
        threshold,
        top,
        format.clone(),
        include_components,
        critical_only,
        verbose,
    )
    .await?;

    if let Some(output_path) = output {
        std::fs::write(output_path, output_content)?;
        eprintln!("✅ TDG analysis saved to {}", output_path.display());
    } else {
        print!("{output_content}");
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_tdg(
    path: PathBuf,
    file: Option<PathBuf>,
    files: Vec<PathBuf>,
    threshold: f64,
    top: usize,
    format: TdgOutputFormat,
    _include_components: bool,
    output: Option<PathBuf>,
    _critical_only: bool,
    _verbose: bool,
    include: Vec<String>,
    watch: bool,
) -> Result<()> {
    use crate::services::tdg_calculator::TDGCalculator;

    if watch {
        #[cfg(feature = "watch")]
        {
            return run_tdg_watch_mode(
                path,
                threshold,
                top,
                format,
                _include_components,
                output,
                _critical_only,
                _verbose,
            )
            .await;
        }
        #[cfg(not(feature = "watch"))]
        {
            anyhow::bail!("Watch mode requires the 'watch' feature. Rebuild with: cargo build --features watch");
        }
    }

    eprintln!("🔍 Analyzing Technical Debt Gradient...");

    // Create TDG calculator
    let calculator = TDGCalculator::new();

    // Determine analysis mode and generate output
    let output_content = run_tdg_analysis(
        &calculator,
        &path,
        file,
        files,
        include,
        threshold,
        top,
        format,
        _include_components,
        _critical_only,
        _verbose,
    )
    .await?;

    // Output results
    write_tdg_output(output, &output_content).await?;

    eprintln!("✅ TDG analysis complete");
    Ok(())
}

#[cfg(feature = "watch")]
/// Run TDG analysis in watch mode
#[allow(clippy::too_many_arguments)]
async fn run_tdg_watch_mode(
    path: PathBuf,
    threshold: f64,
    top: usize,
    format: TdgOutputFormat,
    include_components: bool,
    output: Option<PathBuf>,
    critical_only: bool,
    verbose: bool,
) -> Result<()> {
    use notify::{RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::mpsc;
    use tokio::time::Duration;

    eprintln!("👁️  Watching for changes in TDG analysis...");
    let (tx, rx) = mpsc::channel();

    let mut watcher = RecommendedWatcher::new(
        tx,
        notify::Config::default().with_poll_interval(Duration::from_secs(2)),
    )?;
    watcher.watch(&path, RecursiveMode::Recursive)?;

    // Initial analysis
    let calculator = crate::services::tdg_calculator::TDGCalculator::new();
    perform_tdg_analysis(
        &calculator,
        &path,
        threshold,
        top,
        &format,
        include_components,
        &output,
        critical_only,
        verbose,
    )
    .await?;

    loop {
        match rx.recv() {
            Ok(_event) => {
                eprintln!("🔄 Change detected, re-analyzing...");
                perform_tdg_analysis(
                    &calculator,
                    &path,
                    threshold,
                    top,
                    &format,
                    include_components,
                    &output,
                    critical_only,
                    verbose,
                )
                .await?;
            }
            Err(e) => {
                eprintln!("❌ Watch error: {e}");
                break;
            }
        }
    }
    Ok(())
}

/// Run the appropriate TDG analysis based on input mode
#[allow(clippy::too_many_arguments)]
async fn run_tdg_analysis(
    calculator: &crate::services::tdg_calculator::TDGCalculator,
    path: &Path,
    file: Option<PathBuf>,
    files: Vec<PathBuf>,
    include: Vec<String>,
    threshold: f64,
    top: usize,
    format: TdgOutputFormat,
    include_components: bool,
    critical_only: bool,
    verbose: bool,
) -> Result<String> {
    if let Some(single_file) = file {
        // Single file mode
        analyze_single_file(
            calculator,
            path,
            single_file,
            threshold,
            format,
            include_components,
            critical_only,
            verbose,
        )
        .await
    } else if !files.is_empty() {
        // Multiple files mode (MCP tool composition)
        analyze_multiple_files(
            calculator,
            path,
            files,
            threshold,
            top,
            format,
            include_components,
            critical_only,
            verbose,
        )
        .await
    } else {
        // Project mode
        analyze_project(
            calculator,
            path,
            include,
            threshold,
            top,
            format,
            include_components,
            critical_only,
            verbose,
        )
        .await
    }
}

/// Write TDG output to file or stdout
async fn write_tdg_output(output: Option<PathBuf>, content: &str) -> Result<()> {
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, content).await?;
        eprintln!("📝 Results written to {}", output_path.display());
    } else {
        println!("{content}");
    }
    Ok(())
}

// Helper functions for TDG analysis

/// Analyze a single file and return formatted output
#[allow(clippy::too_many_arguments)]
async fn analyze_single_file(
    calculator: &crate::services::tdg_calculator::TDGCalculator,
    project_path: &Path,
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
    format_tdg_single_file_output(&score, &full_path, format, include_components, verbose)
}

/// Analyze multiple files and return formatted output
#[allow(clippy::too_many_arguments)]
async fn analyze_multiple_files(
    calculator: &crate::services::tdg_calculator::TDGCalculator,
    project_path: &Path,
    files: Vec<PathBuf>,
    threshold: f64,
    top_files: usize,
    format: TdgOutputFormat,
    include_components: bool,
    critical_only: bool,
    verbose: bool,
) -> Result<String> {
    eprintln!("📄 Analyzing TDG for {} files...", files.len());

    let results =
        process_files_for_tdg(calculator, project_path, files, threshold, critical_only).await;

    let filtered_results = apply_results_filtering(results, top_files);
    let summary = create_summary_from_file_results(&filtered_results);

    format_output_from_summary(&summary, format, include_components, verbose)
}

/// Process multiple files for TDG analysis
async fn process_files_for_tdg(
    calculator: &crate::services::tdg_calculator::TDGCalculator,
    project_path: &Path,
    files: Vec<PathBuf>,
    threshold: f64,
    critical_only: bool,
) -> Vec<(crate::models::tdg::TDGScore, PathBuf)> {
    let mut results = Vec::new();

    for file_path in files {
        let full_path = resolve_file_path(project_path, file_path);

        if !full_path.exists() {
            eprintln!("⚠️  Skipping missing file: {}", full_path.display());
            continue;
        }

        if let Some(score) =
            calculate_and_filter_file(calculator, &full_path, threshold, critical_only).await
        {
            results.push((score, full_path));
        }
    }

    results
}

/// Resolve file path relative to project directory
fn resolve_file_path(project_path: &Path, file_path: PathBuf) -> PathBuf {
    if file_path.is_absolute() {
        file_path
    } else {
        project_path.join(&file_path)
    }
}

/// Calculate TDG score for file and apply filters
async fn calculate_and_filter_file(
    calculator: &crate::services::tdg_calculator::TDGCalculator,
    full_path: &Path,
    threshold: f64,
    critical_only: bool,
) -> Option<crate::models::tdg::TDGScore> {
    match calculator.calculate_file(full_path).await {
        Ok(score) => {
            if should_include_score(&score, threshold, critical_only) {
                Some(score)
            } else {
                None
            }
        }
        Err(e) => {
            eprintln!("⚠️  Error analyzing {}: {}", full_path.display(), e);
            None
        }
    }
}

/// Check if score should be included based on filters
fn should_include_score(
    score: &crate::models::tdg::TDGScore,
    threshold: f64,
    critical_only: bool,
) -> bool {
    if critical_only && score.value <= 2.5 {
        return false;
    }
    if score.value < threshold {
        return false;
    }
    true
}

/// Apply sorting and `top_files` limit to results
fn apply_results_filtering(
    mut results: Vec<(crate::models::tdg::TDGScore, PathBuf)>,
    top_files: usize,
) -> Vec<(crate::models::tdg::TDGScore, PathBuf)> {
    // Sort by TDG score descending
    results.sort_unstable_by(|a, b| {
        b.0.value
            .partial_cmp(&a.0.value)
            .expect("NaN values should not occur in complexity scores")
    });

    // Apply top_files limit
    if top_files > 0 && results.len() > top_files {
        results.truncate(top_files);
    }

    results
}

/// Analyze entire project and return formatted output
#[allow(clippy::too_many_arguments)]
async fn analyze_project(
    calculator: &crate::services::tdg_calculator::TDGCalculator,
    project_path: &Path,
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
    summary.hotspots = summary
        .hotspots
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
fn create_summary_from_file_results(
    results: &[(crate::models::tdg::TDGScore, PathBuf)],
) -> crate::models::tdg::TDGSummary {
    use crate::models::tdg::{TDGHotspot, TDGSeverity, TDGSummary};

    let total_files = results.len();
    let critical_files = results
        .iter()
        .filter(|(s, _)| matches!(s.severity, TDGSeverity::Critical))
        .count();
    let warning_files = results
        .iter()
        .filter(|(s, _)| matches!(s.severity, TDGSeverity::Warning))
        .count();

    let tdg_values: Vec<f64> = results.iter().map(|(s, _)| s.value).collect();
    let average_tdg = if tdg_values.is_empty() {
        0.0
    } else {
        tdg_values.iter().sum::<f64>() / tdg_values.len() as f64
    };

    // Calculate percentiles
    let mut sorted_values = tdg_values;
    sorted_values.sort_unstable_by(|a, b| {
        a.partial_cmp(b)
            .expect("NaN values should not occur in numeric data")
    });

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
    summary: &crate::models::tdg::TDGSummary,
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

/// Format single file output for TDG
fn format_tdg_single_file_output(
    score: &crate::models::tdg::TDGScore,
    path: &Path,
    format: TdgOutputFormat,
    include_components: bool,
    verbose: bool,
) -> Result<String> {
    use crate::models::tdg::{TDGHotspot, TDGSeverity, TDGSummary};

    // Create a single-file summary
    let hotspot = TDGHotspot {
        path: path.display().to_string(),
        tdg_score: score.value,
        primary_factor: identify_primary_factor(&score.components),
        estimated_hours: estimate_refactoring_hours(score.value),
    };

    let summary = TDGSummary {
        total_files: 1,
        critical_files: usize::from(matches!(score.severity, TDGSeverity::Critical)),
        warning_files: usize::from(matches!(score.severity, TDGSeverity::Warning)),
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

fn format_table_output(
    summary: &crate::models::tdg::TDGSummary,
    include_components: bool,
    verbose: bool,
) -> String {
    let mut table = String::new();
    table.push_str("\n# Technical Debt Gradient Analysis\n\n");
    table.push_str(&format!(
        "📊 **Total Files Analyzed**: {}\n",
        summary.total_files
    ));

    if summary.total_files > 0 {
        table.push_str(&format!(
            "🔴 **Critical Files**: {} ({:.1}%)\n",
            summary.critical_files,
            (summary.critical_files as f64 / summary.total_files as f64) * 100.0
        ));
        table.push_str(&format!(
            "🟡 **Warning Files**: {} ({:.1}%)\n",
            summary.warning_files,
            (summary.warning_files as f64 / summary.total_files as f64) * 100.0
        ));
    }

    table.push_str(&format!("📈 **Average TDG**: {:.2}\n", summary.average_tdg));
    table.push_str(&format!("📊 **95th Percentile**: {:.2}\n", summary.p95_tdg));
    table.push_str(&format!("📊 **99th Percentile**: {:.2}\n", summary.p99_tdg));
    table.push_str(&format!(
        "⏱️  **Estimated Debt**: {:.1} hours\n\n",
        summary.estimated_debt_hours
    ));

    if !summary.hotspots.is_empty() {
        table.push_str("## Top Hotspots\n\n");
        table.push_str("| File | TDG Score | Primary Factor | Est. Hours |\n");
        table.push_str("|------|-----------|----------------|------------|\n");

        for hotspot in &summary.hotspots {
            table.push_str(&format!(
                "| {} | {:.2} | {} | {:.1} |\n",
                hotspot.path, hotspot.tdg_score, hotspot.primary_factor, hotspot.estimated_hours
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

fn format_json_output(
    summary: &crate::models::tdg::TDGSummary,
    include_components: bool,
) -> String {
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

fn format_markdown_output(
    summary: &crate::models::tdg::TDGSummary,
    include_components: bool,
) -> String {
    let mut md = String::new();

    add_markdown_header(&mut md);
    add_markdown_summary(&mut md, summary);
    add_markdown_hotspots(&mut md, summary);

    if include_components {
        add_markdown_components(&mut md);
    }

    md
}

/// Extract Method: Add markdown header
fn add_markdown_header(md: &mut String) {
    md.push_str("# Technical Debt Gradient Analysis\n\n");
}

/// Extract Method: Add summary section
fn add_markdown_summary(md: &mut String, summary: &crate::models::tdg::TDGSummary) {
    md.push_str("## Summary\n\n");
    md.push_str(&format!("- **Total Files**: {}\n", summary.total_files));

    if summary.total_files > 0 {
        add_markdown_file_stats(md, summary);
    }

    add_markdown_tdg_stats(md, summary);
}

/// Extract Method: Add file statistics
fn add_markdown_file_stats(md: &mut String, summary: &crate::models::tdg::TDGSummary) {
    let critical_pct = (summary.critical_files as f64 / summary.total_files as f64) * 100.0;
    let warning_pct = (summary.warning_files as f64 / summary.total_files as f64) * 100.0;

    md.push_str(&format!(
        "- **Critical Files**: {} ({:.1}%)\n",
        summary.critical_files, critical_pct
    ));
    md.push_str(&format!(
        "- **Warning Files**: {} ({:.1}%)\n",
        summary.warning_files, warning_pct
    ));
}

/// Extract Method: Add TDG statistics
fn add_markdown_tdg_stats(md: &mut String, summary: &crate::models::tdg::TDGSummary) {
    md.push_str(&format!("- **Average TDG**: {:.2}\n", summary.average_tdg));
    md.push_str(&format!("- **95th Percentile**: {:.2}\n", summary.p95_tdg));
    md.push_str(&format!("- **99th Percentile**: {:.2}\n", summary.p99_tdg));
    md.push_str(&format!(
        "- **Estimated Technical Debt**: {:.1} hours\n\n",
        summary.estimated_debt_hours
    ));
}

/// Extract Method: Add hotspots section
fn add_markdown_hotspots(md: &mut String, summary: &crate::models::tdg::TDGSummary) {
    if !summary.hotspots.is_empty() {
        md.push_str("## Hotspots\n\n");
        for (i, hotspot) in summary.hotspots.iter().enumerate() {
            md.push_str(&format!("### {}. {}\n\n", i + 1, hotspot.path));
            md.push_str(&format!("- **TDG Score**: {:.2}\n", hotspot.tdg_score));
            md.push_str(&format!(
                "- **Primary Factor**: {}\n",
                hotspot.primary_factor
            ));
            md.push_str(&format!(
                "- **Estimated Refactoring Time**: {:.1} hours\n\n",
                hotspot.estimated_hours
            ));
        }
    }
}

/// Extract Method: Add components section
fn add_markdown_components(md: &mut String) {
    md.push_str("## TDG Components\n\n");
    md.push_str(
        "The Technical Debt Gradient is calculated using the following weighted components:\n\n",
    );
    md.push_str("- **Complexity** (30%): Cyclomatic and cognitive complexity\n");
    md.push_str("- **Code Churn** (35%): Frequency of changes over time\n");
    md.push_str("- **Coupling** (15%): Dependencies between modules\n");
    md.push_str("- **Domain Risk** (10%): Critical domain areas (auth, crypto, etc.)\n");
    md.push_str("- **Duplication** (10%): Code duplication percentage\n");
}

fn format_sarif_output(summary: &crate::models::tdg::TDGSummary) -> String {
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

pub fn percentile(sorted_values: &[f64], p: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }

    let index = (sorted_values.len() as f64 * p) as usize;
    let index = index.min(sorted_values.len() - 1);
    sorted_values[index]
}

pub fn identify_primary_factor(components: &crate::models::tdg::TDGComponents) -> String {
    let mut factors = [
        (components.complexity * 0.30, "High Complexity"),
        (components.churn * 0.35, "Frequent Changes"),
        (components.coupling * 0.15, "High Coupling"),
        (components.domain_risk * 0.10, "Domain Risk"),
        (components.duplication * 0.10, "Code Duplication"),
    ];

    factors.sort_unstable_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .expect("NaN values should not occur in factor scores")
    });
    factors[0].1.to_string()
}

pub fn estimate_refactoring_hours(tdg_score: f64) -> f64 {
    // Empirical formula: hours = base * multiplier^tdg
    let base_hours = 2.0;
    let multiplier: f64 = 1.8;
    base_hours * multiplier.powf(tdg_score)
}

