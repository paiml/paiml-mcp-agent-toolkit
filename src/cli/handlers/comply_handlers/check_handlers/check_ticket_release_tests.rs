// Included from check.rs — do NOT add `use` imports or `#!` attributes here.
//
// PMAT-724 (goal-mode.md §11 step 5): CB-2112 (invariant A) and CB-2114
// (invariants B/F1). Every test names the mutant it dies under in its doc
// comment, so a survivor is a test that should be deleted, not kept.
#[cfg(all(test, not(coverage_nightly)))]
mod tests_ticket_release {
    use super::*;
    use crate::models::comply_config::{CheckSeverity, ComplyConfig};
    use std::path::Path;

    const LINKAGE: &str = "CB-2112: Ticket Linkage";
    const RELEASE: &str = "CB-2114: Release Binding";

    fn iso(when: chrono::DateTime<chrono::Utc>) -> String {
        when.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
    }
    fn days_ago(n: i64) -> String {
        iso(chrono::Utc::now() - chrono::Duration::days(n))
    }

    /// A project whose roadmap names `repo` (or none) and holds `items`.
    fn project(repo: Option<&str>, items: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        let rm = dir.path().join("docs/roadmaps/roadmap.yaml");
        std::fs::create_dir_all(rm.parent().expect("parent")).expect("mkdir");
        let repo_line = repo.map_or("null".to_string(), str::to_string);
        std::fs::write(
            &rm,
            format!("roadmap_version: \"1.0\"\ngithub_enabled: true\ngithub_repo: {repo_line}\nroadmap:\n{items}"),
        )
        .expect("write roadmap");
        dir
    }

    /// One roadmap item. `release` is written only when given, exactly as the
    /// serializer does (`skip_serializing_if`).
    fn item(id: &str, title: &str, status: &str, issue: Option<u64>, release: Option<&str>) -> String {
        let issue = issue.map_or("null".to_string(), |n| n.to_string());
        let release = release.map_or(String::new(), |r| format!("    release: \"{r}\"\n"));
        let updated = days_ago(2);
        format!("  - id: {id}\n    title: {title}\n    status: {status}\n    github_issue: {issue}\n    updated: {updated}\n{release}")
    }

    /// One issue as the snapshot saw it; `milestone` is the milestone TITLE
    /// (the release key, §4.1) or none.
    fn issue(number: u64, title: &str, state: &str, labels: &[&str], milestone: Option<&str>) -> serde_json::Value {
        serde_json::json!({"number": number, "title": title, "state": state, "labels": labels, "milestone": milestone, "updated_at": days_ago(2)})
    }

    /// Write `snapshot.json` carrying `issues` and the open milestones titled
    /// `milestones`; given to the rules on the command line, never through
    /// `.pmat.yaml`.
    fn snapshot(dir: &Path, issues: Vec<serde_json::Value>, milestones: &[&str]) {
        let ms: Vec<serde_json::Value> = milestones.iter().map(|t| serde_json::json!({"title": t, "state": "open"})).collect();
        let snap = serde_json::json!({"repo": "paiml/fixture", "taken_at": "2026-09-10T00:00:00Z", "issues": issues, "milestones": ms});
        std::fs::write(dir.join("snapshot.json"), snap.to_string()).expect("write snapshot");
    }

    fn both(dir: &Path, github_snapshot: Option<&Path>) -> Vec<ComplianceCheck> {
        let config: ComplyConfig = PmatYamlConfig::load(dir).expect(".pmat.yaml parses").comply;
        let checks = build_ticket_release_checks(dir, &config, github_snapshot);
        assert_eq!(checks.len(), 2, "exactly one CB-2112 row and one CB-2114 row");
        checks
    }
    fn pick(mut checks: Vec<ComplianceCheck>, name: &str) -> ComplianceCheck {
        let i = checks.iter().position(|c| c.name == name).unwrap_or_else(|| panic!("no row named {name}"));
        checks.remove(i)
    }
    fn linkage_with(dir: &Path, github_snapshot: Option<&Path>) -> ComplianceCheck {
        pick(both(dir, github_snapshot), LINKAGE)
    }
    fn release_with(dir: &Path, github_snapshot: Option<&Path>) -> ComplianceCheck {
        pick(both(dir, github_snapshot), RELEASE)
    }
    /// The common case: judge from the fixture's `snapshot.json`.
    fn linkage(dir: &Path) -> ComplianceCheck {
        linkage_with(dir, Some(&dir.join("snapshot.json")))
    }
    fn release(dir: &Path) -> ComplianceCheck {
        release_with(dir, Some(&dir.join("snapshot.json")))
    }

