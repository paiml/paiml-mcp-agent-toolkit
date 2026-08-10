// Regression test: `comply check -p <missing>` must say the path is missing.
//
// It used to guard existence with a `debug_assert!`, which is compiled out of
// the released binary. The run then fell through to
// `load_or_create_project_config`, which does `create_dir_all(<path>/.pmat)`,
// so the user saw "Error: Permission denied (os error 13)" and exit 126 — an
// error naming neither the path nor the real problem.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod comply_check_path_guard_tests {
    use crate::cli::commands::ComplyOutputFormat;
    use std::path::PathBuf;

    fn missing_path() -> PathBuf {
        PathBuf::from("/nonexistent-pmat-comply-check-4b81/does/not/exist")
    }

    #[tokio::test]
    async fn comply_check_rejects_a_missing_project_path() {
        let path = missing_path();
        let err = super::handle_check(&path, false, false, ComplyOutputFormat::Text)
            .await
            .expect_err("a missing project path must not produce a compliance report");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("Path not found"),
            "expected the missing-path guard, got: {msg}"
        );
        assert!(
            msg.contains(&path.display().to_string()),
            "error must name the offending path, got: {msg}"
        );
        assert!(
            !msg.contains("Permission denied"),
            "the guard must not be reported as a permissions failure: {msg}"
        );
    }

    #[tokio::test]
    async fn comply_check_does_not_create_a_pmat_dir_under_a_missing_path() {
        // The old code's first filesystem call was a `create_dir_all` under the
        // path it had not checked.
        let path = missing_path();
        let _ = super::handle_check(&path, false, false, ComplyOutputFormat::Text).await;
        assert!(
            !path.join(".pmat").exists(),
            "comply check must not materialise a config dir under a nonexistent path"
        );
    }
}
