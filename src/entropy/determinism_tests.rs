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

    // The dominant-pattern block used to be synthesised when there were no
    // patterns at all: {"pattern_type": "ControlFlow", "repetitions": 0,
    // "variation_score": 0.0, "example_code": ""} — a default rendered as a
    // finding about a project where nothing was found.
    assert!(
        report.pattern_summary.is_none(),
        "no pattern was found, so there is no most-common pattern to report"
    );
    assert!(json["pattern_summary"].is_null());
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

/// A file with N structurally identical lines reports N, at every N.
///
/// Observed before the fix, one file, N exact copies of one detected line:
/// N=6->6, 7->7, 8->8, 9->9, 10->10, then 11->10, 12->10, 20->10, 40->10,
/// 100->10. The headline number stopped measuring the input.
#[tokio::test]
async fn test_repetition_count_never_saturates() {
    for n in [6usize, 10, 11, 12, 20, 40, 100] {
        let mut body = String::from("pub fn check(v: &str) {\n");
        for _ in 0..n {
            body.push_str("    if v.is_empty() && v.len() > 0 { return; }\n");
        }
        body.push_str("}\n");
        let dir = write_project(&[("lib.rs", body)]);

        let report = EntropyAnalyzer::new()
            .analyze(dir.path())
            .await
            .expect("analyze");

        assert_eq!(
            report.entropy_metrics.total_instances, n,
            "{n} identical lines must report {n} instances"
        );
    }
}

/// The two halves of one sentence must agree. At N=100 the summary printed
/// "DataValidation pattern repeated 10 times (saves 43 lines)": `repetitions`
/// was frozen at the cap while `estimated_loc_reduction` kept following the real
/// count, so the message contradicted itself about how much code there is.
#[tokio::test]
async fn test_repetition_message_and_saving_agree_about_the_input() {
    let build = |copies: usize| {
        let mut body = String::from("pub fn check(v: &str) {\n");
        for _ in 0..copies {
            body.push_str("    if v.is_empty() && v.len() > 0 { return; }\n");
        }
        body.push_str("}\n");
        body
    };
    let small = build(6);
    let large = build(100);

    let analyzer = EntropyAnalyzer::new();
    let a = analyzer
        .analyze(write_project(&[("lib.rs", small)]).path())
        .await
        .expect("analyze");
    let b = analyzer
        .analyze(write_project(&[("lib.rs", large)]).path())
        .await
        .expect("analyze");

    let reps = |r: &EntropyReport| -> usize {
        r.actionable_violations
            .iter()
            .filter_map(|v| v.pattern.as_ref())
            .map(|p| p.repetitions)
            .max()
            .unwrap_or(0)
    };
    let saving = |r: &EntropyReport| r.total_loc_reduction();

    // The exact counts, not just an ordering: with the cap in place 100 copies
    // still reported 10, which is greater than 6 and so passed a rank check.
    assert_eq!(reps(&a), 6, "6 identical lines must report 6");
    assert_eq!(reps(&b), 100, "100 identical lines must report 100");
    assert!(saving(&b) > saving(&a));

    // The saving must scale with the count the message states, so the two halves
    // of "repeated N times (saves M lines)" cannot disagree. Per-instance size is
    // 3 lines for this construct and 80% is assumed removable, so the ratio of
    // savings tracks the ratio of (count - 1).
    let expected_ratio = 99.0 / 5.0;
    let actual_ratio = saving(&b) as f64 / saving(&a) as f64;
    assert!(
        (actual_ratio - expected_ratio).abs() < 1.0,
        "saving must follow the stated repetition count: {} vs {} (ratio {actual_ratio:.2}, \
         expected ~{expected_ratio:.2})",
        saving(&a),
        saving(&b)
    );
}

/// No part may exceed its whole: the lines a refactor could remove cannot
/// exceed the lines that were read.
///
/// Removing the per-file cap exposed this: `estimated_loc` was
/// `matches * loc_per_match`, a per-construct guess (5, 3, 4, 6, 2, 3 lines).
/// 100 one-line validation checks in a 102-line file were reported as
/// "Potential LOC Reduction: 237 lines (232.4% of analyzed code)".
#[tokio::test]
async fn test_reduction_never_exceeds_the_code_it_was_measured_from() {
    let analyzer = EntropyAnalyzer::new();

    // A dense single-construct file, the shape that produced 232.4%.
    for copies in [6usize, 20, 100, 400] {
        let mut body = String::from("pub fn check(v: &str) {\n");
        for _ in 0..copies {
            body.push_str("    if v.is_empty() && v.len() > 0 { return; }\n");
        }
        body.push_str("}\n");
        let dir = write_project(&[("lib.rs", body)]);
        let report = analyzer.analyze(dir.path()).await.expect("analyze");

        let loc = report.entropy_metrics.total_loc;
        let reduction = report.total_loc_reduction();
        assert!(
            reduction <= loc,
            "{copies} copies: reduction {reduction} exceeds the {loc} lines analyzed"
        );
        assert!(
            report.reduction_percentage() <= 100.0,
            "{copies} copies: reduction_percentage {} is above 100",
            report.reduction_percentage()
        );
    }

    // A multi-file tree with several constructs at once.
    let files = many_repetitive_files(20);
    let borrowed: Vec<(&str, String)> =
        files.iter().map(|(n, c)| (n.as_str(), c.clone())).collect();
    let dir = write_project(&borrowed);
    let report = analyzer.analyze(dir.path()).await.expect("analyze");
    assert!(
        report.reduction_percentage() <= 100.0,
        "multi-file: reduction_percentage {} is above 100",
        report.reduction_percentage()
    );
}

