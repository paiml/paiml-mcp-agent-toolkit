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

    #[test]
    fn duplicate_entries_skips_without_provable_contract() {
        let tmp = tempdir().unwrap();
        let check = check_no_duplicate_provable_entries(tmp.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn test_id_exists_skips_without_provable_contract() {
        let tmp = tempdir().unwrap();
        let check = check_test_id_exists_in_yaml(tmp.path());
        assert_eq!(check.status, CheckStatus::Skip);
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
