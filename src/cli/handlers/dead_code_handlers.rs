//! Dead Code Analysis Handler
//!
//! Extracted from complexity_handlers.rs for file health compliance (CB-040).
//! Contains dead code analysis handler and all related helper functions.
//!
//! Submodule layout (include! pattern):
//! - dead_code_handlers_analysis.rs: Core analysis logic and cargo integration
//! - dead_code_handlers_output.rs: Output formatting (JSON, SARIF, summary, markdown)

#![cfg_attr(coverage_nightly, coverage(off))]
use crate::cli::DeadCodeOutputFormat;
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Configuration for dead code analysis
#[allow(clippy::too_many_arguments)]
struct DeadCodeAnalysisFilters {
    include_unreachable: bool,
    include_tests: bool,
    min_dead_lines: usize,
    top_files: Option<usize>,
    include: Vec<String>,
    exclude: Vec<String>,
    max_depth: usize,
}

/// Handle dead code analysis command - REFACTORED
/// Cognitive complexity reduced from 244 to ~10
#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_analyze_dead_code(
    path: PathBuf,
    format: DeadCodeOutputFormat,
    top_files: Option<usize>,
    include_unreachable: bool,
    min_dead_lines: usize,
    include_tests: bool,
    output: Option<PathBuf>,
    fail_on_violation: bool,
    max_percentage: f64,
    timeout: u64,
    include: Vec<String>,
    exclude: Vec<String>,
    max_depth: usize,
) -> Result<()> {
    crate::status_eprintln!("☠️ Analyzing dead code in project...");
    crate::status_eprintln!("⏰ Analysis timeout set to {timeout} seconds");

    // Apply include/exclude filters if specified
    if !include.is_empty() || !exclude.is_empty() {
        crate::status_eprintln!("🔍 Applying file filters...");
        if !include.is_empty() {
            crate::status_eprintln!("  Include patterns: {include:?}");
        }
        if !exclude.is_empty() {
            crate::status_eprintln!("  Exclude patterns: {exclude:?}");
        }
    }

    // Run analysis with timeout
    let timeout_duration = tokio::time::Duration::from_secs(timeout);
    let outcome = tokio::time::timeout(timeout_duration, async {
        run_dead_code_analysis_with_filters(
            &path,
            DeadCodeAnalysisFilters {
                include_unreachable,
                include_tests,
                min_dead_lines,
                top_files,
                include,
                exclude,
                max_depth,
            },
        )
        .await
    })
    .await
    .map_err(|_| anyhow::anyhow!("Dead code analysis timed out after {timeout} seconds"))??;

    let result = outcome.report;
    let scope = outcome.scope;

    crate::status_eprintln!(
        "📊 Analysis complete: {} files analyzed, {} with dead code",
        result.summary.total_files_analyzed,
        result.summary.files_with_dead_code
    );

    // Format output
    let formatted_output = format_dead_code_result(&result, &format, scope)?;

    // Write output
    write_dead_code_output(formatted_output, output).await?;

    // Check for violations and exit with error code if requested.
    //
    // The gate used to read `result.summary.dead_percentage`, which is scoped to
    // the REPORTED list — `--top-files 5` shrinks it, `--top-files 10000` grows
    // it — so the same project passed or failed `--max-dead-code` depending on
    // how many files the user asked to see. It compares the project-wide figure
    // now, and refuses to render a verdict when there is no project-wide figure
    // to compare: a threshold that cannot be evaluated must not report a pass.
    if fail_on_violation {
        match dead_code_gate_verdict(outcome.project_dead_percentage, max_percentage) {
            DeadCodeGateVerdict::Pass => {}
            DeadCodeGateVerdict::Violation(dead_code_percentage) => {
                eprintln!(
                    "\n❌ Dead code violations found: {dead_code_percentage:.1}% of all lines \
                     walked exceeds threshold of {max_percentage:.1}%"
                );
                std::process::exit(1);
            }
            DeadCodeGateVerdict::Unmeasurable => anyhow::bail!(
                "--fail-on-violation cannot be enforced here: no project-wide dead-code \
                 percentage was measured for this project (the multi-language analyzer \
                 does not count total project lines). Re-run without --fail-on-violation \
                 to see the report."
            ),
        }
    }

    Ok(())
}

/// What `--fail-on-violation` decides.
#[derive(Debug, PartialEq)]
enum DeadCodeGateVerdict {
    Pass,
    Violation(f32),
    /// No project-wide percentage exists, so the threshold cannot be evaluated.
    Unmeasurable,
}

