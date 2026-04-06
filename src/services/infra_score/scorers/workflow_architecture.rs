#![cfg_attr(coverage_nightly, coverage(off))]
//! Workflow Architecture Scorer (25 points)
//!
//! WA-01 (5pts): Reusable workflow (uses: *.yml@*)
//! WA-02 (5pts): Self-hosted runners
//! WA-03 (3pts): workflow_dispatch trigger
//! WA-04 (3pts): Concurrency groups with cancel-in-progress
//! WA-05 (3pts): Gate/aggregation job (if: always())
//! WA-06 (3pts): Branch protection indicators
//! WA-07 (3pts): Required status check patterns

use super::{read_workflow_files, InfraScorer};
use crate::services::infra_score::models::*;
use async_trait::async_trait;
use std::path::Path;

pub struct WorkflowArchitectureScorer;

impl WorkflowArchitectureScorer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WorkflowArchitectureScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl InfraScorer for WorkflowArchitectureScorer {
    fn category_name(&self) -> &str {
        "Workflow Architecture"
    }

    fn max_score(&self) -> f64 {
        25.0
    }

    async fn score(&self, repo_path: &Path) -> anyhow::Result<InfraCategoryScore> {
        debug_assert!(
            repo_path.exists(),
            "repo_path must exist: {}",
            repo_path.display()
        );
        let workflows = read_workflow_files(repo_path);
        let all_content: String = workflows
            .iter()
            .map(|(_, c)| c.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        let mut checks = Vec::new();
        let mut findings = Vec::new();

        // WA-01 (5pts): Reusable workflow call
        let wa01 = check_reusable_workflow(&workflows);
        // Check ALL workflow content for sovereign-ci (not just WA-01 evidence)
        let uses_sovereign_ci = all_content.contains("sovereign-ci");
        if !wa01.passed {
            findings.push(InfraFinding {
                severity: InfraSeverity::Fail,
                check_id: "WA-01".to_string(),
                message: "No reusable workflow call found. Use `uses: org/.github/.github/workflows/*.yml@main`.".to_string(),
                location: Some(".github/workflows/".to_string()),
                impact_points: -5.0,
            });
        }
        checks.push(wa01);

        // WA-02 (5pts): Self-hosted runners
        // If sovereign-ci.yml is used, auto-pass (it runs on self-hosted)
        let wa02 = if uses_sovereign_ci {
            InfraCheck::pass(
                "WA-02",
                "Self-hosted runners",
                5.0,
                vec!["Implied by sovereign-ci.yml (self-hosted clean-room)".to_string()],
            )
        } else {
            check_self_hosted(&all_content)
        };
        if !wa02.passed {
            findings.push(InfraFinding {
                severity: InfraSeverity::Fail,
                check_id: "WA-02".to_string(),
                message: "No self-hosted runners detected. Use `runs-on: [self-hosted, ...]` for sovereignty.".to_string(),
                location: Some(".github/workflows/".to_string()),
                impact_points: -5.0,
            });
        }
        checks.push(wa02);

        // WA-03 (3pts): workflow_dispatch trigger
        let wa03 = check_workflow_dispatch(&all_content);
        if !wa03.passed {
            findings.push(InfraFinding {
                severity: InfraSeverity::Warning,
                check_id: "WA-03".to_string(),
                message: "No workflow_dispatch trigger found. Add manual trigger capability."
                    .to_string(),
                location: Some(".github/workflows/".to_string()),
                impact_points: -3.0,
            });
        }
        checks.push(wa03);

        // WA-04 (3pts): Concurrency with cancel-in-progress
        let wa04 = check_concurrency(&all_content);
        if !wa04.passed {
            findings.push(InfraFinding {
                severity: InfraSeverity::Warning,
                check_id: "WA-04".to_string(),
                message: "No concurrency group with cancel-in-progress found.".to_string(),
                location: Some(".github/workflows/".to_string()),
                impact_points: -3.0,
            });
        }
        checks.push(wa04);

        // WA-05 (3pts): Gate/aggregation job
        // If sovereign-ci.yml is used, auto-pass (it has a gate job)
        let wa05 = if uses_sovereign_ci {
            InfraCheck::pass(
                "WA-05",
                "Gate job",
                3.0,
                vec!["Implied by sovereign-ci.yml (gate job with if: always())".to_string()],
            )
        } else {
            check_gate_job(&all_content)
        };
        if !wa05.passed {
            findings.push(InfraFinding {
                severity: InfraSeverity::Warning,
                check_id: "WA-05".to_string(),
                message: "No gate/aggregation job with `if: always()` found.".to_string(),
                location: Some(".github/workflows/".to_string()),
                impact_points: -3.0,
            });
        }
        checks.push(wa05);

        // WA-06 (3pts): Branch protection
        let wa06 = check_branch_protection(repo_path);
        if !wa06.passed {
            findings.push(InfraFinding {
                severity: InfraSeverity::Warning,
                check_id: "WA-06".to_string(),
                message: "No branch protection indicators found in .github/ directory.".to_string(),
                location: Some(".github/".to_string()),
                impact_points: -3.0,
            });
        }
        checks.push(wa06);

        // WA-07 (3pts): Required status checks
        let wa07 = check_required_status_checks(&all_content);
        if !wa07.passed {
            findings.push(InfraFinding {
                severity: InfraSeverity::Warning,
                check_id: "WA-07".to_string(),
                message: "No required status check patterns (needs: [gate]) found.".to_string(),
                location: Some(".github/workflows/".to_string()),
                impact_points: -3.0,
            });
        }
        checks.push(wa07);

        Ok(InfraCategoryScore::new(self.max_score(), checks, findings))
    }
}

/// WA-01: Check for reusable workflow call (uses: *.yml@*)
fn check_reusable_workflow(workflows: &[(String, String)]) -> InfraCheck {
    for (name, content) in workflows {
        // Pattern: uses: <anything>.yml@<ref>  or  uses: <anything>.yaml@<ref>
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("uses:") || trimmed.starts_with("- uses:") {
                let value = trimmed
                    .trim_start_matches("- ")
                    .trim_start_matches("uses:")
                    .trim();
                if (value.contains(".yml@") || value.contains(".yaml@"))
                    && !value.starts_with("actions/")
                {
                    return InfraCheck::pass(
                        "WA-01",
                        "Reusable workflow",
                        5.0,
                        vec![format!("Found reusable workflow in {}: {}", name, value)],
                    );
                }
            }
        }
    }
    InfraCheck::fail(
        "WA-01",
        "Reusable workflow",
        5.0,
        vec!["No reusable workflow call (uses: *.yml@*) found".to_string()],
    )
}

