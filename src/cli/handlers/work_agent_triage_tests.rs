// Tests for `pmat work triage` (ULTRA-003).

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod work_agent_triage_tests {
    use super::*;
    use crate::cli::commands::QaOutputFormat;

    fn now() -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }

    fn record(examined: u32, acted: u32) -> TriageRecord {
        new_triage_record("alpha", "round-5 dogfood findings", examined, acted, now())
    }

    // ---- the accounting rule ------------------------------------------------

    #[test]
    fn a_pass_that_acted_on_everything_it_examined_accounts() {
        assert!(record(7, 7).audit().is_empty());
        assert!(record(0, 0).audit().is_empty());
    }

    #[test]
    fn the_real_defect_reproduced_39_examined_7_filed_32_dropped() {
        // The measured failure: an agent triaged 39 findings and filed 7,
        // dropping 32 without saying so. Nothing it printed was false.
        let bare = record(39, 7);
        let defects = bare.audit();
        assert_eq!(bare.gap(), 32);
        assert!(
            defects.iter().any(|d| d.contains("32 item(s)")),
            "the 32 dropped items must be named as the problem, got: {defects:?}"
        );
        assert!(
            defects.iter().any(|d| d.contains("no --reason")),
            "a gap with no reason is its own defect, got: {defects:?}"
        );
    }

    #[test]
    fn a_partial_deferred_list_is_still_a_silent_drop() {
        let mut r = record(39, 7);
        r.deferred = (0..5).map(|i| format!("F-{i}")).collect();
        r.reason = Some("out of budget".into());
        let defects = r.audit();
        assert_eq!(defects.len(), 1, "got: {defects:?}");
        assert!(defects[0].contains("27 item(s) would"), "got: {defects:?}");
    }

    #[test]
    fn a_fully_named_gap_accounts() {
        let mut r = record(39, 7);
        r.deferred = (0..32).map(|i| format!("F-{i}")).collect();
        r.reason = Some("deferred to round 6: all are docs-only".into());
        assert!(r.audit().is_empty(), "got: {:?}", r.audit());
    }

    #[test]
    fn acting_on_more_than_was_examined_is_refused() {
        let defects = record(3, 9).audit();
        assert!(
            defects.iter().any(|d| d.contains("exceeds examined")),
            "got: {defects:?}"
        );
    }

    #[test]
    fn an_empty_scope_cannot_be_audited() {
        let mut r = record(5, 5);
        r.scope = "   ".into();
        assert!(r.audit().iter().any(|d| d.contains("scope is empty")));
    }

    #[test]
    fn a_whitespace_only_reason_does_not_count_as_a_reason() {
        let mut r = record(2, 1);
        r.deferred = vec!["F-1".into()];
        r.reason = Some("   ".into());
        assert!(
            r.audit().iter().any(|d| d.contains("no --reason")),
            "blank prose must not discharge the obligation"
        );
    }

    // ---- handlers -----------------------------------------------------------

    async fn record_pass(
        dir: &Path,
        examined: u32,
        acted: u32,
        deferred: Vec<String>,
        reason: Option<&str>,
        work_item: Option<&str>,
    ) -> Result<()> {
        handle_work_triage_record(
            "alpha".to_string(),
            "round-5 dogfood findings".to_string(),
            examined,
            acted,
            deferred,
            reason.map(String::from),
            work_item.map(String::from),
            QaOutputFormat::Text,
            Some(dir.to_path_buf()),
        )
        .await
    }

    #[tokio::test]
    async fn recording_an_unaccounted_pass_is_refused_and_writes_nothing() {
        let tmp = tempfile::TempDir::new().expect("tmp");
        let err = record_pass(tmp.path(), 39, 7, vec![], None, Some("ULTRA-003"))
            .await
            .expect_err("39 examined / 7 acted with nothing named must not be recordable");
        assert!(err.to_string().contains("unaccounted pass"), "got: {err}");
        assert!(
            TriageLedger::new(tmp.path())
                .load_records()
                .expect("load")
                .is_empty(),
            "a refused record must not reach the journal"
        );
    }

    #[tokio::test]
    async fn recording_an_accounted_pass_persists_it() {
        let tmp = tempfile::TempDir::new().expect("tmp");
        let deferred: Vec<String> = (0..32).map(|i| format!("F-{i}")).collect();
        record_pass(
            tmp.path(),
            39,
            7,
            deferred,
            Some("docs-only, deferred to round 6"),
            Some("ULTRA-003"),
        )
        .await
        .expect("an accounted pass records");

        let records = TriageLedger::new(tmp.path()).load_records().expect("load");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].deferred.len(), 32);
        assert_eq!(records[0].work_item_id.as_deref(), Some("ULTRA-003"));
    }

    #[tokio::test]
    async fn verify_fails_when_the_ticket_has_no_triage_record_at_all() {
        let tmp = tempfile::TempDir::new().expect("tmp");
        record_pass(tmp.path(), 3, 3, vec![], None, Some("OTHER-1"))
            .await
            .expect("record for a different ticket");

        let err = handle_work_triage_verify(
            Some("ULTRA-003".to_string()),
            None,
            QaOutputFormat::Text,
            Some(tmp.path().to_path_buf()),
        )
        .await
        .expect_err("unstated coverage must not pass as complete coverage");
        assert!(err.to_string().contains("never stated"), "got: {err}");
    }

    #[tokio::test]
    async fn verify_passes_once_every_examined_item_is_accounted_for() {
        let tmp = tempfile::TempDir::new().expect("tmp");
        record_pass(tmp.path(), 3, 3, vec![], None, Some("ULTRA-003"))
            .await
            .expect("record");
        handle_work_triage_verify(
            Some("ULTRA-003".to_string()),
            None,
            QaOutputFormat::Json,
            Some(tmp.path().to_path_buf()),
        )
        .await
        .expect("an accounted ticket verifies");
    }

    #[tokio::test]
    async fn verify_catches_an_unaccounted_record_edited_into_the_journal() {
        // `record` refuses at write time, so the only way an unaccounted line
        // exists is a hand edit. `verify` must still find it.
        let tmp = tempfile::TempDir::new().expect("tmp");
        let ledger = TriageLedger::new(tmp.path());
        let mut r = record(39, 7);
        r.work_item_id = Some("ULTRA-003".into());
        ledger.append(&r).expect("append");

        let err = handle_work_triage_verify(
            Some("ULTRA-003".to_string()),
            None,
            QaOutputFormat::Text,
            Some(tmp.path().to_path_buf()),
        )
        .await
        .expect_err("a hand-edited silent drop must still fail the gate");
        assert!(err.to_string().contains("do not account"), "got: {err}");
    }

    #[tokio::test]
    async fn verify_filtered_by_agent_narrows_what_is_measured() {
        let tmp = tempfile::TempDir::new().expect("tmp");
        record_pass(tmp.path(), 4, 4, vec![], None, None)
            .await
            .expect("record");
        handle_work_triage_verify(
            None,
            Some("alpha".to_string()),
            QaOutputFormat::Text,
            Some(tmp.path().to_path_buf()),
        )
        .await
        .expect("alpha has a record");
        let err = handle_work_triage_verify(
            None,
            Some("beta".to_string()),
            QaOutputFormat::Text,
            Some(tmp.path().to_path_buf()),
        )
        .await
        .expect_err("beta measured nothing, so beta must not verify clean");
        assert!(err.to_string().contains("never stated"), "got: {err}");
    }

    #[test]
    fn a_corrupt_triage_line_is_an_error_not_a_skipped_line() {
        let tmp = tempfile::TempDir::new().expect("tmp");
        std::fs::create_dir_all(tmp.path().join(".pmat-work")).expect("mkdir");
        std::fs::write(
            tmp.path().join(".pmat-work").join("triage.jsonl"),
            "{\"id\":\"tr-1\"}\n",
        )
        .expect("write");
        let err = TriageLedger::new(tmp.path())
            .load_records()
            .expect_err("skipping the line would hide exactly what this gate hunts");
        assert!(err.to_string().contains("line 1"), "got: {err}");
    }

    #[test]
    fn a_verification_with_zero_records_is_never_ok() {
        assert!(!TriageVerification::default().ok());
        assert!(!verify_triage_records(&[]).ok());
        assert!(verify_triage_records(&[record(5, 5)]).ok());
    }
}
