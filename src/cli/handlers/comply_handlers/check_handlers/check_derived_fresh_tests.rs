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

    #[test]
    fn generators_do_not_run_unless_the_rule_is_selected() {
        let decl = "[[artifact]]\npath = \"out.txt\"\ngenerate = [\"touch\", \"ran\"]\n";
        let dir = repo("a\n", "a\n", decl);
        let check = check_derived_fresh(dir.path(), &[]);
        assert_eq!(check.status, CheckStatus::Skip, "{}", check.message);
        assert!(check.message.contains("--checks CB-2120"), "{}", check.message);
        assert!(!dir.path().join("ran").exists(), "a generator ran unasked");
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
        let check = check_derived_fresh(dir.path(), &selected());
        assert!(check.message.contains("not in HEAD"), "{}", check.message);

        let rewrite = "[[artifact]]\npath = \"out.txt\"\ngenerate = [\"cp\", \"src.txt\", \"out.txt\"]\n";
        let dir = repo("a\n", "b\n", rewrite);
        let check = check_derived_fresh(dir.path(), &selected());
        assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
        assert!(check.message.contains("rewrote"), "{}", check.message);
    }
}
