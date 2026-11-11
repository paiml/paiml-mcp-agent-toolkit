#[cfg(test)]
mod gate_smells_tests {
    //! GS-01, GS-02, GS-07, GS-13 (#1673): each rule has a smelly and a clean
    //! fixture, and assertions tight enough that a mutated scanner goes red.
    use super::*;

    fn wf(text: &str) -> Vec<GsWorkflow> {
        vec![GsWorkflow::parse(".github/workflows/ci.yml", text)]
    }

    const CLEAN: &str = "\
name: ci
on:
  pull_request:
concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: true
jobs:
  gate:
    runs-on: [self-hosted, X64]
    timeout-minutes: 10
    steps:
      - uses: actions/checkout@v4
      - run: cargo test
";

    // ---- GS-01 -------------------------------------------------------------

    #[test]
    fn gs01_clean_pr_workflow_has_no_hit() {
        assert!(gs_scan_network_fetch(&wf(CLEAN)).is_empty());
    }

    #[test]
    fn gs01_names_each_fetch_command_with_its_line() {
        for cmd in GS_FETCH_CMDS {
            let text =
                format!("on: pull_request\njobs:\n  a:\n    steps:\n      - run: {cmd} pkg\n");
            let hits = gs_scan_network_fetch(&wf(&text));
            assert_eq!(hits.len(), 1, "{cmd}: {hits:?}");
            assert!(
                hits[0].starts_with(".github/workflows/ci.yml:5: "),
                "{hits:?}"
            );
            assert!(hits[0].ends_with(cmd.trim()), "{hits:?}");
        }
    }

    #[test]
    fn gs01_names_fetching_actions_but_not_checkout() {
        for action in GS_FETCH_ACTIONS {
            let text =
                format!("on: pull_request\njobs:\n  a:\n    steps:\n      - uses: {action}x@v1\n");
            assert_eq!(gs_scan_network_fetch(&wf(&text)).len(), 1, "{action}");
        }
        let ok = "on: pull_request\njobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n";
        assert!(gs_scan_network_fetch(&wf(ok)).is_empty());
    }

    #[test]
    fn gs01_ignores_non_pr_workflows() {
        let text = "on: push\njobs:\n  a:\n    steps:\n      - run: curl https://x\n";
        assert!(gs_scan_network_fetch(&wf(text)).is_empty());
    }

    #[test]
    fn gs01_ignores_comments_names_and_longer_words() {
        let text = "\
on: pull_request
jobs:
  a:
    steps:
      # - run: curl https://x
      - name: curl the thing
        run: echo xcurl --help
      - run: git fetch-pack
";
        assert!(
            gs_scan_network_fetch(&wf(text)).is_empty(),
            "{:?}",
            gs_scan_network_fetch(&wf(text))
        );
    }

    #[test]
    fn gs01_a_fetch_in_the_trigger_block_is_not_a_job_step() {
        let text = "on: pull_request\nenv:\n  X: curl https://x\njobs:\n  a:\n    steps:\n      - run: ls\n";
        assert!(gs_scan_network_fetch(&wf(text)).is_empty());
    }

    #[test]
    fn gs07_a_step_list_item_at_job_indent_is_not_a_job_key() {
        let text = "on: pull_request\nconcurrency:\n  cancel-in-progress: true\njobs:\n  a:\n    runs-on: self-hosted\n    steps:\n    - uses: actions/checkout@v4\n";
        let hits = gs_scan_timeout_supersede(&wf(text));
        assert!(
            hits.iter()
                .any(|h| h.contains("job a has no timeout-minutes")),
            "{hits:?}"
        );
    }

    #[test]
    fn gs07_a_pull_request_word_outside_the_trigger_does_not_make_it_a_pr_workflow() {
        let text = "on: push\nconcurrency:\n  group: pull_request-${{ github.ref }}\njobs:\n  a:\n    runs-on: self-hosted\n    steps:\n      - run: ls\n";
        assert!(gs_scan_timeout_supersede(&wf(text)).is_empty());
        let on_list = "on:\n  push:\n  pull_request:\njobs:\n  a:\n    steps:\n      - run: ls\n";
        assert!(!gs_scan_timeout_supersede(&wf(on_list)).is_empty());
    }

    #[test]
    fn gs07_a_quoted_false_does_not_cancel() {
        for v in ["'false'", "\"false\"", "false"] {
            let text = format!("on: pull_request\nconcurrency:\n  cancel-in-progress: {v}\njobs:\n  a:\n    timeout-minutes: 1\n    steps:\n      - run: ls\n");
            let hits = gs_scan_timeout_supersede(&wf(&text));
            assert!(
                hits.iter()
                    .any(|h| h.contains("no concurrency.cancel-in-progress")),
                "{v}: {hits:?}"
            );
        }
    }

