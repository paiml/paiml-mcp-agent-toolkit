// Core TDG analysis functions: single file, multiple files, project-wide

/// Analyze a single file and return formatted output
async fn analyze_single_file(
    calculator: &TDGCalculator,
    project_path: &PathBuf,
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
    format_single_file_output(&score, &full_path, format, include_components, verbose)
}

/// Analyze multiple files and return formatted output
async fn analyze_multiple_files(
    calculator: &TDGCalculator,
    project_path: &PathBuf,
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

    let mut results = Vec::new();
    for file_path in files {
        if let Some(result) = analyze_single_file(calculator, project_path, file_path, threshold, critical_only).await {
            results.push(result);
        }
    }

    results.sort_unstable_by(|a, b| b.0.value.total_cmp(&a.0.value));
    if top_files > 0 && results.len() > top_files {
        results.truncate(top_files);
    }

    let summary = create_summary_from_file_results(&results);
    format_output_from_summary(&summary, format, include_components, verbose)
}

async fn analyze_single_file(
    calculator: &TDGCalculator,
    project_path: &PathBuf,
    file_path: PathBuf,
    threshold: f64,
    critical_only: bool,
) -> Option<(crate::models::tdg::TDGScore, PathBuf)> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
    debug_assert!(threshold >= 0.0, "threshold must be non-negative");
    let full_path = if file_path.is_absolute() {
        file_path
    } else {
        project_path.join(&file_path)
    };

    if !full_path.exists() {
        eprintln!("⚠️  Skipping missing file: {}", full_path.display());
        return None;
    }

    match calculator.calculate_file(&full_path).await {
        Ok(score) if should_include_score(&score, threshold, critical_only) => {
            Some((score, full_path))
        }
        Ok(_) => None,
        Err(e) => {
            eprintln!("⚠️  Error analyzing {}: {}", full_path.display(), e);
            None
        }
    }
}

fn should_include_score(score: &crate::models::tdg::TDGScore, threshold: f64, critical_only: bool) -> bool {
    debug_assert!(threshold >= 0.0, "threshold must be non-negative");
    if critical_only && score.value <= 2.5 {
        return false;
    }
    score.value >= threshold
}

/// Analyze entire project and return formatted output
async fn analyze_project(
    calculator: &TDGCalculator,
    project_path: &PathBuf,
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
    summary.hotspots = summary.hotspots
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
fn create_summary_from_file_results(results: &[(TDGScore, PathBuf)]) -> TDGSummary {
    debug_assert!(!results.is_empty(), "results must not be empty");
    let total_files = results.len();
    let critical_files = results.iter().filter(|(s, _)| matches!(s.severity, TDGSeverity::Critical)).count();
    let warning_files = results.iter().filter(|(s, _)| matches!(s.severity, TDGSeverity::Warning)).count();

    let tdg_values: Vec<f64> = results.iter().map(|(s, _)| s.value).collect();
    let average_tdg = if tdg_values.is_empty() {
        0.0
    } else {
        tdg_values.iter().sum::<f64>() / tdg_values.len() as f64
    };

    // Calculate percentiles
    let mut sorted_values = tdg_values.clone();
    sorted_values.sort_unstable_by(|a, b| a.total_cmp(b));

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
