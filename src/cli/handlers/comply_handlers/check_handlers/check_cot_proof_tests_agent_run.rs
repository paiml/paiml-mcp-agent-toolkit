// Tests for CB-1644 (agent run replayable) and CB-1646 (CoT derivation SHA).
// Included from `check_cot_proof.rs` — do NOT add `use` imports or `#!` attributes here.

#[cfg(all(test, not(coverage_nightly)))]
mod tests_cot_agent_run {
    use super::*;
    use serde_json::json;
    use tempfile::tempdir;

    fn write_contract(project: &Path, ticket: &str, contract: Value) {
        let dir = project.join(".pmat-work").join(ticket);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("contract.json"),
            serde_json::to_string_pretty(&contract).unwrap(),
        )
        .unwrap();
    }

    // ── CB-1644 agent run replayable tests ───────────────────────────────

    fn write_agent_run(project: &Path, ticket: &str, run_id: &str, body: &str) {
        let dir = project.join(".pmat-work").join(ticket).join("agent-runs");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{}.json", run_id)), body).unwrap();
    }

    #[test]
    fn agent_run_skip_when_no_work_dir() {
        let project = tempdir().unwrap();
        let check = check_agent_run_replayable(project.path());
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(check.message.contains("`.pmat-work/`"));
    }

    #[test]
    fn agent_run_skip_when_no_agent_runs_dirs() {
        let project = tempdir().unwrap();
        write_contract(project.path(), "T1", json!({ "work_item_id": "T1" }));
        let check = check_agent_run_replayable(project.path());
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(check.message.contains("agent-runs"));
    }

    #[test]
    fn agent_run_pass_with_full_schema() {
        let project = tempdir().unwrap();
        write_agent_run(
            project.path(),
            "T1",
            "run-001",
            r#"{
                "prompt_sha": "abc123",
                "tool_calls": [{"name": "Read"}, {"name": "Edit"}],
                "commit_sha": "deadbeef"
            }"#,
        );
        let check = check_agent_run_replayable(project.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
        assert!(check.message.contains("1 agent run"));
    }

    #[test]
    fn agent_run_fail_when_field_missing() {
        let project = tempdir().unwrap();
        write_agent_run(
            project.path(),
            "T1",
            "run-001",
            r#"{
                "prompt_sha": "abc123",
                "tool_calls": []
            }"#,
        );
        let check = check_agent_run_replayable(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("commit_sha"));
        assert!(check.message.contains("T1:run-001"));
    }

    #[test]
    fn agent_run_fail_when_null_field() {
        let project = tempdir().unwrap();
        write_agent_run(
            project.path(),
            "T1",
            "run-001",
            r#"{
                "prompt_sha": null,
                "tool_calls": [],
                "commit_sha": "dead"
            }"#,
        );
        let check = check_agent_run_replayable(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("prompt_sha"));
    }

    #[test]
    fn agent_run_fail_when_tool_calls_not_array() {
        let project = tempdir().unwrap();
        write_agent_run(
            project.path(),
            "T1",
            "run-001",
            r#"{
                "prompt_sha": "a",
                "tool_calls": "should-be-array",
                "commit_sha": "b"
            }"#,
        );
        let check = check_agent_run_replayable(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("tool_calls is not an array"));
    }

    #[test]
    fn agent_run_fail_when_malformed_json() {
        let project = tempdir().unwrap();
        write_agent_run(project.path(), "T1", "run-001", "{ not json");
        let check = check_agent_run_replayable(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("unreadable"));
        assert!(check.message.contains("T1:run-001"));
    }

    #[test]
    fn agent_run_fail_when_top_level_not_object() {
        let project = tempdir().unwrap();
        write_agent_run(project.path(), "T1", "run-001", "[1, 2, 3]");
        let check = check_agent_run_replayable(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("not an object"));
    }

    #[test]
    fn agent_run_ignores_non_json_files() {
        let project = tempdir().unwrap();
        // A stray README in agent-runs/ doesn't count toward checked_runs
        let runs_dir = project.path().join(".pmat-work/T1/agent-runs");
        std::fs::create_dir_all(&runs_dir).unwrap();
        std::fs::write(runs_dir.join("README.md"), "notes").unwrap();
        let check = check_agent_run_replayable(project.path());
        // No *.json files present → treat as Skip (no runs emitted)
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn agent_run_ignores_ledger_and_hidden_dirs() {
        let project = tempdir().unwrap();
        std::fs::create_dir_all(project.path().join(".pmat-work/ledger/agent-runs")).unwrap();
        std::fs::create_dir_all(project.path().join(".pmat-work/.hidden/agent-runs")).unwrap();
        // Write valid runs under both — they should be skipped by the check
        std::fs::write(
            project.path().join(".pmat-work/ledger/agent-runs/x.json"),
            r#"{"prompt_sha":"a","tool_calls":[],"commit_sha":"b"}"#,
        )
        .unwrap();
        let check = check_agent_run_replayable(project.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn agent_run_pass_across_multiple_tickets() {
        let project = tempdir().unwrap();
        write_agent_run(
            project.path(),
            "T1",
            "run-001",
            r#"{"prompt_sha":"a","tool_calls":[],"commit_sha":"b"}"#,
        );
        write_agent_run(
            project.path(),
            "T2",
            "run-001",
            r#"{"prompt_sha":"c","tool_calls":[],"commit_sha":"d"}"#,
        );
        let check = check_agent_run_replayable(project.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("2 agent run"));
    }

    // ── CB-1646 CoT derivation SHA tests ─────────────────────────────────

    fn write_digest(project: &Path, ticket: &str, body: &str) {
        let dir = project.join(".pmat-work").join(ticket);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("cot-digest.json"), body).unwrap();
    }

    #[test]
    fn cot_digest_skips_when_no_digest_files() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({ "chain_of_thought": [{ "step": 1, "question": "q", "answer": "a" }] }),
        );
        let check = check_cot_derivation_sha_fresh(project.path());
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(check.message.contains("cot-digest.json"));
    }

    #[test]
    fn cot_digest_passes_when_sha_matches() {
        let project = tempdir().unwrap();
        let contract = json!({
            "chain_of_thought": [
                { "id": "CoT-1", "assumption": { "predicate": "p" } }
            ]
        });
        write_contract(project.path(), "T1", contract.clone());
        let expected = canonical_cot_sha(&contract);
        write_digest(
            project.path(),
            "T1",
            &format!("{{\"sha\": \"{}\"}}", expected),
        );
        let check = check_cot_derivation_sha_fresh(project.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn cot_digest_accepts_digest_key_alias() {
        let project = tempdir().unwrap();
        let contract = json!({ "chain_of_thought": [{ "id": "CoT-1" }] });
        write_contract(project.path(), "T1", contract.clone());
        let expected = canonical_cot_sha(&contract);
        // Alternate naming — `digest` instead of `sha`
        write_digest(
            project.path(),
            "T1",
            &format!("{{\"digest\": \"{}\"}}", expected),
        );
        let check = check_cot_derivation_sha_fresh(project.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn cot_digest_fails_when_sha_mismatches() {
        let project = tempdir().unwrap();
        write_contract(
            project.path(),
            "T1",
            json!({ "chain_of_thought": [{ "id": "CoT-1", "assumption": { "predicate": "p" } }] }),
        );
        // Simulate someone editing the CoT after derivation: record an old SHA
        write_digest(project.path(), "T1", "{\"sha\": \"deadbeef\"}");
        let check = check_cot_derivation_sha_fresh(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("T1"));
        assert!(check.message.contains("deadbeef"));
        assert!(check.message.contains("pmat work cot derive"));
    }

    #[test]
    fn cot_digest_fails_on_malformed_digest_file() {
        let project = tempdir().unwrap();
        write_contract(project.path(), "T1", json!({ "chain_of_thought": [] }));
        write_digest(project.path(), "T1", "{ not valid json");
        let check = check_cot_derivation_sha_fresh(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("unreadable"));
        assert!(check.message.contains("T1"));
    }

    #[test]
    fn cot_digest_case_insensitive_hex_match() {
        // Recorded digest is uppercase; canonical is lowercase — must still match.
        let project = tempdir().unwrap();
        let contract = json!({ "chain_of_thought": [{ "id": "CoT-1" }] });
        write_contract(project.path(), "T1", contract.clone());
        let expected = canonical_cot_sha(&contract).to_uppercase();
        write_digest(
            project.path(),
            "T1",
            &format!("{{\"sha\": \"{}\"}}", expected),
        );
        let check = check_cot_derivation_sha_fresh(project.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn cot_digest_mixed_coverage_checks_only_tickets_with_digest() {
        let project = tempdir().unwrap();
        // T1 has a digest; T2 does not — T2 is silently skipped.
        let contract1 = json!({ "chain_of_thought": [{ "id": "CoT-1" }] });
        write_contract(project.path(), "T1", contract1.clone());
        write_digest(
            project.path(),
            "T1",
            &format!("{{\"sha\": \"{}\"}}", canonical_cot_sha(&contract1)),
        );
        write_contract(project.path(), "T2", json!({ "chain_of_thought": [] }));
        let check = check_cot_derivation_sha_fresh(project.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
        // Only one ticket was checked
        assert!(check.message.contains("1 ticket"));
    }

    #[test]
    fn canonical_cot_sha_is_deterministic() {
        let a = json!({
            "chain_of_thought": [
                { "id": "CoT-1", "assumption": { "predicate": "p" } }
            ]
        });
        let b = json!({
            "chain_of_thought": [
                { "assumption": { "predicate": "p" }, "id": "CoT-1" }
            ]
        });
        // BTreeMap key ordering means differently-authored JSON with the
        // same semantic content hashes identically.
        assert_eq!(canonical_cot_sha(&a), canonical_cot_sha(&b));
    }
}
