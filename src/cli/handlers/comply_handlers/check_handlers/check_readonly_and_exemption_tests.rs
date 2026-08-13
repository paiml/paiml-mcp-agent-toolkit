// Regression tests for the "observation must not mutate, and absence must be
// representable" defects in comply (#939, #945, #986, #987).
//
// Each test was run against the code as it stood before the fix in the same
// file and observed to FAIL; the failure is quoted above the test.
//
// include!()'d into check.rs scope — no `use` at file scope, no inner
// attributes.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod comply_readonly_and_exemption_tests {
    use std::path::Path;

    /// A two-file crate, plus whatever the caller wants on top.
    fn fixture() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("src")).expect("mkdir src");
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"fx\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .expect("write manifest");
        std::fs::write(dir.path().join("src/lib.rs"), "//! x\n").expect("write lib");
        dir
    }

    /// Every path under `root`, relative and sorted — the unit a read-only
    /// claim is actually checked in. Comparing stdout twice would not have
    /// caught #939; comparing the directory does.
    fn tree(root: &Path) -> Vec<String> {
        fn walk(dir: &Path, base: &Path, out: &mut Vec<String>) {
            let Ok(entries) = std::fs::read_dir(dir) else {
                return;
            };
            for e in entries.flatten() {
                let p = e.path();
                out.push(
                    p.strip_prefix(base)
                        .unwrap_or(&p)
                        .to_string_lossy()
                        .to_string(),
                );
                if p.is_dir() {
                    walk(&p, base, out);
                }
            }
        }
        let mut out = Vec::new();
        walk(root, root, &mut out);
        out.sort();
        out
    }

    // #939. Before: `comply migrate --dry-run` on this fixture left
    // `.pmat` and `.pmat/project.toml` behind — measured with a directory diff
    // — while its own banner printed "(dry-run mode - no changes will be
    // made)". `handle_migrate` opened with `load_or_create_project_config`.
    #[tokio::test]
    async fn comply_migrate_dry_run_writes_nothing_into_the_project() {
        let dir = fixture();
        let before = tree(dir.path());

        crate::cli::handlers::comply_handlers::handle_migrate(dir.path(), None, true, true, true)
            .await
            .expect("dry-run migrate");

        assert_eq!(
            before,
            tree(dir.path()),
            "--dry-run promises no changes; it must not create .pmat/project.toml"
        );
    }

    // #939. Before: `comply diff` — a changelog printer that changes
    // nothing by definition — also created `.pmat/project.toml`, for the sole
    // purpose of reading a default version out of it again.
    #[tokio::test]
    async fn comply_diff_writes_nothing_into_the_project() {
        let dir = fixture();
        let before = tree(dir.path());

        crate::cli::handlers::comply_handlers::handle_diff(dir.path(), None, None, false)
            .await
            .expect("diff");

        assert_eq!(
            before,
            tree(dir.path()),
            "comply diff only prints a changelog; it must not write to the project"
        );
    }

    // #939. Before: `migrate_project_version(.., dry_run = true)` returned
    // `Ok(true)` unconditionally, so a dry run reported "Update project.toml
    // version" as pending even when the pin already equalled the target — the
    // "(no changes needed)" branch was unreachable under --dry-run.
    #[test]
    fn migrate_dry_run_reports_no_change_when_the_pin_already_matches() {
        let dir = fixture();
        std::fs::create_dir_all(dir.path().join(".pmat")).expect("mkdir .pmat");
        std::fs::write(
            dir.path().join(".pmat/project.toml"),
            "[pmat]\nversion = \"9.9.9\"\n",
        )
        .expect("write pin");

        assert!(
            !super::migrate_project_version(dir.path(), "9.9.9", true)
                .expect("dry-run version check"),
            "already at the target version: a dry run must say so, not claim an update"
        );
        assert!(
            super::migrate_project_version(dir.path(), "10.0.0", true)
                .expect("dry-run version check"),
            "a real version difference must still be reported as a pending change"
        );
    }

    // #986. Before: `discover_source_files` filtered on the const
    // `DEFAULT_EXCLUDE_PATTERNS` only, so a project's configured excludes were
    // honoured by the CB-040 check and ignored by the two other consumers of
    // the same file set (cross-stack health, ratchet baseline). One rule, two
    // answers, on the same tree.
    #[test]
    fn configured_excludes_apply_to_the_shared_file_discovery() {
        let dir = fixture();
        std::fs::create_dir_all(dir.path().join("src/codegen")).expect("mkdir");
        std::fs::write(
            dir.path().join("src/codegen/emitted_api.rs"),
            "pub fn f() {}\n".repeat(2500),
        )
        .expect("write generated file");

        let before = super::super::check_extended::discover_source_files(dir.path())
            .expect("discover before");
        assert!(
            before.iter().any(|f| f.ends_with("codegen/emitted_api.rs")),
            "control: with no config the generated file is in the set"
        );

        std::fs::write(
            dir.path().join(".pmat-gates.toml"),
            "[exclude]\npaths = [\"**/codegen/**\"]\n",
        )
        .expect("write gates");

        let after = super::super::check_extended::discover_source_files(dir.path())
            .expect("discover after");
        assert!(
            !after.iter().any(|f| f.ends_with("codegen/emitted_api.rs")),
            "[exclude] paths must remove the file from the ONE discovery every \
             file-health consumer shares, not just from the CB-040 check"
        );
    }

    // #987. Before: a tree whose only markdown lived under docs/archive/
    // came back `Skip: No markdown docs to scan` — a false statement about the
    // tree, and one that hides a load-bearing exemption. `scanned == 0` was
    // reported as "there was nothing", never as "it was all exempt".
    #[test]
    fn cb1657_says_docs_were_exempt_rather_than_absent() {
        let dir = fixture();
        std::fs::create_dir_all(dir.path().join("docs/archive")).expect("mkdir docs");
        std::fs::write(
            dir.path().join("docs/archive/sweep-2026-02-23.md"),
            "# sweep\n\nm if m.contains(\"claude-3-opus\") => (0.015, 0.075),\n",
        )
        .expect("write archived doc");

        let check = super::super::check_macs::check_doc_model_drift(dir.path());
        assert_eq!(check.status, super::CheckStatus::Skip);
        assert!(
            check.message.contains("exempt"),
            "the archived doc must be reported as exempt, not as absent; got: {}",
            check.message
        );
        assert!(
            !check.message.contains("No markdown docs to scan"),
            "there WAS a markdown doc; saying otherwise is a false statement \
             about the tree: {}",
            check.message
        );
    }

    // #987 control: the exemption must not become a hole. A live doc
    // still fails, and a Pass still names the exemptions it granted.
    #[test]
    fn cb1657_still_fails_a_live_doc_and_names_exemptions_on_pass() {
        let dir = fixture();
        std::fs::create_dir_all(dir.path().join("docs/archive")).expect("mkdir docs");
        std::fs::write(
            dir.path().join("docs/archive/old.md"),
            "m if m.contains(\"claude-3-opus\") => (0.015, 0.075),\n",
        )
        .expect("write archived doc");
        std::fs::write(
            dir.path().join("docs/live.md"),
            "We call claude-3-opus in production.\n",
        )
        .expect("write live doc");

        let check = super::super::check_macs::check_doc_model_drift(dir.path());
        assert_eq!(
            check.status,
            super::CheckStatus::Fail,
            "a live doc with a superseded id must still fail: {}",
            check.message
        );

        std::fs::write(dir.path().join("docs/live.md"), "nothing stale here\n")
            .expect("rewrite live doc");
        let check = super::super::check_macs::check_doc_model_drift(dir.path());
        assert_eq!(check.status, super::CheckStatus::Pass);
        assert!(
            check.message.contains("1 exempt"),
            "a Pass earned partly by exemption must say how many docs it \
             exempted: {}",
            check.message
        );
    }

    // CB-1303, found while measuring #945's "unavoidable warnings":
    // `content.contains("edition = \"2021\"")` reported drift on
    // `edition="2021"` (measured) — valid TOML that cargo accepts — and on the
    // workspace-inherited form every monorepo member uses.
    #[test]
    fn cb1303_edition_is_parsed_not_substring_matched() {
        for manifest in [
            "[package]\nname = \"a\"\nedition=\"2021\"\n",
            "[package]\nname = \"a\"\nedition = '2024'\n",
            "[package]\nname = \"a\"\nedition.workspace = true\n",
            "[workspace.package]\nedition = \"2021\"\n",
        ] {
            assert!(
                super::cargo_edition_is_modern(manifest),
                "compliant manifest reported as edition drift:\n{manifest}"
            );
        }
        assert!(
            !super::cargo_edition_is_modern("[package]\nname = \"a\"\nedition = \"2015\"\n"),
            "edition 2015 really is drift and must still be reported"
        );
    }

    // #945: the tri-state is deliberate, and code 2 is reachable ONLY
    // through --strict. Pinned here because the counts used to come from
    // `report.checks`, which `--failures-only` filters, so a display flag
    // silently moved the exit code.
    #[test]
    fn strict_exit_codes_are_a_documented_tri_state() {
        fn report(fail: usize, warn: usize, compliant: bool) -> super::ComplianceReport {
            super::ComplianceReport {
                project_version: "1.0".into(),
                project_version_source: super::VersionSource::PinnedByProject,
                current_version: "1.0".into(),
                is_compliant: compliant,
                versions_behind: 0,
                summary: super::CheckSummary {
                    fail,
                    warn,
                    ..Default::default()
                },
                checks: vec![],
                breaking_changes: vec![],
                recommendations: vec![],
                timestamp: chrono::Utc::now(),
                history: None,
            }
        }
        assert_eq!(super::exit_policy(&report(0, 0, true), true).code, 0);
        assert_eq!(super::exit_policy(&report(0, 3, true), false).code, 0);
        assert_eq!(super::exit_policy(&report(0, 3, true), true).code, 2);
        assert_eq!(super::exit_policy(&report(1, 3, false), true).code, 1);
        assert_eq!(super::exit_policy(&report(1, 3, false), false).code, 1);

        let two = super::exit_policy(&report(0, 3, true), true);
        let reason = two.reason.expect("code 2 must explain itself");
        assert!(
            reason.contains("without --strict") || reason.contains("Without --strict"),
            "code 2 must tell the reader the same run exits 0 without --strict: {reason}"
        );
    }
}
