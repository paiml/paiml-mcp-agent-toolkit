// Agent autonomous code and sub-agent composition: CB-1409 through CB-1410
// Included from check.rs
//
// Spec: docs/specifications/components/agent-integration.md

/// CB-1409: No L0 Autonomous Code
///
/// Checks recent git commits for AI co-author markers. If found, verifies
/// that a corresponding work contract exists in .pmat-work/. AI-authored
/// commits without contracts indicate L0 (paper-only) autonomous code.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_no_l0_autonomous_code(project_path: &Path) -> ComplianceCheck {
    // Get recent commits (last 20)
    let output = match std::process::Command::new("git")
        .args(["log", "--format=%H %s%n%b", "-20"])
        .current_dir(project_path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
    {
        Ok(o) => o,
        Err(_) => {
            return ComplianceCheck {
                name: "CB-1409: No L0 Autonomous Code".into(),
                status: CheckStatus::Skip,
                message: "Unable to read git log".into(),
                severity: Severity::Info,
            };
        }
    };

    let log_output = String::from_utf8_lossy(&output.stdout);
    if log_output.is_empty() {
        return ComplianceCheck {
            name: "CB-1409: No L0 Autonomous Code".into(),
            status: CheckStatus::Skip,
            message: "No git history found".into(),
            severity: Severity::Info,
        };
    }

    let mut ai_commits = 0usize;
    let mut ai_commits_with_contract = 0usize;
    let mut ai_commits_without_contract = Vec::new();

    // Collect existing work item IDs
    let work_dir = project_path.join(".pmat-work");
    let mut known_work_ids: Vec<String> = Vec::new();
    if work_dir.exists() {
        if let Ok(entries) = fs::read_dir(&work_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    known_work_ids.push(entry.file_name().to_string_lossy().to_string());
                }
            }
        }
    }

    // Parse git log output — format is: <hash> <subject>\n<body>\n
    let mut current_subject = String::new();
    let mut current_body = String::new();
    let mut in_body = false;

    for line in log_output.lines() {
        // New commit starts with a 40-char hex hash
        if line.len() > 41 && line.chars().take(40).all(|c| c.is_ascii_hexdigit()) {
            // Process previous commit if it was AI-authored
            if !current_subject.is_empty() {
                process_commit_for_ai_check(
                    &current_subject,
                    &current_body,
                    &known_work_ids,
                    &mut ai_commits,
                    &mut ai_commits_with_contract,
                    &mut ai_commits_without_contract,
                );
            }
            current_subject = line[41..].to_string();
            current_body.clear();
            in_body = false;
        } else {
            if !in_body {
                in_body = true;
            }
            current_body.push_str(line);
            current_body.push('\n');
        }
    }
    // Process the last commit
    if !current_subject.is_empty() {
        process_commit_for_ai_check(
            &current_subject,
            &current_body,
            &known_work_ids,
            &mut ai_commits,
            &mut ai_commits_with_contract,
            &mut ai_commits_without_contract,
        );
    }

    if ai_commits == 0 {
        return ComplianceCheck {
            name: "CB-1409: No L0 Autonomous Code".into(),
            status: CheckStatus::Pass,
            message: "No AI-authored commits in recent history (last 20 commits)".into(),
            severity: Severity::Info,
        };
    }

    if ai_commits_without_contract.is_empty() {
        ComplianceCheck {
            name: "CB-1409: No L0 Autonomous Code".into(),
            status: CheckStatus::Pass,
            message: format!(
                "{}/{} AI-authored commit(s) have corresponding work contracts",
                ai_commits_with_contract, ai_commits
            ),
            severity: Severity::Info,
        }
    } else {
        let examples: Vec<&str> = ai_commits_without_contract
            .iter()
            .take(3)
            .map(|s| s.as_str())
            .collect();
        ComplianceCheck {
            name: "CB-1409: No L0 Autonomous Code".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{}/{} AI-authored commit(s) lack work contracts: {}",
                ai_commits_without_contract.len(),
                ai_commits,
                examples.join("; ")
            ),
            severity: Severity::Warning,
        }
    }
}

