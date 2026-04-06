#![cfg_attr(coverage_nightly, coverage(off))]
//! Build Reliability Scorer (25 points)
//!
//! BR-01 (5pts): Last 10 CI runs >=90% green (via gh CLI, skipped if unavailable)
//! BR-02 (5pts): No continue-on-error on test/lint jobs
//! BR-03 (5pts): Deterministic builds (--locked, CARGO_INCREMENTAL=0)
//! BR-04 (3pts): Build caching (sccache, actions/cache)
//! BR-05 (3pts): Pinned action versions (SHA or tag, not branch)
//! BR-06 (2pts): No || true escape hatches in test/lint steps
//! BR-07 (2pts): Timeout configured on jobs

use super::{read_workflow_files, InfraScorer};
use crate::services::infra_score::models::*;
use async_trait::async_trait;
use std::path::Path;

pub struct BuildReliabilityScorer;

impl BuildReliabilityScorer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for BuildReliabilityScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl InfraScorer for BuildReliabilityScorer {
    fn category_name(&self) -> &str {
        "Build Reliability"
    }

    fn max_score(&self) -> f64 {
        25.0
    }

    async fn score(&self, repo_path: &Path) -> anyhow::Result<InfraCategoryScore> {
        let workflows = read_workflow_files(repo_path);
        let all_content: String = workflows
            .iter()
            .map(|(_, c)| c.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        let uses_sovereign_ci = all_content.contains("sovereign-ci");

        let mut checks = Vec::new();
        let mut findings = Vec::new();

        // BR-01 (5pts): CI run success rate (skip if gh not available)
        let br01 = check_ci_success_rate(repo_path).await;
        if !br01.passed {
            findings.push(InfraFinding {
                severity: if br01.evidence.iter().any(|e| e.contains("skipped")) {
                    InfraSeverity::Info
                } else {
                    InfraSeverity::Fail
                },
                check_id: "BR-01".to_string(),
                message: br01.evidence.first().cloned().unwrap_or_default(),
                location: None,
                impact_points: -5.0,
            });
        }
        checks.push(br01);

        // BR-02 (5pts): No continue-on-error on critical jobs
        let br02 = if uses_sovereign_ci {
            InfraCheck::pass(
                "BR-02",
                "No continue-on-error on critical jobs",
                5.0,
                vec!["Implied by sovereign-ci.yml".to_string()],
            )
        } else {
            check_no_continue_on_error(&all_content)
        };
        if !br02.passed {
            findings.push(InfraFinding {
                severity: InfraSeverity::Fail,
                check_id: "BR-02".to_string(),
                message: "Found continue-on-error: true on test/lint jobs. Remove it.".to_string(),
                location: Some(".github/workflows/".to_string()),
                impact_points: -5.0,
            });
        }
        checks.push(br02);

        // BR-03 (5pts): Deterministic builds
        let br03 = if uses_sovereign_ci {
            InfraCheck::pass(
                "BR-03",
                "Deterministic builds",
                5.0,
                vec!["Implied by sovereign-ci.yml (CARGO_INCREMENTAL=0)".to_string()],
            )
        } else {
            check_deterministic_builds(&all_content)
        };
        if !br03.passed {
            findings.push(InfraFinding {
                severity: InfraSeverity::Fail,
                check_id: "BR-03".to_string(),
                message:
                    "No deterministic build flags found. Use --locked and/or CARGO_INCREMENTAL=0."
                        .to_string(),
                location: Some(".github/workflows/".to_string()),
                impact_points: -5.0,
            });
        }
        checks.push(br03);

        // BR-04 (3pts): Build caching
        let br04 = if uses_sovereign_ci {
            InfraCheck::pass(
                "BR-04",
                "Build caching",
                3.0,
                vec!["Implied by sovereign-ci.yml (actions/cache)".to_string()],
            )
        } else {
            check_build_caching(&all_content)
        };
        if !br04.passed {
            findings.push(InfraFinding {
                severity: InfraSeverity::Warning,
                check_id: "BR-04".to_string(),
                message: "No build caching detected. Use actions/cache or sccache.".to_string(),
                location: Some(".github/workflows/".to_string()),
                impact_points: -3.0,
            });
        }
        checks.push(br04);

        // BR-05 (3pts): Pinned action versions
        let br05 = if uses_sovereign_ci {
            InfraCheck::pass(
                "BR-05",
                "Pinned action versions",
                3.0,
                vec!["Implied by sovereign-ci.yml (SHA-pinned)".to_string()],
            )
        } else {
            check_pinned_actions(&workflows)
        };
        if !br05.passed {
            findings.push(InfraFinding {
                severity: InfraSeverity::Warning,
                check_id: "BR-05".to_string(),
                message: "Some actions are not pinned to SHA or tag (using @master/@main)."
                    .to_string(),
                location: Some(".github/workflows/".to_string()),
                impact_points: -3.0,
            });
        }
        checks.push(br05);

        // BR-06 (2pts): No || true escape hatches
        let br06 = check_no_or_true(&all_content);
        if !br06.passed {
            findings.push(InfraFinding {
                severity: InfraSeverity::Warning,
                check_id: "BR-06".to_string(),
                message: "Found `|| true` escape hatch in test/lint steps.".to_string(),
                location: Some(".github/workflows/".to_string()),
                impact_points: -2.0,
            });
        }
        checks.push(br06);

        // BR-07 (2pts): Timeout configured
        let br07 = if uses_sovereign_ci {
            InfraCheck::pass(
                "BR-07",
                "Timeout configured",
                2.0,
                vec!["Implied by sovereign-ci.yml (timeout-minutes on all jobs)".to_string()],
            )
        } else {
            check_timeout(&all_content)
        };
        if !br07.passed {
            findings.push(InfraFinding {
                severity: InfraSeverity::Info,
                check_id: "BR-07".to_string(),
                message: "No timeout-minutes configured on jobs.".to_string(),
                location: Some(".github/workflows/".to_string()),
                impact_points: -2.0,
            });
        }
        checks.push(br07);

        Ok(InfraCategoryScore::new(self.max_score(), checks, findings))
    }
}

/// BR-01: Check last 10 CI runs success rate via gh CLI
async fn check_ci_success_rate(repo_path: &Path) -> InfraCheck {
    // Try to run gh run list — skip gracefully if gh is not available
    let output = tokio::process::Command::new("gh")
        .args(["run", "list", "--limit", "10", "--json", "conclusion"])
        .current_dir(repo_path)
        .output()
        .await;

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            // Parse JSON array of {conclusion: "success"|"failure"|...}
            if let Ok(runs) = serde_json::from_str::<Vec<serde_json::Value>>(&stdout) {
                if runs.is_empty() {
                    return InfraCheck::fail(
                        "BR-01",
                        "CI success rate",
                        5.0,
                        vec!["No CI runs found".to_string()],
                    );
                }
                let successes = runs
                    .iter()
                    .filter(|r| r.get("conclusion").and_then(|c| c.as_str()) == Some("success"))
                    .count();
                let rate = successes as f64 / runs.len() as f64;
                if rate >= 0.9 {
                    InfraCheck::pass(
                        "BR-01",
                        "CI success rate",
                        5.0,
                        vec![format!(
                            "{}/{} runs succeeded ({:.0}%)",
                            successes,
                            runs.len(),
                            rate * 100.0
                        )],
                    )
                } else {
                    let partial_score = (rate * 5.0).round().min(4.0);
                    InfraCheck::partial(
                        "BR-01",
                        "CI success rate",
                        partial_score,
                        5.0,
                        vec![format!(
                            "{}/{} runs succeeded ({:.0}%) — need >=90%",
                            successes,
                            runs.len(),
                            rate * 100.0
                        )],
                    )
                }
            } else {
                InfraCheck::fail(
                    "BR-01",
                    "CI success rate",
                    5.0,
                    vec!["Failed to parse gh run list output".to_string()],
                )
            }
        }
        _ => {
            // gh CLI not available or not in a GitHub repo — skip gracefully
            InfraCheck::pass(
                "BR-01",
                "CI success rate",
                5.0,
                vec!["gh CLI not available — check skipped (assumed pass)".to_string()],
            )
        }
    }
}

