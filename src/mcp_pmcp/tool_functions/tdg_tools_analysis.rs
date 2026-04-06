/// Analyze Technical Debt Grading (TDG) scores using the new TDG implementation
pub async fn analyze_tdg(
    paths: &[PathBuf],
    threshold: Option<f64>,
    top_files: Option<usize>,
    include_components: Option<bool>,
    with_git_context: Option<bool>, // Sprint 65: Git-commit correlation
) -> Result<Value> {
    use crate::tdg::TdgAnalyzer;

    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

    let mut analyzer = TdgAnalyzer::new()?;
    let _threshold = threshold.unwrap_or(1.5);
    let _top_files = top_files.unwrap_or(10);
    let _include_components = include_components.unwrap_or(false);

    // Sprint 65: Extract git context if requested
    if with_git_context.unwrap_or(false) && !paths.is_empty() {
        let git_context = crate::models::git_context::GitContext::try_from_current_dir(&paths[0]);
        analyzer.set_git_context(git_context);
    }

    if paths.len() == 1 {
        analyze_single_tdg_path(&analyzer, &paths[0]).await
    } else {
        analyze_multiple_tdg_paths(&analyzer, paths).await
    }
}

async fn analyze_single_tdg_path(analyzer: &crate::tdg::TdgAnalyzer, path: &Path) -> Result<Value> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    // Sprint 65: Get git context from analyzer for output
    let git_context = analyzer.get_git_context();

    if PathValidator::ensure_directory(path).is_ok() {
        let project_score = analyzer.analyze_project(path).await?;
        Ok(json!({
            "status": "completed",
            "message": "TDG project analysis completed",
            "result_type": "project",
            "results": {
                "average_score": project_score.average_score,
                "average_grade": project_score.average_grade,
                "total_files": project_score.total_files,
                "language_distribution": project_score.language_distribution,
                "files": project_score.files
            },
            "git_context": git_context.map(|git| json!({
                "commit_sha": git.commit_sha,
                "commit_sha_short": git.commit_sha_short,
                "branch": git.branch,
                "author_name": git.author_name,
                "author_email": git.author_email,
                "commit_timestamp": git.commit_timestamp.to_rfc3339(),
                "commit_message": git.commit_message,
                "tags": git.tags,
                "is_clean": git.is_clean,
                "uncommitted_files": git.uncommitted_files,
            }))
        }))
    } else {
        let score = analyzer.analyze_file(path).await?;
        Ok(json!({
            "status": "completed",
            "message": "TDG file analysis completed",
            "result_type": "file",
            "results": score,
            "git_context": git_context.map(|git| json!({
                "commit_sha": git.commit_sha,
                "commit_sha_short": git.commit_sha_short,
                "branch": git.branch,
                "author_name": git.author_name,
                "author_email": git.author_email,
                "commit_timestamp": git.commit_timestamp.to_rfc3339(),
                "commit_message": git.commit_message,
                "tags": git.tags,
                "is_clean": git.is_clean,
                "uncommitted_files": git.uncommitted_files,
            }))
        }))
    }
}

async fn analyze_multiple_tdg_paths(
    analyzer: &crate::tdg::TdgAnalyzer,
    paths: &[PathBuf],
) -> Result<Value> {
    use crate::tdg::ProjectScore;
    let mut all_scores = Vec::new();

    // Sprint 65: Get git context from analyzer for output
    let git_context = analyzer.get_git_context();

    for path in paths {
        if PathValidator::ensure_directory(path).is_ok() {
            let project_score = analyzer.analyze_project(path).await?;
            all_scores.extend(project_score.files);
        } else {
            let score = analyzer.analyze_file(path).await?;
            all_scores.push(score);
        }
    }

    let aggregated = ProjectScore::aggregate(all_scores);
    Ok(json!({
        "status": "completed",
        "message": "TDG multi-path analysis completed",
        "result_type": "multi_path",
        "results": {
            "average_score": aggregated.average_score,
            "average_grade": aggregated.average_grade,
            "total_files": aggregated.total_files,
            "language_distribution": aggregated.language_distribution,
            "files": aggregated.files
        },
        "git_context": git_context.map(|git| json!({
            "commit_sha": git.commit_sha,
            "commit_sha_short": git.commit_sha_short,
            "branch": git.branch,
            "author_name": git.author_name,
            "author_email": git.author_email,
            "commit_timestamp": git.commit_timestamp.to_rfc3339(),
            "commit_message": git.commit_message,
            "tags": git.tags,
            "is_clean": git.is_clean,
            "uncommitted_files": git.uncommitted_files,
        }))
    }))
}

/// Compare TDG scores between two files or directories
pub async fn compare_tdg(
    path1: &Path,
    path2: &Path,
    with_git_context: Option<bool>, // Sprint 65: Git-commit correlation
) -> Result<Value> {
    debug_assert!(path1.exists(), "path1 must exist: {}", path1.display());
    debug_assert!(path2.exists(), "path2 must exist: {}", path2.display());
    use crate::tdg::TdgAnalyzer;

    let mut analyzer = TdgAnalyzer::new()?;

    // Sprint 65: Extract git context if requested (uses first path as reference)
    if with_git_context.unwrap_or(false) {
        let git_context = crate::models::git_context::GitContext::try_from_current_dir(path1);
        analyzer.set_git_context(git_context.clone());
    }

    let comparison = analyzer.compare(path1, path2).await?;
    let git_context = analyzer.get_git_context();

    Ok(json!({
        "status": "completed",
        "message": "TDG comparison completed",
        "result_type": "comparison",
        "results": comparison,
        "git_context": git_context.map(|git| json!({
            "commit_sha": git.commit_sha,
            "commit_sha_short": git.commit_sha_short,
            "branch": git.branch,
            "author_name": git.author_name,
            "author_email": git.author_email,
            "commit_timestamp": git.commit_timestamp.to_rfc3339(),
            "commit_message": git.commit_message,
            "tags": git.tags,
            "is_clean": git.is_clean,
            "uncommitted_files": git.uncommitted_files,
        }))
    }))
}
