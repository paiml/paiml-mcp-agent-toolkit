/// Format output based on format type
fn format_output(
    report: &DuplicateReport,
    format: crate::cli::DuplicateOutputFormat,
    top_files: usize,
) -> Result<String> {
    match format {
        crate::cli::DuplicateOutputFormat::Json => format_json_output(report, top_files),
        crate::cli::DuplicateOutputFormat::Human => {
            format_text_output(report, top_files, TextDetail::Human)
        }
        crate::cli::DuplicateOutputFormat::Summary => {
            format_text_output(report, top_files, TextDetail::Summary)
        }
        crate::cli::DuplicateOutputFormat::Detailed => {
            format_text_output(report, top_files, TextDetail::Detailed)
        }
        crate::cli::DuplicateOutputFormat::Sarif => format_sarif_output(report, top_files),
        crate::cli::DuplicateOutputFormat::Csv => format_csv_output(report, top_files),
    }
}

/// The rows a machine-readable rendering LISTS, and the totals it left out.
///
/// Issue #1050 P10. `--top-files` reached the three TEXT renderers and stopped
/// there: `analyze duplicates --format json` and `--format json --top-files 5`
/// were byte-identical, at 853,863 bytes on copia and **387 MB on depyler**.
/// The document carried no limit, no truncation control and no truncation
/// marker, so a machine consumer had no bounded way to read this command at all
/// — and no way to tell a complete listing from a clipped one.
///
/// `--top-files N` is documented as "Number of top files to show by duplication
/// (0 = all)", so the listing is the top N FILES and the blocks that fall in
/// them. What it is NOT allowed to touch is the measurement: `total_duplicates`,
/// `duplicate_lines`, `duplication_percentage` and the per-class counts stay
/// whole-project, exactly as the text path already requires (see
/// `top_files_does_not_change_the_measurement_tests`). A display limit that
/// moved the headline number is the defect this one replaced, not a licence to
/// reintroduce it.
struct DuplicateListing<'a> {
    /// The listed files, most-duplicated first — the same order, and the same
    /// `0 = all` rule, as the text renderer's "Top Files by Duplication".
    files: Vec<(&'a String, &'a FileStats)>,
    /// Every block with at least one location in a listed file.
    blocks: Vec<&'a DuplicateBlock>,
    /// Files the run read, before the limit.
    files_total: usize,
    /// Blocks the run found, before the limit.
    blocks_total: usize,
    /// The limit as the user gave it, so the disclosure names the knob.
    top_files: usize,
}

impl<'a> DuplicateListing<'a> {
    fn of(report: &'a DuplicateReport, top_files: usize) -> Self {
        let sorted = get_sorted_file_stats(&report.file_statistics);
        let files: Vec<_> = crate::cli::top_files_slice(&sorted, top_files).to_vec();
        let listed: std::collections::HashSet<&str> =
            files.iter().map(|(path, _)| path.as_str()).collect();
        // A block is listed when one of its sites is in a listed file — OR
        // when NONE of its sites is a file this report has statistics for.
        //
        // The second clause is not a corner case, it is the honesty rule: a
        // finding that cannot be attributed to any counted file cannot be
        // withheld on the grounds that its file did not make the top N, because
        // no top-N decision was ever made about it. Dropping unattributable
        // findings silently is the defect class this whole listing exists to
        // close, one level down.
        let blocks = report
            .duplicate_blocks
            .iter()
            .filter(|block| {
                block
                    .locations
                    .iter()
                    .any(|loc| listed.contains(loc.file.as_str()))
                    || !block
                        .locations
                        .iter()
                        .any(|loc| report.file_statistics.contains_key(&loc.file))
            })
            .collect();
        Self {
            files,
            blocks,
            files_total: report.file_statistics.len(),
            blocks_total: report.duplicate_blocks.len(),
            top_files,
        }
    }

    /// What this listing shows and what it hides, in the same document.
    ///
    /// Always present, including when nothing was dropped: "the listing is
    /// complete" is a claim a consumer needs to be able to READ, and an absent
    /// field is not one.
    fn disclosure(&self) -> serde_json::Value {
        serde_json::json!({
            "top_files": self.top_files,
            "files_total": self.files_total,
            "files_listed": self.files.len(),
            "files_truncated": self.files.len() < self.files_total,
            "blocks_total": self.blocks_total,
            "blocks_listed": self.blocks.len(),
            "blocks_truncated": self.blocks.len() < self.blocks_total,
        })
    }
}

/// How much of the report a text rendering carries.
///
/// `--format summary`, `--format detailed` and `--format human` all routed to
/// `format_human_output`, so the three produced byte-identical output: the
/// format documented as "Summary statistics only" printed the whole per-block
/// detail listing, and "Detailed duplicate listing" silently stopped at the
/// twentieth block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextDetail {
    /// Header + summary + top files. No per-block listing.
    Summary,
    /// Summary plus the first [`HUMAN_BLOCK_LIMIT`] blocks.
    Human,
    /// Summary plus EVERY block.
    Detailed,
}

/// Blocks the default (`human`) rendering lists before summarising the rest.
const HUMAN_BLOCK_LIMIT: usize = 20;

/// How many blocks of this report were measured to be `clone_type`.
fn count_of(report: &DuplicateReport, clone_type: CloneType) -> usize {
    report
        .duplicate_blocks
        .iter()
        .filter(|b| b.clone_type == clone_type)
        .count()
}

