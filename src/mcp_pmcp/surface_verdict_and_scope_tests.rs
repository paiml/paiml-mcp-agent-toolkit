//! Three rules the MCP surface broke, pinned here.
//!
//! R13 — `quality_gate` read `let project_path = &paths[0]` and dropped every
//! other path on the floor. `{"paths":["ok.rs","a.sh"]}` answered
//! `{"passed":true,"score":90.0,"grade":"A","not_measured":[],"files_analyzed":1}`:
//! one of two paths measured, and `not_measured: []` — the field a reader
//! consults to learn what a verdict does NOT cover — asserting full coverage.
//!
//! R17 — the verdict was `tdg_passed && satd.is_empty()`, so ANY finding of ANY
//! severity decided it. Nine unchanged Rust files (the `enforce_handlers`
//! sources) scored 81.67/B+ and came back `passed:false` on one
//! `severity:"info"` row whose "finding" was the literal text `// TODO` quoted
//! inside a sentence describing a CLI requirement.
//!
//! R18 — `analyze_deep_context`'s `include_patterns` parsed into
//! `_include_patterns` and was thrown away. `{"paths":[dir]}` and
//! `{"paths":[dir],"include_patterns":["*.py"]}` both answered `file_count: 3`
//! over a directory holding `a.go app.ts main.py`, while the tool's own schema
//! advertised the argument as "accepted but not yet applied as a filter".

use crate::mcp_pmcp::tool_functions::{
    analyze_deep_context, check_quality_gate_file, check_quality_gates, quality_gate_summary,
};
use serde_json::Value;
use std::path::{Path, PathBuf};

fn write(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, body).expect("write fixture");
    path
}

/// A documented function with a bare `TODO:` — the SATD detector classifies it
/// `severity:"info"`, and TDG grades the file in the 90s.
const ADVISORY_TODO: &str =
    "/// Adds.\npub fn add(a: i32, b: i32) -> i32 {\n    // TODO: handle overflow\n    a + b\n}\n";

/// The same file with a `FIXME:` — `severity:"error"`.
const BLOCKING_FIXME: &str =
    "/// Adds.\npub fn add(a: i32, b: i32) -> i32 {\n    // FIXME: handle overflow\n    a + b\n}\n";

fn severities(json: &Value) -> Vec<String> {
    json["violations"]
        .as_array()
        .expect("violations is an array")
        .iter()
        .map(|v| v["severity"].as_str().unwrap_or_default().to_string())
        .collect()
}

fn not_measured(json: &Value) -> Vec<String> {
    json["not_measured"]
        .as_array()
        .expect("not_measured is an array")
        .iter()
        .map(|v| v.as_str().unwrap_or_default().to_string())
        .collect()
}

// ---------------------------------------------------------------------------
// R13 — every path is measured, or named as a hole with its reason
// ---------------------------------------------------------------------------

/// The exact repro: two paths in, one graded, `not_measured: []`.
#[tokio::test]
async fn a_path_this_gate_cannot_grade_is_named_not_averaged_away() {
    let dir = tempfile::tempdir().expect("tempdir");
    let graded = write(dir.path(), "ok.rs", "/// Doc.\npub fn a() -> i32 { 1 }\n");
    let ungraded = write(dir.path(), "a.sh", "#!/bin/sh\na() { echo 1; }\n");

    let json = check_quality_gates(&[graded, ungraded.clone()], false)
        .await
        .expect("quality_gate reports");

    assert!(
        not_measured(&json).contains(&ungraded.display().to_string()),
        "a path the gate could not grade must appear in not_measured, not vanish: {json}"
    );
    assert_eq!(
        json["passed"],
        Value::Bool(false),
        "half a verdict is not a pass: {json}"
    );

    // "…with its reason": the hole is a row a client can read, not a bare path.
    let reason = json["violations"]
        .as_array()
        .expect("violations")
        .iter()
        .find(|v| v["check_type"] == "not_graded" && v["file"] == ungraded.display().to_string())
        .cloned()
        .unwrap_or_else(|| panic!("no not_graded row for the ungraded path: {json}"));
    assert!(
        reason["message"]
            .as_str()
            .is_some_and(|m| m.contains(".sh")),
        "the reason must name what could not be graded: {reason}"
    );
}

