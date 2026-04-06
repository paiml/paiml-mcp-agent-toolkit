// TDG handler - main entry point and orchestration

#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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
