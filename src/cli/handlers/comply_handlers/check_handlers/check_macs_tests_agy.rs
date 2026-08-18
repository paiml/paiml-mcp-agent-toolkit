// Included from check_macs.rs — do NOT add `use` imports or `#!` attributes here.

#[cfg(all(test, not(coverage_nightly)))]
mod tests_macs_agy {
    use super::*;
    use tempfile::{tempdir, TempDir};

    const GOOD_HOOKS: &str = r#"{
      "pmat-quality-feedback": {
        "PreToolUse": [
          {
            "matcher": "write_file|code_execution",
            "hooks": [
              { "type": "command", "command": "./.agents/hooks/pmat-quality-feedback.sh agy" }
            ]
          }
        ]
      }
    }"#;

    const GOOD_MCP: &str = r#"{
      "mcpServers": { "pmat": { "command": "pmat", "args": ["--mode", "mcp"], "env": {} } }
    }"#;

    /// The config this repo actually shipped, and the class of bug CB-1666
    /// exists for: `pmat serve --transport stdio` writes 0 bytes of MCP.
    const DEAD_MCP: &str = r#"{
      "mcpServers": {
        "pmat": {
          "command": "cargo",
          "args": ["run", "--bin", "pmat", "--", "serve", "--transport", "stdio"]
        }
      }
    }"#;

    fn write(root: &Path, rel: &str, body: &str) {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, body).unwrap();
    }

    fn make_executable(path: &Path) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(path).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(path, perms).unwrap();
        }
    }

    /// A `.agents/` tree that every CB-1663..CB-1666 check should pass.
    fn valid_tree() -> TempDir {
        let dir = tempdir().unwrap();
        let root = dir.path();
        write(root, ".agents/hooks.json", GOOD_HOOKS);
        write(root, ".agents/mcp_config.json", GOOD_MCP);
        write(root, ".agents/hooks/pmat-quality-feedback.sh", "#!/bin/sh\nexit 0\n");
        make_executable(&root.join(".agents/hooks/pmat-quality-feedback.sh"));
        write(root, ".agents/rules/house-style.md", "# House style\nBe precise.\n");
        write(
            root,
            ".agents/skills/dogfood/SKILL.md",
            "---\neffort: medium\ndescription: |\n  Dogfood the binary.\n---\n\n# Dogfood\nbody\n",
        );
        dir
    }

    // ---- CB-1663: structure + JSON syntax -------------------------------

    #[test]
    fn cb1663_skips_with_explicit_reason_when_agents_absent() {
        let project = tempdir().unwrap();
        let check = check_agy_structure(project.path());
        assert_eq!(check.status, CheckStatus::Skip);
        // The whole design constraint: a skip must say it judged nothing.
        assert!(check.message.contains("0 AGY artifacts judged"), "{}", check.message);
    }

    #[test]
    fn cb1663_green_on_valid_tree_and_reports_the_count() {
        let dir = valid_tree();
        let check = check_agy_structure(dir.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
        // 2 json + 1 rule + 1 skill dir
        assert!(check.message.contains("judged 4 AGY artifact(s)"), "{}", check.message);
    }

    #[test]
    fn cb1663_red_on_malformed_json() {
        let dir = valid_tree();
        write(dir.path(), ".agents/mcp_config.json", "{ \"mcpServers\": ");
        let check = check_agy_structure(dir.path());
        assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
        assert!(check.message.contains("invalid JSON"), "{}", check.message);
        assert!(check.message.contains(".agents/mcp_config.json"), "{}", check.message);
    }

    #[test]
    fn cb1663_red_on_empty_rule_file() {
        let dir = valid_tree();
        write(dir.path(), ".agents/rules/blank.md", "   \n\n");
        let check = check_agy_structure(dir.path());
        assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
        assert!(check.message.contains("empty rule file"), "{}", check.message);
    }

    #[test]
    fn cb1663_red_on_skill_dir_without_skill_md() {
        let dir = valid_tree();
        std::fs::create_dir_all(dir.path().join(".agents/skills/ghost")).unwrap();
        let check = check_agy_structure(dir.path());
        assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
        assert!(check.message.contains("no SKILL.md"), "{}", check.message);
    }

    #[test]
    fn cb1663_red_when_agents_dir_is_empty_never_pass() {
        // An empty `.agents/` is the exact case a Pass would lie about.
        let project = tempdir().unwrap();
        std::fs::create_dir_all(project.path().join(".agents")).unwrap();
        let check = check_agy_structure(project.path());
        assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
        assert!(check.message.contains("0 artifacts judged"), "{}", check.message);
    }

    #[test]
    fn cb1663_red_when_agents_is_a_file_not_a_directory() {
        let project = tempdir().unwrap();
        std::fs::write(project.path().join(".agents"), "oops").unwrap();
        let check = check_agy_structure(project.path());
        assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
        assert!(check.message.contains("not a directory"), "{}", check.message);
    }

    // ---- CB-1664: hooks.json PreToolUse ---------------------------------

    #[test]
    fn cb1664_skips_when_agents_absent() {
        let project = tempdir().unwrap();
        let check = check_agy_hooks_schema(project.path());
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(check.message.contains("0 hooks judged"), "{}", check.message);
    }

    #[test]
    fn cb1664_skips_naming_the_missing_artifact() {
        let dir = valid_tree();
        std::fs::remove_file(dir.path().join(".agents/hooks.json")).unwrap();
        let check = check_agy_hooks_schema(dir.path());
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(check.message.contains("no hooks.json"), "{}", check.message);
    }

    #[test]
    fn cb1664_green_on_valid_absolute_command() {
        let dir = valid_tree();
        let script = dir.path().join(".agents/hooks/pmat-quality-feedback.sh");
        write(
            dir.path(),
            ".agents/hooks.json",
            &GOOD_HOOKS.replace(
                "./.agents/hooks/pmat-quality-feedback.sh",
                &script.display().to_string(),
            ),
        );
        let check = check_agy_hooks_schema(dir.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
        assert!(check.message.contains("judged 1 hook command(s)"), "{}", check.message);
    }

    #[test]
    fn cb1664_warns_but_does_not_fail_on_a_relative_command() {
        // Relative commands resolve only from the project root; both clients
        // fail open, so a wrong cwd is a silent no-op. Warn, do not pass.
        let dir = valid_tree();
        let check = check_agy_hooks_schema(dir.path());
        assert_eq!(check.status, CheckStatus::Warn, "{}", check.message);
        assert!(check.message.contains("is relative"), "{}", check.message);
        assert!(check.message.contains("judged 1 hook command(s)"), "{}", check.message);
    }

    #[test]
    fn cb1664_red_when_the_hook_script_is_missing() {
        let dir = valid_tree();
        std::fs::remove_file(dir.path().join(".agents/hooks/pmat-quality-feedback.sh")).unwrap();
        let check = check_agy_hooks_schema(dir.path());
        assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
        assert!(check.message.contains("does not exist"), "{}", check.message);
        assert!(check.message.contains("silent no-op"), "{}", check.message);
    }

    #[test]
    fn cb1664_red_when_the_hook_script_is_not_executable() {
        let dir = valid_tree();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let path = dir.path().join(".agents/hooks/pmat-quality-feedback.sh");
            let mut perms = std::fs::metadata(&path).unwrap().permissions();
            perms.set_mode(0o644);
            std::fs::set_permissions(&path, perms).unwrap();
            let check = check_agy_hooks_schema(dir.path());
            assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
            assert!(check.message.contains("not executable"), "{}", check.message);
        }
    }

    #[test]
    fn cb1664_red_on_missing_matcher() {
        let dir = valid_tree();
        write(
            dir.path(),
            ".agents/hooks.json",
            r#"{"h": {"PreToolUse": [{"hooks": [{"type": "command", "command": "echo hi"}]}]}}"#,
        );
        let check = check_agy_hooks_schema(dir.path());
        assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
        assert!(check.message.contains("missing `matcher`"), "{}", check.message);
    }

    /// The flat layout is REPORTED, not gated.
    ///
    /// This asserted `Fail` in its first revision, which made an invented schema
    /// a gating error — including against the very layout this repository
    /// shipped at HEAD. Issue #1031 / PMAT-INIT-002 claim 1 is a single checkbox
    /// ("writes a compliant .agents/hooks.json with PreToolUse schema"); it names
    /// the schema and specifies no shape. The nesting pmat emits was derived from
    /// Claude Code's hook format, a different product, and no Anti-Gravity hook
    /// schema is published anywhere this repository can cite.
    ///
    /// Failing on a guess is exactly what `pmat agy sync` refuses to do
    /// (MACS-017, #984) and what `pmat init` refuses for `plugins.json`. So the
    /// migration is surfaced and the build is not broken over it. Flip this to
    /// `Fail` only when an Anti-Gravity schema exists to point at.
    #[test]
    fn cb1664_warns_but_does_not_gate_on_the_older_flat_layout() {
        let dir = valid_tree();
        write(
            dir.path(),
            ".agents/hooks.json",
            r#"{"hooks":[{"event":"pre_tool_execution","matcher":"write_file",
                "handler":{"type":"command","command":".agents/hooks/x.sh"}}]}"#,
        );
        let check = check_agy_hooks_schema(dir.path());
        assert_eq!(
            check.status,
            CheckStatus::Warn,
            "an unciteable layout preference must not gate a build: {}",
            check.message
        );
        assert!(check.message.contains("flat layout"), "{}", check.message);
        assert!(check.message.contains("PreToolUse"), "{}", check.message);
        // The message must say WHY it is not a failure, or the next reader
        // "fixes" it back into a gate.
        assert!(
            check.message.contains("Reported, not gated"),
            "the message must state that this is not a requirement: {}",
            check.message
        );
    }

    #[test]
    fn cb1664_red_on_malformed_json() {
        let dir = valid_tree();
        write(dir.path(), ".agents/hooks.json", "{ nope");
        let check = check_agy_hooks_schema(dir.path());
        assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
        assert!(check.message.contains("not valid JSON"), "{}", check.message);
    }

    #[test]
    fn cb1664_red_on_empty_object() {
        let dir = valid_tree();
        write(dir.path(), ".agents/hooks.json", "{}");
        let check = check_agy_hooks_schema(dir.path());
        assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
        assert!(check.message.contains("declares 0 hooks"), "{}", check.message);
    }

    // ---- CB-1665: SKILL.md frontmatter ----------------------------------

    #[test]
    fn cb1665_skips_when_agents_absent() {
        let project = tempdir().unwrap();
        let check = check_agy_skill_frontmatter(project.path());
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(check.message.contains("0 skills judged"), "{}", check.message);
    }

    #[test]
    fn cb1665_green_on_valid_frontmatter() {
        let dir = valid_tree();
        let check = check_agy_skill_frontmatter(dir.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
        assert!(check.message.contains("judged 1 SKILL.md"), "{}", check.message);
    }

    #[test]
    fn cb1665_names_the_missing_key() {
        let dir = valid_tree();
        write(
            dir.path(),
            ".agents/skills/dogfood/SKILL.md",
            "---\neffort: medium\n---\n\n# Dogfood\n",
        );
        let check = check_agy_skill_frontmatter(dir.path());
        assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
        assert!(
            check.message.contains("missing required frontmatter key `description`"),
            "{}",
            check.message
        );
    }

    #[test]
    fn cb1665_names_a_missing_effort_key() {
        let dir = valid_tree();
        write(
            dir.path(),
            ".agents/skills/dogfood/SKILL.md",
            "---\ndescription: x\n---\n\n# Dogfood\n",
        );
        let check = check_agy_skill_frontmatter(dir.path());
        assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
        assert!(
            check.message.contains("missing required frontmatter key `effort`"),
            "{}",
            check.message
        );
    }

    #[test]
    fn cb1665_red_on_session_only_effort() {
        let dir = valid_tree();
        write(
            dir.path(),
            ".agents/skills/dogfood/SKILL.md",
            "---\neffort: ultracode\ndescription: x\n---\n",
        );
        let check = check_agy_skill_frontmatter(dir.path());
        assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
        assert!(check.message.contains("session-only"), "{}", check.message);
    }

    #[test]
    fn cb1665_red_on_absent_frontmatter() {
        let dir = valid_tree();
        write(dir.path(), ".agents/skills/dogfood/SKILL.md", "# Dogfood\nno fence\n");
        let check = check_agy_skill_frontmatter(dir.path());
        assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
        assert!(check.message.contains("no `---` fenced"), "{}", check.message);
    }

    #[test]
    fn cb1665_red_on_empty_skills_dir_never_skip() {
        // CB-1650 skips here. That is the Skip-that-reads-like-a-Pass this
        // check deliberately refuses to repeat.
        let dir = valid_tree();
        std::fs::remove_dir_all(dir.path().join(".agents/skills/dogfood")).unwrap();
        let check = check_agy_skill_frontmatter(dir.path());
        assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
        assert!(check.message.contains("0 skills judged"), "{}", check.message);
    }

    #[test]
    fn cb1665_tolerates_a_trailing_comment_and_block_scalar() {
        let dir = valid_tree();
        write(
            dir.path(),
            ".agents/skills/dogfood/SKILL.md",
            "---\neffort: medium          # MACS F4: pinned\n\
             allowed-tools: Bash(cargo:*), Read\n\
             description: |\n  Multi-line\n  body: not a key\n---\n",
        );
        let check = check_agy_skill_frontmatter(dir.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    // ---- CB-1666: mcp_config.json ---------------------------------------

    #[test]
    fn cb1666_skips_when_agents_absent() {
        let project = tempdir().unwrap();
        let check = check_agy_mcp_config(project.path());
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(check.message.contains("0 MCP servers judged"), "{}", check.message);
    }

    #[test]
    fn cb1666_green_on_the_measured_working_entrypoint() {
        let dir = valid_tree();
        let check = check_agy_mcp_config(dir.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
        assert!(check.message.contains("judged 1 MCP server"), "{}", check.message);
    }

    #[test]
    fn cb1666_red_on_the_dead_serve_stdio_entrypoint() {
        let dir = valid_tree();
        write(dir.path(), ".agents/mcp_config.json", DEAD_MCP);
        let check = check_agy_mcp_config(dir.path());
        assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
        assert!(check.message.contains("--transport stdio"), "{}", check.message);
        assert!(check.message.contains("cargo run"), "{}", check.message);
    }

    #[test]
    fn cb1666_red_on_bare_pmat_serve() {
        let dir = valid_tree();
        write(
            dir.path(),
            ".agents/mcp_config.json",
            r#"{"mcpServers":{"pmat":{"command":"pmat","args":["serve"]}}}"#,
        );
        let check = check_agy_mcp_config(dir.path());
        assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
        // The reason bare `pmat serve` is wrong in an mcpServers entry is that
        // it never speaks MCP on **stdio**, which is what an mcpServers entry
        // launches — not that the command is unimplemented. This asserted
        // `NOT IMPLEMENTED`, quoting a help text that denied the shipped
        // streamable-HTTP transport; the finding has to be true of the binary
        // it is describing.
        assert!(
            check.message.contains("never on stdio"),
            "{}",
            check.message
        );
        assert!(
            check.message.contains("--mode") && check.message.contains("mcp"),
            "the finding must name the entrypoint that does work: {}",
            check.message
        );
    }

    #[test]
    fn cb1666_accepts_serve_with_mode_mcp_because_that_one_works() {
        // Measured: `pmat serve --mode mcp` does return a JSON-RPC initialize
        // result. Only the stdio-transport and bare-serve forms are dead.
        let dir = valid_tree();
        write(
            dir.path(),
            ".agents/mcp_config.json",
            r#"{"mcpServers":{"pmat":{"command":"pmat","args":["serve","--mode","mcp"]}}}"#,
        );
        let check = check_agy_mcp_config(dir.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn cb1666_red_on_missing_command() {
        let dir = valid_tree();
        write(
            dir.path(),
            ".agents/mcp_config.json",
            r#"{"mcpServers":{"pmat":{"args":["--mode","mcp"]}}}"#,
        );
        let check = check_agy_mcp_config(dir.path());
        assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
        assert!(check.message.contains("missing `command`"), "{}", check.message);
    }

    #[test]
    fn cb1666_red_on_missing_mcp_servers_key() {
        let dir = valid_tree();
        write(dir.path(), ".agents/mcp_config.json", r#"{"servers":{}}"#);
        let check = check_agy_mcp_config(dir.path());
        assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
        assert!(check.message.contains("no `mcpServers` object"), "{}", check.message);
    }

    #[test]
    fn cb1666_red_on_zero_servers() {
        let dir = valid_tree();
        write(dir.path(), ".agents/mcp_config.json", r#"{"mcpServers":{}}"#);
        let check = check_agy_mcp_config(dir.path());
        assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
        assert!(check.message.contains("declares 0 MCP servers"), "{}", check.message);
    }

    #[test]
    fn cb1666_does_not_score_a_foreign_server_against_pmats_facts() {
        // #1007's lesson: this binary has no source of truth for other servers.
        let dir = valid_tree();
        write(
            dir.path(),
            ".agents/mcp_config.json",
            r#"{"mcpServers":{"other":{"command":"node","args":["serve","--transport","stdio"]}}}"#,
        );
        let check = check_agy_mcp_config(dir.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    // ---- the real tree ---------------------------------------------------

    #[test]
    fn agy_checks_actually_open_this_repos_own_agents_dir() {
        // Dogfood pin. `.agents/hooks.json` is git-tracked here, so both of
        // these must JUDGE something rather than skip. A Skip on a tree that
        // does contain the artifact is the exact failure mode CB-1663..1666
        // exist to prevent, and it is invisible in a summary line.
        let repo = Path::new(env!("CARGO_MANIFEST_DIR"));
        if !repo.join(AGY_DIR).join("hooks.json").is_file() {
            return; // checkout without .agents/hooks.json: nothing to pin
        }
        for check in [check_agy_structure(repo), check_agy_hooks_schema(repo)] {
            assert_ne!(
                check.status,
                CheckStatus::Skip,
                "{} went quiet on this repo's own .agents/: {}",
                check.name,
                check.message
            );
            assert!(
                check.message.contains("judged"),
                "{} reported no judged count: {}",
                check.name,
                check.message
            );
        }
    }
}
