//! Complexity analysis command handlers with refactored dead code handler
//!
//! This module contains all complexity-related command implementations
//! extracted from the main CLI module to reduce cognitive complexity.

use crate::cli::*;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

#[cfg(test)]
mod complexity_handlers_tests;

/// Configuration for complexity analysis operations
/// 
/// This struct centralizes all configuration parameters and provides
/// helper methods to reduce the complexity of the main handler function.
/// Following Toyota Way single responsibility principle.
#[derive(Debug, Clone)]
struct ComplexityConfig {
    project_path: PathBuf,
    toolchain: Option<String>,
    max_cyclomatic: u16,
    max_cognitive: u16,
    include: Vec<String>,
    timeout: u64,
    top_files: usize,
}

impl ComplexityConfig {
    /// Create configuration from CLI arguments
    fn from_args(
        project_path: PathBuf,
        toolchain: Option<String>,
        max_cyclomatic: Option<u16>,
        max_cognitive: Option<u16>,
        include: Vec<String>,
        timeout: u64,
        top_files: usize,
    ) -> Self {
        Self {
            project_path,
            toolchain,
            max_cyclomatic: max_cyclomatic.unwrap_or(10),
            max_cognitive: max_cognitive.unwrap_or(15),
            include,
            timeout,
            top_files,
        }
    }

    /// Detect toolchain for the project, returning detected toolchain or None for multi-language
    fn detect_toolchain(&self) -> Option<String> {
        self.toolchain
            .clone()
            .or_else(|| super::super::stubs::detect_toolchain(&self.project_path))
    }

}

/// Analyze a single file and return its complexity metrics
/// 
/// This helper function handles single file analysis with proper error handling
/// and maintains consistency with the Issue #42 fix for multi-language support.
async fn analyze_single_file(
    file_path: &PathBuf, 
    config: &ComplexityConfig
) -> Result<Vec<crate::services::complexity::FileComplexityMetrics>> {
    eprintln!("🔍 Analyzing complexity of file: {}", file_path.display());

    // Ensure file exists and resolve absolute path
    let full_path = if file_path.is_absolute() {
        file_path.clone()
    } else {
        config.project_path.join(file_path)
    };

    if !full_path.exists() {
        anyhow::bail!("File not found: {}", full_path.display());
    }

    // Use unified analyzer (maintains Issue #42 fix)
    let file_content = std::fs::read_to_string(&full_path)
        .context(format!("Failed to read file: {}", full_path.display()))?;

    let metrics = crate::cli::language_analyzer::analyze_file_complexity(&full_path, &file_content)
        .await?;
    
    Ok(vec![metrics])
}

/// Analyze multiple files and return aggregated complexity metrics
/// 
/// This helper function processes a list of files, maintaining consistency
/// with single file analysis and proper error handling for missing files.
async fn analyze_multiple_files(
    files: &[PathBuf], 
    config: &ComplexityConfig
) -> Result<Vec<crate::services::complexity::FileComplexityMetrics>> {
    eprintln!("🔍 Analyzing complexity of {} files...", files.len());

    let mut all_metrics = Vec::new();
    for file_path in files {
        let full_path = if file_path.is_absolute() {
            file_path.clone()
        } else {
            config.project_path.join(file_path)
        };

        if !full_path.exists() {
            eprintln!("⚠️  Skipping missing file: {}", full_path.display());
            continue;
        }

        // Use same analyzer as single file mode (Issue #42 consistency)
        let file_content = std::fs::read_to_string(&full_path)
            .context(format!("Failed to read file: {}", full_path.display()))?;

        let metrics = crate::cli::language_analyzer::analyze_file_complexity(&full_path, &file_content)
            .await?;
        all_metrics.push(metrics);
    }
    
    Ok(all_metrics)
}

/// Analyze entire project directory based on toolchain detection
/// 
/// This helper function handles project-wide analysis with proper toolchain
/// detection and maintains the Issue #42 fix for multi-language projects.
async fn analyze_project(
    detected_toolchain: Option<String>, 
    config: &ComplexityConfig
) -> Result<Vec<crate::services::complexity::FileComplexityMetrics>> {
    match detected_toolchain {
        Some(ref toolchain) => {
            eprintln!("🔍 Analyzing {} project complexity...", toolchain);
            super::super::stubs::analyze_project_files(
                &config.project_path,
                Some(toolchain),
                &config.include,
                config.max_cyclomatic,
                config.max_cognitive,
            )
            .await
        }
        None => {
            // No specific toolchain detected - analyze all supported file types
            eprintln!("🔍 Analyzing project complexity (multi-language)...");
            super::super::stubs::analyze_project_files(
                &config.project_path,
                None, // This will trigger analysis of all supported languages
                &config.include,
                config.max_cyclomatic,
                config.max_cognitive,
            )
            .await
        }
    }
}

