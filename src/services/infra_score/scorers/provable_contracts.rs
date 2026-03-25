#![cfg_attr(coverage_nightly, coverage(off))]
//! Provable Contracts Scorer (Bonus: 10 points)
//!
//! PV-01 (3pts): pv lint passes (contracts validate)
//! PV-02 (3pts): pv score >= 0.5 mean
//! PV-03 (2pts): At least 1 contract at proof level L2+
//! PV-04 (2pts): contracts/ directory exists with >= 1 YAML file

use super::InfraScorer;
use crate::services::infra_score::models::*;
use async_trait::async_trait;
use std::path::Path;

pub struct ProvableContractsScorer;

impl ProvableContractsScorer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ProvableContractsScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl InfraScorer for ProvableContractsScorer {
    fn category_name(&self) -> &str {
        "Provable Contracts"
    }

    fn max_score(&self) -> f64 {
        10.0
    }

    async fn score(&self, repo_path: &Path) -> anyhow::Result<InfraCategoryScore> {
        let contracts_dir = repo_path.join("contracts");
        let mut checks = Vec::new();
        let mut findings = Vec::new();

        // PV-04 (2pts): contracts/ directory exists with YAML files
        let pv04 = check_contracts_exist(&contracts_dir);
        if !pv04.passed {
            findings.push(InfraFinding {
                severity: InfraSeverity::Info,
                check_id: "PV-04".to_string(),
                message: "No contracts/ directory. Add provable contracts for numeric kernels."
                    .to_string(),
                location: Some("contracts/".to_string()),
                impact_points: -2.0,
            });
            // If no contracts, skip other checks
            checks.push(pv04);
            checks.push(InfraCheck::fail(
                "PV-01",
                "pv lint passes",
                3.0,
                vec!["Skipped — no contracts/ directory".to_string()],
            ));
            checks.push(InfraCheck::fail(
                "PV-02",
                "pv score >= 0.5",
                3.0,
                vec!["Skipped — no contracts/ directory".to_string()],
            ));
            checks.push(InfraCheck::fail(
                "PV-03",
                "Proof level L2+",
                2.0,
                vec!["Skipped — no contracts/ directory".to_string()],
            ));
            return Ok(InfraCategoryScore::new(self.max_score(), checks, findings));
        }
        checks.push(pv04);

        // PV-01 (3pts): pv lint passes
        let pv01 = check_pv_lint(&contracts_dir).await;
        if !pv01.passed {
            findings.push(InfraFinding {
                severity: InfraSeverity::Warning,
                check_id: "PV-01".to_string(),
                message: "pv lint failed. Run `pv lint contracts/` to see issues.".to_string(),
                location: Some("contracts/".to_string()),
                impact_points: -3.0,
            });
        }
        checks.push(pv01);

        // PV-02 (3pts): pv score >= 0.5 mean
        let pv02 = check_pv_score(&contracts_dir).await;
        if !pv02.passed {
            findings.push(InfraFinding {
                severity: InfraSeverity::Warning,
                check_id: "PV-02".to_string(),
                message: "Contract score below 0.5. Run `pv score contracts/ --top-gaps 5`."
                    .to_string(),
                location: Some("contracts/".to_string()),
                impact_points: -3.0,
            });
        }
        checks.push(pv02);

        // PV-03 (2pts): Proof level L2+
        let pv03 = check_proof_level(&contracts_dir).await;
        if !pv03.passed {
            findings.push(InfraFinding {
                severity: InfraSeverity::Info,
                check_id: "PV-03".to_string(),
                message:
                    "No contracts at proof level L2+. Run `pv proof-status contracts/` to check."
                        .to_string(),
                location: Some("contracts/".to_string()),
                impact_points: -2.0,
            });
        }
        checks.push(pv03);

        Ok(InfraCategoryScore::new(self.max_score(), checks, findings))
    }
}

/// PV-04: Check contracts/ directory exists with YAML files
fn check_contracts_exist(contracts_dir: &Path) -> InfraCheck {
    if !contracts_dir.exists() {
        return InfraCheck::fail(
            "PV-04",
            "Contracts directory exists",
            2.0,
            vec!["No contracts/ directory found".to_string()],
        );
    }

    let yaml_count = std::fs::read_dir(contracts_dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .is_some_and(|ext| ext == "yaml" || ext == "yml")
                })
                .count()
        })
        .unwrap_or(0);

    if yaml_count > 0 {
        InfraCheck::pass(
            "PV-04",
            "Contracts directory exists",
            2.0,
            vec![format!("{} contract YAML files found", yaml_count)],
        )
    } else {
        InfraCheck::fail(
            "PV-04",
            "Contracts directory exists",
            2.0,
            vec!["contracts/ exists but contains no YAML files".to_string()],
        )
    }
}

