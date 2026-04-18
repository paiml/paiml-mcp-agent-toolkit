// Tests for CB-1630 codegen-CLI-succeeds, CB-1635 binds_to modified, and
// CB-1636 macros-compile receipts.
// Included from check_codegen.rs — do NOT add `use` imports or `#!` attributes here.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_receipts {
    use super::*;
    use tempfile::tempdir;

    // ── CB-1630 codegen-CLI-succeeds tests ───────────────────────────────

    fn write_codegen_receipt(project: &Path, body: &str) {
        let dir = project.join(".pmat-work").join("codegen");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("last-run.json"), body).unwrap();
    }

    #[test]
    fn cb1630_skips_when_codegen_dir_missing() {
        let tmp = tempdir().unwrap();
        let r = check_codegen_cli_succeeds(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No `.pmat-work/codegen/` directory"));
    }

    #[test]
    fn cb1630_skips_when_receipt_missing() {
        let tmp = tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".pmat-work").join("codegen")).unwrap();
        let r = check_codegen_cli_succeeds(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("last-run.json"));
    }

    #[test]
    fn cb1630_passes_on_success_true() {
        let tmp = tempdir().unwrap();
        write_codegen_receipt(tmp.path(), r#"{"success": true}"#);
        let r = check_codegen_cli_succeeds(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn cb1630_fails_on_success_false() {
        let tmp = tempdir().unwrap();
        write_codegen_receipt(tmp.path(), r#"{"success": false}"#);
        let r = check_codegen_cli_succeeds(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("success=false"));
    }

    #[test]
    fn cb1630_passes_on_exit_code_zero() {
        let tmp = tempdir().unwrap();
        write_codegen_receipt(tmp.path(), r#"{"exit_code": 0}"#);
        let r = check_codegen_cli_succeeds(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn cb1630_fails_on_exit_code_nonzero() {
        let tmp = tempdir().unwrap();
        write_codegen_receipt(tmp.path(), r#"{"exit_code": 1}"#);
        let r = check_codegen_cli_succeeds(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("exit_code=1"));
    }

    #[test]
    fn cb1630_passes_on_status_pass() {
        let tmp = tempdir().unwrap();
        write_codegen_receipt(tmp.path(), r#"{"status": "pass"}"#);
        let r = check_codegen_cli_succeeds(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn cb1630_passes_on_status_ok_or_success() {
        let tmp = tempdir().unwrap();
        write_codegen_receipt(tmp.path(), r#"{"status": "ok"}"#);
        let r1 = check_codegen_cli_succeeds(tmp.path());
        assert_eq!(r1.status, CheckStatus::Pass, "{}", r1.message);

        let tmp2 = tempdir().unwrap();
        write_codegen_receipt(tmp2.path(), r#"{"status": "success"}"#);
        let r2 = check_codegen_cli_succeeds(tmp2.path());
        assert_eq!(r2.status, CheckStatus::Pass, "{}", r2.message);
    }

    #[test]
    fn cb1630_fails_on_status_fail() {
        let tmp = tempdir().unwrap();
        write_codegen_receipt(tmp.path(), r#"{"status": "fail"}"#);
        let r = check_codegen_cli_succeeds(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("status=\"fail\""));
    }

    #[test]
    fn cb1630_skips_on_unknown_schema() {
        let tmp = tempdir().unwrap();
        write_codegen_receipt(tmp.path(), r#"{"unexpected_field": 42}"#);
        let r = check_codegen_cli_succeeds(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip, "{}", r.message);
        assert!(r.message.contains("schema not settled"));
    }

    #[test]
    fn cb1630_fails_on_malformed_json() {
        let tmp = tempdir().unwrap();
        write_codegen_receipt(tmp.path(), "not-json");
        let r = check_codegen_cli_succeeds(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("Malformed JSON"));
    }

    #[test]
    fn cb1630_success_takes_precedence_over_exit_code() {
        // If both keys exist, `success` wins because it's the most explicit.
        let tmp = tempdir().unwrap();
        write_codegen_receipt(tmp.path(), r#"{"success": true, "exit_code": 99}"#);
        let r = check_codegen_cli_succeeds(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    // ── CB-1635 binds_to-function-modified tests ─────────────────────────

    fn write_ticket_contract(project: &Path, ticket: &str, body: &str) {
        let dir = project.join(".pmat-work").join(ticket);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("contract.json"), body).unwrap();
    }

    fn write_modified_files(project: &Path, ticket: &str, body: &str) {
        let dir = project.join(".pmat-work").join(ticket);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("modified-files.json"), body).unwrap();
    }

    #[test]
    fn cb1635_resolves_binds_to_candidates() {
        let c = resolve_binds_to_candidates("crate::a::b::c::func");
        // Most specific first
        assert!(c[0] == "src/a/b/c.rs");
        assert!(c.iter().any(|s| s == "src/a/b/c/mod.rs"));
        assert!(c.iter().any(|s| s == "src/a/b.rs"));
        assert!(c.iter().any(|s| s == "src/a.rs"));
        assert!(c.iter().any(|s| s == "src/lib.rs"));
    }

    #[test]
    fn cb1635_resolves_bare_ident() {
        // `crate::func` has no module prefix — pop leaves parts empty,
        // so only crate-root fallbacks remain.
        let c = resolve_binds_to_candidates("crate::top_level_fn");
        assert!(c.iter().any(|s| s == "src/lib.rs"));
        assert!(c.iter().any(|s| s == "src/main.rs"));
    }

    #[test]
    fn cb1635_skips_when_no_work_dir() {
        let tmp = tempdir().unwrap();
        let r = check_binds_to_function_modified(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No `.pmat-work/`"));
    }

    #[test]
    fn cb1635_skips_when_no_ticket_has_binds_to() {
        let tmp = tempdir().unwrap();
        write_ticket_contract(
            tmp.path(),
            "T-1",
            r#"{"require":[{"id":"R1","expr":"x > 0"}]}"#,
        );
        let r = check_binds_to_function_modified(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip, "{}", r.message);
        assert!(r.message.contains("No ticket has a clause with `binds_to`"));
    }

    #[test]
    fn cb1635_skips_when_no_modified_files_artifact() {
        let tmp = tempdir().unwrap();
        write_ticket_contract(
            tmp.path(),
            "T-1",
            r#"{"require":[{"id":"R1","binds_to":"crate::a::f"}]}"#,
        );
        // no modified-files.json
        let r = check_binds_to_function_modified(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip, "{}", r.message);
        assert!(r.message.contains("modified-files.json"));
    }

    #[test]
    fn cb1635_passes_when_binds_to_matches_modified_file() {
        let tmp = tempdir().unwrap();
        write_ticket_contract(
            tmp.path(),
            "T-1",
            r#"{"require":[{"id":"R1","binds_to":"crate::a::b::f"}]}"#,
        );
        write_modified_files(
            tmp.path(),
            "T-1",
            r#"{"files":["src/a/b.rs","src/other.rs"]}"#,
        );
        let r = check_binds_to_function_modified(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn cb1635_fails_when_binds_to_target_untouched() {
        let tmp = tempdir().unwrap();
        write_ticket_contract(
            tmp.path(),
            "T-1",
            r#"{"require":[{"id":"R1","binds_to":"crate::a::b::f"}]}"#,
        );
        write_modified_files(tmp.path(), "T-1", r#"{"files":["src/elsewhere.rs"]}"#);
        let r = check_binds_to_function_modified(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("T-1"));
        assert!(r.message.contains("R1"));
    }

    #[test]
    fn cb1635_accepts_mod_rs_candidate() {
        let tmp = tempdir().unwrap();
        write_ticket_contract(
            tmp.path(),
            "T-1",
            r#"{"require":[{"id":"R1","binds_to":"crate::a::b::f"}]}"#,
        );
        // File is at src/a/b/mod.rs not src/a/b.rs — resolver should find it
        write_modified_files(tmp.path(), "T-1", r#"{"files":["src/a/b/mod.rs"]}"#);
        let r = check_binds_to_function_modified(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn cb1635_accepts_top_level_array_shape() {
        let tmp = tempdir().unwrap();
        write_ticket_contract(
            tmp.path(),
            "T-1",
            r#"{"require":[{"id":"R1","binds_to":"crate::a::f"}]}"#,
        );
        // Plain array shape
        write_modified_files(tmp.path(), "T-1", r#"["src/a.rs"]"#);
        let r = check_binds_to_function_modified(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn cb1635_accepts_modified_key_shape() {
        let tmp = tempdir().unwrap();
        write_ticket_contract(
            tmp.path(),
            "T-1",
            r#"{"require":[{"id":"R1","binds_to":"crate::a::f"}]}"#,
        );
        write_modified_files(tmp.path(), "T-1", r#"{"modified":["src/a.rs"]}"#);
        let r = check_binds_to_function_modified(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn cb1635_aggregates_across_clauses_and_tickets() {
        let tmp = tempdir().unwrap();
        // T-GOOD: binds_to target matches modified file
        write_ticket_contract(
            tmp.path(),
            "T-GOOD",
            r#"{"require":[{"id":"R1","binds_to":"crate::a::f"}]}"#,
        );
        write_modified_files(tmp.path(), "T-GOOD", r#"{"files":["src/a.rs"]}"#);

        // T-BAD: binds_to target untouched
        write_ticket_contract(
            tmp.path(),
            "T-BAD",
            r#"{"ensure":[{"id":"E1","binds_to":"crate::x::f"}]}"#,
        );
        write_modified_files(tmp.path(), "T-BAD", r#"{"files":["src/a.rs"]}"#);

        let r = check_binds_to_function_modified(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("T-BAD"));
        assert!(!r.message.contains("T-GOOD"), "{}", r.message);
    }

    #[test]
    fn cb1635_binds_to_in_ensure_and_invariant_sections() {
        let tmp = tempdir().unwrap();
        write_ticket_contract(
            tmp.path(),
            "T-1",
            r#"{"ensure":[{"id":"E1","binds_to":"crate::a::f"}],"invariant":[{"id":"I1","binds_to":"crate::b::g"}]}"#,
        );
        write_modified_files(tmp.path(), "T-1", r#"{"files":["src/a.rs","src/b.rs"]}"#);
        let r = check_binds_to_function_modified(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    // ── CB-1636 macros-compile tests ─────────────────────────────────────

    fn write_compile_status(project: &Path, body: &str) {
        let dir = project.join(".pmat-work").join("codegen");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("compile-status.json"), body).unwrap();
    }

    #[test]
    fn cb1636_skips_when_codegen_dir_missing() {
        let tmp = tempdir().unwrap();
        let r = check_macros_compile_debug_and_release(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No `.pmat-work/codegen/`"));
    }

    #[test]
    fn cb1636_skips_when_receipt_missing() {
        let tmp = tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".pmat-work").join("codegen")).unwrap();
        let r = check_macros_compile_debug_and_release(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("compile-status.json"));
    }

    #[test]
    fn cb1636_passes_on_nested_object_both_success() {
        let tmp = tempdir().unwrap();
        write_compile_status(
            tmp.path(),
            r#"{"debug":{"success":true},"release":{"success":true}}"#,
        );
        let r = check_macros_compile_debug_and_release(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn cb1636_fails_when_debug_fails() {
        let tmp = tempdir().unwrap();
        write_compile_status(
            tmp.path(),
            r#"{"debug":{"success":false},"release":{"success":true}}"#,
        );
        let r = check_macros_compile_debug_and_release(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("debug"));
    }

    #[test]
    fn cb1636_fails_when_release_fails() {
        let tmp = tempdir().unwrap();
        write_compile_status(
            tmp.path(),
            r#"{"debug":{"success":true},"release":{"success":false}}"#,
        );
        let r = check_macros_compile_debug_and_release(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("release"));
    }

    #[test]
    fn cb1636_passes_on_exit_code_shape() {
        let tmp = tempdir().unwrap();
        write_compile_status(tmp.path(), r#"{"debug":0,"release":0}"#);
        let r = check_macros_compile_debug_and_release(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn cb1636_fails_on_nonzero_exit_code() {
        let tmp = tempdir().unwrap();
        write_compile_status(tmp.path(), r#"{"debug":0,"release":1}"#);
        let r = check_macros_compile_debug_and_release(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("release"));
        assert!(r.message.contains("exit_code=1"));
    }

    #[test]
    fn cb1636_passes_on_flat_success_shape() {
        let tmp = tempdir().unwrap();
        write_compile_status(
            tmp.path(),
            r#"{"debug_success":true,"release_success":true}"#,
        );
        let r = check_macros_compile_debug_and_release(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn cb1636_fails_on_flat_failure() {
        let tmp = tempdir().unwrap();
        write_compile_status(
            tmp.path(),
            r#"{"debug_success":true,"release_success":false}"#,
        );
        let r = check_macros_compile_debug_and_release(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("release"));
    }

    #[test]
    fn cb1636_passes_on_status_strings() {
        let tmp = tempdir().unwrap();
        write_compile_status(
            tmp.path(),
            r#"{"debug":{"status":"pass"},"release":{"status":"ok"}}"#,
        );
        let r = check_macros_compile_debug_and_release(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn cb1636_fails_when_missing_one_profile() {
        // release key is absent → "no evidence" for release → Fail because
        // we cannot attest to both profiles.
        let tmp = tempdir().unwrap();
        write_compile_status(tmp.path(), r#"{"debug":{"success":true}}"#);
        let r = check_macros_compile_debug_and_release(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("release"));
        assert!(r.message.contains("no evidence"));
    }

    #[test]
    fn cb1636_skips_when_schema_unknown() {
        let tmp = tempdir().unwrap();
        write_compile_status(tmp.path(), r#"{"foo":1,"bar":2}"#);
        let r = check_macros_compile_debug_and_release(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip, "{}", r.message);
        assert!(r.message.contains("schema not settled"));
    }

    #[test]
    fn cb1636_fails_on_malformed_json() {
        let tmp = tempdir().unwrap();
        write_compile_status(tmp.path(), "definitely-not-json");
        let r = check_macros_compile_debug_and_release(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("Malformed JSON"));
    }
}
