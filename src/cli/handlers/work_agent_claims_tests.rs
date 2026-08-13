// Tests for `pmat work claim` (ULTRA-002).

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod work_agent_claims_tests {
    use super::*;
    use crate::cli::commands::QaOutputFormat;

    fn t(secs: i64) -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::parse_from_rfc3339("2026-08-13T00:00:00Z")
            .expect("static timestamp must parse")
            .with_timezone(&chrono::Utc)
            + chrono::Duration::seconds(secs)
    }

    fn acquire(agent: &str, paths: &[&str], at: i64, ttl: i64) -> FileClaimRecord {
        let mut r = new_acquire_record(
            agent,
            paths.iter().map(|p| p.to_string()).collect(),
            ttl as u64,
            t(at),
        );
        r.id = format!("cl-{agent}-{at}");
        r
    }

    fn release(agent: &str, paths: &[&str], at: i64) -> FileClaimRecord {
        new_release_record(agent, paths.iter().map(|p| p.to_string()).collect(), t(at))
    }

    // ---- path normalization -------------------------------------------------

    #[test]
    fn normalize_strips_dot_slash_and_trailing_separators() {
        let root = Path::new(".");
        assert_eq!(
            normalize_claim_path("./src/cli/handlers/foo.rs", root).expect("ok"),
            "src/cli/handlers/foo.rs"
        );
        assert_eq!(
            normalize_claim_path("src/cli/handlers/", root).expect("ok"),
            "src/cli/handlers"
        );
    }

    #[test]
    fn normalize_refuses_globs_dotdot_and_whole_repo() {
        let root = Path::new(".");
        for bad in ["src/**/*.rs", "src/../etc", "  ", ".", "./"] {
            assert!(
                normalize_claim_path(bad, root).is_err(),
                "'{bad}' must be refused, not silently widened"
            );
        }
    }

    #[test]
    fn normalize_makes_absolute_paths_repo_relative_and_refuses_outsiders() {
        let tmp = tempfile::TempDir::new().expect("tmp");
        std::fs::create_dir_all(tmp.path().join("src")).expect("mkdir");
        let inside = tmp.path().join("src").join("lib.rs");
        assert_eq!(
            normalize_claim_path(&inside.to_string_lossy(), tmp.path()).expect("ok"),
            "src/lib.rs"
        );
        let err = normalize_claim_path("/etc/passwd", tmp.path())
            .expect_err("a path outside the project is not claimable here");
        assert!(
            err.to_string().contains("outside the project root"),
            "got: {err}"
        );
    }

    // ---- overlap ------------------------------------------------------------

    #[test]
    fn overlap_is_component_wise_not_string_prefix() {
        assert!(claim_paths_overlap("src/cli", "src/cli"));
        assert!(claim_paths_overlap("src/cli", "src/cli/handlers/a.rs"));
        assert!(claim_paths_overlap("src/cli/handlers/a.rs", "src/cli"));
        // The bug a naive starts_with() would ship: two disjoint directories
        // sharing a textual prefix would deadlock every agent that touched them.
        assert!(!claim_paths_overlap("src/cli", "src/cli_x"));
        assert!(!claim_paths_overlap("src/a.rs", "src/ab.rs"));
    }

    // ---- fold ---------------------------------------------------------------

    #[test]
    fn second_agent_cannot_take_a_live_claim_but_gets_it_after_release() {
        let held = fold_claims(
            &[
                acquire("alpha", &["src/a.rs"], 0, 3600),
                acquire("beta", &["src/a.rs"], 10, 3600),
            ],
            t(20),
        );
        assert_eq!(held.len(), 1, "one path, one owner");
        assert_eq!(held[0].agent, "alpha", "file order settles the winner");

        let after = fold_claims(
            &[
                acquire("alpha", &["src/a.rs"], 0, 3600),
                acquire("beta", &["src/a.rs"], 10, 3600),
                release("alpha", &["src/a.rs"], 20),
                acquire("beta", &["src/a.rs"], 30, 3600),
            ],
            t(40),
        );
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].agent, "beta");
    }

    #[test]
    fn a_directory_claim_blocks_a_file_beneath_it() {
        let held = fold_claims(
            &[
                acquire("alpha", &["src/cli/handlers"], 0, 3600),
                acquire("beta", &["src/cli/handlers/foo.rs"], 5, 3600),
            ],
            t(10),
        );
        assert_eq!(held.len(), 1);
        assert_eq!(held[0].agent, "alpha");
        assert_eq!(held[0].path, "src/cli/handlers");
    }

    #[test]
    fn a_lapsed_claim_frees_the_path_and_stays_visible() {
        let records = [acquire("alpha", &["src/a.rs"], 0, 60)];
        let live = fold_claims(&records, t(30));
        assert!(!live[0].expired, "still inside the TTL");
        let lapsed = fold_claims(&records, t(120));
        assert!(
            lapsed[0].expired,
            "a crashed agent's claim must lapse, and must still be listed as lapsed"
        );

        let taken = fold_claims(
            &[
                acquire("alpha", &["src/a.rs"], 0, 60),
                acquire("beta", &["src/a.rs"], 120, 3600),
            ],
            t(130),
        );
        assert_eq!(
            taken.len(),
            1,
            "the superseded lapsed row must not remain beside its replacement"
        );
        assert_eq!(
            taken[0].agent, "beta",
            "beta takes over once alpha's TTL runs out"
        );
    }

    #[test]
    fn an_unparseable_expiry_counts_as_lapsed_not_as_forever() {
        assert!(is_expired("", t(0)));
        assert!(is_expired("not-a-timestamp", t(0)));
    }

    // ---- handlers -----------------------------------------------------------

    async fn acquire_for(dir: &Path, agent: &str, paths: &[&str]) -> Result<()> {
        handle_work_claim_acquire(
            paths.iter().map(|p| p.to_string()).collect(),
            agent.to_string(),
            3600,
            None,
            None,
            None,
            QaOutputFormat::Json,
            Some(dir.to_path_buf()),
        )
        .await
    }

    #[tokio::test]
    async fn acquire_then_a_second_agent_is_refused_and_claims_nothing() {
        let tmp = tempfile::TempDir::new().expect("tmp");
        acquire_for(tmp.path(), "alpha", &["src/a.rs", "src/b.rs"])
            .await
            .expect("first claim is granted");

        let err = acquire_for(tmp.path(), "beta", &["src/b.rs", "src/c.rs"])
            .await
            .expect_err("overlapping claim must exit non-zero");
        assert!(
            err.to_string().contains("held by another agent"),
            "got: {err}"
        );

        // All-or-nothing: beta must not have quietly taken the free path.
        let ledger = FileClaimLedger::new(tmp.path());
        let held = ledger.active_claims(chrono::Utc::now()).expect("fold");
        assert_eq!(held.len(), 2, "only alpha's two paths are held");
        assert!(held.iter().all(|c| c.agent == "alpha"));
    }

    #[tokio::test]
    async fn an_agent_may_refresh_its_own_claim() {
        let tmp = tempfile::TempDir::new().expect("tmp");
        acquire_for(tmp.path(), "alpha", &["src/a.rs"])
            .await
            .expect("first");
        acquire_for(tmp.path(), "alpha", &["src/a.rs"])
            .await
            .expect("re-claiming your own path is a refresh, not a conflict");
    }

    #[tokio::test]
    async fn losing_the_append_race_rolls_the_claim_back() {
        // The window `confirm_or_yield` exists to close: our line reaches the
        // journal *after* a competitor's, so our write succeeded and our claim
        // did not. Constructed by appending both lines directly.
        let tmp = tempfile::TempDir::new().expect("tmp");
        let ledger = FileClaimLedger::new(tmp.path());
        let winner = acquire("alpha", &["src/a.rs"], 0, 3600);
        let loser = acquire("beta", &["src/a.rs"], 0, 3600);
        ledger.append(&winner).expect("append winner");
        ledger.append(&loser).expect("append loser");

        let err = confirm_or_yield(&ledger, &loser, &["src/a.rs".to_string()], t(1))
            .expect_err("the loser must not believe its own write");
        assert!(
            err.to_string().contains("lost an append race"),
            "got: {err}"
        );

        let records = ledger.load_records().expect("load");
        assert_eq!(records.len(), 3, "a rollback release is journalled");
        assert_eq!(records[2].action, FileClaimAction::Release);
        assert_eq!(records[2].agent, "beta");
    }

    #[tokio::test]
    async fn releasing_a_path_nobody_holds_is_an_error_not_a_no_op() {
        let tmp = tempfile::TempDir::new().expect("tmp");
        let err = handle_work_claim_release(
            vec!["src/a.rs".to_string()],
            "alpha".to_string(),
            false,
            None,
            QaOutputFormat::Text,
            Some(tmp.path().to_path_buf()),
        )
        .await
        .expect_err("a release that frees nothing must not report success");
        assert!(err.to_string().contains("no active claim"), "got: {err}");
    }

    #[tokio::test]
    async fn release_all_frees_only_the_calling_agents_paths() {
        let tmp = tempfile::TempDir::new().expect("tmp");
        acquire_for(tmp.path(), "alpha", &["src/a.rs"])
            .await
            .expect("a");
        acquire_for(tmp.path(), "beta", &["src/b.rs"])
            .await
            .expect("b");
        handle_work_claim_release(
            vec![],
            "alpha".to_string(),
            true,
            None,
            QaOutputFormat::Text,
            Some(tmp.path().to_path_buf()),
        )
        .await
        .expect("release --all");

        let held = FileClaimLedger::new(tmp.path())
            .active_claims(chrono::Utc::now())
            .expect("fold");
        assert_eq!(held.len(), 1);
        assert_eq!(held[0].agent, "beta");
    }

    #[tokio::test]
    async fn releasing_another_agents_path_needs_force_and_a_reason() {
        let tmp = tempfile::TempDir::new().expect("tmp");
        acquire_for(tmp.path(), "alpha", &["src/a.rs"])
            .await
            .expect("a");
        let err = handle_work_claim_release(
            vec!["src/a.rs".to_string()],
            "beta".to_string(),
            false,
            None,
            QaOutputFormat::Text,
            Some(tmp.path().to_path_buf()),
        )
        .await
        .expect_err("beta cannot quietly free alpha's path");
        assert!(err.to_string().contains("--force --reason"), "got: {err}");

        handle_work_claim_release(
            vec!["src/a.rs".to_string()],
            "beta".to_string(),
            false,
            Some("alpha crashed; verified its PID is gone".to_string()),
            QaOutputFormat::Text,
            Some(tmp.path().to_path_buf()),
        )
        .await
        .expect("forced release with a reason is allowed");
        assert!(FileClaimLedger::new(tmp.path())
            .active_claims(chrono::Utc::now())
            .expect("fold")
            .is_empty());
    }

    #[tokio::test]
    async fn check_exits_non_zero_only_for_paths_owned_by_someone_else() {
        let tmp = tempfile::TempDir::new().expect("tmp");
        acquire_for(tmp.path(), "alpha", &["src/cli/handlers"])
            .await
            .expect("a");

        handle_work_claim_check(
            vec!["src/cli/handlers/foo.rs".to_string()],
            Some("alpha".to_string()),
            QaOutputFormat::Text,
            Some(tmp.path().to_path_buf()),
        )
        .await
        .expect("the owner sees its own path as free");

        let err = handle_work_claim_check(
            vec!["src/cli/handlers/foo.rs".to_string()],
            Some("beta".to_string()),
            QaOutputFormat::Text,
            Some(tmp.path().to_path_buf()),
        )
        .await
        .expect_err("beta must be told the path is taken");
        assert!(
            err.to_string().contains("claimed by another agent"),
            "got: {err}"
        );
    }

    #[tokio::test]
    async fn a_missing_project_path_is_an_error_not_an_empty_all_clear() {
        let err = handle_work_claim_list(
            None,
            false,
            QaOutputFormat::Text,
            Some(PathBuf::from("/does/not/exist/pmat-claims-test")),
        )
        .await
        .expect_err("a mistyped -p must not report an empty, conflict-free pool");
        assert!(err.to_string().contains("does not exist"), "got: {err}");
    }

    #[tokio::test]
    async fn a_zero_ttl_claim_is_refused() {
        let tmp = tempfile::TempDir::new().expect("tmp");
        let err = handle_work_claim_acquire(
            vec!["src/a.rs".to_string()],
            "alpha".to_string(),
            0,
            None,
            None,
            None,
            QaOutputFormat::Text,
            Some(tmp.path().to_path_buf()),
        )
        .await
        .expect_err("a claim that never lapses would strand the pool");
        assert!(err.to_string().contains("--ttl 0"), "got: {err}");
    }

    #[test]
    fn a_corrupt_journal_line_is_an_error_not_a_skipped_line() {
        let tmp = tempfile::TempDir::new().expect("tmp");
        std::fs::create_dir_all(tmp.path().join(".pmat-work")).expect("mkdir");
        std::fs::write(
            tmp.path().join(".pmat-work").join("claims.jsonl"),
            "{\"not\":\"a claim\"}\n",
        )
        .expect("write");
        let err = FileClaimLedger::new(tmp.path())
            .load_records()
            .expect_err("dropping the line would under-report conflicts");
        assert!(err.to_string().contains("line 1"), "got: {err}");
    }

    #[test]
    fn an_over_long_claim_is_refused_rather_than_torn() {
        let tmp = tempfile::TempDir::new().expect("tmp");
        let paths: Vec<&str> = Vec::new();
        let mut record = acquire("alpha", &paths, 0, 3600);
        record.paths = (0..500)
            .map(|i| format!("src/generated/file_{i:04}.rs"))
            .collect();
        let err = FileClaimLedger::new(tmp.path())
            .append(&record)
            .expect_err("a line past PIPE_BUF can interleave with another agent's");
        assert!(err.to_string().contains("atomic-append"), "got: {err}");
    }
}
