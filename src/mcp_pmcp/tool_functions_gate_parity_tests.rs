//! The MCP `quality_gate` tool must report what it actually ran — and run what
//! it advertises.
//!
//! Measured on a one-file fixture carrying two planted SATD markers:
//!
//! ```text
//! MCP  quality_gate {paths:[FX]}                   -> violations {satd: 2},
//!                                                     not_measured: []
//! CLI  quality-gate --project-path FX --checks all -> violations {satd: 2, coverage: 1}
//! ```
//!
//! The tool described itself as "comprehensive quality-gate checks (complexity,
//! SATD, dead code, lint, docs, etc.)" and ran TDG + SATD. The missing row is
//! the coverage check's own disclosure ("Code coverage was NOT measured …"), so
//! the surface that ran two of nine checks was also the one asserting, through
//! an empty `not_measured`, that its verdict left nothing out.
//!
//! These tests compare the two SURFACES on one fixture rather than comparing one
//! implementation to itself: the CLI half goes through
//! `analysis_utilities::handle_quality_gate`, i.e. what `pmat quality-gate`
//! runs, and the MCP half through `check_quality_gates`, i.e. what the tool
//! handler calls.

use super::{check_quality_gate_file, check_quality_gates};
use crate::cli::{QualityCheckType, QualityGateOutputFormat};
use serde_json::Value;
use std::path::{Path, PathBuf};

/// A fixture with one SATD marker the gate calls an error and one it calls a
/// warning.
///
/// The markers are assembled rather than written out: this file lives inside the
/// tree pmat gates, and a literal marker here would be a finding there — the
/// detector cannot tell a fixture from a confession.
fn fixture_source() -> String {
    let defect = format!("{}ME: handle overflow", "FIX");
    let design = format!("{}CK: no bounds check", "HA");
    format!(
        "/// Adds two numbers.\n\
         pub fn add(a: i32, b: i32) -> i32 {{\n    // {defect}\n    a + b\n}}\n\
         \n\
         /// Subtracts two numbers.\n\
         pub fn sub(a: i32, b: i32) -> i32 {{\n    // {design}\n    a - b\n}}\n"
    )
}

fn write_fixture(dir: &Path) -> PathBuf {
    let file = dir.join("lib.rs");
    std::fs::write(&file, fixture_source()).expect("write fixture");
    file
}

/// The findings, as comparable tuples: the fields both surfaces publish.
fn findings(violations: &[Value]) -> Vec<(String, String, String, String)> {
    let mut rows: Vec<_> = violations
        .iter()
        .map(|v| {
            (
                v["check_type"].as_str().unwrap_or_default().to_string(),
                v["severity"].as_str().unwrap_or_default().to_string(),
                v["file"].as_str().unwrap_or_default().to_string(),
                v["message"].as_str().unwrap_or_default().to_string(),
            )
        })
        .collect();
    rows.sort();
    rows
}

fn array(json: &Value, key: &str) -> Vec<Value> {
    json[key]
        .as_array()
        .unwrap_or_else(|| panic!("`{key}` must be an array: {json}"))
        .clone()
}

fn strings(json: &Value, key: &str) -> Vec<String> {
    array(json, key)
        .iter()
        .map(|v| v.as_str().unwrap_or_default().to_string())
        .collect()
}

/// What `pmat quality-gate --project-path <dir> --checks all --format json`
/// writes, parsed.
async fn cli_gate(project_path: &Path, out: &Path) -> Value {
    crate::cli::analysis_utilities::handle_quality_gate(
        project_path.to_path_buf(),
        None,
        QualityGateOutputFormat::Json,
        // No `--fail-on-violation`: that flag calls `std::process::exit`, which
        // would take the test harness with it.
        false,
        vec![QualityCheckType::All],
        crate::cli::analysis_utilities::GATE_DEFAULT_MAX_DEAD_CODE,
        None,
        crate::cli::analysis_utilities::GATE_DEFAULT_MAX_COMPLEXITY_P99,
        false,
        Some(out.to_path_buf()),
        false,
    )
    .await
    .expect("the CLI gate reports");
    serde_json::from_str(&std::fs::read_to_string(out).expect("the CLI wrote its report"))
        .expect("the CLI report is JSON")
}