/// Compare the PROJECT-wide dead-code percentage against `--max-dead-code`.
///
/// Takes `Option` deliberately: an analyzer that cannot produce a project-wide
/// figure yields no verdict at all, rather than a pass.
fn dead_code_gate_verdict(
    project_dead_percentage: Option<f32>,
    max_percentage: f64,
) -> DeadCodeGateVerdict {
    let Some(pct) = project_dead_percentage else {
        return DeadCodeGateVerdict::Unmeasurable;
    };
    #[allow(clippy::cast_possible_truncation)]
    if pct > max_percentage as f32 {
        DeadCodeGateVerdict::Violation(pct)
    } else {
        DeadCodeGateVerdict::Pass
    }
}

// --- Submodule includes ---

include!("dead_code_handlers_analysis.rs");
include!("dead_code_handlers_output.rs");

// The report's own account of what it did and did not measure.
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
#[path = "dead_code_handlers_scope_tests.rs"]
mod scope_tests;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod output_tests {
    //! Covers format_dead_code_* + write_*_section helpers in
    //! dead_code_handlers_output.rs (315 uncov on broad, 0% cov).
    //! The async write_dead_code_output is skipped (requires fs/IO setup).
    use super::*;
    use crate::models::dead_code::{
        ConfidenceLevel, DeadCodeItem, DeadCodeResult, DeadCodeSummary, DeadCodeType,
        FileDeadCodeMetrics,
    };

    fn item(ty: DeadCodeType, line: u32, name: &str, reason: &str) -> DeadCodeItem {
        DeadCodeItem {
            item_type: ty,
            name: name.to_string(),
            line,
            reason: reason.to_string(),
        }
    }

    fn file(path: &str, conf: ConfidenceLevel, items: Vec<DeadCodeItem>) -> FileDeadCodeMetrics {
        FileDeadCodeMetrics {
            path: path.to_string(),
            dead_lines: 10,
            total_lines: 100,
            dead_percentage: 10.0,
            dead_functions: 1,
            dead_classes: 0,
            dead_modules: 0,
            unreachable_blocks: 0,
            dead_score: 0.0,
            confidence: conf,
            items,
        }
    }

    fn empty_summary() -> DeadCodeSummary {
        DeadCodeSummary {
            total_files_analyzed: 5,
            files_with_dead_code: 0,
            total_dead_lines: 0,
            dead_percentage: 0.0,
            dead_functions: 0,
            dead_classes: 0,
            dead_modules: 0,
            unreachable_blocks: 0,
        }
    }

    fn full_summary() -> DeadCodeSummary {
        DeadCodeSummary {
            total_files_analyzed: 5,
            files_with_dead_code: 2,
            total_dead_lines: 20,
            dead_percentage: 12.0,
            dead_functions: 3,
            dead_classes: 2,
            dead_modules: 1,
            unreachable_blocks: 4,
        }
    }

    fn populated_result() -> DeadCodeResult {
        DeadCodeResult {
            summary: full_summary(),
            files: vec![
                file(
                    "src/a.rs",
                    ConfidenceLevel::High,
                    vec![
                        item(DeadCodeType::Function, 10, "f", "no callers"),
                        item(DeadCodeType::Class, 20, "C", "unused"),
                    ],
                ),
                file(
                    "src/b.rs",
                    ConfidenceLevel::Medium,
                    vec![item(DeadCodeType::Variable, 5, "x", "never read")],
                ),
                file(
                    "src/c.rs",
                    ConfidenceLevel::Low,
                    vec![item(
                        DeadCodeType::UnreachableCode,
                        99,
                        "block",
                        "after panic",
                    )],
                ),
            ],
            total_files: 5,
            analyzed_files: 5,
            files_with_dead_code_found: 2,
            files_truncated: false,
        }
    }

    fn empty_result() -> DeadCodeResult {
        DeadCodeResult {
            summary: empty_summary(),
            files: vec![],
            total_files: 5,
            analyzed_files: 5,
            files_with_dead_code_found: 2,
            files_truncated: false,
        }
    }

    // ── format_dead_code_result dispatcher ──

    #[test]
    fn test_format_dispatcher_json_arm() {
        let r = format_dead_code_result(
            &empty_result(),
            &DeadCodeOutputFormat::Json,
            DeadCodeReportScope::default(),
        )
        .unwrap();
        // serde_json output is non-empty even for empty data.
        assert!(r.contains("summary") || r.contains("files"));
    }

