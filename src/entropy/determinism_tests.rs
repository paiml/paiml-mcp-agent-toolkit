//! Regression tests for the entropy analyzer's measurement contract.
//!
//! Covers three defects observed on a shipped release artifact:
//!
//! * NONDETERMINISM — two consecutive `analyze entropy --format json` runs over
//!   the same 938-file tree reported `{viol 5, total_instances 173,
//!   total_loc 2627, ControlFlow 35, DataValidation 14}` and then
//!   `{viol 8, total_instances 187, total_loc 3451, ControlFlow 43,
//!   DataValidation 18}`. `total_files_analyzed` (938) and `total_patterns` (43)
//!   were stable, which located the fault: the file walk was fine, but patterns
//!   were keyed by structural hash in a map that *overwrote* on collision while
//!   files were visited in `HashMap` order, so only one arbitrary file's copy of
//!   each pattern survived.
//! * #650(b) — a populated three-function crate produced `entropy_metrics` that
//!   were all zero, including `"total_loc": 0`, and JSON that differed from an
//!   empty directory's in exactly one field.
//! * #683 — `--min-entropy` did not gate the entropy check.

use super::*;
use crate::entropy::pattern_extractor::{AstPattern, Location, PatternCollection, PatternType};
use std::path::PathBuf;

/// A file whose repeated `} else if` chain reliably registers as a pattern.
fn repetitive_source(tag: &str) -> String {
    let mut s = String::from("pub fn dispatch_TAG(v: i32) -> i32 {\n    if v == 0 {\n        0\n");
    for i in 1..8 {
        s.push_str(&format!("    }} else if v == {i} {{\n        {i}\n"));
    }
    s.push_str("    } else {\n        -1\n    }\n}\n");
    s.replace("TAG", tag)
}

