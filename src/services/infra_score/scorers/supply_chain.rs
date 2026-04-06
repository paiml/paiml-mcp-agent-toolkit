#![cfg_attr(coverage_nightly, coverage(off))]
//! Supply Chain Security Scorer (15 points)
//!
//! SC-01 (5pts): Branch protection (org-level ruleset, CODEOWNERS, etc.)
//! SC-02 (3pts): No hardcoded secrets in workflow files
//! SC-03 (3pts): Dependency review (cargo-deny, dependabot)
//! SC-04 (2pts): Provenance/attestation (SLSA)
//! SC-05 (2pts): Signed commits config

use super::{read_workflow_files, InfraScorer};
use crate::services::infra_score::models::*;
use async_trait::async_trait;
use std::path::Path;

pub struct SupplyChainScorer;

impl SupplyChainScorer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SupplyChainScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl InfraScorer for SupplyChainScorer {
    fn category_name(&self) -> &str {
        "Supply Chain Security"
    }

    fn max_score(&self) -> f64 {
        15.0
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

        let uses_sovereign_ci = all_content.contains("sovereign-ci");

        let mut checks = Vec::new();
        let mut findings = Vec::new();

        // SC-01 (5pts): Branch protection
        let sc01 = check_branch_protection(repo_path, &workflows);
        if !sc01.passed {
            findings.push(InfraFinding {
                severity: InfraSeverity::Fail,
                check_id: "SC-01".to_string(),
                message: "No branch protection indicators found.".to_string(),
                location: Some(".github/".to_string()),
                impact_points: -5.0,
            });
        }
        checks.push(sc01);

        // SC-02 (3pts): No hardcoded secrets
        let sc02 = check_no_hardcoded_secrets(&all_content);
        if !sc02.passed {
            findings.push(InfraFinding {
                severity: InfraSeverity::Fail,
                check_id: "SC-02".to_string(),
                message: "Potential hardcoded secrets found in workflow files.".to_string(),
                location: Some(".github/workflows/".to_string()),
                impact_points: -3.0,
            });
        }
        checks.push(sc02);

        // SC-03 (3pts): Dependency review
        let sc03 = check_dependency_review(&all_content, repo_path);
        if !sc03.passed {
            findings.push(InfraFinding {
                severity: InfraSeverity::Warning,
                check_id: "SC-03".to_string(),
                message: "No dependency review tool found (cargo-deny, dependabot).".to_string(),
                location: None,
                impact_points: -3.0,
            });
        }
        checks.push(sc03);

        // SC-04 (2pts): Provenance/attestation
        let sc04 = if uses_sovereign_ci {
            InfraCheck::pass(
                "SC-04",
                "SLSA provenance",
                2.0,
                vec!["Implied by sovereign-ci.yml (attest-build-provenance)".to_string()],
            )
        } else {
            check_provenance(&all_content)
        };
        if !sc04.passed {
            findings.push(InfraFinding {
                severity: InfraSeverity::Info,
                check_id: "SC-04".to_string(),
                message: "No SLSA provenance or attestation found.".to_string(),
                location: Some(".github/workflows/".to_string()),
                impact_points: -2.0,
            });
        }
        checks.push(sc04);

        // SC-05 (2pts): Signed commits
        let sc05 = check_signed_commits(repo_path, &all_content);
        if !sc05.passed {
            findings.push(InfraFinding {
                severity: InfraSeverity::Info,
                check_id: "SC-05".to_string(),
                message: "No signed commits configuration found.".to_string(),
                location: None,
                impact_points: -2.0,
            });
        }
        checks.push(sc05);

        // HD-01 (advisory): Dangerous workflow pattern detection
        let hd01 = check_dangerous_workflow(&workflows);
        if !hd01.passed {
            findings.push(InfraFinding {
                severity: InfraSeverity::Warning,
                check_id: "HD-01".to_string(),
                message: "Untrusted context interpolation in run: blocks.".to_string(),
                location: Some(".github/workflows/".to_string()),
                impact_points: 0.0, // Advisory — no point deduction
            });
        }

        Ok(InfraCategoryScore::new(self.max_score(), checks, findings))
    }
}