    #[test]
    fn test_format_dispatcher_sarif_arm() {
        let r = format_dead_code_result(
            &populated_result(),
            &DeadCodeOutputFormat::Sarif,
            DeadCodeReportScope::default(),
        )
        .unwrap();
        assert!(r.contains("\"version\": \"2.1.0\""));
        assert!(r.contains("dead-code"));
    }

    #[test]
    fn test_format_dispatcher_summary_arm() {
        let r = format_dead_code_result(
            &populated_result(),
            &DeadCodeOutputFormat::Summary,
            DeadCodeReportScope::default(),
        )
        .unwrap();
        assert!(!r.is_empty());
    }

    #[test]
    fn test_format_dispatcher_markdown_arm() {
        let r = format_dead_code_result(
            &populated_result(),
            &DeadCodeOutputFormat::Markdown,
            DeadCodeReportScope::default(),
        )
        .unwrap();
        assert!(r.contains("# Dead Code Analysis Report"));
    }

    // ── format_dead_code_as_sarif: confidence + item type arms ──

    #[test]
    fn test_sarif_levels_for_each_confidence() {
        // High → "error", Medium → "warning", Low → "note".
        let r = format_dead_code_as_sarif(&populated_result()).unwrap();
        assert!(r.contains("\"error\""));
        assert!(r.contains("\"warning\""));
        assert!(r.contains("\"note\""));
    }

    #[test]
    fn test_sarif_message_for_each_dead_code_type() {
        // Function/Class/Variable/UnreachableCode label arms.
        let r = format_dead_code_as_sarif(&populated_result()).unwrap();
        assert!(r.contains("Dead function"));
        assert!(r.contains("Dead class"));
        assert!(r.contains("Dead variable"));
        assert!(r.contains("Unreachable code"));
    }

    #[test]
    fn test_sarif_empty_files_yields_empty_results_array() {
        let r = format_dead_code_as_sarif(&empty_result()).unwrap();
        assert!(r.contains("\"results\": []"));
    }

    // ── format_dead_code_as_summary: branch arms ──

    #[test]
    fn test_summary_with_dead_functions_emits_breakdown_section() {
        let r = format_dead_code_as_summary(&populated_result()).unwrap();
        assert!(r.contains("Dead Code by Type"));
        assert!(r.contains("Top Files"));
    }

    #[test]
    /// UPDATED in round 3: this asserted that the breakdown is skipped whenever
    /// `dead_functions == 0`, which hid it exactly when it was needed — on the
    /// real repo every dead item was a field, so 26 dead lines were reported
    /// with no types at all. Dead code that is not a function is still dead
    /// code, and it now has a row of its own.
    fn test_summary_without_dead_functions_still_breaks_down_by_type() {
        let mut res = populated_result();
        res.summary.dead_functions = 0;
        let r = format_dead_code_as_summary(&res).unwrap();
        assert!(r.contains("Dead Code by Type"));
        assert!(r.contains("Other (fields, constants, statics):"));
        // Top Files still emitted (files non-empty).
        assert!(r.contains("Top Files"));
    }

    #[test]
    fn test_summary_empty_files_skips_top_files_section() {
        let r = format_dead_code_as_summary(&empty_result()).unwrap();
        assert!(!r.contains("Top Files"));
    }

    // ── format_dead_code_as_markdown: section gating ──

    #[test]
    fn test_markdown_with_full_data_emits_all_sections() {
        let r = format_dead_code_as_markdown(&populated_result()).unwrap();
        assert!(r.contains("# Dead Code Analysis Report"));
        assert!(r.contains("## Summary"));
        assert!(r.contains("## Dead Code Breakdown"));
        assert!(r.contains("## File Details"));
        assert!(r.contains("## Recommendations"));
    }

    /// #721: the breakdown table printed `summary.dead_modules` under a
    /// "Variables" heading. `dead_modules` is a MODULE count on the cargo path,
    /// so a cargo run reported its dead modules under a row no producer fills,
    /// while real Variable items were counted in no row at all.
    #[test]
    fn test_markdown_breakdown_labels_dead_modules_as_modules_not_variables() {
        let r = format_dead_code_as_markdown(&populated_result()).unwrap();

        assert!(
            r.contains("| Modules | 1 |"),
            "dead_modules must be labelled Modules; got:\n{r}"
        );
        assert!(
            !r.contains("| Variables |"),
            "the module count must not be labelled Variables; got:\n{r}"
        );
    }