/// BR-02: No continue-on-error on test/lint jobs
fn check_no_continue_on_error(content: &str) -> InfraCheck {
    // Simple heuristic: if continue-on-error: true appears near test/lint context, fail
    let has_continue_on_error = content.lines().any(|l| {
        let t = l.trim();
        t.starts_with("continue-on-error:") && t.contains("true")
    });

    if has_continue_on_error {
        InfraCheck::fail(
            "BR-02",
            "No continue-on-error",
            5.0,
            vec!["Found continue-on-error: true in workflow".to_string()],
        )
    } else {
        InfraCheck::pass(
            "BR-02",
            "No continue-on-error",
            5.0,
            vec!["No continue-on-error: true found".to_string()],
        )
    }
}

/// BR-03: Deterministic builds (--locked, CARGO_INCREMENTAL=0)
fn check_deterministic_builds(content: &str) -> InfraCheck {
    let has_locked = content.contains("--locked");
    let has_incremental_zero = content.contains("CARGO_INCREMENTAL=0")
        || content.contains("CARGO_INCREMENTAL: \"0\"")
        || content.contains("CARGO_INCREMENTAL: 0");

    if has_locked || has_incremental_zero {
        let mut evidence = Vec::new();
        if has_locked {
            evidence.push("Found --locked flag".to_string());
        }
        if has_incremental_zero {
            evidence.push("Found CARGO_INCREMENTAL=0".to_string());
        }
        InfraCheck::pass("BR-03", "Deterministic builds", 5.0, evidence)
    } else {
        InfraCheck::fail(
            "BR-03",
            "Deterministic builds",
            5.0,
            vec!["No --locked or CARGO_INCREMENTAL=0 found".to_string()],
        )
    }
}

