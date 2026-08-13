#![cfg_attr(coverage_nightly, coverage(off))]
//! #923, surviving half: a walk that measured nothing must not render as a
//! clean bill of health.
//!
//! Before these tests, `pmat analyze defects -p <repo>/examples` printed
//! `total_files_scanned: 117, critical: 0` and exited 0 over 117 files, 32 of
//! which contain `.unwrap()` — while `analyze satd` on the same directory
//! refused with exit 5. Two separate lies:
//!
//!   1. the count: 117 files whose every finding was suppressed unread were
//!      reported as 117 files scanned;
//!   2. the verdict: "every candidate was excluded" and "the code is clean"
//!      were the same empty vector rendered as the same sentence with exit 0.

use super::handler::{
    calculate_summary, collect_source_files, handle_analyze_defects, scan_files, ScanTally,
    EXIT_NOTHING_MEASURED,
};
use super::types::OutputFormat;
use crate::services::defect_detector::unmeasured;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const UNWRAP_SOURCE: &str = "pub fn f(x: Option<i32>) -> i32 {\n    x.unwrap()\n}\n";

/// A crate root with a manifest, so `source_scope::project_root_of` has a
/// boundary to measure layout against (#923's first half).
fn write_crate(root: &Path, files: &[(&str, &str)]) {
    std::fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("write manifest");
    for (relative, content) in files {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(path, content).expect("write source");
    }
}

/// The headline. `-p <crate>/examples` used to return 0.
#[tokio::test]
async fn test_examples_only_walk_refuses_instead_of_reporting_clean() {
    let temp = TempDir::new().expect("temp dir");
    write_crate(
        temp.path(),
        &[
            ("src/lib.rs", "pub fn ok() -> i32 { 1 }\n"),
            ("examples/e.rs", UNWRAP_SOURCE),
        ],
    );

    let exit = handle_analyze_defects(
        Some(&temp.path().join("examples")),
        None,
        None,
        OutputFormat::Json,
    )
    .await
    .expect("handler runs");

    assert_eq!(
        exit, EXIT_NOTHING_MEASURED,
        "a walk in which every discovered file was excluded took no measurement; \
         it used to exit 0 with total_files_scanned: 1, critical: 0"
    );
}

/// The same event reached through `--file`, which bypasses the walk.
#[tokio::test]
async fn test_single_excluded_file_refuses_instead_of_reporting_clean() {
    let temp = TempDir::new().expect("temp dir");
    write_crate(
        temp.path(),
        &[
            ("src/lib.rs", "pub fn ok() -> i32 { 1 }\n"),
            ("tests/t.rs", UNWRAP_SOURCE),
        ],
    );

    let exit = handle_analyze_defects(
        None,
        Some(&temp.path().join("tests/t.rs")),
        None,
        OutputFormat::Json,
    )
    .await
    .expect("handler runs");

    assert_eq!(exit, EXIT_NOTHING_MEASURED);
}

/// A directory with no Rust source in it is not a clean Rust project either.
#[tokio::test]
async fn test_directory_without_rust_sources_refuses() {
    let temp = TempDir::new().expect("temp dir");
    std::fs::write(temp.path().join("notes.md"), "# nothing here\n").expect("write");

    let exit = handle_analyze_defects(Some(temp.path()), None, None, OutputFormat::Text)
        .await
        .expect("handler runs");

    assert_eq!(
        exit, EXIT_NOTHING_MEASURED,
        "0 files read is not 0 defects found"
    );
}

/// The refusal must not swallow real work: a crate with production code still
/// reports, and still fails on a critical defect.
#[tokio::test]
async fn test_production_code_is_still_measured_and_still_fails() {
    let temp = TempDir::new().expect("temp dir");
    write_crate(
        temp.path(),
        &[
            ("src/lib.rs", UNWRAP_SOURCE),
            ("examples/e.rs", UNWRAP_SOURCE),
        ],
    );

    let exit = handle_analyze_defects(Some(temp.path()), None, None, OutputFormat::Json)
        .await
        .expect("handler runs");

    assert_eq!(exit, 1, "the src/ unwrap is still a critical defect");
}