    /// #721 companion: Variable items are counted from the items themselves, in
    /// their own row, exactly as the text renderer already did.
    #[test]
    fn test_markdown_breakdown_counts_variable_items_in_their_own_row() {
        // populated_result() has exactly one Variable item (src/b.rs).
        let r = format_dead_code_as_markdown(&populated_result()).unwrap();

        assert!(
            r.contains("| Other (fields, constants, statics) | 1 |"),
            "Variable items must be counted in their own row; got:\n{r}"
        );
    }

    #[test]
    fn test_markdown_empty_skips_breakdown_and_files() {
        let r = format_dead_code_as_markdown(&empty_result()).unwrap();
        // Always includes summary + recommendations.
        assert!(r.contains("## Summary"));
        assert!(r.contains("## Recommendations"));
        // Skipped when dead_functions == 0 and files empty.
        assert!(!r.contains("## Dead Code Breakdown"));
        assert!(!r.contains("## File Details"));
    }

    #[test]
    fn test_markdown_file_details_section_takes_first_20_files() {
        let mut res = populated_result();
        // Bloat to 30 files; details section caps at 20.
        for i in 0..30 {
            res.files.push(file(
                &format!("src/extra-{i}.rs"),
                ConfidenceLevel::High,
                vec![],
            ));
        }
        let r = format_dead_code_as_markdown(&res).unwrap();
        // First file always included.
        assert!(r.contains("src/a.rs"));
        // 20-cap means "src/extra-29.rs" must NOT appear.
        assert!(!r.contains("src/extra-29.rs"));
    }

    #[test]
    fn test_summary_top_files_section_takes_first_10_files() {
        let mut res = populated_result();
        for i in 0..15 {
            res.files.push(file(
                &format!("src/extra-{i}.rs"),
                ConfidenceLevel::High,
                vec![],
            ));
        }
        let r = format_dead_code_as_summary(&res).unwrap();
        // 10-file cap → extra-14 must NOT appear.
        assert!(!r.contains("src/extra-14.rs"));
    }

    #[test]
    fn test_recommendations_section_is_static_text() {
        // Pure static-text helper; no inputs.
        let r = format_dead_code_recommendations_section();
        assert!(r.contains("## Recommendations"));
        assert!(r.contains("High Confidence Dead Code"));
        assert!(r.contains("Test Coverage"));
    }

    // ── round 3: measured line counts and self-consistent summaries ─────────

    use crate::services::cargo_dead_code_analyzer::{
        DeadCodeKind, DeadItem, FileDeadCode as CargoFileDeadCode,
    };

    fn cargo_item(name: &str, kind: DeadCodeKind, line: usize) -> DeadItem {
        DeadItem {
            name: name.to_string(),
            kind,
            line,
            column: 1,
            message: format!("`{name}` is never used"),
        }
    }

    /// Observed on the real repo: every listed file reported `total_lines: 100`
    /// — a 370-line file, a 503-line file and a 1287-line file alike — next to
    /// a `dead_percentage` computed from the REAL count, so the two disagreed
    /// by up to 13x (dead_lines 24 / total_lines 100 printed as 6.49%).
    #[test]
    fn test_per_file_total_lines_is_the_measured_count() {
        let files = vec![
            CargoFileDeadCode {
                file_path: std::path::PathBuf::from("src/big.rs"),
                dead_items: vec![
                    cargo_item("f", DeadCodeKind::Function, 10),
                    cargo_item("S", DeadCodeKind::Struct, 20),
                ],
                unreachable_items: Vec::new(),
                file_dead_percentage: 8.0 / 370.0 * 100.0,
                total_lines: Some(370),
            },
            CargoFileDeadCode {
                file_path: std::path::PathBuf::from("src/small.rs"),
                dead_items: vec![cargo_item("g", DeadCodeKind::Function, 3)],
                unreachable_items: Vec::new(),
                file_dead_percentage: 5.0 / 20.0 * 100.0,
                total_lines: Some(20),
            },
        ];

        let metrics = convert_cargo_files_to_metrics(files, 0, false);

        assert_eq!(metrics.len(), 2);
        let by_path = |name: &str| {
            metrics
                .iter()
                .find(|m| m.path.ends_with(name))
                .unwrap_or_else(|| panic!("{name} missing"))
        };
        let big = by_path("big.rs");
        let small = by_path("small.rs");
        assert_eq!(big.total_lines, 370, "the constant 100 is the bug");
        assert_eq!(small.total_lines, 20);
        assert_ne!(
            big.total_lines, small.total_lines,
            "two files of different length cannot share one line count"
        );
        // dead_lines uses the shared estimator: 5 (fn) + 3 (struct) = 8.
        assert_eq!(big.dead_lines, 8);
        assert_eq!(small.dead_lines, 5);
        // The percentage beside it must be that ratio, not a different one.
        let expected = 8.0 / 370.0 * 100.0;
        assert!(
            (big.dead_percentage - expected).abs() < 0.01,
            "dead_percentage {} does not match dead_lines/total_lines {expected}",
            big.dead_percentage
        );
        // Items are carried, so the counts above are checkable.
        assert_eq!(big.items.len(), 2);
    }