/// Every finding `pmat quality-gate` reports over a directory, the MCP tool
/// reports too — and nothing else.
///
/// Before the fix the CLI found `{satd: 2, coverage: 1}` here and MCP found
/// `{satd: 2}`.
#[tokio::test]
async fn the_two_surfaces_report_the_same_findings_for_one_directory() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_fixture(dir.path());

    let cli = cli_gate(dir.path(), &dir.path().join("gate.json")).await;
    let mcp = check_quality_gates(&[dir.path().to_path_buf()], false)
        .await
        .expect("the MCP gate reports");

    let cli_findings = findings(&array(&cli, "violations"));
    let mcp_findings = findings(&array(&mcp, "violations"));

    assert!(
        cli_findings
            .iter()
            .any(|(check_type, ..)| check_type == "coverage"),
        "fixture assumption: an unmeasured project discloses its coverage gap: {cli_findings:?}"
    );
    assert_eq!(
        cli_findings, mcp_findings,
        "one name, one gate: `pmat quality-gate` and the MCP `quality_gate` tool \
         must report the same findings for the same path\nCLI: {cli_findings:?}\nMCP: {mcp_findings:?}"
    );
    assert_eq!(
        cli["results"]["passed"], mcp["passed"],
        "…and therefore the same verdict: CLI={cli} MCP={mcp}"
    );
}

/// The invariant behind the payload: no advertised check may be absent in
/// silence. Either it ran, or `not_measured` names it.
///
/// `not_measured: []` beside seven checks that never ran is the defect; an
/// empty list is a positive claim of full coverage, as this tool's own design
/// comment says.
fn assert_every_check_is_run_or_disclosed(gate: &Value) {
    let ran = gate["checks"]["ran"]
        .as_array()
        .unwrap_or_else(|| panic!("`checks.ran` must be an array: {gate}"))
        .iter()
        .map(|v| v.as_str().unwrap_or_default().to_string())
        .collect::<Vec<_>>();
    let not_measured = strings(gate, "not_measured");

    for check in QualityCheckType::default_checks() {
        let name = check.to_string();
        assert!(
            ran.contains(&name) || not_measured.contains(&name),
            "`{name}` is advertised by this tool but the payload neither ran it \
             nor named it in `not_measured`: {gate}"
        );
    }
}

#[tokio::test]
async fn a_directory_gate_accounts_for_every_advertised_check() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_fixture(dir.path());

    let bare = check_quality_gates(&[dir.path().to_path_buf()], false)
        .await
        .expect("the MCP gate reports");

    assert_every_check_is_run_or_disclosed(&bare);
    assert_eq!(
        strings(&bare, "not_measured"),
        vec!["coverage".to_string(), "sections".to_string()],
        "with no coverage report and no README those two checks read nothing, \
         and `ran` means measured: {bare}"
    );

    // …and the disclosure is not a constant: give the fixture the two artifacts
    // the gate reads but does not produce, and the list empties.
    crate::cli::analysis_utilities::write_gate_artifacts(dir.path(), 95.0);
    let measured = check_quality_gates(&[dir.path().to_path_buf()], false)
        .await
        .expect("the MCP gate reports");

    assert_every_check_is_run_or_disclosed(&measured);
    assert!(
        strings(&measured, "not_measured").is_empty(),
        "a directory with both artifacts answers all nine checks: {measured}"
    );
}