/// The measurement note must state the thresholds the extractors enforce.
///
/// It used to promise "at least 3 structurally identical occurrences" for every
/// construct. For validation the real minimum is 6: a fixture with 3 and with 5
/// identical validation lines both measured nothing while the note said 3 would
/// do, so a user who added a fourth and a fifth copy still got nothing.
#[tokio::test]
async fn test_measurement_note_states_the_enforced_thresholds() {
    let dir = write_project(&[("lib.rs", "pub fn a() -> i32 {\n    1\n}\n".to_string())]);
    let report = EntropyAnalyzer::new()
        .analyze(dir.path())
        .await
        .expect("analyze");
    let note = report.measurement_note.as_ref().expect("note");

    for threshold in crate::entropy::pattern_extractor::RUST_PATTERN_THRESHOLDS {
        let claim = format!("{} {}", threshold.name, threshold.effective_minimum());
        assert!(
            note.contains(&claim),
            "note must quote the enforced threshold {claim:?}, got: {note}"
        );
    }
    assert!(
        !note.contains("at least 3 structurally identical"),
        "the old blanket claim was false for four of six constructs: {note}"
    );
}

/// The thresholds the note quotes are the ones actually applied: one copy short
/// measures nothing, exactly that many measures something.
#[tokio::test]
async fn test_quoted_thresholds_are_the_enforced_thresholds() {
    // (line that the construct's regex matches, PatternType index)
    let cases: [(&str, PatternType); 3] = [
        (
            "    if v.is_empty() && v.len() > 0 { return; }\n",
            PatternType::DataValidation,
        ),
        (
            "    let _ = xs.iter().map(|x| x).count();\n",
            PatternType::DataTransformation,
        ),
        ("    client.send(); // call\n", PatternType::ApiCall),
    ];

    let analyzer = EntropyAnalyzer::new();
    for (line, pattern_type) in cases {
        let minimum =
            crate::entropy::pattern_extractor::rust_threshold(pattern_type).effective_minimum();

        for (copies, expect_pattern) in [(minimum - 1, false), (minimum, true)] {
            let mut body = String::from("pub fn f(v: &str, xs: &[i32], client: &C) {\n");
            for _ in 0..copies {
                body.push_str(line);
            }
            body.push_str("}\n");
            let dir = write_project(&[("lib.rs", body)]);
            let report = analyzer.analyze(dir.path()).await.expect("analyze");

            let found = report
                .entropy_metrics
                .patterns_by_type
                .contains_key(&pattern_type);
            assert_eq!(
                found, expect_pattern,
                "{pattern_type:?}: {copies} copies (threshold {minimum}) \
                 expected detected={expect_pattern}, got {found}"
            );
        }
    }
}

/// The low-diversity finding carries no fabricated pattern and no fabricated
/// saving.
///
/// It used to carry `repetitions: 0`, `example_code: "Various repetitive
/// patterns"` and `variation_score = 1 - diversity`, so one object said
/// "diversity is LOW (11.9%)" and "variation_score is HIGH (0.88)" about the
/// same number; and `estimated_loc_reduction` was `total_loc * 0.15` — a fixed
/// 15% whatever the diversity was (358 LOC -> 53, 1200 -> 180, 158020 -> 23703).
#[tokio::test]
async fn test_low_diversity_finding_carries_no_fabricated_fields() {
    let files = many_repetitive_files(6);
    let borrowed: Vec<(&str, String)> =
        files.iter().map(|(n, c)| (n.as_str(), c.clone())).collect();
    let dir = write_project(&borrowed);

    let config = EntropyConfig {
        min_pattern_diversity: 0.99, // force the finding
        ..Default::default()
    };
    let report = EntropyAnalyzer::with_config(config)
        .analyze(dir.path())
        .await
        .expect("analyze");

    let diversity = report
        .actionable_violations
        .iter()
        .find(|v| v.message.starts_with("Low pattern diversity"))
        .expect("a 99% requirement must not be silently satisfied");

    assert!(
        diversity.pattern.is_none(),
        "a project-level finding must not carry a placeholder pattern summary"
    );
    assert!(
        diversity.estimated_loc_reduction.is_none(),
        "a saving that was never derived from a measured pattern size must be absent"
    );

    let json = serde_json::to_value(&report).expect("serialize");
    let rendered = serde_json::to_string(&json).expect("string");
    assert!(
        !rendered.contains("Various repetitive patterns"),
        "the placeholder example_code must be gone: {rendered}"
    );
}

/// A saving invariant to its input is a constant, not an estimate. Three
/// projects of very different sizes must not all be told they would save ~15%.
#[tokio::test]
async fn test_diversity_finding_reports_no_fixed_percentage_saving() {
    let mut percentages = Vec::new();
    for count in [4usize, 10, 24] {
        let files = many_repetitive_files(count);
        let borrowed: Vec<(&str, String)> =
            files.iter().map(|(n, c)| (n.as_str(), c.clone())).collect();
        let dir = write_project(&borrowed);

        let config = EntropyConfig {
            min_pattern_diversity: 0.99,
            ..Default::default()
        };
        let report = EntropyAnalyzer::with_config(config)
            .analyze(dir.path())
            .await
            .expect("analyze");

        let diversity_saving = report
            .actionable_violations
            .iter()
            .find(|v| v.message.starts_with("Low pattern diversity"))
            .expect("diversity finding")
            .estimated_loc_reduction;
        assert_eq!(diversity_saving, None);

        percentages.push(report.reduction_percentage());
    }
    // With the constant gone, the reported percentage is driven by the measured
    // patterns, so it is not the same 15% at every size.
    assert!(
        percentages
            .iter()
            .any(|p| (p - percentages[0]).abs() > 0.01),
        "reduction percentage was identical at every project size: {percentages:?}"
    );
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
