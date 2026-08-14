//! R13: a project score must never drop a source file in silence.
//!
//! `quality_gate` over a directory of twelve files answered
//! `{"passed":true,"score":90.0,"grade":"A","not_measured":[],"files_analyzed":6}`.
//! Half the tree never reached the analyzer: the walk filtered on a hardcoded
//! extension whitelist and dropped the rest without producing an error, so there
//! was nothing for `not_measured` to report — and an empty `not_measured` is a
//! positive claim of full coverage. The whitelist was also narrower than TDG's
//! own language table, so `.lua` (which HAS an analyzer) was among the dropped.
//!
//! The rule these tests pin: every source file under the walked tree is either
//! graded or named with a reason. Nothing in between.

use crate::tdg::analyzer_simple::TdgAnalyzer;
use std::path::{Path, PathBuf};

/// The exact fixture from the report.
fn twelve_file_tree(dir: &Path) {
    let files: [(&str, &str); 12] = [
        ("a.rs", "pub fn a() -> i32 { 1 }\n"),
        ("a.py", "def a():\n    return 1\n"),
        ("a.go", "package main\n\nfunc a() int { return 1 }\n"),
        ("a.ts", "export const a = (): number => 1;\n"),
        ("a.js", "export const a = () => 1;\n"),
        ("a.c", "int a(void) { return 1; }\n"),
        ("a.sh", "#!/bin/sh\na() { echo 1; }\n"),
        ("a.php", "<?php function a() { return 1; }\n"),
        ("a.md", "# Title\n\nprose\n"),
        ("a.lua", "function a() return 1 end\n"),
        ("a.cs", "class A { int a() { return 1; } }\n"),
        ("a.zig", "pub fn a() i32 { return 1; }\n"),
    ];
    for (name, body) in files {
        std::fs::write(dir.join(name), body).expect("write fixture");
    }
}

fn scan(dir: &Path) -> (crate::tdg::ProjectScore, Vec<(PathBuf, String)>) {
    TdgAnalyzer::new()
        .expect("analyzer")
        .analyze_project_reporting_ungraded(dir)
        .expect("scan")
}

fn names(ungraded: &[(PathBuf, String)]) -> Vec<String> {
    ungraded
        .iter()
        .map(|(p, _)| {
            p.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_string()
        })
        .collect()
}

/// The defect: four source files vanished from the verdict entirely.
#[test]
fn source_files_this_build_cannot_grade_are_named_not_dropped() {
    let dir = tempfile::tempdir().expect("tempdir");
    twelve_file_tree(dir.path());

    let (_project, ungraded) = scan(dir.path());
    let listed = names(&ungraded);

    for expected in ["a.sh", "a.php", "a.cs", "a.zig"] {
        assert!(
            listed.iter().any(|n| n == expected),
            "{expected} is source code this build cannot grade, and it was dropped \
             from the score without appearing anywhere: {listed:?}"
        );
    }
}

/// Every refusal carries a reason, because "not measured" with no explanation
/// is only marginally better than silence.
#[test]
fn every_ungraded_file_states_why() {
    let dir = tempfile::tempdir().expect("tempdir");
    twelve_file_tree(dir.path());

    let (_project, ungraded) = scan(dir.path());
    assert!(!ungraded.is_empty(), "the fixture has ungradable source");

    for (path, reason) in &ungraded {
        assert!(
            !reason.trim().is_empty(),
            "{} was refused with no reason",
            path.display()
        );
    }
}

/// The arithmetic that made the hole invisible: `files_analyzed` counts what
/// SUCCEEDED, so a shrinking denominator reads as a clean run. Graded plus
/// disclosed must cover every source file in the tree.
#[test]
fn graded_plus_disclosed_accounts_for_every_source_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    twelve_file_tree(dir.path());

    let (project, ungraded) = scan(dir.path());

    // Eleven of the twelve fixtures are source code; `a.md` is documentation
    // and was never in a *source* average's population.
    let accounted = project.total_files + ungraded.len();
    assert_eq!(
        accounted,
        11,
        "11 source files in, {} graded + {} disclosed = {accounted} accounted for",
        project.total_files,
        ungraded.len()
    );
}

/// `.lua` has a TDG analyzer and was skipped anyway, because the walk's
/// whitelist and TDG's language table were maintained separately.
#[test]
fn a_language_with_an_analyzer_is_actually_graded() {
    let dir = tempfile::tempdir().expect("tempdir");
    twelve_file_tree(dir.path());

    let (project, ungraded) = scan(dir.path());
    let graded: Vec<String> = project
        .files
        .iter()
        .filter_map(|s| s.file_path.as_ref())
        .filter_map(|p| p.file_name().and_then(|n| n.to_str()))
        .map(str::to_string)
        .collect();

    assert!(
        graded.iter().any(|n| n == "a.lua"),
        "TDG has a Lua analyzer; the file must be scored, not skipped. \
         graded={graded:?} ungraded={:?}",
        names(&ungraded)
    );
}

/// The disclosure travels ON the score too, not only beside it: a caller
/// holding a bare `ProjectScore` (`--format json`, SARIF, the table renderer)
/// must be able to see the hole.
#[test]
fn the_project_score_itself_carries_the_skip_list() {
    let dir = tempfile::tempdir().expect("tempdir");
    twelve_file_tree(dir.path());

    let (project, ungraded) = scan(dir.path());
    assert!(
        !project.ungraded_files.is_empty(),
        "the fixture has ungradable source, so the score must carry it"
    );
    assert_eq!(
        project.ungraded_files.len(),
        ungraded.len(),
        "the score and the side channel must agree"
    );
    assert!(project
        .ungraded_files
        .iter()
        .all(|u| !u.reason.trim().is_empty()));
}

/// ...and a tree with nothing ungradable still reports an empty list, so the
/// field keeps meaning something.
#[test]
fn a_fully_graded_tree_reports_nothing_ungraded() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("a.rs"), "pub fn a() -> i32 { 1 }\n").expect("write");
    std::fs::write(dir.path().join("b.rs"), "pub fn b() -> i32 { 2 }\n").expect("write");

    let (project, ungraded) = scan(dir.path());
    assert_eq!(project.total_files, 2);
    assert!(ungraded.is_empty(), "{:?}", names(&ungraded));
    assert!(project.ungraded_files.is_empty());
}
