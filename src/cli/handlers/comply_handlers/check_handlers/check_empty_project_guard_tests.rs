// Regression test: an EMPTY directory is not a compliant project.
//
// `mkdir empty && cd empty && pmat comply check` reported "Project Version:
// 3.30.0 / Versions Behind: 0 / Status: COMPLIANT" over "154 checks (0 fail)",
// exit 0 — the same headline check count this 4260-file repository reports, and
// a better verdict than this repository gets (2 fail, NON-COMPLIANT, exit 1).
// Every one of the 154 checks had skipped for want of anything to look at, and
// the run had written a `.pmat/` directory into the empty tree to invent the
// version it then reported as up to date. A number identical for an empty
// directory and a whole repository measures nothing.
//
// The positive cases go through `no_project_here` rather than `handle_check`:
// `handle_check` ends in `apply_exit_policy`, which calls `process::exit` on a
// non-compliant report and would take the test binary with it.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod comply_check_empty_project_guard_tests {
    use crate::cli::commands::ComplyOutputFormat;

    #[tokio::test]
    async fn comply_check_refuses_a_directory_with_no_project_in_it() {
        let dir = tempfile::tempdir().expect("tempdir");

        let err = super::handle_check(dir.path(), false, false, ComplyOutputFormat::Text)
            .await
            .expect_err("an empty directory must not be reported as COMPLIANT");

        let msg = format!("{err:#}");
        assert!(
            msg.contains("No project found"),
            "the refusal must name the cause, got: {msg}"
        );
        assert!(
            !dir.path().join(".pmat").exists(),
            "comply check must not manufacture its own evidence of a project"
        );
    }

    /// The guard must not become a second, stricter gate: a manifest, a `.git`,
    /// or any readable file is enough to have something to check.
    #[test]
    fn a_directory_with_anything_in_it_is_a_project() {
        for marker in ["Cargo.toml", "README.md", "run.sh"] {
            let dir = tempfile::tempdir().expect("tempdir");
            std::fs::write(dir.path().join(marker), "x\n").expect("marker");
            assert!(
                super::no_project_here(dir.path()).is_none(),
                "{marker} is enough to have something to check"
            );
        }
    }

    /// `.pmat/` is written by comply itself, so it may not count as evidence —
    /// otherwise the second run over an empty directory would be accepted on
    /// the strength of the first run's side effect.
    #[test]
    fn a_directory_holding_only_comply_own_output_is_still_empty() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join(".pmat")).expect("mkdir");
        std::fs::write(dir.path().join(".pmat/project.toml"), "x\n").expect("write");
        assert!(
            super::no_project_here(dir.path()).is_some(),
            "comply must not accept its own scratch directory as a project"
        );
    }
}