    /// `--include-unreachable` was inert on EVERY input: the CLI path could not
    /// produce an unreachable block at all (`unreachable_blocks: 0` was
    /// hardcoded and the diagnostic parser dropped rustc's `unreachable_code`
    /// warnings), so a fixture with four statements after a `return` printed
    /// "Unreachable blocks: 0" with and without the flag, byte-identical in
    /// json, sarif, markdown and summary alike.
    #[test]
    fn include_unreachable_is_the_only_way_an_unreachable_block_is_reported() {
        let files = || {
            vec![CargoFileDeadCode {
                file_path: std::path::PathBuf::from("src/lib.rs"),
                dead_items: vec![cargo_item("helper", DeadCodeKind::Function, 10)],
                unreachable_items: vec![
                    cargo_item("let y = x * 2;", DeadCodeKind::UnreachableCode, 3),
                    cargo_item("let z = y + 3;", DeadCodeKind::UnreachableCode, 4),
                ],
                file_dead_percentage: 5.0 / 40.0 * 100.0,
                total_lines: Some(40),
            }]
        };

        let off = convert_cargo_files_to_metrics(files(), 0, false);
        assert_eq!(off[0].unreachable_blocks, 0, "off means absent");
        assert_eq!(off[0].items.len(), 1, "only the unused item is listed");

        let on = convert_cargo_files_to_metrics(files(), 0, true);
        assert_eq!(on[0].unreachable_blocks, 2);
        assert_eq!(on[0].items.len(), 3, "the two unreachable rows are carried");
        assert!(on[0].items.iter().any(|i| matches!(
            i.item_type,
            crate::models::dead_code::DeadCodeType::UnreachableCode
        )));

        // The flag must not move any figure a default run prints.
        assert_eq!(off[0].dead_lines, on[0].dead_lines);
        assert_eq!(off[0].dead_functions, on[0].dead_functions);
        assert!((off[0].dead_percentage - on[0].dead_percentage).abs() < f32::EPSILON);
    }

    /// A file whose ONLY finding is unreachable code scores zero dead lines, so
    /// `--min-dead-lines` (default 10) cut it out — the second reason the flag
    /// showed nothing. It survives when it has an unreachable block to report,
    /// and is still absent when the flag is off.
    #[test]
    fn an_unreachable_only_file_survives_min_dead_lines_when_asked_for() {
        let files = || {
            vec![CargoFileDeadCode {
                file_path: std::path::PathBuf::from("src/live.rs"),
                dead_items: vec![],
                unreachable_items: vec![cargo_item("let y = 1;", DeadCodeKind::UnreachableCode, 3)],
                file_dead_percentage: 0.0,
                total_lines: Some(40),
            }]
        };

        assert!(
            convert_cargo_files_to_metrics(files(), 10, false).is_empty(),
            "nothing to report without the flag"
        );
        let on = convert_cargo_files_to_metrics(files(), 10, true);
        assert_eq!(on.len(), 1);
        assert_eq!(on[0].unreachable_blocks, 1);
    }