/// SC-01: Branch protection (CODEOWNERS, rulesets, PR requirements)
fn check_branch_protection(repo_path: &Path, workflows: &[(String, String)]) -> InfraCheck {
    debug_assert!(
        repo_path.exists(),
        "repo_path must exist: {}",
        repo_path.display()
    );
    let github_dir = repo_path.join(".github");

    // Check for CODEOWNERS
    if github_dir.join("CODEOWNERS").exists() || repo_path.join("CODEOWNERS").exists() {
        return InfraCheck::pass(
            "SC-01",
            "Branch protection",
            5.0,
            vec!["Found CODEOWNERS file".to_string()],
        );
    }

    // Check for ruleset/settings files
    let settings_files = ["settings.yml", "branch-protection.yml", "ruleset.yml"];
    for file in &settings_files {
        if github_dir.join(file).exists() {
            return InfraCheck::pass(
                "SC-01",
                "Branch protection",
                5.0,
                vec![format!("Found .github/{}", file)],
            );
        }
    }

    // Check if any workflow uses pull_request trigger (implies branch protection)
    for (name, content) in workflows {
        if content.contains("pull_request:") || content.contains("pull_request_target:") {
            return InfraCheck::pass(
                "SC-01",
                "Branch protection",
                5.0,
                vec![format!("Workflow {} uses pull_request trigger", name)],
            );
        }
    }

    InfraCheck::fail(
        "SC-01",
        "Branch protection",
        5.0,
        vec!["No branch protection indicators found".to_string()],
    )
}

/// SC-02: No hardcoded secrets in workflow files
fn check_no_hardcoded_secrets(content: &str) -> InfraCheck {
    let secret_patterns = [
        // API keys and tokens
        "AKIA",           // AWS access key prefix
        "sk-",            // OpenAI/Stripe secret key prefix
        "ghp_",           // GitHub personal access token
        "ghs_",           // GitHub server-to-server token
        "github_pat_",    // GitHub fine-grained PAT
        "-----BEGIN RSA", // Private key
        "-----BEGIN PRIVATE",
    ];

    for pattern in &secret_patterns {
        for line in content.lines() {
            let trimmed = line.trim();
            // Skip comment lines and ${{ secrets.* }} references
            if trimmed.starts_with('#') || trimmed.contains("secrets.") {
                continue;
            }
            if trimmed.contains(pattern) {
                return InfraCheck::fail(
                    "SC-02",
                    "No hardcoded secrets",
                    3.0,
                    vec![format!("Potential secret pattern found: {}", pattern)],
                );
            }
        }
    }

    InfraCheck::pass(
        "SC-02",
        "No hardcoded secrets",
        3.0,
        vec!["No hardcoded secret patterns detected".to_string()],
    )
}

/// SC-03: Dependency review (cargo-deny, dependabot, etc.)
fn check_dependency_review(content: &str, repo_path: &Path) -> InfraCheck {
    debug_assert!(
        repo_path.exists(),
        "repo_path must exist: {}",
        repo_path.display()
    );
    // Check workflow content for dependency review tools
    let dep_review_patterns = [
        "cargo-deny",
        "cargo deny",
        "dependency-review-action",
        "dependabot",
        "snyk",
        "renovate",
    ];

    for pattern in &dep_review_patterns {
        if content.contains(pattern) {
            return InfraCheck::pass(
                "SC-03",
                "Dependency review",
                3.0,
                vec![format!("Found dependency review: {}", pattern)],
            );
        }
    }

    // Check for config files
    let config_files = [
        ".github/dependabot.yml",
        ".github/dependabot.yaml",
        "deny.toml",
        "renovate.json",
        ".renovaterc",
    ];

    for file in &config_files {
        if repo_path.join(file).exists() {
            return InfraCheck::pass(
                "SC-03",
                "Dependency review",
                3.0,
                vec![format!("Found dependency review config: {}", file)],
            );
        }
    }

    InfraCheck::fail(
        "SC-03",
        "Dependency review",
        3.0,
        vec!["No dependency review tool found".to_string()],
    )
}

/// SC-04: SLSA provenance / attestation
fn check_provenance(content: &str) -> InfraCheck {
    let provenance_patterns = [
        "slsa-framework",
        "slsa-verifier",
        "slsa-github-generator",
        "attest-build-provenance",
        "provenance",
        "attestation",
        "sigstore",
        "cosign",
    ];

    for pattern in &provenance_patterns {
        if content.to_lowercase().contains(pattern) {
            return InfraCheck::pass(
                "SC-04",
                "Provenance/attestation",
                2.0,
                vec![format!("Found provenance indicator: {}", pattern)],
            );
        }
    }

    InfraCheck::fail(
        "SC-04",
        "Provenance/attestation",
        2.0,
        vec!["No SLSA provenance or attestation found".to_string()],
    )
}