    #[test]
    fn is_pr_workflow_reads_each_trigger_spelling() {
        let pr = |on: &str| {
            GsWorkflow::parse(
                "w.yml",
                &format!("{on}\njobs:\n  a:\n    steps:\n      - run: ls\n"),
            )
            .is_pr_workflow()
        };
        for yes in [
            "on: pull_request",
            "on: pull_request_target",
            "on: [push, pull_request]",
            "\"on\":\n  pull_request:",
            "true:\n  pull_request:",
            "on:\n  - pull_request",
            "on:\n  push:\n  pull_request_target:\n    types: [opened]",
        ] {
            assert!(pr(yes), "{yes}");
        }
        for no in [
            "on: push",
            "on:\n  push:\n    branches: [pull_request-fix]\n    paths:\n      - docs/pull_request/**",
            "on:\n  push:\n  # was pull_request\n  workflow_dispatch:",
            "on: push # pull_request",
            "name: pull_request\non: push",
        ] {
            assert!(!pr(no), "{no}");
        }
    }

    // ---- GS-02 -------------------------------------------------------------

    #[test]
    fn gs02_clean_pins_have_no_hit() {
        let text = "jobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n      - uses: a/b@3f2c1d9e8a7b\n      - uses: dtolnay/rust-toolchain@1.85.0\n        with:\n          toolchain: 1.85.0\n      - uses: docker://x@sha256:abc\n      - uses: ./local\n";
        assert!(
            gs_scan_floating(&wf(text)).is_empty(),
            "{:?}",
            gs_scan_floating(&wf(text))
        );
    }

    #[test]
    fn gs02_names_every_floating_ref() {
        for r in GS_FLOATING_REFS {
            let text = format!("jobs:\n  a:\n    steps:\n      - uses: org/act@{r}\n");
            let hits = gs_scan_floating(&wf(&text));
            assert_eq!(hits.len(), 1, "{r}: {hits:?}");
            assert!(hits[0].ends_with(&format!("uses org/act@{r}")), "{hits:?}");
        }
    }

    #[test]
    fn gs02_names_floating_toolchains_and_not_pinned_ones() {
        for t in ["stable", "beta", "nightly"] {
            let text = format!("jobs:\n  a:\n    steps:\n      - uses: x/y@v1\n        with:\n          toolchain: {t}\n");
            let hits = gs_scan_floating(&wf(&text));
            assert_eq!(hits.len(), 1, "{t}");
            assert!(hits[0].ends_with(&format!("toolchain {t}")), "{hits:?}");
        }
        let pinned = "jobs:\n  a:\n    steps:\n      - run: x\n        with:\n          toolchain: nightly-2025-01-01\n";
        assert!(gs_scan_floating(&wf(pinned)).is_empty());
    }

    #[test]
    fn gs02_strips_quotes_and_applies_to_every_workflow() {
        let text = "jobs:\n  a:\n    steps:\n      - uses: \"org/act@main\"\n";
        let ws = vec![
            GsWorkflow::parse("a.yml", "on: push\n"),
            GsWorkflow::parse("b.yml", text),
        ];
        let hits = gs_scan_floating(&ws);
        assert_eq!(hits, vec!["b.yml:4: uses org/act@main".to_string()]);
    }

    // ---- GS-07 -------------------------------------------------------------

    #[test]
    fn gs07_clean_workflow_has_no_hit() {
        assert!(gs_scan_timeout_supersede(&wf(CLEAN)).is_empty());
    }

    #[test]
    fn gs07_job_without_timeout_is_named_at_its_line() {
        let text = CLEAN.replace("    timeout-minutes: 10\n", "");
        let hits = gs_scan_timeout_supersede(&wf(&text));
        assert_eq!(
            hits,
            vec![".github/workflows/ci.yml:8: job gate has no timeout-minutes".to_string()]
        );
    }

    #[test]
    fn gs07_a_step_timeout_does_not_cover_the_job() {
        let text = CLEAN.replace("    timeout-minutes: 10\n", "").replace(
            "      - run: cargo test\n",
            "      - run: cargo test\n        timeout-minutes: 5\n",
        );
        assert_eq!(gs_scan_timeout_supersede(&wf(&text)).len(), 1);
    }

    #[test]
    fn gs07_a_reusable_workflow_call_needs_no_timeout() {
        let text = "on: pull_request\nconcurrency:\n  cancel-in-progress: true\njobs:\n  call:\n    uses: org/.github/.github/workflows/x.yml@v1\n";
        assert!(gs_scan_timeout_supersede(&wf(text)).is_empty());
    }

