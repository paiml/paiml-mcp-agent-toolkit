#![cfg_attr(coverage_nightly, coverage(off))]
//! Make the lint claim satisfiable, the same way GH #629 fixed supply chain.
//!
//! Claim [17] reads `.pmat-metrics/lint-status.json`, falling back to
//! `lint-cache.txt`, and blocks once either is over
//! [`CACHE_BLOCK_HOURS`](super::types::CACHE_BLOCK_HOURS) old. **Nothing has
//! ever written either file** — not `make lint` (whose recipe is two `echo`s
//! and a `cargo clippy`), not `scripts/record-metric.sh` (which writes the
//! differently-named `.pmat-metrics/lint.result`), not CI, not a hook. So the
//! advice it printed, "Run 'make lint' first", could not clear it, exactly as
//! with the deny cache.
//!
//! pmat now derives the verdict itself. The command run is the one **CI**
//! enforces, not the one the Makefile runs: `ci / lint` uses `--all-targets`
//! while `Makefile: lint` uses `--lib --bins`, and a gate that certifies the
//! narrower set would certify a build CI then rejects.

use super::deny_refresh::run_with_timeout;
use super::types::CachedMetric;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

/// Result of trying to bring the lint cache up to date.
pub(crate) enum LintRefresh {
    /// clippy ran to completion; the cache now reflects its verdict.
    Recorded(Box<CachedMetric>),
    /// clippy could not be run, or its verdict could not be persisted.
    Failed(String),
}

/// Where the refreshed verdict is written, relative to the project root.
pub(crate) const LINT_STATUS_PATH: &str = ".pmat-metrics/lint-status.json";

/// Clippy is a full compile of every target; give it room without hanging.
const LINT_TIMEOUT: Duration = Duration::from_secs(900);

/// Count real clippy diagnostics.
///
/// Deliberately not `contains("error")`: pmat depends on `thiserror`, so a cold
/// build's `Compiling thiserror v2.0` made that substring test report a lint
/// failure on a perfectly clean run.
pub(crate) fn count_lint_errors(output: &str) -> u64 {
    output
        .lines()
        .filter(|l| {
            let l = l.trim_start();
            l.starts_with("error[") || l.starts_with("error:")
        })
        .count() as u64
}

/// Run the lint CI enforces and persist its verdict.
pub(crate) fn refresh_lint_cache(project_path: &Path) -> LintRefresh {
    let output = match run_with_timeout(
        Command::new("cargo")
            .args([
                "clippy",
                "--all-targets",
                "--",
                "-D",
                "warnings",
                "-A",
                "unused-variables",
            ])
            .current_dir(project_path),
        LINT_TIMEOUT,
    ) {
        Ok(Some(output)) => output,
        Ok(None) => {
            return LintRefresh::Failed(format!(
                "'cargo clippy --all-targets' did not finish within {}s",
                LINT_TIMEOUT.as_secs()
            ))
        }
        Err(e) => return LintRefresh::Failed(format!("could not run 'cargo clippy': {e}")),
    };

    let stderr = String::from_utf8_lossy(&output.stderr);
    let passed = output.status.success();
    let value = serde_json::json!({
        "passed": passed,
        "error_count": count_lint_errors(&stderr),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "source": "pmat work complete (cargo clippy --all-targets)",
    });

    if let Err(e) = super::deny_refresh::write_json_status(project_path, LINT_STATUS_PATH, &value) {
        return LintRefresh::Failed(format!("could not write {LINT_STATUS_PATH}: {e}"));
    }

    LintRefresh::Recorded(Box::new(CachedMetric {
        value,
        age_minutes: 0,
        is_stale_warn: false,
        is_stale_block: false,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thiserror_is_not_a_lint_error() {
        // The bug this replaces: `!content.contains("error")` scored a clean
        // cold build as failing, because cargo prints `Compiling thiserror`.
        let out = "   Compiling thiserror v2.0.17\n\
                      Compiling pmat v3.25.0\n\
                    Finished `dev` profile\n";
        assert_eq!(count_lint_errors(out), 0);
    }

    #[test]
    fn real_diagnostics_are_counted() {
        let out = "error[E0432]: unresolved import\n\
                   warning: unused variable\n\
                   error: aborting due to 1 previous error\n";
        assert_eq!(count_lint_errors(out), 2);
    }

    #[test]
    fn indented_diagnostics_are_counted() {
        assert_eq!(count_lint_errors("  error[E0001]: boom\n"), 1);
    }

    #[test]
    fn clean_output_counts_zero() {
        assert_eq!(count_lint_errors(""), 0);
        assert_eq!(count_lint_errors("Finished `dev` profile\n"), 0);
    }
}