#[tokio::test]
async fn test_clean_production_code_still_passes() {
    let temp = TempDir::new().expect("temp dir");
    write_crate(temp.path(), &[("src/lib.rs", "pub fn ok() -> i32 { 1 }\n")]);

    let exit = handle_analyze_defects(Some(temp.path()), None, None, OutputFormat::Json)
        .await
        .expect("handler runs");

    assert_eq!(exit, 0);
}

/// The count. `total_files_scanned` used to be every `.rs` file the walk
/// DISCOVERED, so a project of 3 files, 2 of them excluded unread, claimed 3
/// files scanned.
#[test]
fn test_total_files_scanned_counts_only_files_actually_analysed() {
    let temp = TempDir::new().expect("temp dir");
    write_crate(
        temp.path(),
        &[
            ("src/lib.rs", "pub fn ok() -> i32 { 1 }\n"),
            ("tests/t.rs", UNWRAP_SOURCE),
            ("examples/e.rs", UNWRAP_SOURCE),
        ],
    );

    let discovered = collect_source_files(temp.path()).expect("walk");
    assert_eq!(discovered.len(), 3, "all three files are discovered");

    let (defects, scan) = scan_files(&discovered);
    assert_eq!(scan.analysed, 1, "only src/lib.rs is production code");
    assert_eq!(
        scan.skipped.get(&unmeasured::Reason::NonProductionDir),
        Some(&2),
        "tests/ and examples/ were skipped, and the reason is retained"
    );

    let summary = calculate_summary(scan.analysed, &defects);
    assert_eq!(
        summary.total_files_scanned, 1,
        "a count of files whose findings were suppressed unread is not a count of files analysed"
    );
}

/// The refusal names which rule swallowed the walk, per reason, with counts.
#[test]
fn test_refusal_names_the_rules_that_swallowed_the_walk() {
    let temp = TempDir::new().expect("temp dir");
    write_crate(
        temp.path(),
        &[
            ("examples/a.rs", UNWRAP_SOURCE),
            ("examples/b.rs", UNWRAP_SOURCE),
        ],
    );
    let examples = temp.path().join("examples");
    let discovered = collect_source_files(&examples).expect("walk");
    let (_, scan) = scan_files(&discovered);

    let message = unmeasured::refusal(
        "defect",
        &examples,
        discovered.len(),
        &scan.describe_skips(),
        "point it somewhere else",
    );
    assert!(message.contains("all 2 source file(s)"), "{message}");
    assert!(
        message.contains("2 in the package's own tests/, benches/, examples/ or fuzz/ tree"),
        "{message}"
    );
    assert!(message.contains("This is not a clean result"), "{message}");
}

/// "No files at all" and "every file excluded" are different events and must
/// not print the same sentence.
#[test]
fn test_empty_walk_and_fully_excluded_walk_read_differently() {
    let root = PathBuf::from("/does/not/matter");
    let nothing_found = unmeasured::refusal("defect", &root, 0, "", "remedy");
    let all_excluded = unmeasured::refusal("defect", &root, 4, "4 unreadable", "remedy");

    assert!(nothing_found.contains("no source files were found"));
    assert!(all_excluded.contains("all 4 source file(s)"));
    assert_ne!(nothing_found, all_excluded);
    for message in [&nothing_found, &all_excluded] {
        assert!(
            message.contains("This is not a clean result"),
            "{message}: neither absence is a pass"
        );
    }
}

/// A tally with nothing in it must still say so out loud rather than render as
/// an empty phrase inside the refusal.
#[test]
fn test_describe_skips_never_renders_as_silence() {
    let empty = ScanTally::default();
    assert_eq!(empty.describe_skips(), "no reason recorded");
}

// =========================================================================
// #926: the command could only ever reach ONE of the five rule sets.
//
// `handle_analyze_defects` constructed a bare `RustDefectDetector` and walked
// the tree keeping `ext == "rs"`. The one Rust rule is `Critical`, so
// `--severity high|medium|low` returned `total_defects: 0` with exit 0 on
// every project in every language, and a Lua file that `analyze tdg` grades F
// on 15 critical defects was passed clean.
// =========================================================================

