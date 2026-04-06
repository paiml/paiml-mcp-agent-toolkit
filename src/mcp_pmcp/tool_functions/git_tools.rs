#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn git_clone(
    url: &str,
    target_dir: Option<&Path>,
    _branch: Option<&str>,
    _depth: Option<u32>,
) -> Result<PathBuf> {
    debug_assert!(!url.is_empty(), "url must not be empty");
    // Return the path where it would be cloned
    Ok(target_dir
        .map(std::path::Path::to_path_buf)
        .unwrap_or_else(|| {
            // Extract repo name from URL
            let repo_name = url
                .split('/')
                .next_back()
                .unwrap_or("repo")
                .trim_end_matches(".git");
            PathBuf::from(repo_name)
        }))
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn git_status(path: &Path) -> Result<Value> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    use crate::models::git_context::GitContext;

    // Extract git context from the repository
    let git_context = GitContext::from_current_dir(path)?;

    Ok(json!({
        "status": "completed",
        "message": "Git status retrieved successfully",
        "git_status": {
            "commit_sha": git_context.commit_sha.clone(),
            "commit_sha_short": git_context.commit_sha_short.clone(),
            "branch": git_context.branch.clone(),
            "author_name": git_context.author_name.clone(),
            "author_email": git_context.author_email.clone(),
            "commit_timestamp": git_context.commit_timestamp.to_rfc3339(),
            "commit_message": git_context.commit_message.clone(),
            "tags": git_context.tags.clone(),
            "parent_commits": git_context.parent_commits.clone(),
            "remote_url": git_context.remote_url.clone(),
            "is_clean": git_context.is_clean,
            "uncommitted_files": git_context.uncommitted_files,
        }
    }))
}