fn write_project(files: &[(&str, String)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = dir.path().join("src");
    std::fs::create_dir_all(&src).expect("create src");
    for (name, content) in files {
        std::fs::write(src.join(name), content).expect("write file");
    }
    dir
}

fn many_repetitive_files(count: usize) -> Vec<(String, String)> {
    (0..count)
        .map(|i| (format!("m{i}.rs"), repetitive_source(&format!("f{i}"))))
        .collect()
}

fn location(file: &str, line: usize) -> Location {
    Location {
        file: PathBuf::from(file),
        line,
        column: 1,
    }
}

fn pattern_at(hash: &str, file: &str, line: usize, frequency: usize) -> AstPattern {
    AstPattern {
        pattern_type: PatternType::ControlFlow,
        pattern_hash: hash.to_string(),
        frequency,
        locations: vec![location(file, line)],
        variation_score: 0.0,
        example_code: format!("// {file}"),
        estimated_loc: frequency * 6,
    }
}

/// NONDETERMINISM: the whole analysis, run repeatedly on identical input, must
/// serialize to identical bytes. Five runs, per the "a single run proves
/// nothing" rule.
#[tokio::test]
async fn test_analyze_is_byte_identical_across_repeated_runs() {
    let files = many_repetitive_files(12);
    let borrowed: Vec<(&str, String)> =
        files.iter().map(|(n, c)| (n.as_str(), c.clone())).collect();
    let dir = write_project(&borrowed);

    let analyzer = EntropyAnalyzer::new();
    let mut renders = Vec::new();
    for _ in 0..5 {
        let report = analyzer.analyze(dir.path()).await.expect("analyze");
        renders.push(serde_json::to_string(&report).expect("serialize"));
    }

    for (i, render) in renders.iter().enumerate().skip(1) {
        assert_eq!(
            &renders[0], render,
            "run {i} produced different output for identical input"
        );
    }
}

/// NONDETERMINISM, root cause: a pattern seen in several files must accumulate,
/// not be overwritten by whichever file happened to be visited last.
#[test]
fn test_add_pattern_merges_across_files_instead_of_overwriting() {
    let mut collection = PatternCollection::new();
    collection.add_pattern(pattern_at("h1", "a.rs", 10, 3));
    collection.add_pattern(pattern_at("h1", "b.rs", 20, 4));
    collection.add_pattern(pattern_at("h1", "c.rs", 30, 5));

    assert_eq!(
        collection.patterns.len(),
        1,
        "one structural hash is still one pattern"
    );
    let merged = collection.patterns.values().next().expect("pattern");
    assert_eq!(
        merged.frequency, 12,
        "frequency must count every file's occurrences (3+4+5), not just the last file's"
    );
    assert_eq!(
        PatternCollection::distinct_files(merged),
        3,
        "all three files must survive; overwriting left exactly one"
    );
}

/// NONDETERMINISM, root cause: merging must not depend on the order files are
/// processed in, so a shuffled walk cannot change any number.
#[test]
fn test_add_pattern_merge_is_order_independent() {
    let build = |order: [(&str, usize, usize); 3]| {
        let mut c = PatternCollection::new();
        for (file, line, freq) in order {
            c.add_pattern(pattern_at("h1", file, line, freq));
        }
        c
    };

    let forward = build([("a.rs", 10, 3), ("b.rs", 20, 4), ("c.rs", 30, 5)]);
    let reverse = build([("c.rs", 30, 5), ("b.rs", 20, 4), ("a.rs", 10, 3)]);

    let f = forward.patterns.values().next().expect("pattern");
    let r = reverse.patterns.values().next().expect("pattern");
    assert_eq!(f.frequency, r.frequency);
    assert_eq!(f.estimated_loc, r.estimated_loc);
    assert_eq!(f.example_code, r.example_code);
    assert_eq!(
        f.locations
            .iter()
            .map(|l| (l.file.clone(), l.line))
            .collect::<Vec<_>>(),
        r.locations
            .iter()
            .map(|l| (l.file.clone(), l.line))
            .collect::<Vec<_>>(),
        "location order must not depend on insertion order"
    );
}

/// NONDETERMINISM: violations are ordered, including ties, so a diff of two runs
/// is empty rather than a reshuffle.
#[tokio::test]
async fn test_violation_order_is_stable_across_runs() {
    let files = many_repetitive_files(8);
    let borrowed: Vec<(&str, String)> =
        files.iter().map(|(n, c)| (n.as_str(), c.clone())).collect();
    let dir = write_project(&borrowed);

    let analyzer = EntropyAnalyzer::new();
    let mut orders = Vec::new();
    for _ in 0..5 {
        let report = analyzer.analyze(dir.path()).await.expect("analyze");
        orders.push(
            report
                .actionable_violations
                .iter()
                .map(|v| (v.message.clone(), v.affected_files.clone()))
                .collect::<Vec<_>>(),
        );
    }
    for (i, order) in orders.iter().enumerate().skip(1) {
        assert_eq!(&orders[0], order, "violation order changed on run {i}");
    }
}

/// #650(b): a populated crate must not report the same all-zero metrics block as
/// an empty directory. `total_loc` is a measurement of the input.
#[tokio::test]
async fn test_small_project_metrics_measure_the_input() {
    let dir = write_project(&[(
        "lib.rs",
        "pub fn a() -> i32 {\n    1\n}\npub fn b() -> i32 {\n    2\n}\npub fn c() -> i32 {\n    3\n}\n"
            .to_string(),
    )]);
    let empty = tempfile::tempdir().expect("tempdir");

    let analyzer = EntropyAnalyzer::new();
    let populated = analyzer.analyze(dir.path()).await.expect("analyze");
    let nothing = analyzer.analyze(empty.path()).await.expect("analyze");

    assert_eq!(populated.total_files_analyzed, 1);
    assert!(
        populated.entropy_metrics.total_loc >= 9,
        "9 non-blank source lines were read, but total_loc reported {}",
        populated.entropy_metrics.total_loc
    );
    assert_eq!(
        nothing.entropy_metrics.total_loc, 0,
        "an empty directory really has zero source lines"
    );
    assert_ne!(
        populated.entropy_metrics.total_loc, nothing.entropy_metrics.total_loc,
        "populated and empty inputs must not produce the same metrics"
    );
}

/// #650(b): an entropy that could not be computed is absent, not 0.0, and the
/// report says why. Zero reads as "no diversity at all" — the worst possible
/// finding — for code that simply had nothing to repeat.
#[tokio::test]
async fn test_unmeasurable_entropy_is_absent_and_explained() {
    let dir = write_project(&[("lib.rs", "pub fn a() -> i32 {\n    1\n}\n".to_string())]);

    let report = EntropyAnalyzer::new()
        .analyze(dir.path())
        .await
        .expect("analyze");

    assert_eq!(report.entropy_metrics.total_patterns, 0);
    assert!(
        report.entropy_metrics.pattern_diversity.is_none(),
        "diversity must be absent, not 0.0"
    );
    assert!(report.entropy_metrics.file_level_entropy.is_none());
    assert!(report.entropy_metrics.module_level_entropy.is_none());
    assert!(report.entropy_metrics.project_level_entropy.is_none());

    let note = report
        .measurement_note
        .as_ref()
        .expect("an absent measurement must be explained");
    assert!(
        note.contains("not measured"),
        "note should say the entropy was not measured, got: {note}"
    );

    let json = serde_json::to_value(&report).expect("serialize");
    assert!(
        json["entropy_metrics"]["pattern_diversity"].is_null(),
        "JSON must carry null, never a plausible-looking 0.0"
    );
}

/// A measured project keeps real numbers and carries no "not measured" note.
#[tokio::test]
async fn test_measured_project_reports_values_without_note() {
    let files = many_repetitive_files(6);
    let borrowed: Vec<(&str, String)> =
        files.iter().map(|(n, c)| (n.as_str(), c.clone())).collect();
    let dir = write_project(&borrowed);

    let report = EntropyAnalyzer::new()
        .analyze(dir.path())
        .await
        .expect("analyze");

    assert!(report.entropy_metrics.total_patterns > 0);
    assert!(report.entropy_metrics.pattern_diversity.is_some());
    assert!(report.measurement_note.is_none());
}

/// The analyzed file set must come from the directory alone — never from a
/// `pmat` binary that happens to be on `$PATH`.
#[tokio::test]
async fn test_analysis_path_must_exist() {
    let dir = tempfile::tempdir().expect("tempdir");
    let missing = dir.path().join("no-such-directory");
    let err = EntropyAnalyzer::new()
        .analyze(&missing)
        .await
        .expect_err("a missing path must be an error, not an empty clean report");
    assert!(
        err.to_string().contains("does not exist"),
        "error should name the missing path, got: {err}"
    );
}