/// WA-02: Check for self-hosted runners
fn check_self_hosted(content: &str) -> InfraCheck {
    debug_assert!(!content.is_empty(), "content must not be empty");
    for line in content.lines() {
        let trimmed = line.trim().to_lowercase();
        if trimmed.contains("runs-on") && trimmed.contains("self-hosted") {
            return InfraCheck::pass(
                "WA-02",
                "Self-hosted runners",
                5.0,
                vec![format!("Found self-hosted runner: {}", line.trim())],
            );
        }
    }
    InfraCheck::fail(
        "WA-02",
        "Self-hosted runners",
        5.0,
        vec!["No self-hosted runners found in workflow files".to_string()],
    )
}

/// WA-03: Check for workflow_dispatch trigger
fn check_workflow_dispatch(content: &str) -> InfraCheck {
    debug_assert!(!content.is_empty(), "content must not be empty");
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("workflow_dispatch") {
            return InfraCheck::pass(
                "WA-03",
                "Manual dispatch trigger",
                3.0,
                vec!["Found workflow_dispatch trigger".to_string()],
            );
        }
    }
    InfraCheck::fail(
        "WA-03",
        "Manual dispatch trigger",
        3.0,
        vec!["No workflow_dispatch trigger found".to_string()],
    )
}

/// WA-04: Check for concurrency with cancel-in-progress
fn check_concurrency(content: &str) -> InfraCheck {
    debug_assert!(!content.is_empty(), "content must not be empty");
    let has_concurrency = content
        .lines()
        .any(|l| l.trim().starts_with("concurrency:") || l.trim() == "concurrency:");
    let has_cancel = content.lines().any(|l| {
        let t = l.trim();
        t.contains("cancel-in-progress") && t.contains("true")
    });

    if has_concurrency && has_cancel {
        InfraCheck::pass(
            "WA-04",
            "Concurrency groups",
            3.0,
            vec!["Found concurrency with cancel-in-progress: true".to_string()],
        )
    } else {
        InfraCheck::fail(
            "WA-04",
            "Concurrency groups",
            3.0,
            vec!["No concurrency group with cancel-in-progress found".to_string()],
        )
    }
}