    fn git(dir: &Path, args: &[&str]) {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(["-c", "user.name=t", "-c", "user.email=t@example.invalid", "-c", "commit.gpgsign=false"])
            .args(args)
            .env("LC_ALL", "C")
            .output()
            .expect("git runs");
        assert!(out.status.success(), "git failed: {}", String::from_utf8_lossy(&out.stderr));
    }

    /// PMAT-001 ↔ #1, open, on milestone 3.41.0 which exists: both rules pass.
    fn bound() -> tempfile::TempDir {
        let dir = project(Some("paiml/fixture"), &item("PMAT-001", "planned work", "planned", Some(1), Some("3.41.0")));
        snapshot(dir.path(), vec![issue(1, "planned work", "open", &[], Some("3.41.0"))], &["3.41.0"]);
        dir
    }

    fn assert_not_measured(c: &ComplianceCheck, needle: &str) {
        assert_eq!(c.status, CheckStatus::Fail, "{}", c.message);
        assert!(c.message.starts_with("not_measured:"), "{}", c.message);
        assert!(c.message.contains(needle), "{}", c.message);
    }

    // ───────────────────────── shared preamble ─────────────────────────

    /// Mutant: a rule that judges an absent roadmap as a Fail (or a Pass).
    #[test]
    fn a_project_without_a_roadmap_is_skipped_by_both_rules() {
        let dir = tempfile::tempdir().expect("tempdir");
        for c in both(dir.path(), Some(&dir.path().join("snapshot.json"))) {
            assert_eq!(c.status, CheckStatus::Skip, "{}: {}", c.name, c.message);
            assert!(c.message.contains("docs/roadmaps/roadmap.yaml"), "{}", c.message);
        }
    }

    /// Mutant: a rule that reaches for `gh` when nothing names GitHub.
    #[test]
    fn a_roadmap_naming_no_repository_and_no_issues_is_skipped_by_both_rules() {
        let dir = project(None, &item("PMAT-001", "planned work", "planned", None, None));
        for c in both(dir.path(), None) {
            assert_eq!(c.status, CheckStatus::Skip, "{}: {}", c.name, c.message);
            assert!(c.message.contains("GitHub repository"), "{}", c.message);
        }
    }

    /// Mutant: `!path.exists()` → Skip without asking git whether it was committed.
    #[test]
    fn a_committed_then_deleted_roadmap_is_not_measured_by_both_rules() {
        let dir = bound();
        git(dir.path(), &["init", "-q", "-b", "master"]);
        git(dir.path(), &["add", "-A"]);
        git(dir.path(), &["commit", "-q", "-m", "roadmap"]);
        git(dir.path(), &["rm", "-q", "docs/roadmaps/roadmap.yaml"]);
        for c in both(dir.path(), Some(&dir.path().join("snapshot.json"))) {
            assert_not_measured(&c, "committed and is now gone");
        }
    }

    /// Mutant: an unreadable snapshot mapped to Skip — "we could not measure
    /// it" rendered as "it did not regress".
    #[test]
    fn a_snapshot_that_cannot_be_read_is_not_measured_by_both_rules() {
        let dir = project(Some("paiml/fixture"), &item("PMAT-001", "planned work", "planned", Some(1), Some("3.41.0")));
        for c in both(dir.path(), Some(&dir.path().join("snapshot.json"))) {
            assert_not_measured(&c, "snapshot.json");
        }
    }

    /// Mutant: the `.pmat.yaml` `snapshot` key read as a file source — the
    /// bypass token §12 forbids, refused per rule key.
    #[test]
    fn a_snapshot_path_committed_in_pmat_yaml_is_refused_by_each_rule() {
        for key in ["cb-2112", "cb-2114"] {
            let dir = bound();
            std::fs::write(
                dir.path().join(".pmat.yaml"),
                format!("comply:\n  checks:\n    {key}:\n      options:\n        snapshot: snapshot.json\n"),
            )
            .expect("write .pmat.yaml");
            let c = if key == "cb-2112" { linkage(dir.path()) } else { release(dir.path()) };
            assert_not_measured(&c, ".pmat.yaml");
            assert!(c.message.contains("--github-snapshot"), "{}", c.message);
            assert!(c.message.contains(key), "{}", c.message);
        }
    }

    // ───────────────────────── CB-2112 ─────────────────────────

