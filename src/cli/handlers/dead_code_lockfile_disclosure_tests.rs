//! What the USER is told when the compiler layer could not run.
//!
//! `pmat analyze dead-code` used to shell out to `cargo check`, which generates
//! a `Cargo.lock` when none exists — so an analysis wrote a source-controlled
//! artifact into a repository it was only asked to read (#1076). Passing
//! `--locked` makes cargo refuse instead, and the refusal costs rustc's
//! dead-code lint: only explicit `allow(dead_code)` admissions can still be
//! found.
//!
//! That is a real loss of fidelity, and the report's SHAPE does not change when
//! it happens. `Total dead lines: 0` from a full compile and `Total dead lines:
//! 0` from a suppression scan alone are the same characters and different
//! facts. These tests pin that both the human surface and the machine surfaces
//! say which one the reader is holding.

use super::{
    format_dead_code_result, run_dead_code_analysis_with_filters, DeadCodeAnalysisFilters,
};
use crate::cli::DeadCodeOutputFormat;
use tempfile::TempDir;

fn filters() -> DeadCodeAnalysisFilters {
    DeadCodeAnalysisFilters {
        include_unreachable: false,
        include_tests: false,
        min_dead_lines: 0,
        top_files: None,
        include: Vec::new(),
        exclude: Vec::new(),
        max_depth: 10,
        no_cache: false,
    }
}

/// A crate with a dead private function and — deliberately — no lockfile.
fn crate_without_lockfile() -> TempDir {
    let temp = TempDir::new().expect("tempdir");
    std::fs::write(
        temp.path().join("Cargo.toml"),
        "[package]\nname = \"lockdiscl\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[workspace]\n",
    )
    .expect("write Cargo.toml");
    std::fs::create_dir_all(temp.path().join("src")).expect("mkdir");
    std::fs::write(
        temp.path().join("src/lib.rs"),
        "pub fn entry(n: u64) -> u64 {\n    n + 1\n}\n\nfn dead_one() -> u64 {\n    1\n}\n",
    )
    .expect("write lib.rs");
    temp
}

async fn analyse(path: &std::path::Path) -> crate::models::dead_code::DeadCodeResult {
    run_dead_code_analysis_with_filters(path, filters(), std::time::Duration::from_secs(600))
        .await
        .expect("cargo analysis runs")
        .report
}

/// The command that caused the problem must also leave the tree it analysed
/// exactly as it found it.
#[ignore = "#1076 is OPEN: --locked was reverted because it silently disabled the compiler scan (80 dead functions -> 0) on any repo with an absent or stale lockfile. This test is the SPEC for the real fix — analyse a copy, or snapshot/restore the lockfile — and must go green when that lands, not be deleted."]
#[tokio::test]
async fn the_analysis_leaves_no_lockfile_in_the_analysed_tree() {
    let temp = crate_without_lockfile();
    let _report = analyse(temp.path()).await;
    assert!(
        !temp.path().join("Cargo.lock").exists(),
        "analyse-only created Cargo.lock; whether a lockfile belongs in a tree is \
         the project's decision, and `git add -A` after this would commit one \
         nobody chose"
    );
}

/// `--format json` — the surface CI and agents read, and the one where a silent
/// reduction is most dangerous, because there is no prose beside it.
#[ignore = "#1076 is OPEN: --locked was reverted because it silently disabled the compiler scan (80 dead functions -> 0) on any repo with an absent or stale lockfile. This test is the SPEC for the real fix — analyse a copy, or snapshot/restore the lockfile — and must go green when that lands, not be deleted."]
#[tokio::test]
async fn the_json_report_declares_the_reduced_scan_and_why() {
    let temp = crate_without_lockfile();
    let report = analyse(temp.path()).await;
    let json = format_dead_code_result(
        &report,
        &DeadCodeOutputFormat::Json,
        super::DeadCodeReportScope::default(),
    )
    .expect("json renders");
    let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");

    assert_eq!(
        value["compiler_scan"]["verdict"].as_str(),
        Some("reduced"),
        "the payload does not say rustc's dead-code lint never ran, so its counts \
         read as a measurement of the whole crate: {json}"
    );
    assert_eq!(
        value["compiler_scan"]["reason"].as_str(),
        Some("lockfile-would-be-written"),
        "the cause must be a stable token, not prose a client has to grep: {json}"
    );
    let detail = value["compiler_scan"]["detail"]
        .as_str()
        .expect("a detail string");
    assert!(
        detail.contains("Cargo.lock"),
        "the reason must name the artifact that was not written: {detail}"
    );
    assert!(
        detail.contains("allow(dead_code)"),
        "the reason must say what WAS searched, so the count beside it can be \
         weighed: {detail}"
    );
}