/// WA-05: Check for gate/aggregation job with if: always()
fn check_gate_job(content: &str) -> InfraCheck {
    debug_assert!(!content.is_empty(), "content must not be empty");
    let has_always = content.lines().any(|l| {
        let t = l.trim();
        t.contains("if:") && t.contains("always()")
    });

    if has_always {
        InfraCheck::pass(
            "WA-05",
            "Gate/aggregation job",
            3.0,
            vec!["Found gate job with if: always()".to_string()],
        )
    } else {
        InfraCheck::fail(
            "WA-05",
            "Gate/aggregation job",
            3.0,
            vec!["No gate job with if: always() found".to_string()],
        )
    }
}

/// WA-06: Check for branch protection indicators in .github/
fn check_branch_protection(repo_path: &Path) -> InfraCheck {
    debug_assert!(
        repo_path.exists(),
        "repo_path must exist: {}",
        repo_path.display()
    );
    let github_dir = repo_path.join(".github");

    // Check for various branch protection indicators
    let indicators = [
        "CODEOWNERS",
        "branch-protection.yml",
        "ruleset.yml",
        "settings.yml",
    ];

    for indicator in &indicators {
        if github_dir.join(indicator).exists() {
            return InfraCheck::pass(
                "WA-06",
                "Branch protection",
                3.0,
                vec![format!(
                    "Found branch protection indicator: .github/{}",
                    indicator
                )],
            );
        }
    }

    // Also check if workflows reference branch protection via pull_request trigger
    let workflows = read_workflow_files(repo_path);
    for (name, content) in &workflows {
        if content.contains("pull_request:") || content.contains("pull_request_target:") {
            return InfraCheck::pass(
                "WA-06",
                "Branch protection",
                3.0,
                vec![format!(
                    "Workflow {} uses pull_request trigger (implies branch protection)",
                    name
                )],
            );
        }
    }

    InfraCheck::fail(
        "WA-06",
        "Branch protection",
        3.0,
        vec!["No branch protection indicators found".to_string()],
    )
}

