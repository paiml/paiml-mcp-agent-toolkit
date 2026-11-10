#[cfg(test)]
mod runner_smells_tests {
    //! RS-01..RS-12 (#1673): parity with paiml/.github `scripts/runner-smells.sh`.
    //! `tests/fixtures/runner-smells/<case>/expected.tsv` holds what that script prints
    //! for the case's workflows; the Rust port must print the same lines.
    use super::*;

    fn fixtures() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/runner-smells")
    }

    fn tsv(found: &[RsFinding]) -> String {
        found.iter().map(|f| format!("{}\n", f.tsv())).collect()
    }

    #[test]
    fn every_fixture_matches_the_script_byte_for_byte() {
        let mut cases = 0;
        for entry in std::fs::read_dir(fixtures()).expect("fixtures dir") {
            let dir = entry.expect("entry").path();
            let expected = std::fs::read_to_string(dir.join("expected.tsv")).expect("expected.tsv");
            let found = rs_scan(&dir).expect("fixture has workflows");
            assert_eq!(tsv(&found), expected, "{}", dir.display());
            cases += 1;
        }
        assert_eq!(cases, 14, "bad-rs01..12 + good + good-nightly");
    }

    #[test]
    fn each_bad_fixture_raises_exactly_its_own_rule() {
        for n in 1..=12 {
            let id = format!("RS-{n:02}");
            let dir = fixtures().join(format!("bad-rs{n:02}"));
            let found = rs_scan(&dir).expect("scanned");
            assert_eq!(
                found.iter().map(|f| f.rule).collect::<Vec<_>>(),
                [id.as_str()],
                "{id}"
            );
        }
    }

    #[test]
    fn clean_fixtures_have_no_findings_and_nightly_mutation_is_allowed() {
        assert!(rs_scan(&fixtures().join("good"))
            .expect("scanned")
            .is_empty());
        assert!(rs_scan(&fixtures().join("good-nightly"))
            .expect("scanned")
            .is_empty());
    }

    #[test]
    fn a_clean_workflow_past_the_pipe_buffer_stays_clean() {
        let good = std::fs::read_to_string(fixtures().join("good/.github/workflows/ci.yml"))
            .expect("good");
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join(".github/workflows")).expect("mkdir");
        let big = format!("{good}{}", "# padding line past the buffer\n".repeat(4000));
        assert!(big.len() > 64 * 1024);
        std::fs::write(dir.path().join(".github/workflows/ci.yml"), big).expect("write");
        assert!(rs_scan(dir.path()).expect("scanned").is_empty());
    }

    #[test]
    fn no_workflows_means_nothing_scanned_and_every_rule_skips() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert!(rs_scan(dir.path()).is_none());
        let checks = check_runner_smells(dir.path());
        assert_eq!(checks.len(), 12);
        assert!(checks.iter().all(|c| c.status == CheckStatus::Skip));
    }

    #[test]
    fn the_group_emits_twelve_ordered_warn_level_checks() {
        let checks = check_runner_smells(&fixtures().join("bad-rs03"));
        let ids: Vec<&str> = checks.iter().map(|c| check_id(&c.name)).collect();
        let want: Vec<String> = (1..=12).map(|n| format!("RS-{n:02}")).collect();
        assert_eq!(ids, want.iter().map(String::as_str).collect::<Vec<_>>());
        let warn: Vec<&str> = checks
            .iter()
            .filter(|c| c.status == CheckStatus::Warn)
            .map(|c| check_id(&c.name))
            .collect();
        assert_eq!(warn, ["RS-03"]);
        assert!(checks.iter().all(|c| c.status != CheckStatus::Fail));
        assert!(
            checks[2].message.starts_with("1 hit(s): ci.yml:"),
            "{}",
            checks[2].message
        );
    }

    // ---- boundaries the fixtures do not reach ---------------------------------------

    fn scan_one(text: &str) -> Vec<String> {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join(".github/workflows")).expect("mkdir");
        std::fs::write(dir.path().join(".github/workflows/w.yml"), text).expect("write");
        rs_scan(dir.path())
            .expect("scanned")
            .iter()
            .map(RsFinding::tsv)
            .collect()
    }

    #[test]
    fn rs01_false_cancel_is_a_hit_and_an_expression_is_not() {
        let base = "on: pull_request\nconcurrency:\n  cancel-in-progress: V\njobs:\n  a:\n    runs-on: self-hosted\n    timeout-minutes: 1\n    steps:\n      - run: ls\n";
        let hit = |v: &str| {
            scan_one(&base.replace('V', v))
                .iter()
                .any(|l| l.starts_with("RS-01\t"))
        };
        assert!(hit("false"));
        assert!(!hit("true"));
        assert!(!hit("${{ github.event_name == 'pull_request' }}"));
    }

    #[test]
    fn rs02_exempts_reusable_workflow_jobs_and_names_the_job_line() {
        let text = "on: push\njobs:\n  call:\n    uses: o/r/.github/workflows/x.yml@abc\n  plain:\n    runs-on: self-hosted\n";
        assert_eq!(
            scan_one(text).iter().filter(|l| l.starts_with("RS-02\t")).collect::<Vec<_>>(),
            ["RS-02\tw.yml:5\tjob 'plain' has no timeout-minutes: a hung job holds its runner for 6 h"]
        );
    }

    #[test]
    fn rs03_finds_hosted_labels_in_block_lists_only_on_pr_workflows() {
        let on = |t: &str| {
            format!("on: {t}\njobs:\n  a:\n    timeout-minutes: 1\n    runs-on:\n      - self-hosted\n      - ubuntu-22.04\n")
        };
        assert!(scan_one(&on("pull_request"))
            .iter()
            .any(|l| l.starts_with("RS-03\tw.yml:3\t")));
        assert!(!scan_one(&on("push")).iter().any(|l| l.starts_with("RS-03")));
    }

    #[test]
    fn rs05_reports_the_first_pull_request_target_line_only() {
        let text = "on:\n  pull_request_target:\n  push:\n# pull_request_target in a comment\n";
        let found = scan_one(text);
        let rs05: Vec<&String> = found.iter().filter(|l| l.starts_with("RS-05")).collect();
        assert_eq!(rs05.len(), 1);
        assert!(rs05[0].starts_with("RS-05\tw.yml:2\t"), "{rs05:?}");
    }

    #[test]
    fn rs08_accepts_a_forty_hex_sha_docker_and_local_actions() {
        let sha = "a".repeat(40);
        let text = format!(
            "on: push\njobs:\n  a:\n    timeout-minutes: 1\n    steps:\n      - uses: o/a@{sha} # v1\n      - uses: docker://alpine:3\n      - uses: ./local\n      - uses: o/b@v4\n      - uses: o/c@{}\n",
            "b".repeat(39)
        );
        let found: Vec<String> = scan_one(&text)
            .into_iter()
            .filter(|l| l.starts_with("RS-08"))
            .collect();
        assert_eq!(found.len(), 2, "{found:?}");
        assert!(
            found[0].ends_with("not pinned to a commit sha: o/b@v4"),
            "{found:?}"
        );
    }

    #[test]
    fn rs10_fires_above_three_pull_request_workflows_and_not_at_three() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join(".github/workflows")).expect("mkdir");
        let write = |n: usize| {
            for i in 0..n {
                std::fs::write(
                    dir.path().join(format!(".github/workflows/w{i}.yml")),
                    "on: pull_request\n",
                )
                .expect("write");
            }
        };
        write(3);
        assert!(!rs_scan(dir.path())
            .expect("scanned")
            .iter()
            .any(|f| f.rule == "RS-10"));
        write(4);
        let found = rs_scan(dir.path()).expect("scanned");
        let rs10: Vec<&RsFinding> = found.iter().filter(|f| f.rule == "RS-10").collect();
        assert_eq!(rs10.len(), 1);
        assert!(rs10[0]
            .message
            .starts_with("4 workflow files start on pull_request (limit 3)"));
    }

    #[test]
    fn rs11_ignores_clippy_and_check_but_not_test_all_targets() {
        let on = |cmd: &str| {
            format!("on: pull_request\nenv:\n  CARGO_PROFILE_DEV_DEBUG: 0\n  CARGO_INCREMENTAL: 0\njobs:\n  a:\n    timeout-minutes: 1\n    steps:\n      - run: {cmd}\n")
        };
        assert!(!scan_one(&on("cargo clippy --all-targets"))
            .iter()
            .any(|l| l.starts_with("RS-11")));
        assert!(scan_one(&on("cargo test --all-targets"))
            .iter()
            .any(|l| l.starts_with("RS-11\tw.yml:9\t")));
    }

    #[test]
    fn rs12_allows_schedule_and_dispatch_only_workflows() {
        let on = |t: &str| {
            format!("on:\n{t}\njobs:\n  m:\n    timeout-minutes: 1\n    steps:\n      - run: cargo mutants --in-diff x\n")
        };
        assert!(!scan_one(&on(
            "  schedule:\n    - cron: '0 3 * * *'\n  workflow_dispatch:"
        ))
        .iter()
        .any(|l| l.starts_with("RS-12")));
        assert!(
            scan_one(&on("  schedule:\n    - cron: '0 3 * * *'\n  push:"))
                .iter()
                .any(|l| l.starts_with("RS-12"))
        );
    }
}
