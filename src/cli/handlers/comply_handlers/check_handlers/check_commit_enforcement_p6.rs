
/// Get staged files from git diff --cached --name-only.
/// Returns relative file paths. Falls back to empty vec if git not available.
fn get_staged_files(project_path: &Path) -> Vec<String> {
    let output = std::process::Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .current_dir(project_path)
        .output();

    match output {
        Ok(o) if o.status.success() => {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter(|l| !l.is_empty())
                .map(|l| l.to_string())
                .collect()
        }
        _ => Vec::new(),
    }
}

/// CB-1351: Binding Index Freshness
///
/// Checks that `.pmat/binding-index.json` exists and is not stale.
/// Staleness: >7 days = warning, >30 days = error.
/// Freshness is essential for O(1) differential obligation checks.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_binding_index_freshness(project_path: &Path) -> ComplianceCheck {
    let idx_path = project_path.join(".pmat/binding-index.json");
    let alt_path = project_path.join("contracts/binding-index.json");

    let path = if idx_path.exists() {
        idx_path
    } else if alt_path.exists() {
        alt_path
    } else {
        return ComplianceCheck {
            name: "CB-1351: Binding Index Freshness".into(),
            status: CheckStatus::Skip,
            message: "No binding-index.json found".into(),
            severity: Severity::Info,
        };
    };

    let metadata = match fs::metadata(&path) {
        Ok(m) => m,
        Err(_) => {
            return ComplianceCheck {
                name: "CB-1351: Binding Index Freshness".into(),
                status: CheckStatus::Warn,
                message: "Could not read binding-index.json metadata".into(),
                severity: Severity::Warning,
            };
        }
    };

    let modified = match metadata.modified() {
        Ok(t) => t,
        Err(_) => {
            return ComplianceCheck {
                name: "CB-1351: Binding Index Freshness".into(),
                status: CheckStatus::Warn,
                message: "Could not determine binding-index.json age".into(),
                severity: Severity::Warning,
            };
        }
    };

    let age = std::time::SystemTime::now()
        .duration_since(modified)
        .unwrap_or_default();
    let days = age.as_secs() / 86400;

    if days > 30 {
        ComplianceCheck {
            name: "CB-1351: Binding Index Freshness".into(),
            status: CheckStatus::Fail,
            message: format!(
                "binding-index.json is {} days old (>30 days, run pmat comply refresh-bindings)",
                days
            ),
            severity: Severity::Error,
        }
    } else if days > 7 {
        ComplianceCheck {
            name: "CB-1351: Binding Index Freshness".into(),
            status: CheckStatus::Warn,
            message: format!(
                "binding-index.json is {} days old (>7 days, consider refreshing)",
                days
            ),
            severity: Severity::Warning,
        }
    } else {
        ComplianceCheck {
            name: "CB-1351: Binding Index Freshness".into(),
            status: CheckStatus::Pass,
            message: format!("binding-index.json fresh ({} day(s) old)", days),
            severity: Severity::Info,
        }
    }
}

