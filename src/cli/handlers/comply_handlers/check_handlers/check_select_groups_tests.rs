// Included from check.rs — do NOT add `use` imports or `#!` attributes here.
#[cfg(all(test, not(coverage_nightly)))]
mod tests_select_groups {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    /// A group that counts its runs and emits one check under its first id.
    fn fake(
        name: &'static str,
        ids: &'static [&'static str],
        hits: Arc<AtomicUsize>,
    ) -> CheckGroup<'static> {
        let first = ids[0];
        (
            name,
            ids,
            Box::new(move || {
                hits.fetch_add(1, Ordering::SeqCst);
                vec![ComplianceCheck {
                    name: format!("{}: {name} rule", first.to_ascii_uppercase()),
                    status: CheckStatus::Pass,
                    message: "ran".into(),
                    severity: Severity::Info,
                }]
            }),
        )
    }

    fn counters(n: usize) -> Vec<Arc<AtomicUsize>> {
        (0..n).map(|_| Arc::new(AtomicUsize::new(0))).collect()
    }

    /// PMAT-1296: `--checks CB-2113` ran all 21 groups (9.5 minutes in CI) and
    /// only then relabelled the rest. A group holding no selected rule must not run.
    #[test]
    fn a_group_holding_no_selected_rule_is_not_run() {
        let hits = counters(3);
        let groups = vec![
            fake("one", &["cb-9001"], hits[0].clone()),
            fake("two", &["cb-9002"], hits[1].clone()),
            fake("three", &["cb-9003"], hits[2].clone()),
        ];
        let _ = run_check_groups(groups, &["CB-9002".to_string()]);
        let ran: Vec<usize> = hits.iter().map(|h| h.load(Ordering::SeqCst)).collect();
        assert_eq!(ran, vec![0, 1, 0], "only the group holding CB-9002 may run");
    }

    /// A deselected rule is reported, never absent (PMAT-718), even when its
    /// group was not run, and the row says it was not run.
    #[test]
    fn a_skipped_group_still_reports_each_rule_as_not_selected() {
        let hits = counters(2);
        let groups = vec![
            fake("one", &["cb-9001"], hits[0].clone()),
            fake("two", &["cb-9002"], hits[1].clone()),
        ];
        let checks = run_check_groups(groups, &["CB-9002".to_string()]);
        let one = checks
            .iter()
            .find(|c| check_id(&c.name).eq_ignore_ascii_case("cb-9001"))
            .expect("CB-9001 is reported");
        assert_eq!(one.status, CheckStatus::Skip);
        assert!(one.message.contains("not selected"), "{}", one.message);
        assert!(one.message.contains("not run"), "{}", one.message);
    }

    /// No `--checks`: every group runs, exactly once.
    #[test]
    fn with_no_selection_every_group_runs() {
        let hits = counters(2);
        let groups = vec![
            fake("one", &["cb-9001"], hits[0].clone()),
            fake("two", &["cb-9002"], hits[1].clone()),
        ];
        let _ = run_check_groups(groups, &[]);
        assert!(hits.iter().all(|h| h.load(Ordering::SeqCst) == 1));
    }

    /// The declarations are worth only what keeps them true: every real group,
    /// run on a fixture crate, emits only ids it declares. A rule added to a
    /// builder without its id declared would be skipped under `--checks` without
    /// a word; this test names it instead.
    #[test]
    fn every_group_declares_every_rule_it_emits() {
        let dir = tempfile::tempdir().expect("tempdir");
        let d = dir.path();
        std::fs::write(
            d.join("Cargo.toml"),
            "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .expect("write Cargo.toml");
        std::fs::create_dir_all(d.join("src")).expect("mkdir src");
        std::fs::write(d.join("src/lib.rs"), "pub fn f() {}\n").expect("write lib.rs");
        let cfg = crate::models::comply_config::ComplyConfig::default();
        let overrides = CheckOverrides::default();
        let mut undeclared = vec![];
        for (name, ids, run) in compliance_check_groups(d, &cfg, "3.40.0", &overrides) {
            for c in run() {
                let id = check_id(&c.name);
                if !ids.iter().any(|x| x.eq_ignore_ascii_case(id)) {
                    undeclared.push(format!("{name}: {}", c.name));
                }
            }
        }
        assert!(
            undeclared.is_empty(),
            "rules emitted without a declared id: {undeclared:?}"
        );
    }
}
