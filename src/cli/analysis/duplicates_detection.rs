#[derive(Debug, Clone, Serialize, Deserialize)]
/// Duplicate block.
pub struct DuplicateBlock {
    pub hash: String,
    pub locations: Vec<DuplicateLocation>,
    pub lines: usize,
    pub tokens: usize,
    pub similarity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Duplicate location.
pub struct DuplicateLocation {
    pub file: String,
    pub start_line: usize,
    pub end_line: usize,
    pub content_preview: String,
}

#[derive(Debug, Serialize)]
/// Report containing duplicate data.
pub struct DuplicateReport {
    pub total_duplicates: usize,
    pub duplicate_lines: usize,
    pub total_lines: usize,
    pub duplication_percentage: f32,
    pub duplicate_blocks: Vec<DuplicateBlock>,
    /// DETERMINISM (round-3 sweep): a `BTreeMap`, not a `HashMap`. serde emits
    /// a map in iteration order, so `analyze duplicates --format json` listed
    /// the same per-file statistics under a different key order on every run.
    pub file_statistics: BTreeMap<String, FileStats>,
}

#[derive(Debug, Serialize)]
/// Statistics for file.
pub struct FileStats {
    pub duplicate_lines: usize,
    pub total_lines: usize,
    pub duplication_percentage: f32,
}

/// Main entry point for duplicate analysis
#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_analyze_duplicates(
    project_path: PathBuf,
    detection_type: crate::cli::DuplicateType,
    threshold: f32,
    min_lines: usize,
    max_tokens: usize,
    format: crate::cli::DuplicateOutputFormat,
    perf: bool,
    include: Option<String>,
    exclude: Option<String>,
    output: Option<PathBuf>,
    top_files: usize,
) -> Result<()> {
    // A nonexistent path previously produced an empty report and exit 0.
    crate::cli::ensure_analysis_path_exists(&project_path)?;

    {
        use crate::cli::colors as c;
        eprintln!("{}", c::dim("Analyzing code similarity..."));
    }

    let start_time = std::time::Instant::now();

    let mut report = run_duplicate_detection(
        &project_path,
        detection_type,
        threshold,
        min_lines,
        max_tokens,
        &include,
        &exclude,
    )
    .await?;

    apply_top_files_filtering(&mut report, top_files);
    print_duplicate_summary(&report);

    if perf {
        use crate::cli::colors as c;
        let duration = start_time.elapsed();
        eprintln!("\n{}Performance Metrics:{}", c::BOLD, c::RESET);
        eprintln!("   {}Analysis time:{} {}{:.2}ms{}", c::BOLD, c::RESET, c::BOLD_WHITE, duration.as_millis(), c::RESET);
        eprintln!("   {}Files processed:{} {}{}{}", c::BOLD, c::RESET, c::BOLD_WHITE, report.file_statistics.len(), c::RESET);
        eprintln!("   {}Blocks analyzed:{} {}{}{}", c::BOLD, c::RESET, c::BOLD_WHITE, report.duplicate_blocks.len(), c::RESET);
    }

    {
        use crate::cli::colors as c;
        eprintln!("\n{}", c::pass("Analysis Complete"));
    }

    write_duplicate_output(&report, format, output).await
}

/// Run duplicate detection analysis
async fn run_duplicate_detection(
    project_path: &Path,
    detection_type: crate::cli::DuplicateType,
    threshold: f32,
    min_lines: usize,
    max_tokens: usize,
    include: &Option<String>,
    exclude: &Option<String>,
) -> Result<DuplicateReport> {
    detect_duplicates(
        project_path,
        detection_type,
        threshold,
        min_lines,
        max_tokens,
        include,
        exclude,
    )
    .await
}

/// Apply top files filtering to report
fn apply_top_files_filtering(report: &mut DuplicateReport, top_files: usize) {
    if top_files == 0 {
        return;
    }

    let top_file_names = get_top_files_by_duplication(&report.file_statistics, top_files);
    filter_blocks_by_files(report, &top_file_names);
    recalculate_statistics_after_filtering(report);
}

