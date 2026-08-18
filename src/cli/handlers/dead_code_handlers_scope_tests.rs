//! The dead-code report may not describe a scope it did not have.
//!
//! Four claims the summary used to make and could not support:
//!
//! * **R38** — "Files skipped (out of scope): 1117 (tests, examples and
//!   benches…)" printed directly above a Top Files list made of `examples/`.
//!   The cargo scan skips only the test tree; `examples/` and `benches/` are in
//!   scope with or without `--include-tests`, and `--include-tests` raised
//!   "Files analyzed" by exactly the skipped count, proving all of them were
//!   tests. The parenthetical is now supplied by whichever analyzer did the
//!   skipping instead of being hardcoded.
//! * **R39** — one run reported three different analyzed-file counts: stdout
//!   "Files analyzed: 2", stderr "0 files analyzed" and JSON
//!   `summary.total_files_analyzed: 0`. `DeadCodeSummary::from_files` derives
//!   every field from the files it is handed, which is right for
//!   `files_with_dead_code` and wrong for a count of files READ.
//! * **R40** — the multi-language path printed "Dead code percentage: 100.0%"
//!   and, in the same run, `--fail-on-violation` bailed with "no project-wide
//!   dead-code percentage was measured for this project". The report both
//!   stated a measurement and denied making one.
//! * **R41** — `--include`/`--exclude` filter the reported list, not the walk,
//!   so "Files analyzed" and the percentage denominator do not move. Files they
//!   removed were reported as "below --min-dead-lines".

use crate::models::dead_code::{
    ConfidenceLevel, DeadCodeItem, DeadCodeResult, DeadCodeSummary, DeadCodeType,
    FileDeadCodeMetrics,
};

fn summary() -> DeadCodeSummary {
    DeadCodeSummary {
        total_files_analyzed: 40,
        files_with_dead_code: 1,
        total_dead_lines: 12,
        dead_percentage: 3.0,
        dead_functions: 1,
        dead_classes: 0,
        dead_modules: 0,
        unreachable_blocks: 0,
    }
}

fn listed_file() -> FileDeadCodeMetrics {
    FileDeadCodeMetrics {
        path: "examples/demo.rs".to_string(),
        dead_lines: 12,
        total_lines: 100,
        dead_percentage: 12.0,
        dead_functions: 1,
        dead_classes: 0,
        dead_modules: 0,
        unreachable_blocks: 0,
        dead_score: 12.0,
        confidence: ConfidenceLevel::High,
        items: vec![DeadCodeItem {
            item_type: DeadCodeType::Function,
            name: "helper".to_string(),
            line: 3,
            reason: "no callers".to_string(),
        }],
    }
}

/// A report with more project files than analyzed files, so the skipped line
/// fires, and more found-with-dead-code than listed, so the omission line does.
fn narrowed_result() -> DeadCodeResult {
    DeadCodeResult {
        summary: summary(),
        files: vec![listed_file()],
        total_files: 50,
        analyzed_files: 40,
        files_with_dead_code_found: 3,
        files_truncated: false,
        library_target: None,
    }
}

fn render(result: &DeadCodeResult, scope: super::DeadCodeReportScope) -> String {
    super::format_dead_code_as_summary_scoped(result, scope).expect("summary renders")
}

// ── R38: the skipped-file disclosure names what was actually skipped ────────

#[test]
fn the_skipped_line_repeats_the_analyzers_own_words() {
    let rendered = render(
        &narrowed_result(),
        super::DeadCodeReportScope {
            skipped_kind: Some("test code; --include-tests scans it too"),
            ..super::DeadCodeReportScope::default()
        },
    );
    assert!(
        rendered
            .contains("Files skipped (out of scope): 10 (test code; --include-tests scans it too)"),
        "{rendered}"
    );
}

/// The old text named `examples` and `benches` as skipped while listing an
/// `examples/` file two sections below. Nothing may reintroduce that pairing.
#[test]
fn the_skipped_line_does_not_claim_examples_or_benches_were_skipped() {
    let rendered = render(
        &narrowed_result(),
        super::DeadCodeReportScope {
            skipped_kind: Some("test code; --include-tests scans it too"),
            ..super::DeadCodeReportScope::default()
        },
    );
    let skipped_line = rendered
        .lines()
        .find(|l| l.contains("Files skipped"))
        .expect("the skipped line must be present");
    assert!(
        !skipped_line.contains("examples") && !skipped_line.contains("benches"),
        "skipped line still names in-scope trees: {skipped_line:?}"
    );
    // …and the thing it denied skipping is right there in the report.
    assert!(rendered.contains("examples/demo.rs"), "{rendered}");
}

