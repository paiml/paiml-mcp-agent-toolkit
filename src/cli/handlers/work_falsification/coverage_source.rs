#![cfg_attr(coverage_nightly, coverage(off))]
//! Real line-coverage data for the ladder's coverage claims.
//!
//! Three claims — differential coverage, absolute coverage and per-file
//! coverage — used to consult either nothing at all or
//! `target/llvm-cov/coverage.json`, a path no tool in this repository has ever
//! written (`make coverage` produces `target/coverage/lcov.info` and
//! `summary.txt`). They therefore passed on every repository without measuring
//! anything.
//!
//! This module points them at the artifact a coverage run actually leaves, via
//! the same discovery `pmat query --coverage` already uses. Coverage is far too
//! expensive to derive inside a gate — unlike cargo-deny or clippy, an
//! llvm-cov run is many minutes — so when no artifact exists the claims report
//! *not measured* rather than fabricating a verdict or blocking on data the
//! user cannot cheaply produce.

use std::collections::HashMap;
use std::path::Path;

/// Line hits per file, keyed by project-relative path.
pub(crate) type LineCoverage = HashMap<String, HashMap<usize, u64>>;

/// Load coverage produced by `make coverage` (or any llvm-cov run).
pub(crate) fn load(project_path: &Path) -> Option<LineCoverage> {
    crate::services::agent_context::query::discover_line_coverage(project_path)
}

/// How to obtain coverage, for messages that would otherwise be a dead end.
pub(crate) const COVERAGE_HINT: &str =
    "run 'make coverage' (writes target/coverage/lcov.info) to enable this claim";

/// Percentage of instrumented lines that were executed, over the whole project.
pub(crate) fn total_percent(coverage: &LineCoverage) -> Option<f64> {
    let (covered, total) = coverage.values().fold((0usize, 0usize), |(c, t), lines| {
        (
            c + lines.values().filter(|hits| **hits > 0).count(),
            t + lines.len(),
        )
    });
    (total > 0).then(|| covered as f64 * 100.0 / total as f64)
}

/// Percentage of instrumented lines executed in one file.
pub(crate) fn file_percent(lines: &HashMap<usize, u64>) -> Option<f64> {
    (!lines.is_empty()).then(|| {
        lines.values().filter(|hits| **hits > 0).count() as f64 * 100.0 / lines.len() as f64
    })
}

/// Look a file up by project-relative path, tolerating leading `./`.
///
/// lcov `SF:` records and `git diff --name-only` do not always agree on that
/// prefix, and a lookup miss here would silently read as "no uncovered lines".
pub(crate) fn lookup<'a>(
    coverage: &'a LineCoverage,
    path: &str,
) -> Option<&'a HashMap<usize, u64>> {
    let normalized = path.trim_start_matches("./");
    coverage
        .get(normalized)
        .or_else(|| coverage.get(path))
        .or_else(|| {
            coverage
                .iter()
                .find(|(k, _)| k.trim_start_matches("./") == normalized)
                .map(|(_, v)| v)
        })
}

/// Line numbers added or modified per file between `baseline` and HEAD.
///
/// Parses `git diff -U0`, whose hunk headers give the post-image start line and
/// length, so the result is exactly the set of lines this work introduced —
/// which is what a *differential* coverage claim must judge.
pub(crate) fn changed_lines(diff: &str) -> HashMap<String, Vec<usize>> {
    let mut result: HashMap<String, Vec<usize>> = HashMap::new();
    let mut current: Option<String> = None;

    for line in diff.lines() {
        if let Some(path) = line.strip_prefix("+++ b/") {
            current = (path != "/dev/null").then(|| path.to_string());
            continue;
        }
        let Some(hunk) = line.strip_prefix("@@ ") else {
            continue;
        };
        let Some(ref file) = current else { continue };
        // `@@ -old,len +new,len @@` — only the post-image side matters.
        let Some(new_part) = hunk.split_whitespace().find(|p| p.starts_with('+')) else {
            continue;
        };
        let mut nums = new_part.trim_start_matches('+').splitn(2, ',');
        let Some(start) = nums.next().and_then(|n| n.parse::<usize>().ok()) else {
            continue;
        };
        // A missing length means exactly one line; a length of 0 means a pure
        // deletion, which adds nothing to cover.
        let len = nums.next().map_or(1, |n| n.parse::<usize>().unwrap_or(1));
        if len > 0 {
            result
                .entry(file.clone())
                .or_default()
                .extend(start..start + len);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cov(pairs: &[(&str, &[(usize, u64)])]) -> LineCoverage {
        pairs
            .iter()
            .map(|(f, lines)| (f.to_string(), lines.iter().copied().collect()))
            .collect()
    }

    #[test]
    fn total_percent_counts_executed_lines() {
        let c = cov(&[("a.rs", &[(1, 1), (2, 0)]), ("b.rs", &[(1, 5), (2, 3)])]);
        assert_eq!(total_percent(&c), Some(75.0));
    }

    #[test]
    fn total_percent_is_none_without_instrumented_lines() {
        assert_eq!(total_percent(&cov(&[])), None);
        assert_eq!(total_percent(&cov(&[("a.rs", &[])])), None);
    }

    #[test]
    fn file_percent_is_per_file() {
        let c = cov(&[("a.rs", &[(1, 1), (2, 0), (3, 0), (4, 0)])]);
        assert_eq!(file_percent(lookup(&c, "a.rs").unwrap()), Some(25.0));
    }

    #[test]
    fn lookup_tolerates_a_leading_dot_slash() {
        let c = cov(&[("src/a.rs", &[(1, 1)])]);
        assert!(lookup(&c, "./src/a.rs").is_some());
        assert!(lookup(&c, "src/a.rs").is_some());
        assert!(lookup(&c, "src/missing.rs").is_none());
    }

    #[test]
    fn changed_lines_reads_the_post_image_side() {
        let diff = "diff --git a/src/a.rs b/src/a.rs\n\
                    --- a/src/a.rs\n\
                    +++ b/src/a.rs\n\
                    @@ -10,0 +11,3 @@\n\
                    +one\n+two\n+three\n";
        let changed = changed_lines(diff);
        assert_eq!(changed["src/a.rs"], vec![11, 12, 13]);
    }

    #[test]
    fn changed_lines_handles_a_single_line_hunk() {
        // A hunk with no length means exactly one line.
        let diff = "+++ b/src/a.rs\n@@ -5 +5 @@\n-old\n+new\n";
        assert_eq!(changed_lines(diff)["src/a.rs"], vec![5]);
    }

    #[test]
    fn changed_lines_ignores_pure_deletions() {
        // `+7,0` adds nothing, so there is nothing to require coverage for.
        let diff = "+++ b/src/a.rs\n@@ -7,3 +7,0 @@\n-gone\n";
        assert!(!changed_lines(diff).contains_key("src/a.rs"));
    }

    #[test]
    fn changed_lines_tracks_multiple_files() {
        let diff = "+++ b/src/a.rs\n@@ -1,0 +1,2 @@\n\
                    +++ b/src/b.rs\n@@ -1,0 +9,1 @@\n";
        let changed = changed_lines(diff);
        assert_eq!(changed["src/a.rs"], vec![1, 2]);
        assert_eq!(changed["src/b.rs"], vec![9]);
    }
}
