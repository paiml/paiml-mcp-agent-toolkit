// TDG core analysis - single file, multiple files, and project analysis

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
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    debug_assert!(file.exists(), "file must exist: {}", file.display());
    debug_assert!(threshold >= 0.0, "threshold must be non-negative");
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
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    debug_assert!(threshold >= 0.0, "threshold must be non-negative");
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
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    debug_assert!(threshold >= 0.0, "threshold must be non-negative");
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

/// Calculate TDG score for file and apply filters
async fn calculate_and_filter_file(
    calculator: &crate::services::tdg_calculator::TDGCalculator,
    full_path: &Path,
    threshold: f64,
    critical_only: bool,
) -> Option<crate::models::tdg::TDGScore> {
    debug_assert!(full_path.exists(), "full_path must exist: {}", full_path.display());
    debug_assert!(threshold >= 0.0, "threshold must be non-negative");
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
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    debug_assert!(threshold >= 0.0, "threshold must be non-negative");
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
