/// Handle comprehensive analysis with configuration struct
async fn handle_analyze_comprehensive_with_config(config: ComprehensiveConfig) -> Result<()> {
    let start_time = Instant::now();

    info!("🔍 Starting comprehensive analysis");

    // Determine analysis mode
    let (analysis_path, single_file_mode, target_files) = determine_analysis_mode(&config)?;

    // Log enabled analyses
    let enabled_analyses = get_enabled_analyses(&config);
    info!("📊 Enabled analyses: {}", enabled_analyses.join(", "));

    // Create defect report service
    let service = DefectReportService::new();

    // Generate comprehensive report
    let mut report = service.generate_report(&analysis_path).await?;

    // Apply include/exclude file filtering (Feature #52)
    report = DefectReportService::filter_by_pattern(
        &report,
        config.include.clone(),
        config.exclude.clone(),
        config.min_lines,
    );

    // Apply filters based on confidence threshold and file targeting
    let filtered_defects = filter_defects(
        &report.defects,
        single_file_mode,
        &target_files,
        config.confidence_threshold,
    );

    info!("📈 Total defects found: {}", report.defects.len());
    info!(
        "📉 After confidence filter (>={:.0}%): {}",
        config.confidence_threshold * 100.0,
        filtered_defects.len()
    );

    // Format output
    let formatted_output = format_report(&service, &report, filtered_defects, &config)?;

    // Write output
    write_output(&config.output, &formatted_output).await?;

    let elapsed = start_time.elapsed();

    // Print performance metrics if requested
    if config.perf {
        print_performance_metrics(elapsed, &report);
    }

    // Print summary by category
    info!("\n📊 Defects by Category:");
    for (category, count) in &report.summary.by_category {
        info!("  {}: {}", category, count);
    }

    // Warn about ignored parameters (for transparency)
    warn_ignored_parameters(&config);

    Ok(())
}

/// Determine the analysis mode based on configuration
fn determine_analysis_mode(config: &ComprehensiveConfig) -> Result<(PathBuf, bool, Vec<PathBuf>)> {
    let analysis_path = if let Some(ref file) = config.file {
        // Single file mode
        find_project_root(file)?
    } else {
        config.project_path.clone()
    };

    let single_file_mode = config.file.is_some();

    let target_files = if !config.files.is_empty() {
        config.files.clone()
    } else if let Some(ref file) = config.file {
        vec![file.clone()]
    } else {
        vec![]
    };

    Ok((analysis_path, single_file_mode, target_files))
}

/// Get list of enabled analyses
fn get_enabled_analyses(config: &ComprehensiveConfig) -> Vec<String> {
    let mut analyses = Vec::new();

    if config.include_complexity {
        analyses.push("Complexity".to_string());
    }
    if config.include_tdg {
        analyses.push("TDG".to_string());
    }
    if config.include_defects {
        analyses.push("Defects".to_string());
    }
    if config.include_dead_code {
        analyses.push("Dead Code".to_string());
    }
    if config.include_duplicates {
        analyses.push("Duplicates".to_string());
    }

    analyses
}

/// Filter defects based on criteria
fn filter_defects(
    defects: &[crate::models::defect_report::Defect],
    single_file_mode: bool,
    target_files: &[PathBuf],
    confidence_threshold: f32,
) -> Vec<crate::models::defect_report::Defect> {
    defects
        .iter()
        .filter(|defect| {
            // Filter by confidence threshold
            let confidence = defect.metrics.get("confidence").copied().unwrap_or(1.0) as f32;

            if confidence < confidence_threshold {
                return false;
            }

            // Filter by target files if specified
            if single_file_mode && !target_files.is_empty() {
                return target_files.contains(&defect.file_path);
            }

            true
        })
        .cloned()
        .collect()
}

/// Format the report based on configuration
fn format_report(
    service: &DefectReportService,
    report: &crate::models::defect_report::DefectReport,
    filtered_defects: Vec<crate::models::defect_report::Defect>,
    config: &ComprehensiveConfig,
) -> Result<String> {
    let format = match config.format {
        ComprehensiveOutputFormat::Json => ReportFormat::Json,
        ComprehensiveOutputFormat::Summary => ReportFormat::Markdown,
        ComprehensiveOutputFormat::Detailed => ReportFormat::Markdown,
        ComprehensiveOutputFormat::Markdown => ReportFormat::Markdown,
        ComprehensiveOutputFormat::Sarif => ReportFormat::Json, // SARIF is JSON-based
    };

    // Create a modified report with filtered defects
    let mut filtered_report = report.clone();
    filtered_report.defects = filtered_defects;

    // Format the report
    match format {
        ReportFormat::Json => serde_json::to_string_pretty(&filtered_report)
            .context("Failed to serialize report to JSON"),
        ReportFormat::Markdown | ReportFormat::Text => service
            .format_text(&filtered_report)
            .context("Failed to format report as text"),
        ReportFormat::Csv => {
            // For CSV, we'll use JSON as a fallback for now
            serde_json::to_string_pretty(&filtered_report).context("Failed to serialize report")
        }
    }
}

/// Write output to file or stdout
async fn write_output(output: &Option<PathBuf>, content: &str) -> Result<()> {
    if let Some(output_path) = output {
        tokio::fs::write(output_path, content)
            .await
            .context("Failed to write output file")?;
        info!("📝 Report written to: {}", output_path.display());
    } else {
        println!("{content}");
    }
    Ok(())
}

/// Print performance metrics
fn print_performance_metrics(
    elapsed: std::time::Duration,
    report: &crate::models::defect_report::DefectReport,
) {
    info!("\n⏱️  Performance Metrics:");
    info!("  Total time: {:.2}s", elapsed.as_secs_f64());
    info!("  Hotspot files: {}", report.summary.hotspot_files.len());
    info!("  Defects found: {}", report.summary.total_defects);
    let defects_per_second = report.summary.total_defects as f64 / elapsed.as_secs_f64();
    info!("  Defects/second: {:.2}", defects_per_second);
}

/// Warn about ignored parameters
fn warn_ignored_parameters(_config: &ComprehensiveConfig) {
    // Feature #52: include/exclude/min_lines filtering now implemented
    // No warnings needed - all parameters are handled by DefectReportService::filter_by_pattern()
}

/// Find the project root by looking for Cargo.toml
fn find_project_root(start_path: &Path) -> Result<PathBuf> {
    let start_dir = if start_path.is_file() {
        start_path
            .parent()
            .context("File has no parent directory")?
    } else {
        start_path
    };
    walk_up_to_cargo_toml(start_dir)
        .ok_or_else(|| anyhow::anyhow!("No Cargo.toml found"))
        .or_else(|_| Ok(start_dir.to_path_buf()))
}

fn walk_up_to_cargo_toml(start: &Path) -> Option<PathBuf> {
    let mut current = start;
    loop {
        if current.join("Cargo.toml").exists() {
            return Some(current.to_path_buf());
        }
        let parent = current.parent()?;
        if is_system_root(parent) {
            return None;
        }
        current = parent;
    }
}

fn is_system_root(path: &Path) -> bool {
    path == Path::new("/tmp") || path == Path::new("/") || path == Path::new("/home")
}
