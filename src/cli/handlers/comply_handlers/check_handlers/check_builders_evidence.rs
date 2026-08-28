/// CB-1700..CB-1703: evidence-derived gates (branch protection, supply chain,
/// review changeset size, documented rule count). See `check_evidence_gates.rs`
/// for why these are numbered 1700 and not 1300.
///
/// The GitHub API is consulted **once** per run and the answer feeds both
/// CB-1700 and CB-1701, so adding the supply-chain wiring check costs no extra
/// round trip.
fn build_evidence_gate_checks(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
) -> Vec<ComplianceCheck> {
    use super::check_evidence_gates as ev;

    let protection = ev::fetch_branch_protection(project_path);
    let gates = evidence_gate_contexts(comply_config);

    let required = match &protection {
        ev::ProtectionEvidence::Fetched(body) => {
            ev::RequiredContexts::Known(ev::required_contexts(body))
        }
        ev::ProtectionEvidence::Unavailable(why) => ev::RequiredContexts::Unknown(why.clone()),
        ev::ProtectionEvidence::NoGitHubRemote => ev::RequiredContexts::NotAGitHubProject,
    };

    let supply = ev::SupplyChainEvidence {
        deny_toml: std::fs::read_to_string(project_path.join("deny.toml")).ok(),
        required,
        jobs: ev::scan_workflow_jobs(project_path),
        advisory_db: ev::advisory_db_age(),
    };

    let readme = std::fs::read_to_string(project_path.join("README.md")).unwrap_or_default();
    let registry = ev::enumerate_comply_rule_ids(project_path).map(|ids| ids.len());

    vec![
        filter_check_by_config(
            ev::evaluate_branch_protection(&protection, &gates),
            "cb-1700",
            comply_config,
        ),
        filter_check_by_config(ev::evaluate_supply_chain(&supply), "cb-1701", comply_config),
        filter_check_by_config(
            ev::evaluate_diff_size(&ev::sample_diff_sizes(project_path, 50)),
            "cb-1702",
            comply_config,
        ),
        filter_check_by_config(
            ev::evaluate_rule_count_claim(
                &ev::extract_rule_count_claim(&readme),
                readme.contains("pmat comply"),
                registry,
            ),
            "cb-1703",
            comply_config,
        ),
    ]
}

/// Doctrinal gate names CB-1700 requires among the *required contexts*.
///
/// Default `["gate"]`, which `context_satisfies_gate` matches against both a
/// bare `gate` context and the `ci / gate` a reusable workflow produces.
/// Override per project in `.pmat.yaml`:
///
/// ```yaml
/// comply:
///   checks:
///     cb-1700:
///       options:
///         gate_contexts: ["gate", "score"]
/// ```
fn evidence_gate_contexts(
    comply_config: &crate::models::comply_config::ComplyConfig,
) -> Vec<String> {
    comply_config
        .checks
        .get("cb-1700")
        .and_then(|c| c.options.get("gate_contexts"))
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_str())
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| vec!["gate".to_string()])
}
