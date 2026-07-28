// Agent iteration and evidence checks: CB-1405 through CB-1408
// Included from check.rs
//
// Spec: docs/specifications/components/agent-integration.md

/// CB-1405: Contract References Present
///
/// Checks that work contracts have research references (arXiv, spec sections,
/// or oracle links). Contracts without references lack traceability.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_agent_references_present(project_path: &Path) -> ComplianceCheck {
    let work_dir = project_path.join(".pmat-work");
    if !work_dir.exists() {
        return ComplianceCheck {
            name: "CB-1405: Contract References".into(),
            status: CheckStatus::Skip,
            message: "No .pmat-work/ directory found".into(),
            severity: Severity::Info,
        };
    }

    let mut total_contracts = 0usize;
    let mut contracts_with_refs = 0usize;

    if let Ok(entries) = fs::read_dir(&work_dir) {
        for entry in entries.flatten() {
            let contract_path = entry.path().join("contract.json");
            if !contract_path.exists() {
                continue;
            }
            total_contracts += 1;

            if let Ok(content) = fs::read_to_string(&contract_path) {
                let has_refs = content.contains("\"arxiv\"")
                    || content.contains("\"spec_sections\"")
                    || content.contains("\"five_whys_id\"")
                    || content.contains("\"references\"");
                if has_refs {
                    contracts_with_refs += 1;
                }
            }
        }
    }

    if total_contracts == 0 {
        return ComplianceCheck {
            name: "CB-1405: Contract References".into(),
            status: CheckStatus::Skip,
            message: "No work contracts found".into(),
            severity: Severity::Info,
        };
    }

    ComplianceCheck {
        name: "CB-1405: Contract References".into(),
        status: CheckStatus::Pass,
        message: format!(
            "{}/{} contract(s) have references (arXiv, spec, five-whys)",
            contracts_with_refs, total_contracts
        ),
        severity: Severity::Info,
    }
}

/// CB-1406: Chain-of-Thought Audit Trail
///
/// Checks that work contracts have chain-of-thought entries. The audit trail
/// documents decision history for future inspection.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_agent_chain_of_thought(project_path: &Path) -> ComplianceCheck {
    let work_dir = project_path.join(".pmat-work");
    if !work_dir.exists() {
        return ComplianceCheck {
            name: "CB-1406: Chain-of-Thought Audit".into(),
            status: CheckStatus::Skip,
            message: "No .pmat-work/ directory found".into(),
            severity: Severity::Info,
        };
    }

    let mut total_contracts = 0usize;
    let mut contracts_with_cot = 0usize;

    if let Ok(entries) = fs::read_dir(&work_dir) {
        for entry in entries.flatten() {
            let contract_path = entry.path().join("contract.json");
            if !contract_path.exists() {
                continue;
            }
            total_contracts += 1;

            if let Ok(content) = fs::read_to_string(&contract_path) {
                // chain_of_thought field with at least one entry
                let has_cot = content.contains("\"chain_of_thought\"")
                    && !content.contains("\"chain_of_thought\": []")
                    && !content.contains("\"chain_of_thought\":[]");
                if has_cot {
                    contracts_with_cot += 1;
                }
            }
        }
    }

    if total_contracts == 0 {
        return ComplianceCheck {
            name: "CB-1406: Chain-of-Thought Audit".into(),
            status: CheckStatus::Skip,
            message: "No work contracts found".into(),
            severity: Severity::Info,
        };
    }

    ComplianceCheck {
        name: "CB-1406: Chain-of-Thought Audit".into(),
        status: CheckStatus::Pass,
        message: format!(
            "{}/{} contract(s) have chain-of-thought audit trail",
            contracts_with_cot, total_contracts
        ),
        severity: Severity::Info,
    }
}

/// CB-1407: Five Whys Linked for Defects
///
/// For work contracts related to defects (status=blocked, or tagged as bug/fix),
/// checks that a five_whys_id reference exists in the contract references.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_agent_five_whys_linked(project_path: &Path) -> ComplianceCheck {
    let work_dir = project_path.join(".pmat-work");
    if !work_dir.exists() {
        return ComplianceCheck {
            name: "CB-1407: Five Whys Linked".into(),
            status: CheckStatus::Skip,
            message: "No .pmat-work/ directory found".into(),
            severity: Severity::Info,
        };
    }

    let mut defect_contracts = 0usize;
    let mut defects_with_whys = 0usize;

    if let Ok(entries) = fs::read_dir(&work_dir) {
        for entry in entries.flatten() {
            let contract_path = entry.path().join("contract.json");
            if !contract_path.exists() {
                continue;
            }

            if let Ok(content) = fs::read_to_string(&contract_path) {
                // Detect defect-related contracts
                let is_defect = content.contains("\"bug\"")
                    || content.contains("\"fix\"")
                    || content.contains("\"defect\"")
                    || content.contains("\"blocked\"");

                if is_defect {
                    defect_contracts += 1;
                    if content.contains("\"five_whys_id\"") {
                        defects_with_whys += 1;
                    }
                }
            }
        }
    }

    if defect_contracts == 0 {
        return ComplianceCheck {
            name: "CB-1407: Five Whys Linked".into(),
            status: CheckStatus::Pass,
            message: "No defect-related contracts found (check N/A)".into(),
            severity: Severity::Info,
        };
    }

    if defects_with_whys == defect_contracts {
        ComplianceCheck {
            name: "CB-1407: Five Whys Linked".into(),
            status: CheckStatus::Pass,
            message: format!(
                "{}/{} defect contract(s) have five_whys_id reference",
                defects_with_whys, defect_contracts
            ),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-1407: Five Whys Linked".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{}/{} defect contract(s) lack five_whys_id — use pmat five-whys for root cause",
                defect_contracts - defects_with_whys, defect_contracts
            ),
            severity: Severity::Warning,
        }
    }
}

