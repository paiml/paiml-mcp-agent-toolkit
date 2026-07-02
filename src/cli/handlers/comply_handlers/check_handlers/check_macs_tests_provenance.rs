// Included from check_macs.rs — do NOT add `use` imports or `#!` attributes here.

#[cfg(all(test, not(coverage_nightly)))]
mod tests_macs_provenance {
    use super::*;
    use serde_json::json;
    use tempfile::tempdir;

    fn write_receipt(project: &Path, ticket: &str, receipt: serde_json::Value) {
        let dir = project.join(".pmat-work").join(ticket).join("falsification");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("receipt-2026-07-02T00-00-00.json"),
            serde_json::to_string_pretty(&receipt).unwrap(),
        )
        .unwrap();
    }

    fn write_events(project: &Path, ticket: &str, lines: &[serde_json::Value]) {
        let dir = project.join(".pmat-work").join(ticket);
        std::fs::create_dir_all(&dir).unwrap();
        let body: String = lines
            .iter()
            .map(|l| format!("{l}\n"))
            .collect();
        std::fs::write(dir.join("events.jsonl"), body).unwrap();
    }

    #[test]
    fn cb1651_skips_without_v2_receipts() {
        let project = tempdir().unwrap();
        std::fs::create_dir_all(project.path().join(".pmat-work")).unwrap();
        let check = check_receipt_provenance_present(project.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn cb1651_ignores_v1_receipts() {
        let project = tempdir().unwrap();
        write_receipt(
            project.path(),
            "T-1",
            json!({"id": "r1", "work_item_id": "T-1", "content_hash": "x"}),
        );
        let check = check_receipt_provenance_present(project.path());
        assert_eq!(check.status, CheckStatus::Skip, "v1 receipts are exempt");
    }

    #[test]
    fn cb1651_red_on_v2_receipt_without_agent() {
        let project = tempdir().unwrap();
        write_receipt(
            project.path(),
            "T-2",
            json!({"schema_version": 2, "id": "r2", "work_item_id": "T-2"}),
        );
        let check = check_receipt_provenance_present(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("no agent"));
    }

    #[test]
    fn cb1651_green_when_v2_receipts_carry_agent() {
        let project = tempdir().unwrap();
        write_receipt(
            project.path(),
            "T-3",
            json!({
                "schema_version": 2, "id": "r3", "work_item_id": "T-3",
                "agent": {"model": "claude-fable-5", "effort": "high",
                          "harness": "claude_code", "source": "declared"}
            }),
        );
        let check = check_receipt_provenance_present(project.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn cb1654_skips_without_journals() {
        let project = tempdir().unwrap();
        std::fs::create_dir_all(project.path().join(".pmat-work")).unwrap();
        let check = check_refusal_events_acked(project.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn cb1654_red_on_unacked_refusal() {
        let project = tempdir().unwrap();
        write_events(
            project.path(),
            "T-4",
            &[json!({"id": "ev-1", "recorded_at": "t",
                     "event": {"type": "refusal", "at": "t"}})],
        );
        let check = check_refusal_events_acked(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("ev-1"));
    }

    #[test]
    fn cb1654_green_after_ack() {
        let project = tempdir().unwrap();
        write_events(
            project.path(),
            "T-5",
            &[
                json!({"id": "ev-1", "recorded_at": "t",
                       "event": {"type": "refusal", "at": "t"}}),
                json!({"id": "ev-2", "recorded_at": "t",
                       "event": {"type": "ack", "at": "t", "ack_of": "ev-1",
                                 "reason": "root cause: phrasing; rephrased"}}),
            ],
        );
        let check = check_refusal_events_acked(project.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn cb1654_model_switch_never_blocks() {
        let project = tempdir().unwrap();
        write_events(
            project.path(),
            "T-6",
            &[json!({"id": "ev-1", "recorded_at": "t",
                     "event": {"type": "model_switch", "at": "t",
                               "from": "claude-fable-5", "to": "claude-opus-4-8"}})],
        );
        let check = check_refusal_events_acked(project.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }
}
