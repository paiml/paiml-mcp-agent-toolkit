#![cfg_attr(coverage_nightly, coverage(off))]
//! Provable Contracts Scorer (Bonus: 12 points)
//!
//! PV-01 (3pts): pv lint passes (contracts validate)
//! PV-02 (3pts): pv score >= 0.5 mean
//! PV-03 (2pts): At least 1 contract at proof level L2+
//! PV-04 (2pts): contracts/ directory exists with >= 1 YAML file
//! PV-05 (2pts): Enforcement quality — contract call sites found in source

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
        12.0
    }

    async fn score(&self, repo_path: &Path) -> anyhow::Result<InfraCategoryScore> {
        debug_assert!(
            repo_path.exists(),
            "repo_path must exist: {}",
            repo_path.display()
        );
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
            checks.push(InfraCheck::fail(
                "PV-05",
                "Enforcement quality",
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

        // PV-05 (2pts): Enforcement quality — contract call sites in source
        let pv05 = check_enforcement(repo_path).await;
        if !pv05.passed {
            findings.push(InfraFinding {
                severity: InfraSeverity::Info,
                check_id: "PV-05".to_string(),
                message: "No contract call sites in source. Add contract_pre_*/contract_post_* macro invocations.".to_string(),
                location: Some("src/".to_string()),
                impact_points: -2.0,
            });
        }
        checks.push(pv05);

        Ok(InfraCategoryScore::new(self.max_score(), checks, findings))
    }
}

/// PV-04: Check contracts/ directory exists with YAML files (recursive)
fn check_contracts_exist(contracts_dir: &Path) -> InfraCheck {
    debug_assert!(
        contracts_dir.exists(),
        "contracts_dir must exist: {}",
        contracts_dir.display()
    );
    if !contracts_dir.exists() {
        return InfraCheck::fail(
            "PV-04",
            "Contracts directory exists",
            2.0,
            vec!["No contracts/ directory found".to_string()],
        );
    }

    let yaml_count = count_yaml_files_recursive(contracts_dir);

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

/// Check whether a path has a YAML extension (.yaml or .yml).
fn is_yaml_file(path: &Path) -> bool {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    path.extension()
        .is_some_and(|ext| ext == "yaml" || ext == "yml")
}

/// Check whether a filename indicates a binding file (excluded from contract counts).
fn is_binding_file(path: &Path) -> bool {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    path.file_name()
        .is_some_and(|n| n.to_string_lossy().contains("binding"))
}

/// Check whether file content contains provable-contracts schema markers.
fn has_contract_markers(content: &str) -> bool {
    const MARKERS: &[&str] = &[
        "proof_obligations",
        "equations:",
        "falsification_tests",
        "kani_harnesses",
    ];
    MARKERS.iter().any(|m| content.contains(m))
}

/// Check whether a single file is a valid provable-contract YAML.
fn is_provable_contract_file(path: &Path) -> bool {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    if !is_yaml_file(path) || is_binding_file(path) {
        return false;
    }
    std::fs::read_to_string(path)
        .map(|content| has_contract_markers(&content))
        .unwrap_or(false)
}

/// Recursively count provable-contract YAML files in a directory tree.
/// Excludes binding files and non-contract YAMLs (matching CB-1200 logic).
fn count_yaml_files_recursive(dir: &Path) -> usize {
    debug_assert!(dir.exists(), "dir must exist: {}", dir.display());
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return 0,
    };

    entries
        .filter_map(|e| e.ok())
        .map(|entry| {
            let path = entry.path();
            if path.is_dir() {
                count_yaml_files_recursive(&path)
            } else if is_provable_contract_file(&path) {
                1
            } else {
                0
            }
        })
        .sum()
}

/// PV-01: Run pv lint (via CLI if available, else check YAML structure)
async fn check_pv_lint(contracts_dir: &Path) -> InfraCheck {
    debug_assert!(
        contracts_dir.exists(),
        "contracts_dir must exist: {}",
        contracts_dir.display()
    );
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
    debug_assert!(
        contracts_dir.exists(),
        "contracts_dir must exist: {}",
        contracts_dir.display()
    );
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
            // pv not available — cannot verify score, fail with guidance
            InfraCheck::fail(
                "PV-02",
                "Contract score >= 0.5",
                3.0,
                vec![
                    "pv CLI not installed; install: cargo install provable-contracts-cli"
                        .to_string(),
                ],
            )
        }
    }
}

