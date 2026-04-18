// Work Falsification Unification — shared test fixtures.
//
// Included from `check_falsification_unification.rs` — do NOT add `use`
// imports or `#!` attributes here.
//
// Keeping fixtures at module scope (guarded by `#[cfg(test)]`) so every
// per-CB test partition can reach them via `super::*`.

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
fn write_contract_json(project: &Path, ticket: &str, contract: &WorkContract) {
    let dir = project.join(".pmat-work").join(ticket);
    std::fs::create_dir_all(&dir).unwrap();
    let json = serde_json::to_string_pretty(contract).unwrap();
    std::fs::write(dir.join("contract.json"), json).unwrap();
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
fn write_log(project: &Path, ticket: &str, jsonl: &str) {
    let dir = project.join(".pmat-work").join(ticket);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("falsification.log"), jsonl).unwrap();
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
fn write_yaml_at(project: &Path, rel: &str, body: &str) {
    let p = project.join(rel);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&p, body).unwrap();
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
fn contract_at_level(ticket: &str, level: &str) -> WorkContract {
    let mut c = WorkContract::new(ticket.into(), "deadbeef".into());
    c.verification_level = level.into();
    c
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
fn contract_with_provable(
    ticket: &str,
    yaml_path: &str,
    equation: &str,
    test_id: &str,
) -> WorkContract {
    use crate::cli::handlers::work_contract::{EvidenceType, FalsifiableClaim};
    let mut c = WorkContract::new(ticket.into(), "deadbeef".into());
    c.claims.push(FalsifiableClaim {
        hypothesis: "inherited claim".into(),
        falsification_method: FalsificationMethod::ProvableContract {
            yaml_path: PathBuf::from(yaml_path),
            equation: equation.into(),
            test_id: test_id.into(),
            expected: "\"canonical\"".into(),
        },
        evidence_required: EvidenceType::BooleanCheck(true),
        result: None,
        override_info: None,
    });
    c
}
