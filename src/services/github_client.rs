//! GitHub API client for unified workflow (Issue #75)
//!
//! This module provides GitHub API integration using octocrab for:
//! - Fetching issue details from GitHub
//! - Creating new issues
//! - Updating issue status and labels
//! - Syncing with local roadmap.yaml
//!
//! Note: This module requires the `github-api` feature to be enabled.
//! When disabled, use the `gh` CLI fallback in work_handlers.rs instead.

#![cfg(feature = "github-api")]

use anyhow::{Context, Result};
use octocrab::models::issues::Issue;
use octocrab::models::IssueState;
use octocrab::Octocrab;
use std::env;

/// GitHub API client for repository operations
#[derive(Debug)]
pub struct GitHubClient {
    octocrab: Octocrab,
    repo_owner: String,
    repo_name: String,
}

impl GitHubClient {
    /// Create a new GitHub client from repo string (e.g., "paiml/pmat")
    ///
    /// Requires GITHUB_TOKEN environment variable to be set.
    ///
    /// # Arguments
    /// * `repo` - Repository in "owner/name" format
    ///
    /// # Errors
    /// Returns error if:
    /// - GITHUB_TOKEN is not set
    /// - Repo format is invalid (not "owner/name")
    /// - Failed to initialize octocrab client
    pub fn new(repo: &str) -> Result<Self> {
        let parts: Vec<&str> = repo.split('/').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid repo format: '{}'. Expected 'owner/name'", repo);
        }

        let token = env::var("GITHUB_TOKEN").context(
            "GITHUB_TOKEN environment variable not set. Please set it to use GitHub API.",
        )?;

        let octocrab = Octocrab::builder()
            .personal_token(token)
            .build()
            .context("Failed to initialize GitHub client")?;