/// CB-1408: Agent Evidence Executability
///
/// Checks that work contracts have valid evidence mechanisms — either
/// "evidence": "command" strings (DbC v5.0 spec format) or
/// "falsification_method": "MethodName" (Popperian claims format).
/// Contracts with placeholder-only evidence are flagged.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_agent_evidence_executable(project_path: &Path) -> ComplianceCheck {
    let work_dir = project_path.join(".pmat-work");
    if !work_dir.exists() {
        return ComplianceCheck {
            name: "CB-1408: Agent Evidence Executable".into(),
            status: CheckStatus::Skip,
            message: "No .pmat-work/ directory found".into(),
            severity: Severity::Info,
        };
    }

    let mut total_contracts = 0usize;
    let mut contracts_with_evidence = 0usize;
    let mut contracts_without_evidence = Vec::new();

    // Known valid falsification methods (Popperian claims format)
    let valid_methods = [
        "ManifestIntegrity", "MetaFalsification", "CoverageGaming",
        "DifferentialCoverage", "AbsoluteCoverage", "TdgRegression",
        "ComplexityRegression", "FileSizeRegression", "SpecQuality",
        "GitHubSync", "SupplyChainIntegrity", "SatdRegression",
        "DeadCodeRegression", "UnwrapRegression",
    ];

    // Known placeholder patterns
    let placeholder_patterns = ["TODO", "FIXME", "placeholder", "TBD"];

    if let Ok(entries) = fs::read_dir(&work_dir) {
        for entry in entries.flatten() {
            let contract_path = entry.path().join("contract.json");
            if !contract_path.exists() {
                continue;
            }
            total_contracts += 1;

            if let Ok(content) = fs::read_to_string(&contract_path) {
                let id = entry.file_name().to_string_lossy().to_string();

                // Check for Popperian falsification methods (named variants)
                let has_methods = valid_methods
                    .iter()
                    .any(|m| content.contains(m));

                // Check for falsification_method fields (v5.0 arbitrary commands/strings)
                let has_falsification_method = content.contains("\"falsification_method\":")
                    && !content.contains("\"falsification_method\": \"\"");

                // Check for evidence command strings
                let has_evidence_cmds = content.contains("\"evidence\":")
                    && !content.contains("\"evidence\": \"\"");

                // Any of these counts as executable evidence
                let has_methods = has_methods || has_falsification_method;

                // Check for placeholder-dominated evidence
                let is_placeholder = has_evidence_cmds
                    && !has_methods
                    && content
                        .lines()
                        .filter(|l| l.contains("\"evidence\""))
                        .all(|l| placeholder_patterns.iter().any(|pp| l.contains(pp)));

                if (has_methods || has_evidence_cmds) && !is_placeholder {
                    contracts_with_evidence += 1;
                } else if contracts_without_evidence.len() < 5 {
                    contracts_without_evidence.push(id);
                }
            }
        }
    }

    if total_contracts == 0 {
        return ComplianceCheck {
            name: "CB-1408: Agent Evidence Executable".into(),
            status: CheckStatus::Skip,
            message: "No work contracts found".into(),
            severity: Severity::Info,
        };
    }

    if contracts_without_evidence.is_empty() {
        ComplianceCheck {
            name: "CB-1408: Agent Evidence Executable".into(),
            status: CheckStatus::Pass,
            message: format!(
                "{}/{} contract(s) have valid evidence mechanisms (falsification methods or commands)",
                contracts_with_evidence, total_contracts
            ),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-1408: Agent Evidence Executable".into(),
            status: CheckStatus::Fail,
            message: format!(
                "{} contract(s) lack valid evidence: {}",
                contracts_without_evidence.len(),
                contracts_without_evidence.join(", ")
            ),
            severity: Severity::Error,
        }
    }
}

#[cfg(test)]
mod tests_agent_iteration {
    use super::*;

