#![cfg_attr(coverage_nightly, coverage(off))]
//! Quality Pipeline Scorer (20 points)
//!
//! QP-01 (5pts): Test job (cargo test or equivalent)
//! QP-02 (5pts): Lint job (cargo clippy -- -D warnings)
//! QP-03 (4pts): Coverage (llvm-cov, codecov)
//! QP-04 (3pts): Security audit (cargo audit, CodeQL)
//! QP-05 (3pts): Format check (cargo fmt -- --check)

use super::{read_workflow_files, InfraScorer};
use crate::services::infra_score::models::*;
use async_trait::async_trait;
use std::path::Path;

pub struct QualityPipelineScorer;

impl QualityPipelineScorer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for QualityPipelineScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl InfraScorer for QualityPipelineScorer {
    fn category_name(&self) -> &str {
        "Quality Pipeline"
    }

    fn max_score(&self) -> f64 {
        20.0
    }

    async fn score(&self, repo_path: &Path) -> anyhow::Result<InfraCategoryScore> {
        let workflows = read_workflow_files(repo_path);
        let all_content: String = workflows
            .iter()
            .map(|(_, c)| c.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        // Check if sovereign-ci.yml is used — it provides test, lint, coverage, security, fmt
        let uses_sovereign_ci = all_content.contains("sovereign-ci");

        let mut checks = Vec::new();
        let mut findings = Vec::new();

        // QP-01 (5pts): Test job
        let qp01 = if uses_sovereign_ci {
            InfraCheck::pass("QP-01", "Test job", 5.0, vec!["Provided by sovereign-ci.yml".to_string()])
        } else {
            check_test_job(&all_content)
        };
        if !qp01.passed {
            findings.push(InfraFinding {
                severity: InfraSeverity::Fail,
                check_id: "QP-01".to_string(),
                message: "No test job found in CI workflows. Add `cargo test` or equivalent.".to_string(),
                location: Some(".github/workflows/".to_string()),
                impact_points: -5.0,
            });
        }
        checks.push(qp01);

        // QP-02 (5pts): Lint job
        let qp02 = if uses_sovereign_ci {
            InfraCheck::pass("QP-02", "Lint job", 5.0, vec!["Implied by sovereign-ci.yml (cargo clippy -D warnings)".to_string()])
        } else {
            check_lint_job(&all_content)
        };
        if !qp02.passed {
            findings.push(InfraFinding {
                severity: InfraSeverity::Fail,
                check_id: "QP-02".to_string(),
                message: "No lint job found. Add `cargo clippy -- -D warnings`.".to_string(),
                location: Some(".github/workflows/".to_string()),
                impact_points: -5.0,
            });
        }
        checks.push(qp02);

        // QP-03 (4pts): Coverage
        let qp03 = if uses_sovereign_ci {
            InfraCheck::pass("QP-03", "Coverage reporting", 4.0, vec!["Implied by sovereign-ci.yml (cargo llvm-cov)".to_string()])
        } else {
            check_coverage(&all_content)
        };
        if !qp03.passed {
            findings.push(InfraFinding {
                severity: InfraSeverity::Warning,
                check_id: "QP-03".to_string(),
                message: "No coverage reporting found. Add llvm-cov or codecov.".to_string(),
                location: Some(".github/workflows/".to_string()),
                impact_points: -4.0,
            });
        }
        checks.push(qp03);

        // QP-04 (3pts): Security audit
        let qp04 = check_security_audit(&all_content, repo_path);
        if !qp04.passed {
            findings.push(InfraFinding {
                severity: InfraSeverity::Warning,
                check_id: "QP-04".to_string(),
                message: "No security audit found. Add `cargo audit` or dependabot.".to_string(),
                location: Some(".github/workflows/".to_string()),
                impact_points: -3.0,
            });
        }
        checks.push(qp04);

        // QP-05 (3pts): Format check
        let qp05 = if uses_sovereign_ci {
            InfraCheck::pass("QP-05", "Format check", 3.0, vec!["Implied by sovereign-ci.yml (cargo fmt --check)".to_string()])
        } else {
            check_format(&all_content)
        };
        if !qp05.passed {
            findings.push(InfraFinding {
                severity: InfraSeverity::Warning,
                check_id: "QP-05".to_string(),
                message: "No format check found. Add `cargo fmt -- --check`.".to_string(),
                location: Some(".github/workflows/".to_string()),
                impact_points: -3.0,
            });
        }
        checks.push(qp05);

        Ok(InfraCategoryScore::new(self.max_score(), checks, findings))
    }
}

/// QP-01: Test job exists (cargo test, npm test, pytest, etc.)
fn check_test_job(content: &str) -> InfraCheck {
    let test_patterns = [
        "cargo test",
        "cargo nextest",
        "npm test",
        "pytest",
        "go test",
        "make test",
        "make check",
    ];

    for pattern in &test_patterns {
        if content.contains(pattern) {
            return InfraCheck::pass(
                "QP-01",
                "Test job",
                5.0,
                vec![format!("Found test command: {}", pattern)],
            );
        }
    }

    InfraCheck::fail(
        "QP-01",
        "Test job",
        5.0,
        vec!["No test command found in workflows".to_string()],
    )
}

/// QP-02: Lint job (cargo clippy -- -D warnings)
fn check_lint_job(content: &str) -> InfraCheck {
    let lint_patterns = [
        "cargo clippy",
        "clippy -- -D warnings",
        "clippy --all-targets",
        "eslint",
        "pylint",
        "golangci-lint",
        "make lint",
    ];

    for pattern in &lint_patterns {
        if content.contains(pattern) {
            // Extra credit: check for -D warnings (strict mode)
            let strict = content.contains("-D warnings") || content.contains("-Dwarnings");
            let evidence = if strict {
                format!("Found lint with -D warnings: {}", pattern)
            } else {
                format!("Found lint command: {} (consider adding -D warnings)", pattern)
            };
            return InfraCheck::pass("QP-02", "Lint job", 5.0, vec![evidence]);
        }
    }

    InfraCheck::fail(
        "QP-02",
        "Lint job",
        5.0,
        vec!["No lint command found in workflows".to_string()],
    )
}

/// QP-03: Coverage reporting (llvm-cov, codecov, coveralls)
fn check_coverage(content: &str) -> InfraCheck {
    let coverage_patterns = [
        "llvm-cov",
        "codecov",
        "coveralls",
        "coverage",
        "tarpaulin",
        "cargo-llvm-cov",
        "lcov",
    ];

    for pattern in &coverage_patterns {
        if content.contains(pattern) {
            return InfraCheck::pass(
                "QP-03",
                "Coverage reporting",
                4.0,
                vec![format!("Found coverage tool: {}", pattern)],
            );
        }
    }

    InfraCheck::fail(
        "QP-03",
        "Coverage reporting",
        4.0,
        vec!["No coverage reporting found in workflows".to_string()],
    )
}

/// QP-04: Security audit (cargo audit, dependabot, CodeQL)
fn check_security_audit(content: &str, repo_path: &Path) -> InfraCheck {
    let audit_patterns = [
        "cargo audit",
        "cargo-deny",
        "CodeQL",
        "codeql",
        "security-audit",
        "snyk",
        "trivy",
    ];

    for pattern in &audit_patterns {
        if content.contains(pattern) {
            return InfraCheck::pass(
                "QP-04",
                "Security audit",
                3.0,
                vec![format!("Found security audit: {}", pattern)],
            );
        }
    }

    // Check for dependabot.yml
    let dependabot_path = repo_path.join(".github/dependabot.yml");
    let dependabot_yaml_path = repo_path.join(".github/dependabot.yaml");
    if dependabot_path.exists() || dependabot_yaml_path.exists() {
        return InfraCheck::pass(
            "QP-04",
            "Security audit",
            3.0,
            vec!["Found dependabot configuration".to_string()],
        );
    }

    // Check for deny.toml
    let deny_path = repo_path.join("deny.toml");
    if deny_path.exists() {
        return InfraCheck::pass(
            "QP-04",
            "Security audit",
            3.0,
            vec!["Found deny.toml (cargo-deny)".to_string()],
        );
    }

    InfraCheck::fail(
        "QP-04",
        "Security audit",
        3.0,
        vec!["No security audit found".to_string()],
    )
}

/// QP-05: Format check (cargo fmt -- --check)
fn check_format(content: &str) -> InfraCheck {
    let fmt_patterns = [
        "cargo fmt",
        "rustfmt",
        "prettier",
        "black --check",
        "gofmt",
        "make fmt",
    ];

    for pattern in &fmt_patterns {
        if content.contains(pattern) {
            return InfraCheck::pass(
                "QP-05",
                "Format check",
                3.0,
                vec![format!("Found format check: {}", pattern)],
            );
        }
    }

    InfraCheck::fail(
        "QP-05",
        "Format check",
        3.0,
        vec!["No format check found in workflows".to_string()],
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
        let scorer = QualityPipelineScorer::new();
        let result = scorer.score(tmp.path()).await.unwrap();
        assert!((result.score - 0.0).abs() < f64::EPSILON);
        assert_eq!(result.checks.len(), 5);
    }

    #[test]
    fn test_qp01_cargo_test_pass() {
        let check = check_test_job("run: cargo test --all-targets");
        assert!(check.passed);
    }

    #[test]
    fn test_qp01_make_test_pass() {
        let check = check_test_job("run: make test");
        assert!(check.passed);
    }

    #[test]
    fn test_qp01_no_test() {
        let check = check_test_job("run: cargo build");
        assert!(!check.passed);
    }

    #[test]
    fn test_qp02_clippy_pass() {
        let check = check_lint_job("run: cargo clippy -- -D warnings");
        assert!(check.passed);
    }

    #[test]
    fn test_qp02_no_lint() {
        let check = check_lint_job("run: cargo build");
        assert!(!check.passed);
    }

    #[test]
    fn test_qp03_llvm_cov_pass() {
        let check = check_coverage("run: cargo llvm-cov --html");
        assert!(check.passed);
    }

    #[test]
    fn test_qp03_codecov_pass() {
        let check = check_coverage("- uses: codecov/codecov-action@v4");
        assert!(check.passed);
    }

    #[test]
    fn test_qp03_no_coverage() {
        let check = check_coverage("run: cargo test");
        assert!(!check.passed);
    }

    #[test]
    fn test_qp04_cargo_audit_pass() {
        let check = check_security_audit("run: cargo audit", Path::new("/nonexistent"));
        assert!(check.passed);
    }

    #[test]
    fn test_qp04_dependabot_pass() {
        let tmp = TempDir::new().unwrap();
        let gh_dir = tmp.path().join(".github");
        fs::create_dir_all(&gh_dir).unwrap();
        fs::write(gh_dir.join("dependabot.yml"), "version: 2").unwrap();
        let check = check_security_audit("", tmp.path());
        assert!(check.passed);
    }

    #[test]
    fn test_qp04_deny_toml_pass() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("deny.toml"), "[advisories]").unwrap();
        let check = check_security_audit("", tmp.path());
        assert!(check.passed);
    }

    #[test]
    fn test_qp04_no_audit() {
        let tmp = TempDir::new().unwrap();
        let check = check_security_audit("run: cargo build", tmp.path());
        assert!(!check.passed);
    }

    #[test]
    fn test_qp05_fmt_pass() {
        let check = check_format("run: cargo fmt -- --check");
        assert!(check.passed);
    }

    #[test]
    fn test_qp05_no_fmt() {
        let check = check_format("run: cargo build");
        assert!(!check.passed);
    }

    #[tokio::test]
    async fn test_perfect_quality_pipeline() {
        let tmp = TempDir::new().unwrap();
        let wf_dir = tmp.path().join(".github/workflows");
        fs::create_dir_all(&wf_dir).unwrap();
        let content = r#"name: CI
on: push
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - run: cargo test --all-targets
      - run: cargo clippy -- -D warnings
      - run: cargo llvm-cov --html
      - run: cargo audit
      - run: cargo fmt -- --check
"#;
        fs::write(wf_dir.join("ci.yml"), content).unwrap();

        let scorer = QualityPipelineScorer::new();
        let result = scorer.score(tmp.path()).await.unwrap();
        assert!((result.score - 20.0).abs() < f64::EPSILON);
    }
}