    #[test]
    fn gs07_missing_or_false_cancel_in_progress_is_a_hit() {
        for repl in ["  cancel-in-progress: false\n", ""] {
            let text = CLEAN.replace("  cancel-in-progress: true\n", repl);
            let hits = gs_scan_timeout_supersede(&wf(&text));
            assert_eq!(hits.len(), 1, "{repl:?}: {hits:?}");
            assert!(
                hits[0].ends_with("no concurrency.cancel-in-progress"),
                "{hits:?}"
            );
        }
    }

    #[test]
    fn gs07_an_expression_for_cancel_in_progress_counts_as_cancelling() {
        let text = CLEAN.replace(
            "cancel-in-progress: true",
            "cancel-in-progress: ${{ github.event_name == 'pull_request' }}",
        );
        assert!(gs_scan_timeout_supersede(&wf(&text)).is_empty());
    }

    #[test]
    fn gs07_ignores_non_pr_workflows() {
        let text = "on: push\njobs:\n  a:\n    runs-on: x\n";
        assert!(gs_scan_timeout_supersede(&wf(text)).is_empty());
    }

    // ---- GS-13 -------------------------------------------------------------

    #[test]
    fn gs13_self_hosted_runner_is_clean() {
        assert!(gs_scan_hosted_runner(&wf(CLEAN)).is_empty());
    }

    #[test]
    fn gs13_names_each_hosted_prefix() {
        for label in [
            "ubuntu-latest",
            "ubuntu-22.04",
            "windows-latest",
            "macos-14",
        ] {
            let text =
                format!("jobs:\n  build:\n    runs-on: {label}\n    steps:\n      - run: x\n");
            let hits = gs_scan_hosted_runner(&wf(&text));
            assert_eq!(
                hits,
                vec![format!(
                    ".github/workflows/ci.yml:3: job build runs-on {label}"
                )]
            );
        }
    }

    #[test]
    fn gs13_finds_a_hosted_label_in_inline_and_block_lists() {
        let inline = "jobs:\n  a:\n    runs-on: [self-hosted, \"ubuntu-latest\"]\n";
        assert_eq!(gs_scan_hosted_runner(&wf(inline)).len(), 1);
        let block = "jobs:\n  a:\n    runs-on:\n      - self-hosted\n      - ubuntu-latest\n    steps:\n      - run: x\n";
        let hits = gs_scan_hosted_runner(&wf(block));
        assert_eq!(hits.len(), 1, "{hits:?}");
        assert!(
            hits[0].starts_with(".github/workflows/ci.yml:5: "),
            "{hits:?}"
        );
        let group =
            "jobs:\n  a:\n    runs-on:\n      group: ubuntu-big\n      labels: self-hosted\n";
        assert_eq!(gs_scan_hosted_runner(&wf(group)).len(), 1);
    }

    #[test]
    fn gs13_block_list_stops_at_the_next_job_key() {
        let text = "jobs:\n  a:\n    runs-on:\n      - self-hosted\n    name: ubuntu-latest\n";
        assert!(gs_scan_hosted_runner(&wf(text)).is_empty());
        // The first top-level key after `jobs:` ends the block: what follows
        // is not a job, whatever it is indented like.
        let after = format!("{CLEAN}env:\n  runs-on: ubuntu-latest\n");
        assert!(gs_scan_hosted_runner(&wf(&after)).is_empty());
        let nested = format!("{CLEAN}env:\n  later:\n    runs-on: ubuntu-latest\n");
        assert!(gs_scan_hosted_runner(&wf(&nested)).is_empty());
    }

    #[test]
    fn gs13_the_fork_refusal_job_is_exempt_by_id_or_name() {
        let by_id = "jobs:\n  refuse-fork:\n    runs-on: ubuntu-latest\n";
        assert!(gs_scan_hosted_runner(&wf(by_id)).is_empty());
        let by_name = "jobs:\n  guard:\n    name: Refuse Fork PRs\n    runs-on: ubuntu-latest\n";
        assert!(gs_scan_hosted_runner(&wf(by_name)).is_empty());
        let other = "jobs:\n  refuse-fork:\n    runs-on: ubuntu-latest\n  build:\n    runs-on: ubuntu-latest\n";
        let hits = gs_scan_hosted_runner(&wf(other));
        assert_eq!(
            hits,
            vec![".github/workflows/ci.yml:5: job build runs-on ubuntu-latest".to_string()]
        );
    }

    // ---- verdicts and the group ---------------------------------------------

