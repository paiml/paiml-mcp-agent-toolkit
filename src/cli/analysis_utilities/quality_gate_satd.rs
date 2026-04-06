// Quality gate handlers - extracted for file health (CB-040)
#[allow(clippy::too_many_arguments)]
/// Toyota Way: Strategy Pattern + Extract Method - reduced complexity from 21→≤8  
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_analyze_satd(
    path: PathBuf,
    format: SatdOutputFormat,
    severity: Option<SatdSeverity>,
    critical_only: bool,
    include_tests: bool,
    evolution: bool,
    days: u32,
    metrics: bool,
    output: Option<PathBuf>,
) -> Result<()> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    use crate::services::satd_detector::SATDDetector;
    eprintln!("🔍 Analyzing Self-Admitted Technical Debt (SATD)...");

    let detector = SATDDetector::new();
    let satd_items = analyze_satd_items(&detector, &path, include_tests).await?;
    let filtered_items = apply_satd_filters(satd_items, severity, critical_only);
    let output_content = generate_satd_output(format, &filtered_items, metrics, evolution, days);

    write_satd_output(output, &output_content).await?;

    if metrics {
        print_satd_metrics(&filtered_items);
    }

    Ok(())
}

/// Toyota Way: Extract Method - analyze SATD items (complexity ≤3)
async fn analyze_satd_items(
    detector: &crate::services::satd_detector::SATDDetector,
    path: &Path,
    include_tests: bool,
) -> Result<Vec<crate::services::satd_detector::TechnicalDebt>> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    if include_tests {
        detector
            .analyze_directory_with_tests(path, true)
            .await
            .map_err(Into::into)
    } else {
        detector.analyze_directory(path).await.map_err(Into::into)
    }
}

/// Toyota Way: Extract Method - apply SATD filters (complexity ≤8)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn apply_satd_filters(
    mut satd_items: Vec<crate::services::satd_detector::TechnicalDebt>,
    severity: Option<SatdSeverity>,
    critical_only: bool,
) -> Vec<crate::services::satd_detector::TechnicalDebt> {
    debug_assert!(!satd_items.is_empty(), "satd_items must not be empty");
    // Filter by severity if specified
    if let Some(min_severity) = severity {
        let min_sev = match min_severity {
            SatdSeverity::Critical => crate::services::satd_detector::Severity::Critical,
            SatdSeverity::High => crate::services::satd_detector::Severity::High,
            SatdSeverity::Medium => crate::services::satd_detector::Severity::Medium,
            SatdSeverity::Low => crate::services::satd_detector::Severity::Low,
        };
        // Severity enum: Low=0, Medium=1, High=2, Critical=3
        // Filter should keep items with severity >= min_sev (show critical and above)
        satd_items.retain(|item| item.severity as u8 >= min_sev as u8);
    }

    // Filter for critical items only if requested
    if critical_only {
        satd_items.retain(|item| {
            matches!(
                item.severity,
                crate::services::satd_detector::Severity::Critical
                    | crate::services::satd_detector::Severity::High
            )
        });
    }

    satd_items
}

/// Toyota Way: Strategy Pattern - generate output by format (complexity ≤4)
fn generate_satd_output(
    format: SatdOutputFormat,
    filtered_items: &[crate::services::satd_detector::TechnicalDebt],
    metrics: bool,
    evolution: bool,
    days: u32,
) -> String {
    match format {
        SatdOutputFormat::Summary => format_satd_summary(filtered_items),
        SatdOutputFormat::Json => format_satd_json(filtered_items, metrics, evolution),
        SatdOutputFormat::Sarif => format_satd_sarif(filtered_items),
        SatdOutputFormat::Markdown => format_satd_markdown(filtered_items, evolution, days),
    }
}

/// Toyota Way: Extract Method - handle output writing (complexity ≤3)
async fn write_satd_output(output: Option<PathBuf>, content: &str) -> Result<()> {
    debug_assert!(!content.is_empty(), "content must not be empty");
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, content).await?;
        eprintln!("✅ SATD analysis written to: {}", output_path.display());
    } else {
        println!("{content}");
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_analyze_dag(
    dag_type: DagType,
    project_path: PathBuf,
    output: Option<PathBuf>,
    max_depth: Option<usize>,
    filter_external: bool,
    show_complexity: bool,
    include_duplicates: bool,
    include_dead_code: bool,
    enhanced: bool,
) -> Result<()> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    eprintln!("🔍 Analyzing Directed Acyclic Graph (DAG)...");
    eprintln!("📊 DAG Type: {dag_type:?}");
    eprintln!("📁 Project: {}", project_path.display());

    // Simple DAG analysis implementation
    let mut output_content = String::new();
    output_content.push_str(&format!("# {dag_type:?} DAG Analysis\n\n"));
    output_content.push_str(&format!("Project: {}\n", project_path.display()));

    if let Some(depth) = max_depth {
        output_content.push_str(&format!("Max depth: {depth}\n"));
    }

    output_content.push_str(&format!("Filter external: {filter_external}\n"));
    output_content.push_str(&format!("Show complexity: {show_complexity}\n"));
    output_content.push_str(&format!("Include duplicates: {include_duplicates}\n"));
    output_content.push_str(&format!("Include dead code: {include_dead_code}\n"));
    output_content.push_str(&format!("Enhanced mode: {enhanced}\n"));

    output_content.push_str("\n## Analysis Results\n");
    output_content.push_str(
        "DAG analysis functionality will be implemented with proper AST-based analysis.\n",
    );

    // Write output
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &output_content).await?;
        eprintln!("✅ DAG analysis written to: {}", output_path.display());
    } else {
        println!("{output_content}");
    }

    Ok(())
}
