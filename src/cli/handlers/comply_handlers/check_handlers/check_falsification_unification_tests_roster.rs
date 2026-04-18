// Work Falsification Unification — roster integrity tests.
//
// Covers yaml_falsification_test_ids parser + CB-1620/1623/1626 skip-path
// tests + ProvableContract serde round-trip.
//
// Included from `check_falsification_unification.rs` — do NOT add `use`
// imports or `#!` attributes here.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_roster {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn yaml_ids_extracts_list_form() {
        let y = "falsification_tests:\n  - id: a\n  - id: \"b\"\n  - id: 'c'\n";
        assert_eq!(yaml_falsification_test_ids(y), vec!["a", "b", "c"]);
    }

    #[test]
    fn yaml_ids_ignores_other_sections() {
        let y = "equations:\n  - id: not_here\nfalsification_tests:\n  - id: actual\nkani_harnesses:\n  - id: neither\n";
        assert_eq!(yaml_falsification_test_ids(y), vec!["actual"]);
    }

    #[test]
    fn yaml_ids_empty_when_section_absent() {
        assert!(yaml_falsification_test_ids("equations:\n  rope: {}\n").is_empty());
    }

    #[test]
    fn yaml_ids_empty_flow_list() {
        let y = "falsification_tests: []\n";
        assert!(yaml_falsification_test_ids(y).is_empty());
    }

    #[test]
    fn roster_coverage_skips_without_bindings() {
        let tmp = tempdir().unwrap();
        let check = check_inherited_roster_coverage(tmp.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    // ─── CB-1620: roster coverage Pass + Warn/Fail on cutoff ─────────────────

    fn contract_with_binding_and_provable(
        ticket: &str,
        yaml_rel: &str,
        equation: &str,
        test_id: &str,
    ) -> WorkContract {
        use crate::cli::handlers::work_contract::ContractBinding;
        let mut c = contract_with_provable(ticket, yaml_rel, equation, test_id);
        c.implements.push(ContractBinding {
            contract: "k".into(),
            equation: equation.into(),
            file: PathBuf::from(yaml_rel),
            sha: "deadbeef".into(),
            bound_at: chrono::Utc::now(),
        });
        c
    }

    #[test]
    fn roster_coverage_passes_when_every_yaml_test_is_inherited() {
        let tmp = tempdir().unwrap();
        write_yaml_at(
            tmp.path(),
            "contracts/k.yaml",
            "falsification_tests:\n  - id: foo\n",
        );
        let c =
            contract_with_binding_and_provable("PMAT-620", "contracts/k.yaml", "rope", "foo");
        write_contract_json(tmp.path(), "PMAT-620", &c);
        let check = check_inherited_roster_coverage(tmp.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn roster_coverage_warns_on_gap_before_cutoff() {
        let tmp = tempdir().unwrap();
        write_yaml_at(
            tmp.path(),
            "contracts/k.yaml",
            "falsification_tests:\n  - id: foo\n  - id: bar\n",
        );
        // Ticket only inherits `foo`, leaving `bar` uncovered.
        let c =
            contract_with_binding_and_provable("PMAT-620", "contracts/k.yaml", "rope", "foo");
        write_contract_json(tmp.path(), "PMAT-620", &c);
        let pre = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
        let check = check_inherited_roster_coverage_at(tmp.path(), pre);
        assert_eq!(check.status, CheckStatus::Warn, "{}", check.message);
        assert!(check.message.contains("bar"));
        assert!(check.message.contains("closes"));
    }

    #[test]
    fn roster_coverage_fails_on_gap_at_or_after_cutoff() {
        let tmp = tempdir().unwrap();
        write_yaml_at(
            tmp.path(),
            "contracts/k.yaml",
            "falsification_tests:\n  - id: foo\n  - id: bar\n",
        );
        let c =
            contract_with_binding_and_provable("PMAT-620", "contracts/k.yaml", "rope", "foo");
        write_contract_json(tmp.path(), "PMAT-620", &c);
        let post = NaiveDate::from_ymd_opt(2026, 5, 17).unwrap();
        let check = check_inherited_roster_coverage_at(tmp.path(), post);
        assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
        assert!(check.message.contains("bar"));
        assert!(check.message.contains("closed on"));
    }

    #[test]
    fn duplicate_entries_skips_without_provable_contract() {
        let tmp = tempdir().unwrap();
        let check = check_no_duplicate_provable_entries(tmp.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    // ─── CB-1623: no-duplicate Pass + Fail ───────────────────────────────────

    #[test]
    fn duplicate_entries_passes_on_unique_roster() {
        let tmp = tempdir().unwrap();
        let c = contract_with_provable("PMAT-620", "contracts/k.yaml", "rope", "foo");
        write_contract_json(tmp.path(), "PMAT-620", &c);
        let check = check_no_duplicate_provable_entries(tmp.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn duplicate_entries_fails_on_collision() {
        let tmp = tempdir().unwrap();
        // Two claims with the same (yaml_path, test_id) — different equation
        // so we exercise the (yaml, test_id) dedup rule, not an equation check.
        let mut c = contract_with_provable("PMAT-620", "contracts/k.yaml", "rope", "foo");
        let dupe = contract_with_provable("_", "contracts/k.yaml", "softmax", "foo");
        c.claims.extend(dupe.claims);
        write_contract_json(tmp.path(), "PMAT-620", &c);
        let check = check_no_duplicate_provable_entries(tmp.path());
        assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
        assert!(check.message.contains("PMAT-620"));
        assert!(check.message.contains("foo"));
    }

    #[test]
    fn test_id_exists_skips_without_provable_contract() {
        let tmp = tempdir().unwrap();
        let check = check_test_id_exists_in_yaml(tmp.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    // ─── CB-1626: referenced test_id existence Pass + Fail ──────────────────

    #[test]
    fn test_id_exists_passes_when_yaml_lists_id() {
        let tmp = tempdir().unwrap();
        write_yaml_at(
            tmp.path(),
            "contracts/k.yaml",
            "falsification_tests:\n  - id: foo\n",
        );
        let c = contract_with_provable("PMAT-620", "contracts/k.yaml", "rope", "foo");
        write_contract_json(tmp.path(), "PMAT-620", &c);
        let check = check_test_id_exists_in_yaml(tmp.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn test_id_exists_fails_on_stale_reference() {
        let tmp = tempdir().unwrap();
        // YAML no longer contains `foo` (typo or post-unbind rename).
        write_yaml_at(
            tmp.path(),
            "contracts/k.yaml",
            "falsification_tests:\n  - id: fooo\n",
        );
        let c = contract_with_provable("PMAT-620", "contracts/k.yaml", "rope", "foo");
        write_contract_json(tmp.path(), "PMAT-620", &c);
        let check = check_test_id_exists_in_yaml(tmp.path());
        assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
        assert!(check.message.contains("foo"));
    }

    #[test]
    fn provable_contract_variant_round_trips_through_serde() {
        let m = FalsificationMethod::ProvableContract {
            yaml_path: PathBuf::from("contracts/rope-kernel-v1.yaml"),
            equation: "rope".into(),
            test_id: "rope_periodicity_test".into(),
            expected: "1.0".into(),
        };
        let json = serde_json::to_string(&m).unwrap();
        let back: FalsificationMethod = serde_json::from_str(&json).unwrap();
        match back {
            FalsificationMethod::ProvableContract {
                yaml_path,
                equation,
                test_id,
                expected,
            } => {
                assert_eq!(yaml_path, PathBuf::from("contracts/rope-kernel-v1.yaml"));
                assert_eq!(equation, "rope");
                assert_eq!(test_id, "rope_periodicity_test");
                assert_eq!(expected, "1.0");
            }
            other => panic!("round-trip landed on wrong variant: {:?}", other),
        }
    }
}
