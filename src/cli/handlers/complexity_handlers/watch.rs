//! File watching and live complexity tracking.

#[cfg(feature = "watch")]
use crate::cli::ComplexityOutputFormat;
#[cfg(feature = "watch")]
use crate::services::complexity::FileComplexityMetrics;
#[cfg(feature = "watch")]
use anyhow::Result;
#[cfg(feature = "watch")]
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
#[cfg(feature = "watch")]
use std::path::{Path, PathBuf};
#[cfg(feature = "watch")]
use std::sync::mpsc::channel;
#[cfg(feature = "watch")]
use std::time::Duration;

#[cfg(feature = "watch")]
use super::analysis::{
    analyze_project, analyze_single_file, apply_complexity_filters, apply_top_files_limit,
};
#[cfg(feature = "watch")]
use super::ComplexityConfig;

#[cfg(feature = "watch")]
/// Configuration for synchronous complexity analysis
#[derive(Debug, Clone)]
pub(super) struct SyncAnalysisConfig<'a> {
    pub(super) path: &'a Path,
    pub(super) toolchain: Option<&'a str>,
    pub(super) max_cyclomatic: Option<u16>,
    pub(super) max_cognitive: Option<u16>,
    pub(super) include: &'a [String],
    pub(super) timeout: u64,
    pub(super) top_files: usize,
    pub(super) format: ComplexityOutputFormat,
    pub(super) output: Option<&'a Path>,
}

#[cfg(feature = "watch")]
/// Handle watch mode for continuous complexity analysis
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_watch_mode(
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
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    print_watch_mode_intro(path);
    let (mut watcher, rx) = create_file_watcher(path)?;

    let config = create_sync_config(
        path,
        toolchain,
        max_cyclomatic,
        max_cognitive,
        &include,
        timeout,
        top_files,
        format,
        output,
    );

    // Initial analysis
    run_initial_analysis(&config)?;

    // Watch for changes
    watch_for_file_changes(rx, &config, &include, &mut watcher)
}

#[cfg(feature = "watch")]
/// Print watch mode introduction messages
fn print_watch_mode_intro(path: &Path) {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    eprintln!("👁️  Starting watch mode for complexity analysis...");
    eprintln!("📁 Watching: {}", path.display());
    eprintln!("🔄 Press Ctrl+C to stop watching\n");
}

#[cfg(feature = "watch")]
/// Create file system watcher
fn create_file_watcher(
    path: &Path,
) -> Result<(RecommendedWatcher, std::sync::mpsc::Receiver<Event>)> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let (tx, rx) = channel();

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

    Ok((watcher, rx))
}

#[cfg(feature = "watch")]
/// Create synchronous analysis configuration
#[allow(clippy::too_many_arguments)]
fn create_sync_config<'a>(
    path: &'a Path,
    toolchain: Option<&'a str>,
    max_cyclomatic: Option<u16>,
    max_cognitive: Option<u16>,
    include: &'a [String],
    timeout: u64,
    top_files: usize,
    format: ComplexityOutputFormat,
    output: Option<&'a Path>,
) -> SyncAnalysisConfig<'a> {
    SyncAnalysisConfig {
        path,
        toolchain,
        max_cyclomatic,
        max_cognitive,
        include,
        timeout,
        top_files,
        format,
        output,
    }
}

#[cfg(feature = "watch")]
/// Run initial complexity analysis
fn run_initial_analysis(config: &SyncAnalysisConfig) -> Result<()> {
    eprintln!("📊 Running initial complexity analysis...\n");
    run_complexity_analysis_sync(config.clone())
}

#[cfg(feature = "watch")]
/// Watch for file changes and reanalyze when needed
fn watch_for_file_changes(
    rx: std::sync::mpsc::Receiver<Event>,
    config: &SyncAnalysisConfig,
    include: &[String],
    _watcher: &mut RecommendedWatcher,
) -> Result<()> {
    loop {
        match rx.recv() {
            Ok(event) => {
                if should_reanalyze(&event, include) {
                    handle_file_change_event(&event, config)?;
                }
            }
            Err(e) => {
                eprintln!("⚠️  Watch error: {e}");
                break;
            }
        }
    }
    Ok(())
}

