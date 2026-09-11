// Included from check.rs — do NOT add `use` imports or `#!` attributes here.
//
// PMAT-1299 (goal-mode.md §11 step 7): CB-2111 (invariant E.1), the spec
// review artifact judged offline from a file. Every test names the mutant it
// dies under in its doc comment, so a survivor is a test that should be
// deleted, not kept.
#[cfg(all(test, not(coverage_nightly)))]
mod tests_spec_reviews {
    use super::*;
    use crate::models::comply_config::ComplyConfig;
    use crate::services::spec_review::{artifact_path, sha256_hex, BASE_ROLES};
    use std::path::Path;

    const REVIEWS: &str = "CB-2111: Spec Reviews";

    /// A project with an empty `docs/specifications/`.
    fn project() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("docs/specifications")).expect("mkdir specs");
        dir
    }

    /// Write `docs/specifications/<rel>`; returns its project-relative path.
    fn spec(dir: &Path, rel: &str, text: &str) -> String {
        let p = dir.join("docs/specifications").join(rel);
        std::fs::create_dir_all(p.parent().expect("parent")).expect("mkdir");
        std::fs::write(p, text).expect("write spec");
        format!("docs/specifications/{rel}")
    }

    /// goal-mode.md's own header shape.
    fn fm(status: &str, vendors: &str) -> String {
        format!("---\nepic: null\nstatus: {status}\nvendors: {vendors}\n---\n\n# A spec\n\nBody.\n")
    }

    /// A review of `spec_path` as the file reads NOW, one PASS lane per role.
    fn review(dir: &Path, spec_path: &str, roles: &[&str]) {
        let text = std::fs::read_to_string(dir.join(spec_path)).expect("read spec");
        let lanes: Vec<serde_json::Value> = roles
            .iter()
            .map(|r| serde_json::json!({"role": r, "executor": "human", "verdict": "PASS", "summary": "read it"}))
            .collect();
        let artifact = serde_json::json!({
            "spec": spec_path,
            "spec_sha256": sha256_hex(&text),
            "plan": {"tool": "claude-plan", "ref": "plan.md", "sha256": "64879f7d6b960a01909762d911a32d4582c20010c5641ee90278b644a9e3b525"},
            "lanes": lanes,
            "agreed": true,
            "partial": false
        });
        let p = dir.join(artifact_path(spec_path));
        std::fs::create_dir_all(p.parent().expect("parent")).expect("mkdir audits");
        std::fs::write(p, artifact.to_string()).expect("write review");
    }

    /// Append one space to a spec: the §7 falsifier.
    fn append_a_space(dir: &Path, spec_path: &str) {
        let p = dir.join(spec_path);
        let text = std::fs::read_to_string(&p).expect("read spec");
        std::fs::write(&p, format!("{text} ")).expect("append a space");
    }

    fn rule(dir: &Path) -> ComplianceCheck {
        let config: ComplyConfig = PmatYamlConfig::load(dir).expect(".pmat.yaml parses").comply;
        let mut checks = build_spec_review_checks(dir, &config);
        assert_eq!(checks.len(), 1, "exactly one CB-2111 row");
        let c = checks.remove(0);
        assert_eq!(c.name, REVIEWS);
        c
    }

    fn assert_fail_with(c: &ComplianceCheck, needle: &str) {
        assert_eq!(c.status, CheckStatus::Fail, "{}", c.message);
        assert!(!c.message.starts_with("not_measured:"), "{}", c.message);
        assert!(c.message.contains(needle), "{} lacks {needle:?}", c.message);
    }

    /// Host git configuration is kept out of the fixture.
    fn git(dir: &Path, args: &[&str]) {
        let status = std::process::Command::new("git")
            .arg("-C")
            .arg(dir)
            .args([
                "-c",
                "user.name=pmat1299",
                "-c",
                "user.email=pmat1299@example.invalid",
                "-c",
                "commit.gpgsign=false",
            ])
            .args(args)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .status()
            .expect("git runs");
        assert!(status.success(), "git {args:?}");
    }

    /// Mutant: the absent-by-design case fails, or passes as though judged.
    #[test]
    fn a_project_without_a_specifications_directory_is_skipped() {
        let dir = tempfile::tempdir().expect("tempdir");
        let c = rule(dir.path());
        assert_eq!(c.status, CheckStatus::Skip, "{}", c.message);
        assert!(
            c.message.contains("no docs/specifications"),
            "{}",
            c.message
        );
    }

    /// §12: deleting a gate's input is not a way of passing it. Mutant: the
    /// NotFound arm skips without asking git whether the directory was committed.
    #[test]
    fn a_committed_then_deleted_specifications_directory_is_not_measured() {
        let dir = project();
        spec(dir.path(), "a.md", &fm("active", "[]"));
        git(dir.path(), &["init", "-q"]);
        git(dir.path(), &["add", "-A"]);
        git(dir.path(), &["commit", "-q", "-m", "specs"]);
        std::fs::remove_dir_all(dir.path().join("docs/specifications")).expect("rm specs");
        let c = rule(dir.path());
        assert_eq!(c.status, CheckStatus::Fail, "{}", c.message);
        assert!(c.message.starts_with("not_measured:"), "{}", c.message);
        assert!(
            c.message.contains("committed and is now gone"),
            "{}",
            c.message
        );
    }

    /// The guard. Mutant: the reviewed counter never moves.
    #[test]
    fn an_active_spec_with_a_current_complete_review_passes() {
        let dir = project();
        let a = spec(dir.path(), "a.md", &fm("active", "[]"));
        review(dir.path(), &a, &BASE_ROLES);
        let c = rule(dir.path());
        assert_eq!(c.status, CheckStatus::Pass, "{}", c.message);
        assert!(c.message.contains("reviewed 1"), "{}", c.message);
    }

    /// Mutant: a missing artifact file read as "nothing to judge".
    #[test]
    fn an_active_spec_with_no_review_is_refused() {
        let dir = project();
        spec(dir.path(), "a.md", &fm("active", "[]"));
        assert_fail_with(
            &rule(dir.path()),
            "NO-REVIEW docs/specifications/a.md: no docs/audits/spec-a-review.json",
        );
    }

    /// The §7 falsifier through the rule. Mutant: the rule hashes something
    /// other than the file's bytes now (the recorded hash, or nothing).
    #[test]
    fn the_section_7_falsifier_appending_one_space_stales_the_review() {
        let dir = project();
        let a = spec(dir.path(), "a.md", &fm("active", "[]"));
        review(dir.path(), &a, &BASE_ROLES);
        append_a_space(dir.path(), &a);
        assert_fail_with(&rule(dir.path()), "STALE-REVIEW docs/specifications/a.md");
    }

    /// §4.3: historical and superseded are exempt, and every verdict names
    /// them. Mutant: the exempt list dropped from the Pass message, or an
    /// exempt spec judged.
    #[test]
    fn a_historical_spec_needs_no_review_and_is_named() {
        let dir = project();
        spec(dir.path(), "old.md", &fm("historical", "[]"));
        let a = spec(dir.path(), "a.md", &fm("active", "[]"));
        review(dir.path(), &a, &BASE_ROLES);
        let c = rule(dir.path());
        assert_eq!(c.status, CheckStatus::Pass, "{}", c.message);
        assert!(
            c.message
                .contains("docs/specifications/old.md (historical)"),
            "{}",
            c.message
        );
    }

    /// A spec whose front-matter does not parse cannot say which roles its
    /// review needs. Mutant: such a spec is silently passed over.
    #[test]
    fn a_spec_whose_front_matter_does_not_parse_is_unjudgeable_not_skipped() {
        let dir = project();
        spec(dir.path(), "a.md", "# no front-matter\n");
        assert_fail_with(&rule(dir.path()), "UNJUDGEABLE docs/specifications/a.md");
    }

    /// §4.3 through the rule. Mutant: the rule passes an empty vendor list to
    /// the judge instead of the front-matter's.
    #[test]
    fn a_front_matter_vendor_is_required_through_the_rule() {
        let dir = project();
        let a = spec(dir.path(), "a.md", &fm("active", "[cuda]"));
        review(dir.path(), &a, &BASE_ROLES);
        assert_fail_with(
            &rule(dir.path()),
            "MISSING-ROLE docs/specifications/a.md: no `vendor:cuda` lane",
        );
    }

    /// Mutant: the unjudgeable specs rendered ahead of the judged ones
    /// instead of in path order.
    #[test]
    fn findings_render_in_path_order_with_their_class_counts() {
        let dir = project();
        spec(dir.path(), "a.md", &fm("active", "[]"));
        spec(dir.path(), "b.md", "# no front-matter\n");
        let c = rule(dir.path());
        assert_fail_with(
            &c,
            "2 finding(s) — NO-REVIEW 1, UNJUDGEABLE 1: NO-REVIEW docs/specifications/a.md",
        );
    }

    /// Adversarial lane: `a/b.md` and `a-b.md` share one artifact path, so
    /// one of them could never pass. The rule names the collision instead of
    /// judging a review against the wrong spec. Mutant: the collision check dropped.
    #[test]
    fn two_active_specs_sharing_an_artifact_path_are_named_as_a_collision() {
        let dir = project();
        spec(dir.path(), "a/b.md", &fm("active", "[]"));
        spec(dir.path(), "a-b.md", &fm("active", "[]"));
        let c = rule(dir.path());
        assert_fail_with(&c, "SLUG-COLLISION docs/specifications/a-b.md:");
        assert_fail_with(&c, "SLUG-COLLISION docs/specifications/a/b.md:");
        assert!(!c.message.contains("NO-REVIEW"), "{}", c.message);
    }
}