/// CB-1352: Assume-Guarantee Chain Validation
///
/// When multiple work items touch overlapping code, one commit can break
/// another's assumptions. Scans active work contracts for `assumes` and
/// `guarantees` fields, builds a dependency DAG, and checks if staged
/// changes would break any guarantee that another work item assumes.
///
/// Spec: Phase 5 of commit-level-contract-enforcement.md
/// Basis: Pacti (ACM TCPS 2025); Dewes & Dimitrova (AAAI 2025)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_assume_guarantee_chains(project_path: &Path) -> ComplianceCheck {
    let work_dir = project_path.join(".pmat-work");
    if !work_dir.exists() {
        return ComplianceCheck {
            name: "CB-1352: Assume-Guarantee Chains".into(),
            status: CheckStatus::Skip,
            message: "No .pmat-work/ directory".into(),
            severity: Severity::Info,
        };
    }

    // Collect active contracts with assumes/guarantees
    let mut contracts_with_ag: Vec<AgContract> = Vec::new();

    let entries = match fs::read_dir(&work_dir) {
        Ok(e) => e,
        Err(_) => {
            return ComplianceCheck {
                name: "CB-1352: Assume-Guarantee Chains".into(),
                status: CheckStatus::Skip,
                message: "Could not read .pmat-work/".into(),
                severity: Severity::Info,
            };
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let contract_path = path.join("contract.json");
        if !contract_path.exists() {
            continue;
        }
        if let Ok(content) = fs::read_to_string(&contract_path) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                let id = v.get("work_item_id")
                    .and_then(|w| w.as_str())
                    .unwrap_or_else(|| path.file_name().unwrap_or_default().to_str().unwrap_or("unknown"));
                let assumes = extract_string_array(&v, "assumes");
                let guarantees = extract_string_array(&v, "guarantees");
                let files = extract_string_array(&v, "files")
                    .into_iter()
                    .chain(extract_string_array(&v, "touched_files"))
                    .collect::<Vec<_>>();
                if !assumes.is_empty() || !guarantees.is_empty() {
                    contracts_with_ag.push(AgContract {
                        id: id.to_string(),
                        assumes,
                        guarantees,
                        files,
                    });
                }
            }
        }
    }

    if contracts_with_ag.is_empty() {
        return ComplianceCheck {
            name: "CB-1352: Assume-Guarantee Chains".into(),
            status: CheckStatus::Pass,
            message: "No work contracts with assume-guarantee declarations".into(),
            severity: Severity::Info,
        };
    }

    // Get staged files
    let staged = get_staged_files(project_path);
    if staged.is_empty() {
        return ComplianceCheck {
            name: "CB-1352: Assume-Guarantee Chains".into(),
            status: CheckStatus::Pass,
            message: format!(
                "{} A/G contract(s), no staged files",
                contracts_with_ag.len()
            ),
            severity: Severity::Info,
        };
    }

    // Check: for each staged file, find contracts whose guarantees cover that file.
    // Then check if any OTHER contract assumes that guarantee.
    let mut broken: Vec<String> = Vec::new();

    for contract in &contracts_with_ag {
        // Check if staged files overlap with this contract's guaranteed files
        let overlaps = contract.files.iter().any(|f| {
            staged.iter().any(|sf| sf.contains(f) || f.contains(sf))
        });
        if !overlaps {
            continue;
        }
        // This contract's guarantees might be affected. Check who assumes them.
        for guarantee in &contract.guarantees {
            for other in &contracts_with_ag {
                if other.id == contract.id {
                    continue;
                }
                if other.assumes.contains(guarantee) {
                    broken.push(format!(
                        "{} guarantees '{}' assumed by {}",
                        contract.id, guarantee, other.id
                    ));
                }
            }
        }
    }

    if broken.is_empty() {
        ComplianceCheck {
            name: "CB-1352: Assume-Guarantee Chains".into(),
            status: CheckStatus::Pass,
            message: format!(
                "{} A/G contract(s), no broken chains for {} staged file(s)",
                contracts_with_ag.len(),
                staged.len()
            ),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-1352: Assume-Guarantee Chains".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{} broken chain(s): {}",
                broken.len(),
                broken.join("; ")
            ),
            severity: Severity::Warning,
        }
    }
}

/// Internal struct for assume-guarantee contract parsing.
struct AgContract {
    id: String,
    assumes: Vec<String>,
    guarantees: Vec<String>,
    files: Vec<String>,
}

