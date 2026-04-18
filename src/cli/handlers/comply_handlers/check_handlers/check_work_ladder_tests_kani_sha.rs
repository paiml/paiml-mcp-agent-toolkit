// Tests for CB-1615 (Kani harness SHA drift). Included into
// `check_work_ladder.rs`.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_work_ladder_kani_sha {
    use super::*;
    use crate::cli::handlers::work_contract::ContractBinding;
    use std::path::PathBuf;
    use tempfile::tempdir;

    // ─── CB-1615: Kani harness SHA drift ─────────────────────────────────────

    fn write_harness_snapshot(project: &Path, id: &str, body: &str) {
        let dir = project.join(".pmat-work").join(id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("kani-harness-shas.json"), body).unwrap();
    }

    fn make_l4_bound_contract(id: &str, yaml_relpath: &str) -> WorkContract {
        let mut c = make_contract(id, "L4");
        c.implements.push(ContractBinding {
            contract: "proto".into(),
            equation: "eq".into(),
            file: PathBuf::from(yaml_relpath),
            sha: "deadbeef".into(),
            bound_at: chrono::Utc::now(),
        });
        c
    }

    #[test]
    fn kani_sha_skips_with_no_contracts() {
        let tmp = tempdir().unwrap();
        let r = check_ladder_kani_harness_sha(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No `.pmat-work/*/contract.json`"));
    }

    #[test]
    fn kani_sha_skips_with_no_l4_tickets() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L3").save(tmp.path()).unwrap();
        let r = check_ladder_kani_harness_sha(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No L4+ ticket"));
    }

    #[test]
    fn kani_sha_skips_with_no_bindings() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L4").save(tmp.path()).unwrap();
        let r = check_ladder_kani_harness_sha(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("implements"));
    }

    #[test]
    fn kani_sha_skips_with_no_snapshot_file() {
        let tmp = tempdir().unwrap();
        write_yaml(
            tmp.path(),
            "proto",
            "equations:\n  eq: {}\nkani_harnesses:\n  - name: h1\n    sha: aaaa\n",
        );
        make_l4_bound_contract("T-1", "contracts/proto.yaml")
            .save(tmp.path())
            .unwrap();
        let r = check_ladder_kani_harness_sha(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("kani-harness-shas.json"));
    }

    #[test]
    fn kani_sha_skips_when_snapshot_empty() {
        let tmp = tempdir().unwrap();
        write_yaml(
            tmp.path(),
            "proto",
            "equations:\n  eq: {}\nkani_harnesses:\n  - name: h1\n    sha: aaaa\n",
        );
        make_l4_bound_contract("T-1", "contracts/proto.yaml")
            .save(tmp.path())
            .unwrap();
        write_harness_snapshot(tmp.path(), "T-1", r#"{"harnesses": []}"#);
        let r = check_ladder_kani_harness_sha(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("all empty"));
    }

    #[test]
    fn kani_sha_passes_when_snapshot_matches_yaml_array_shape() {
        let tmp = tempdir().unwrap();
        write_yaml(
            tmp.path(),
            "proto",
            "equations:\n  eq: {}\nkani_harnesses:\n  - name: h1\n    sha: aaaa\n  - name: h2\n    sha: bbbb\n",
        );
        make_l4_bound_contract("T-1", "contracts/proto.yaml")
            .save(tmp.path())
            .unwrap();
        write_harness_snapshot(
            tmp.path(),
            "T-1",
            r#"{"harnesses": [{"name": "h1", "sha": "aaaa"}, {"name": "h2", "sha": "bbbb"}]}"#,
        );
        let r = check_ladder_kani_harness_sha(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
        assert!(r.message.contains("match bind-time"));
    }

    #[test]
    fn kani_sha_passes_when_snapshot_uses_map_shape() {
        let tmp = tempdir().unwrap();
        write_yaml(
            tmp.path(),
            "proto",
            "equations:\n  eq: {}\nkani_harnesses:\n  - name: h1\n    sha: aaaa\n",
        );
        make_l4_bound_contract("T-1", "contracts/proto.yaml")
            .save(tmp.path())
            .unwrap();
        write_harness_snapshot(tmp.path(), "T-1", r#"{"harnesses": {"h1": "aaaa"}}"#);
        let r = check_ladder_kani_harness_sha(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn kani_sha_fails_on_drift() {
        let tmp = tempdir().unwrap();
        write_yaml(
            tmp.path(),
            "proto",
            "equations:\n  eq: {}\nkani_harnesses:\n  - name: h1\n    sha: zzzzzzzz\n",
        );
        make_l4_bound_contract("T-1", "contracts/proto.yaml")
            .save(tmp.path())
            .unwrap();
        write_harness_snapshot(
            tmp.path(),
            "T-1",
            r#"{"harnesses": [{"name": "h1", "sha": "aaaaaaaa"}]}"#,
        );
        let r = check_ladder_kani_harness_sha(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("h1"));
        assert!(r.message.contains("drifted"));
    }

    #[test]
    fn kani_sha_fails_when_harness_removed_from_yaml() {
        let tmp = tempdir().unwrap();
        write_yaml(
            tmp.path(),
            "proto",
            // h1 present, h2 removed
            "equations:\n  eq: {}\nkani_harnesses:\n  - name: h1\n    sha: aaaa\n",
        );
        make_l4_bound_contract("T-1", "contracts/proto.yaml")
            .save(tmp.path())
            .unwrap();
        write_harness_snapshot(
            tmp.path(),
            "T-1",
            r#"{"harnesses": [{"name": "h1", "sha": "aaaa"}, {"name": "h2", "sha": "bbbb"}]}"#,
        );
        let r = check_ladder_kani_harness_sha(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("h2"));
        assert!(r.message.contains("removed post-bind"));
    }

    #[test]
    fn kani_sha_fails_on_malformed_snapshot_json() {
        let tmp = tempdir().unwrap();
        write_yaml(
            tmp.path(),
            "proto",
            "equations:\n  eq: {}\nkani_harnesses:\n  - name: h1\n    sha: aaaa\n",
        );
        make_l4_bound_contract("T-1", "contracts/proto.yaml")
            .save(tmp.path())
            .unwrap();
        write_harness_snapshot(tmp.path(), "T-1", "not json at all");
        let r = check_ladder_kani_harness_sha(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("malformed"));
    }

    #[test]
    fn kani_sha_parses_yaml_shas_for_multiple_items() {
        // Regression: state-machine must commit between list items.
        let yaml = "kani_harnesses:\n  - name: a\n    sha: 111\n  - name: b\n    sha: 222\n  - name: c\n    sha: 333\n";
        let got = yaml_kani_harness_shas(yaml).unwrap();
        assert_eq!(got.get("a").unwrap(), "111");
        assert_eq!(got.get("b").unwrap(), "222");
        assert_eq!(got.get("c").unwrap(), "333");
    }

    #[test]
    fn kani_sha_yaml_shas_none_when_section_absent() {
        let yaml = "equations:\n  eq: {}\n";
        assert!(yaml_kani_harness_shas(yaml).is_none());
    }

    #[test]
    fn kani_sha_yaml_shas_skips_items_without_sha() {
        // String-form and name-only object-form items should be silently
        // skipped so they don't participate in drift detection.
        let yaml =
            "kani_harnesses:\n  - name: a\n    sha: 111\n  - name: b\n  - plain_string_form\n";
        let got = yaml_kani_harness_shas(yaml).unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got.get("a").unwrap(), "111");
    }

    #[test]
    fn kani_sha_snapshot_parser_rejects_non_array_non_object_harnesses() {
        // harnesses is a scalar — schema mismatch.
        let body = r#"{"harnesses": "oops"}"#;
        assert!(parse_kani_harness_sha_snapshot(body).is_none());
    }

    #[test]
    fn kani_sha_only_checks_l4_plus() {
        // An L3 ticket with a snapshot should be ignored entirely — CB-1615
        // only gates L4+ bindings.
        let tmp = tempdir().unwrap();
        write_yaml(
            tmp.path(),
            "proto",
            "equations:\n  eq: {}\nkani_harnesses:\n  - name: h1\n    sha: aaaa\n",
        );
        let mut c = make_contract("T-1", "L3");
        c.implements.push(ContractBinding {
            contract: "proto".into(),
            equation: "eq".into(),
            file: PathBuf::from("contracts/proto.yaml"),
            sha: "deadbeef".into(),
            bound_at: chrono::Utc::now(),
        });
        c.save(tmp.path()).unwrap();
        // Drift wouldn't matter — ticket is L3.
        write_harness_snapshot(
            tmp.path(),
            "T-1",
            r#"{"harnesses": [{"name": "h1", "sha": "zzzzzzzz"}]}"#,
        );
        let r = check_ladder_kani_harness_sha(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No L4+ ticket"));
    }

    #[test]
    fn kani_sha_l5_tickets_also_gated() {
        let tmp = tempdir().unwrap();
        write_yaml(
            tmp.path(),
            "proto",
            "equations:\n  eq: {}\nkani_harnesses:\n  - name: h1\n    sha: aaaa\n",
        );
        let mut c = make_contract("T-1", "L5");
        c.implements.push(ContractBinding {
            contract: "proto".into(),
            equation: "eq".into(),
            file: PathBuf::from("contracts/proto.yaml"),
            sha: "deadbeef".into(),
            bound_at: chrono::Utc::now(),
        });
        c.save(tmp.path()).unwrap();
        write_harness_snapshot(
            tmp.path(),
            "T-1",
            r#"{"harnesses": [{"name": "h1", "sha": "aaaa"}]}"#,
        );
        let r = check_ladder_kani_harness_sha(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }
}