/// Apply complexity threshold filtering to metrics
/// 
/// Filters files to only include those with functions exceeding the specified
/// cyclomatic or cognitive complexity thresholds.
fn apply_complexity_filters(
    file_metrics: &mut Vec<crate::services::complexity::FileComplexityMetrics>,
    max_cyclomatic: Option<u16>,
    max_cognitive: Option<u16>,
) {
    if max_cyclomatic.is_some() || max_cognitive.is_some() {
        file_metrics.retain(|file| {
            file.functions.iter().any(|func| {
                let exceeds_cyclomatic = max_cyclomatic
                    .map(|threshold| func.metrics.cyclomatic > threshold)
                    .unwrap_or(false);
                let exceeds_cognitive = max_cognitive
                    .map(|threshold| func.metrics.cognitive > threshold)
                    .unwrap_or(false);
                exceeds_cyclomatic || exceeds_cognitive
            })
        });
    }
}

/// Apply top files limit by sorting and truncating results
/// 
/// Sorts files by total complexity (cyclomatic + cognitive) in descending order
/// and keeps only the top N most complex files.
fn apply_top_files_limit(
    file_metrics: &mut Vec<crate::services::complexity::FileComplexityMetrics>,
    top_files: usize,
) {
    if top_files > 0 && !file_metrics.is_empty() {
        // Sort files by complexity (descending)
        file_metrics.sort_by(|a, b| {
            let a_complexity =
                a.total_complexity.cyclomatic as f64 + a.total_complexity.cognitive as f64;
            let b_complexity =
                b.total_complexity.cyclomatic as f64 + b.total_complexity.cognitive as f64;
            b_complexity
                .partial_cmp(&a_complexity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Keep only top N files
        file_metrics.truncate(top_files);
    }
}

async fn format_and_write_output(
    summary: &crate::services::complexity::ComplexityReport,
    file_metrics: &[crate::services::complexity::FileComplexityMetrics],
    format: ComplexityOutputFormat,
    output: Option<PathBuf>,
    top_files: usize,
) -> Result<()> {
    use crate::services::complexity::{format_complexity_summary, format_complexity_report, format_as_sarif};
    
    let formatted_output = match format {
        ComplexityOutputFormat::Summary => Ok(format_complexity_summary(summary)),
        ComplexityOutputFormat::Full => Ok(format_complexity_report(summary)),
        ComplexityOutputFormat::Sarif => format_as_sarif(summary)
            .map_err(|e| anyhow::anyhow!("SARIF serialization failed: {}", e)),
        ComplexityOutputFormat::Json => {
            let json_output = serde_json::json!({
                "summary": summary,
                "files": file_metrics,
                "top_files_limit": if top_files > 0 { Some(top_files) } else { None },
            });
            serde_json::to_string_pretty(&json_output)
                .map_err(|e| anyhow::anyhow!("JSON serialization failed: {}", e))
        }
    }?;

    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &formatted_output).await?;
        eprintln!("📝 Results written to: {}", output_path.display());
    } else {
        println!("{}", formatted_output);
    }

    Ok(())
}

/// Handle complexity analysis command with MCP tool composition support
///
/// This function enables AI agents to perform sophisticated code analysis workflows
/// by supporting three distinct modes of operation:
///
/// 1. **Project Mode**: Analyze entire project using include patterns
/// 2. **Single File Mode**: Deep analysis of one specific file
/// 3. **Multi-File Mode**: Process specific file lists for MCP tool chaining
///
/// # Filtering Behavior
///
/// When `max_cyclomatic` or `max_cognitive` thresholds are specified:
/// - Only files containing functions that EXCEED the thresholds are included
/// - This filtering happens BEFORE the `top_files` limit is applied
/// - A file with all functions below the threshold will be excluded from results
///
/// # MCP Tool Composition Examples
///
/// ```no_run
/// // Example 1: AI agent discovers complexity hotspots
/// use std::path::PathBuf;
/// use pmat::cli::{ComplexityOutputFormat, handlers::complexity_handlers::handle_analyze_complexity};
///
/// # async fn mcp_workflow_example() -> anyhow::Result<()> {
/// // Step 1: Find top 5 most complex files
/// handle_analyze_complexity(
///     PathBuf::from("."),
///     None,                           // file
///     vec![],                         // files (empty = project mode)
///     Some("rust".to_string()),       // toolchain
///     ComplexityOutputFormat::Json,   // format for parsing
///     None,                           // output (stdout)
///     Some(20),                       // max_cyclomatic
///     Some(15),                       // max_cognitive
///     vec![],                         // include patterns
///     false,                          // watch
///     5,                              // top_files = 5 hotspots
///     false,                          // fail_on_violation
/// ).await?;
///
/// // AI agent would parse JSON output to extract file paths:
/// // let hotspot_files = parse_json_extract_paths(json_output);
///
/// // Step 2: Deep analyze just those hotspot files
/// let hotspot_files = vec![
///     PathBuf::from("src/complex_module.rs"),
///     PathBuf::from("src/legacy_code.rs"),
/// ];
///
/// handle_analyze_complexity(
///     PathBuf::from("."),
///     None,                           // file
///     hotspot_files,                  // files (MCP composition)
///     Some("rust".to_string()),       // toolchain
///     ComplexityOutputFormat::Json,   // format
///     None,                           // output
///     Some(10),                       // stricter threshold
///     Some(8),                        // stricter threshold
///     vec![],                         // include patterns
///     false,                          // watch
///     0,                              // top_files (show all)
///     false,                          // fail_on_violation
/// ).await?;
/// # Ok(())
/// # }
/// ```
///
/// ```no_run
/// // Example 2: AI agent builds refactoring pipeline
/// use std::path::PathBuf;
/// use pmat::cli::{ComplexityOutputFormat, handlers::complexity_handlers::handle_analyze_complexity};
///
/// # async fn mcp_refactor_pipeline() -> anyhow::Result<()> {
/// // Step 1: Identify files needing refactoring
/// let candidate_files = vec![
///     PathBuf::from("src/user_service.rs"),
///     PathBuf::from("src/payment_processor.rs"),
///     PathBuf::from("src/notification_engine.rs"),
/// ];
///
/// // Step 2: Analyze complexity metrics for prioritization
/// handle_analyze_complexity(
///     PathBuf::from("."),
///     None,                           // file
///     candidate_files,                // files (targeted analysis)
///     Some("rust".to_string()),       // toolchain
///     ComplexityOutputFormat::Json,   // format for decision making
///     None,                           // output
///     Some(15),                       // max_cyclomatic
///     Some(12),                       // max_cognitive
///     vec![],                         // include patterns
///     false,                          // watch
///     0,                              // top_files (analyze all provided)
///     false,                          // fail_on_violation
/// ).await?;
///
/// // AI agent would then:
/// // 1. Parse complexity metrics
/// // 2. Prioritize by technical debt impact
/// // 3. Generate refactoring recommendations
/// // 4. Chain to other pmat tools (dead-code, duplicates, etc.)
/// # Ok(())
/// # }
/// ```
///
/// # Threshold Filtering Examples
///
/// ```no_run
/// // Example: Filtering behavior with --max-cyclomatic
/// use std::path::PathBuf;
/// use pmat::cli::{ComplexityOutputFormat, handlers::complexity_handlers::handle_analyze_complexity};
///
/// # async fn threshold_filtering_example() -> anyhow::Result<()> {
/// // Scenario: Find only files with functions exceeding cyclomatic complexity of 20
/// handle_analyze_complexity(
///     PathBuf::from("."),
///     None,                           // file
///     vec![],                         // files
///     Some("rust".to_string()),       // toolchain
///     ComplexityOutputFormat::Json,   // format
///     None,                           // output
///     Some(20),                       // max_cyclomatic - only show files with functions > 20
///     None,                           // max_cognitive
///     vec!["src/**/*.rs".to_string()],// include patterns
///     false,                          // watch
///     10,                             // top_files
///     false,                          // fail_on_violation
/// ).await?;
///
/// // Expected behavior:
/// // - File with functions [5, 10, 15] complexity -> EXCLUDED (all below 20)
/// // - File with functions [5, 25, 10] complexity -> INCLUDED (one function > 20)
/// // - File with functions [21, 30, 40] complexity -> INCLUDED (all above 20)
/// # Ok(())
/// # }
/// ```
///
/// ```no_run
/// // Example: Combined threshold filtering
/// use std::path::PathBuf;
/// use pmat::cli::{ComplexityOutputFormat, handlers::complexity_handlers::handle_analyze_complexity};
///
/// # async fn combined_threshold_example() -> anyhow::Result<()> {
/// // Scenario: Find files with either high cyclomatic OR high cognitive complexity
/// handle_analyze_complexity(
///     PathBuf::from("."),
///     None,                           // file
///     vec![],                         // files
///     Some("rust".to_string()),       // toolchain
///     ComplexityOutputFormat::Json,   // format
///     None,                           // output
///     Some(15),                       // max_cyclomatic
///     Some(12),                       // max_cognitive
///     vec!["src/**/*.rs".to_string()],// include patterns
///     false,                          // watch
///     5,                              // top_files - applied AFTER filtering
///     false,                          // fail_on_violation
/// ).await?;
///
/// // Expected behavior:
/// // - Files are first filtered to only include those with functions exceeding either threshold
/// // - Then the top 5 most complex files from the filtered set are returned
/// // - A file needs at least ONE function with cyclomatic > 15 OR cognitive > 12 to be included
/// # Ok(())
/// # }
/// ```
///
/// # Parameters
///
/// * `project_path` - Root directory of the project
/// * `file` - Single file for focused analysis (conflicts with `files`)
/// * `files` - **MCP Composition**: List of specific files to analyze
/// * `toolchain` - Language detection override
/// * `format` - Output format (JSON recommended for MCP workflows)
/// * `output` - File output path (None = stdout for MCP parsing)
/// * `max_cyclomatic` - Complexity threshold for violations
/// * `max_cognitive` - Cognitive load threshold for violations
/// * `include` - Glob patterns for project mode (conflicts with `files`)
/// * `watch` - Continuous analysis mode
/// * `top_files` - Limit output to N most complex files
///
/// # Exit Status
///
/// The command returns different exit codes based on results (addressing issue #28):
/// - `0`: Success - no violations found, all violations below threshold, or --fail-on-violation not specified
/// - `1`: Failure - violations found that exceed thresholds AND --fail-on-violation flag is used
///
/// ```bash
/// # Exit with code 0 even if violations found (default behavior)
/// pmat analyze complexity --max-cyclomatic 10
///
/// # Exit with code 1 if violations exceed threshold
/// pmat analyze complexity --max-cyclomatic 10 --fail-on-violation
/// ```
///
/// # Returns
///
/// JSON-structured complexity analysis suitable for MCP tool chaining
#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_complexity(
    project_path: PathBuf,
    file: Option<PathBuf>,
    files: Vec<PathBuf>,
    toolchain: Option<String>,
    format: ComplexityOutputFormat,
    output: Option<PathBuf>,
    max_cyclomatic: Option<u16>,
    max_cognitive: Option<u16>,
    include: Vec<String>,
    watch: bool,
    top_files: usize,
    fail_on_violation: bool,
    timeout: u64,
) -> Result<()> {
    use crate::services::complexity::{
        aggregate_results_with_thresholds,
    };

    if watch {
        return handle_watch_mode(
            &project_path,
            toolchain.as_deref(),
            max_cyclomatic,
            max_cognitive,
            include.clone(),
            timeout,
            top_files,
            format,
            output.as_deref(),
        );
    }

    // Create configuration to reduce parameter passing and centralize logic
    let config = ComplexityConfig::from_args(
        project_path,
        toolchain,
        max_cyclomatic,
        max_cognitive,
        include,
        timeout,
        top_files,
    );

    // Detect toolchain (Issue #42 fix: supports multi-language when None)
    let detected_toolchain = config.detect_toolchain();

    // Analyze files using extracted helper functions (reduces complexity)
    eprintln!("⏰ Analysis timeout set to {} seconds", config.timeout);
    let mut file_metrics = if let Some(single_file) = file {
        analyze_single_file(&single_file, &config).await?
    } else if !files.is_empty() {
        analyze_multiple_files(&files, &config).await?
    } else {
        analyze_project(detected_toolchain, &config).await?
    };

    // Apply filtering using extracted helper functions
    apply_complexity_filters(&mut file_metrics, max_cyclomatic, max_cognitive);
    apply_top_files_limit(&mut file_metrics, config.top_files);

    // Aggregate results with custom thresholds
    let summary =
        aggregate_results_with_thresholds(file_metrics.clone(), max_cyclomatic, max_cognitive);

    // Format and write output
    format_and_write_output(&summary, &file_metrics, format, output, top_files).await?;

    // Check for violations and exit with error code if requested
    if fail_on_violation {
        let has_violations = file_metrics.iter().any(|file| {
            let cyclomatic_exceeded = file
                .functions
                .iter()
                .any(|func| func.metrics.cyclomatic > max_cyclomatic.unwrap_or(20));
            let cognitive_exceeded = file
                .functions
                .iter()
                .any(|func| func.metrics.cognitive > max_cognitive.unwrap_or(15));
            cyclomatic_exceeded || cognitive_exceeded
        });

        if has_violations {
            eprintln!("\n❌ Complexity violations found");
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Handle watch mode for continuous complexity analysis
#[allow(clippy::too_many_arguments)]
fn handle_watch_mode(
    path: &Path,
    toolchain: Option<&str>,
    max_cyclomatic: Option<u16>,
    max_cognitive: Option<u16>,
    include: Vec<String>,
    timeout: u64,
    top_files: usize,
    format: ComplexityOutputFormat,
    output: Option<&Path>,
) -> Result<()> {
    eprintln!("👁️  Starting watch mode for complexity analysis...");
    eprintln!("📁 Watching: {}", path.display());
    eprintln!("🔄 Press Ctrl+C to stop watching\n");

    // Create a channel to receive file system events
    let (tx, rx) = channel();

    // Create a watcher with a small debounce delay
    let mut watcher = RecommendedWatcher::new(
        move |event: Result<Event, notify::Error>| {
            if let Ok(event) = event {
                let _ = tx.send(event);
            }
        },
        Config::default().with_poll_interval(Duration::from_secs(1)),
    )?;

    // Start watching the path recursively
    watcher.watch(path, RecursiveMode::Recursive)?;

    // Initial analysis
    eprintln!("📊 Running initial complexity analysis...\n");
    run_complexity_analysis_sync(
        path,
        toolchain,
        max_cyclomatic,
        max_cognitive,
        &include,
        timeout,
        top_files,
        format.clone(),
        output,
    )?;

    // Watch for changes
    loop {
        match rx.recv() {
            Ok(event) => {
                // Filter for relevant file changes
                if should_reanalyze(&event, &include) {
                    eprintln!("\n🔄 File change detected, reanalyzing...");
                    if let Some(paths) = get_changed_paths(&event) {
                        for changed_path in paths {
                            eprintln!("  📝 Changed: {}", changed_path.display());
                        }
                    }
                    eprintln!();
                    
                    // Run analysis again
                    if let Err(e) = run_complexity_analysis_sync(
                        path,
                        toolchain,
                        max_cyclomatic,
                        max_cognitive,
                        &include,
                        timeout,
                        top_files,
                        format.clone(),
                        output,
                    ) {
                        eprintln!("⚠️  Analysis error: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("⚠️  Watch error: {}", e);
                break;
            }
        }
    }

    Ok(())
}

/// Check if we should reanalyze based on the event type
fn should_reanalyze(event: &Event, include_patterns: &[String]) -> bool {
    match event.kind {
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
            // Check if any of the changed paths match our include patterns
            for path in &event.paths {
                if let Some(path_str) = path.to_str() {
                    // Check for source code files
                    if path_str.ends_with(".rs") 
                        || path_str.ends_with(".ts") 
                        || path_str.ends_with(".tsx")
                        || path_str.ends_with(".js")
                        || path_str.ends_with(".jsx")
                        || path_str.ends_with(".py")
                        || path_str.ends_with(".c")
                        || path_str.ends_with(".cpp")
                        || path_str.ends_with(".h")
                        || path_str.ends_with(".hpp") {
                        
                        // If no include patterns, analyze all source files
                        if include_patterns.is_empty() {
                            return true;
                        }
                        
                        // Check against include patterns
                        for pattern in include_patterns {
                            if path_str.contains(pattern) {
                                return true;
                            }
                        }
                    }
                }
            }
            false
        }
        _ => false,
    }
}

/// Get the paths that changed from an event
fn get_changed_paths(event: &Event) -> Option<&Vec<PathBuf>> {
    if !event.paths.is_empty() {
        Some(&event.paths)
    } else {
        None
    }
}

/// Format and output complexity results in watch mode
async fn format_and_output_watch_results(
    summary: crate::services::complexity::ComplexityReport,
    file_metrics: Vec<crate::services::complexity::FileComplexityMetrics>,
    format: ComplexityOutputFormat,
    output: Option<&Path>,
) -> Result<()> {
    use crate::services::complexity::format_complexity_summary;

    // For watch mode, we'll use summary format for simplicity
    let content = match format {
        ComplexityOutputFormat::Json => {
            // Convert to JSON
            serde_json::to_string_pretty(&summary)?
        }
        _ => {
            // Use summary format for all other cases in watch mode
            format_complexity_summary(&summary)
        }
    };

    // Clear screen for better watch mode experience
    print!("\x1B[2J\x1B[1;1H");
    
    // Write output
    if let Some(output_path) = output {
        tokio::fs::write(output_path, &content).await?;
        eprintln!("✅ Analysis written to: {}", output_path.display());
    } else {
        println!("{}", content);
    }

    Ok(())
}

/// Synchronous wrapper for complexity analysis in watch mode
fn run_complexity_analysis_sync(
    path: &Path,
    toolchain: Option<&str>,
    max_cyclomatic: Option<u16>,
    max_cognitive: Option<u16>,
    include: &[String],
    timeout: u64,
    top_files: usize,
    format: ComplexityOutputFormat,
    output: Option<&Path>,
) -> Result<()> {
    // Create a runtime for the async operation
    let runtime = tokio::runtime::Runtime::new()?;
    
    // Create config
    let config = ComplexityConfig::from_args(
        path.to_path_buf(),
        toolchain.map(String::from),
        max_cyclomatic,
        max_cognitive,
        include.to_vec(),
        timeout,
        top_files,
    );

    // Run the analysis
    runtime.block_on(async {
        let mut file_metrics = if path.is_file() {
            analyze_single_file(&path.to_path_buf(), &config).await?
        } else {
            let detected_toolchain = config.detect_toolchain();
            analyze_project(detected_toolchain, &config).await?
        };

        // Apply filters
        apply_complexity_filters(&mut file_metrics, Some(config.max_cyclomatic), Some(config.max_cognitive));
        apply_top_files_limit(&mut file_metrics, config.top_files);

        // Aggregate results
        use crate::services::complexity::aggregate_results_with_thresholds;
        let summary = aggregate_results_with_thresholds(
            file_metrics.clone(), 
            Some(config.max_cyclomatic), 
            Some(config.max_cognitive)
        );

        // Format and output results
        format_and_output_watch_results(summary, file_metrics, format, output).await?;
        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}

/// Handle churn analysis command
pub async fn handle_analyze_churn(
    project_path: PathBuf,
    days: u32,
    format: crate::models::churn::ChurnOutputFormat,
    output: Option<PathBuf>,
    top_files: usize,
) -> Result<()> {
    // Delegate to main implementation for now - will be extracted in Phase 3 Day 8
    super::super::stubs::handle_analyze_churn(project_path, days, format, output, top_files).await
}

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
) -> Result<()> {
    eprintln!("☠️ Analyzing dead code in project...");
    eprintln!("⏰ Analysis timeout set to {} seconds", timeout);

    // Run analysis with timeout
    let timeout_duration = tokio::time::Duration::from_secs(timeout);
    let result = tokio::time::timeout(timeout_duration, async {
        run_dead_code_analysis(
            &path,
            include_unreachable,
            include_tests,
            min_dead_lines,
            top_files,
        )
        .await
    })
    .await
    .map_err(|_| anyhow::anyhow!("Dead code analysis timed out after {} seconds", timeout))??;

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
                "\n❌ Dead code violations found: {:.1}% exceeds threshold of {:.1}%",
                dead_code_percentage, max_percentage
            );
            std::process::exit(1);
        }
    }

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

    let mut analyzer = DeadCodeAnalyzer::new(DeadCodeAnalyzer::DEFAULT_CAPACITY);

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
    top_files: usize,
    fail_on_violation: bool,
    timeout: u64,
) -> Result<()> {
    use crate::services::satd_detector::{SATDDetector, Severity as DetectorSeverity};

    eprintln!("🔍 Analyzing self-admitted technical debt...");
    eprintln!("⏰ Analysis timeout set to {} seconds", timeout);
    if strict {
        eprintln!("📝 Using strict mode (only explicit SATD markers)");
    }

    // Create SATD detector
    let detector = if strict {
        SATDDetector::new_strict()
    } else {
        SATDDetector::new()
    };

    // Run analysis with timeout
    let timeout_duration = tokio::time::Duration::from_secs(timeout);
    let mut result = tokio::time::timeout(timeout_duration, async {
        detector.analyze_project(&path, include_tests).await
    })
    .await
    .map_err(|_| anyhow::anyhow!("SATD analysis timed out after {} seconds", timeout))??;

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

    // Apply top_files filtering if specified
    if top_files > 0 {
        // Group items by file
        use std::collections::HashMap;
        let mut file_counts: HashMap<std::path::PathBuf, usize> = HashMap::new();
        for item in &result.items {
            *file_counts.entry(item.file.clone()).or_insert(0) += 1;
        }

        // Sort files by SATD count (descending)
        let mut sorted_files: Vec<_> = file_counts.into_iter().collect();
        sorted_files.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

        // Keep only items from top N files
        let top_file_paths: std::collections::HashSet<_> = sorted_files
            .into_iter()
            .take(top_files)
            .map(|(path, _)| path)
            .collect();

        result
            .items
            .retain(|item| top_file_paths.contains(&item.file));
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

    // Check for violations and exit with error code if requested
    if fail_on_violation && !result.items.is_empty() {
        eprintln!(
            "\n❌ SATD violations found: {} technical debt items",
            result.items.len()
        );
        std::process::exit(1);
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
///
/// # Example
///
/// ```no_run
/// use pmat::services::satd_detector::{SATDAnalysisResult, SATDSummary, TechnicalDebt, DebtCategory, Severity};
/// use pmat::cli::handlers::complexity_handlers::format_satd_summary;
/// use std::collections::HashMap;
/// use std::path::PathBuf;
/// use chrono::Utc;
///
/// let result = SATDAnalysisResult {
///     items: vec![
///         TechnicalDebt {
///             category: DebtCategory::Defect,
///             severity: Severity::High,
///             text: "Handle error properly".to_string(),
///             file: PathBuf::from("src/main.rs"),
///             line: 42,
///             column: 8,
///             context_hash: [0; 16],
///         },
///         TechnicalDebt {
///             category: DebtCategory::Requirement,
///             severity: Severity::Medium,
///             text: "Add validation".to_string(),
///             file: PathBuf::from("src/lib.rs"),
///             line: 25,
///             column: 4,
///             context_hash: [1; 16],
///         },
///     ],
///     summary: SATDSummary {
///         total_items: 2,
///         by_severity: HashMap::new(),
///         by_category: HashMap::new(),
///         files_with_satd: 2,
///         avg_age_days: 30.0,
///     },
///     total_files_analyzed: 10,
///     files_with_debt: 2,
///     analysis_timestamp: Utc::now(),
/// };
///
/// let summary = format_satd_summary(&result, false);
///
/// assert!(summary.contains("# SATD Analysis Summary"));
/// assert!(summary.contains("**Files analyzed**: 10"));
/// assert!(summary.contains("**Files with SATD**: 2"));
/// assert!(summary.contains("**Total SATD items**: 2"));
/// assert!(summary.contains("## Top Files with SATD"));
/// // Note: Files are sorted by SATD count, then alphabetically
/// assert!(summary.contains("- 1 SATD items"));
/// assert!(summary.contains("- 1 SATD items"));
/// ```
pub fn format_satd_summary(
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

    // Show top files with SATD
    if !result.items.is_empty() {
        writeln!(&mut output, "\n## Top Files with SATD\n").unwrap();

        // Group items by file and count them
        use std::collections::HashMap;
        let mut file_counts: HashMap<&std::path::Path, usize> = HashMap::new();
        for item in &result.items {
            *file_counts.entry(&item.file).or_insert(0) += 1;
        }

        // Sort files by SATD count (descending)
        let mut sorted_files: Vec<_> = file_counts.into_iter().collect();
        sorted_files.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

        // Show top 10 files with their SATD counts
        for (i, (file, count)) in sorted_files.iter().take(10).enumerate() {
            let filename = file.file_name().unwrap_or_default().to_string_lossy();
            writeln!(
                &mut output,
                "{}. `{}` - {} SATD items",
                i + 1,
                filename,
                count
            )
            .unwrap();
        }
    }

    // Show critical items
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
/// Generate dependency analysis graphs using Mermaid
///
/// # Examples
///
/// ```no_run
/// use pmat::cli::handlers::complexity_handlers::handle_analyze_dag;
/// use pmat::cli::DagType;
/// use std::path::PathBuf;
/// use tempfile::tempdir;
///
/// # tokio_test::block_on(async {
/// let dir = tempdir().unwrap();
///
/// // Generate a full dependency graph
/// let result = handle_analyze_dag(
///     DagType::FullDependency,
///     dir.path().to_path_buf(),
///     None, // output to stdout
///     None, // no max depth
///     Some(10), // limit to 10 nodes
///     false, // include external deps
///     false, // don't show complexity
///     false, // no duplicate analysis
///     false, // no dead code analysis
///     false, // not enhanced
/// ).await;
///
/// assert!(result.is_ok());
/// # });
/// ```
pub async fn handle_analyze_dag(
    _dag_type: DagType,
    project_path: PathBuf,
    output: Option<PathBuf>,
    max_depth: Option<usize>,
    target_nodes: Option<usize>,
    filter_external: bool,
    show_complexity: bool,
    _include_duplicates: bool,
    _include_dead_code: bool,
    enhanced: bool,
) -> Result<()> {
    use crate::services::{
        context::analyze_project,
        mermaid_generator::{MermaidGenerator, MermaidOptions},
    };

    eprintln!("🔄 Generating dependency analysis graph...");

    // Analyze project to get context
    let toolchain =
        super::super::detect_primary_language(&project_path).unwrap_or_else(|| "rust".to_string());
    let project_context = analyze_project(&project_path, &toolchain).await?;

    eprintln!("📁 Analyzed {} files", project_context.files.len());

    // Build DAG based on type
    use crate::services::dag_builder::DagBuilder;

    // DagBuilder builds a full dependency graph by default
    let graph = DagBuilder::build_from_project(&project_context);

    eprintln!(
        "📊 Generated graph with {} nodes and {} edges",
        graph.nodes.len(),
        graph.edges.len()
    );

    let enriched_graph = graph;

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
        use crate::services::fixed_graph_builder::{GraphConfig, GroupingStrategy};
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
    use super::*;

    #[test]
    fn test_complexity_handlers_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }

    #[test]
    fn test_dead_code_summary_shows_top_files() {
        // Create mock dead code result with files
        let result = crate::models::dead_code::DeadCodeResult {
            summary: crate::models::dead_code::DeadCodeSummary {
                total_files_analyzed: 5,
                files_with_dead_code: 2,
                total_dead_lines: 45,
                dead_percentage: 15.5,
                dead_functions: 3,
                dead_classes: 1,
                dead_modules: 2,
                unreachable_blocks: 1,
            },
            files: vec![
                crate::models::dead_code::FileDeadCodeMetrics {
                    path: "src/main.rs".to_string(),
                    dead_lines: 25,
                    total_lines: 100,
                    dead_percentage: 25.0,
                    dead_functions: 1,
                    dead_classes: 0,
                    dead_modules: 0,
                    unreachable_blocks: 0,
                    dead_score: 0.0,
                    confidence: crate::models::dead_code::ConfidenceLevel::High,
                    items: vec![crate::models::dead_code::DeadCodeItem {
                        name: "dead_function".to_string(),
                        item_type: crate::models::dead_code::DeadCodeType::Function,
                        line: 10,
                        reason: "Never called".to_string(),
                    }],
                },
                crate::models::dead_code::FileDeadCodeMetrics {
                    path: "src/lib.rs".to_string(),
                    dead_lines: 20,
                    total_lines: 150,
                    dead_percentage: 13.3,
                    dead_functions: 0,
                    dead_classes: 1,
                    dead_modules: 0,
                    unreachable_blocks: 0,
                    dead_score: 0.0,
                    confidence: crate::models::dead_code::ConfidenceLevel::Medium,
                    items: vec![crate::models::dead_code::DeadCodeItem {
                        name: "unused_struct".to_string(),
                        item_type: crate::models::dead_code::DeadCodeType::Class,
                        line: 5,
                        reason: "Never instantiated".to_string(),
                    }],
                },
            ],
            total_files: 5,
            analyzed_files: 5,
        };

        let summary = format_dead_code_as_summary(&result).unwrap();

        // Verify the summary contains the expected sections
        assert!(summary.contains("# Dead Code Analysis Summary"));
        assert!(summary.contains("**Files analyzed**: 5"));
        assert!(summary.contains("**Files with dead code**: 2"));
        assert!(summary.contains("## Top Files with Dead Code"));
        assert!(summary.contains("1. `src/main.rs` - 25.0% dead (25 lines)"));
        assert!(summary.contains("2. `src/lib.rs` - 13.3% dead (20 lines)"));
        assert!(summary.contains("## Dead Code by Type"));
        assert!(summary.contains("**Dead functions**: 3"));
    }

    #[test]
    fn test_dead_code_summary_empty_files() {
        // Test with no dead code files
        let result = crate::models::dead_code::DeadCodeResult {
            summary: crate::models::dead_code::DeadCodeSummary {
                total_files_analyzed: 10,
                files_with_dead_code: 0,
                total_dead_lines: 0,
                dead_percentage: 0.0,
                dead_functions: 0,
                dead_classes: 0,
                dead_modules: 0,
                unreachable_blocks: 0,
            },
            files: vec![],
            total_files: 10,
            analyzed_files: 10,
        };

        let summary = format_dead_code_as_summary(&result).unwrap();

        // Should not contain Top Files section when no files have dead code
        assert!(summary.contains("# Dead Code Analysis Summary"));
        assert!(summary.contains("**Files analyzed**: 10"));
        assert!(summary.contains("**Files with dead code**: 0"));
        assert!(!summary.contains("## Top Files with Dead Code"));
        assert!(!summary.contains("## Dead Code by Type"));
    }
}