    /// The §7 falsifier: null one `github_issue`. Mutant: verdict inversion.
    #[test]
    fn an_open_item_with_no_issue_fails_linkage() {
        let dir = project(Some("paiml/fixture"), &item("PMAT-001", "planned work", "planned", None, None));
        snapshot(dir.path(), vec![], &[]);
        let c = linkage(dir.path());
        assert_eq!(c.status, CheckStatus::Fail, "{}", c.message);
        assert!(c.message.contains("NO-ISSUE") && c.message.contains("PMAT-001"), "{}", c.message);
        assert!(c.message.contains("--github-issue"), "the fixer is named: {}", c.message);
    }

    /// Mutant: `state` not consulted — a closed issue counted as linked.
    #[test]
    fn an_open_item_naming_a_closed_issue_fails_linkage() {
        let dir = project(Some("paiml/fixture"), &item("PMAT-001", "planned work", "planned", Some(1), None));
        snapshot(dir.path(), vec![issue(1, "planned work", "closed", &[], None)], &[]);
        let c = linkage(dir.path());
        assert_eq!(c.status, CheckStatus::Fail, "{}", c.message);
        assert!(c.message.contains("ISSUE-CLOSED") && c.message.contains("#1"), "{}", c.message);
    }

    /// Mutant: `snapshot.issue(n)` None treated as "nothing to say".
    #[test]
    fn an_open_item_naming_an_absent_issue_fails_linkage() {
        let dir = project(Some("paiml/fixture"), &item("PMAT-001", "planned work", "planned", Some(1), None));
        snapshot(dir.path(), vec![], &[]);
        let c = linkage(dir.path());
        assert_eq!(c.status, CheckStatus::Fail, "{}", c.message);
        assert!(c.message.contains("ISSUE-ABSENT") && c.message.contains("#1"), "{}", c.message);
    }

    /// The clause that makes this invariant A and not a repeat of §5.1: the
    /// issue number must be the item's numeric tail (PMAT-714, #1240 mints
    /// `PMAT-N` from `--github-issue N`). Mutant: the tail clause dropped.
    #[test]
    fn an_open_item_naming_an_open_issue_that_is_not_its_tail_fails_linkage() {
        let dir = project(Some("paiml/fixture"), &item("PMAT-001", "planned work", "planned", Some(7), None));
        snapshot(dir.path(), vec![issue(7, "planned work", "open", &[], None)], &[]);
        let c = linkage(dir.path());
        assert_eq!(c.status, CheckStatus::Fail, "{}", c.message);
        assert!(c.message.contains("TAIL-MISMATCH") && c.message.contains("PMAT-001") && c.message.contains("#7"), "{}", c.message);
    }

    /// Mutant: an id with no digits parsed as tail 0 and #0 never matching —
    /// the message must say the id has no numeric tail, not invent one.
    #[test]
    fn an_id_without_a_numeric_tail_is_named_as_such() {
        let dir = project(Some("paiml/fixture"), &item("EPIC", "planned work", "planned", Some(1), None));
        snapshot(dir.path(), vec![issue(1, "planned work", "open", &[], None)], &[]);
        let c = linkage(dir.path());
        assert_eq!(c.status, CheckStatus::Fail, "{}", c.message);
        assert!(c.message.contains("TAIL-MISMATCH") && c.message.contains("no numeric tail"), "{}", c.message);
    }

    /// Mutant: a `no-roadmap`-labelled issue counted as a link — the label
    /// says no item should name it.
    #[test]
    fn an_open_item_naming_an_excluded_issue_fails_linkage() {
        let dir = project(Some("paiml/fixture"), &item("PMAT-001", "planned work", "planned", Some(1), None));
        snapshot(dir.path(), vec![issue(1, "planned work", "open", &["no-roadmap"], None)], &[]);
        let c = linkage(dir.path());
        assert_eq!(c.status, CheckStatus::Fail, "{}", c.message);
        assert!(c.message.contains("ISSUE-EXCLUDED") && c.message.contains("no-roadmap"), "{}", c.message);
    }