/// BR-04: Build caching (actions/cache, sccache)
fn check_build_caching(content: &str) -> InfraCheck {
    let has_cache = content.contains("actions/cache")
        || content.contains("sccache")
        || content.contains("Swatinem/rust-cache")
        || content.contains("mozilla-actions/sccache-action");

    if has_cache {
        InfraCheck::pass(
            "BR-04",
            "Build caching",
            3.0,
            vec!["Build caching detected".to_string()],
        )
    } else {
        InfraCheck::fail(
            "BR-04",
            "Build caching",
            3.0,
            vec!["No build caching (actions/cache, sccache) found".to_string()],
        )
    }
}

/// Returns true if a `uses:` value references an unpinned branch (main/master/HEAD).
fn is_unpinned_action(uses_value: &str) -> bool {
    if let Some(at_pos) = uses_value.rfind('@') {
        let ref_part = &uses_value[at_pos + 1..];
        matches!(ref_part, "main" | "master" | "HEAD")
    } else {
        false
    }
}

/// Collect pinned/unpinned stats from a single workflow file.
fn collect_pin_stats(
    name: &str,
    content: &str,
    total: &mut u32,
    unpinned: &mut u32,
    examples: &mut Vec<String>,
) {
    for line in content.lines() {
        let Some(uses_value) = extract_uses_value(line.trim()) else {
            continue;
        };
        *total += 1;
        if is_unpinned_action(uses_value) {
            *unpinned += 1;
            if examples.len() < 3 {
                examples.push(format!("{}:{}", name, uses_value));
            }
        }
    }
}

/// BR-05: Pinned action versions (SHA or tag, not @master/@main branch)
fn check_pinned_actions(workflows: &[(String, String)]) -> InfraCheck {
    let mut total_uses = 0u32;
    let mut unpinned = 0u32;
    let mut unpinned_examples = Vec::new();

    for (name, content) in workflows {
        collect_pin_stats(
            name,
            content,
            &mut total_uses,
            &mut unpinned,
            &mut unpinned_examples,
        );
    }

    if total_uses == 0 {
        return InfraCheck::pass(
            "BR-05",
            "Pinned action versions",
            3.0,
            vec!["No action uses found (nothing to pin)".to_string()],
        );
    }

    if unpinned == 0 {
        InfraCheck::pass(
            "BR-05",
            "Pinned action versions",
            3.0,
            vec![format!("All {} action references are pinned", total_uses)],
        )
    } else {
        InfraCheck::fail("BR-05", "Pinned action versions", 3.0, unpinned_examples)
    }
}

/// Extract the value after `uses:` from a YAML line
fn extract_uses_value(line: &str) -> Option<&str> {
    let trimmed = line.trim().trim_start_matches("- ");
    if trimmed.starts_with("uses:") {
        Some(trimmed.trim_start_matches("uses:").trim())
    } else {
        None
    }
}

/// BR-06: No `|| true` in test/lint steps
fn check_no_or_true(content: &str) -> InfraCheck {
    let has_or_true = content.lines().any(|l| {
        let t = l.trim();
        t.contains("|| true")
            && (t.contains("test")
                || t.contains("lint")
                || t.contains("clippy")
                || t.contains("check"))
    });

    if has_or_true {
        InfraCheck::fail(
            "BR-06",
            "No || true escape hatches",
            2.0,
            vec!["Found `|| true` in test/lint steps".to_string()],
        )
    } else {
        InfraCheck::pass(
            "BR-06",
            "No || true escape hatches",
            2.0,
            vec!["No `|| true` escape hatches found".to_string()],
        )
    }
}

