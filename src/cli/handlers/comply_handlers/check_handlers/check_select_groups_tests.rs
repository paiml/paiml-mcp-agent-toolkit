// Included from check.rs — do NOT add `use` imports or `#!` attributes here.
#[cfg(all(test, not(coverage_nightly)))]
mod tests_select_groups {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    /// A group that counts its runs and emits one check under its first id.
    fn fake(
        name: &'static str,
        ids: &'static [RuleDecl],
        hits: Arc<AtomicUsize>,
    ) -> CheckGroup<'static> {
        let first = ids[0].0;
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
            fake("one", &[("cb-9001", "rule")], hits[0].clone()),
            fake("two", &[("cb-9002", "rule")], hits[1].clone()),
            fake("three", &[("cb-9003", "rule")], hits[2].clone()),
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
            fake("one", &[("cb-9001", "rule")], hits[0].clone()),
            fake("two", &[("cb-9002", "rule")], hits[1].clone()),
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
            fake("one", &[("cb-9001", "rule")], hits[0].clone()),
            fake("two", &[("cb-9002", "rule")], hits[1].clone()),
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
                if !ids.iter().any(|(x, _)| x.eq_ignore_ascii_case(id)) {
                    undeclared.push(format!("{name}: {}", c.name));
                }
            }
        }
        assert!(
            undeclared.is_empty(),
            "rules emitted without a declared id: {undeclared:?}"
        );
    }

    /// ONT-11 (#1422): `comply check --list` prints each rule's declared title,
    /// so the title a group emits must be the one it declares — one name per
    /// rule, from one list. A qualifier in brackets (`Custom Score [x]`) names
    /// the instance, not the rule.
    #[test]
    fn every_emitted_rule_carries_its_declared_title() {
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
        let mut drifted = vec![];
        for (name, rules, run) in compliance_check_groups(d, &cfg, "3.40.0", &overrides) {
            for c in run() {
                let id = check_id(&c.name);
                // A rule with no CB id is named by its whole id (`check_id`).
                let emitted = if id.len() < c.name.len() && id.starts_with("CB-") {
                    &c.name[id.len() + 2..]
                } else {
                    id
                };
                let emitted = emitted.split(" [").next().unwrap_or(emitted);
                if let Some((_, title)) = rules.iter().find(|(x, _)| x.eq_ignore_ascii_case(id)) {
                    if *title != emitted {
                        drifted.push(format!("{name}: {} (declared {title:?})", c.name));
                    }
                }
            }
        }
        assert!(drifted.is_empty(), "rules emitted under another title: {drifted:#?}");
    }

    /// Every declared rule has a non-empty title, and no id is declared twice.
    #[test]
    fn every_declared_rule_has_one_title() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cfg = crate::models::comply_config::ComplyConfig::default();
        let overrides = CheckOverrides::default();
        let mut seen = std::collections::HashSet::new();
        for (name, rules, _) in compliance_check_groups(dir.path(), &cfg, "3.40.0", &overrides) {
            for (id, title) in rules {
                assert!(!title.trim().is_empty(), "{name}: {id} has no title");
                assert!(seen.insert(*id), "{id} is declared twice");
            }
        }
    }

    /// `select_checks` relabels every deselected rule; it must not erase the
    /// reason a rule has no verdict at all (its group was not run). Without this
    /// test nothing reads that message after `select_checks` has run.
    #[test]
    fn select_checks_keeps_the_reason_a_rule_was_not_run() {
        let mut checks = not_run_rows("codegen", &[("cb-1630", "rule")]);
        checks.push(ComplianceCheck {
            name: "CB-2113: Commit Traceability".into(),
            status: CheckStatus::Pass,
            message: "ok".into(),
            severity: Severity::Info,
        });
        select_checks(&mut checks, &["CB-2113".to_string()]).expect("CB-2113 is known");
        let row = checks
            .iter()
            .find(|c| c.name.starts_with("CB-1630"))
            .expect("the CB-1630 row");
        assert_eq!(row.status, CheckStatus::Skip);
        assert!(row.message.contains("was not run"), "{}", row.message);
    }
}
