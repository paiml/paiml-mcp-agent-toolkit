use anyhow::Result;
use git2::{build::RepoBuilder, FetchOptions, Progress, RemoteCallbacks, Repository};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::Instant;

lazy_static! {
    // Pre-compiled regex patterns for GitHub URL parsing
    // Name pattern: alphanumeric at start/end, can contain dash, underscore, dot in middle
    // Single char names are also valid
    static ref NAME_PATTERN: &'static str = r"[a-zA-Z0-9](?:[a-zA-Z0-9\-_\.]*[a-zA-Z0-9])?";

    static ref GITHUB_HTTPS_REGEX: Regex = {
        Regex::new(&format!(
            r"^https://github\.com/({name})/({name})(?:\.git)?/?$",
            name = *NAME_PATTERN
        ))
        .expect("Invalid HTTPS regex pattern")
    };

    static ref GITHUB_SSH_REGEX: Regex = {
        Regex::new(&format!(
            r"^git@github\.com:({name})/({name})(?:\.git)?$",
            name = *NAME_PATTERN
        ))
        .expect("Invalid SSH regex pattern")
    };

    static ref GITHUB_SHORT_REGEX: Regex = {
        Regex::new(&format!(
            r"^({name})/({name})$",
            name = *NAME_PATTERN
        ))
        .expect("Invalid short format regex pattern")
    };
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CloneProgress {
    pub stage: String,
    pub current: usize,
    pub total: usize,
    pub bytes_transferred: usize,
}

#[derive(Clone, Debug)]
pub struct ClonedRepo {
    pub path: PathBuf,
    pub url: String,
    pub cached: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum CloneError {
    #[error("Git error: {0}")]
    GitError(#[from] git2::Error),

    #[error("Repository too large: {size_mb}MB exceeds limit")]
    TooLarge { size_mb: u64 },

    #[error("Clone operation timed out")]
    Timeout,

    #[error("Invalid GitHub URL: {0}")]
    InvalidUrl(String),

    #[error("GitHub API error: {0}")]
    ApiError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Clone)]
pub struct GitCloner {
    cache_dir: PathBuf,
    progress: Arc<Mutex<CloneProgress>>,
    timeout: Duration,
    max_size_bytes: u64,
}

impl GitCloner {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            cache_dir,
            progress: Arc::new(Mutex::new(CloneProgress {
                stage: "Initializing".to_string(),
                current: 0,
                total: 0,
                bytes_transferred: 0,
            })),
            timeout: Duration::from_secs(300), // 5 minutes default
            max_size_bytes: 500_000_000,       // 500MB default
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_max_size(mut self, max_size_bytes: u64) -> Self {
        self.max_size_bytes = max_size_bytes;
        self
    }

    pub async fn get_progress(&self) -> CloneProgress {
        self.progress.lock().await.clone()
    }

    pub async fn clone_or_update(&self, url: &str) -> Result<ClonedRepo, CloneError> {
        // Validate URL format
        let _parsed_url = self.parse_github_url(url)?;

        // Check repository size via GitHub API (optional, requires API token)
        // For now, we'll skip this and rely on the clone timeout

        let cache_key = self.compute_cache_key(url);
        let target_path = self.cache_dir.join(&cache_key);

        // Check if already cached and fresh
        if target_path.exists() {
            if let Ok(repo) = Repository::open(&target_path) {
                // Check if repository is valid and relatively fresh
                if self.is_cache_fresh(&repo).await.unwrap_or(false) {
                    return Ok(ClonedRepo {
                        path: target_path,
                        url: url.to_string(),
                        cached: true,
                    });
                }

                // Try to update existing repository
                if self.update_repository(&repo).await.is_ok() {
                    return Ok(ClonedRepo {
                        path: target_path,
                        url: url.to_string(),
                        cached: true,
                    });
                }
            }

            // If we can't open or update, remove and re-clone
            let _ = tokio::fs::remove_dir_all(&target_path).await;
        }

        // Create cache directory if it doesn't exist
        tokio::fs::create_dir_all(&self.cache_dir)
            .await
            .map_err(CloneError::IoError)?;

        // Clone with timeout
        let progress = self.progress.clone();
        let url_clone = url.to_string();
        let target_clone = target_path.clone();

        let clone_future = tokio::task::spawn_blocking(move || {
            // Create a temporary cloner for the blocking task
            let temp_cloner = GitCloner {
                cache_dir: PathBuf::new(), // Not used in clone_shallow
                progress,
                timeout: Duration::from_secs(300),
                max_size_bytes: 0,
            };
            temp_cloner.clone_shallow(&url_clone, &target_clone)
        });

        let _start = Instant::now();
        let result = tokio::select! {
            result = clone_future => {
                match result {
                    Ok(Ok(_)) => Ok(ClonedRepo {
                        path: target_path.clone(),
                        url: url.to_string(),
                        cached: false,
                    }),
                    Ok(Err(e)) => Err(e),
                    Err(e) => Err(CloneError::GitError(git2::Error::from_str(&e.to_string()))),
                }
            }
            _ = tokio::time::sleep(self.timeout) => {
                Err(CloneError::Timeout)
            }
        };

        // Clean up on failure
        if result.is_err() && target_path.exists() {
            let _ = tokio::fs::remove_dir_all(&target_path).await;
        }

        result
    }

    fn clone_shallow(&self, url: &str, target: &Path) -> Result<(), CloneError> {
        let progress = self.progress.clone();

        // Set up fetch options
        let mut fetch_opts = FetchOptions::new();
        fetch_opts.depth(1); // Shallow clone

        // Set up callbacks for progress reporting
        let mut callbacks = RemoteCallbacks::new();
        callbacks.transfer_progress(move |stats: Progress| {
            let progress_update = CloneProgress {
                stage: "Receiving objects".to_string(),
                current: stats.received_objects(),
                total: stats.total_objects(),
                bytes_transferred: stats.received_bytes(),
            };

            // Update progress (blocking is ok here since we're in a callback)
            if let Ok(mut p) = progress.try_lock() {
                *p = progress_update;
            }
            true
        });

        fetch_opts.remote_callbacks(callbacks);

        // Configure the repository builder
        let mut builder = RepoBuilder::new();
        // Don't specify a branch - let git2 figure out the default branch
        builder.fetch_options(fetch_opts);

        // Perform the clone
        builder.clone(url, target).map_err(CloneError::GitError)?;

        Ok(())
    }

    async fn update_repository(&self, repo: &Repository) -> Result<()> {
        // This is a simplified update - in production you'd want more sophisticated logic
        let mut remote = repo.find_remote("origin")?;

        let mut fetch_opts = FetchOptions::new();
        fetch_opts.download_tags(git2::AutotagOption::All);

        remote.fetch(&["HEAD"], Some(&mut fetch_opts), None)?;

        // Fast-forward to origin/HEAD if possible
        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;

        let analysis = repo.merge_analysis(&[&fetch_commit])?;

        if analysis.0.is_fast_forward() {
            let refname = "refs/heads/master"; // Assuming master branch
            let mut reference = repo.find_reference(refname)?;
            reference.set_target(fetch_commit.id(), "Fast-forward")?;
            repo.set_head(refname)?;
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
        }

        Ok(())
    }

    async fn is_cache_fresh(&self, _repo: &Repository) -> Result<bool> {
        // Check if the cached repository is less than 1 hour old
        // In a real implementation, you might check the last fetch time
        // For now, we'll use file modification time
        if let Ok(metadata) = tokio::fs::metadata(_repo.path().join(".git")).await {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = modified.elapsed() {
                    return Ok(elapsed < Duration::from_secs(3600));
                }
            }
        }
        Ok(false)
    }

    pub fn parse_github_url(&self, url: &str) -> Result<ParsedGitHubUrl, CloneError> {
        // Support various GitHub URL formats
        let url = url.trim();

        // HTTPS format: https://github.com/owner/repo or https://github.com/owner/repo.git
        if let Some(captures) = GITHUB_HTTPS_REGEX.captures(url) {
            let owner = captures[1].to_string();
            let mut repo = captures[2].to_string();

            // Strip .git suffix if present (but only if it makes the name valid)
            if repo.ends_with(".git") && repo.len() > 4 {
                let without_git = &repo[..repo.len() - 4];
                // Only strip .git if the result is still a valid name
                if self.validate_github_name(without_git) {
                    repo = without_git.to_string();
                }
            }

            // Additional validation
            if self.validate_github_name(&owner) && self.validate_github_name(&repo) {
                return Ok(ParsedGitHubUrl { owner, repo });
            }
        }

        // SSH format: git@github.com:owner/repo.git
        if let Some(captures) = GITHUB_SSH_REGEX.captures(url) {
            let owner = captures[1].to_string();
            let mut repo = captures[2].to_string();

            // Strip .git suffix if present
            if repo.ends_with(".git") && repo.len() > 4 {
                let without_git = &repo[..repo.len() - 4];
                if self.validate_github_name(without_git) {
                    repo = without_git.to_string();
                }
            }

            // Additional validation
            if self.validate_github_name(&owner) && self.validate_github_name(&repo) {
                return Ok(ParsedGitHubUrl { owner, repo });
            }
        }

        // Short format: owner/repo
        if let Some(captures) = GITHUB_SHORT_REGEX.captures(url) {
            let owner = captures[1].to_string();
            let repo = captures[2].to_string();

            // Additional validation
            if self.validate_github_name(&owner) && self.validate_github_name(&repo) {
                return Ok(ParsedGitHubUrl { owner, repo });
            }
        }

        Err(CloneError::InvalidUrl(format!("Invalid GitHub URL: {url}")))
    }

    fn validate_github_name(&self, name: &str) -> bool {
        // Reject empty names
        if name.is_empty() || name.len() > 100 {
            return false;
        }

        // Reject path traversal attempts
        if name == ".." || name == "." {
            return false;
        }

        // Reject names that start or end with dots
        if name.starts_with('.') || name.ends_with('.') {
            return false;
        }

        // Reject names containing consecutive dots
        if name.contains("..") {
            return false;
        }

        // Reject names with path separators
        if name.contains('/') || name.contains('\\') {
            return false;
        }

        // Reject special Git names
        let forbidden_names = [".git", ".gitignore", ".gitmodules", ".gitattributes"];
        if forbidden_names.contains(&name) {
            return false;
        }

        // Reject URL encoded characters
        if name.contains('%') {
            return false;
        }

        // Reject control characters and non-ASCII characters
        // GitHub requires ASCII-only names
        if !name.chars().all(|c| c.is_ascii() && !c.is_control()) {
            return false;
        }

        // Ensure name matches our regex pattern (alphanumeric start/end)
        if name.len() == 1 {
            name.chars().all(|c| c.is_ascii_alphanumeric())
        } else {
            let chars: Vec<char> = name.chars().collect();
            chars.first().is_some_and(|c| c.is_ascii_alphanumeric())
                && chars.last().is_some_and(|c| c.is_ascii_alphanumeric())
        }
    }

    pub fn compute_cache_key(&self, url: &str) -> String {
        // Create a cache key from the URL
        // In production, you might want to use a hash
        url.chars()
            .map(|c| match c {
                '/' | ':' | '.' => '_',
                c if c.is_alphanumeric() || c == '-' || c == '_' => c,
                _ => '_',
            })
            .collect()
    }

    pub async fn check_repo_size(&self, _parsed_url: &ParsedGitHubUrl) -> Result<u64> {
        // This would require GitHub API access
        // For now, return a dummy value
        Ok(0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedGitHubUrl {
    pub owner: String,
    pub repo: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_parse_github_urls() {
        let temp_dir = TempDir::new().unwrap();
        let cloner = GitCloner::new(temp_dir.path().to_path_buf());

        // Create long strings outside the vec to avoid lifetime issues
        let long_owner = format!("https://github.com/{}/repo", "a".repeat(101));
        let long_repo = format!("https://github.com/owner/{}", "b".repeat(101));

        // Test various URL formats
        let test_cases = vec![
            // Valid URLs
            ("https://github.com/rust-lang/rust", true),
            ("https://github.com/rust-lang/rust.git", true),
            ("git@github.com:rust-lang/rust.git", true),
            ("rust-lang/rust", true),
            ("https://github.com/user123/repo456", true),
            ("https://github.com/a/b", true),
            // Invalid URLs - wrong domain
            ("https://gitlab.com/rust-lang/rust", false),
            ("not-a-url", false),
            // Security-sensitive patterns that should be rejected
            ("https://github.com/../repo", false),
            ("https://github.com/owner/..", false),
            ("https://github.com/.git/config", false),
            ("https://github.com/./repo", false),
            ("https://github.com/owner/.", false),
            ("https://github.com/.gitignore/repo", false),
            ("https://github.com/owner/.gitmodules", false),
            ("https://github.com/%2e%2e/repo", false),
            ("https://github.com/owner%2frepo/test", false),
            ("https://github.com//double-slash", false),
            ("https://github.com/owner//double-slash", false),
            // Names with dots
            ("https://github.com/.hidden/repo", false),
            ("https://github.com/owner/repo.", false),
            ("https://github.com/owner..name/repo", false),
            // Empty components
            ("https://github.com//repo", false),
            ("https://github.com/owner/", false),
            ("https://github.com/ /repo", false),
            // Too long
            (long_owner.as_str(), false),
            (long_repo.as_str(), false),
        ];

        for (url, should_succeed) in test_cases {
            let result = cloner.parse_github_url(url);
            assert_eq!(
                result.is_ok(),
                should_succeed,
                "URL '{}' should {} but got {:?}",
                url,
                if should_succeed { "succeed" } else { "fail" },
                result
            );
        }
    }

    #[tokio::test]
    async fn test_validate_github_name() {
        let temp_dir = TempDir::new().unwrap();
        let cloner = GitCloner::new(temp_dir.path().to_path_buf());

        // Valid names
        assert!(cloner.validate_github_name("rust"));
        assert!(cloner.validate_github_name("rust-lang"));
        assert!(cloner.validate_github_name("user_name"));
        assert!(cloner.validate_github_name("repo.name"));
        assert!(cloner.validate_github_name("123"));
        assert!(cloner.validate_github_name("a1b2c3"));

        // Invalid names
        assert!(!cloner.validate_github_name(""));
        assert!(!cloner.validate_github_name("."));
        assert!(!cloner.validate_github_name(".."));
        assert!(!cloner.validate_github_name(".hidden"));
        assert!(!cloner.validate_github_name("hidden."));
        assert!(!cloner.validate_github_name("name..name"));
        assert!(!cloner.validate_github_name(".git"));
        assert!(!cloner.validate_github_name(".gitignore"));
        assert!(!cloner.validate_github_name("name/path"));
        assert!(!cloner.validate_github_name("name\\path"));
        assert!(!cloner.validate_github_name("name%20space"));
        assert!(!cloner.validate_github_name("name\0null"));
        assert!(!cloner.validate_github_name(&"a".repeat(101)));
    }

    #[tokio::test]
    async fn test_cache_key_generation() {
        let temp_dir = TempDir::new().unwrap();
        let cloner = GitCloner::new(temp_dir.path().to_path_buf());

        let key = cloner.compute_cache_key("https://github.com/rust-lang/rust.git");
        assert!(!key.contains('/'));
        assert!(!key.contains(':'));
        assert!(key.contains("github"));
        assert!(key.contains("rust"));
    }
}
