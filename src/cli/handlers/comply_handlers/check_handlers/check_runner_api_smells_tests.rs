// Included from check.rs — do NOT add `use` imports or `#!` attributes here.
#[cfg(all(test, not(coverage_nightly)))]
mod tests_runner_api_smells {
    use super::*;
    use crate::models::comply_config::ComplyConfig;
    use crate::services::runner_api_smells::{ActionsApi, RunSample, Window};
    use chrono::{DateTime, Duration, TimeZone, Utc};

    struct Fixture(Result<Window, String>);
    impl ActionsApi for Fixture {
        fn window(&self, _repo: &str, _since: DateTime<Utc>) -> Result<Window, String> {
            self.0.clone()
        }
    }

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 10, 12, 0, 0).unwrap()
    }

    fn healthy() -> Window {
        let since = now() - Duration::hours(24);
        let runs = (0..4)
            .map(|_| RunSample { created_at: since, started_at: Some(since + Duration::seconds(5)), conclusion: Some("success".into()) })
            .collect();
        Window { repo: "o/r".into(), since, runs_total: 4, runs, merged_prs: 2 }
    }

    fn rows(config: &ComplyConfig, repo: Result<Option<String>, String>, api: Result<Window, String>) -> Vec<ComplianceCheck> {
        build_runner_api_smells_checks(config, repo, &Fixture(api), now())
    }

    fn some_repo() -> Result<Option<String>, String> {
        Ok(Some("o/r".to_string()))
    }

    #[test]
    fn a_healthy_window_passes_all_three_named_rows() {
        let out = rows(&on(), some_repo(), Ok(healthy()));
        let names: Vec<&str> = out.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["RS-13: Runs per Merge", "RS-14: Cancelled Run Share", "RS-15: Run Queue Time p95"]);
        assert!(out.iter().all(|c| c.status == CheckStatus::Pass), "{out:?}");
    }

    #[test]
    fn a_runs_per_merge_over_30_fails_with_error_severity_and_over_15_warns() {
        let mut w = healthy();
        w.runs_total = 400;
        let out = rows(&on(), some_repo(), Ok(w.clone()));
        assert_eq!(out[0].status, CheckStatus::Fail);
        assert_eq!(out[0].severity, Severity::Error);
        w.runs_total = 40;
        let out = rows(&on(), some_repo(), Ok(w));
        assert_eq!(out[0].status, CheckStatus::Warn);
    }

    #[test]
    fn a_failed_read_is_not_measured_on_every_row_never_a_pass() {
        let out = rows(&on(), some_repo(), Err("gh api failed: HTTP 403 rate limit".into()));
        assert_eq!(out.len(), 3);
        for c in &out {
            assert_eq!(c.status, CheckStatus::Fail, "{c:?}");
            assert!(c.message.starts_with("not_measured:"), "{}", c.message);
            assert!(c.message.contains("rate limit"), "{}", c.message);
        }
    }

    #[test]
    fn no_resolvable_repository_skips_and_a_failed_lookup_is_not_measured() {
        let skipped = rows(&on(), Ok(None), Ok(healthy()));
        assert!(skipped.iter().all(|c| c.status == CheckStatus::Skip), "{skipped:?}");
        let failed = rows(&on(), Err("no origin".into()), Ok(healthy()));
        assert!(failed.iter().all(|c| c.status == CheckStatus::Fail && c.message.starts_with("not_measured:")), "{failed:?}");
    }

    fn on() -> ComplyConfig {
        config_from("comply:\n  checks:\n    rs-13:\n      enabled: true\n    rs-14:\n      enabled: true\n    rs-15:\n      enabled: true\n")
    }

    fn config_from(yaml: &str) -> ComplyConfig {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join(".pmat.yaml"), yaml).expect("write .pmat.yaml");
        PmatYamlConfig::load(dir.path()).expect(".pmat.yaml parses").comply
    }

    #[test]
    fn a_tightened_option_turns_a_pass_into_a_fail_and_a_bad_option_is_not_measured() {
        let tight = config_from("comply:\n  checks:\n    rs-13:\n      enabled: true\n      options:\n        runs_per_merge_warn: 0.5\n        runs_per_merge_fail: 1.0\n");
        let out = rows(&tight, some_repo(), Ok(healthy()));
        assert_eq!(out[0].status, CheckStatus::Fail, "{}", out[0].message);
        let bad = config_from("comply:\n  checks:\n    rs-13:\n      enabled: true\n      options:\n        runs_per_merge_warn: lots\n");
        let out = rows(&bad, some_repo(), Ok(healthy()));
        assert!(out[0].status == CheckStatus::Fail && out[0].message.contains("runs_per_merge_warn"), "{out:?}");
    }

    #[test]
    fn a_disabled_row_skips_and_the_others_still_judge() {
        let cfg = config_from("comply:\n  checks:\n    rs-13:\n      enabled: true\n    rs-14:\n      enabled: false\n    rs-15:\n      enabled: true\n");
        let out = rows(&cfg, some_repo(), Ok(healthy()));
        assert_eq!(out[1].status, CheckStatus::Skip);
        assert_eq!(out[0].status, CheckStatus::Pass);
        assert_eq!(out[2].status, CheckStatus::Pass);
    }

    #[test]
    fn the_default_config_leaves_the_api_rules_off_so_a_plain_comply_stays_offline() {
        let out = rows(&ComplyConfig::default(), some_repo(), Ok(healthy()));
        assert!(out.iter().all(|c| c.status == CheckStatus::Skip), "{out:?}");
    }

    #[test]
    fn the_group_declares_the_three_ids_and_is_selectable() {
        let ids: Vec<&str> = GROUP_IDS_RUNNER_API_SMELLS.iter().map(|(id, _)| *id).collect();
        assert_eq!(ids, ["rs-13", "rs-14", "rs-15"]);
    }
}