    /// Observed on the real repo: `summary.files_with_dead_code: 26` above a
    /// 4-entry `files` array (and `1` above an EMPTY array on a fixture), with
    /// `total_dead_lines: 94` while the rows summed to 76.
    #[test]
    fn test_summary_agrees_with_the_list_it_heads() {
        let listed = vec![
            file("src/a.rs", ConfidenceLevel::High, vec![]),
            file("src/b.rs", ConfidenceLevel::High, vec![]),
        ];
        let mut summary = DeadCodeSummary {
            total_files_analyzed: 4257,
            files_with_dead_code: 26, // the pre-fix value
            total_dead_lines: 94,     // from a different estimator
            dead_percentage: 9.9,
            dead_functions: 11,
            dead_classes: 5,
            dead_modules: 2,
            unreachable_blocks: 0,
        };

        resummarize_from_listed_files(&mut summary, &listed, 10_000);

        assert_eq!(summary.files_with_dead_code, listed.len());
        assert_eq!(
            summary.total_dead_lines,
            listed.iter().map(|f| f.dead_lines).sum::<usize>()
        );
        assert_eq!(
            summary.dead_functions,
            listed.iter().map(|f| f.dead_functions).sum::<usize>()
        );
        // percentage = listed dead lines / project lines, and never above 100.
        assert!((summary.dead_percentage - 0.2).abs() < 1e-4);
        assert!(summary.dead_percentage <= 100.0);
    }

    /// An empty list must summarise as zeros, not as the pre-filter counts.
    #[test]
    fn test_empty_list_summarises_as_empty() {
        let mut summary = DeadCodeSummary {
            total_files_analyzed: 10,
            files_with_dead_code: 1,
            total_dead_lines: 40,
            dead_percentage: 4.0,
            dead_functions: 2,
            dead_classes: 0,
            dead_modules: 0,
            unreachable_blocks: 0,
        };

        resummarize_from_listed_files(&mut summary, &[], 1000);

        assert_eq!(summary.files_with_dead_code, 0);
        assert_eq!(summary.total_dead_lines, 0);
        assert_eq!(summary.dead_percentage, 0.0);
    }
}

/// The multi-language path bills a flat 10 lines per dead function against a
/// MEASURED file length, so the estimate could describe more lines than the file
/// contains.
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod multi_language_percentage_tests {
    use super::*;
    use tempfile::TempDir;

    fn filters() -> DeadCodeAnalysisFilters {
        DeadCodeAnalysisFilters {
            top_files: None,
            include_unreachable: false,
            min_dead_lines: 0,
            include_tests: false,
            include: Vec::new(),
            exclude: Vec::new(),
            max_depth: 10,
        }
    }

    /// Observed before the fix: a 2-line `h.py` holding one dead function
    /// reported `dead_lines: 10, total_lines: 2, dead_percentage: 500.0`, and a
    /// 10-line `m.py` with two reported `20 / 10 = 200.0%`. The project summary
    /// read 250%.
    #[test]
    fn test_dead_lines_never_exceed_the_file_and_percentage_never_exceeds_100() {
        let temp = TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("m.py"),
            "def used():\n    return 1\n\ndef dead_one():\n    return 2\n\ndef dead_two():\n    return 3\n\nprint(used())\n",
        )
        .unwrap();
        // Two physical lines, one dead function: the flat estimate is 10.
        std::fs::write(
            temp.path().join("h.py"),
            "def helper_dead():\n    return 4\n",
        )
        .unwrap();
        std::fs::write(
            temp.path().join("pyproject.toml"),
            "[project]\nname=\"t\"\n",
        )
        .unwrap();

        let result = run_multi_language_dead_code(temp.path(), &filters(), "python")
            .unwrap()
            .report;

        for f in &result.files {
            assert!(
                f.dead_lines <= f.total_lines,
                "{}: {} dead lines in a {}-line file -- a part exceeding its whole",
                f.path,
                f.dead_lines,
                f.total_lines
            );
            assert!(
                f.dead_percentage <= 100.0,
                "{}: dead_percentage {} exceeds 100",
                f.path,
                f.dead_percentage
            );
        }

        assert!(
            result.summary.dead_percentage <= 100.0,
            "project dead_percentage {} exceeds 100",
            result.summary.dead_percentage
        );
    }

    /// #720: the file count reported beside the summary must be the same number
    /// the summary counted, and must be a FILE count.
    #[test]
    fn test_total_files_agrees_with_the_summary_it_heads() {
        let temp = TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("m.py"),
            "def used():\n    return 1\n\ndef dead_one():\n    return 2\n\ndef dead_two():\n    return 3\n\nprint(used())\n",
        )
        .unwrap();
        std::fs::write(
            temp.path().join("h.py"),
            "def helper_dead():\n    return 4\n",
        )
        .unwrap();
        std::fs::write(
            temp.path().join("pyproject.toml"),
            "[project]\nname=\"t\"\n",
        )
        .unwrap();

        let result = run_multi_language_dead_code(temp.path(), &filters(), "python")
            .unwrap()
            .report;

        // Two .py files on disk. This was 4 -- the function count.
        assert_eq!(result.total_files, 2, "total_files must be the file count");
        assert_eq!(result.analyzed_files, 2);
        assert_eq!(
            result.total_files, result.summary.total_files_analyzed,
            "the headline file count must agree with the summary beneath it"
        );
    }
}