/// Get top files by duplication percentage
fn get_top_files_by_duplication(
    file_statistics: &BTreeMap<String, FileStats>,
    top_files: usize,
) -> std::collections::HashSet<String> {
    let mut file_stats: Vec<_> = file_statistics.iter().collect();
    // DETERMINISM: duplication percentage is not a total order (whole trees tie
    // at 0.0), so without the path tie-break `.take(top_files)` kept whichever
    // tied files the map iteration happened to visit first.
    file_stats.sort_by(|a, b| {
        b.1.duplication_percentage
            .partial_cmp(&a.1.duplication_percentage)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(b.0))
    });

    file_stats
        .into_iter()
        .take(top_files)
        .map(|(name, _)| name.clone())
        .collect()
}

/// Filter blocks to only include those in specified files
fn filter_blocks_by_files(
    report: &mut DuplicateReport,
    top_file_names: &std::collections::HashSet<String>,
) {
    report.duplicate_blocks.retain(|block| {
        block
            .locations
            .iter()
            .any(|loc| top_file_names.contains(&loc.file))
    });
}

/// Recalculate statistics after filtering.
///
/// Counts DISTINCT physical lines that participate in duplication.
///
/// This was `block.lines * block.locations.len()`, which counts one physical
/// line once per window covering it. Detection uses an overlapping sliding
/// window, so a single 5-line function is reported at lines 2-6, 3-7 AND 4-8 —
/// three "locations" over the same 5 lines. Summing them produced
/// `duplicate_lines` GREATER than `total_lines` and percentages that cannot
/// exist: a 36-line fixture reported "Duplication: 455.6% (164 / 36 lines)",
/// and a file of four entirely distinct functions reported 211.5%.
///
/// A percentage above 100 is self-evidently fabricated, so this is measured as
/// a set: a line is duplicated, or it is not, however many windows cover it.
/// See contracts/pmat-no-fabrication-v1.yaml, equation `measured_or_absent`.
fn recalculate_statistics_after_filtering(report: &mut DuplicateReport) {
    let mut duplicated: std::collections::HashSet<(&str, usize)> = std::collections::HashSet::new();
    for block in &report.duplicate_blocks {
        for loc in &block.locations {
            for line in loc.start_line..=loc.end_line {
                duplicated.insert((loc.file.as_str(), line));
            }
        }
    }
    let duplicate_lines = duplicated.len();

    report.duplicate_lines = duplicate_lines;
    report.total_duplicates = report.duplicate_blocks.len();

    if report.total_lines > 0 {
        #[allow(clippy::cast_precision_loss)]
        let pct = (duplicate_lines as f32 / report.total_lines as f32) * 100.0;
        // Distinct duplicated lines can never exceed the lines counted, but
        // total_lines is gathered by a separate pass; clamp so a disagreement
        // between the two surfaces as 100%, never as an impossible number.
        report.duplication_percentage = pct.min(100.0);
    }
}

/// Print duplicate analysis summary
fn print_duplicate_summary(report: &DuplicateReport) {
    use crate::cli::colors as c;
    eprintln!(
        "{} Found {} duplicate blocks",
        c::pass(""),
        c::number(&report.total_duplicates.to_string())
    );
    eprintln!(
        "  {}Duplication:{} {} ({} / {} lines)",
        c::BOLD, c::RESET,
        c::pct(report.duplication_percentage as f64, 5.0, 15.0),
        c::number(&report.duplicate_lines.to_string()),
        c::number(&report.total_lines.to_string()),
    );
}

/// Write duplicate output to file or stdout
async fn write_duplicate_output(
    report: &DuplicateReport,
    format: crate::cli::DuplicateOutputFormat,
    output: Option<PathBuf>,
) -> Result<()> {
    let content = format_output(report, format)?;

    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        eprintln!("📄 Report written to: {}", output_path.display());
    } else {
        println!("{content}");
    }

    Ok(())
}

/// Detect duplicate code blocks
async fn detect_duplicates(
    project_path: &Path,
    detection_type: crate::cli::DuplicateType,
    threshold: f32,
    min_lines: usize,
    max_tokens: usize,
    include: &Option<String>,
    exclude: &Option<String>,
) -> Result<DuplicateReport> {
    let (all_blocks, total_lines, mut file_stats) = collect_code_blocks(
        project_path,
        detection_type,
        min_lines,
        max_tokens,
        include,
        exclude,
    )
    .await?;

    let duplicate_blocks = find_duplicate_blocks(all_blocks, threshold);
    let duplicate_lines = calculate_duplicate_statistics(&duplicate_blocks, &mut file_stats);
    let duplication_percentage = calculate_duplication_percentage(duplicate_lines, total_lines);

    Ok(build_duplicate_report(
        duplicate_blocks,
        duplicate_lines,
        total_lines,
        duplication_percentage,
        file_stats,
    ))
}