/// The other half of the same defect: a path after `paths[0]` that CAN be
/// graded was never analysed at all, so it could not fail either.
#[tokio::test]
async fn every_path_the_caller_passed_is_actually_analysed() {
    let dir = tempfile::tempdir().expect("tempdir");
    let first = write(
        dir.path(),
        "first.rs",
        "/// Doc.\npub fn a() -> i32 { 1 }\n",
    );
    let second = write(
        dir.path(),
        "second.rs",
        "/// Doc.\npub fn b() -> i32 { 2 }\n",
    );

    let json = check_quality_gates(&[first, second], false)
        .await
        .expect("quality_gate reports");

    assert_eq!(
        json["files_analyzed"], 2,
        "two gradable paths in, two graded out: {json}"
    );
}

/// And a blocking finding under a later path must reach the verdict, rather
/// than being invisible because it was not `paths[0]`.
#[tokio::test]
async fn a_blocking_finding_under_a_later_path_still_fails_the_gate() {
    let dir = tempfile::tempdir().expect("tempdir");
    let clean = write(
        dir.path(),
        "clean.rs",
        "/// Doc.\npub fn a() -> i32 { 1 }\n",
    );
    let dirty = write(dir.path(), "dirty.rs", BLOCKING_FIXME);

    let json = check_quality_gates(&[clean, dirty], false)
        .await
        .expect("quality_gate reports");

    assert_eq!(
        json["passed"],
        Value::Bool(false),
        "a FIXME under paths[1] must fail the gate exactly as it does under paths[0]: {json}"
    );
}

/// `quality_gate_summary` averages the same population and reported
/// `not_measured: []` for it, because it read `ProjectScore::not_measured` —
/// a field that is only ever non-empty when NOTHING graded.
#[tokio::test]
async fn the_summary_names_the_files_its_average_left_out() {
    let dir = tempfile::tempdir().expect("tempdir");
    write(dir.path(), "ok.rs", "/// Doc.\npub fn a() -> i32 { 1 }\n");
    write(dir.path(), "a.sh", "#!/bin/sh\na() { echo 1; }\n");

    let json = quality_gate_summary(&[dir.path().to_path_buf()])
        .await
        .expect("summary");
    let summary = &json["summary"];

    assert_eq!(summary["total_files"], 1, "one of two graded: {json}");
    assert!(
        not_measured(summary)
            .iter()
            .any(|entry| entry.ends_with("a.sh")),
        "the file the average left out must be named: {json}"
    );
    assert!(
        summary["ungraded_files"]
            .as_array()
            .expect("ungraded_files")
            .iter()
            .any(|row| row["reason"].as_str().is_some_and(|r| r.contains(".sh"))),
        "…with the reason it was left out: {json}"
    );
}

// ---------------------------------------------------------------------------
// R17 — one severity rule, one place, both entry points
// ---------------------------------------------------------------------------

/// The repro, reduced: an `info` finding must be REPORTED and must not decide.
#[tokio::test]
async fn an_informational_finding_is_reported_but_does_not_decide_the_verdict() {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = write(dir.path(), "advisory.rs", ADVISORY_TODO);

    let json = check_quality_gates(&[file], false)
        .await
        .expect("quality_gate reports");

    assert_eq!(
        severities(&json),
        vec!["info".to_string()],
        "fixture must produce exactly one advisory finding and nothing else: {json}"
    );
    assert_eq!(
        json["passed"],
        Value::Bool(true),
        "an informational finding must not flip a verdict: {json}"
    );
    assert_eq!(
        json["blocking_violations"], 0,
        "the count that decided the verdict must be stated, not inferred: {json}"
    );
    assert!(
        !json["violations"]
            .as_array()
            .expect("violations")
            .is_empty(),
        "not deciding is not the same as hiding — the finding stays on the wire: {json}"
    );
}

/// The rule must not become "SATD never fails the gate".
#[tokio::test]
async fn an_error_severity_finding_still_fails_the_gate() {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = write(dir.path(), "blocking.rs", BLOCKING_FIXME);

    let json = check_quality_gates(&[file], false)
        .await
        .expect("quality_gate reports");

    assert!(
        severities(&json).contains(&"error".to_string()),
        "fixture must produce an error-severity finding: {json}"
    );
    assert_eq!(
        json["passed"],
        Value::Bool(false),
        "an actionable finding must still fail: {json}"
    );
}

