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
        // `"name":"pmat"` is load-bearing: this is pmat's OWN manifest, gone
        // stale. Without it the manifest is some other server's and CB-1656
        // correctly declines to score it (see cb1656_skips_a_foreign_manifest).
        write(
            project.path(),
            "mcp.json",
            r#"{"name":"pmat","mcp":{"tools":{"generate_template":{},"generate_unified_context":{}}}}"#,
        );
        let check = check_mcp_manifest_faithful(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("drifted"), "{}", check.message);
    }

    /// #1007: CB-1656 compared every repo's manifest against pmat's compiled-in
    /// tool list, so any other project shipping an MCP server failed by
    /// construction — `missing` was pmat's whole tool set and `extra` was the
    /// repo's whole tool set. Reproduces rmedia's report.
    #[test]
    fn cb1656_skips_a_foreign_manifest() {
        let project = tempdir().unwrap();
        write(
            project.path(),
            "mcp.json",
            r#"{"name":"rmedia","mcp":{"tools":[{"name":"transcode"},{"name":"probe"}]}}"#,
        );
        let check = check_mcp_manifest_faithful(project.path());
        assert_eq!(
            check.status,
            CheckStatus::Skip,
            "a non-pmat server's manifest is not drift in pmat's tool set: {}",
            check.message
        );
        assert!(
            check.message.contains("rmedia"),
            "the skip must name what it found: {}",
            check.message
        );
        // The old failure text must not reappear under any status.
        assert!(
            !check.message.contains("pmat_query_code"),
            "must not demand pmat's tools of another server: {}",
            check.message
        );
    }

    /// A faithful foreign manifest must not be scored WORSE than having no
    /// manifest at all — the old behaviour rewarded deleting the file.
    #[test]
    fn cb1656_does_not_punish_having_a_manifest() {
        let absent = tempdir().unwrap();
        let present = tempdir().unwrap();
        write(
            present.path(),
            "mcp.json",
            r#"{"name":"rmedia","mcp":{"tools":[{"name":"transcode"}]}}"#,
        );
        assert_eq!(
            check_mcp_manifest_faithful(present.path()).status,
            check_mcp_manifest_faithful(absent.path()).status,
            "shipping a faithful non-pmat manifest scored worse than shipping none"
        );
    }

    /// The `{"mcpServers": {…}}` client-config shape carries no `name` and no
    /// `mcp.tools`, so the old code read zero declared tools and reported all
    /// 16 of pmat's as missing.
    #[test]
    fn cb1656_skips_an_mcp_client_config() {
        let project = tempdir().unwrap();
        write(
            project.path(),
            "mcp.json",
            r#"{"mcpServers":{"pmat":{"command":"pmat","args":["mcp"]}}}"#,
        );
        let check = check_mcp_manifest_faithful(project.path());
        assert_eq!(check.status, CheckStatus::Skip, "{}", check.message);
        assert!(check.message.contains("no `name`"), "{}", check.message);
    }

    /// pmat's own committed manifest must still be scored — the narrowing must
    /// not turn this check off where it is the point.
    #[test]
    fn cb1656_still_scores_pmats_own_committed_manifest() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let check = check_mcp_manifest_faithful(root);
        assert_eq!(
            check.status,
            CheckStatus::Pass,
            "pmat's own mcp.json must be scored, not skipped: {}",
            check.message
        );
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
        write(
            project.path(),
            "docs/guide.md",
            "Use claude-3-opus for this.",
        );
        let check = check_doc_model_drift(project.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("guide.md"), "{}", check.message);
    }

    #[test]
    fn cb1657_allowlist_only_in_registry_and_spec() {
        let project = tempdir().unwrap();
        // Deny-listed ids ARE allowed in the registry history + the spec.
        write(
            project.path(),
            "docs/agent-models.md",
            "| claude-3-opus | ... |",
        );
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
        assert_eq!(
            check_roadmap_fresh(project.path()).status,
            CheckStatus::Skip
        );
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
        assert_eq!(
            check_roadmap_fresh(project.path()).status,
            CheckStatus::Pass
        );
    }
}

#[cfg(test)]
mod docs {
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn refactor_auto_references_registry() {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
        let refactor_auto = PathBuf::from(manifest_dir)
            .join("docs")
            .join("features")
            .join("refactor-auto.md");
        if refactor_auto.exists() {
            let content = fs::read_to_string(&refactor_auto).unwrap();
            assert!(
                content.contains("agent-models.md"),
                "refactor-auto.md must point at docs/agent-models.md for model ids"
            );
        }
    }
}