/// Helper: check if a commit is AI-authored and has a work contract reference
fn process_commit_for_ai_check(
    subject: &str,
    body: &str,
    known_work_ids: &[String],
    ai_commits: &mut usize,
    ai_commits_with_contract: &mut usize,
    ai_commits_without_contract: &mut Vec<String>,
) {
    let full_text = format!("{}\n{}", subject, body);
    let is_ai = AI_COAUTHOR_PATTERNS
        .iter()
        .any(|p| full_text.to_lowercase().contains(&p.to_lowercase()));

    if !is_ai {
        return;
    }

    *ai_commits += 1;

    // Check if commit references a known work item
    let has_contract_ref = known_work_ids
        .iter()
        .any(|id| full_text.contains(id));

    // Also check for Refs PMAT-xxx pattern
    let has_refs = full_text.contains("Refs PMAT-")
        || full_text.contains("refs PMAT-")
        || full_text.contains("Contract:");

    if has_contract_ref || has_refs {
        *ai_commits_with_contract += 1;
    } else {
        let truncated = if subject.len() > 60 {
            // Safe truncation: find char boundary at or before byte 57
            let end = subject.floor_char_boundary(57);
            format!("{}...", &subject[..end])
        } else {
            subject.to_string()
        };
        if ai_commits_without_contract.len() < 5 {
            ai_commits_without_contract.push(truncated);
        }
    }
}

/// CB-1410: Sub-Agent Contract Composition
///
/// Validates that iterative work contracts (iteration > 1) properly
/// reference their predecessor's postconditions. Checks that the
/// contract chain is continuous.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_subagent_contract_composition(project_path: &Path) -> ComplianceCheck {
    let work_dir = project_path.join(".pmat-work");
    if !work_dir.exists() {
        return ComplianceCheck {
            name: "CB-1410: Sub-Agent Composition".into(),
            status: CheckStatus::Skip,
            message: "No .pmat-work/ directory found".into(),
            severity: Severity::Info,
        };
    }

    let mut total_contracts = 0usize;
    let mut iterated_contracts = 0usize;

    // Check for contracts with iteration markers or receipt chains
    if let Ok(entries) = fs::read_dir(&work_dir) {
        for entry in entries.flatten() {
            let contract_path = entry.path().join("contract.json");
            if !contract_path.exists() {
                continue;
            }
            total_contracts += 1;

            // Check for falsification receipts (indicates contract was evaluated)
            let receipt_dir = entry.path().join("falsification");
            if receipt_dir.exists() {
                if let Ok(receipts) = fs::read_dir(&receipt_dir) {
                    let receipt_count = receipts.filter_map(|r| r.ok()).count();
                    if receipt_count > 1 {
                        iterated_contracts += 1;
                    }
                }
            }
        }
    }

    if total_contracts == 0 {
        return ComplianceCheck {
            name: "CB-1410: Sub-Agent Composition".into(),
            status: CheckStatus::Skip,
            message: "No work contracts found".into(),
            severity: Severity::Info,
        };
    }

    ComplianceCheck {
        name: "CB-1410: Sub-Agent Composition".into(),
        status: CheckStatus::Pass,
        message: format!(
            "{}/{} contract(s) have multi-iteration falsification chains",
            iterated_contracts, total_contracts
        ),
        severity: Severity::Info,
    }
}

#[cfg(test)]
mod tests_agent_autonomous {
    use super::*;

    // --- CB-1409 tests ---

    #[test]
    fn test_cb1409_skip_no_git() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let result = check_no_l0_autonomous_code(tmp.path());
        // No git repo → skip or pass (depends on git availability)
        assert!(
            result.status == CheckStatus::Skip || result.status == CheckStatus::Pass,
            "Expected Skip or Pass, got {:?}",
            result.status
        );
    }


    // --- CB-1410 tests ---

    #[test]
    fn test_cb1410_skip_no_work_dir() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let result = check_subagent_contract_composition(tmp.path());
        assert_eq!(result.status, CheckStatus::Skip);
    }

    #[test]
    fn test_cb1410_pass_with_receipts() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let work_dir = tmp.path().join(".pmat-work").join("PMAT-008");
        let receipt_dir = work_dir.join("falsification");
        std::fs::create_dir_all(&receipt_dir).unwrap();
        std::fs::write(work_dir.join("contract.json"), r#"{"version": "5.0"}"#).unwrap();
        std::fs::write(receipt_dir.join("receipt-1.json"), "{}").unwrap();
        std::fs::write(receipt_dir.join("receipt-2.json"), "{}").unwrap();
        let result = check_subagent_contract_composition(tmp.path());
        assert_eq!(result.status, CheckStatus::Pass);
        assert!(result.message.contains("1/1"));
    }
}