/// Lua: a global-assignment/`os.execute` file the TDG gate calls Critical must
/// not be reported clean by the command that exists to find critical defects.
#[tokio::test]
async fn test_lua_file_is_graded_not_skimmed() {
    let temp = TempDir::new().expect("temp dir");
    let lua = temp.path().join("bad.lua");
    let mut source: String = (0..15).map(|i| format!("g{i} = {i}\n")).collect();
    source.push_str("os.execute(\"rm -rf /\")\n");
    std::fs::write(&lua, source).expect("write lua");

    let exit = handle_analyze_defects(None, Some(&lua), None, OutputFormat::Json)
        .await
        .expect("handler runs");

    assert_eq!(
        exit, 1,
        "the Lua rule set rates this Critical; the command used to read the file with a Rust \
         regex, find nothing, and print total_files_scanned: 1, total_defects: 0, exit 0"
    );
}

/// The severity filters. Every non-Critical rule pmat owns lives in a rule set
/// the command could not reach, which is what made three of the four values
/// this flag accepts inert.
#[test]
fn test_every_severity_is_reachable_from_a_walk() {
    use crate::services::defect_detector::Severity;

    let temp = TempDir::new().expect("temp dir");
    std::fs::write(
        temp.path().join("package.json"),
        "{\"name\":\"fixture\",\"version\":\"0.1.0\"}\n",
    )
    .expect("write manifest");
    std::fs::create_dir_all(temp.path().join("src")).expect("mkdir");
    // Critical (Rust unwrap), High (TS non-null assertion), Medium (`any`),
    // Low (coercing equality), plus a Python Critical for good measure.
    std::fs::write(temp.path().join("src/lib.rs"), UNWRAP_SOURCE).expect("write rs");
    std::fs::write(
        temp.path().join("src/app.ts"),
        "export function f(u: any) {\n  return find(u)!.name;\n}\nexport const eq = (a, b) => a == b;\n",
    )
    .expect("write ts");
    std::fs::write(
        temp.path().join("src/run.py"),
        "def go(request):\n    return eval(request.body)\n",
    )
    .expect("write py");

    let discovered = collect_source_files(temp.path()).expect("walk");
    let (defects, scan) = scan_files(&discovered);
    assert_eq!(
        scan.analysed, 3,
        "three languages, three rule sets: {scan:?}"
    );

    let summary = calculate_summary(scan.analysed, &defects);
    for (label, count) in [
        ("critical", summary.by_severity.critical),
        ("high", summary.by_severity.high),
        ("medium", summary.by_severity.medium),
        ("low", summary.by_severity.low),
    ] {
        assert!(
            count > 0,
            "--severity {label} matched nothing; before #926 only `critical` could ever be \
             non-zero, in any project on earth. summary: {summary:?}"
        );
    }

    // And the filter the handler applies actually partitions them.
    for severity in [
        Severity::Critical,
        Severity::High,
        Severity::Medium,
        Severity::Low,
    ] {
        let kept: Vec<_> = defects
            .iter()
            .filter(|d| d.severity == severity)
            .cloned()
            .collect();
        assert!(
            !kept.is_empty(),
            "no rule emits {severity:?} where the walk can reach it"
        );
    }
}

/// A language pmat has no rules for is not a language pmat has found clean.
#[tokio::test]
async fn test_unsupported_language_refuses_rather_than_passing() {
    let temp = TempDir::new().expect("temp dir");
    let go = temp.path().join("main.go");
    std::fs::write(&go, "package main\n\nfunc main() {}\n").expect("write go");

    let exit = handle_analyze_defects(None, Some(&go), None, OutputFormat::Json)
        .await
        .expect("handler runs");

    assert_eq!(
        exit, EXIT_NOTHING_MEASURED,
        "`--file main.go` used to be read, graded by nothing, and reported as \
         total_files_scanned: 1, total_defects: 0, exit 0"
    );
}

/// …and the refusal it prints must send the user somewhere that can help,
/// not back to "point at the project root" — which would refuse identically.
#[test]
fn test_unsupported_language_remedy_names_the_languages_with_rules() {
    let mut scan = ScanTally::default();
    scan.skipped.insert(unmeasured::Reason::NoRuleSet, 3);
    let remedy = scan.remedy();
    assert!(remedy.contains("rs"), "{remedy}");
    assert!(remedy.contains("lua"), "{remedy}");
    assert!(!remedy.contains("project root"), "{remedy}");

    let mut excluded = ScanTally::default();
    excluded
        .skipped
        .insert(unmeasured::Reason::NonProductionDir, 3);
    assert!(excluded.remedy().contains("project root"));
}