/// Format output as JSON
fn format_json_output(report: &DuplicateReport, top_files: usize) -> Result<String> {
    let listing = DuplicateListing::of(report, top_files);
    // Create enhanced JSON with test-expected fields
    let enhanced_json = serde_json::json!({
        "total_duplicates": report.total_duplicates,
        "duplicate_lines": report.duplicate_lines,
        "total_lines": report.total_lines,
        "duplication_percentage": report.duplication_percentage,
        // Bounded by `--top-files`, and the bound is declared beside it under
        // `listing`. See [`DuplicateListing`]: this document used to be the
        // whole engine output with no limit and no marker — 387 MB on depyler —
        // while the flag that exists to bound it changed not one byte.
        "duplicate_blocks": listing.blocks,
        "file_statistics": listing
            .files
            .iter()
            .map(|(path, stats)| ((*path).clone(), stats))
            .collect::<std::collections::BTreeMap<_, _>>(),
        "listing": listing.disclosure(),
        // Each count reads the block's MEASURED clone class. `exact_duplicates`
        // used to be `similarity >= 1.0` over blocks whose only producer wrote
        // `similarity: 1.0` as a literal, so it equalled `total_duplicates` for
        // every input and every `--detection-type`: under `renamed` it claimed
        // that clones which differ on 8 of 39 lines are byte identical.
        "exact_duplicates": count_of(report, CloneType::Exact),
        // Type-2: identical up to identifier names. It had no count at all, so
        // the only clone class `--detection-type renamed` can find was reported
        // under the name of the one class it cannot.
        "renamed_duplicates": count_of(report, CloneType::Renamed),
        // Near-miss (Type-3) groups: blocks whose members are similar but NOT
        // identical. The predicate used to be `0.8 <= s < 1.0`, and the only
        // producer of a block set `similarity: 1.0` as a literal, so this was
        // unsatisfiable for every possible input — a hard 0 printed beside a
        // real `exact_duplicates` count, which is what made it look measured.
        // It counts the class now rather than re-deriving one from a similarity
        // band, so a change to how similarity is measured can never silently
        // empty it again.
        "structural_similarities": count_of(report, CloneType::NearMiss),
        // `entropy_analysis` and `analysis_time_ms` used to be emitted here as
        // the constants 0.5 / 0 / 100 in EVERY run, regardless of input. This
        // command runs no entropy analysis and did not time itself, so those
        // were fabricated measurements sitting beside real ones -- which is
        // precisely what makes them dangerous: the real neighbours lend them
        // credibility. They are omitted rather than defaulted; a consumer that
        // needs entropy should call `pmat analyze entropy`, which measures it.
        //
        // See contracts/pmat-no-fabrication-v1.yaml, equation `measured_or_absent`.
        // The MEASUREMENT, whole-project and untouched by the listing limit.
        "metrics": {
            "files_processed": report.file_statistics.len(),
            "blocks_analyzed": report.duplicate_blocks.len()
        }
    });

    Ok(serde_json::to_string_pretty(&enhanced_json)?)
}

/// Format output for human reading
///
/// # Example
///
/// ```no_run
/// use pmat::cli::analysis::duplicates::{format_human_output, DuplicateReport, FileStats};
/// use std::collections::BTreeMap;
///
/// let mut file_stats = BTreeMap::new();
/// file_stats.insert("src/main.rs".to_string(), FileStats {
///     duplicate_lines: 10,
///     total_lines: 100,
///     duplication_percentage: 10.0,
/// });
/// file_stats.insert("src/lib.rs".to_string(), FileStats {
///     duplicate_lines: 5,
///     total_lines: 50,
///     duplication_percentage: 10.0,
/// });
///
/// let report = DuplicateReport {
///     total_duplicates: 2,
///     duplicate_lines: 15,
///     total_lines: 150,
///     duplication_percentage: 10.0,
///     duplicate_blocks: vec![],
///     file_statistics: file_stats,
/// };
///
/// let output = format_human_output(&report).unwrap();
///
/// assert!(output.contains("Duplicate Code Analysis"));
/// assert!(output.contains("Total duplicate blocks:"));
/// assert!(output.contains("Top Files by Duplication"));
/// assert!(output.contains("main.rs"));
/// ```
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_human_output(report: &DuplicateReport) -> Result<String> {
    format_human_output_with_limit(report, DEFAULT_TOP_FILES)
}

/// The CLI's `--top-files` default; also the row limit `format_human_output`
/// renders with when no explicit limit is supplied.
const DEFAULT_TOP_FILES: usize = 10;

/// Same report, with the "Top Files by Duplication" list limited to `top_files`
/// rows (`0` = every file).
///
/// The limit reaches the renderer instead of the report: `--top-files` is a
/// display control, and the row list was independently hardcoded to ten rows,
/// so `--top-files 3` printed ten.
pub fn format_human_output_with_limit(
    report: &DuplicateReport,
    top_files: usize,
) -> Result<String> {
    format_text_output(report, top_files, TextDetail::Human)
}

/// The one text renderer behind `--format summary|human|detailed`; `detail`
/// decides how much of the block listing it carries.
fn format_text_output(
    report: &DuplicateReport,
    top_files: usize,
    detail: TextDetail,
) -> Result<String> {
    let mut output = String::new();

    write_header(&mut output)?;
    write_summary(&mut output, report)?;
    write_top_files_section(&mut output, report, top_files)?;
    if detail != TextDetail::Summary {
        write_duplicate_blocks_section(&mut output, report, detail)?;
    }

    Ok(output)
}

/// Write the header section
fn write_header(output: &mut String) -> Result<()> {
    use crate::cli::colors as c;
    use std::fmt::Write;
    writeln!(output, "{}", c::header("Duplicate Code Analysis"))?;
    writeln!(output)?;
    Ok(())
}

