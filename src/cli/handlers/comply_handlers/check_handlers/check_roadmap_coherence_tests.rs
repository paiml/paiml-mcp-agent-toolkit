// Included from check.rs — do NOT add `use` imports or `#!` attributes here.
#[cfg(all(test, not(coverage_nightly)))]
mod tests_roadmap_coherence {
    use super::*;
    use crate::models::comply_config::{CheckSeverity, ComplyConfig};
    use std::path::Path;

    const NAME: &str = "CB-2115: Roadmap Coherence";

    fn iso(when: chrono::DateTime<chrono::Utc>) -> String {
        when.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
    }
    fn days_ago(n: i64) -> String {
        iso(chrono::Utc::now() - chrono::Duration::days(n))
    }
    fn minutes_ago(n: i64) -> String {
        iso(chrono::Utc::now() - chrono::Duration::minutes(n))
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

    fn item(id: &str, title: &str, status: &str, issue: Option<u64>, updated: &str) -> String {
        let issue = issue.map_or("null".to_string(), |n| n.to_string());
        format!("  - id: {id}\n    title: {title}\n    status: {status}\n    github_issue: {issue}\n    updated: {updated}\n")
    }

    fn issue(number: u64, title: &str, state: &str, labels: &[&str], updated_at: &str) -> serde_json::Value {
        serde_json::json!({"number": number, "title": title, "state": state, "labels": labels, "updated_at": updated_at})
    }

    /// Write `snapshot.json` and point `.pmat.yaml`'s `cb-2115` options at it.
    fn snapshot(dir: &Path, issues: Vec<serde_json::Value>, grace_minutes: Option<i64>) {
        let snap = serde_json::json!({"repo": "paiml/fixture", "taken_at": "2026-09-10T00:00:00Z", "issues": issues, "milestones": []});
        std::fs::write(dir.join("snapshot.json"), snap.to_string()).expect("write snapshot");
        let grace = grace_minutes.map_or(String::new(), |g| format!("        grace_minutes: {g}\n"));
        std::fs::write(
            dir.join(".pmat.yaml"),
            format!("comply:\n  checks:\n    cb-2115:\n      options:\n        snapshot: snapshot.json\n{grace}"),
        )
        .expect("write .pmat.yaml");
    }

    fn run(dir: &Path) -> ComplianceCheck {
        let config: ComplyConfig = PmatYamlConfig::load(dir).expect(".pmat.yaml parses").comply;
        let mut checks = build_roadmap_coherence_checks(dir, &config);
        assert_eq!(checks.len(), 1, "exactly one CB-2115 row");
        checks.remove(0)
    }

    fn bijection() -> tempfile::TempDir {
        let dir = project(Some("paiml/fixture"), &item("PMAT-001", "planned work", "planned", Some(1), &days_ago(2)));
        snapshot(dir.path(), vec![issue(1, "planned work", "open", &[], &days_ago(2))], None);
        dir
    }

    #[test]
    fn a_project_without_a_roadmap_is_skipped() {
        let dir = tempfile::tempdir().expect("tempdir");
        let c = run(dir.path());
        assert_eq!(c.status, CheckStatus::Skip, "{}", c.message);
        assert!(c.message.contains("docs/roadmaps/roadmap.yaml"), "{}", c.message);
    }

    #[test]
    fn a_roadmap_naming_no_github_repository_is_skipped() {
        // No `github_repo`, no git remote, no snapshot: nothing to be coherent WITH.
        let dir = project(None, &item("PMAT-001", "planned work", "planned", Some(1), &days_ago(2)));
        let c = run(dir.path());
        assert_eq!(c.status, CheckStatus::Skip, "{}", c.message);
        assert!(c.message.contains("GitHub repository"), "{}", c.message);
    }

    #[test]
    fn a_snapshot_the_rule_cannot_read_is_not_measured_and_fails() {
        // An input the rule expected and could not read is a FAILURE, not a pass
        // (goal-mode.md doctrine 2; .pmat-ratchet.toml's rule for a zero it
        // cannot tell from a rotted pathspec).
        let dir = project(Some("paiml/fixture"), &item("PMAT-001", "planned work", "planned", Some(1), &days_ago(2)));
        std::fs::write(
            dir.path().join(".pmat.yaml"),
            "comply:\n  checks:\n    cb-2115:\n      options:\n        snapshot: missing.json\n",
        )
        .expect("write .pmat.yaml");
        let c = run(dir.path());
        assert_eq!(c.status, CheckStatus::Fail, "{}", c.message);
        assert_eq!(c.severity, CheckSeverity::Error.into());
        assert!(c.message.starts_with("not_measured:"), "{}", c.message);
        assert!(c.message.contains("missing.json"), "{}", c.message);
    }

    #[test]
    fn a_closed_linked_issue_with_an_open_item_fails() {
        // goal-mode.md §7, the CB-2115 falsifier: close one linked issue, leave
        // the item open.
        let dir = project(Some("paiml/fixture"), &item("PMAT-001", "planned work", "planned", Some(1), &days_ago(2)));
        snapshot(dir.path(), vec![issue(1, "planned work", "closed", &[], &days_ago(1))], None);
        let c = run(dir.path());
        assert_eq!(c.status, CheckStatus::Fail, "{}", c.message);
        assert_eq!(c.severity, CheckSeverity::Error.into());
        for needle in ["ORPHAN-ROADMAP", "PMAT-001", "#1", "closed"] {
            assert!(c.message.contains(needle), "message must name {needle}: {}", c.message);
        }
    }

    #[test]
    fn a_bijection_passes_and_names_its_snapshot() {
        let dir = bijection();
        let c = run(dir.path());
        assert_eq!(c.status, CheckStatus::Pass, "{}", c.message);
        assert!(c.message.contains("matched 1"), "{}", c.message);
        assert!(c.message.contains("snapshot.json"), "the verdict must say where the GitHub side came from: {}", c.message);
    }

    #[test]
    fn a_terminal_item_naming_a_closed_issue_is_not_a_finding() {
        // Control: only the OPEN sets are compared (§5.1). A completed item whose
        // issue closed is the normal end of a ticket's life, not an orphan.
        let dir = project(Some("paiml/fixture"), &item("PMAT-001", "done work", "completed", Some(1), &days_ago(2)));
        snapshot(dir.path(), vec![issue(1, "done work", "closed", &[], &days_ago(2))], None);
        let c = run(dir.path());
        assert_eq!(c.status, CheckStatus::Pass, "{}", c.message);
    }

    #[test]
    fn two_items_naming_one_issue_is_a_collision_and_fails() {
        let items = format!(
            "{}{}",
            item("PMAT-001", "first", "planned", Some(1), &days_ago(2)),
            item("PMAT-002", "second", "planned", Some(1), &days_ago(2))
        );
        let dir = project(Some("paiml/fixture"), &items);
        snapshot(dir.path(), vec![issue(1, "first", "open", &[], &days_ago(2))], None);
        let c = run(dir.path());
        assert_eq!(c.status, CheckStatus::Fail, "{}", c.message);
        for needle in ["COLLISION", "#1", "PMAT-001", "PMAT-002"] {
            assert!(c.message.contains(needle), "message must name {needle}: {}", c.message);
        }
    }

    #[test]
    fn an_open_unlabelled_issue_no_item_names_fails() {
        let dir = project(Some("paiml/fixture"), &item("PMAT-001", "planned work", "planned", Some(1), &days_ago(2)));
        snapshot(
            dir.path(),
            vec![
                issue(1, "planned work", "open", &[], &days_ago(2)),
                issue(2, "stray", "open", &[], &days_ago(2)),
            ],
            None,
        );
        let c = run(dir.path());
        assert_eq!(c.status, CheckStatus::Fail, "{}", c.message);
        assert!(c.message.contains("ORPHAN-GITHUB") && c.message.contains("#2"), "{}", c.message);
    }

    #[test]
    fn a_no_roadmap_issue_is_outside_the_universe() {
        let dir = project(Some("paiml/fixture"), &item("PMAT-001", "planned work", "planned", Some(1), &days_ago(2)));
        snapshot(
            dir.path(),
            vec![
                issue(1, "planned work", "open", &[], &days_ago(2)),
                issue(2, "a question", "open", &["no-roadmap"], &days_ago(2)),
            ],
            None,
        );
        let c = run(dir.path());
        assert_eq!(c.status, CheckStatus::Pass, "{}", c.message);
    }

    #[test]
    fn a_title_disagreement_past_the_grace_window_fails() {
        let dir = project(Some("paiml/fixture"), &item("PMAT-001", "old title", "planned", Some(1), &days_ago(2)));
        snapshot(dir.path(), vec![issue(1, "new title", "open", &[], &days_ago(2))], None);
        let c = run(dir.path());
        assert_eq!(c.status, CheckStatus::Fail, "{}", c.message);
        for needle in ["DRIFT", "PMAT-001", "title"] {
            assert!(c.message.contains(needle), "message must name {needle}: {}", c.message);
        }
    }

    #[test]
    fn a_title_disagreement_inside_the_grace_window_is_tolerated() {
        // §5.2: a bound in TIME, not a count. The issue moved a minute ago; the
        // roadmap has the default 60 minutes to catch up.
        let dir = project(Some("paiml/fixture"), &item("PMAT-001", "old title", "planned", Some(1), &days_ago(2)));
        snapshot(dir.path(), vec![issue(1, "new title", "open", &[], &minutes_ago(1))], None);
        let c = run(dir.path());
        assert_eq!(c.status, CheckStatus::Pass, "{}", c.message);
        assert!(c.message.contains("tolerated 1"), "{}", c.message);
    }

    #[test]
    fn grace_minutes_is_read_from_the_project_config() {
        // The same 30-minute-old disagreement: tolerated under the default 60,
        // a finding under `grace_minutes: 10`. Proves the option is READ.
        let dir = project(Some("paiml/fixture"), &item("PMAT-001", "old title", "planned", Some(1), &minutes_ago(30)));
        snapshot(dir.path(), vec![issue(1, "new title", "open", &[], &minutes_ago(30))], None);
        let c = run(dir.path());
        assert_eq!(c.status, CheckStatus::Pass, "default grace is 60 minutes: {}", c.message);

        snapshot(dir.path(), vec![issue(1, "new title", "open", &[], &minutes_ago(30))], Some(10));
        let c = run(dir.path());
        assert_eq!(c.status, CheckStatus::Fail, "grace_minutes: 10 must make it a finding: {}", c.message);
        assert!(c.message.contains("DRIFT"), "{}", c.message);
    }

    #[test]
    fn the_check_is_named_error_severity_and_registered() {
        let dir = bijection();
        assert_eq!(run(dir.path()).name, NAME);
        let severity = ComplyConfig::default().checks.get("cb-2115").map(|c| c.severity);
        assert_eq!(
            severity,
            Some(CheckSeverity::Error),
            "cb-2115 must be an Error-severity member of the roster CB-2100 verifies"
        );
    }
}