/// Collect code blocks from all source files
async fn collect_code_blocks(
    project_path: &Path,
    detection_type: crate::cli::DuplicateType,
    min_lines: usize,
    max_tokens: usize,
    include: &Option<String>,
    exclude: &Option<String>,
) -> Result<(
    Vec<(String, String, usize, usize, String)>,
    usize,
    BTreeMap<String, FileStats>,
)> {
    use crate::services::file_discovery::ProjectFileDiscovery;

    let mut all_blocks = Vec::new();
    let mut total_lines = 0usize;
    let mut file_stats = BTreeMap::new();

    let discovered_files = ProjectFileDiscovery::new(project_path.to_path_buf())
        .discover_files()
        .unwrap_or_default();

    for path in discovered_files {
        let path = path.as_path();

        if should_analyze_file(path, include, exclude) {
            if let Some((blocks, lines_count)) =
                process_source_file(path, detection_type.clone(), min_lines, max_tokens).await
            {
                all_blocks.extend(blocks);
                total_lines += lines_count;

                file_stats.insert(
                    path.to_string_lossy().to_string(),
                    FileStats {
                        duplicate_lines: 0,
                        total_lines: lines_count,
                        duplication_percentage: 0.0,
                    },
                );
            }
        }
    }

    Ok((all_blocks, total_lines, file_stats))
}

/// Check if file should be analyzed
fn should_analyze_file(path: &Path, include: &Option<String>, exclude: &Option<String>) -> bool {
    path.is_file() && is_source_file(path) && should_process_file(path, include, exclude)
}

/// Process a single source file for duplicate detection
async fn process_source_file(
    path: &Path,
    detection_type: crate::cli::DuplicateType,
    min_lines: usize,
    max_tokens: usize,
) -> Option<(Vec<(String, String, usize, usize, String)>, usize)> {
    if let Ok(content) = tokio::fs::read_to_string(path).await {
        let lines: Vec<&str> = content.lines().collect();
        let blocks = extract_blocks(&lines, path, min_lines, max_tokens, detection_type);
        Some((blocks, lines.len()))
    } else {
        None
    }
}

/// Calculate duplicate statistics and update file stats
fn calculate_duplicate_statistics(
    duplicate_blocks: &[DuplicateBlock],
    file_stats: &mut BTreeMap<String, FileStats>,
) -> usize {
    let mut duplicate_lines = 0;

    for block in duplicate_blocks {
        duplicate_lines += block.lines * block.locations.len();

        for loc in &block.locations {
            if let Some(stats) = file_stats.get_mut(&loc.file) {
                stats.duplicate_lines += block.lines;
            }
        }
    }

    update_file_duplication_percentages(file_stats);
    duplicate_lines
}

/// Update duplication percentages for all files
fn update_file_duplication_percentages(file_stats: &mut BTreeMap<String, FileStats>) {
    for stats in file_stats.values_mut() {
        if stats.total_lines > 0 {
            stats.duplication_percentage =
                (stats.duplicate_lines as f32 / stats.total_lines as f32) * 100.0;
        }
    }
}

/// Calculate overall duplication percentage
fn calculate_duplication_percentage(duplicate_lines: usize, total_lines: usize) -> f32 {
    if total_lines > 0 {
        (duplicate_lines as f32 / total_lines as f32) * 100.0
    } else {
        0.0
    }
}

/// Build the final duplicate report
fn build_duplicate_report(
    duplicate_blocks: Vec<DuplicateBlock>,
    duplicate_lines: usize,
    total_lines: usize,
    duplication_percentage: f32,
    file_stats: BTreeMap<String, FileStats>,
) -> DuplicateReport {
    DuplicateReport {
        total_duplicates: duplicate_blocks.len(),
        duplicate_lines,
        total_lines,
        duplication_percentage,
        duplicate_blocks,
        file_statistics: file_stats,
    }
}