/// With no analyzer to quote, the line says it does not know rather than
/// inventing a plausible category.
#[test]
fn an_unattributed_skip_says_so() {
    let rendered = render(&narrowed_result(), super::DeadCodeReportScope::default());
    assert!(
        rendered.contains("(the analyzer did not say which files)"),
        "{rendered}"
    );
}

/// The cargo path must hand over a description that matches its own policy:
/// `CargoDeadCodeAnalyzer::should_analyze` drops the test tree only.
#[test]
fn the_cargo_path_no_longer_hardcodes_examples_and_benches() {
    let src = include_str!("dead_code_handlers_output.rs");
    assert!(
        !src.contains("tests, examples and benches"),
        "the renderer must not hardcode a skipped-file category"
    );
}

// ── R40: a percentage the gate will not stand behind is labelled ────────────

#[test]
fn an_unmeasured_project_percentage_is_disclosed_beside_the_figure() {
    let rendered = render(&narrowed_result(), super::DeadCodeReportScope::default());
    assert!(
        rendered.contains("no project-wide figure was measured for this project"),
        "the multi-language report printed a bare percentage the gate then \
         refused to have measured: {rendered}"
    );
}

#[test]
fn a_measured_project_percentage_is_printed_next_to_the_listed_one() {
    let rendered = render(
        &narrowed_result(),
        super::DeadCodeReportScope {
            project_dead_percentage: Some(7.5),
            ..super::DeadCodeReportScope::default()
        },
    );
    assert!(
        rendered.contains("project-wide: 7.5%"),
        "the gate's figure must be visible beside the list-scoped one: {rendered}"
    );
    assert!(!rendered.contains("no project-wide figure"), "{rendered}");
}

/// An unnarrowed report with a real project figure says nothing extra — the
/// two numbers are the same measurement.
#[test]
fn an_unnarrowed_report_carries_no_scope_note() {
    let result = DeadCodeResult {
        files_with_dead_code_found: 1,
        ..narrowed_result()
    };
    let rendered = render(
        &result,
        super::DeadCodeReportScope {
            project_dead_percentage: Some(3.0),
            ..super::DeadCodeReportScope::default()
        },
    );
    assert!(!rendered.contains("listed files only"), "{rendered}");
}

// ── R41: --include/--exclude filter the report, not the scan ────────────────

#[test]
fn a_filtered_report_says_the_filters_did_not_narrow_the_scan() {
    let rendered = render(
        &narrowed_result(),
        super::DeadCodeReportScope {
            project_dead_percentage: Some(3.0),
            list_filtered: true,
            ..super::DeadCodeReportScope::default()
        },
    );
    assert!(
        rendered.contains("--include/--exclude filter this report, not the scan"),
        "'Files analyzed: 40' must not read as a filtered count: {rendered}"
    );
}

/// Blaming `--min-dead-lines` for a file `--exclude` removed sent readers to
/// the wrong knob.
#[test]
fn the_omission_reason_names_the_filters_when_filters_ran() {
    let rendered = render(
        &narrowed_result(),
        super::DeadCodeReportScope {
            list_filtered: true,
            ..super::DeadCodeReportScope::default()
        },
    );
    let line = rendered
        .lines()
        .find(|l| l.contains("Files found with dead code"))
        .expect("the omission line must be present");
    assert!(
        line.contains("removed by --include/--exclude"),
        "omission reason blames the wrong knob: {line:?}"
    );
}

/// The omission line used to report only how many FILES were cut, so the "Dead
/// functions: 0" printed below it read as a measurement of the project rather
/// than of what survived the filters. It names what went with them.
#[test]
fn the_omission_line_names_what_was_cut_not_just_how_many_files() {
    let rendered = render(
        &narrowed_result(),
        super::DeadCodeReportScope {
            omitted: super::DeadCodeFindingTotals {
                files: 2,
                dead_lines: 9,
                dead_functions: 1,
                dead_classes: 2,
                dead_modules: 0,
                unreachable_blocks: 0,
            },
            ..super::DeadCodeReportScope::default()
        },
    );
    let line = rendered
        .lines()
        .find(|l| l.contains("Files found with dead code"))
        .expect("the omission line must be present");
    assert!(
        line.contains("1 dead function,") && line.contains("2 dead classes"),
        "the cut items are not named: {line:?}"
    );
    // Categories with nothing in them are not invented.
    assert!(!line.contains("dead module"), "{line:?}");
}