/// `quality_gate`'s two entry points must apply ONE rule. They were
/// `tdg_passed && satd.is_empty()` and `tdg_passed && violations.is_empty()` —
/// two spellings of one rule, free to drift the moment either was touched.
#[tokio::test]
async fn both_entry_points_apply_the_same_severity_rule() {
    let dir = tempfile::tempdir().expect("tempdir");
    for (name, body) in [("a.rs", ADVISORY_TODO), ("b.rs", BLOCKING_FIXME)] {
        let file = write(dir.path(), name, body);
        let by_paths = check_quality_gates(std::slice::from_ref(&file), false)
            .await
            .expect("quality_gate reports");
        let by_file = check_quality_gate_file(&file, false)
            .await
            .expect("quality_gate_file reports");
        assert_eq!(
            by_paths["passed"], by_file["passed"],
            "{name}: one tool, one verdict — paths={by_paths} file={by_file}"
        );
        assert_eq!(
            by_paths["blocking_violations"], by_file["blocking_violations"],
            "{name}: the same findings must be verdict-bearing on both entry points"
        );
    }
}

/// A hole in the verdict is `severity:"error"`, so the severity rule must not
/// have quietly re-opened the "unmeasured passes" defect.
#[tokio::test]
async fn an_unmeasured_path_is_still_not_a_pass() {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = write(dir.path(), "a.sh", "echo hi\n");

    let json = check_quality_gates(&[file], false)
        .await
        .expect("quality_gate reports");

    assert_eq!(
        json["passed"],
        Value::Bool(false),
        "a gate with no measurement must not pass: {json}"
    );
}

// ---------------------------------------------------------------------------
// R18 — include_patterns is refused, not ignored
// ---------------------------------------------------------------------------

fn three_language_dir(dir: &Path) {
    write(dir, "a.go", "package main\n\nfunc a() int { return 1 }\n");
    write(dir, "app.ts", "export const a = (): number => 1;\n");
    write(dir, "main.py", "def a():\n    return 1\n");
}

/// The repro: the same `file_count` with and without the filter.
#[tokio::test]
async fn include_patterns_is_refused_rather_than_silently_ignored() {
    let dir = tempfile::tempdir().expect("tempdir");
    three_language_dir(dir.path());
    let paths = vec![dir.path().to_path_buf()];

    let unfiltered = analyze_deep_context(&paths, None)
        .await
        .expect("deep context without a filter still works");
    let baseline = unfiltered["results"]["file_count"].clone();
    assert_eq!(baseline, 3, "fixture must hold three files: {unfiltered}");

    let err = analyze_deep_context(&paths, Some(vec!["*.py".to_string()]))
        .await
        .expect_err("a filter this pipeline cannot apply must not be accepted in silence");
    let message = err.to_string();
    assert!(
        message.contains("include_patterns"),
        "the refusal must name the argument it refused: {message}"
    );
    assert!(
        message.contains("*.py"),
        "the refusal must echo what was asked for: {message}"
    );
}

/// An empty list asks for nothing, so it is not a refusal.
#[tokio::test]
async fn an_empty_include_patterns_list_is_not_a_refusal() {
    let dir = tempfile::tempdir().expect("tempdir");
    three_language_dir(dir.path());
    let paths = vec![dir.path().to_path_buf()];

    let json = analyze_deep_context(&paths, Some(Vec::new()))
        .await
        .expect("an empty filter asks for nothing and changes nothing");
    assert_eq!(json["results"]["file_count"], 3, "{json}");
}

/// The schema must not advertise a knob wired to nothing. It described
/// `include_patterns` as "accepted but not yet applied as a filter" — a defect
/// annotated and shipped.
#[test]
fn the_schema_no_longer_advertises_a_filter_the_pipeline_cannot_apply() {
    use pmcp::ToolHandler;

    let info = crate::mcp_pmcp::analyze_handlers::AnalyzeDeepContextTool::new()
        .metadata()
        .expect("analyze_deep_context publishes metadata");
    let schema = serde_json::to_value(&info.input_schema).expect("schema serialises");
    let properties = schema["properties"]
        .as_object()
        .expect("inputSchema has properties");

    assert!(
        !properties.contains_key("include_patterns"),
        "an argument the tool refuses must not be advertised as one it takes: {schema}"
    );
    assert!(
        properties.contains_key("paths"),
        "…and removing it must not have taken `paths` with it: {schema}"
    );
}