    /// Mutant: verdict inversion (a rule that always fails survives every RED
    /// test above). Two items, two open issues, each numbered as its tail.
    #[test]
    fn every_open_item_linked_to_its_own_open_issue_passes_linkage() {
        let items = format!(
            "{}{}",
            item("PMAT-001", "first", "planned", Some(1), None),
            item("GH-2", "second", "inprogress", Some(2), None)
        );
        let dir = project(Some("paiml/fixture"), &items);
        snapshot(dir.path(), vec![issue(1, "first", "open", &[], None), issue(2, "second", "open", &[], None)], &[]);
        let c = linkage(dir.path());
        assert_eq!(c.status, CheckStatus::Pass, "{}", c.message);
        assert!(c.message.contains("linked 2"), "{}", c.message);
        assert!(c.message.contains("snapshot.json"), "the source is named: {}", c.message);
    }

    /// Mutant: scope widened to terminal items (§4.2 scopes to non-terminal
    /// and says so).
    #[test]
    fn a_completed_item_without_an_issue_is_out_of_scope_for_linkage() {
        let items = format!(
            "{}{}",
            item("PMAT-001", "shipped", "completed", None, None),
            item("PMAT-002", "dropped", "cancelled", Some(9), None)
        );
        let dir = project(Some("paiml/fixture"), &items);
        snapshot(dir.path(), vec![], &[]);
        let c = linkage(dir.path());
        assert_eq!(c.status, CheckStatus::Pass, "{}", c.message);
        assert!(c.message.contains("0 open item"), "{}", c.message);
        assert!(c.message.contains("completed") && c.message.contains("cancelled"), "the scope is stated: {}", c.message);
    }

    /// Mutant: findings capped without saying so — a message that silently
    /// drops the ninth finding.
    #[test]
    fn linkage_findings_beyond_eight_are_counted_not_dropped() {
        let items: String = (1..=10).map(|i| item(&format!("PMAT-{i:03}"), "work", "planned", None, None)).collect();
        let dir = project(Some("paiml/fixture"), &items);
        snapshot(dir.path(), vec![], &[]);
        let c = linkage(dir.path());
        assert_eq!(c.status, CheckStatus::Fail, "{}", c.message);
        assert!(c.message.starts_with("10 finding(s)"), "{}", c.message);
        assert!(c.message.contains("(+2 more)"), "{}", c.message);
    }

    // ───────────────────────── CB-2114 ─────────────────────────

    /// The §7 falsifier: remove one `release:`. Mutant: verdict inversion.
    #[test]
    fn an_open_item_without_a_release_fails_binding() {
        let dir = project(Some("paiml/fixture"), &item("PMAT-001", "planned work", "planned", Some(1), None));
        snapshot(dir.path(), vec![issue(1, "planned work", "open", &[], Some("3.41.0"))], &["3.41.0"]);
        let c = release(dir.path());
        assert_eq!(c.status, CheckStatus::Fail, "{}", c.message);
        assert!(c.message.contains("NO-RELEASE") && c.message.contains("PMAT-001"), "{}", c.message);
        assert!(c.message.contains("github-to-yaml"), "the fixer is named: {}", c.message);
    }

    /// §4.1: the key is the bare string, the `v` is added in exactly one
    /// place. Mutant: `trim_start_matches('v')` normalising instead of refusing.
    #[test]
    fn a_v_prefixed_release_fails_binding() {
        let dir = project(Some("paiml/fixture"), &item("PMAT-001", "planned work", "planned", Some(1), Some("v3.41.0")));
        snapshot(dir.path(), vec![issue(1, "planned work", "open", &[], Some("3.41.0"))], &["3.41.0"]);
        let c = release(dir.path());
        assert_eq!(c.status, CheckStatus::Fail, "{}", c.message);
        assert!(c.message.contains("PREFIXED") && c.message.contains("v3.41.0"), "{}", c.message);
    }

    /// RR-RELEASE: valid iff a milestone with that EXACT title exists.
    /// Mutant: the milestone list not consulted (or matched by prefix).
    #[test]
    fn a_release_with_no_milestone_of_that_title_fails_binding() {
        let dir = project(Some("paiml/fixture"), &item("PMAT-001", "planned work", "planned", Some(1), Some("3.41.0")));
        snapshot(dir.path(), vec![issue(1, "planned work", "open", &[], Some("3.41.0"))], &["3.41.0-rc1", "3.42.0"]);
        let c = release(dir.path());
        assert_eq!(c.status, CheckStatus::Fail, "{}", c.message);
        assert!(c.message.contains("NO-MILESTONE") && c.message.contains("3.41.0"), "{}", c.message);
    }

