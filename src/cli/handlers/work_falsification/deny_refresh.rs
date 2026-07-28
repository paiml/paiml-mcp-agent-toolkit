#![cfg_attr(coverage_nightly, coverage(off))]
//! GH #629: make the supply-chain claim satisfiable by refreshing its own cache.
//!
//! The gate reads `.pmat-metrics/deny-status.json` (falling back to
//! `deny-cache.txt`) and blocks once that file is older than
//! [`CACHE_BLOCK_HOURS`](super::types::CACHE_BLOCK_HOURS). Nothing in pmat, and
//! nothing pmat installs into a consumer repo, ever wrote either file — so the
//! advice it printed ("Run 'cargo deny check' first") could not clear it.
//! `cargo deny` writes to stdout, not to the cache. Every `pmat work complete`
//! therefore needed `--override-claims supply-chain`, which trains operators to
//! wave through a *security* gate.
//!
//! The fix is for pmat to populate the cache itself, from cargo-deny's exit
//! code. The O(1) contract is kept where it matters: a fresh cache is still a
//! single `stat` + parse, and the subprocess runs at most once per block window.

use super::types::CachedMetric;
use std::path::Path;
use std::process::Command;

/// Result of trying to bring the deny cache up to date.
pub(crate) enum DenyRefresh {
    /// cargo-deny ran to completion; the cache now reflects its verdict.
    Recorded(Box<CachedMetric>),
    /// cargo-deny is not available, so the claim cannot be evaluated here.
    ToolMissing,
    /// cargo-deny could not be run, or its verdict could not be persisted.
    Failed(String),
}

/// Where the refreshed verdict is written, relative to the project root.
pub(crate) const DENY_STATUS_PATH: &str = ".pmat-metrics/deny-status.json";

/// Install hint shown when cargo-deny is absent.
pub(crate) const CARGO_DENY_INSTALL_HINT: &str = "cargo install cargo-deny --locked";

/// True if cargo reported that the `deny` subcommand does not exist.
///
/// `Command::new("cargo")` succeeds whenever cargo itself is installed, so a
/// missing cargo-deny surfaces as a normal non-zero exit with this message
/// rather than as an `io::ErrorKind::NotFound`.
fn is_missing_subcommand(stderr: &str) -> bool {
    stderr.contains("no such command") || stderr.contains("no such subcommand")
}

/// Count vulnerability diagnostics in cargo-deny output.
///
/// cargo-deny emits one `error[vulnerability]` block per advisory. Bans,
/// licence and source violations use their own codes and are deliberately not
/// counted here — they still fail the gate via the exit code, but the numeric
/// evidence attached to the claim is specifically a vulnerability count.
fn count_vulnerabilities(output: &str) -> u64 {
    output.matches("error[vulnerability]").count() as u64
}

/// A one-line reason a cargo-deny run failed.
///
/// cargo-deny fails for bans, licences and sources as well as advisories, and
/// those carry no `error[vulnerability]`. Without this, such a run was reported
/// as "0 vulnerabilities" — a failure verdict phrased as a success, which is
/// worse than no message. Prefers cargo-deny's own per-check summary
/// (`advisories ok, bans ok, licenses FAILED, sources ok`), then the first
/// error diagnostic.
fn failure_summary(stdout: &str, stderr: &str) -> String {
    let lines = stdout.lines().chain(stderr.lines()).map(str::trim);
    let mut first_error = None;

    for line in lines {
        if line.contains("FAILED") && line.contains(" ok") {
            return line.to_string();
        }
        if first_error.is_none() && line.starts_with("error") {
            first_error = Some(line.to_string());
        }
    }
    first_error.unwrap_or_else(|| "cargo deny check failed".to_string())
}

/// Run `cargo deny check` in `project_path` and persist its verdict.
pub(crate) fn refresh_deny_cache(project_path: &Path) -> DenyRefresh {
    let output = match Command::new("cargo")
        .args(["deny", "check"])
        .current_dir(project_path)
        .output()
    {
        Ok(output) => output,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return DenyRefresh::ToolMissing,
        Err(e) => return DenyRefresh::Failed(format!("could not run 'cargo deny check': {e}")),
    };

    // cargo-deny writes its diagnostics to stderr and its per-check summary to
    // stdout; scan both so the count is independent of that split.
    let stderr = String::from_utf8_lossy(&output.stderr);
    if is_missing_subcommand(&stderr) {
        return DenyRefresh::ToolMissing;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);

    let passed = output.status.success();
    let count = count_vulnerabilities(&stderr) + count_vulnerabilities(&stdout);
    let value = serde_json::json!({
        "passed": passed,
        "vulnerability_count": count,
        "summary": if passed { String::new() } else { failure_summary(&stdout, &stderr) },
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "source": "pmat work complete (cargo deny check)",
    });

    if let Err(e) = write_status(project_path, &value) {
        return DenyRefresh::Failed(format!("could not write {DENY_STATUS_PATH}: {e}"));
    }

    DenyRefresh::Recorded(Box::new(CachedMetric {
        value,
        age_minutes: 0,
        is_stale_warn: false,
        is_stale_block: false,
    }))
}

