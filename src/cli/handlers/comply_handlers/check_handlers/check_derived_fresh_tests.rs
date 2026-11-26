#[cfg(test)]
mod derived_fresh_tests {
    //! CB-2120 (GH-1641): a declared committed derived file that is stale vs the
    //! tree turns the rule red before the push does.
    use super::*;
    use std::path::Path;

    fn git(root: &Path, args: &[&str]) {
        let ok = std::process::Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["-c", "user.email=t@t", "-c", "user.name=t"])
            .args(["-c", "core.hooksPath=/dev/null", "-c", "commit.gpgsign=false"])
            .args(args)
            .output()
            .expect("git runs")
            .status
            .success();
        assert!(ok, "git {args:?}");
    }

    /// A repository whose `out.txt` is declared as `cat src.txt`.
    fn repo(src: &str, out: &str, decl: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("src.txt"), src).expect("src");
        std::fs::write(dir.path().join("out.txt"), out).expect("out");
        std::fs::write(dir.path().join(DERIVED_FILE), decl).expect("decl");
        git(dir.path(), &["init", "-q"]);
        git(dir.path(), &["add", "-A"]);
        git(dir.path(), &["commit", "-qm", "base"]);
        dir
    }

    const CAT: &str = "[[artifact]]\npath = \"out.txt\"\ngenerate = [\"cat\", \"src.txt\"]\n";

    fn selected() -> Vec<String> {
        vec!["cb-2120".to_string()]
    }

    #[test]
    fn a_stale_artifact_fails_and_names_the_line_and_the_command() {
        let dir = repo("a\nb\n", "a\nc\n", CAT);
        let check = check_derived_fresh(dir.path(), &selected());
        assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
        for want in ["out.txt", "stale", "line 2", "cat src.txt"] {
            assert!(check.message.contains(want), "{want}: {}", check.message);
        }
    }

    #[test]
    fn a_fresh_artifact_passes() {
        let dir = repo("a\n", "a\n", CAT);
        let check = check_derived_fresh(dir.path(), &selected());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn regenerated_but_not_committed_still_fails() {
        let dir = repo("a\n", "old\n", CAT);
        std::fs::write(dir.path().join("out.txt"), "a\n").expect("regen");
        let check = check_derived_fresh(dir.path(), &selected());
        assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
        assert!(check.message.contains("not committed"), "{}", check.message);
    }

    /// The generator touches a sentinel outside the repository and the export,
    /// so the test sees whether it ran wherever it ran.
    fn touch_decl(sentinel: &Path) -> String {
        format!(
            "[[artifact]]\npath = \"out.txt\"\ngenerate = [\"touch\", \"{}\"]\n",
            sentinel.display()
        )
    }

    #[test]
    fn generators_do_not_run_unless_the_rule_is_selected() {
        let marks = tempfile::tempdir().expect("tempdir");
        let sentinel = marks.path().join("ran");
        let dir = repo("a\n", "a\n", &touch_decl(&sentinel));
        let check = check_derived_fresh(dir.path(), &[]);
        assert_eq!(check.status, CheckStatus::Skip, "{}", check.message);
        assert!(check.message.contains("--checks CB-2120"), "{}", check.message);
        assert!(!sentinel.exists(), "a generator ran unasked");
    }

    #[test]
    fn a_selected_generator_runs() {
        let marks = tempfile::tempdir().expect("tempdir");
        let sentinel = marks.path().join("ran");
        let dir = repo("a\n", "a\n", &touch_decl(&sentinel));
        let _ = check_derived_fresh(dir.path(), &selected());
        assert!(sentinel.exists(), "the selected generator did not run");
    }

    #[test]
    fn no_declaration_is_a_skip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let check = check_derived_fresh(dir.path(), &selected());
        assert_eq!(check.status, CheckStatus::Skip, "{}", check.message);
    }

    #[test]
    fn a_declaration_that_cannot_be_measured_fails() {
        for decl in [
            "",
            "artifact = 3\n",
            "[[artifact]]\npath = \"out.txt\"\ngenerate = []\n",
            "[[artifact]]\npath = \"../out.txt\"\ngenerate = [\"cat\", \"src.txt\"]\n",
            "[[artifact]]\npath = \"out.txt\"\ngenerate = [\"cat\", \"src.txt\"]\ntypo = 1\n",
        ] {
            let dir = repo("a\n", "a\n", decl);
            let check = check_derived_fresh(dir.path(), &[]);
            assert_eq!(check.status, CheckStatus::Fail, "{decl:?}: {}", check.message);
        }
    }

    #[test]
    fn a_generator_that_fails_hangs_or_is_missing_fails() {
        for (generate, want) in [
            ("[\"false\"]", "exited"),
            ("[\"no-such-generator-2120\"]", "could not start"),
            ("[\"sleep\", \"30\"]\ntimeout_secs = 1", "timed out"),
        ] {
            let decl = format!("[[artifact]]\npath = \"out.txt\"\ngenerate = {generate}\n");
            let dir = repo("a\n", "a\n", &decl);
            let check = check_derived_fresh(dir.path(), &selected());
            assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
            assert!(check.message.contains(want), "{want}: {}", check.message);
        }
    }

    #[test]
    fn an_uncommitted_artifact_or_a_rewriting_generator_fails() {
        let missing = "[[artifact]]\npath = \"new.txt\"\ngenerate = [\"cat\", \"src.txt\"]\n";
        let dir = repo("a\n", "a\n", missing);
        std::fs::write(dir.path().join("new.txt"), "a\n").expect("uncommitted");
        let check = check_derived_fresh(dir.path(), &selected());
        assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
        assert!(check.message.contains("not in HEAD"), "{}", check.message);

        let rewrite = "[[artifact]]\npath = \"out.txt\"\ngenerate = [\"cp\", \"src.txt\", \"out.txt\"]\n";
        let dir = repo("a\n", "b\n", rewrite);
        let check = check_derived_fresh(dir.path(), &selected());
        assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
        assert!(check.message.contains("rewrote"), "{}", check.message);
    }

    /// REFUTE 1 (GH-1641, 21:25Z): the verdict is about HEAD. An uncommitted edit
    /// to the generator's input must not make a consistent HEAD report stale.
    #[test]
    fn an_uncommitted_input_edit_does_not_fail_a_consistent_head() {
        let dir = repo("a\nb\n", "a\nb\n", CAT);
        std::fs::write(dir.path().join("src.txt"), "a\nc\n").expect("edit");
        let check = check_derived_fresh(dir.path(), &selected());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    /// REFUTE 1: a stale HEAD must not pass just because the uncommitted input
    /// now regenerates to the stale HEAD bytes.
    #[test]
    fn a_stale_head_is_not_hidden_by_a_matching_uncommitted_input() {
        let dir = repo("a\nb\n", "a\nc\n", CAT);
        std::fs::write(dir.path().join("src.txt"), "a\nc\n").expect("edit");
        let check = check_derived_fresh(dir.path(), &selected());
        assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
        assert!(check.message.contains("stale"), "{}", check.message);
    }

    /// The declaration is judged at HEAD too: deleting or breaking the working
    /// copy of `.pmat-derived.toml` must not turn a stale HEAD into a Skip or a
    /// parse failure.
    #[test]
    fn an_uncommitted_declaration_edit_does_not_change_the_verdict() {
        let dir = repo("a\nb\n", "a\nc\n", CAT);
        std::fs::remove_file(dir.path().join(DERIVED_FILE)).expect("delete");
        let check = check_derived_fresh(dir.path(), &selected());
        assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
        std::fs::write(dir.path().join(DERIVED_FILE), "garbage").expect("break");
        let check = check_derived_fresh(dir.path(), &selected());
        assert!(check.message.contains("stale"), "{}", check.message);
    }

    /// Uncommitted `.gitattributes` must not change the exported bytes: the
    /// verdict is the committed bytes, not a filtered checkout.
    #[test]
    fn an_uncommitted_attribute_does_not_make_a_consistent_head_stale() {
        let dir = repo("a\nb\n", "a\nb\n", CAT);
        std::fs::write(dir.path().join(".gitattributes"), "*.txt text eol=crlf\n").expect("attr");
        let check = check_derived_fresh(dir.path(), &selected());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    /// A project below the repository root: its paths are relative to itself,
    /// and the export still holds the whole of HEAD.
    #[test]
    fn a_project_below_the_repository_root_is_judged_at_its_own_root() {
        let top = tempfile::tempdir().expect("tempdir");
        let sub = top.path().join("sub");
        std::fs::create_dir(&sub).expect("sub");
        std::fs::write(top.path().join("root.txt"), "root\n").expect("root");
        std::fs::write(sub.join("src.txt"), "a\nb\n").expect("src");
        std::fs::write(sub.join("out.txt"), "a\nb\n").expect("out");
        std::fs::write(sub.join(DERIVED_FILE), CAT).expect("decl");
        git(top.path(), &["init", "-q"]);
        git(top.path(), &["add", "-A"]);
        git(top.path(), &["commit", "-qm", "base"]);
        let check = check_derived_fresh(&sub, &selected());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
        std::fs::write(sub.join("out.txt"), "a\nc\n").expect("stale");
        git(top.path(), &["commit", "-qam", "stale"]);
        let check = check_derived_fresh(&sub, &selected());
        assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
        assert!(check.message.contains("stale"), "{}", check.message);
    }

    /// A same-bytes rewrite is still a rewrite: the generator must print.
    #[test]
    fn a_generator_that_rewrites_the_same_bytes_fails() {
        let decl = "[[artifact]]\npath = \"out.txt\"\ngenerate = [\"cp\", \"src.txt\", \"out.txt\"]\n";
        let dir = repo("a\n", "a\n", decl);
        let check = check_derived_fresh(dir.path(), &selected());
        assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
        assert!(check.message.contains("rewrote"), "{}", check.message);
    }
}
