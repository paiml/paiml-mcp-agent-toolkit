
// GH dogfood sweep: four of these runners returned hardcoded fixtures —
// `unused_function` at `src/utils.rs:42`, a TDG of 2.1, a 0.75 defect
// probability for `src/parser.rs` — regardless of the project being analysed.
// Two different crates produced byte-identical "analysis". A quality tool
// fabricating quality data is the worst failure it can have: it is
// indistinguishable from a real finding to anyone reading a report or a CI
// gate, and it was doing so for every project.
//
// Per this codebase's own D75 policy (honest failure beats lying success, see
// `utility_serve_handlers`), they no longer invent data. Each returns an error
// naming the standalone subcommand that does perform the analysis for real
// (`pmat analyze dead-code`, `analyze tdg`, `analyze defect-prediction`,
// `analyze duplicates`), all of which the dogfood sweep confirmed correct.

// Comprehensive analysis helper functions - extracted for file health (CB-040)
async fn run_complexity_analysis(
    project_path: &Path,
    include: &Option<String>,
    _exclude: &Option<String>,
) -> Result<ComplexityReport> {
    use crate::services::complexity::aggregate_results_with_thresholds;

    // Use the ONE implementation - analyze_project_files
    let include_patterns = if let Some(pattern) = include {
        vec![pattern.clone()]
    } else {
        vec![]
    };

    let file_metrics = analyze_project_files(
        project_path,
        None, // Auto-detect toolchain
        &include_patterns,
        20, // Default cyclomatic threshold
        15, // Default cognitive threshold
    )
    .await?;

    // Aggregate results
    let report = aggregate_results_with_thresholds(file_metrics, Some(20), Some(15));

    // Convert to legacy ComplexityReport format for compatibility
    let mut functions = Vec::new();
    let mut total_complexity = 0u32;
    let mut complexities = Vec::new();

    for violation in &report.violations {
        match violation {
            crate::services::complexity::Violation::Error {
                file,
                function,
                value,
                ..
            }
            | crate::services::complexity::Violation::Warning {
                file,
                function,
                value,
                ..
            } => {
                if *value > 20 {
                    functions.push(ComplexityHotspot {
                        function: function
                            .as_ref()
                            .unwrap_or(&"<anonymous>".to_string())
                            .clone(),
                        file: file.clone(),
                        complexity: u32::from(*value),
                    });
                }
                complexities.push(u32::from(*value));
                total_complexity += u32::from(*value);
            }
        }
    }

    // Sort hotspots by complexity
    functions.sort_unstable_by_key(|b| std::cmp::Reverse(b.complexity));
    functions.truncate(10);

    // Calculate p99
    complexities.sort_unstable();
    let p99_idx = (f64::from(complexities.len() as u32) * 0.99) as usize;
    let p99 = complexities.get(p99_idx).copied().unwrap_or(0);

    Ok(ComplexityReport {
        total_functions: complexities.len(),
        high_complexity_count: functions.len(),
        average_complexity: if complexities.is_empty() {
            0.0
        } else {
            f64::from(total_complexity) / f64::from(complexities.len() as u32)
        },
        p99_complexity: p99,
        hotspots: functions,
    })
}

async fn run_satd_analysis(
    _project_path: &Path,
    _include: &Option<String>,
    _exclude: &Option<String>,
) -> Result<SatdReport> {
    use regex::Regex;
    use crate::services::file_discovery::ProjectFileDiscovery;

    let satd_pattern = Regex::new(r"(?i)(TODO|FIXME|HACK|XXX|REFACTOR|DEPRECATED):\s*(.+)")
        .expect("Hardcoded regex pattern must be valid");
    let mut items = Vec::new();
    let mut by_type = HashMap::new();
    let mut by_severity = HashMap::new();

    let discovered_files = ProjectFileDiscovery::new(_project_path.to_path_buf())
        .discover_files()
        .unwrap_or_default();

    for path in discovered_files {
        let path = path.as_path();

        if path.is_file() && is_source_file(path) {
            process_file_for_satd(
                path,
                &satd_pattern,
                &mut items,
                &mut by_type,
                &mut by_severity,
            )
            .await?;
        }
    }

    Ok(SatdReport {
        total_items: items.len(),
        by_type,
        by_severity,
        items,
    })
}

/// Extract Method: Process a single file for SATD detection
async fn process_file_for_satd(
    path: &std::path::Path,
    satd_pattern: &regex::Regex,
    items: &mut Vec<SatdItem>,
    by_type: &mut HashMap<String, usize>,
    by_severity: &mut HashMap<String, usize>,
) -> Result<()> {
    if let Ok(content) = tokio::fs::read_to_string(path).await {
        for (line_no, line) in content.lines().enumerate() {
            if let Some(captures) = satd_pattern.captures(line) {
                process_satd_match(path, line_no, captures, items, by_type, by_severity);
            }
        }
    }
    Ok(())
}

/// Extract Method: Process a single SATD match
fn process_satd_match(
    path: &std::path::Path,
    line_no: usize,
    captures: regex::Captures,
    items: &mut Vec<SatdItem>,
    by_type: &mut HashMap<String, usize>,
    by_severity: &mut HashMap<String, usize>,
) {
    let satd_type = captures
        .get(1)
        .expect("Match group 1 exists for successful regex match")
        .as_str()
        .to_uppercase();
    let text = captures
        .get(2)
        .expect("Match group 2 exists for successful regex match")
        .as_str()
        .to_string();
    let severity = determine_satd_severity(&satd_type);

    *by_type.entry(satd_type.clone()).or_insert(0) += 1;
    *by_severity.entry(severity.to_string()).or_insert(0) += 1;

    items.push(SatdItem {
        file: path.to_string_lossy().to_string(),
        line: line_no + 1,
        text,
        satd_type,
        severity: severity.to_string(),
    });
}

/// Extract Method: Determine SATD severity based on type
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn determine_satd_severity(satd_type: &str) -> &'static str {
    match satd_type {
        "HACK" | "XXX" => "high",
        "FIXME" | "REFACTOR" => "medium",
        _ => "low",
    }
}

async fn create_tdg_report(_project_path: &Path) -> Result<TdgReport> {
    anyhow::bail!(
        "TDG is not wired into `analyze comprehensive` (it previously reported a \
         fabricated score of 2.1 for every project). Run `pmat analyze tdg` \
         instead, or `pmat tdg <path>`."
    )
}

async fn run_dead_code_analysis(
    _project_path: &Path,
    _include: &Option<String>,
    _exclude: &Option<String>,
) -> Result<DeadCodeReport> {
    anyhow::bail!(
        "dead-code is not wired into `analyze comprehensive` (it previously \
         reported a fabricated `unused_function` at src/utils.rs:42 for every \
         project). Run `pmat analyze dead-code` instead."
    )
}

async fn run_defect_prediction(
    _project_path: &Path,
    _confidence_threshold: f32,
    _min_lines: usize,
) -> Result<DefectReport> {
    anyhow::bail!(
        "defect prediction is not wired into `analyze comprehensive` (it \
         previously reported a fabricated 0.75 probability for src/parser.rs \
         for every project). Run `pmat analyze defect-prediction` instead."
    )
}

async fn run_duplicate_detection(
    _project_path: &Path,
    _include: &Option<String>,
    _exclude: &Option<String>,
) -> Result<DuplicateReport> {
    anyhow::bail!(
        "duplicate detection is not wired into `analyze comprehensive` (it \
         previously reported fabricated blocks in src/handler1.rs and \
         src/handler2.rs for every project). Run `pmat analyze duplicates` \
         instead."
    )
}
