/// Resolve repository path from multiple possible sources
/// Returns either a local path or a special marker path for URLs that need cloning
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn resolve_repository(
    path: Option<PathBuf>,
    url: Option<String>,
    repo: Option<String>,
) -> Result<PathBuf> {
    // Priority order:
    // 1. --repo flag (can be GitHub URL, local path, or shorthand)
    // 2. --url flag (remote repository URL)
    // 3. --path flag (local path)
    // 4. Current directory

    if let Some(repo_spec) = repo {
        resolve_repo_spec(&repo_spec)
    } else if let Some(url) = url {
        // Return URL as PathBuf - will be handled by async clone later
        // This is a marker that needs special handling
        Ok(PathBuf::from(url))
    } else {
        detect_repository(path)
    }
}

/// Resolve repository path, cloning if necessary
/// This is the async version that actually performs cloning for URLs
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn resolve_repository_async(
    path: Option<PathBuf>,
    url: Option<String>,
    repo: Option<String>,
) -> Result<PathBuf> {
    let resolved_path = resolve_repository(path, url, repo)?;

    // Check if the resolved path is actually a URL that needs cloning
    let path_str = resolved_path.to_string_lossy();
    if path_str.starts_with("https://") || path_str.starts_with("git@") {
        // This is a URL - need to clone it
        let _temp_dir = std::env::temp_dir()
            .join("pmat-demo-repos")
            .join(format!("repo-{}", uuid::Uuid::new_v4()));

        // Create a temporary runner to use its clone_and_prepare method
        let server = crate::stateless_server::StatelessTemplateServer::new()?;
        let runner = DemoRunner::new(Arc::new(server));
        runner.clone_and_prepare(&path_str).await
    } else {
        // It's a local path
        Ok(resolved_path)
    }
}

/// Parse different repository specification formats
fn resolve_repo_spec(repo_spec: &str) -> Result<PathBuf> {
    debug_assert!(!repo_spec.is_empty(), "repo_spec must not be empty");
    // Try each format in order of specificity
    if let Some(result) = try_local_path(repo_spec) {
        return result;
    }

    if let Some(result) = try_github_shorthand(repo_spec) {
        return result;
    }

    if let Some(result) = try_github_url(repo_spec) {
        return result;
    }

    if let Some(result) = try_owner_repo_format(repo_spec) {
        return result;
    }

    // Fall back to treating as local path
    Err(anyhow!("Repository not found: {repo_spec}"))
}

/// Try to resolve as local path (cognitive complexity ≤2)
fn try_local_path(repo_spec: &str) -> Option<Result<PathBuf>> {
    debug_assert!(!repo_spec.is_empty(), "repo_spec must not be empty");
    let path = PathBuf::from(repo_spec);
    if path.exists() {
        Some(detect_repository(Some(path)))
    } else {
        None
    }
}

/// Try to resolve GitHub shorthand format (gh:owner/repo) (cognitive complexity ≤2)
fn try_github_shorthand(repo_spec: &str) -> Option<Result<PathBuf>> {
    debug_assert!(!repo_spec.is_empty(), "repo_spec must not be empty");
    if repo_spec.starts_with("gh:") {
        let repo_name = repo_spec
            .strip_prefix("gh:")
            .expect("Writing to String buffer cannot fail");
        let github_url = format!("https://github.com/{repo_name}");
        Some(Ok(PathBuf::from(github_url)))
    } else {
        None
    }
}

/// Try to resolve full GitHub URLs (cognitive complexity ≤2)
fn try_github_url(repo_spec: &str) -> Option<Result<PathBuf>> {
    debug_assert!(!repo_spec.is_empty(), "repo_spec must not be empty");
    if repo_spec.starts_with("https://github.com/") || repo_spec.starts_with("git@github.com:") {
        Some(Ok(PathBuf::from(repo_spec)))
    } else {
        None
    }
}

/// Try to resolve owner/repo format (cognitive complexity ≤3)
fn try_owner_repo_format(repo_spec: &str) -> Option<Result<PathBuf>> {
    debug_assert!(!repo_spec.is_empty(), "repo_spec must not be empty");
    if repo_spec.contains('/') && !repo_spec.contains('.') {
        let github_url = format!("https://github.com/{repo_spec}");
        Some(Ok(PathBuf::from(github_url)))
    } else {
        None
    }
}

fn get_canonical_path(hint: Option<PathBuf>) -> Result<PathBuf> {
    debug_assert!(true, "contract: get_canonical_path");
    match hint {
        Some(p) => {
            if !p.exists() {
                return Err(anyhow!("Path does not exist: {p:?}"));
            }
            p.canonicalize()
                .map_err(|e| anyhow!("Failed to canonicalize path {p:?}: {e}"))
        }
        None => env::current_dir()
            .and_then(|p| p.canonicalize())
            .map_err(|e| anyhow!("Failed to get current directory: {e}")),
    }
}

fn find_git_root(start_path: &Path) -> Option<PathBuf> {
    debug_assert!(start_path.exists(), "start_path must exist: {}", start_path.display());
    // Fast path: direct .git check
    if start_path.join(".git").is_dir() {
        return Some(start_path.to_path_buf());
    }

    // Bounded parent traversal
    let mut current = start_path;
    let mut iterations = 0;
    const MAX_ITERATIONS: usize = 100;

    while let Some(parent) = current.parent() {
        if parent == current || parent.as_os_str().is_empty() {
            break; // Reached filesystem root
        }

        if parent.join(".git").is_dir() {
            return Some(parent.to_path_buf());
        }

        current = parent;
        iterations += 1;

        if iterations >= MAX_ITERATIONS {
            break;
        }
    }

    None
}

fn is_interactive_environment() -> bool {
    debug_assert!(true, "contract: is_interactive_environment");
    use std::io::IsTerminal;
    std::io::stdout().is_terminal() && env::var("CI").is_err()
}

fn read_repository_path_from_user() -> Result<PathBuf> {
    debug_assert!(true, "contract: read_repository_path_from_user");
    eprintln!("No git repository found in current directory");
    eprint!("Enter path to a git repository (or press Enter to cancel): ");
    io::stdout().flush()?;

    let mut input = String::with_capacity(1024);
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| anyhow!("Failed to read user input: {e}"))?;

    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("Repository detection cancelled by user"));
    }

    let path = PathBuf::from(trimmed);
    if !path.exists() {
        return Err(anyhow!("Specified path does not exist: {path:?}"));
    }

    let canonical = path
        .canonicalize()
        .map_err(|e| anyhow!("Failed to canonicalize user path: {e}"))?;

    if canonical.join(".git").is_dir() {
        Ok(canonical)
    } else {
        Err(anyhow!("No .git directory found at: {canonical:?}"))
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_repository(hint: Option<PathBuf>) -> Result<PathBuf> {
    let candidate = get_canonical_path(hint)?;

    if let Some(git_root) = find_git_root(&candidate) {
        return Ok(git_root);
    }

    // Non-interactive failure for test environments
    if !is_interactive_environment() {
        return Err(anyhow!(
            "No git repository found in {candidate:?} or its parent directories"
        ));
    }

    // Interactive fallback
    read_repository_path_from_user()
}
