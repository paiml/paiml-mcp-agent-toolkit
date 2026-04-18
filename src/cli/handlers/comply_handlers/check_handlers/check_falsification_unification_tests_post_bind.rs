// Work Falsification Unification — CB-1627 post-bind YAML drift tests.
//
// Included from `check_falsification_unification.rs` — do NOT add `use`
// imports or `#!` attributes here.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_post_bind {
    use super::*;
    use tempfile::tempdir;

    fn write_bound_contract(
        project: &Path,
        ticket: &str,
        yaml_rel: &str,
        equation: &str,
        seeded_test_ids: &[&str],
    ) {
        use crate::cli::handlers::work_contract::{
            ContractBinding, EvidenceType, FalsifiableClaim,
        };
        let mut c = WorkContract::new(ticket.into(), "deadbeef".into());
        c.implements.push(ContractBinding {
            contract: "k".into(),
            equation: equation.into(),
            file: PathBuf::from(yaml_rel),
            sha: "abc".into(),
            bound_at: chrono::Utc::now(),
        });
        for id in seeded_test_ids {
            c.claims.push(FalsifiableClaim {
                hypothesis: "inherited".into(),
                falsification_method: FalsificationMethod::ProvableContract {
                    yaml_path: PathBuf::from(yaml_rel),
                    equation: equation.into(),
                    test_id: (*id).into(),
                    expected: "\"canonical\"".into(),
                },
                evidence_required: EvidenceType::BooleanCheck(true),
                result: None,
                override_info: None,
            });
        }
        write_contract_json(project, ticket, &c);
    }

    fn yaml_with_tests(ids: &[&str]) -> String {
        let mut s = String::from("falsification_tests:\n");
        for id in ids {
            s.push_str(&format!("  - id: {}\n", id));
        }
        s
    }

    #[test]
    fn post_bind_drift_skips_without_pmat_work() {
        let tmp = tempdir().unwrap();
        let r = check_post_bind_yaml_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No `.pmat-work/`"));
    }

    #[test]
    fn post_bind_drift_skips_with_no_contracts() {
        let tmp = tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".pmat-work")).unwrap();
        let r = check_post_bind_yaml_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("contract.json"));
    }

    #[test]
    fn post_bind_drift_skips_without_bindings() {
        let tmp = tempdir().unwrap();
        let c = WorkContract::new("T-1".into(), "deadbeef".into());
        write_contract_json(tmp.path(), "T-1", &c);
        let r = check_post_bind_yaml_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("`implements:` bindings"));
    }

    #[test]
    fn post_bind_drift_skips_when_yaml_missing() {
        let tmp = tempdir().unwrap();
        write_bound_contract(tmp.path(), "T-1", "contracts/k.yaml", "rope", &["t1"]);
        // YAML file does not exist on disk
        let r = check_post_bind_yaml_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("falsification_tests"));
    }

    #[test]
    fn post_bind_drift_skips_when_yaml_has_no_falsification_tests() {
        let tmp = tempdir().unwrap();
        write_bound_contract(tmp.path(), "T-1", "contracts/k.yaml", "rope", &["t1"]);
        write_yaml_at(tmp.path(), "contracts/k.yaml", "equations:\n  rope: {}\n");
        let r = check_post_bind_yaml_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
    }

    #[test]
    fn post_bind_drift_passes_when_roster_matches_yaml() {
        let tmp = tempdir().unwrap();
        write_bound_contract(tmp.path(), "T-1", "contracts/k.yaml", "rope", &["t1", "t2"]);
        write_yaml_at(
            tmp.path(),
            "contracts/k.yaml",
            &yaml_with_tests(&["t1", "t2"]),
        );
        let r = check_post_bind_yaml_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn post_bind_drift_passes_when_roster_superset_of_yaml() {
        // Deletion from YAML is CB-1626's concern (stale roster references),
        // not CB-1627's — this check only flags ADDITIONS post-bind.
        let tmp = tempdir().unwrap();
        write_bound_contract(
            tmp.path(),
            "T-1",
            "contracts/k.yaml",
            "rope",
            &["t1", "t2", "t3"],
        );
        write_yaml_at(
            tmp.path(),
            "contracts/k.yaml",
            &yaml_with_tests(&["t1", "t2"]),
        );
        let r = check_post_bind_yaml_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass);
    }

    #[test]
    fn post_bind_drift_warns_on_new_yaml_entry() {
        let tmp = tempdir().unwrap();
        write_bound_contract(tmp.path(), "T-1", "contracts/k.yaml", "rope", &["t1"]);
        write_yaml_at(
            tmp.path(),
            "contracts/k.yaml",
            &yaml_with_tests(&["t1", "t2"]),
        );
        let r = check_post_bind_yaml_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Warn);
        assert!(r.message.contains("T-1"));
        assert!(r.message.contains("t2"));
    }

    #[test]
    fn post_bind_drift_warns_listing_multiple_additions() {
        let tmp = tempdir().unwrap();
        write_bound_contract(tmp.path(), "T-1", "contracts/k.yaml", "rope", &["t1"]);
        write_yaml_at(
            tmp.path(),
            "contracts/k.yaml",
            &yaml_with_tests(&["t1", "t2", "t3"]),
        );
        let r = check_post_bind_yaml_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Warn);
        assert!(r.message.contains("t2"));
        assert!(r.message.contains("t3"));
        assert!(r.message.contains("2 new"));
    }

    #[test]
    fn post_bind_drift_is_per_binding_equation_scoped() {
        // A ProvableContract for equation `rope` should not mask drift
        // for equation `linear` in the same YAML.
        let tmp = tempdir().unwrap();
        use crate::cli::handlers::work_contract::{
            ContractBinding, EvidenceType, FalsifiableClaim,
        };
        let mut c = WorkContract::new("T-1".into(), "deadbeef".into());
        c.implements.push(ContractBinding {
            contract: "k".into(),
            equation: "linear".into(),
            file: PathBuf::from("contracts/k.yaml"),
            sha: "abc".into(),
            bound_at: chrono::Utc::now(),
        });
        // Claim is for equation `rope`, NOT `linear`.
        c.claims.push(FalsifiableClaim {
            hypothesis: "inherited".into(),
            falsification_method: FalsificationMethod::ProvableContract {
                yaml_path: PathBuf::from("contracts/k.yaml"),
                equation: "rope".into(),
                test_id: "t1".into(),
                expected: "\"\"".into(),
            },
            evidence_required: EvidenceType::BooleanCheck(true),
            result: None,
            override_info: None,
        });
        write_contract_json(tmp.path(), "T-1", &c);
        write_yaml_at(tmp.path(), "contracts/k.yaml", &yaml_with_tests(&["t1"]));
        // From the `linear` binding's perspective, seeded set is EMPTY — so
        // YAML entry `t1` shows up as drift.
        let r = check_post_bind_yaml_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Warn);
        assert!(r.message.contains("linear"));
    }

    #[test]
    fn post_bind_drift_aggregates_across_tickets() {
        let tmp = tempdir().unwrap();
        write_bound_contract(tmp.path(), "T-1", "contracts/a.yaml", "eq", &["t1"]);
        write_bound_contract(tmp.path(), "T-2", "contracts/b.yaml", "eq", &["s1"]);
        write_yaml_at(
            tmp.path(),
            "contracts/a.yaml",
            &yaml_with_tests(&["t1", "t2"]),
        );
        write_yaml_at(tmp.path(), "contracts/b.yaml", &yaml_with_tests(&["s1"]));
        let r = check_post_bind_yaml_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Warn);
        assert!(r.message.contains("T-1"));
        assert!(!r.message.contains("T-2"));
    }
}