/// Write the summary section
fn write_summary(output: &mut String, report: &DuplicateReport) -> Result<()> {
    use crate::cli::colors as c;
    use std::fmt::Write;

    writeln!(output, "{}", c::subheader("Summary"))?;
    writeln!(
        output,
        "  Total duplicate blocks: {}",
        c::number(&report.total_duplicates.to_string())
    )?;
    writeln!(
        output,
        "  Duplicate lines: {} / {}",
        c::number(&report.duplicate_lines.to_string()),
        c::number(&report.total_lines.to_string())
    )?;
    writeln!(
        output,
        "  Duplication percentage: {}\n",
        c::pct(report.duplication_percentage as f64, 5.0, 15.0)
    )?;

    Ok(())
}

/// Write the top files by duplication section
fn write_top_files_section(
    output: &mut String,
    report: &DuplicateReport,
    top_files: usize,
) -> Result<()> {
    if report.file_statistics.is_empty() {
        return Ok(());
    }

    use crate::cli::colors as c;
    use std::fmt::Write;
    writeln!(output, "{}\n", c::subheader("Top Files by Duplication"))?;

    let sorted_files = get_sorted_file_stats(&report.file_statistics);
    write_file_stats_list(output, &sorted_files, top_files)?;

    Ok(())
}

/// Get file statistics sorted by duplication percentage
fn get_sorted_file_stats(
    file_stats: &std::collections::BTreeMap<String, FileStats>,
) -> Vec<(&String, &FileStats)> {
    let mut sorted_files: Vec<_> = file_stats.iter().collect();
    // DETERMINISM: path breaks ties so "Top Files by Duplication" is a function
    // of the tree, not of the map's iteration order.
    sorted_files.sort_by(|a, b| {
        b.1.duplication_percentage
            .partial_cmp(&a.1.duplication_percentage)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(b.0))
    });
    sorted_files
}

/// Write the list of file statistics
fn write_file_stats_list(
    output: &mut String,
    sorted_files: &[(&String, &FileStats)],
    top_files: usize,
) -> Result<()> {
    use crate::cli::colors as c;
    use std::fmt::Write;

    // Was `.take(10)` regardless of `--top-files`, so the flag documented as
    // "Number of top files to show by duplication (0 = all)" printed ten rows
    // for `--top-files 3` and ten rows for `--top-files 0`. The "0 = all" rule
    // now lives in exactly one place, `crate::cli::top_files_slice`.
    for (i, (file_path, stats)) in crate::cli::top_files_slice(sorted_files, top_files)
        .iter()
        .enumerate()
    {
        // Was `extract_filename`, which printed only the basename: this repo has
        // two `core_tests_properties.rs`, so slots 1 and 4 of the top-ten were
        // the same text with the same numbers and no way to tell them apart.
        let filename = crate::cli::report_paths::report_path(file_path);
        writeln!(
            output,
            "  {}. {} - {} duplication ({} / {} lines)",
            c::number(&(i + 1).to_string()),
            c::path(filename),
            c::pct(stats.duplication_percentage as f64, 5.0, 15.0),
            c::number(&stats.duplicate_lines.to_string()),
            c::number(&stats.total_lines.to_string()),
        )?;
    }
    writeln!(output)?;
    Ok(())
}

/// Write the duplicate blocks section
fn write_duplicate_blocks_section(
    output: &mut String,
    report: &DuplicateReport,
    detail: TextDetail,
) -> Result<()> {
    if report.duplicate_blocks.is_empty() {
        return Ok(());
    }

    use crate::cli::colors as c;
    use std::fmt::Write;
    writeln!(output, "{}\n", c::subheader("Duplicate Blocks"))?;

    // `detailed` is documented as the "Detailed duplicate listing"; capping it
    // at twenty blocks like the default rendering is what made it identical to
    // `human`.
    let block_limit = match detail {
        TextDetail::Detailed => usize::MAX,
        _ => HUMAN_BLOCK_LIMIT,
    };
    write_block_details(output, &report.duplicate_blocks, block_limit)?;
    if block_limit != usize::MAX {
        write_remaining_blocks_count(output, report.duplicate_blocks.len())?;
    }

    Ok(())
}

/// Write detailed information about duplicate blocks
fn write_block_details(
    output: &mut String,
    duplicate_blocks: &[DuplicateBlock],
    limit: usize,
) -> Result<()> {
    for (i, block) in duplicate_blocks.iter().enumerate().take(limit) {
        write_block_header(output, i + 1, block)?;
        write_block_locations(output, block)?;
        write_block_preview(output, block)?;
    }
    Ok(())
}

/// Write block header with summary info.
///
/// Carries the same measured clone class the JSON and CSV surfaces carry: the
/// text rendering used to say only "N lines, M locations", so the one surface a
/// human reads could not tell an exact clone from a renamed one while the JSON
/// beside it claimed everything was exact.
fn write_block_header(output: &mut String, block_num: usize, block: &DuplicateBlock) -> Result<()> {
    use crate::cli::colors as c;
    use std::fmt::Write;
    writeln!(
        output,
        "  {}Block {}{} ({}, {} lines, {} locations)",
        c::seq(c::BOLD),
        block_num,
        c::seq(c::RESET),
        block.clone_type.label(),
        c::number(&block.lines.to_string()),
        c::number(&block.locations.len().to_string()),
    )?;
    Ok(())
}

/// Write block location information
fn write_block_locations(output: &mut String, block: &DuplicateBlock) -> Result<()> {
    use crate::cli::colors as c;
    use std::fmt::Write;
    for loc in &block.locations {
        writeln!(
            output,
            "    {}{}{}:{}{}{}-{}{}{}",
            c::seq(c::CYAN),
            loc.file,
            c::seq(c::RESET),
            c::seq(c::BOLD_WHITE),
            loc.start_line,
            c::seq(c::RESET),
            c::seq(c::BOLD_WHITE),
            loc.end_line,
            c::seq(c::RESET),
        )?;
    }
    Ok(())
}

