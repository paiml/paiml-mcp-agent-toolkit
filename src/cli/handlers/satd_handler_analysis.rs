// Sprint 89 GREEN Phase: Refactored handle_analyze_satd function
// BEFORE: Complexity 13 (High entropy, mixed concerns)
// AFTER: Complexity 6 (A+ standard, single responsibility)
pub async fn handle_analyze_satd(config: SatdAnalysisConfig) -> Result<()> {
    eprintln!("🔍 Analyzing Self-Admitted Technical Debt (SATD)...");

    // Delegate filter logging to extracted function
    log_filter_info(&config);

    // Delegate analysis execution to extracted function
    let result = execute_satd_analysis(&config).await?;

    // Delegate result filtering to extracted function
    let filtered_result = apply_analysis_filters(result, &config)?;

    // Delegate output formatting and writing to extracted function
    write_satd_output(&filtered_result, &config).await?;

    Ok(())
}

// Sprint 89 GREEN Phase: NEW EXTRACTED FUNCTIONS (A+ ≤10 complexity each)

/// Log filter information if filters are specified - EXTRACTED FUNCTION
/// Complexity: 5 (A+ standard)
fn log_filter_info(config: &SatdAnalysisConfig) {
    if !config.include.is_empty() || !config.exclude.is_empty() {
        eprintln!("🔍 Applying file filters...");
        if !config.include.is_empty() {
            eprintln!("  Include patterns: {:?}", config.include);
        }
        if !config.exclude.is_empty() {
            eprintln!("  Exclude patterns: {:?}", config.exclude);
        }
    }
}

/// Execute SATD analysis using facade - EXTRACTED FUNCTION
/// Complexity: 6 (A+ standard)
async fn execute_satd_analysis(config: &SatdAnalysisConfig) -> Result<SatdAnalysisResult> {
    // Create service registry and facade
    let registry = Arc::new(ServiceRegistry::new());
    let facade = SatdFacade::new(registry);

    // Log extended mode if enabled
    if config.extended {
        eprintln!("📋 Extended mode: detecting euphemisms (placeholder, stub, for now...)");
    }

    // Build analysis request
    let request = SatdAnalysisRequest {
        path: config.path.clone(),
        strict_mode: config.strict,
        include_tests: config.include_tests,
        extended: config.extended,
    };

    // Perform analysis
    facade.analyze_project(request).await
}

/// Apply file and severity filters to analysis results - EXTRACTED FUNCTION
/// Complexity: 8 (A+ standard)
fn apply_analysis_filters(
    mut result: SatdAnalysisResult,
    config: &SatdAnalysisConfig,
) -> Result<SatdAnalysisResult> {
    // Apply file filter to results if filters are specified
    if !config.include.is_empty() || !config.exclude.is_empty() {
        use crate::utils::file_filter::FileFilter;
        let filter = FileFilter::new(config.include.clone(), config.exclude.clone())?;

        if filter.has_filters() {
            result.violations.retain(|violation| {
                let path = std::path::Path::new(&violation.file_path);
                filter.should_include(path)
            });

            // Update total files count
            let unique_files: std::collections::HashSet<_> =
                result.violations.iter().map(|v| &v.file_path).collect();
            result.total_files = unique_files.len();
        }
    }

    // Apply severity and criticality filters
    Ok(apply_filters(
        result,
        config.severity.clone(),
        config.critical_only,
    ))
}

/// Format and write SATD output - EXTRACTED FUNCTION
/// Complexity: 7 (A+ standard)
async fn write_satd_output(
    filtered_result: &SatdAnalysisResult,
    config: &SatdAnalysisConfig,
) -> Result<()> {
    // Format output
    let content = format_output(
        filtered_result,
        config.format.clone(),
        config.evolution,
        config.days,
        config.metrics,
    );

    // Write to file or stdout
    if let Some(output_path) = &config.output {
        tokio::fs::write(output_path, &content).await?;
        eprintln!("✅ SATD analysis written to: {}", output_path.display());
    } else {
        println!("{content}");
    }

    // Print metrics if requested
    if config.metrics {
        print_metrics(filtered_result);
    }

    Ok(())
}

/// Apply severity and criticality filters
fn apply_filters(
    mut result: SatdAnalysisResult,
    severity: Option<SatdSeverity>,
    critical_only: bool,
) -> SatdAnalysisResult {
    if let Some(min_severity) = severity {
        result.violations.retain(|v| match min_severity {
            SatdSeverity::Critical => matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::Critical
            ),
            SatdSeverity::High => matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::Critical
                    | crate::services::facades::satd_facade::SatdSeverity::High
            ),
            SatdSeverity::Medium => !matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::Low
            ),
            SatdSeverity::Low => true,
        });
    }

    if critical_only {
        result.violations.retain(|v| {
            matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::Critical
                    | crate::services::facades::satd_facade::SatdSeverity::High
            )
        });
    }

    result
}