/// Extract a JSON array of strings from a field.
fn extract_string_array(v: &serde_json::Value, field: &str) -> Vec<String> {
    v.get(field)
        .and_then(|a| a.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|s| s.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

/// CB-1353: Assume-Guarantee Cycle Detection
///
/// The work contract dependency DAG (assumes → guarantees) must be acyclic.
/// Cycles create circular proof obligations that can never be resolved.
/// Uses DFS-based cycle detection on the assumes→guarantees graph.
///
/// Basis: Dardik & Kang (2025) compositional inductive invariant inference
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_ag_cycle_detection(project_path: &Path) -> ComplianceCheck {
    let work_dir = project_path.join(".pmat-work");
    if !work_dir.exists() {
        return ComplianceCheck {
            name: "CB-1353: A/G Cycle Detection".into(),
            status: CheckStatus::Skip,
            message: "No .pmat-work/ directory".into(),
            severity: Severity::Info,
        };
    }

    // Build graph: work_item → [work_items it depends on (via assumes matching guarantees)]
    let mut guarantee_to_owner: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut item_assumes: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

    let entries = match fs::read_dir(&work_dir) {
        Ok(e) => e,
        Err(_) => {
            return ComplianceCheck {
                name: "CB-1353: A/G Cycle Detection".into(),
                status: CheckStatus::Skip,
                message: "Could not read .pmat-work/".into(),
                severity: Severity::Info,
            };
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let contract_path = path.join("contract.json");
        if !contract_path.exists() {
            continue;
        }
        if let Ok(content) = fs::read_to_string(&contract_path) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                let id = v.get("work_item_id")
                    .and_then(|w| w.as_str())
                    .unwrap_or_else(|| path.file_name().unwrap_or_default().to_str().unwrap_or("unknown"))
                    .to_string();
                let assumes = extract_string_array(&v, "assumes");
                let guarantees = extract_string_array(&v, "guarantees");

                for g in &guarantees {
                    guarantee_to_owner.insert(g.clone(), id.clone());
                }
                if !assumes.is_empty() {
                    item_assumes.insert(id, assumes);
                }
            }
        }
    }

    if guarantee_to_owner.is_empty() && item_assumes.is_empty() {
        return ComplianceCheck {
            name: "CB-1353: A/G Cycle Detection".into(),
            status: CheckStatus::Pass,
            message: "No assume-guarantee relationships to check".into(),
            severity: Severity::Info,
        };
    }

    // Build adjacency: item → [items it depends on]
    let mut adj: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for (item, assumes) in &item_assumes {
        for a in assumes {
            if let Some(owner) = guarantee_to_owner.get(a) {
                if owner != item {
                    adj.entry(item.clone()).or_default().push(owner.clone());
                }
            }
        }
    }

    // DFS cycle detection
    let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut in_stack: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut cycles: Vec<String> = Vec::new();

    let all_nodes: Vec<String> = adj.keys().cloned().collect();
    for node in &all_nodes {
        if !visited.contains(node) {
            dfs_cycle_check(node, &adj, &mut visited, &mut in_stack, &mut cycles);
        }
    }

    if cycles.is_empty() {
        ComplianceCheck {
            name: "CB-1353: A/G Cycle Detection".into(),
            status: CheckStatus::Pass,
            message: format!(
                "{} A/G relationship(s), DAG is acyclic",
                guarantee_to_owner.len()
            ),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-1353: A/G Cycle Detection".into(),
            status: CheckStatus::Fail,
            message: format!(
                "Cycle(s) in A/G DAG: {}",
                cycles.join(", ")
            ),
            severity: Severity::Error,
        }
    }
}

/// DFS cycle detection helper.
fn dfs_cycle_check(
    node: &str,
    adj: &std::collections::HashMap<String, Vec<String>>,
    visited: &mut std::collections::HashSet<String>,
    in_stack: &mut std::collections::HashSet<String>,
    cycles: &mut Vec<String>,
) {
    visited.insert(node.to_string());
    in_stack.insert(node.to_string());

    if let Some(neighbors) = adj.get(node) {
        for neighbor in neighbors {
            if !visited.contains(neighbor.as_str()) {
                dfs_cycle_check(neighbor, adj, visited, in_stack, cycles);
            } else if in_stack.contains(neighbor.as_str()) {
                cycles.push(format!("{} → {}", node, neighbor));
            }
        }
    }

    in_stack.remove(node);
}
