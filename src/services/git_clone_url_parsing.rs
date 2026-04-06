// git_clone_url_parsing.rs — URL parsing, validation, cache keys, and repo size for GitCloner
// Included from git_clone.rs — do NOT add `use` imports or `#!` inner attributes here.

impl GitCloner {
    #[inline]
    pub fn parse_github_url(&self, url: &str) -> Result<ParsedGitHubUrl, CloneError> {
        debug_assert!(!url.is_empty(), "url must not be empty");
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
        debug_assert!(!name.is_empty(), "name must not be empty");
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
            chars.first().is_some_and(char::is_ascii_alphanumeric)
                && chars.last().is_some_and(char::is_ascii_alphanumeric)
        }
    }

    #[must_use]
    pub fn compute_cache_key(&self, url: &str) -> String {
        debug_assert!(!url.is_empty(), "url must not be empty");
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

    /// Check the size of a GitHub repository using the GitHub API
    ///
    /// This function queries the GitHub API to get repository metadata
    /// and returns the size in kilobytes.
    ///
    /// # Arguments
    /// * `parsed_url` - A parsed GitHub URL containing owner and repo information
    ///
    /// # Returns
    /// The repository size in kilobytes
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use pmat::services::git_clone::{GitCloner, ParsedGitHubUrl};
    /// # use std::path::PathBuf;
    /// #
    /// # #[tokio::test]
    /// # async fn test_repo_size() -> anyhow::Result<()> {
    /// let git_clone = GitCloner::new(PathBuf::from(".cache"));
    /// let parsed_url = ParsedGitHubUrl {
    ///     owner: "rust-lang".to_string(),
    ///     repo: "rust".to_string(),
    /// };
    ///
    /// let size_kb = git_clone.check_repo_size(&parsed_url).await?;
    /// assert!(size_kb > 0, "Repository should have non-zero size");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Property Tests
    ///
    /// ```no_run
    /// # use pmat::services::git_clone::{GitCloner, ParsedGitHubUrl};
    /// # use std::path::PathBuf;
    /// #
    /// # #[tokio::test]
    /// # async fn test_repo_size_properties() -> anyhow::Result<()> {
    /// let git_clone = GitCloner::new(PathBuf::from(".cache"));
    ///
    /// // Test with well-known repositories
    /// let repos = vec![
    ///     ("rust-lang", "rust"),
    ///     ("torvalds", "linux"),
    /// ];
    ///
    /// for (owner, repo) in repos {
    ///     let parsed_url = ParsedGitHubUrl {
    ///         owner: owner.to_string(),
    ///         repo: repo.to_string(),
    ///     };
    ///
    ///     let size = git_clone.check_repo_size(&parsed_url).await?;
    ///
    ///     // Properties: Size should be positive and reasonable
    ///     assert!(size > 0, "Size should be positive");
    ///     assert!(size < 10_000_000, "Size should be reasonable (< 10GB)");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn check_repo_size(&self, parsed_url: &ParsedGitHubUrl) -> Result<u64> {
        use anyhow::anyhow;
        use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};

        // Build GitHub API URL
        let api_url = format!(
            "https://api.github.com/repos/{}/{}",
            parsed_url.owner, parsed_url.repo
        );

        // Create HTTP client with headers
        let client = reqwest::Client::new();
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("pmat-cli"));
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github.v3+json"),
        );

        // Add auth token if available
        if let Ok(token) = std::env::var("GITHUB_TOKEN") {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("token {token}"))?,
            );
        }

        // Make API request
        let response = client.get(&api_url).headers(headers).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "GitHub API request failed with status: {}",
                response.status()
            ));
        }

        // Parse response
        #[derive(serde::Deserialize)]
        struct RepoInfo {
            size: u64, // Size in KB from GitHub API
        }

        let repo_info: RepoInfo = response.json().await?;

        // Return size in KB as received from GitHub API
        Ok(repo_info.size)
    }
}