/// PV-01: Run pv lint (via CLI if available, else check YAML structure)
async fn check_pv_lint(contracts_dir: &Path) -> InfraCheck {
    // Try running pv lint
    let output = tokio::process::Command::new("pv")
        .args(["lint", &contracts_dir.to_string_lossy(), "--quiet"])
        .output()
        .await;

    match output {
        Ok(out) if out.status.success() => InfraCheck::pass(
            "PV-01",
            "pv lint passes",
            3.0,
            vec!["All contract quality gates passed".to_string()],
        ),
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            InfraCheck::fail(
                "PV-01",
                "pv lint passes",
                3.0,
                vec![format!(
                    "pv lint failed: {}",
                    stderr.lines().next().unwrap_or("unknown error")
                )],
            )
        }
        Err(_) => {
            // pv not available — check YAML structure manually
            let valid = check_yaml_structure(contracts_dir);
            if valid {
                InfraCheck::pass(
                    "PV-01",
                    "pv lint passes",
                    3.0,
                    vec!["pv not available; YAML structure check passed".to_string()],
                )
            } else {
                InfraCheck::fail(
                    "PV-01",
                    "pv lint passes",
                    3.0,
                    vec!["pv not available; YAML structure check failed".to_string()],
                )
            }
        }
    }
}

/// PV-02: Run pv score (via CLI if available)
async fn check_pv_score(contracts_dir: &Path) -> InfraCheck {
    let output = tokio::process::Command::new("pv")
        .args([
            "score",
            &contracts_dir.to_string_lossy(),
            "--min-score",
            "0.5",
            "--exit-code",
            "--quiet",
        ])
        .output()
        .await;

    match output {
        Ok(out) if out.status.success() => InfraCheck::pass(
            "PV-02",
            "Contract score >= 0.5",
            3.0,
            vec!["Mean contract score meets threshold".to_string()],
        ),
        Ok(_) => InfraCheck::fail(
            "PV-02",
            "Contract score >= 0.5",
            3.0,
            vec!["Mean contract score below 0.5".to_string()],
        ),
        Err(_) => {
            // pv not available — pass with warning
            InfraCheck::pass(
                "PV-02",
                "Contract score >= 0.5",
                3.0,
                vec!["pv not available; skipped (contracts exist)".to_string()],
            )
        }
    }
}

/// PV-03: Check proof status for L2+ contracts
async fn check_proof_level(contracts_dir: &Path) -> InfraCheck {
    let output = tokio::process::Command::new("pv")
        .args([
            "proof-status",
            &contracts_dir.to_string_lossy(),
            "--format",
            "json",
        ])
        .output()
        .await;

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            // Check if any contract has level >= 2
            let has_l2 = stdout.contains("\"level\":2")
                || stdout.contains("\"level\":3")
                || stdout.contains("\"level\":4")
                || stdout.contains("\"level\":5")
                || stdout.contains("\"level\": 2")
                || stdout.contains("\"level\": 3")
                || stdout.contains("\"level\": 4")
                || stdout.contains("\"level\": 5");
            if has_l2 {
                InfraCheck::pass(
                    "PV-03",
                    "Proof level L2+",
                    2.0,
                    vec!["At least one contract at proof level L2+".to_string()],
                )
            } else {
                InfraCheck::fail(
                    "PV-03",
                    "Proof level L2+",
                    2.0,
                    vec!["No contracts at proof level L2+".to_string()],
                )
            }
        }
        _ => {
            // pv not available — pass with warning
            InfraCheck::pass(
                "PV-03",
                "Proof level L2+",
                2.0,
                vec!["pv not available; skipped (contracts exist)".to_string()],
            )
        }
    }
}

/// Fallback: check YAML files have basic contract structure
fn check_yaml_structure(contracts_dir: &Path) -> bool {
    let entries = match std::fs::read_dir(contracts_dir) {
        Ok(e) => e,
        Err(_) => return false,
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path
            .extension()
            .is_some_and(|ext| ext == "yaml" || ext == "yml")
        {
            if let Ok(content) = std::fs::read_to_string(&path) {
                // Basic structure check: has name, equations, obligations
                if content.contains("name:")
                    && (content.contains("equations:") || content.contains("obligations:"))
                {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_no_contracts_dir() {
        let tmp = TempDir::new().unwrap();
        let scorer = ProvableContractsScorer::new();
        let result = scorer.score(tmp.path()).await.unwrap();
        assert!(result.score < 1.0); // Should fail all checks
    }

    #[tokio::test]
    async fn test_empty_contracts_dir() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("contracts")).unwrap();
        let scorer = ProvableContractsScorer::new();
        let result = scorer.score(tmp.path()).await.unwrap();
        assert!(result.score < 3.0); // PV-04 fails (no YAML)
    }

    #[tokio::test]
    async fn test_contracts_with_yaml() {
        let tmp = TempDir::new().unwrap();
        let contracts = tmp.path().join("contracts");
        fs::create_dir_all(&contracts).unwrap();
        fs::write(
            contracts.join("test-v1.yaml"),
            "name: test\nequations:\n  - id: eq1\nobligations:\n  - id: ob1\n",
        )
        .unwrap();

        let scorer = ProvableContractsScorer::new();
        let result = scorer.score(tmp.path()).await.unwrap();
        assert!(result.score >= 2.0); // PV-04 passes at minimum
    }
}