/// WA-07: Check for required status check patterns (needs: [...])
fn check_required_status_checks(content: &str) -> InfraCheck {
    debug_assert!(!content.is_empty(), "content must not be empty");
    let has_needs = content.lines().any(|l| {
        let t = l.trim();
        t.starts_with("needs:") || t.starts_with("needs: [")
    });

    if has_needs {
        InfraCheck::pass(
            "WA-07",
            "Required status checks",
            3.0,
            vec!["Found job dependency chain (needs:)".to_string()],
        )
    } else {
        InfraCheck::fail(
            "WA-07",
            "Required status checks",
            3.0,
            vec!["No required status check patterns found".to_string()],
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
    async fn test_empty_repo_scores_zero() {
        let tmp = TempDir::new().unwrap();
        let scorer = WorkflowArchitectureScorer::new();
        let result = scorer.score(tmp.path()).await.unwrap();
        assert!((result.score - 0.0).abs() < f64::EPSILON);
        assert_eq!(result.checks.len(), 7);
    }

    #[test]
    fn test_wa01_reusable_workflow_pass() {
        let workflows = vec![(
            "ci.yml".to_string(),
            "jobs:\n  gate:\n    uses: paiml/.github/.github/workflows/unified-gate.yml@main"
                .to_string(),
        )];
        let check = check_reusable_workflow(&workflows);
        assert!(check.passed);
        assert!((check.score - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_wa01_reusable_workflow_fail() {
        let workflows = vec![(
            "ci.yml".to_string(),
            "jobs:\n  test:\n    runs-on: ubuntu-latest".to_string(),
        )];
        let check = check_reusable_workflow(&workflows);
        assert!(!check.passed);
    }

    #[test]
    fn test_wa01_ignores_actions_uses() {
        let workflows = vec![(
            "ci.yml".to_string(),
            "    - uses: actions/checkout@v4".to_string(),
        )];
        let check = check_reusable_workflow(&workflows);
        assert!(!check.passed);
    }

    #[test]
    fn test_wa02_self_hosted_pass() {
        let check = check_self_hosted("runs-on: [self-hosted, linux, x64]");
        assert!(check.passed);
    }

    #[test]
    fn test_wa02_self_hosted_fail() {
        let check = check_self_hosted("runs-on: ubuntu-latest");
        assert!(!check.passed);
    }

    #[test]
    fn test_wa03_dispatch_pass() {
        let check = check_workflow_dispatch("on:\n  workflow_dispatch:\n  push:");
        assert!(check.passed);
    }

    #[test]
    fn test_wa03_dispatch_fail() {
        let check = check_workflow_dispatch("on:\n  push:\n    branches: [main]");
        assert!(!check.passed);
    }

    #[test]
    fn test_wa04_concurrency_pass() {
        let content = "concurrency:\n  group: ${{ github.workflow }}\n  cancel-in-progress: true";
        let check = check_concurrency(content);
        assert!(check.passed);
    }

    #[test]
    fn test_wa04_concurrency_no_cancel() {
        let content = "concurrency:\n  group: ${{ github.workflow }}";
        let check = check_concurrency(content);
        assert!(!check.passed);
    }

    #[test]
    fn test_wa05_gate_job_pass() {
        let content = "  gate:\n    if: always()\n    needs: [test, lint]";
        let check = check_gate_job(content);
        assert!(check.passed);
    }

    #[test]
    fn test_wa05_gate_job_fail() {
        let content = "  test:\n    runs-on: ubuntu-latest";
        let check = check_gate_job(content);
        assert!(!check.passed);
    }

    #[test]
    fn test_wa06_branch_protection_codeowners() {
        let tmp = TempDir::new().unwrap();
        let gh_dir = tmp.path().join(".github");
        fs::create_dir_all(&gh_dir).unwrap();
        fs::write(gh_dir.join("CODEOWNERS"), "* @org/team").unwrap();
        let check = check_branch_protection(tmp.path());
        assert!(check.passed);
    }

    #[test]
    fn test_wa06_branch_protection_pr_trigger() {
        let tmp = setup_repo_with_workflow("on:\n  pull_request:\n    branches: [main]");
        let check = check_branch_protection(tmp.path());
        assert!(check.passed);
    }

    #[test]
    fn test_wa07_required_status_pass() {
        let content = "  gate:\n    needs: [test, lint]\n    if: always()";
        let check = check_required_status_checks(content);
        assert!(check.passed);
    }

    #[test]
    fn test_wa07_required_status_fail() {
        let content = "  test:\n    runs-on: ubuntu-latest\n    steps:";
        let check = check_required_status_checks(content);
        assert!(!check.passed);
    }

    #[tokio::test]
    async fn test_full_score_perfect_workflow() {
        let content = r#"name: CI
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
  workflow_dispatch:

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

jobs:
  test:
    uses: paiml/.github/.github/workflows/unified-gate.yml@main
    with:
      runner: self-hosted
  lint:
    runs-on: [self-hosted, linux]
    steps:
      - run: cargo clippy
  gate:
    needs: [test, lint]
    if: always()
    runs-on: ubuntu-latest
    steps:
      - run: echo "gate"
"#;
        let tmp = setup_repo_with_workflow(content);
        let scorer = WorkflowArchitectureScorer::new();
        let result = scorer.score(tmp.path()).await.unwrap();
        assert!((result.score - 25.0).abs() < f64::EPSILON);
    }
}