/// PV-03: Check proof status for L2+ contracts
async fn check_proof_level(contracts_dir: &Path) -> InfraCheck {
    debug_assert!(
        contracts_dir.exists(),
        "contracts_dir must exist: {}",
        contracts_dir.display()
    );
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
            // Check if any contract has proof_level >= L2
            // pv outputs: "proof_level": "L3" (string format)
            let has_l2 = stdout.contains("\"proof_level\": \"L2\"")
                || stdout.contains("\"proof_level\": \"L3\"")
                || stdout.contains("\"proof_level\": \"L4\"")
                || stdout.contains("\"proof_level\": \"L5\"")
                || stdout.contains("\"proof_level\":\"L2\"")
                || stdout.contains("\"proof_level\":\"L3\"")
                || stdout.contains("\"proof_level\":\"L4\"")
                || stdout.contains("\"proof_level\":\"L5\"");
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
            // pv not available — cannot verify proof level, fail with guidance
            InfraCheck::fail(
                "PV-03",
                "Proof level L2+",
                2.0,
                vec![
                    "pv CLI not installed; install: cargo install provable-contracts-cli"
                        .to_string(),
                ],
            )
        }
    }
}

/// PV-05: Check enforcement quality via `pv coverage --enforcement`
async fn check_enforcement(repo_path: &Path) -> InfraCheck {
    debug_assert!(
        repo_path.exists(),
        "repo_path must exist: {}",
        repo_path.display()
    );
    // Find binding.yaml
    let canonical = std::fs::canonicalize(repo_path).unwrap_or_else(|_| repo_path.to_path_buf());
    let project_name = canonical.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let binding = canonical
        .parent()
        .map(|p| {
            p.join("provable-contracts")
                .join("contracts")
                .join(project_name)
                .join("binding.yaml")
        })
        .filter(|p| p.exists());

    let binding_path = match binding {
        Some(p) => p,
        None => {
            return InfraCheck::fail(
                "PV-05",
                "Enforcement quality",
                2.0,
                vec!["No binding.yaml found".to_string()],
            );
        }
    };

    let output = tokio::process::Command::new("pv")
        .args([
            "coverage",
            "--enforcement",
            ".",
            "--binding",
            &binding_path.to_string_lossy(),
        ])
        .current_dir(repo_path)
        .output()
        .await;

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let combined = format!("{stdout}{stderr}");

            // Parse E0/E1/E2 counts
            let e0 = parse_enforcement_metric(&combined, "E0");
            let e1 = parse_enforcement_metric(&combined, "E1");
            let e2 = parse_enforcement_metric(&combined, "E2");
            let total = e0 + e1 + e2;

            if total > 0 {
                InfraCheck::pass(
                    "PV-05",
                    "Enforcement quality",
                    2.0,
                    vec![format!("{total} call sites (E0={e0}, E1={e1}, E2={e2})")],
                )
            } else {
                InfraCheck::fail(
                    "PV-05",
                    "Enforcement quality",
                    2.0,
                    vec!["0 contract call sites in source".to_string()],
                )
            }
        }
        _ => InfraCheck::fail(
            "PV-05",
            "Enforcement quality",
            2.0,
            vec!["pv CLI not available".to_string()],
        ),
    }
}

/// Parse E-level count from pv coverage output
fn parse_enforcement_metric(output: &str, level: &str) -> usize {
    output
        .lines()
        .find(|l| l.contains(&format!("{level} (")))
        .and_then(|l| l.split(':').next_back())
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(0)
}

/// Check whether a single YAML file has basic contract structure (name + equations/obligations).
fn has_basic_contract_structure(path: &Path) -> bool {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    if !is_yaml_file(path) {
        return false;
    }
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    content.contains("name:")
        && (content.contains("equations:") || content.contains("obligations:"))
}

/// Fallback: check YAML files have basic contract structure
fn check_yaml_structure(contracts_dir: &Path) -> bool {
    debug_assert!(
        contracts_dir.exists(),
        "contracts_dir must exist: {}",
        contracts_dir.display()
    );
    let entries = match std::fs::read_dir(contracts_dir) {
        Ok(e) => e,
        Err(_) => return false,
    };

    entries
        .filter_map(|e| e.ok())
        .any(|entry| has_basic_contract_structure(&entry.path()))
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