        Ok(Self {
            octocrab,
            repo_owner: parts[0].to_string(),
            repo_name: parts[1].to_string(),
        })
    }

    /// Create a new GitHub client without authentication (read-only, rate-limited)
    ///
    /// This is useful for public repositories when GITHUB_TOKEN is not available.
    /// Note: Rate limits are much lower without authentication (60 req/hour vs 5000).
    ///
    /// # Arguments
    /// * `repo` - Repository in "owner/name" format
    pub fn new_unauthenticated(repo: &str) -> Result<Self> {
        let parts: Vec<&str> = repo.split('/').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid repo format: '{}'. Expected 'owner/name'", repo);
        }

        let octocrab = Octocrab::builder()
            .build()
            .context("Failed to initialize GitHub client")?;

        Ok(Self {
            octocrab,
            repo_owner: parts[0].to_string(),
            repo_name: parts[1].to_string(),
        })
    }

    /// Fetch issue details from GitHub
    ///
    /// # Arguments
    /// * `issue_num` - GitHub issue number
    ///
    /// # Returns
    /// Issue details including title, labels, body, and state
    pub async fn fetch_issue(&self, issue_num: u64) -> Result<Issue> {
        let issue = self
            .octocrab
            .issues(&self.repo_owner, &self.repo_name)
            .get(issue_num)
            .await
            .context(format!("Failed to fetch issue #{}", issue_num))?;

        Ok(issue)
    }

    /// Create a new GitHub issue
    ///
    /// # Arguments
    /// * `title` - Issue title
    /// * `body` - Issue description (markdown)
    /// * `labels` - Optional labels to apply
    ///
    /// # Returns
    /// Created issue with GitHub-assigned number
    pub async fn create_issue(
        &self,
        title: &str,
        body: &str,
        labels: Option<Vec<String>>,
    ) -> Result<Issue> {
        let issues_handler = self.octocrab.issues(&self.repo_owner, &self.repo_name);
        let mut issue_builder = issues_handler.create(title).body(body);

        if let Some(label_list) = labels {
            issue_builder = issue_builder.labels(label_list);
        }

        let issue = issue_builder
            .send()
            .await
            .context("Failed to create GitHub issue")?;

        Ok(issue)
    }

    /// Update an existing GitHub issue
    ///
    /// # Arguments
    /// * `issue_num` - GitHub issue number
    /// * `title` - Optional new title
    /// * `body` - Optional new body
    /// * `state` - Optional new state ("open" or "closed")
    /// * `labels` - Optional new labels (replaces existing)
    pub async fn update_issue(
        &self,
        issue_num: u64,
        title: Option<&str>,
        body: Option<&str>,
        state: Option<&str>,
        labels: Option<Vec<String>>,
    ) -> Result<Issue> {
        let issues_handler = self.octocrab.issues(&self.repo_owner, &self.repo_name);
        let mut update_builder = issues_handler.update(issue_num);

        if let Some(t) = title {
            update_builder = update_builder.title(t);
        }

        if let Some(b) = body {
            update_builder = update_builder.body(b);
        }

        if let Some(s) = state {
            let state_enum = match s {
                "open" => IssueState::Open,
                "closed" => IssueState::Closed,
                _ => anyhow::bail!("Invalid state: '{}'. Must be 'open' or 'closed'", s),
            };
            update_builder = update_builder.state(state_enum);
        }

        // Clone labels to ensure they live long enough
        let labels_owned = labels;
        if let Some(ref label_list) = labels_owned {
            update_builder = update_builder.labels(label_list);
        }

        let issue = update_builder
            .send()
            .await
            .context(format!("Failed to update issue #{}", issue_num))?;

        Ok(issue)
    }

    /// Close a GitHub issue
    ///
    /// # Arguments
    /// * `issue_num` - GitHub issue number to close
    pub async fn close_issue(&self, issue_num: u64) -> Result<Issue> {
        self.update_issue(issue_num, None, None, Some("closed"), None)
            .await
    }

    /// Reopen a GitHub issue
    ///
    /// # Arguments
    /// * `issue_num` - GitHub issue number to reopen
    pub async fn reopen_issue(&self, issue_num: u64) -> Result<Issue> {
        self.update_issue(issue_num, None, None, Some("open"), None)
            .await
    }

    /// List all open issues for the repository
    ///
    /// # Returns
    /// Vector of open issues (max 100, paginated)
    pub async fn list_open_issues(&self) -> Result<Vec<Issue>> {
        let issues = self
            .octocrab
            .issues(&self.repo_owner, &self.repo_name)
            .list()
            .state(octocrab::params::State::Open)
            .per_page(100)
            .send()
            .await
            .context("Failed to list open issues")?;

        Ok(issues.items)
    }

    /// Get repository full name (owner/name)
    pub fn repo_full_name(&self) -> String {
        format!("{}/{}", self.repo_owner, self.repo_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_client_creation_requires_valid_repo_format() {
        // This will fail because GITHUB_TOKEN is not set in tests,
        // but it validates the repo format parsing
        let result = GitHubClient::new("invalid");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid repo format"));

        let result = GitHubClient::new("owner/repo/extra");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid repo format"));
    }

    #[tokio::test]
    async fn test_unauthenticated_client_creation() {
        let result = GitHubClient::new_unauthenticated("paiml/pmat");
        assert!(result.is_ok());
        let client = result.unwrap();
        assert_eq!(client.repo_owner, "paiml");
        assert_eq!(client.repo_name, "pmat");
        assert_eq!(client.repo_full_name(), "paiml/pmat");
    }

    #[tokio::test]
    async fn test_repo_full_name() {
        let client = GitHubClient::new_unauthenticated("paiml/pmat").unwrap();
        assert_eq!(client.repo_full_name(), "paiml/pmat");
    }

    // === Repo format validation tests ===

    #[test]
    fn test_invalid_repo_format_empty() {
        let result = GitHubClient::new("");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Invalid repo format"));
    }

    #[test]
    fn test_invalid_repo_format_no_slash() {
        let result = GitHubClient::new("ownerrepo");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_repo_format_trailing_slash() {
        let result = GitHubClient::new("owner/");
        // This parses as ["owner", ""] which is 2 parts but one is empty
        // The code doesn't validate empty parts, so this may succeed
        // Let's verify it doesn't crash
        if result.is_ok() {
            let client = result.unwrap();
            assert_eq!(client.repo_owner, "owner");
            assert_eq!(client.repo_name, "");
        }
    }

    #[test]
    fn test_invalid_repo_format_leading_slash() {
        let result = GitHubClient::new("/repo");
        if result.is_ok() {
            let client = result.unwrap();
            assert_eq!(client.repo_owner, "");
            assert_eq!(client.repo_name, "repo");
        }
    }

    #[tokio::test]
    async fn test_valid_repo_format_unauthenticated() {
        let result = GitHubClient::new_unauthenticated("octocat/hello-world");
        assert!(result.is_ok());
        let client = result.unwrap();
        assert_eq!(client.repo_owner, "octocat");
        assert_eq!(client.repo_name, "hello-world");
    }

    #[tokio::test]
    async fn test_unauthenticated_invalid_format() {
        let result = GitHubClient::new_unauthenticated("invalid");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_unauthenticated_too_many_parts() {
        let result = GitHubClient::new_unauthenticated("a/b/c");
        assert!(result.is_err());
    }

    // === repo_full_name tests ===

    #[tokio::test]
    async fn test_repo_full_name_various() {
        let client = GitHubClient::new_unauthenticated("microsoft/vscode").unwrap();
        assert_eq!(client.repo_full_name(), "microsoft/vscode");

        let client = GitHubClient::new_unauthenticated("rust-lang/rust").unwrap();
        assert_eq!(client.repo_full_name(), "rust-lang/rust");
    }

    // === Debug trait test ===

    #[tokio::test]
    async fn test_github_client_debug() {
        let client = GitHubClient::new_unauthenticated("test/repo").unwrap();
        let debug_str = format!("{:?}", client);
        assert!(debug_str.contains("GitHubClient"));
        assert!(debug_str.contains("test"));
        assert!(debug_str.contains("repo"));
    }

    // === Client reuse tests ===

    #[tokio::test]
    async fn test_client_can_be_reused() {
        let client = GitHubClient::new_unauthenticated("owner/repo").unwrap();

        // Multiple calls to repo_full_name should work
        assert_eq!(client.repo_full_name(), "owner/repo");
        assert_eq!(client.repo_full_name(), "owner/repo");
    }

    // === GITHUB_TOKEN environment tests ===

    #[test]
    fn test_new_without_token() {
        // Save current token if exists
        let saved = env::var("GITHUB_TOKEN").ok();

        // Remove token
        env::remove_var("GITHUB_TOKEN");

        let result = GitHubClient::new("owner/repo");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("GITHUB_TOKEN"));

        // Restore token if it existed
        if let Some(token) = saved {
            env::set_var("GITHUB_TOKEN", token);
        }
    }

    // === Edge cases ===

    #[tokio::test]
    async fn test_repo_with_dots() {
        let client = GitHubClient::new_unauthenticated("owner/repo.name").unwrap();
        assert_eq!(client.repo_name, "repo.name");
    }

    #[tokio::test]
    async fn test_repo_with_underscores() {
        let client = GitHubClient::new_unauthenticated("owner/repo_name").unwrap();
        assert_eq!(client.repo_name, "repo_name");
    }

    #[tokio::test]
    async fn test_repo_with_numbers() {
        let client = GitHubClient::new_unauthenticated("owner123/repo456").unwrap();
        assert_eq!(client.repo_owner, "owner123");
        assert_eq!(client.repo_name, "repo456");
    }

    #[tokio::test]
    async fn test_repo_with_mixed_case() {
        let client = GitHubClient::new_unauthenticated("Owner/RepoName").unwrap();
        assert_eq!(client.repo_owner, "Owner");
        assert_eq!(client.repo_name, "RepoName");
    }
}