/// `--fail-on-violation` used to compare `summary.dead_percentage`, which is
/// scoped to the REPORTED list, so the verdict moved with `--top-files`.
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod gate_scope_tests {
    use super::*;
    use crate::models::dead_code::{DeadCodeSummary, FileDeadCodeMetrics};

    fn dead_file(path: &str, dead_lines: usize) -> FileDeadCodeMetrics {
        let mut f = FileDeadCodeMetrics::new(path.to_string());
        f.dead_lines = dead_lines;
        f.total_lines = 1_000;
        f.dead_functions = 1;
        f
    }

    fn blank_summary() -> DeadCodeSummary {
        DeadCodeSummary {
            total_files_analyzed: 5,
            files_with_dead_code: 0,
            total_dead_lines: 0,
            dead_percentage: 0.0,
            dead_functions: 0,
            dead_classes: 0,
            dead_modules: 0,
            unreachable_blocks: 0,
        }
    }

    /// The number the gate used to read moves when `--top-files` truncates the
    /// list — which is exactly why it must not be the number the gate reads.
    #[test]
    fn summary_percentage_shrinks_with_top_files_so_it_cannot_gate() {
        let files: Vec<FileDeadCodeMetrics> = (0..5)
            .map(|i| dead_file(&format!("f{i}.rs"), 200))
            .collect();

        let mut summary = blank_summary();
        resummarize_from_listed_files(&mut summary, &files, 10_000);
        let with_all_files = summary.dead_percentage;

        // What `--top-files 1` leaves behind.
        let mut summary = blank_summary();
        resummarize_from_listed_files(&mut summary, &files[..1], 10_000);
        let with_top_1 = summary.dead_percentage;

        assert!(
            with_top_1 < with_all_files,
            "the summary percentage is list-scoped ({with_top_1} vs {with_all_files})"
        );
    }

    /// The project-wide figure the gate now reads does not move with the cap,
    /// so the same project gets the same verdict at any `--top-files`.
    #[test]
    fn the_gate_verdict_is_the_same_at_every_top_files_value() {
        // 12% dead over the whole project, whatever the list was cut down to.
        let measured = Some(12.0_f32);

        assert_eq!(
            dead_code_gate_verdict(measured, 10.0),
            DeadCodeGateVerdict::Violation(12.0)
        );
        assert_eq!(
            dead_code_gate_verdict(measured, 12.0),
            DeadCodeGateVerdict::Pass
        );
    }

    /// The gate must not be re-wired back to the list-scoped figure.
    ///
    /// The verdict function itself cannot see where its argument came from, and
    /// `handle_analyze_dead_code` calls `std::process::exit` so it cannot be
    /// driven from a unit test. The wiring is therefore pinned against this
    /// module's own source, the same way `entropy_semantic.rs` pins its banners
    /// to stderr.
    #[test]
    fn the_gate_is_not_wired_to_the_list_scoped_summary() {
        let src = include_str!("dead_code_handlers.rs");
        let call = src
            .lines()
            .find(|l| l.contains("dead_code_gate_verdict(") && !l.contains("fn "))
            .expect("the gate must call dead_code_gate_verdict");
        assert!(
            !call.contains("summary.dead_percentage"),
            "--fail-on-violation must not compare the list-scoped percentage: {call}"
        );
        assert!(
            call.contains("project_dead_percentage"),
            "--fail-on-violation must compare the project-wide percentage: {call}"
        );
    }

    /// An analyzer that never measured a project-wide percentage must not have
    /// its threshold silently reported as met.
    #[test]
    fn an_unmeasured_percentage_is_not_a_pass() {
        assert_eq!(
            dead_code_gate_verdict(None, 10.0),
            DeadCodeGateVerdict::Unmeasurable,
            "no measurement means no verdict, not a pass"
        );
    }
}
