// Included from mod.rs — do NOT add `use` imports or `#!` attributes here.
//
// PMAT-1614 (FLOW-010 F3): `pmat spec quorum-status` judges receipt files and
// prints the `quorum` status payload; `--post` hands it to `gh api`.

/// Judge `<sha>.<seat>.json` receipts in `dir` and emit the status payload.
/// Exits non-zero when the quorum is not met, so a caller never posts success
/// off a printed failure.
pub fn handle_quorum_status(
    sha: &str,
    dir: &Path,
    repo: Option<&str>,
    post: bool,
) -> anyhow::Result<()> {
    use crate::services::quorum_status::{gh_args, judge, load_dir};
    let receipts = load_dir(dir, sha)?;
    let payload = judge(sha, &receipts);
    println!(
        "{}",
        serde_json::json!({"state": payload.state, "context": payload.context, "description": payload.description})
    );
    if post {
        let repo = repo.ok_or_else(|| anyhow::anyhow!("--post needs --repo"))?;
        let status = std::process::Command::new("gh")
            .args(gh_args(repo, sha, &payload))
            .status()?;
        anyhow::ensure!(status.success(), "gh api exited {status}");
    }
    anyhow::ensure!(
        payload.state == "success",
        "quorum not met: {}",
        payload.description
    );
    Ok(())
}

#[cfg(test)]
mod quorum_status_handler_tests {
    use super::*;

    #[test]
    fn empty_dir_is_not_met_and_post_needs_repo() -> anyhow::Result<()> {
        let d = std::env::temp_dir().join(format!("qsh-{}", std::process::id()));
        std::fs::create_dir_all(&d)?;
        let sha = "0123456789abcdef0123456789abcdef01234567";
        assert!(handle_quorum_status(sha, &d, None, false).is_err());
        let msg = match handle_quorum_status(sha, &d, None, true) {
            Err(e) => e.to_string(),
            Ok(()) => String::new(),
        };
        assert!(msg.contains("--repo"));
        std::fs::remove_dir_all(&d)?;
        Ok(())
    }

    #[test]
    fn missing_dir_is_an_error() {
        assert!(handle_quorum_status("x", Path::new("/nonexistent-qsh"), None, false).is_err());
    }
}
