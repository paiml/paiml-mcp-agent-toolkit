/// Format output based on format type
fn format_output(
    report: &DuplicateReport,
    format: crate::cli::DuplicateOutputFormat,
) -> Result<String> {
    match format {
        crate::cli::DuplicateOutputFormat::Json => format_json_output(report),
        crate::cli::DuplicateOutputFormat::Human => format_human_output(report),
        crate::cli::DuplicateOutputFormat::Summary => {
            format_text_output(report, DEFAULT_TOP_FILES, TextDetail::Summary)
        }
        crate::cli::DuplicateOutputFormat::Detailed => {
            format_text_output(report, DEFAULT_TOP_FILES, TextDetail::Detailed)
        }
        crate::cli::DuplicateOutputFormat::Sarif => format_sarif_output(report),
        crate::cli::DuplicateOutputFormat::Csv => format_csv_output(report),
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

/// Format output as JSON
fn format_json_output(report: &DuplicateReport) -> Result<String> {
    // Create enhanced JSON with test-expected fields
    let enhanced_json = serde_json::json!({
        "total_duplicates": report.total_duplicates,
        "duplicate_lines": report.duplicate_lines,
        "total_lines": report.total_lines,
        "duplication_percentage": report.duplication_percentage,
        "duplicate_blocks": report.duplicate_blocks,
        "file_statistics": report.file_statistics,
        "exact_duplicates": report.duplicate_blocks.iter().filter(|b| b.similarity >= 1.0).count(),
        "structural_similarities": report.duplicate_blocks.iter().filter(|b| b.similarity >= 0.8 && b.similarity < 1.0).count(),
        // `entropy_analysis` and `analysis_time_ms` used to be emitted here as
        // the constants 0.5 / 0 / 100 in EVERY run, regardless of input. This
        // command runs no entropy analysis and did not time itself, so those
        // were fabricated measurements sitting beside real ones -- which is
        // precisely what makes them dangerous: the real neighbours lend them
        // credibility. They are omitted rather than defaulted; a consumer that
        // needs entropy should call `pmat analyze entropy`, which measures it.
        //
        // See contracts/pmat-no-fabrication-v1.yaml, equation `measured_or_absent`.
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

/// Write block header with summary info
fn write_block_header(output: &mut String, block_num: usize, block: &DuplicateBlock) -> Result<()> {
    use crate::cli::colors as c;
    use std::fmt::Write;
    writeln!(
        output,
        "  {}Block {}{} ({} lines, {} locations)",
        c::seq(c::BOLD),
        block_num,
        c::seq(c::RESET),
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
        let of = |f| format_output(&report, f).unwrap();
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
        let sarif = format_sarif_output(&report).unwrap();
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
fn format_sarif_output(report: &DuplicateReport) -> Result<String> {
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
            "results": report.duplicate_blocks.iter().map(|block| {
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

/// Format output as CSV
fn format_csv_output(report: &DuplicateReport) -> Result<String> {
    let mut csv = String::new();
    csv.push_str("Type,File1,Start1,End1,File2,Start2,End2\n");

    for block in &report.duplicate_blocks {
        if block.locations.len() >= 2 {
            let loc1 = &block.locations[0];
            let loc2 = &block.locations[1];
            csv.push_str(&format!(
                "exact,{},{},{},{},{},{}\n",
                loc1.file,
                loc1.start_line,
                loc1.end_line,
                loc2.file,
                loc2.start_line,
                loc2.end_line
            ));
        }
    }

    Ok(csv)
}