/// SC-05: Signed commits configuration
fn check_signed_commits(repo_path: &Path, content: &str) -> InfraCheck {
    debug_assert!(
        repo_path.exists(),
        "repo_path must exist: {}",
        repo_path.display()
    );
    // Check for GPG/SSH signing indicators
    if content.contains("verify-signatures")
        || content.contains("gpg")
        || content.contains("signed")
    {
        return InfraCheck::pass(
            "SC-05",
            "Signed commits",
            2.0,
            vec!["Found signing reference in workflows".to_string()],
        );
    }

    // Check for .gitsigners or commit signing config
    let signing_files = [".gitsigners", ".allowed_signers"];
    for file in &signing_files {
        if repo_path.join(file).exists() {
            return InfraCheck::pass(
                "SC-05",
                "Signed commits",
                2.0,
                vec![format!("Found signing config: {}", file)],
            );
        }
    }

    // Check if org ruleset is referenced (implies signed commits through org policy)
    if content.contains("ruleset") || content.contains("branch-protection") {
        return InfraCheck::pass(
            "SC-05",
            "Signed commits",
            2.0,
            vec!["Org ruleset reference found (may enforce signing)".to_string()],
        );
    }

    InfraCheck::fail(
        "SC-05",
        "Signed commits",
        2.0,
        vec!["No signed commits configuration found".to_string()],
    )
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_empty_repo() {
        let tmp = TempDir::new().unwrap();
        let scorer = SupplyChainScorer::new();
        let result = scorer.score(tmp.path()).await.unwrap();
        // SC-02 passes (no secrets in empty), rest fail
        assert_eq!(result.checks.len(), 5);
        assert!((result.score - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_sc01_codeowners() {
        let tmp = TempDir::new().unwrap();
        let gh_dir = tmp.path().join(".github");
        fs::create_dir_all(&gh_dir).unwrap();
        fs::write(gh_dir.join("CODEOWNERS"), "* @org/team").unwrap();
        let check = check_branch_protection(tmp.path(), &[]);
        assert!(check.passed);
    }

    #[test]
    fn test_sc01_pr_trigger() {
        let tmp = TempDir::new().unwrap();
        let workflows = vec![(
            "ci.yml".to_string(),
            "on:\n  pull_request:\n    branches: [main]".to_string(),
        )];
        let check = check_branch_protection(tmp.path(), &workflows);
        assert!(check.passed);
    }

    #[test]
    fn test_sc01_no_protection() {
        let tmp = TempDir::new().unwrap();
        let check = check_branch_protection(tmp.path(), &[]);
        assert!(!check.passed);
    }

    #[test]
    fn test_sc02_clean() {
        let check =
            check_no_hardcoded_secrets("run: cargo test\nenv:\n  TOKEN: ${{ secrets.MY_TOKEN }}");
        assert!(check.passed);
    }

    #[test]
    fn test_sc02_aws_key() {
        let check = check_no_hardcoded_secrets("env:\n  AWS_KEY: AKIAIOSFODNN7EXAMPLE");
        assert!(!check.passed);
    }

    #[test]
    fn test_sc02_github_token() {
        let check =
            check_no_hardcoded_secrets("env:\n  TOKEN: ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
        assert!(!check.passed);
    }

    #[test]
    fn test_sc02_secrets_ref_ok() {
        // Using secrets.* reference should not trigger
        let check = check_no_hardcoded_secrets("env:\n  TOKEN: ${{ secrets.GITHUB_TOKEN }}");
        assert!(check.passed);
    }

    #[test]
    fn test_sc03_cargo_deny() {
        let tmp = TempDir::new().unwrap();
        let check = check_dependency_review("run: cargo deny check", tmp.path());
        assert!(check.passed);
    }

    #[test]
    fn test_sc03_dependabot_file() {
        let tmp = TempDir::new().unwrap();
        let gh_dir = tmp.path().join(".github");
        fs::create_dir_all(&gh_dir).unwrap();
        fs::write(gh_dir.join("dependabot.yml"), "version: 2").unwrap();
        let check = check_dependency_review("", tmp.path());
        assert!(check.passed);
    }

    #[test]
    fn test_sc03_deny_toml() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("deny.toml"), "[advisories]").unwrap();
        let check = check_dependency_review("", tmp.path());
        assert!(check.passed);
    }

    #[test]
    fn test_sc03_no_dep_review() {
        let tmp = TempDir::new().unwrap();
        let check = check_dependency_review("run: cargo build", tmp.path());
        assert!(!check.passed);
    }

    #[test]
    fn test_sc04_slsa() {
        let check = check_provenance("- uses: slsa-framework/slsa-github-generator@v2");
        assert!(check.passed);
    }

    #[test]
    fn test_sc04_sigstore() {
        let check = check_provenance("- uses: sigstore/cosign-installer@v3");
        assert!(check.passed);
    }

    #[test]
    fn test_sc04_no_provenance() {
        let check = check_provenance("run: cargo build");
        assert!(!check.passed);
    }

    #[test]
    fn test_sc05_gitsigners() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join(".gitsigners"), "key-id").unwrap();
        let check = check_signed_commits(tmp.path(), "");
        assert!(check.passed);
    }

    #[test]
    fn test_sc05_gpg_reference() {
        let tmp = TempDir::new().unwrap();
        let check = check_signed_commits(tmp.path(), "run: gpg --verify");
        assert!(check.passed);
    }

    #[test]
    fn test_sc05_no_signing() {
        let tmp = TempDir::new().unwrap();
        let check = check_signed_commits(tmp.path(), "run: cargo build");
        assert!(!check.passed);
    }

    #[tokio::test]
    async fn test_perfect_supply_chain() {
        let tmp = TempDir::new().unwrap();
        let gh_dir = tmp.path().join(".github");
        let wf_dir = gh_dir.join("workflows");
        fs::create_dir_all(&wf_dir).unwrap();
        fs::write(gh_dir.join("CODEOWNERS"), "* @org/team").unwrap();
        fs::write(gh_dir.join("dependabot.yml"), "version: 2").unwrap();
        fs::write(
            wf_dir.join("ci.yml"),
            r#"name: CI
on:
  pull_request:
    branches: [main]
jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: sigstore/cosign-installer@v3
      - run: gpg --verify signed-artifact
"#,
        )
        .unwrap();

        let scorer = SupplyChainScorer::new();
        let result = scorer.score(tmp.path()).await.unwrap();
        assert!((result.score - 15.0).abs() < f64::EPSILON);
    }
}