#[test]
fn the_omission_reason_omits_filters_that_did_not_run() {
    let rendered = render(&narrowed_result(), super::DeadCodeReportScope::default());
    let line = rendered
        .lines()
        .find(|l| l.contains("Files found with dead code"))
        .expect("the omission line must be present");
    assert!(!line.contains("--include/--exclude"), "{line:?}");
    assert!(!line.contains("--top-files"), "{line:?}");
    assert!(line.contains("below --min-dead-lines"), "{line:?}");
}

// ── R39: one analyzed-file count per run, on the multi-language path ────────

/// stdout read `analyzed_files`, stderr and JSON read
/// `summary.total_files_analyzed`, and on this path the two disagreed because
/// `from_files` counted the files WITH dead code.
#[test]
fn the_multi_language_path_reports_one_analyzed_file_count() {
    let tmp = tempfile::TempDir::new().expect("tempdir");
    let root = tmp.path();
    std::fs::write(
        root.join("used.py"),
        "def used():\n    return 1\n\nused()\n",
    )
    .expect("write used.py");
    std::fs::write(root.join("dead.py"), "def never_called():\n    return 2\n")
        .expect("write dead.py");
    // Source the analyzer will not read: it picks ONE language per project.
    std::fs::write(root.join("a.go"), "package main\nfunc main() {}\n").expect("write a.go");
    std::fs::write(root.join("b.go"), "package main\nfunc helper() {}\n").expect("write b.go");
    std::fs::write(root.join("c.ts"), "export function t() {}\n").expect("write c.ts");

    let filters = super::DeadCodeAnalysisFilters {
        include_unreachable: false,
        include_tests: false,
        min_dead_lines: 0,
        top_files: None,
        include: Vec::new(),
        exclude: Vec::new(),
        max_depth: 10,
    };
    let outcome =
        super::run_multi_language_dead_code(root, &filters, "python").expect("analysis runs");

    assert_eq!(
        outcome.report.summary.total_files_analyzed, outcome.report.analyzed_files,
        "stdout reads analyzed_files while stderr and JSON read \
         summary.total_files_analyzed; they must be the same number"
    );
    assert!(
        outcome.report.analyzed_files > 0,
        "two readable .py files were analyzed, got {}",
        outcome.report.analyzed_files
    );
}

/// The three `.go`/`.ts` files went unread. Reporting them as in scope made the
/// run look like a clean bill of health over the whole tree.
#[test]
fn the_multi_language_path_discloses_the_files_it_did_not_read() {
    let tmp = tempfile::TempDir::new().expect("tempdir");
    let root = tmp.path();
    std::fs::write(root.join("dead.py"), "def never_called():\n    return 2\n")
        .expect("write dead.py");
    std::fs::write(root.join("a.go"), "package main\nfunc main() {}\n").expect("write a.go");
    std::fs::write(root.join("b.go"), "package main\nfunc helper() {}\n").expect("write b.go");
    std::fs::write(root.join("c.ts"), "export function t() {}\n").expect("write c.ts");

    let filters = super::DeadCodeAnalysisFilters {
        include_unreachable: false,
        include_tests: false,
        min_dead_lines: 0,
        top_files: None,
        include: Vec::new(),
        exclude: Vec::new(),
        max_depth: 10,
    };
    let outcome =
        super::run_multi_language_dead_code(root, &filters, "python").expect("analysis runs");

    assert!(
        outcome.report.total_files > outcome.report.analyzed_files,
        "3 of 4 source files went unread but total_files ({}) does not exceed \
         analyzed_files ({})",
        outcome.report.total_files,
        outcome.report.analyzed_files
    );
    let rendered = render(&outcome.report, outcome.scope);
    assert!(
        rendered.contains("Files skipped (out of scope):"),
        "{rendered}"
    );
    assert!(
        rendered.contains("languages this run did not read"),
        "{rendered}"
    );
}

#[test]
fn count_source_files_counts_source_and_ignores_the_rest() {
    let tmp = tempfile::TempDir::new().expect("tempdir");
    let root = tmp.path();
    for name in ["a.py", "b.go", "c.ts", "d.rs"] {
        std::fs::write(root.join(name), "x\n").expect("write source");
    }
    for name in ["README.md", "Cargo.lock", "notes.txt"] {
        std::fs::write(root.join(name), "x\n").expect("write non-source");
    }
    assert_eq!(super::count_source_files(root), 4);
}