/// Write block content preview
fn write_block_preview(output: &mut String, block: &DuplicateBlock) -> Result<()> {
    use crate::cli::colors as c;
    use std::fmt::Write;
    writeln!(output, "    {}Preview:{}", c::seq(c::DIM), c::seq(c::RESET))?;
    writeln!(
        output,
        "    {}{}{}",
        c::seq(c::DIM),
        block.locations[0].content_preview,
        c::seq(c::RESET)
    )?;
    writeln!(output)?;
    Ok(())
}

/// Write count of remaining blocks if there are more than 20
fn write_remaining_blocks_count(output: &mut String, total_blocks: usize) -> Result<()> {
    if total_blocks > HUMAN_BLOCK_LIMIT {
        use crate::cli::colors as c;
        use std::fmt::Write;
        writeln!(
            output,
            "  {}... and {} more blocks{}",
            c::seq(c::DIM),
            total_blocks - HUMAN_BLOCK_LIMIT,
            c::seq(c::RESET)
        )?;
    }
    Ok(())
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod top_files_is_a_row_limit_tests {
    use super::*;

    fn report_with_files(n: usize) -> DuplicateReport {
        let mut file_statistics = BTreeMap::new();
        for i in 0..n {
            file_statistics.insert(
                format!("src/f{i:02}.rs"),
                FileStats {
                    duplicate_lines: n - i,
                    total_lines: 100,
                    duplication_percentage: (n - i) as f32,
                },
            );
        }
        DuplicateReport {
            total_duplicates: 7,
            duplicate_lines: 42,
            total_lines: 1000,
            duplication_percentage: 4.2,
            duplicate_blocks: vec![],
            file_statistics,
        }
    }

    fn row_count(rendered: &str) -> usize {
        rendered
            .lines()
            .filter(|l| l.contains(" duplication ("))
            .count()
    }

    /// The list was hardcoded to `.take(10)`, so every value of `--top-files`
    /// printed ten rows.
    #[test]
    fn top_files_limits_the_printed_rows() {
        let report = report_with_files(25);
        for requested in [1usize, 2, 3, 10, 17] {
            let rendered = format_human_output_with_limit(&report, requested).unwrap();
            assert_eq!(
                row_count(&rendered),
                requested,
                "--top-files {requested} must print {requested} rows"
            );
        }
    }

    /// `--top-files 0` is documented as "0 = all".
    #[test]
    fn top_files_zero_prints_every_file() {
        let report = report_with_files(25);
        let rendered = format_human_output_with_limit(&report, 0).unwrap();
        assert_eq!(row_count(&rendered), 25);
    }

    /// `--color never` produced the same 68 escape-bearing lines as `--color
    /// auto` here: the block renderers interpolated the raw `pub const`
    /// sequences, which cannot consult `colors_enabled()`. `c::seq` can.
    #[test]
    fn human_output_is_plain_text_when_colour_is_disabled() {
        assert!(
            !crate::cli::colors::colors_enabled(),
            "cargo test captures stdout, so colour must resolve to off here"
        );

        let mut report = report_with_files(3);
        report.duplicate_blocks = vec![DuplicateBlock {
            hash: "deadbeef".to_string(),
            lines: 12,
            tokens: 40,
            similarity: 1.0,
            clone_type: CloneType::Exact,
            locations: vec![
                DuplicateLocation {
                    file: "src/a.rs".to_string(),
                    start_line: 1,
                    end_line: 12,
                    content_preview: "fn dup() {}".to_string(),
                },
                DuplicateLocation {
                    file: "src/b.rs".to_string(),
                    start_line: 40,
                    end_line: 51,
                    content_preview: "fn dup() {}".to_string(),
                },
            ],
        }];

        let rendered = format_human_output(&report).unwrap();
        assert!(
            !rendered.contains('\u{1b}'),
            "no ANSI escape may reach a redirected stdout: {:?}",
            rendered
                .lines()
                .filter(|l| l.contains('\u{1b}'))
                .collect::<Vec<_>>()
        );
        assert!(rendered.contains("Duplicate Code Analysis"));
        assert!(rendered.contains("Block 1"));
        assert!(rendered.contains("src/a.rs:1-12"));
        assert!(rendered.contains("Preview:"));
    }

    fn report_with_blocks(n: usize) -> DuplicateReport {
        let mut report = report_with_files(3);
        report.duplicate_blocks = (0..n)
            .map(|i| DuplicateBlock {
                hash: format!("h{i:02}"),
                lines: 6,
                tokens: 20,
                similarity: 1.0,
                clone_type: CloneType::Exact,
                locations: vec![
                    DuplicateLocation {
                        file: format!("src/a{i:02}.rs"),
                        start_line: 1,
                        end_line: 6,
                        content_preview: "fn dup() {}".to_string(),
                    },
                    DuplicateLocation {
                        file: format!("src/b{i:02}.rs"),
                        start_line: 1,
                        end_line: 6,
                        content_preview: "fn dup() {}".to_string(),
                    },
                ],
            })
            .collect();
        report
    }

    fn block_count(rendered: &str) -> usize {
        rendered
            .lines()
            .filter(|l| l.contains(" locations)"))
            .count()
    }

    /// `summary`, `detailed` and `human` all routed to `format_human_output`,
    /// so `md5sum` of the three renderings matched: "Summary statistics only"
    /// printed the entire per-block detail listing.
    #[test]
    fn summary_omits_the_per_block_listing() {
        let report = report_with_blocks(3);
        let summary = format_text_output(&report, DEFAULT_TOP_FILES, TextDetail::Summary).unwrap();

        assert!(!summary.contains("Duplicate Blocks"), "{summary}");
        assert_eq!(block_count(&summary), 0);
        // The statistics themselves must survive.
        assert!(summary.contains("Total duplicate blocks:"));
        assert!(summary.contains("Duplication percentage:"));
        assert!(summary.contains("Top Files by Duplication"));
    }

    /// The three text formats must not be byte-identical: `detailed` lists
    /// every block, `human` stops at twenty, `summary` lists none.
    #[test]
    fn the_three_text_formats_differ() {
        let report = report_with_blocks(25);
        let summary = format_text_output(&report, DEFAULT_TOP_FILES, TextDetail::Summary).unwrap();
        let human = format_text_output(&report, DEFAULT_TOP_FILES, TextDetail::Human).unwrap();
        let detailed =
            format_text_output(&report, DEFAULT_TOP_FILES, TextDetail::Detailed).unwrap();

        assert_eq!(block_count(&summary), 0);
        assert_eq!(block_count(&human), HUMAN_BLOCK_LIMIT);
        assert_eq!(block_count(&detailed), 25);

        assert!(human.contains("... and 5 more blocks"));
        assert!(
            !detailed.contains("more blocks"),
            "detailed lists every block, so nothing remains to summarise"
        );

        assert_ne!(summary, human);
        assert_ne!(human, detailed);
    }

    /// The dispatcher is what `--format` reaches; assert there, too, so a
    /// re-collapse of the match arms fails.
    #[test]
    fn the_format_dispatcher_keeps_the_three_text_formats_apart() {
        let report = report_with_blocks(25);
        let of = |f| format_output(&report, f, DEFAULT_TOP_FILES).unwrap();
        let summary = of(crate::cli::DuplicateOutputFormat::Summary);
        let human = of(crate::cli::DuplicateOutputFormat::Human);
        let detailed = of(crate::cli::DuplicateOutputFormat::Detailed);

        assert_ne!(summary, human);
        assert_ne!(human, detailed);
        assert_ne!(summary, detailed);
    }

    /// SARIF `tool.driver.version` / `semanticVersion` were the literals
    /// "1.0.0" and "2.97.0" — a run of any later pmat reported provenance for
    /// a release it was not.
    #[test]
    fn sarif_reports_the_running_pmat_version() {
        let report = report_with_blocks(1);
        let sarif = format_sarif_output(&report, DEFAULT_TOP_FILES).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&sarif).unwrap();
        let driver = &parsed["runs"][0]["tool"]["driver"];

        assert_eq!(driver["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(driver["semanticVersion"], env!("CARGO_PKG_VERSION"));
        assert_ne!(driver["version"], "1.0.0");
        assert_ne!(driver["semanticVersion"], "2.97.0");
    }

    /// Changing the row limit must not touch a single measured figure.
    #[test]
    fn the_summary_is_identical_at_every_row_limit() {
        let report = report_with_files(25);
        let summary_of = |limit: usize| {
            format_human_output_with_limit(&report, limit)
                .unwrap()
                .lines()
                .filter(|l| l.contains("Duplication percentage") || l.contains("Duplicate lines"))
                .collect::<Vec<_>>()
                .join("\n")
        };
        let baseline = summary_of(1);
        for limit in [2usize, 3, 10, 0] {
            assert_eq!(
                baseline,
                summary_of(limit),
                "limit {limit} changed the summary"
            );
        }
    }
}

/// Format output as SARIF
fn format_sarif_output(report: &DuplicateReport, top_files: usize) -> Result<String> {
    let listing = DuplicateListing::of(report, top_files);
    // `tool.driver.version` / `semanticVersion` were the literals "1.0.0" and
    // "2.97.0" — two different, both wrong, answers to "which pmat produced
    // this?". SARIF consumers key result provenance on these, so a run from
    // any pmat since 2.97.0 claimed to be 2.97.0. The running version is the
    // only honest value.
    let sarif = serde_json::json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat-duplicates",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                    "semanticVersion": env!("CARGO_PKG_VERSION")
                }
            },
            // Bounded by `--top-files`, like the JSON listing beside it: a
            // SARIF file is read by a machine too, and an unbounded one is the
            // same defect in a second format.
            "results": listing.blocks.iter().map(|block| {
                serde_json::json!({
                    "ruleId": "duplicate-code",
                    "level": "warning",
                    "message": {
                        "text": format!("Duplicate code block found ({} lines)", block.lines)
                    },
                    "locations": block.locations.iter().map(|loc| {
                        serde_json::json!({
                            "physicalLocation": {
                                "artifactLocation": {
                                    "uri": loc.file
                                },
                                "region": {
                                    "startLine": loc.start_line,
                                    "endLine": loc.end_line
                                }
                            }
                        })
                    }).collect::<Vec<_>>()
                })
            }).collect::<Vec<_>>()
        }]
    });

    Ok(serde_json::to_string_pretty(&sarif)?)
}