/// The human summary carries the same disclosure, next to the figures it
/// qualifies.
#[ignore = "#1076 is OPEN: --locked was reverted because it silently disabled the compiler scan (80 dead functions -> 0) on any repo with an absent or stale lockfile. This test is the SPEC for the real fix — analyse a copy, or snapshot/restore the lockfile — and must go green when that lands, not be deleted."]
#[tokio::test]
async fn the_text_summary_declares_the_reduced_scan() {
    let temp = crate_without_lockfile();
    let report = analyse(temp.path()).await;
    let rendered = format_dead_code_result(
        &report,
        &DeadCodeOutputFormat::Summary,
        super::DeadCodeReportScope::default(),
    )
    .expect("summary renders");

    assert!(
        rendered.contains("Compiler scan:"),
        "the summary states a dead-line count without saying whether the compiler \
         layer that finds dead lines ran: {rendered}"
    );
    assert!(rendered.contains("reduced"), "{rendered}");
    assert!(
        rendered.contains("Cargo.lock"),
        "the reason must reach the human surface too, not just the JSON: {rendered}"
    );
}

/// EVERY renderer, not three of four. A disclosure one format drops is a
/// disclosure the consumer who chose that format never sees — and `sarif` is
/// what a CI pipeline ingests.
#[ignore = "#1076 is OPEN: --locked was reverted because it silently disabled the compiler scan (80 dead functions -> 0) on any repo with an absent or stale lockfile. This test is the SPEC for the real fix — analyse a copy, or snapshot/restore the lockfile — and must go green when that lands, not be deleted."]
#[tokio::test]
async fn every_output_format_carries_the_reduced_verdict() {
    let temp = crate_without_lockfile();
    let report = analyse(temp.path()).await;

    for format in [
        DeadCodeOutputFormat::Json,
        DeadCodeOutputFormat::Sarif,
        DeadCodeOutputFormat::Summary,
        DeadCodeOutputFormat::Markdown,
    ] {
        let rendered =
            format_dead_code_result(&report, &format, super::DeadCodeReportScope::default())
                .expect("renders");
        assert!(
            rendered.contains("reduced"),
            "`--format {format:?}` does not say the compiler layer was skipped, so a \
             consumer of that format reads an empty finding list as a clean crate:\n{rendered}"
        );
        assert!(
            rendered.contains("lockfile-would-be-written"),
            "`--format {format:?}` drops the machine-readable cause:\n{rendered}"
        );
    }
}

/// COUNTER-TEST at the CLI boundary: a crate WITH a lockfile is reported as a
/// full scan, and the finding only a compile can produce is in the list.
///
/// "Flag everything" — always print the reduced caveat — passes every test
/// above and fails here. So does any fix that simply stopped running cargo.
#[tokio::test]
async fn a_crate_with_a_lockfile_reports_a_full_scan_and_the_compiler_finding() {
    let temp = crate_without_lockfile();
    let lockfile = temp.path().join("Cargo.lock");
    std::fs::write(
        &lockfile,
        "# This file is automatically @generated by Cargo.\n\
         # It is not intended for manual editing.\n\
         version = 4\n\n\
         [[package]]\n\
         name = \"lockdiscl\"\n\
         version = \"0.1.0\"\n",
    )
    .expect("write lockfile");
    let before = std::fs::read(&lockfile).expect("read lockfile");

    let report = analyse(temp.path()).await;
    let json = format_dead_code_result(
        &report,
        &DeadCodeOutputFormat::Json,
        super::DeadCodeReportScope::default(),
    )
    .expect("json renders");
    let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");

    assert_eq!(
        value["compiler_scan"]["verdict"].as_str(),
        Some("full"),
        "a crate with a lockfile is compiled as before; reporting `reduced` here \
         would replace a bug with a tool that measures nothing: {json}"
    );
    assert!(
        json.contains("dead_one"),
        "`dead_one` carries no allow(dead_code), so only rustc can find it — its \
         absence means the full scan is a label, not a fact: {json}"
    );
    assert_eq!(
        before,
        std::fs::read(&lockfile).expect("read lockfile"),
        "a read-only analysis rewrote the project's lockfile"
    );
}
