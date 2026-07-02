// Included from check_macs.rs — do NOT add `use` imports or `#!` attributes here.

#[cfg(all(test, not(coverage_nightly)))]
mod tests_macs_skill_effort {
    use super::*;
    use tempfile::tempdir;

    fn write_skill(project: &Path, name: &str, filename: &str, frontmatter: &str) {
        let dir = project.join(".claude").join("skills").join(name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(filename),
            format!("---\n{frontmatter}\n---\n\n# {name}\nbody\n"),
        )
        .unwrap();
    }

    #[test]
    fn cb1650_skips_without_skills_dir() {
        let project = tempdir().unwrap();
        let check = check_skill_effort_pinned(project.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn cb1650_red_on_missing_effort() {
        let project = tempdir().unwrap();
        write_skill(project.path(), "alpha", "SKILL.md", "description: x");
        let check = check_skill_effort_pinned(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("no `effort:`"), "{}", check.message);
    }

    #[test]
    fn cb1650_red_on_session_only_values() {
        let project = tempdir().unwrap();
        write_skill(project.path(), "alpha", "SKILL.md", "effort: max");
        write_skill(project.path(), "beta", "skill.md", "effort: ultracode");
        let check = check_skill_effort_pinned(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("session-only"), "{}", check.message);
        assert!(check.message.contains("alpha") && check.message.contains("beta"));
    }

    #[test]
    fn cb1650_red_on_unknown_value() {
        let project = tempdir().unwrap();
        write_skill(project.path(), "alpha", "SKILL.md", "effort: turbo");
        let check = check_skill_effort_pinned(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("turbo"), "{}", check.message);
    }

    #[test]
    fn cb1650_green_with_trailing_comment_on_effort() {
        // DOGFOOD regression: real skill files write `effort: medium  # note`
        // — the trailing YAML comment must not break the pin match.
        let project = tempdir().unwrap();
        write_skill(
            project.path(),
            "commented",
            "SKILL.md",
            "effort: medium          # MACS F4: pinned for reproducible cost/behavior\ndescription: x",
        );
        let check = check_skill_effort_pinned(project.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn cb1650_green_on_pinned_skills() {
        let project = tempdir().unwrap();
        write_skill(project.path(), "mechanical", "SKILL.md", "effort: medium\ndescription: sweep");
        write_skill(project.path(), "adversarial", "skill.md", "name: x\neffort: xhigh");
        write_skill(project.path(), "io-bound", "skill.md", "effort: low");
        let check = check_skill_effort_pinned(project.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
        assert!(check.message.contains("3 skill(s)"));
    }
}