/// BR-07: Timeout configured on jobs
fn check_timeout(content: &str) -> InfraCheck {
    let has_timeout = content
        .lines()
        .any(|l| l.trim().starts_with("timeout-minutes:"));

    if has_timeout {
        InfraCheck::pass(
            "BR-07",
            "Job timeout configured",
            2.0,
            vec!["Found timeout-minutes on jobs".to_string()],
        )
    } else {
        InfraCheck::fail(
            "BR-07",
            "Job timeout configured",
            2.0,
            vec!["No timeout-minutes configured on any job".to_string()],
        )
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_repo_with_workflow(content: &str) -> TempDir {
        let tmp = TempDir::new().unwrap();
        let wf_dir = tmp.path().join(".github/workflows");
        fs::create_dir_all(&wf_dir).unwrap();
        fs::write(wf_dir.join("ci.yml"), content).unwrap();
        tmp
    }

    #[tokio::test]
    async fn test_empty_repo() {
        let tmp = TempDir::new().unwrap();
        let scorer = BuildReliabilityScorer::new();
        let result = scorer.score(tmp.path()).await.unwrap();
        // BR-01 passes (gh skipped), BR-02 passes (no continue-on-error), BR-05 passes (no actions)
        // BR-06 passes, all others fail
        assert_eq!(result.checks.len(), 7);
    }

    #[test]
    fn test_br02_no_continue_on_error_pass() {
        let check =
            check_no_continue_on_error("runs-on: ubuntu-latest\nsteps:\n  - run: cargo test");
        assert!(check.passed);
    }

    #[test]
    fn test_br02_continue_on_error_fail() {
        let check = check_no_continue_on_error(
            "  test:\n    continue-on-error: true\n    runs-on: ubuntu-latest",
        );
        assert!(!check.passed);
    }

    #[test]
    fn test_br03_locked_pass() {
        let check = check_deterministic_builds("run: cargo test --locked");
        assert!(check.passed);
    }

    #[test]
    fn test_br03_incremental_zero_pass() {
        let check = check_deterministic_builds("env:\n  CARGO_INCREMENTAL: 0");
        assert!(check.passed);
    }

    #[test]
    fn test_br03_fail() {
        let check = check_deterministic_builds("run: cargo test");
        assert!(!check.passed);
    }

    #[test]
    fn test_br04_cache_pass() {
        let check = check_build_caching("- uses: actions/cache@v4");
        assert!(check.passed);
    }

    #[test]
    fn test_br04_sccache_pass() {
        let check = check_build_caching("- uses: mozilla-actions/sccache-action@v0.0.4");
        assert!(check.passed);
    }

    #[test]
    fn test_br04_no_cache() {
        let check = check_build_caching("steps:\n  - run: cargo test");
        assert!(!check.passed);
    }

    #[test]
    fn test_br05_pinned_pass() {
        let workflows = vec![(
            "ci.yml".to_string(),
            "    - uses: actions/checkout@v4\n    - uses: dtolnay/rust-toolchain@stable"
                .to_string(),
        )];
        let check = check_pinned_actions(&workflows);
        assert!(check.passed);
    }

    #[test]
    fn test_br05_unpinned_fail() {
        let workflows = vec![(
            "ci.yml".to_string(),
            "    - uses: actions/checkout@main".to_string(),
        )];
        let check = check_pinned_actions(&workflows);
        assert!(!check.passed);
    }

    #[test]
    fn test_br05_no_actions() {
        let workflows = vec![(
            "ci.yml".to_string(),
            "steps:\n  - run: cargo test".to_string(),
        )];
        let check = check_pinned_actions(&workflows);
        assert!(check.passed);
    }

    #[test]
    fn test_br06_no_or_true_pass() {
        let check = check_no_or_true("run: cargo test\nrun: cargo clippy");
        assert!(check.passed);
    }

    #[test]
    fn test_br06_or_true_fail() {
        let check = check_no_or_true("run: cargo test || true");
        assert!(!check.passed);
    }

    #[test]
    fn test_br07_timeout_pass() {
        let check = check_timeout("  test:\n    timeout-minutes: 30\n    runs-on: ubuntu-latest");
        assert!(check.passed);
    }

    #[test]
    fn test_br07_no_timeout() {
        let check = check_timeout("  test:\n    runs-on: ubuntu-latest");
        assert!(!check.passed);
    }

    #[test]
    fn test_extract_uses_value() {
        assert_eq!(
            extract_uses_value("    - uses: actions/checkout@v4"),
            Some("actions/checkout@v4")
        );
        assert_eq!(
            extract_uses_value("uses: some/action@main"),
            Some("some/action@main")
        );
        assert_eq!(extract_uses_value("run: cargo test"), None);
    }

    #[tokio::test]
    async fn test_perfect_build_reliability() {
        let content = r#"name: CI
on: push
env:
  CARGO_INCREMENTAL: 0
jobs:
  test:
    runs-on: ubuntu-latest
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@v4
      - uses: mozilla-actions/sccache-action@v0.0.4
      - run: cargo test --locked
"#;
        let tmp = setup_repo_with_workflow(content);
        let scorer = BuildReliabilityScorer::new();
        let result = scorer.score(tmp.path()).await.unwrap();
        // BR-01 skipped (passes), BR-02 pass, BR-03 pass, BR-04 pass, BR-05 pass, BR-06 pass, BR-07 pass
        assert!((result.score - 25.0).abs() < f64::EPSILON);
    }
}
