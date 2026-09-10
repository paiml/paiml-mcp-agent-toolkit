// Included from check.rs — do NOT add `use` imports or `#!` attributes here.
//
// PMAT-728 (goal-mode.md §11 step 6): CB-2110 (invariant E) and the
// retirement of CB-148. Every test names the mutant it dies under in its doc
// comment, so a survivor is a test that should be deleted, not kept.
#[cfg(all(test, not(coverage_nightly)))]
mod tests_spec_epics {
    use super::*;
    use crate::models::comply_config::{CheckSeverity, ComplyConfig};
    use std::path::Path;

    const EPICS: &str = "CB-2110: Spec Epics";

    /// A project with an empty `docs/specifications/` and a roadmap that
    /// names `repo` (or none), so the snapshot resolves the way the roadmap
    /// rules resolve it.
    fn project(repo: Option<&str>) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("docs/specifications")).expect("mkdir specs");
        let rm = dir.path().join("docs/roadmaps/roadmap.yaml");
        std::fs::create_dir_all(rm.parent().expect("parent")).expect("mkdir");
        let repo_line = repo.map_or("null".to_string(), str::to_string);
        std::fs::write(&rm, format!("roadmap_version: \"1.0\"\ngithub_enabled: true\ngithub_repo: {repo_line}\nroadmap: []\n")).expect("write roadmap");
        dir
    }

    /// Write `docs/specifications/<rel>`; `rel` may hold `components/`.
    fn spec(dir: &Path, rel: &str, text: &str) {
        let p = dir.join("docs/specifications").join(rel);
        std::fs::create_dir_all(p.parent().expect("parent")).expect("mkdir");
        std::fs::write(p, text).expect("write spec");
    }

    /// goal-mode.md's own header shape.
    fn fm(epic: &str, status: &str) -> String {
        format!("---\nepic: {epic}\nstatus: {status}\nvendors: []\n---\n\n# A spec\n\nBody.\n")
    }

    /// One issue as the snapshot saw it; `sub_issues` is written only when
    /// measured, exactly as the serializer does.
    fn issue(number: u64, state: &str, labels: &[&str], sub_issues: Option<u64>) -> serde_json::Value {
        let mut v = serde_json::json!({"number": number, "title": format!("issue {number}"), "state": state, "labels": labels, "updated_at": "2026-09-10T00:00:00Z"});
        if let Some(n) = sub_issues {
            v["sub_issues"] = serde_json::json!(n);
        }
        v
    }

    fn snapshot(dir: &Path, issues: Vec<serde_json::Value>) {
        let snap = serde_json::json!({"repo": "paiml/fixture", "taken_at": "2026-09-10T00:00:00Z", "issues": issues, "milestones": []});
        std::fs::write(dir.join("snapshot.json"), snap.to_string()).expect("write snapshot");
    }

    fn rule_with(dir: &Path, github_snapshot: Option<&Path>) -> ComplianceCheck {
        let config: ComplyConfig = PmatYamlConfig::load(dir).expect(".pmat.yaml parses").comply;
        let mut checks = build_spec_epic_checks(dir, &config, github_snapshot);
        assert_eq!(checks.len(), 1, "exactly one CB-2110 row");
        let c = checks.remove(0);
        assert_eq!(c.name, EPICS);
        c
    }
    /// The common case: judge from the fixture's `snapshot.json`.
    fn rule(dir: &Path) -> ComplianceCheck {
        rule_with(dir, Some(&dir.join("snapshot.json")))
    }

    /// One active spec bound to epic #7: open, labelled, one sub-issue.
    fn bound() -> tempfile::TempDir {
        let dir = project(Some("paiml/fixture"));
        spec(dir.path(), "a.md", &fm("7", "active"));
        snapshot(dir.path(), vec![issue(7, "open", &["epic"], Some(1))]);
        dir
    }

    fn assert_not_measured(c: &ComplianceCheck, needle: &str) {
        assert_eq!(c.status, CheckStatus::Fail, "{}", c.message);
        assert!(c.message.starts_with("not_measured:"), "{}", c.message);
        assert!(c.message.contains(needle), "{} lacks {needle:?}", c.message);
    }

    fn assert_fail_with(c: &ComplianceCheck, rendered_prefix: &str) {
        assert_eq!(c.status, CheckStatus::Fail, "{}", c.message);
        assert!(!c.message.starts_with("not_measured:"), "{}", c.message);
        assert!(c.message.contains(rendered_prefix), "{} lacks {rendered_prefix:?}", c.message);
    }

    /// Host git configuration is kept out of the fixture.
    fn git(dir: &Path, args: &[&str]) {
        let status = std::process::Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(["-c", "user.name=pmat728", "-c", "user.email=pmat728@example.invalid", "-c", "commit.gpgsign=false"])
            .args(args)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .status()
            .expect("git runs");
        assert!(status.success(), "git {args:?}");
    }

    // ───────────────────────── inputs ─────────────────────────

    /// Mutant (quorum on PMAT-728, delegate-verified §12 hole): a NotFound or
    /// empty `docs/specifications` mapped to Skip without asking git whether
    /// it was committed — `rm -rf docs/specifications` then passed the rule.
    /// And the converse: a repository whose history never held the directory
    /// still skips (a structural absence), so "any git repo ⇒ not_measured"
    /// dies too.
    #[test]
    fn a_committed_then_deleted_specifications_directory_is_not_measured() {
        let dir = bound();
        git(dir.path(), &["init", "-q", "-b", "master"]);
        git(dir.path(), &["add", "-A"]);
        git(dir.path(), &["commit", "-q", "-m", "specs"]);
        git(dir.path(), &["rm", "-r", "-q", "docs/specifications"]);
        assert!(!dir.path().join("docs/specifications").exists());
        assert_not_measured(&rule(dir.path()), "committed and is now gone");

        // Emptied, not removed: git tracks no empty directory, so this is the
        // same deletion.
        std::fs::create_dir_all(dir.path().join("docs/specifications")).expect("mkdir");
        assert_not_measured(&rule(dir.path()), "committed and is now gone");

        let never = project(Some("paiml/fixture"));
        git(never.path(), &["init", "-q", "-b", "master"]);
        git(never.path(), &["add", "-A"]);
        git(never.path(), &["commit", "-q", "-m", "roadmap only"]);
        std::fs::remove_dir_all(never.path().join("docs/specifications")).expect("rm specs");
        let c = rule_with(never.path(), None);
        assert_eq!(c.status, CheckStatus::Skip, "{}", c.message);
        assert!(c.message.contains("no docs/specifications"), "{}", c.message);
    }

    /// Mutant (quorum lane 2): `findings.sort_by(..)` deleted — the parse
    /// leg's findings then precede the epic leg's regardless of path, and
    /// the header's first-seen class order follows.
    #[test]
    fn findings_from_both_legs_are_rendered_in_path_order() {
        let dir = project(Some("paiml/fixture"));
        spec(dir.path(), "a.md", &fm("7", "active"));
        spec(dir.path(), "z.md", "---\nstatus: active\nvendors: []\n---\n# Z\n");
        snapshot(dir.path(), vec![issue(7, "closed", &["epic"], Some(1))]);
        let c = rule(dir.path());
        assert!(
            c.message.starts_with("2 finding(s) — EPIC-CLOSED 1, NO-EPIC 1: EPIC-CLOSED docs/specifications/a.md:"),
            "{}",
            c.message
        );
        assert!(c.message.contains("; NO-EPIC docs/specifications/z.md:"), "{}", c.message);
    }

    /// Mutant (quorum lane 3): `repo_hint` unconditionally `None` — the
    /// roadmap's `github_repo` then never reaches the live source and the
    /// verdict reads "no GitHub repository resolves" instead of naming the
    /// repository it tried. `paiml/fixture` does not exist, so the live read
    /// fails whether or not `gh` is installed or authenticated, and the row
    /// is not_measured either way; what this pins is the SOURCE named.
    #[test]
    fn the_roadmaps_github_repo_is_the_live_source_when_no_snapshot_is_given() {
        let dir = project(Some("paiml/fixture"));
        spec(dir.path(), "a.md", &fm("7", "active"));
        let c = rule_with(dir.path(), None);
        assert_not_measured(&c, "snapshot gh paiml/fixture");
        assert!(!c.message.contains("no GitHub repository resolves"), "{}", c.message);
    }

    /// Mutant: a project with no specs read as a failure (or a pass) instead
    /// of a structural absence.
    #[test]
    fn a_project_without_a_specifications_directory_is_skipped() {
        let dir = tempfile::tempdir().expect("tempdir");
        let c = rule_with(dir.path(), None);
        assert_eq!(c.status, CheckStatus::Skip, "{}", c.message);
        assert!(c.message.contains("no docs/specifications"), "{}", c.message);
    }

    /// Mutant: `list_specs` skipping `components/`, taking non-`.md` files, or
    /// returning absolute or unsorted paths.
    #[test]
    fn list_specs_walks_components_sorted_with_project_relative_paths() {
        let dir = project(None);
        spec(dir.path(), "b.md", "# B\n");
        spec(dir.path(), "components/a.md", "# CA\n");
        spec(dir.path(), "a.md", "# A\n");
        spec(dir.path(), "notes.txt", "not a spec\n");
        let specs = list_specs(dir.path()).expect("lists");
        let paths: Vec<&str> = specs.iter().map(|s| s.path.as_str()).collect();
        assert_eq!(paths, vec!["docs/specifications/a.md", "docs/specifications/b.md", "docs/specifications/components/a.md"]);
        assert_eq!(specs[2].text, "# CA\n");
        let empty = tempfile::tempdir().expect("tempdir");
        assert!(list_specs(empty.path()).is_err(), "no directory is an error the caller turns into a Skip");
    }

    /// Mutant: the snapshot loaded before the parse leg, so a project whose
    /// only findings are offline goes `not_measured` when GitHub is
    /// unreachable — the parse leg needs no network. (The roadmap names a
    /// repository, so a live fetch would be attempted, and would fail here.)
    #[test]
    fn the_parse_leg_needs_no_snapshot_when_no_active_spec_names_an_epic() {
        let dir = project(Some("paiml/fixture"));
        spec(dir.path(), "a.md", &fm("null", "active"));
        let c = rule_with(dir.path(), None);
        assert_fail_with(&c, "NO-EPIC docs/specifications/a.md:");
        assert!(c.message.contains("snapshot: not needed"), "{}", c.message);
    }

    /// Mutant: an epic named with no reachable snapshot read as a pass, or as
    /// a plain failure without `not_measured`. Also: the exempt list must
    /// survive the early verdict — the escape hatch is printed on every run.
    #[test]
    fn an_epic_named_with_no_reachable_snapshot_is_not_measured_and_still_names_the_exempt() {
        let dir = project(None);
        spec(dir.path(), "a.md", &fm("7", "active"));
        spec(dir.path(), "h.md", &fm("null", "historical"));
        let c = rule_with(dir.path(), None);
        assert_not_measured(&c, "no GitHub repository resolves");
        assert!(c.message.contains("docs/specifications/h.md (historical)"), "{}", c.message);
    }

    /// Mutant: a missing snapshot file read as an empty snapshot (every epic
    /// then `EPIC-ABSENT`, a red for the wrong reason).
    #[test]
    fn an_unreadable_snapshot_is_not_measured() {
        let dir = bound();
        let c = rule_with(dir.path(), Some(Path::new("missing.json")));
        assert_not_measured(&c, "missing.json");
    }

    /// Mutant: the `.pmat.yaml` `snapshot` key read as a file source — the
    /// bypass token §12 forbids.
    #[test]
    fn a_snapshot_path_committed_in_pmat_yaml_is_refused() {
        let dir = bound();
        std::fs::write(dir.path().join(".pmat.yaml"), "comply:\n  checks:\n    cb-2110:\n      options:\n        snapshot: snapshot.json\n").expect("write .pmat.yaml");
        let c = rule(dir.path());
        assert_not_measured(&c, ".pmat.yaml");
        assert!(c.message.contains("--github-snapshot"), "{}", c.message);
        assert!(c.message.contains("cb-2110"), "{}", c.message);
    }

    // ───────────────────────── the parse leg ─────────────────────────

    /// The §7 falsifier: delete the `epic:` line. Mutant: verdict inversion;
    /// mutant M2: `NO-EPIC` only when the key is missing (null passes).
    #[test]
    fn the_section_7_falsifier_deleting_the_epic_line_is_refused() {
        let dir = bound();
        spec(dir.path(), "a.md", "---\nstatus: active\nvendors: []\n---\n\n# A spec\n");
        let c = rule(dir.path());
        assert_fail_with(&c, "NO-EPIC docs/specifications/a.md:");
        assert!(c.message.contains("create it on GitHub"), "{}", c.message);
        spec(dir.path(), "a.md", &fm("null", "active"));
        let c = rule(dir.path());
        assert_fail_with(&c, "NO-EPIC docs/specifications/a.md:");
    }

    /// Mutant: a spec with no front-matter read as `epic: null` (one class
    /// swallowing another, two fixes conflated).
    #[test]
    fn a_spec_without_front_matter_is_its_own_class() {
        let dir = bound();
        spec(dir.path(), "components/c.md", "# C — no header\n");
        let c = rule(dir.path());
        assert_fail_with(&c, "NO-FRONT-MATTER docs/specifications/components/c.md:");
        assert!(!c.message.contains("NO-EPIC docs/specifications/components/c.md"), "{}", c.message);
    }

    /// Mutant M1: a `historical` spec's exemption leaking into the parse
    /// leg — its broken front-matter must still be a finding.
    #[test]
    fn the_parse_leg_still_judges_a_historical_spec() {
        let dir = bound();
        spec(dir.path(), "h.md", "---\nepic: x\nstatus: historical\n---\n# H\n");
        let c = rule(dir.path());
        assert_fail_with(&c, "BAD-FRONT-MATTER docs/specifications/h.md: epic: \"x\" is neither an issue number nor null");
    }

    // ───────────────────────── the epic leg ─────────────────────────

    /// Mutant: verdict inversion; the count of linked specs not measured.
    #[test]
    fn a_bound_active_spec_passes_naming_linked_1_and_no_exemption() {
        let dir = bound();
        let c = rule(dir.path());
        assert_eq!(c.status, CheckStatus::Pass, "{}", c.message);
        assert!(c.message.contains("linked 1"), "{}", c.message);
        assert!(c.message.contains("exempt from the epic leg: none"), "{}", c.message);
        assert!(c.message.contains("snapshot: file"), "{}", c.message);
    }

    /// Mutant M5: the closed clause dropped.
    #[test]
    fn a_closed_epic_is_refused() {
        let dir = bound();
        snapshot(dir.path(), vec![issue(7, "closed", &["epic"], Some(1))]);
        let c = rule(dir.path());
        assert_fail_with(&c, "EPIC-CLOSED docs/specifications/a.md: names #7");
    }

    /// Mutant: an absent issue read as open (a `find` result unwrapped to a
    /// default).
    #[test]
    fn an_absent_epic_is_refused() {
        let dir = bound();
        snapshot(dir.path(), vec![issue(8, "open", &["epic"], Some(1))]);
        let c = rule(dir.path());
        assert_fail_with(&c, "EPIC-ABSENT docs/specifications/a.md: names #7");
    }

    /// Mutant M4: the label clause dropped — any open issue is an epic.
    #[test]
    fn an_open_issue_not_labelled_epic_is_not_an_epic() {
        let dir = bound();
        snapshot(dir.path(), vec![issue(7, "open", &["bug", "Epic"], Some(1))]);
        let c = rule(dir.path());
        assert_fail_with(&c, "NOT-AN-EPIC docs/specifications/a.md: names #7");
    }

    /// Mutant: `≥1` written as `≥0`.
    #[test]
    fn an_epic_with_no_sub_issue_is_refused() {
        let dir = bound();
        snapshot(dir.path(), vec![issue(7, "open", &["epic"], Some(0))]);
        let c = rule(dir.path());
        assert_fail_with(&c, "NO-SUB-ISSUES docs/specifications/a.md: #7 has no sub-issue");
    }

    /// Mutant M3: a sub-issue count the snapshot did not carry read as zero
    /// (a `NO-SUB-ISSUES` red for the wrong reason) or as a pass.
    #[test]
    fn a_sub_issue_count_the_snapshot_did_not_measure_is_not_measured() {
        let dir = bound();
        snapshot(dir.path(), vec![issue(7, "open", &["epic"], None)]);
        let c = rule(dir.path());
        assert_not_measured(&c, "SUB-ISSUES-UNMEASURED docs/specifications/a.md:");
        assert!(!c.message.contains("NO-SUB-ISSUES"), "{}", c.message);
    }

    /// Mutant: exempt specs judged on the epic leg (their epic fetched and
    /// refused), or not named in the verdict. §4.3: listed by name on every
    /// run — Pass and Fail alike.
    #[test]
    fn historical_and_superseded_are_off_the_epic_leg_and_named_in_pass_and_fail() {
        let dir = bound();
        spec(dir.path(), "h.md", &fm("null", "historical"));
        spec(dir.path(), "s.md", &fm("99", "superseded"));
        let c = rule(dir.path());
        assert_eq!(c.status, CheckStatus::Pass, "{}", c.message);
        let named = "exempt from the epic leg: docs/specifications/h.md (historical), docs/specifications/s.md (superseded)";
        assert!(c.message.contains(named), "{}", c.message);
        assert!(c.message.contains("3 spec(s)"), "{}", c.message);
        assert!(c.message.contains("linked 1"), "{}", c.message);
        snapshot(dir.path(), vec![issue(7, "closed", &["epic"], Some(1))]);
        let c = rule(dir.path());
        assert_fail_with(&c, "EPIC-CLOSED docs/specifications/a.md:");
        assert!(c.message.contains(named), "{}", c.message);
        assert!(!c.message.contains("#99"), "{}", c.message);
    }

    /// Mutant: the ninth finding dropped silently.
    #[test]
    fn more_than_eight_findings_are_counted_not_dropped() {
        let dir = project(Some("paiml/fixture"));
        for i in 0..10 {
            spec(dir.path(), &format!("s{i:02}.md"), &fm("null", "active"));
        }
        snapshot(dir.path(), vec![]);
        let c = rule(dir.path());
        assert!(c.message.starts_with("10 finding(s) — NO-EPIC 10:"), "{}", c.message);
        assert!(c.message.contains("(+2 more)"), "{}", c.message);
    }

    /// Mutant: a header that enumerates every class with a zero count — the
    /// M5 lesson from PMAT-724: `contains("CLASS")` is then true on any
    /// failure.
    #[test]
    fn the_fail_header_names_only_the_classes_that_fired() {
        let dir = project(Some("paiml/fixture"));
        spec(dir.path(), "a.md", &fm("null", "active"));
        spec(dir.path(), "b.md", "# B\n");
        snapshot(dir.path(), vec![]);
        let c = rule(dir.path());
        assert!(c.message.starts_with("2 finding(s) — NO-EPIC 1, NO-FRONT-MATTER 1:"), "{}", c.message);
        assert!(!c.message.contains("EPIC-CLOSED"), "{}", c.message);
    }

    // ───────────────────────── registration, CB-148, this tree ─────────────────────────

    /// Mutant: the rule registered `Warning` (reports and never fails).
    #[test]
    fn the_rule_is_registered_as_an_error_under_cb_2110() {
        let cfg = ComplyConfig::default().checks.get("cb-2110").cloned();
        assert!(cfg.is_some(), "cb-2110 is a default check");
        let cfg = cfg.expect("asserted above");
        assert!(cfg.enabled);
        assert_eq!(cfg.severity, CheckSeverity::Error, "goal-mode.md §7 — anything less reports and never fails");
        let dir = bound();
        let c = rule(dir.path());
        assert_eq!(c.severity, crate::cli::handlers::comply_handlers::check_handlers::types::Severity::from(CheckSeverity::Error));
    }

    /// Mutant: the rule wired into the group list without the override.
    #[test]
    fn the_github_snapshot_override_reaches_the_rule_through_the_group_list() {
        let dir = bound();
        let config: ComplyConfig = PmatYamlConfig::load(dir.path()).expect(".pmat.yaml parses").comply;
        let overrides = CheckOverrides { github_snapshot: Some(dir.path().join("snapshot.json")) };
        let checks = build_all_compliance_checks(dir.path(), &config, "0.0.0", &overrides);
        let rows: Vec<&ComplianceCheck> = checks.iter().filter(|c| c.name == EPICS).collect();
        assert_eq!(rows.len(), 1, "{EPICS} is in the report exactly once");
        assert_eq!(rows[0].status, CheckStatus::Pass, "{}", rows[0].message);
    }

    /// goal-mode.md §11: CB-148 is retired, not patched — the id prints as
    /// `RETIRED — superseded by CB-2110` for one minor version. Mutant: the
    /// row kept as a pass, or the id dropped silently.
    #[test]
    fn cb_148_prints_retired_superseded_by_cb_2110_and_judges_nothing() {
        let dir = project(None);
        spec(dir.path(), "components/x.md", "# X\n\n## Thing (Planned)\n");
        let c = check_spec_work_traceability(dir.path());
        assert_eq!(c.name, "CB-148: RETIRED — superseded by CB-2110");
        assert_eq!(c.status, CheckStatus::Skip, "{}", c.message);
        assert!(c.message.contains("goal-mode.md §11"), "{}", c.message);
        assert!(c.message.contains("CB-2110"), "{}", c.message);
    }

    /// This repository's own specifications, every one of them, carry
    /// front-matter that parses (the step-6 deliverable "front-matter on the
    /// 43 specs"; goal-mode.md carried its own before). Skipped, and said so,
    /// only where the directory is not shipped (a published crate).
    #[test]
    fn every_committed_spec_carries_front_matter_that_parses() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        if !root.join(crate::services::spec_epic::SPECS_DIR).is_dir() {
            eprintln!("docs/specifications is not in this tree (a published crate): nothing to judge");
            return;
        }
        let specs = list_specs(root).expect("the tree's specs list");
        assert!(specs.len() >= 44, "44 specs at PMAT-728, {} found", specs.len());
        let parsed = crate::services::spec_epic::parse_specs(&specs);
        let bad: Vec<String> = parsed.findings.iter().filter(|f| !matches!(f, crate::services::spec_epic::SpecFinding::NoEpic { .. })).map(crate::services::spec_epic::SpecFinding::render).collect();
        assert!(bad.is_empty(), "every committed spec parses: {bad:?}");
        assert_eq!(parsed.specs.len(), specs.len());
    }
}