#[cfg(feature = "watch")]
/// Handle a file change event by reanalyzing
fn handle_file_change_event(event: &Event, config: &SyncAnalysisConfig) -> Result<()> {
    eprintln!("\n🔄 File change detected, reanalyzing...");

    if let Some(paths) = get_changed_paths(event) {
        for changed_path in paths {
            eprintln!("  📝 Changed: {}", changed_path.display());
        }
    }
    eprintln!();

    if let Err(e) = run_complexity_analysis_sync(config.clone()) {
        eprintln!("⚠️  Analysis error: {e}");
    }

    Ok(())
}

#[cfg(feature = "watch")]
/// Check if we should reanalyze based on the event type
fn should_reanalyze(event: &Event, include_patterns: &[String]) -> bool {
    match event.kind {
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => event
            .paths
            .iter()
            .any(|path| should_analyze_path(path, include_patterns)),
        _ => false,
    }
}

#[cfg(feature = "watch")]
/// Check if a specific path should be analyzed
fn should_analyze_path(path: &std::path::Path, include_patterns: &[String]) -> bool {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let Some(path_str) = path.to_str() else {
        return false;
    };

    if !is_source_code_file(path_str) {
        return false;
    }

    should_include_file(path_str, include_patterns)
}

#[cfg(feature = "watch")]
/// Check if file is a source code file
fn is_source_code_file(path_str: &str) -> bool {
    debug_assert!(!path_str.is_empty(), "path_str must not be empty");
    path_str.ends_with(".rs")
        || path_str.ends_with(".ts")
        || path_str.ends_with(".tsx")
        || path_str.ends_with(".js")
        || path_str.ends_with(".jsx")
        || path_str.ends_with(".py")
        || path_str.ends_with(".c")
        || path_str.ends_with(".cpp")
        || path_str.ends_with(".cu")
        || path_str.ends_with(".cuh")
        || path_str.ends_with(".h")
        || path_str.ends_with(".hpp")
}

#[cfg(feature = "watch")]
/// Check if file should be included based on patterns
fn should_include_file(path_str: &str, include_patterns: &[String]) -> bool {
    debug_assert!(!path_str.is_empty(), "path_str must not be empty");
    if include_patterns.is_empty() {
        return true;
    }

    include_patterns
        .iter()
        .any(|pattern| path_str.contains(pattern))
}

#[cfg(feature = "watch")]
/// Get the paths that changed from an event
fn get_changed_paths(event: &Event) -> Option<&Vec<PathBuf>> {
    if event.paths.is_empty() {
        None
    } else {
        Some(&event.paths)
    }
}

#[cfg(feature = "watch")]
/// Format and output complexity results in watch mode
async fn format_and_output_watch_results(
    summary: crate::services::complexity::ComplexityReport,
    _file_metrics: Vec<FileComplexityMetrics>,
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
        println!("{content}");
    }

    Ok(())
}

#[cfg(feature = "watch")]
/// Synchronous wrapper for complexity analysis in watch mode
fn run_complexity_analysis_sync(config: SyncAnalysisConfig) -> Result<()> {
    // Create a runtime for the async operation
    let runtime = tokio::runtime::Runtime::new()?;

    // Create config
    let complexity_config = ComplexityConfig::from_args(
        config.path.to_path_buf(),
        config.toolchain.map(String::from),
        config.max_cyclomatic,
        config.max_cognitive,
        config.include.to_vec(),
        config.timeout,
        config.top_files,
    );

    // Run the analysis
    runtime.block_on(async {
        let mut file_metrics = if config.path.is_file() {
            analyze_single_file(config.path, &complexity_config).await?
        } else {
            let detected_toolchain = complexity_config.detect_toolchain();
            analyze_project(detected_toolchain, &complexity_config).await?
        };

        // Apply filters
        apply_complexity_filters(
            &mut file_metrics,
            Some(complexity_config.max_cyclomatic),
            Some(complexity_config.max_cognitive),
        );
        apply_top_files_limit(&mut file_metrics, complexity_config.top_files);

        // Aggregate results
        use crate::services::complexity::aggregate_results_with_thresholds;
        let summary = aggregate_results_with_thresholds(
            file_metrics.clone(),
            Some(complexity_config.max_cyclomatic),
            Some(complexity_config.max_cognitive),
        );

        // Format and output results
        format_and_output_watch_results(summary, file_metrics, config.format, config.output)
            .await?;
        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}