    // --- CB-1408 tests ---

    #[test]
    fn test_cb1408_skip_no_work_dir() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let result = check_agent_evidence_executable(tmp.path());
        assert_eq!(result.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1408_pass_with_falsification_methods() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let work_dir = tmp.path().join(".pmat-work").join("PMAT-006");
        std::fs::create_dir_all(&work_dir).unwrap();
        std::fs::write(
            work_dir.join("contract.json"),
            r#"{"claims": [{"falsification_method": "ManifestIntegrity"},
                           {"falsification_method": "AbsoluteCoverage"}]}"#,
        )
        .unwrap();
        let result = check_agent_evidence_executable(tmp.path());
        assert_eq!(result.status, CheckStatus::Pass);
        assert!(result.message.contains("1/1"));
    }

    #[test]
    fn test_cb1408_pass_with_evidence_commands() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let work_dir = tmp.path().join(".pmat-work").join("PMAT-006b");
        std::fs::create_dir_all(&work_dir).unwrap();
        std::fs::write(
            work_dir.join("contract.json"),
            r#"{"require": [{"description": "builds", "evidence": "cargo build"}]}"#,
        )
        .unwrap();
        let result = check_agent_evidence_executable(tmp.path());
        assert_eq!(result.status, CheckStatus::Pass);
    }

    #[test]
    fn test_cb1408_fail_no_evidence_or_methods() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let work_dir = tmp.path().join(".pmat-work").join("PMAT-007");
        std::fs::create_dir_all(&work_dir).unwrap();
        std::fs::write(
            work_dir.join("contract.json"),
            r#"{"title": "No evidence at all", "status": "in_progress"}"#,
        )
        .unwrap();
        let result = check_agent_evidence_executable(tmp.path());
        assert_eq!(result.status, CheckStatus::Fail);
        assert!(result.message.contains("lack valid evidence"));
    }

    // --- CB-1405 tests ---

    #[test]
    fn test_cb1405_skip_no_work_dir() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let result = check_agent_references_present(tmp.path());
        assert_eq!(result.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1405_pass_with_references() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let work_dir = tmp.path().join(".pmat-work").join("PMAT-012");
        std::fs::create_dir_all(&work_dir).unwrap();
        std::fs::write(
            work_dir.join("contract.json"),
            r#"{"references": {"arxiv": "2602.22302", "spec_sections": ["§3"]}}"#,
        )
        .unwrap();
        let result = check_agent_references_present(tmp.path());
        assert_eq!(result.status, CheckStatus::Pass);
        assert!(result.message.contains("1/1"));
    }

    // --- CB-1406 tests ---

    #[test]
    fn test_cb1406_skip_no_work_dir() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let result = check_agent_chain_of_thought(tmp.path());
        assert_eq!(result.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1406_pass_with_cot() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let work_dir = tmp.path().join(".pmat-work").join("PMAT-013");
        std::fs::create_dir_all(&work_dir).unwrap();
        std::fs::write(
            work_dir.join("contract.json"),
            r#"{"chain_of_thought": [{"step": "Analyzed root cause", "timestamp": "2026-04-03"}]}"#,
        )
        .unwrap();
        let result = check_agent_chain_of_thought(tmp.path());
        assert_eq!(result.status, CheckStatus::Pass);
        assert!(result.message.contains("1/1"));
    }

    // --- CB-1407 tests ---

    #[test]
    fn test_cb1407_pass_no_defects() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let work_dir = tmp.path().join(".pmat-work").join("PMAT-014");
        std::fs::create_dir_all(&work_dir).unwrap();
        std::fs::write(
            work_dir.join("contract.json"),
            r#"{"title": "New feature", "type": "feature"}"#,
        )
        .unwrap();
        let result = check_agent_five_whys_linked(tmp.path());
        assert_eq!(result.status, CheckStatus::Pass);
    }

    #[test]
    fn test_cb1407_warn_defect_without_whys() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let work_dir = tmp.path().join(".pmat-work").join("PMAT-015");
        std::fs::create_dir_all(&work_dir).unwrap();
        std::fs::write(
            work_dir.join("contract.json"),
            r#"{"title": "fix bug", "type": "fix"}"#,
        )
        .unwrap();
        let result = check_agent_five_whys_linked(tmp.path());
        assert_eq!(result.status, CheckStatus::Warn);
    }

    #[test]
    fn test_cb1407_pass_defect_with_whys() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let work_dir = tmp.path().join(".pmat-work").join("PMAT-016");
        std::fs::create_dir_all(&work_dir).unwrap();
        std::fs::write(
            work_dir.join("contract.json"),
            r#"{"title": "fix bug", "type": "fix",
                "references": {"five_whys_id": "5W-2026-001"}}"#,
        )
        .unwrap();
        let result = check_agent_five_whys_linked(tmp.path());
        assert_eq!(result.status, CheckStatus::Pass);
    }

}