/// Format output as CSV.
///
/// One row per SITE past the first, all sharing the block's first location as
/// `File1`. This used to take `locations[0]` and `locations[1]` and drop the
/// rest, so a clone family living in 10,486 places was exported as a single
/// pair: on this repo's own `src` the CSV carried 98,244 of 175,106 located
/// sites and said nothing about the 76,862 (43.9%) it discarded, with exit code
/// 0. Every location reaches the spreadsheet now, and rows of one family share
/// `File1,Start1,End1`, which is the group key a pivot needs.
///
/// The star (first-vs-each) decomposition, not every pair: the same family of
/// 10,486 sites is 10,485 rows this way and 55 MILLION as a full cross product,
/// and the extra rows would carry no site the star does not already name.
///
/// `Type` is the block's measured clone class. It was the literal string
/// `"exact"` on every row of every run, which carried "these two are byte
/// identical" into a spreadsheet about clones that are not.
fn format_csv_output(report: &DuplicateReport, top_files: usize) -> Result<String> {
    let listing = DuplicateListing::of(report, top_files);
    let mut csv = String::new();
    csv.push_str("Type,File1,Start1,End1,File2,Start2,End2\n");

    for block in &listing.blocks {
        let Some((first, rest)) = block.locations.split_first() else {
            continue;
        };
        for other in rest {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{}\n",
                block.clone_type.label(),
                first.file,
                first.start_line,
                first.end_line,
                other.file,
                other.start_line,
                other.end_line
            ));
        }
    }

    Ok(csv)
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod clone_class_and_csv_completeness_tests {
    //! GH-935 (every clone labelled "exact") and GH-936 (CSV drops every
    //! location past the second).
    //!
    //! Both are read through the emitters rather than through the struct, so
    //! these tests describe what a consumer of `--format json` / `-f csv`
    //! actually receives.
    use super::*;
    use crate::cli::DuplicateType;
    use std::path::Path;

    /// FOUR copies of one body with every identifier renamed — a Type-2 family
    /// with more than two sites, which is what makes it a fixture for both
    /// defects at once.
    const RENAMED_FOURFOLD: &str = "\
fn alpha(input: usize) -> usize {
    let total = input + 1;
    let doubled = total * 2;
    doubled
}
fn beta(value: usize) -> usize {
    let sum = value + 1;
    let twice = sum * 2;
    twice
}
fn gamma(arg: usize) -> usize {
    let acc = arg + 1;
    let scaled = acc * 2;
    scaled
}
fn delta(operand: usize) -> usize {
    let carry = operand + 1;
    let bumped = carry * 2;
    bumped
}
";

    /// TWO byte-identical copies of one body. A genuine Type-1 family, so a fix
    /// that simply relabels everything "renamed" fails here.
    const IDENTICAL_TWICE: &str = "\
fn first_caller() -> usize {
    let accumulator = compute_value();
    let adjusted = accumulator + OFFSET;
    let rounded = adjusted / DIVISOR;
    rounded
}
fn second_caller() -> usize {
    let accumulator = compute_value();
    let adjusted = accumulator + OFFSET;
    let rounded = adjusted / DIVISOR;
    rounded
}
";

    fn report_for(source: &str, kind: DuplicateType) -> DuplicateReport {
        let lines: Vec<&str> = source.lines().collect();
        let blocks = extract_blocks(&lines, Path::new("dup.rs"), 4, 1000, kind);
        let duplicate_blocks = find_duplicate_blocks(blocks);
        DuplicateReport {
            total_duplicates: duplicate_blocks.len(),
            duplicate_lines: 0,
            total_lines: lines.len(),
            duplication_percentage: 0.0,
            duplicate_blocks,
            file_statistics: BTreeMap::new(),
        }
    }

    fn json_of(report: &DuplicateReport) -> serde_json::Value {
        serde_json::from_str(&format_json_output(report, 0).unwrap()).unwrap()
    }

    fn data_rows(csv: &str) -> Vec<&str> {
        csv.lines().skip(1).filter(|l| !l.is_empty()).collect()
    }

    /// GH-935: `--detection-type renamed` can only find Type-2 clones, and every
    /// one of them was reported as `exact_duplicates`, with `similarity: 1.0`
    /// written as a literal. On this repo that claimed `rules.rs:40-78` and
    /// `rules.rs:98-136` are byte identical when they differ on 8 of 39 lines.
    #[test]
    fn renamed_clones_are_not_counted_as_exact() {
        let report = report_for(RENAMED_FOURFOLD, DuplicateType::Renamed);
        assert!(
            report.total_duplicates > 0,
            "fixture must produce renamed clone groups"
        );

        let json = json_of(&report);
        assert_eq!(
            json["exact_duplicates"], 0,
            "no group here is byte identical: {json:#}"
        );
        assert_eq!(
            json["renamed_duplicates"].as_u64().unwrap() as usize,
            report.total_duplicates,
            "every group here is Type-2: {json:#}"
        );
        for block in &json["duplicate_blocks"].as_array().unwrap().clone() {
            assert_eq!(block["clone_type"], "renamed", "{block:#}");
            assert!(
                block["similarity"].as_f64().unwrap() < 1.0,
                "a renamed clone is not a perfect match: {block:#}"
            );
        }
    }

    /// The other half of the same rule: a family that IS byte identical must
    /// still be reported as exact, so the fix cannot be "call everything
    /// renamed".
    #[test]
    fn identical_clones_are_still_counted_as_exact() {
        let report = report_for(IDENTICAL_TWICE, DuplicateType::Exact);
        assert!(report.total_duplicates > 0, "fixture must produce a group");

        let json = json_of(&report);
        assert_eq!(
            json["exact_duplicates"].as_u64().unwrap() as usize,
            report.total_duplicates,
            "{json:#}"
        );
        assert_eq!(json["renamed_duplicates"], 0, "{json:#}");
        for block in json["duplicate_blocks"].as_array().unwrap() {
            assert_eq!(block["clone_type"], "exact", "{block:#}");
            assert_eq!(block["similarity"].as_f64().unwrap(), 1.0, "{block:#}");
        }
    }

    /// GH-935, CSV surface: the `Type` column was the literal `"exact"` on every
    /// row of every run, carrying the same false claim into a spreadsheet.
    #[test]
    fn the_csv_type_column_is_the_measured_clone_class() {
        let renamed = report_for(RENAMED_FOURFOLD, DuplicateType::Renamed);
        let csv = format_csv_output(&renamed, 0).unwrap();
        let rows = data_rows(&csv);
        assert!(!rows.is_empty(), "fixture must produce CSV rows");
        for row in &rows {
            assert!(
                row.starts_with("renamed,"),
                "a Type-2 clone must not be exported as `exact`: {row}"
            );
        }

        let exact = report_for(IDENTICAL_TWICE, DuplicateType::Exact);
        let csv = format_csv_output(&exact, 0).unwrap();
        for row in data_rows(&csv) {
            assert!(row.starts_with("exact,"), "{row}");
        }
    }

    /// GH-936: the CSV took `locations[0]` and `locations[1]` and discarded
    /// every site past the second — 76,862 of 175,106 (43.9%) on this repo's
    /// own `src`, silently and with exit code 0.
    #[test]
    fn the_csv_exports_every_located_site() {
        let report = report_for(RENAMED_FOURFOLD, DuplicateType::Renamed);
        assert!(
            report
                .duplicate_blocks
                .iter()
                .any(|b| b.locations.len() > 2),
            "fixture must contain a family of more than two sites, else this \
             test cannot see the defect"
        );

        let csv = format_csv_output(&report, 0).unwrap();
        let rows = data_rows(&csv);

        let expected: usize = report
            .duplicate_blocks
            .iter()
            .map(|b| b.locations.len().saturating_sub(1))
            .sum();
        assert_eq!(
            rows.len(),
            expected,
            "one row per site past the first; got {} rows for {} sites",
            rows.len(),
            report
                .duplicate_blocks
                .iter()
                .map(|b| b.locations.len())
                .sum::<usize>()
        );

        // Naming the sites is the point: every start line of every location has
        // to be findable in the export.
        for block in &report.duplicate_blocks {
            for loc in &block.locations {
                let needle = format!(",{},{}", loc.start_line, loc.end_line);
                assert!(
                    rows.iter().any(|r| r.contains(&needle)),
                    "{}:{}-{} never reached the CSV",
                    loc.file,
                    loc.start_line,
                    loc.end_line
                );
            }
        }
    }

    /// The header is a published schema; widening the rows must not move a
    /// column.
    #[test]
    fn the_csv_schema_is_unchanged() {
        let report = report_for(RENAMED_FOURFOLD, DuplicateType::Renamed);
        let csv = format_csv_output(&report, 0).unwrap();
        assert!(csv.starts_with("Type,File1,Start1,End1,File2,Start2,End2\n"));
        for row in data_rows(&csv) {
            assert_eq!(row.split(',').count(), 7, "{row}");
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod top_files_bounds_the_machine_readable_listings_tests {
    //! Issue #1050 P10. `--top-files` reached the text renderers and nothing
    //! else, so `analyze duplicates --format json` and the same command with
    //! `--top-files 5` produced byte-identical documents — 853,863 bytes on
    //! copia, 387 MB on depyler — with no limit, no marker and no way for a
    //! consumer to tell a complete listing from a clipped one.
    use super::*;

    /// N files, each with one clone family of its own, so a top-N cut over
    /// files has something to cut. File `i` duplicates `i + 1` lines out of a
    /// fixed 100, which makes the duplication ranking strict and the ordering
    /// of the listing a fact rather than a tie-break.
    fn report_over(files: usize) -> DuplicateReport {
        let mut file_statistics = BTreeMap::new();
        let mut duplicate_blocks = Vec::new();
        for i in 0..files {
            let path = format!("src/f{i:03}.rs");
            // `i` is a small loop index, so the widening is exact.
            let pct = f32::from(u8::try_from(i + 1).expect("fixture stays small"));
            file_statistics.insert(
                path.clone(),
                FileStats {
                    duplicate_lines: i + 1,
                    total_lines: 100,
                    duplication_percentage: pct,
                },
            );
            duplicate_blocks.push(DuplicateBlock {
                hash: format!("h{i:03}"),
                lines: i + 1,
                tokens: 20,
                similarity: 1.0,
                clone_type: CloneType::Exact,
                locations: vec![
                    DuplicateLocation {
                        file: path.clone(),
                        start_line: 1,
                        end_line: 6,
                        content_preview: "fn dup() {}".to_string(),
                    },
                    DuplicateLocation {
                        file: path,
                        start_line: 50,
                        end_line: 55,
                        content_preview: "fn dup() {}".to_string(),
                    },
                ],
            });
        }
        DuplicateReport {
            total_duplicates: duplicate_blocks.len(),
            duplicate_lines: (1..=files).sum(),
            total_lines: files * 100,
            duplication_percentage: 1.0,
            duplicate_blocks,
            file_statistics,
        }
    }

    fn json(report: &DuplicateReport, top_files: usize) -> serde_json::Value {
        serde_json::from_str(&format_json_output(report, top_files).expect("json renders"))
            .expect("the JSON must parse")
    }

    /// The reproducer, in the surface a consumer reads: two documents that used
    /// to be equal must now differ, and the smaller one must be smaller.
    #[test]
    fn top_files_changes_the_json_document() {
        let report = report_over(40);

        let all = format_json_output(&report, 0).expect("json renders");
        let five = format_json_output(&report, 5).expect("json renders");

        assert_ne!(all, five, "--top-files changed not one byte of the JSON");
        assert!(
            five.len() < all.len(),
            "a listing limit must SHRINK the document: {} vs {}",
            five.len(),
            all.len()
        );
    }

    /// What the limit keeps: the top N files by duplication, and the blocks
    /// that fall in them — not an arbitrary prefix.
    #[test]
    fn the_listing_is_the_top_files_by_duplication() {
        let value = json(&report_over(40), 3);

        let files: Vec<&str> = value["file_statistics"]
            .as_object()
            .expect("file_statistics is an object")
            .keys()
            .map(String::as_str)
            .collect();
        // f039 duplicates 40 lines, f038 39, f037 38 — the three worst.
        assert_eq!(files, ["src/f037.rs", "src/f038.rs", "src/f039.rs"]);
        assert_eq!(
            value["duplicate_blocks"]
                .as_array()
                .expect("duplicate_blocks is an array")
                .len(),
            3
        );
    }

    /// The disclosure, which is the half that makes truncation honest: a
    /// consumer must be able to READ that rows were withheld, and how many.
    #[test]
    fn the_document_declares_what_it_withheld() {
        let value = json(&report_over(40), 3);
        let listing = &value["listing"];

        assert_eq!(listing["top_files"], 3);
        assert_eq!(listing["files_total"], 40);
        assert_eq!(listing["files_listed"], 3);
        assert_eq!(listing["files_truncated"], true);
        assert_eq!(listing["blocks_total"], 40);
        assert_eq!(listing["blocks_listed"], 3);
        assert_eq!(listing["blocks_truncated"], true);
    }

    /// The counter-test bounding the correction in two directions at once.
    ///
    /// `0 = all` must still mean all — a fix that bounded the document
    /// unconditionally would replace an unbounded listing with an
    /// undisclosed-but-clipped one, which is worse. And "nothing was withheld"
    /// must be stated rather than inferred from an absent field.
    #[test]
    fn zero_still_lists_everything_and_says_so() {
        let value = json(&report_over(40), 0);

        assert_eq!(
            value["file_statistics"]
                .as_object()
                .expect("file_statistics is an object")
                .len(),
            40
        );
        assert_eq!(
            value["duplicate_blocks"]
                .as_array()
                .expect("duplicate_blocks is an array")
                .len(),
            40
        );
        assert_eq!(value["listing"]["files_truncated"], false);
        assert_eq!(value["listing"]["blocks_truncated"], false);
        assert_eq!(value["listing"]["files_listed"], 40);
    }

    /// The invariant the text path already carries (see
    /// `top_files_does_not_change_the_measurement_tests`), now asserted on the
    /// JSON path too: a DISPLAY limit may not move a MEASUREMENT. The previous
    /// implementation of `--top-files` recomputed the report from the survivors
    /// and made the same tree read 12.2% or 19.4% depending on how many rows
    /// were asked for; nothing here may bring that back.
    #[test]
    fn the_measurement_is_identical_at_every_limit() {
        let report = report_over(40);
        let measured = |v: &serde_json::Value| {
            (
                v["total_duplicates"].clone(),
                v["duplicate_lines"].clone(),
                v["total_lines"].clone(),
                v["duplication_percentage"].clone(),
                v["exact_duplicates"].clone(),
                v["renamed_duplicates"].clone(),
                v["structural_similarities"].clone(),
                v["metrics"].clone(),
            )
        };

        let whole = measured(&json(&report, 0));
        for limit in [1, 3, 5, 10, 39, 41, 1000] {
            assert_eq!(
                measured(&json(&report, limit)),
                whole,
                "--top-files {limit} moved a measured number"
            );
        }
    }

    /// A block whose sites are in NO counted file must not be withheld.
    ///
    /// Caught by the existing CSV/SARIF tests, whose fixtures carry blocks and
    /// an EMPTY `file_statistics`: ranking files and then keeping only their
    /// blocks silently emitted nothing at all — 0 CSV rows for a 4-site clone
    /// family. `ensure_source_files_were_analyzed` makes that unreachable from
    /// the CLI today, which is exactly why it needs a test: the emitters are
    /// reachable from anywhere, and a listing that drops what it cannot rank is
    /// the same defect this module was written to close.
    #[test]
    fn a_block_no_counted_file_owns_is_still_listed() {
        let mut report = report_over(4);
        // A fifth family, in files the statistics know nothing about, while the
        // limit is tight enough to have excluded them had they been ranked.
        report.duplicate_blocks.push(DuplicateBlock {
            hash: "orphan".to_string(),
            lines: 9,
            tokens: 30,
            similarity: 1.0,
            clone_type: CloneType::Exact,
            locations: vec![DuplicateLocation {
                file: "src/uncounted.rs".to_string(),
                start_line: 1,
                end_line: 9,
                content_preview: "fn dup() {}".to_string(),
            }],
        });

        let value = json(&report, 1);
        let listed: Vec<&str> = value["duplicate_blocks"]
            .as_array()
            .expect("array")
            .iter()
            .map(|b| b["hash"].as_str().expect("hash"))
            .collect();
        assert!(listed.contains(&"orphan"), "{listed:?}");
        // …and the counted files are still cut to the limit.
        assert_eq!(value["listing"]["files_listed"], 1);
    }

    /// CSV and SARIF are machine surfaces too, and were unbounded for the same
    /// reason.
    #[test]
    fn csv_and_sarif_are_bounded_by_the_same_flag() {
        let report = report_over(40);

        let rows = |limit| {
            format_csv_output(&report, limit)
                .expect("csv renders")
                .lines()
                .skip(1)
                .filter(|l| !l.is_empty())
                .count()
        };
        assert_eq!(rows(0), 40);
        assert_eq!(rows(3), 3);

        let results = |limit| {
            let v: serde_json::Value =
                serde_json::from_str(&format_sarif_output(&report, limit).expect("sarif renders"))
                    .expect("the SARIF must parse");
            v["runs"][0]["results"]
                .as_array()
                .expect("results is an array")
                .len()
        };
        assert_eq!(results(0), 40);
        assert_eq!(results(3), 3);
    }
}
