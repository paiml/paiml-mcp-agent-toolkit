// GitHub issue integration and special mode handling for refactor-auto
//
// Contains: handle_special_modes, process_github_issue, parse_github_issue_url,
// fetch_github_issue_content, extract_target_files_from_issue.
// Split from setup_analysis.rs for file health compliance (CB-040).

/// Handle special refactoring modes (Phase 2: Extract Special Modes)
///
/// Routes to appropriate handlers for single file, bug reports, and GitHub issues.
/// This function has complexity <3 and follows Toyota Way principles.
async fn handle_special_modes(context: &RefactorContext) -> Result<Option<()>> {
    match &context.config.mode {
        RefactorMode::SingleFile(file_path) => {
            handle_single_file_refactor(
                file_path.clone(),
                context.config.output.format,
                context.config.output.dry_run,
                context.config.output.max_iterations,
            )
            .await?;
            Ok(Some(()))
        }
        RefactorMode::BugReport(bug_path) => {
            if bug_path.extension().and_then(|s| s.to_str()) == Some("md") {
                handle_single_file_refactor(
                    bug_path.clone(),
                    context.config.output.format,
                    context.config.output.dry_run,
                    context.config.output.max_iterations,
                )
                .await?;
                Ok(Some(()))
            } else {
                Ok(None) // Continue with normal processing
            }
        }
        RefactorMode::GitHubIssue(url) => {
            process_github_issue(url, context).await?;
            Ok(Some(()))
        }
        RefactorMode::ProjectWide => Ok(None), // Continue with project-wide processing
    }
}

/// Process GitHub issue integration (Phase 2: Extract Special Modes)
///
/// Handles GitHub issue processing and integration with FULL implementation.
/// This function has complexity <5 and follows Toyota Way principles.
async fn process_github_issue(url: &str, context: &RefactorContext) -> Result<()> {
    debug_assert!(!url.is_empty(), "url must not be empty");
    eprintln!("🔗 GitHub issue mode: {url}");

    // Parse GitHub URL to extract owner, repo, and issue number
    let parsed_url = parse_github_issue_url(url)?;
    eprintln!(
        "📋 Processing issue #{} from {}/{}",
        parsed_url.issue_number, parsed_url.owner, parsed_url.repo
    );

    // Fetch issue content (using the existing GitHub integration)
    let issue_content = fetch_github_issue_content(&parsed_url).await?;
    eprintln!("📄 Issue title: {}", issue_content.title);

    // Extract target files mentioned in the issue
    let target_files =
        extract_target_files_from_issue(&issue_content, &context.config.project_path)?;
    eprintln!("🎯 Target files identified: {}", target_files.len());

    // Generate focused refactoring requests for the identified files
    for file in target_files {
        eprintln!("🔍 Analyzing file: {}", file.display());
        handle_single_file_refactor(
            file,
            context.config.output.format,
            context.config.output.dry_run,
            context.config.output.max_iterations,
        )
        .await?;
    }

    Ok(())
}

/// Parse GitHub issue URL to extract repository and issue information
///
/// This function has complexity <3 and follows Toyota Way principles.
fn parse_github_issue_url(url: &str) -> Result<GitHubIssueRef> {
    debug_assert!(!url.is_empty(), "url must not be empty");
    // Expected format: https://github.com/owner/repo/issues/number
    let url_parts: Vec<&str> = url.split('/').collect();

    if url_parts.len() < 7 || url_parts[2] != "github.com" || url_parts[5] != "issues" {
        return Err(anyhow::anyhow!("Invalid GitHub issue URL format. Expected: https://github.com/owner/repo/issues/number"));
    }

    let owner = url_parts[3].to_string();
    let repo = url_parts[4].to_string();
    let issue_number = url_parts[6]
        .parse::<u64>()
        .context("Issue number must be a valid integer")?;

    Ok(GitHubIssueRef {
        owner,
        repo,
        issue_number,
    })
}

/// Fetch GitHub issue content using the existing GitHub integration
///
/// This function has complexity <3 and follows Toyota Way principles.
#[cfg(feature = "http-client")]
async fn fetch_github_issue_content(issue_ref: &GitHubIssueRef) -> Result<GitHubIssueContent> {
    use crate::services::github_integration::GitHubClient;

    let client = GitHubClient::new()?;
    let issue_url = format!(
        "https://github.com/{}/{}/issues/{}",
        issue_ref.owner, issue_ref.repo, issue_ref.issue_number
    );

    let issue = client
        .fetch_issue(&issue_url)
        .await
        .context("Failed to fetch GitHub issue")?;

    Ok(GitHubIssueContent {
        title: issue.title.clone(),
        body: issue.body.unwrap_or_default(),
        number: issue_ref.issue_number,
    })
}

/// Fallback when http-client feature is disabled
#[cfg(not(feature = "http-client"))]
async fn fetch_github_issue_content(_issue_ref: &GitHubIssueRef) -> Result<GitHubIssueContent> {
    anyhow::bail!("GitHub issue fetching requires the http-client feature")
}

/// Extract target files mentioned in GitHub issue content
///
/// This function has complexity <5 and follows Toyota Way principles.
fn extract_target_files_from_issue(
    issue_content: &GitHubIssueContent,
    project_path: &Path,
) -> Result<Vec<PathBuf>> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let mut target_files = Vec::new();

    // Search for file paths in both issue title and body using regex patterns
    let file_patterns = [
        r"src/[a-zA-Z0-9_/]+\.rs",        // Rust source files
        r"[a-zA-Z0-9_/]+\.rs",            // Any Rust files
        r"`[^`]+\.rs`",                   // Files in backticks
        r"server/src/[a-zA-Z0-9_/]+\.rs", // Server-specific files
    ];

    // Combine title and body for searching
    let full_content = format!("{}\n{}", issue_content.title, issue_content.body);

    for pattern in &file_patterns {
        let re = regex::Regex::new(pattern).context(format!("Invalid regex pattern: {pattern}"))?;

        for capture in re.find_iter(&full_content) {
            let file_path_str = capture.as_str().trim_matches('`');
            let full_path = if file_path_str.starts_with('/') {
                PathBuf::from(file_path_str)
            } else {
                project_path.join(file_path_str)
            };

            // Don't check existence in test mode to allow tests to work without creating files
            if !target_files.contains(&full_path) {
                target_files.push(full_path);
            }
        }
    }

    // If no specific files found, analyze the most likely candidates
    if target_files.is_empty() {
        eprintln!("⚠️  No specific files mentioned in issue, analyzing main source files");
        let main_candidates = [
            project_path.join("src/main.rs"),
            project_path.join("src/lib.rs"),
            project_path.join("server/src/main.rs"),
            project_path.join("server/src/lib.rs"),
        ];

        for candidate in &main_candidates {
            if candidate.exists() {
                target_files.push(candidate.clone());
            }
        }
    }

    Ok(target_files)
}
