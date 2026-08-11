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

    // `--top-files` used to be applied HERE, dropping duplicate blocks and then
    // recomputing the report from what survived: the same tree reported 12.2%
    // duplication at `--top-files 1` and 19.4% at `--top-files 0`, and one
    // 301-line file read as "17.6% (53 / 301 lines)" or "33.6% (101 / 301
    // lines)" depending only on how many rows the user asked to see. A display
    // limit cannot be allowed to change the measurement, so the report is now
    // whole-project and `top_files` travels to the renderer instead.
    let report = run_duplicate_detection(
        &project_path,
        detection_type,
        threshold,
        min_lines,
        max_tokens,
        &include,
        &exclude,
    )
    .await?;

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

    write_duplicate_output(&report, format, output, top_files).await
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod top_files_does_not_change_the_measurement_tests {
    use super::*;

    /// `--top-files N` dropped every duplicate block outside the top N files and
    /// then restated the report from the survivors, so the SAME tree reported
    /// "8 blocks / 12.2%" at `--top-files 1`, "17 / 14.8%" at 3 and "32 / 19.4%"
    /// at 0 — the answer moved with the size of the list the user asked to see.
    #[tokio::test]
    async fn duplication_metrics_are_independent_of_top_files() {
        use tempfile::TempDir;

        let project = TempDir::new().unwrap();
        let out = TempDir::new().unwrap();
        // Two independent clone families plus one unique file: a top-N cut over
        // files drops the whole second family, which is what used to move the
        // headline numbers.
        let family_a: String = (0..14)
            .map(|line| format!("    let a{} = {line} * 3;\n", line % 3))
            .collect();
        let family_b: String = (0..14)
            .map(|line| format!("    let b{} = {line} - 7;\n", line % 4))
            .collect();
        for name in ["a0.rs", "a1.rs", "a2.rs"] {
            std::fs::write(project.path().join(name), &family_a).unwrap();
        }
        for name in ["b0.rs", "b1.rs"] {
            std::fs::write(project.path().join(name), &family_b).unwrap();
        }
        std::fs::write(
            project.path().join("c0.rs"),
            "fn unique() {\n    let only = 1;\n}\n",
        )
        .unwrap();

        let mut baseline: Option<(u64, u64, String)> = None;
        for top_files in [1usize, 2, 3, 10, 0] {
            let report_path = out.path().join(format!("dup-{top_files}.json"));
            handle_analyze_duplicates(
                project.path().to_path_buf(),
                crate::cli::DuplicateType::Exact,
                0.8,
                5,
                100,
                crate::cli::DuplicateOutputFormat::Json,
                false,
                None,
                None,
                Some(report_path.clone()),
                top_files,
            )
            .await
            .expect("analysis must succeed");

            let json: serde_json::Value =
                serde_json::from_str(&std::fs::read_to_string(&report_path).unwrap()).unwrap();
            let measured = (
                json["total_duplicates"].as_u64().unwrap(),
                json["duplicate_lines"].as_u64().unwrap(),
                format!("{:.4}", json["duplication_percentage"].as_f64().unwrap()),
            );

            match &baseline {
                None => baseline = Some(measured),
                Some(expected) => assert_eq!(
                    *expected, measured,
                    "--top-files {top_files} changed the measured duplication"
                ),
            }
        }
    }
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
///
/// `top_files` limits the rendered "Top Files by Duplication" list and nothing
/// else — the machine-readable formats carry the whole measurement.
async fn write_duplicate_output(
    report: &DuplicateReport,
    format: crate::cli::DuplicateOutputFormat,
    output: Option<PathBuf>,
    top_files: usize,
) -> Result<()> {
    let content = match format {
        // The three text formats used to collapse onto one renderer here, so
        // `--format summary` printed the full per-block listing and
        // `--format detailed` printed no more than `--format human`.
        crate::cli::DuplicateOutputFormat::Human => {
            format_text_output(report, top_files, TextDetail::Human)?
        }
        crate::cli::DuplicateOutputFormat::Summary => {
            format_text_output(report, top_files, TextDetail::Summary)?
        }
        crate::cli::DuplicateOutputFormat::Detailed => {
            format_text_output(report, top_files, TextDetail::Detailed)?
        }
        other => format_output(report, other)?,
    };

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

/// Count the DISTINCT physical lines that participate in duplication, per file.
///
/// Detection uses an overlapping sliding window, so one physical line is
/// covered by several blocks that hash differently (a.rs 1-11, 1-5, 2-6, 3-7
/// and 4-8 all appear in one run). Summing `block.lines` per location counted
/// each of those lines once per window: two byte-identical 17-line files were
/// reported as `duplicate_lines: 76, total_lines: 17,
/// duplication_percentage: 447.06` — a part 4.5x its own whole, printed to the
/// user as "1. a.rs - 447.1% duplication (76 / 17 lines)".
///
/// A line is duplicated or it is not, however many windows cover it. See
/// `contracts/pmat-no-fabrication-v1.yaml`, equation `measured_or_absent`.
fn calculate_duplicate_statistics(
    duplicate_blocks: &[DuplicateBlock],
    file_stats: &mut BTreeMap<String, FileStats>,
) -> usize {
    let mut duplicated: HashMap<&str, std::collections::HashSet<usize>> = HashMap::new();
    for block in duplicate_blocks {
        for loc in &block.locations {
            let lines = duplicated.entry(loc.file.as_str()).or_default();
            for line in loc.start_line..=loc.end_line {
                lines.insert(line);
            }
        }
    }

    for (path, stats) in file_stats.iter_mut() {
        let counted = duplicated.get(path.as_str()).map_or(0, |set| {
            // A block may name a line past the end of the file if extraction
            // over-ran; clamp so no part can exceed its whole.
            set.iter().filter(|line| **line <= stats.total_lines).count()
        });
        stats.duplicate_lines = counted;
        stats.duplication_percentage = if stats.total_lines > 0 {
            #[allow(clippy::cast_precision_loss)]
            let pct = (counted as f32 / stats.total_lines as f32) * 100.0;
            pct.min(100.0)
        } else {
            0.0
        };
    }

    // The project total is the sum of the per-file totals, so the headline and
    // the rows can never disagree.
    file_stats.values().map(|s| s.duplicate_lines).sum()
}

/// Calculate overall duplication percentage.
///
/// Clamped at 100: distinct duplicated lines can never exceed the lines
/// counted, but the two come from separate passes, so a disagreement must
/// surface as 100%, never as an impossible number.
fn calculate_duplication_percentage(duplicate_lines: usize, total_lines: usize) -> f32 {
    if total_lines > 0 {
        #[allow(clippy::cast_precision_loss)]
        let pct = (duplicate_lines as f32 / total_lines as f32) * 100.0;
        pct.min(100.0)
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
