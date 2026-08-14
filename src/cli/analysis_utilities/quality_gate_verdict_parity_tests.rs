//! One verdict rule, one SATD severity scale, every quality-gate surface.
//!
//! `pmat quality-gate --project-path X`, `pmat quality-gate --file X/a.rs` and
//! the MCP `quality_gate` tool used to answer the SAME file three different
//! ways: `passed:false` from both CLI entry points (`violations.is_empty()`)
//! against `passed:true` from MCP (which ignores advisory findings), and a SATD
//! severity of `warning` from `--file` against `info` from the other two,
//! because `--file` ran its own hardcoded regex instead of the SATD detector.
//! These tests pin both halves: the rule and the scale.

use super::*;
use tempfile::TempDir;

/// One advisory finding: the detector classifies this `// TODO` as
/// `Requirement` / `Low` => `severity:"info"` — reported, never blocking.
const ADVISORY_SOURCE: &str = "/// Computes the area of a rectangle.\n\
pub fn area(w: i32, h: i32) -> i32 {\n\
\x20   // TODO: support non-integer dimensions\n\
\x20   w * h\n\
}\n";

/// One blocking finding: `HACK` classifies as `Design` / `Medium` =>
/// `severity:"warning"`, which every surface must fail on.
const BLOCKING_SOURCE: &str = "/// Parses a config value.\n\
pub fn parse(s: &str) -> i32 {\n\
\x20   // HACK: this workaround will break when the format changes\n\
\x20   s.len() as i32\n\
}\n";

fn corpus(source: &str) -> (TempDir, std::path::PathBuf) {
    let dir = TempDir::new().expect("tempdir");
    let file = dir.path().join("widget.rs");
    std::fs::write(&file, source).expect("write fixture");
    (dir, file)
}

/// The CLI project gate, the CLI single-file gate and the MCP tool must return
/// the SAME verdict for the SAME corpus.
#[tokio::test]
async fn test_cli_and_mcp_quality_gate_agree_on_one_corpus() {
    for (label, source, expect_pass) in [
        ("advisory info finding", ADVISORY_SOURCE, true),
        ("blocking warning finding", BLOCKING_SOURCE, false),
    ] {
        let (dir, file) = corpus(source);

        // Surface 1: `pmat quality-gate --project-path`.
        let project_violations = check_satd(dir.path()).await.expect("check_satd");
        assert_eq!(
            project_violations.len(),
            1,
            "{label}: fixture must produce exactly one SATD finding, got {project_violations:?}"
        );
        let project_passed = violations_pass(&project_violations);

        // Surface 2: `pmat quality-gate --file`.
        let file_violations = check_satd_file(dir.path(), &file)
            .await
            .expect("check_satd_file");
        let file_passed = violations_pass(&file_violations);

        // Surface 3: the MCP `quality_gate` tool over the same directory.
        let mcp = crate::mcp_pmcp::tool_functions::check_quality_gates(
            &[dir.path().to_path_buf()],
            false,
        )
        .await
        .expect("mcp quality_gate");
        let mcp_score = mcp["score"].as_f64().unwrap_or(0.0);
        assert!(
            mcp_score >= 50.0,
            "{label}: fixture must be TDG-clean so the comparison is about the \
             verdict rule, not the score (got {mcp_score})"
        );
        let mcp_passed = mcp["passed"].as_bool().expect("mcp passed");
        let mcp_blocking = mcp["blocking_violations"]
            .as_u64()
            .expect("mcp blocking_violations") as usize;

        assert_eq!(project_passed, expect_pass, "{label}: CLI project verdict");
        assert_eq!(file_passed, expect_pass, "{label}: CLI --file verdict");
        assert_eq!(mcp_passed, expect_pass, "{label}: MCP verdict");
        assert_eq!(
            blocking_violation_count(&project_violations),
            mcp_blocking,
            "{label}: CLI and MCP must count the same blocking findings"
        );
    }
}

/// The project gate and the single-file gate must use ONE SATD classifier: same
/// severity, same line, same message for the same file. The `--file` path used
/// to stamp `warning` on a finding the project path called `info`, which is a
/// second severity scale silently deciding pass/fail.
#[tokio::test]
async fn test_project_and_single_file_satd_share_one_severity_scale() {
    for source in [ADVISORY_SOURCE, BLOCKING_SOURCE] {
        let (dir, file) = corpus(source);

        let project = check_satd(dir.path()).await.expect("check_satd");
        let single = check_satd_file(dir.path(), &file)
            .await
            .expect("check_satd_file");

        assert_eq!(
            single.len(),
            project.len(),
            "one file, one detector, one finding count"
        );
        assert_eq!(
            single[0].severity, project[0].severity,
            "one severity scale"
        );
        assert_eq!(single[0].line, project[0].line);
        assert_eq!(single[0].message, project[0].message, "one message");
        assert!(
            single[0].severity == "info" || single[0].severity == "warning",
            "detector severities map onto the gate scale, got {}",
            single[0].severity
        );
    }
}

/// The rule itself: `info` is advisory, everything else decides — including an
/// unrecognised or missing severity, which must fail closed rather than be
/// demoted to advice.
#[test]
fn test_verdict_rule_is_one_function_for_both_encodings() {
    let mk = |sev: &str| QualityViolation::new("satd", sev, "a.rs", Some(1), "x");

    assert!(!is_verdict_bearing(&mk(ADVISORY_SEVERITY)));
    assert!(is_verdict_bearing(&mk("warning")));
    assert!(is_verdict_bearing(&mk("error")));
    assert!(
        is_verdict_bearing(&mk("")),
        "an unclassified finding must fail closed"
    );

    assert!(violations_pass(&[mk("info"), mk("info")]));
    assert!(!violations_pass(&[mk("info"), mk("warning")]));
    assert_eq!(blocking_violation_count(&[mk("info"), mk("error")]), 1);

    // The JSON encoding MCP carries must answer identically, or the two
    // encodings become two rules again.
    for sev in ["info", "warning", "error", "unclassified"] {
        assert_eq!(
            json_is_verdict_bearing(&json!({ "severity": sev })),
            is_verdict_bearing(&mk(sev)),
            "severity {sev} must decide the same way in both encodings"
        );
    }
    assert!(
        json_is_verdict_bearing(&json!({ "check_type": "satd" })),
        "a finding with no severity at all must fail closed"
    );
}