const DANGEROUS_PATTERNS: &[&str] = &[
    "github.event.pull_request.title",
    "github.event.pull_request.body",
    "github.event.pull_request.head_ref",
    "github.event.issue.title",
    "github.event.issue.body",
    "github.event.comment.body",
    "github.event.review.body",
    "github.head_ref",
];

/// Returns true if this line starts a `run:` block.
fn is_run_block_start(trimmed: &str) -> bool {
    trimmed.starts_with("run:") || trimmed.starts_with("- run:")
}

/// Returns true if this line exits a `run:` block (a non-continuation, non-indented line).
fn is_run_block_end(trimmed: &str, raw: &str) -> bool {
    !trimmed.starts_with('-')
        && !trimmed.starts_with('#')
        && !raw.starts_with(' ')
        && !raw.starts_with('\t')
}

/// Collect dangerous pattern violations from a single workflow file.
fn collect_dangerous_violations(name: &str, content: &str, violations: &mut Vec<String>) {
    let mut in_run_block = false;
    for (line_no, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if is_run_block_start(trimmed) {
            in_run_block = true;
        } else if is_run_block_end(trimmed, line) {
            in_run_block = false;
        }
        if !in_run_block {
            continue;
        }
        for pattern in DANGEROUS_PATTERNS {
            if trimmed.contains(pattern) {
                violations.push(format!("{}:{}: ${{{{ {} }}}}", name, line_no + 1, pattern));
            }
        }
    }
}

/// HD-01: Check for untrusted context interpolation in run: blocks
fn check_dangerous_workflow(workflows: &[(String, String)]) -> InfraCheck {
    let mut violations = Vec::new();

    for (name, content) in workflows {
        collect_dangerous_violations(name, content, &mut violations);
    }

    if violations.is_empty() {
        InfraCheck::pass(
            "HD-01",
            "No dangerous workflow patterns",
            3.0,
            vec!["No untrusted context interpolation in run: blocks".to_string()],
        )
    } else {
        InfraCheck::fail("HD-01", "No dangerous workflow patterns", 3.0, violations)
    }
}
