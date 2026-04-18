// Tests for CB-1618 (Level Monotonicity). Included into
// `check_work_ladder.rs`.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_work_ladder_monotonicity {
    use super::*;
    use tempfile::tempdir;

    // ─── CB-1618: level monotonicity across checkpoints ──────────────────────

    fn write_checkpoint(project: &Path, id: &str, filename: &str, body: &str) {
        let dir = project.join(".pmat-work").join(id).join("checkpoints");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(filename), body).unwrap();
    }

    fn write_downgrade_ledger(project: &Path, body: &str) {
        let dir = project.join(".pmat-work").join("ledger");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("downgrades.json"), body).unwrap();
    }

    fn checkpoint_body(ts: &str, level: Option<&str>) -> String {
        match level {
            Some(l) => format!(
                "{{\"timestamp\": \"{}\", \"verification_level\": \"{}\"}}",
                ts, l
            ),
            None => format!("{{\"timestamp\": \"{}\"}}", ts),
        }
    }

    #[test]
    fn monotonicity_skips_without_pmat_work() {
        let tmp = tempdir().unwrap();
        let r = check_ladder_monotonicity(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No `.pmat-work/`"));
    }

    #[test]
    fn monotonicity_skips_with_no_checkpoint_dirs() {
        let tmp = tempdir().unwrap();
        make_contract("T-1", "L3").save(tmp.path()).unwrap();
        let r = check_ladder_monotonicity(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r
            .message
            .contains("No ticket has a non-empty `checkpoints/`"));
    }

    #[test]
    fn monotonicity_skips_when_checkpoints_lack_level_field() {
        let tmp = tempdir().unwrap();
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-a.json",
            &checkpoint_body("2026-04-01T10:00:00Z", None),
        );
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-b.json",
            &checkpoint_body("2026-04-01T11:00:00Z", None),
        );
        let r = check_ladder_monotonicity(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("verification_level"));
    }

    #[test]
    fn monotonicity_ignores_ticket_with_one_leveled_checkpoint() {
        // A single leveled checkpoint can't demonstrate regression — ignored.
        let tmp = tempdir().unwrap();
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-a.json",
            &checkpoint_body("2026-04-01T10:00:00Z", Some("L3")),
        );
        let r = check_ladder_monotonicity(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
    }

    #[test]
    fn monotonicity_passes_on_ascending_levels() {
        let tmp = tempdir().unwrap();
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-a.json",
            &checkpoint_body("2026-04-01T10:00:00Z", Some("L1")),
        );
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-b.json",
            &checkpoint_body("2026-04-01T11:00:00Z", Some("L3")),
        );
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-c.json",
            &checkpoint_body("2026-04-01T12:00:00Z", Some("L4")),
        );
        let r = check_ladder_monotonicity(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
        assert!(r.message.contains("1 ticket(s) monotonic"));
    }

    #[test]
    fn monotonicity_passes_on_flat_levels() {
        let tmp = tempdir().unwrap();
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-a.json",
            &checkpoint_body("2026-04-01T10:00:00Z", Some("L3")),
        );
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-b.json",
            &checkpoint_body("2026-04-01T11:00:00Z", Some("L3")),
        );
        let r = check_ladder_monotonicity(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass);
    }

    #[test]
    fn monotonicity_fails_on_regression_without_ledger() {
        let tmp = tempdir().unwrap();
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-a.json",
            &checkpoint_body("2026-04-01T10:00:00Z", Some("L3")),
        );
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-b.json",
            &checkpoint_body("2026-04-01T11:00:00Z", Some("L1")),
        );
        let r = check_ladder_monotonicity(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("T-1"));
        assert!(r.message.contains("L3 → L1"));
    }

    #[test]
    fn monotonicity_passes_on_regression_with_ledger() {
        let tmp = tempdir().unwrap();
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-a.json",
            &checkpoint_body("2026-04-01T10:00:00Z", Some("L4")),
        );
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-b.json",
            &checkpoint_body("2026-04-01T11:00:00Z", Some("L3")),
        );
        write_downgrade_ledger(
            tmp.path(),
            r#"[{"ticket":"T-1","reason":"kani runner offline for review cycle"}]"#,
        );
        let r = check_ladder_monotonicity(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn monotonicity_aggregates_across_tickets() {
        // T-1 monotonic, T-2 regresses without ledger → Fail names only T-2.
        let tmp = tempdir().unwrap();
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-a.json",
            &checkpoint_body("2026-04-01T10:00:00Z", Some("L2")),
        );
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-b.json",
            &checkpoint_body("2026-04-01T11:00:00Z", Some("L3")),
        );
        write_checkpoint(
            tmp.path(),
            "T-2",
            "checkpoint-a.json",
            &checkpoint_body("2026-04-01T10:00:00Z", Some("L4")),
        );
        write_checkpoint(
            tmp.path(),
            "T-2",
            "checkpoint-b.json",
            &checkpoint_body("2026-04-01T11:00:00Z", Some("L2")),
        );
        let r = check_ladder_monotonicity(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("T-2"));
        assert!(!r.message.contains("T-1:"));
    }

    #[test]
    fn monotonicity_sorts_by_timestamp_not_filename() {
        // Filename `z.json` is created first chronologically; filename order
        // would read it second and flag a bogus regression. Verify the check
        // sorts by the `timestamp` field.
        let tmp = tempdir().unwrap();
        write_checkpoint(
            tmp.path(),
            "T-1",
            "z-first.json",
            &checkpoint_body("2026-04-01T09:00:00Z", Some("L1")),
        );
        write_checkpoint(
            tmp.path(),
            "T-1",
            "a-second.json",
            &checkpoint_body("2026-04-01T10:00:00Z", Some("L3")),
        );
        let r = check_ladder_monotonicity(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn monotonicity_skips_hidden_and_ledger_dirs() {
        let tmp = tempdir().unwrap();
        // Hidden dir and `ledger` dir must be ignored, even if they happen
        // to contain `checkpoints/`.
        write_checkpoint(
            tmp.path(),
            ".hidden",
            "checkpoint-a.json",
            &checkpoint_body("2026-04-01T10:00:00Z", Some("L5")),
        );
        write_checkpoint(
            tmp.path(),
            ".hidden",
            "checkpoint-b.json",
            &checkpoint_body("2026-04-01T11:00:00Z", Some("L1")),
        );
        write_checkpoint(
            tmp.path(),
            "ledger",
            "checkpoint-a.json",
            &checkpoint_body("2026-04-01T10:00:00Z", Some("L5")),
        );
        write_checkpoint(
            tmp.path(),
            "ledger",
            "checkpoint-b.json",
            &checkpoint_body("2026-04-01T11:00:00Z", Some("L1")),
        );
        let r = check_ladder_monotonicity(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No ticket"));
    }

    #[test]
    fn monotonicity_ignores_malformed_checkpoint_json() {
        let tmp = tempdir().unwrap();
        write_checkpoint(tmp.path(), "T-1", "checkpoint-a.json", "not-a-json-file");
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-b.json",
            &checkpoint_body("2026-04-01T10:00:00Z", Some("L3")),
        );
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-c.json",
            &checkpoint_body("2026-04-01T11:00:00Z", Some("L4")),
        );
        // Malformed row is dropped; the remaining two rows ascend → Pass.
        let r = check_ladder_monotonicity(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn monotonicity_empty_ledger_does_not_audit() {
        let tmp = tempdir().unwrap();
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-a.json",
            &checkpoint_body("2026-04-01T10:00:00Z", Some("L4")),
        );
        write_checkpoint(
            tmp.path(),
            "T-1",
            "checkpoint-b.json",
            &checkpoint_body("2026-04-01T11:00:00Z", Some("L2")),
        );
        write_downgrade_ledger(tmp.path(), "[]");
        let r = check_ladder_monotonicity(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("T-1"));
    }
}