/// A single file cannot answer the five project-wide checks — so it must say so,
/// on both entry points, with the reason attached.
#[tokio::test]
async fn a_file_gate_names_the_checks_it_cannot_run() {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = write_fixture(dir.path());

    let by_paths = check_quality_gates(std::slice::from_ref(&file), false)
        .await
        .expect("the MCP gate reports");
    let by_file = check_quality_gate_file(&file, false)
        .await
        .expect("the MCP file gate reports");

    for (surface, gate) in [("paths", &by_paths), ("file", &by_file)] {
        assert_every_check_is_run_or_disclosed(gate);

        let not_measured = strings(gate, "not_measured");
        for check in [
            "coverage",
            "duplicates",
            "entropy",
            "provability",
            "sections",
        ] {
            assert!(
                not_measured.iter().any(|n| n == check),
                "via {surface}: `{check}` cannot run for a single file and must be \
                 named as unmeasured: {gate}"
            );
        }

        let not_run = gate["checks"]["not_run"]
            .as_array()
            .unwrap_or_else(|| panic!("`checks.not_run` must be an array: {gate}"));
        assert_eq!(
            not_run.len(),
            7,
            "via {surface}: seven project-wide checks a file cannot answer — the \
             original five plus file-size and churn (AD-05): {gate}"
        );
        assert!(
            not_run
                .iter()
                .all(|u| !u["reason"].as_str().unwrap_or_default().is_empty()),
            "via {surface}: every unrun check must carry the reason it did not run: {gate}"
        );
    }
}

/// The two SATD markers still reach both file entry points: the suite did not
/// replace a finding with a disclosure.
#[tokio::test]
async fn the_file_gate_still_reports_the_debt_it_found() {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = write_fixture(dir.path());

    let by_file = check_quality_gate_file(&file, false)
        .await
        .expect("the MCP file gate reports");

    let satd = findings(&array(&by_file, "violations"))
        .into_iter()
        .filter(|(check_type, ..)| check_type == "satd")
        .count();
    assert_eq!(satd, 2, "both planted markers are findings: {by_file}");
    assert_eq!(by_file["passed"], Value::Bool(false), "{by_file}");
}

/// The description is part of the report: a tool that names a check it does not
/// run is making the same claim an empty `not_measured` makes.
#[test]
fn the_advertised_checks_are_the_checks_it_runs() {
    use pmcp::ToolHandler;

    let info = crate::mcp_pmcp::quality_handlers::QualityGateTool::new()
        .metadata()
        .expect("quality_gate publishes metadata");
    let description = info
        .description
        .clone()
        .expect("quality_gate describes itself");

    assert!(
        !description.to_lowercase().contains("lint"),
        "the MCP suite does not run `lint` (AD-05 made it opt-in via `--checks lint`; it is not in default_checks()) — the suite runs exactly {:?}: {description}",
        QualityCheckType::default_checks()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
    for check in ["complexity", "dead code", "SATD", "coverage", "provability"] {
        assert!(
            description.contains(check),
            "the description must name the checks it runs, and `{check}` is one: {description}"
        );
    }
    assert!(
        description.contains("not_measured"),
        "…and must point at the field that discloses the rest: {description}"
    );

    // The tool is described in two places — this handler and
    // `mcp_tool_schemas/quality_gate.json`, the KAIZEN-0178 schema registry —
    // and the second one said "Checks complexity, SATD, dead code, and
    // coverage" while the first said "complexity, SATD, dead code, lint, docs".
    // Nothing serves the JSON today, which is exactly how it drifted; pin the
    // two together so a future reader cannot be told two different stories
    // about what this tool runs.
    let registered: serde_json::Value = serde_json::from_str(
        crate::mcp_pmcp::tool_schemas_generated::lookup_raw_schema("quality_gate"),
    )
    .expect("the registered schema is JSON");
    assert_eq!(
        registered["description"].as_str(),
        Some(description.as_str()),
        "one tool, one description: `mcp_tool_schemas/quality_gate.json` disagrees \
         with the handler"
    );
}