    /// Mutant: the membership clause dropped — release named, milestone
    /// exists, but the issue sits on another one (or none).
    #[test]
    fn an_issue_on_a_different_milestone_fails_binding() {
        let dir = project(Some("paiml/fixture"), &item("PMAT-001", "planned work", "planned", Some(1), Some("3.41.0")));
        snapshot(dir.path(), vec![issue(1, "planned work", "open", &[], Some("3.42.0"))], &["3.41.0", "3.42.0"]);
        let c = release(dir.path());
        assert_eq!(c.status, CheckStatus::Fail, "{}", c.message);
        assert!(c.message.contains("NOT-ON-MILESTONE") && c.message.contains("3.42.0") && c.message.contains("3.41.0"), "{}", c.message);

        let dir = project(Some("paiml/fixture"), &item("PMAT-001", "planned work", "planned", Some(1), Some("3.41.0")));
        snapshot(dir.path(), vec![issue(1, "planned work", "open", &[], None)], &["3.41.0"]);
        let c = release(dir.path());
        assert_eq!(c.status, CheckStatus::Fail, "{}", c.message);
        assert!(c.message.contains("NOT-ON-MILESTONE") && c.message.contains("no milestone"), "{}", c.message);
    }

    /// Mutant: an item with no issue skipped by the membership clause and so
    /// passing — "its issue is on it" cannot be checked without an issue.
    #[test]
    fn a_release_whose_item_has_no_issue_fails_binding() {
        let dir = project(Some("paiml/fixture"), &item("PMAT-001", "planned work", "planned", None, Some("3.41.0")));
        snapshot(dir.path(), vec![], &["3.41.0"]);
        let c = release(dir.path());
        assert_eq!(c.status, CheckStatus::Fail, "{}", c.message);
        assert!(c.message.contains("NO-ISSUE") && c.message.contains("PMAT-001"), "{}", c.message);
    }

    /// Mutant: verdict inversion.
    #[test]
    fn every_open_item_bound_to_an_existing_milestone_its_issue_is_on_passes_binding() {
        let dir = bound();
        let c = release(dir.path());
        assert_eq!(c.status, CheckStatus::Pass, "{}", c.message);
        assert!(c.message.contains("bound 1"), "{}", c.message);
        assert!(c.message.contains("snapshot.json"), "the source is named: {}", c.message);
    }

    /// §4.2: the 211 completed and 2 cancelled items are not backfilled.
    /// Mutant: scope widened to terminal items.
    #[test]
    fn a_completed_item_without_a_release_is_out_of_scope_for_binding() {
        let dir = project(Some("paiml/fixture"), &item("PMAT-001", "shipped", "completed", Some(1), None));
        snapshot(dir.path(), vec![issue(1, "shipped", "closed", &[], None)], &[]);
        let c = release(dir.path());
        assert_eq!(c.status, CheckStatus::Pass, "{}", c.message);
        assert!(c.message.contains("0 open item"), "{}", c.message);
    }

    // ───────────────────────── registration ─────────────────────────

    #[test]
    fn the_two_rules_are_registered_as_errors_under_their_ids() {
        let defaults = ComplyConfig::default();
        for (key, name) in [("cb-2112", LINKAGE), ("cb-2114", RELEASE)] {
            let cfg = defaults.checks.get(key).unwrap_or_else(|| panic!("{key} is a default check"));
            assert!(cfg.enabled, "{key} enabled by default");
            assert_eq!(cfg.severity, CheckSeverity::Error, "{key}: goal-mode.md §7 — anything less reports and never fails");
            let dir = bound();
            let c = pick(both(dir.path(), Some(&dir.path().join("snapshot.json"))), name);
            assert_eq!(c.severity, crate::cli::handlers::comply_handlers::check_handlers::types::Severity::from(CheckSeverity::Error));
        }
    }

    /// Mutant: a rule wired into the group list without the override — the
    /// flag reaching CB-2115 and not these two.
    #[test]
    fn the_github_snapshot_override_reaches_both_rules_through_the_group_list() {
        let dir = bound();
        let config: ComplyConfig = PmatYamlConfig::load(dir.path()).expect(".pmat.yaml parses").comply;
        let overrides = CheckOverrides { github_snapshot: Some(dir.path().join("snapshot.json")) };
        let checks = build_all_compliance_checks(dir.path(), &config, "0.0.0", &overrides);
        for name in [LINKAGE, RELEASE] {
            let c = checks.iter().find(|c| c.name == name).unwrap_or_else(|| panic!("{name} is in the report"));
            assert_eq!(c.status, CheckStatus::Pass, "{}: {}", c.name, c.message);
        }
    }
}