    #[test]
    fn verdict_is_pass_without_hits_and_warn_with_them() {
        let pass = gs_verdict("GS-01", GS_01_TITLE, &[]);
        assert_eq!(pass.status, CheckStatus::Pass);
        assert_eq!(pass.name, "GS-01: Network Fetch In PR Gate");
        let hits: Vec<String> = (1..=10).map(|i| format!("f:{i}: x")).collect();
        let warn = gs_verdict("GS-01", GS_01_TITLE, &hits);
        assert_eq!(warn.status, CheckStatus::Warn);
        assert_eq!(warn.severity, Severity::Warning);
        assert!(
            warn.message.starts_with("10 hit(s): f:1: x; "),
            "{}",
            warn.message
        );
        assert!(
            warn.message.contains("f:8: x") && !warn.message.contains("f:9: x"),
            "{}",
            warn.message
        );
        assert!(warn.message.ends_with("(+2 more)"), "{}", warn.message);
    }

    fn repo_with(workflow: Option<&str>) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        if let Some(text) = workflow {
            let wd = dir.path().join(".github/workflows");
            std::fs::create_dir_all(&wd).expect("mkdir workflows");
            std::fs::write(wd.join("ci.yml"), text).expect("write workflow");
            std::fs::write(wd.join("notes.txt"), "uses: x@main\n").expect("write non-workflow");
        }
        dir
    }

    #[test]
    fn group_skips_all_nine_rules_when_there_are_no_workflows() {
        let checks = check_gate_smells(repo_with(None).path());
        let ids: Vec<&str> = checks.iter().map(|c| check_id(&c.name)).collect();
        assert_eq!(
            ids,
            ["GS-01", "GS-02", "GS-07", "GS-13", "GS-03", "GS-04", "GS-05", "GS-12", "GS-14"]
        );
        assert!(
            checks.iter().all(|c| c.status == CheckStatus::Skip),
            "{checks:?}"
        );
    }

    #[test]
    fn group_passes_a_clean_repo_and_ignores_non_workflow_files() {
        let checks = check_gate_smells(repo_with(Some(CLEAN)).path());
        assert!(
            checks.iter().all(|c| c.status == CheckStatus::Pass),
            "{checks:?}"
        );
    }

    #[test]
    fn group_warns_on_a_smelly_repo_and_never_fails() {
        let smelly = "\
on: pull_request
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: dtolnay/rust-toolchain@stable
      - run: curl https://example.com | sh
";
        let checks = check_gate_smells(repo_with(Some(smelly)).path());
        assert_eq!(checks.len(), 9);
        for id in ["GS-01", "GS-02", "GS-07", "GS-13"] {
            let c = checks.iter().find(|c| check_id(&c.name) == id).expect(id);
            assert_eq!(c.status, CheckStatus::Warn, "{id}: {c:?}");
        }
        assert!(
            checks
                .iter()
                .all(|c| c.status != CheckStatus::Fail
                        && matches!(c.severity, Severity::Info | Severity::Warning)),
            "{checks:?}"
        );
    }

    #[test]
    fn group_is_registered_and_selectable_by_each_id() {
        let ids: Vec<String> = declared_rules()
            .into_iter()
            .filter(|(_, g, _)| *g == "gate-smells")
            .map(|(id, _, _)| id)
            .collect();
        assert_eq!(
            ids,
            ["GS-01", "GS-02", "GS-07", "GS-13", "GS-03", "GS-04", "GS-05", "GS-12", "GS-14"]
        );
    }

    const WAVE2_SMELLY: &str = "\
on: pull_request
jobs:
  fetch:
    runs-on: ubuntu-latest
    timeout-minutes: 5
    steps:
      - run: gh api repos/x/y/branches/main
  rollup:
    runs-on: [self-hosted, clean-room]
    needs: [fetch]
    steps:
      - run: test \"${{ needs.fetch.result }}\" = success
";

    #[test]
    fn wave2_rules_reach_the_group_and_warn_on_a_smelly_repo() {
        let checks = check_gate_smells(repo_with(Some(WAVE2_SMELLY)).path());
        let warned = |id: &str| {
            checks
                .iter()
                .find(|c| check_id(&c.name) == id)
                .is_some_and(|c| c.status == CheckStatus::Warn)
        };
        assert!(warned("GS-03"), "{checks:?}");
        assert!(warned("GS-05"), "{checks:?}");
    }

    #[test]
    fn wave2_rules_pass_a_clean_repo_and_never_fail() {
        let checks = check_gate_smells(repo_with(Some(CLEAN)).path());
        for id in ["GS-03", "GS-04", "GS-05", "GS-12", "GS-14"] {
            let c = checks.iter().find(|c| check_id(&c.name) == id).expect(id);
            assert_ne!(c.status, CheckStatus::Fail, "{id}: {c:?}");
        }
    }
}