/// Persist the verdict to `.pmat-metrics/deny-status.json`, creating the
/// directory if the consumer repo has never recorded a metric before.
fn write_status(project_path: &Path, value: &serde_json::Value) -> std::io::Result<()> {
    let path = project_path.join(DENY_STATUS_PATH);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(value)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_subcommand_detected_from_cargo_message() {
        // The exact wording cargo uses when cargo-deny is not installed.
        assert!(is_missing_subcommand("error: no such command: `deny`"));
        assert!(is_missing_subcommand("error: no such subcommand: `deny`"));
    }

    #[test]
    fn missing_subcommand_not_confused_with_a_real_failure() {
        assert!(!is_missing_subcommand(
            "error[vulnerability]: RUSTSEC-2024-0001"
        ));
        assert!(!is_missing_subcommand("advisories FAILED"));
        assert!(!is_missing_subcommand(""));
    }

    #[test]
    fn vulnerabilities_counted_per_diagnostic() {
        let out = "error[vulnerability]: RUSTSEC-2024-0001\n\
                   error[vulnerability]: RUSTSEC-2024-0002\n\
                   warning[unmaintained]: RUSTSEC-2024-0003\n";
        assert_eq!(count_vulnerabilities(out), 2);
    }

    #[test]
    fn failure_summary_prefers_the_per_check_line() {
        // The real shape of a licence failure, which carries no
        // error[vulnerability] and so would otherwise report "0 vulnerabilities".
        let stdout = "advisories ok, bans ok, licenses FAILED, sources ok\n";
        let stderr = "error[unlicensed]: dogfood = 0.1.0 is unlicensed\n";
        assert_eq!(
            failure_summary(stdout, stderr),
            "advisories ok, bans ok, licenses FAILED, sources ok"
        );
    }

    #[test]
    fn failure_summary_falls_back_to_the_first_error() {
        let stderr =
            "error[vulnerability]: RUSTSEC-2024-0001\nerror[vulnerability]: RUSTSEC-2024-0002\n";
        assert_eq!(
            failure_summary("", stderr),
            "error[vulnerability]: RUSTSEC-2024-0001"
        );
    }

    #[test]
    fn failure_summary_always_says_something() {
        assert_eq!(failure_summary("", ""), "cargo deny check failed");
    }

    #[test]
    fn clean_output_counts_zero() {
        assert_eq!(count_vulnerabilities("advisories ok\nbans ok\n"), 0);
        assert_eq!(count_vulnerabilities(""), 0);
    }

    #[test]
    fn write_status_creates_missing_metrics_dir() {
        // A consumer repo that has never recorded a metric has no
        // .pmat-metrics/ at all; the refresh must not fail on that.
        let dir = tempfile::tempdir().unwrap();
        let value = serde_json::json!({ "passed": true, "vulnerability_count": 0 });

        write_status(dir.path(), &value).unwrap();

        let written = std::fs::read_to_string(dir.path().join(DENY_STATUS_PATH)).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&written).unwrap();
        assert_eq!(parsed["passed"], serde_json::json!(true));
        assert_eq!(parsed["vulnerability_count"], serde_json::json!(0));
    }

    #[test]
    fn write_status_overwrites_a_stale_verdict() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".pmat-metrics")).unwrap();
        std::fs::write(
            dir.path().join(DENY_STATUS_PATH),
            r#"{"passed": true, "timestamp": "2026-07-01T21:02:57Z"}"#,
        )
        .unwrap();

        write_status(
            dir.path(),
            &serde_json::json!({ "passed": false, "vulnerability_count": 3 }),
        )
        .unwrap();

        let written = std::fs::read_to_string(dir.path().join(DENY_STATUS_PATH)).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&written).unwrap();
        assert_eq!(parsed["passed"], serde_json::json!(false));
        assert_eq!(parsed["vulnerability_count"], serde_json::json!(3));
    }
}
