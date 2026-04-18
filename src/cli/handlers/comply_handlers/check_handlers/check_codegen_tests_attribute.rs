// Tests for CB-1631/1632/1634/1638 — attribute scanning, expr/binds_to linkage,
// and generated-modules-tracked checks.
// Included from check_codegen.rs — do NOT add `use` imports or `#!` attributes here.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_attribute {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn attribute_parser_extracts_id_and_clauses() {
        let (attr, id_rx, req_rx, ens_rx) = attribute_parser();
        let src = r#"#[pmat_work_contract(id = "PMAT-530", require = "R1", ensure = "E1", ensure = "E2")]"#;
        let body = attr.captures(src).unwrap().get(1).unwrap().as_str();
        assert_eq!(
            id_rx.captures(body).unwrap().get(1).unwrap().as_str(),
            "PMAT-530"
        );
        let requires: Vec<&str> = req_rx
            .captures_iter(body)
            .filter_map(|c| c.get(1))
            .map(|m| m.as_str())
            .collect();
        assert_eq!(requires, vec!["R1"]);
        let ensures: Vec<&str> = ens_rx
            .captures_iter(body)
            .filter_map(|c| c.get(1))
            .map(|m| m.as_str())
            .collect();
        assert_eq!(ensures, vec!["E1", "E2"]);
    }

    #[test]
    fn attribute_has_generated_module_skips_when_no_usage() {
        let tmp = tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("src")).unwrap();
        let check = check_attribute_has_generated_module(tmp.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn attribute_has_generated_module_fails_when_file_missing() {
        let tmp = tempdir().unwrap();
        let src = tmp.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(
            src.join("a.rs"),
            r#"#[pmat_work_contract(id = "PMAT-999")] fn f(){}"#,
        )
        .unwrap();
        let check = check_attribute_has_generated_module(tmp.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("PMAT-999"));
    }

    #[test]
    fn attribute_has_generated_module_passes_when_file_present() {
        let tmp = tempdir().unwrap();
        let src = tmp.path().join("src");
        let gen_dir = tmp.path().join("contracts/work");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::create_dir_all(&gen_dir).unwrap();
        std::fs::write(
            src.join("a.rs"),
            r#"#[pmat_work_contract(id = "PMAT-100")] fn f(){}"#,
        )
        .unwrap();
        std::fs::write(gen_dir.join("PMAT-100.rs"), "// generated").unwrap();
        let check = check_attribute_has_generated_module(tmp.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn attribute_clause_ids_exist_skips_without_usage() {
        let tmp = tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("src")).unwrap();
        let check = check_attribute_clause_ids_exist(tmp.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn attribute_clause_ids_exist_fails_on_typo() {
        let tmp = tempdir().unwrap();
        let src = tmp.path().join("src");
        let work = tmp.path().join(".pmat-work/PMAT-100");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::create_dir_all(&work).unwrap();
        std::fs::write(
            src.join("a.rs"),
            r#"#[pmat_work_contract(id = "PMAT-100", require = "R1", ensure = "EX")] fn f(){}"#,
        )
        .unwrap();
        std::fs::write(
            work.join("contract.json"),
            r#"{"require":[{"id":"R1"}],"ensure":[{"id":"E1"}]}"#,
        )
        .unwrap();
        let check = check_attribute_clause_ids_exist(tmp.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("EX"));
    }

    #[test]
    fn attribute_clause_ids_exist_passes_when_all_match() {
        let tmp = tempdir().unwrap();
        let src = tmp.path().join("src");
        let work = tmp.path().join(".pmat-work/PMAT-100");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::create_dir_all(&work).unwrap();
        std::fs::write(
            src.join("a.rs"),
            r#"#[pmat_work_contract(id = "PMAT-100", require = "R1", ensure = "E1")] fn f(){}"#,
        )
        .unwrap();
        std::fs::write(
            work.join("contract.json"),
            r#"{"require":[{"id":"R1"}],"ensure":[{"id":"E1"}]}"#,
        )
        .unwrap();
        let check = check_attribute_clause_ids_exist(tmp.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn expr_binds_to_skips_when_no_expr_in_any_clause() {
        let tmp = tempdir().unwrap();
        let work = tmp.path().join(".pmat-work/PMAT-100");
        std::fs::create_dir_all(&work).unwrap();
        std::fs::write(
            work.join("contract.json"),
            r#"{"require":[{"id":"R1"}],"ensure":[{"id":"E1"}]}"#,
        )
        .unwrap();
        let check = check_expr_clauses_have_binds_to(tmp.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn expr_binds_to_fails_when_missing_binds_to() {
        let tmp = tempdir().unwrap();
        let work = tmp.path().join(".pmat-work/PMAT-100");
        std::fs::create_dir_all(&work).unwrap();
        std::fs::write(
            work.join("contract.json"),
            r#"{"require":[{"id":"R1","expr":"x > 0"}]}"#,
        )
        .unwrap();
        let check = check_expr_clauses_have_binds_to(tmp.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("PMAT-100#R1"));
    }

    #[test]
    fn expr_binds_to_passes_when_present() {
        let tmp = tempdir().unwrap();
        let work = tmp.path().join(".pmat-work/PMAT-100");
        std::fs::create_dir_all(&work).unwrap();
        std::fs::write(
            work.join("contract.json"),
            r#"{"require":[{"id":"R1","expr":"x > 0","binds_to":"crate::f"}]}"#,
        )
        .unwrap();
        let check = check_expr_clauses_have_binds_to(tmp.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn generated_modules_tracked_skips_without_dir() {
        let tmp = tempdir().unwrap();
        let check = check_generated_modules_tracked(tmp.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn generated_modules_tracked_skips_when_dir_empty() {
        let tmp = tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("contracts/work")).unwrap();
        let check = check_generated_modules_tracked(tmp.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }
}
