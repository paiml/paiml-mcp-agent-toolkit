// Included from check_macs.rs — do NOT add `use` imports or `#!` attributes here.

#[cfg(all(test, not(coverage_nightly)))]
mod tests_macs_artifacts {
    use super::*;
    use tempfile::tempdir;

    fn write(project: &Path, rel: &str, body: &str) {
        let path = project.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }

    #[test]
    fn cb1656_skips_without_manifest() {
        let project = tempdir().unwrap();
        assert_eq!(
            check_mcp_manifest_faithful(project.path()).status,
            CheckStatus::Skip
        );
    }

    #[test]
    fn cb1656_red_on_legacy_two_tool_manifest() {
        let project = tempdir().unwrap();
        write(
            project.path(),
            "mcp.json",
            r#"{"mcp":{"tools":{"generate_template":{},"generate_unified_context":{}}}}"#,
        );
        let check = check_mcp_manifest_faithful(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("drifted"), "{}", check.message);
    }

    #[test]
    fn cb1656_green_on_generated_manifest() {
        let project = tempdir().unwrap();
        write(
            project.path(),
            "mcp.json",
            &crate::mcp_pmcp::tool_manifest::render_manifest("9.9.9"),
        );
        let check = check_mcp_manifest_faithful(project.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn cb1656_green_ignores_version_churn() {
        // A different version string must NOT cause drift.
        let project = tempdir().unwrap();
        write(
            project.path(),
            "mcp.json",
            &crate::mcp_pmcp::tool_manifest::render_manifest("1.0.0"),
        );
        assert_eq!(
            check_mcp_manifest_faithful(project.path()).status,
            CheckStatus::Pass
        );
    }

    #[test]
    fn cb1657_red_on_denylist_hit() {
        let project = tempdir().unwrap();
        write(project.path(), "docs/guide.md", "Use claude-3-opus for this.");
        let check = check_doc_model_drift(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("guide.md"), "{}", check.message);
    }

    #[test]
    fn cb1657_allowlist_only_in_registry_and_spec() {
        let project = tempdir().unwrap();
        // Deny-listed ids ARE allowed in the registry history + the spec.
        write(project.path(), "docs/agent-models.md", "| claude-3-opus | ... |");
        write(
            project.path(),
            "docs/specifications/components/modern-agentic-coding-support.md",
            "refactor-auto.md ships claude-3-opus and gpt-4-turbo",
        );
        write(project.path(), "docs/clean.md", "Use claude-fable-5.");
        let check = check_doc_model_drift(project.path());
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn cb1657_denylist_scans_all_occurrences() {
        // ADVERSARIAL-REVIEW regression: an allowed claude-2099 earlier on the
        // line must not mask a denied claude-2 later on the same line.
        assert!(macs_line_has_denied_model(
            "compare claude-2099-future with legacy claude-2 here"
        ));
    }

    #[test]
    fn cb1657_denylist_matching() {
        assert!(macs_line_has_denied_model("primary: claude-3-opus"));
        assert!(macs_line_has_denied_model("gpt-4-turbo fallback"));
        assert!(macs_line_has_denied_model("legacy claude-2 model"));
        // Not denied: current ids, and future claude-2NNN-style ids.
        assert!(!macs_line_has_denied_model("claude-fable-5"));
        assert!(!macs_line_has_denied_model("claude-opus-4-8"));
        assert!(!macs_line_has_denied_model("claude-2099-future"));
    }
}

#[cfg(all(test, not(coverage_nightly)))]
mod tests_macs_roadmap_fresh {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn cb1655_skips_without_ledger() {
        let project = tempdir().unwrap();
        assert_eq!(check_roadmap_fresh(project.path()).status, CheckStatus::Skip);
    }

    #[test]
    fn cb1655_warn_when_roadmap_missing() {
        let project = tempdir().unwrap();
        std::fs::create_dir_all(project.path().join(".pmat-work")).unwrap();
        std::fs::write(project.path().join(".pmat-work/ledger.jsonl"), "{}\n").unwrap();
        let check = check_roadmap_fresh(project.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("roadmap sync"), "{}", check.message);
    }

    #[test]
    fn cb1655_pass_when_fresh() {
        let project = tempdir().unwrap();
        std::fs::create_dir_all(project.path().join(".pmat-work")).unwrap();
        std::fs::write(project.path().join(".pmat-work/ledger.jsonl"), "{}\n").unwrap();
        std::fs::write(project.path().join("ROADMAP.yaml"), "items:\n").unwrap();
        assert_eq!(check_roadmap_fresh(project.path()).status, CheckStatus::Pass);
    }
}
