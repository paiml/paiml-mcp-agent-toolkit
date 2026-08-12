// TDG watch mode - feature-gated file watcher for continuous analysis

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
        crate::status_eprintln!("✅ TDG analysis saved to {}", output_path.display());
    } else {
        print!("{output_content}");
    }

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

    crate::status_eprintln!("👁️  Watching for changes in TDG analysis...");
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
                crate::status_eprintln!("🔄 Change detected, re-analyzing...");
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
