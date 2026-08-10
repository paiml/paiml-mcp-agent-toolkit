use crate::cli::commands::AgyCommands;
use std::path::Path;

pub async fn handle_agy_command(cmd: &AgyCommands, _base_path: &Path) -> anyhow::Result<()> {
    match cmd {
        AgyCommands::Sync { work_dir, out_dir } => sync_contracts_to_agy(work_dir, out_dir),
    }
}

/// `agy sync` used to be a single `println!` of a ✅ success banner followed by
/// `// TODO(MACS-017)` and `Ok(())`: it never read `--work-dir`, never created
/// `--out-dir`, and reported success even for `--work-dir /does/not/exist`.
/// There is no PMAT→Anti-Gravity transpiler behind it, so the command now fails
/// loudly. A command that does nothing must not print ✅ and exit 0.
fn sync_contracts_to_agy(work_dir: &Path, out_dir: &Path) -> anyhow::Result<()> {
    if !work_dir.exists() {
        anyhow::bail!(
            "agy sync: contract directory not found: {}",
            work_dir.display()
        );
    }

    anyhow::bail!(
        "agy sync is not implemented: no PMAT -> Google Anti-Gravity transpiler exists yet, \
         so nothing was written to {}. Contracts under {} were left untouched. \
         Track this under MACS-017.",
        out_dir.display(),
        work_dir.display()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::commands::AgyCommands;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_sync_errors_when_work_dir_is_missing() {
        let cmd = AgyCommands::Sync {
            work_dir: PathBuf::from("/does/not/exist/pmat-work"),
            out_dir: PathBuf::from("/does/not/exist/outdir"),
        };

        let err = handle_agy_command(&cmd, Path::new(""))
            .await
            .expect_err("a nonexistent --work-dir must be an error, not a success banner");
        assert!(
            err.to_string().contains("not found"),
            "unexpected error: {err}"
        );
    }

    #[tokio::test]
    async fn test_sync_does_not_report_success_without_writing_anything() {
        let temp = tempfile::TempDir::new().unwrap();
        let work_dir = temp.path().join(".pmat-work");
        std::fs::create_dir_all(&work_dir).unwrap();
        std::fs::write(
            work_dir.join("CB-001.yaml"),
            "id: CB-001\ntitle: Example contract\n",
        )
        .unwrap();
        let out_dir = temp.path().join("outdir");

        let cmd = AgyCommands::Sync {
            work_dir: work_dir.clone(),
            out_dir: out_dir.clone(),
        };

        let err = handle_agy_command(&cmd, Path::new(""))
            .await
            .expect_err("an unimplemented transpile must not return Ok");
        assert!(
            err.to_string().contains("not implemented"),
            "unexpected error: {err}"
        );
        assert!(
            !out_dir.exists(),
            "nothing was transpiled, so --out-dir must not be claimed as written"
        );
    }
}
