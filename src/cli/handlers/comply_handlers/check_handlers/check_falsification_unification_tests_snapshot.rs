// Work Falsification Unification — CB-1621 expected-snapshot drift tests.
//
// Included from `check_falsification_unification.rs` — do NOT add `use`
// imports or `#!` attributes here.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_snapshot {
    use super::*;
    use tempfile::tempdir;

    fn write_contract_with_expected_snapshot(
        project: &Path,
        ticket: &str,
        yaml_path: &str,
        equation: &str,
        test_id: &str,
        expected_canonical: &str,
    ) {
        use crate::cli::handlers::work_contract::{EvidenceType, FalsifiableClaim};
        let mut c = WorkContract::new(ticket.into(), "deadbeef".into());
        c.claims.push(FalsifiableClaim {
            hypothesis: "inherited claim".into(),
            falsification_method: FalsificationMethod::ProvableContract {
                yaml_path: PathBuf::from(yaml_path),
                equation: equation.into(),
                test_id: test_id.into(),
                expected: expected_canonical.into(),
            },
            evidence_required: EvidenceType::BooleanCheck(true),
            result: None,
            override_info: None,
        });
        let dir = project.join(".pmat-work").join(ticket);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("contract.json"),
            serde_json::to_string_pretty(&c).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn expected_drift_skips_with_no_contracts() {
        let tmp = tempdir().unwrap();
        let r = check_expected_snapshot_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No `.pmat-work/*/contract.json`"));
    }

    #[test]
    fn expected_drift_skips_when_no_snapshot_set() {
        // ProvableContract entry with empty `expected` → no snapshot to compare.
        let tmp = tempdir().unwrap();
        write_contract_with_expected_snapshot(
            tmp.path(),
            "T1",
            "contracts/k.yaml",
            "rope",
            "t1",
            "",
        );
        let r = check_expected_snapshot_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No ProvableContract entry"));
    }

    #[test]
    fn expected_drift_skips_when_no_yaml_declares_expected() {
        let tmp = tempdir().unwrap();
        write_yaml_at(
            tmp.path(),
            "contracts/k.yaml",
            "falsification_tests:\n  - id: t1\n",
        );
        write_contract_with_expected_snapshot(
            tmp.path(),
            "T1",
            "contracts/k.yaml",
            "rope",
            "t1",
            "true",
        );
        let r = check_expected_snapshot_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No bound YAML declares scalar"));
    }

    #[test]
    fn expected_drift_passes_on_matching_bool() {
        let tmp = tempdir().unwrap();
        write_yaml_at(
            tmp.path(),
            "contracts/k.yaml",
            "falsification_tests:\n  - id: t1\n    expected: true\n",
        );
        write_contract_with_expected_snapshot(
            tmp.path(),
            "T1",
            "contracts/k.yaml",
            "rope",
            "t1",
            "true",
        );
        let r = check_expected_snapshot_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn expected_drift_passes_on_matching_number() {
        let tmp = tempdir().unwrap();
        write_yaml_at(
            tmp.path(),
            "contracts/k.yaml",
            "falsification_tests:\n  - id: t1\n    expected: 42\n",
        );
        write_contract_with_expected_snapshot(
            tmp.path(),
            "T1",
            "contracts/k.yaml",
            "rope",
            "t1",
            "42",
        );
        let r = check_expected_snapshot_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn expected_drift_passes_on_matching_quoted_string() {
        // YAML `expected: "abc"` and snapshot `"\"abc\""` should both
        // canonicalize to JSON `"abc"`.
        let tmp = tempdir().unwrap();
        write_yaml_at(
            tmp.path(),
            "contracts/k.yaml",
            "falsification_tests:\n  - id: t1\n    expected: \"abc\"\n",
        );
        write_contract_with_expected_snapshot(
            tmp.path(),
            "T1",
            "contracts/k.yaml",
            "rope",
            "t1",
            "\"abc\"",
        );
        let r = check_expected_snapshot_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn expected_drift_fails_on_bool_flip() {
        let tmp = tempdir().unwrap();
        write_yaml_at(
            tmp.path(),
            "contracts/k.yaml",
            "falsification_tests:\n  - id: t1\n    expected: false\n",
        );
        write_contract_with_expected_snapshot(
            tmp.path(),
            "T1",
            "contracts/k.yaml",
            "rope",
            "t1",
            "true",
        );
        let r = check_expected_snapshot_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("t1"));
        assert!(r.message.contains("drifted"));
    }

    #[test]
    fn expected_drift_fails_on_number_change() {
        let tmp = tempdir().unwrap();
        write_yaml_at(
            tmp.path(),
            "contracts/k.yaml",
            "falsification_tests:\n  - id: t1\n    expected: 100\n",
        );
        write_contract_with_expected_snapshot(
            tmp.path(),
            "T1",
            "contracts/k.yaml",
            "rope",
            "t1",
            "42",
        );
        let r = check_expected_snapshot_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
    }

    #[test]
    fn expected_drift_skips_complex_inline_mapping() {
        // Inline mapping (`expected: {a: 1}`) bails out of scalar extraction
        // → per-entry skip → overall Skip because nothing was compared.
        let tmp = tempdir().unwrap();
        write_yaml_at(
            tmp.path(),
            "contracts/k.yaml",
            "falsification_tests:\n  - id: t1\n    expected: {a: 1}\n",
        );
        write_contract_with_expected_snapshot(
            tmp.path(),
            "T1",
            "contracts/k.yaml",
            "rope",
            "t1",
            "{\"a\": 1}",
        );
        let r = check_expected_snapshot_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("scalar"));
    }

    #[test]
    fn expected_drift_tolerates_missing_yaml() {
        // Bound YAML file doesn't exist — per-entry skip, overall Skip.
        let tmp = tempdir().unwrap();
        write_contract_with_expected_snapshot(
            tmp.path(),
            "T1",
            "contracts/missing.yaml",
            "rope",
            "t1",
            "true",
        );
        let r = check_expected_snapshot_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
    }

    #[test]
    fn expected_drift_reports_multiple_entries() {
        // Two tickets each with a drifted test_id — both show in the message.
        let tmp = tempdir().unwrap();
        write_yaml_at(
            tmp.path(),
            "contracts/k.yaml",
            "falsification_tests:\n  - id: t1\n    expected: false\n  - id: t2\n    expected: 7\n",
        );
        write_contract_with_expected_snapshot(
            tmp.path(),
            "T1",
            "contracts/k.yaml",
            "rope",
            "t1",
            "true",
        );
        write_contract_with_expected_snapshot(
            tmp.path(),
            "T2",
            "contracts/k.yaml",
            "rope",
            "t2",
            "42",
        );
        let r = check_expected_snapshot_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("T1"));
        assert!(r.message.contains("T2"));
    }

    #[test]
    fn yaml_expected_map_handles_multiple_tests() {
        let yaml = "falsification_tests:\n  - id: a\n    expected: true\n  - id: b\n    expected: 2\n  - id: c\n    expected: \"x\"\n";
        let m = yaml_expected_by_test_id(yaml);
        assert_eq!(m.get("a").unwrap(), "true");
        assert_eq!(m.get("b").unwrap(), "2");
        assert_eq!(m.get("c").unwrap(), "\"x\"");
    }

    #[test]
    fn yaml_scalar_null_tilde_becomes_json_null() {
        assert_eq!(yaml_scalar_to_canonical_json("~").unwrap(), "null");
        assert_eq!(yaml_scalar_to_canonical_json("null").unwrap(), "null");
    }

    #[test]
    fn yaml_scalar_bare_string_gets_quoted() {
        assert_eq!(yaml_scalar_to_canonical_json("hello").unwrap(), "\"hello\"");
    }

    #[test]
    fn yaml_scalar_rejects_complex_shapes() {
        assert!(yaml_scalar_to_canonical_json("{a: 1}").is_none());
        assert!(yaml_scalar_to_canonical_json("[1, 2]").is_none());
        assert!(yaml_scalar_to_canonical_json("|").is_none());
    }
}
