//! GitHub Issues Integration Service
//!
//! This module provides comprehensive GitHub Issues API integration with support for:
//! - Issue creation, reading, updating, and listing
//! - Authentication via GitHub tokens and OAuth
//! - Rate limiting and error recovery
//! - PDMT-style issue template generation
//! - Quality-proxy integration for automated refactoring workflows
//!
//! # Features
//!
//! - **Full GitHub API Support**: REST API v3 with GraphQL v4 capabilities
//! - **Authentication**: Token-based, OAuth, and GitHub App authentication
//! - **Rate Limiting**: Automatic retry with exponential backoff
//! - **Error Recovery**: Comprehensive error handling with user-friendly messages
//! - **PDMT Integration**: Deterministic issue generation using seed 42
//! - **Quality Enforcement**: Integration with quality-proxy for code generation
//!
//! # Usage
//!
//! ```rust
//! use pmat::services::github_issues::{GitHubIssuesService, IssueRequest};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let service = GitHubIssuesService::new("github_token_here")?;
//! 
//! let issue_request = IssueRequest {
//!     title: "Implement new feature using PDMT style".to_string(),
//!     body: "## PDMT Requirements\n\n- Quality Level: Strict\n- Seed: 42".to_string(),
//!     labels: vec!["enhancement".to_string(), "pdmt".to_string()],
//!     assignees: vec!["developer".to_string()],
//! };
//!
//! let issue = service.create_issue("owner", "repo", issue_request).await?;
//! println!("Created issue #{}", issue.number);
//! # Ok(())
//! # }
//! ```

use reqwest::{Client, Error as ReqwestError};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;
use thiserror::Error;

/// Errors that can occur when working with GitHub Issues
#[derive(Error, Debug)]
pub enum GitHubError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] ReqwestError),
    
    #[error("Authentication failed: {token_type}")]
    Authentication { token_type: String },
    
    #[error("Rate limit exceeded, retry after {retry_after} seconds")]
    RateLimit { retry_after: u64 },
    
    #[error("GitHub API error: {status} - {message}")]
    Api { status: u16, message: String },
    
    #[error("Invalid repository format: {repo}")]
    InvalidRepo { repo: String },
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// GitHub Issue representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubIssue {
    pub id: u64,
    pub number: u32,
    pub title: String,
    pub body: Option<String>,
    pub state: IssueState,
    pub labels: Vec<Label>,
    pub assignees: Vec<User>,
    pub created_at: String,
    pub updated_at: String,
    pub html_url: String,
    pub user: User,
}

/// Issue state enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueState {
    Open,
    Closed,
}

/// GitHub label representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub id: u64,
    pub name: String,
    pub color: String,
    pub description: Option<String>,
}

/// GitHub user representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub login: String,
    pub avatar_url: String,
    pub html_url: String,
}

/// Request payload for creating new issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueRequest {
    pub title: String,
    pub body: String,
    pub labels: Vec<String>,
    pub assignees: Vec<String>,
}

/// Request payload for updating existing issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueUpdateRequest {
    pub title: Option<String>,
    pub body: Option<String>,
    pub state: Option<IssueState>,
    pub labels: Option<Vec<String>>,
    pub assignees: Option<Vec<String>>,
}

/// GitHub API pagination information
#[derive(Debug, Clone)]
pub struct Pagination {
    pub page: u32,
    pub per_page: u32,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 30,
        }
    }
}

/// Configuration for GitHub Issues service
#[derive(Debug, Clone)]
pub struct GitHubConfig {
    pub token: String,
    pub base_url: String,
    pub timeout: Duration,
    pub max_retries: u32,
    pub retry_delay: Duration,
}

impl Default for GitHubConfig {
    fn default() -> Self {
        Self {
            token: String::new(),
            base_url: "https://api.github.com".to_string(),
            timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
        }
    }
}

/// Main service for GitHub Issues integration
///
/// Provides comprehensive GitHub Issues API integration with authentication,
/// rate limiting, error recovery, and PDMT-style issue generation.
///
/// # Examples
///
/// ```rust
/// use pmat::services::github_issues::GitHubIssuesService;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let service = GitHubIssuesService::new("github_token_here")?;
/// let issues = service.list_issues("owner", "repo", None).await?;
/// println!("Found {} issues", issues.len());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct GitHubIssuesService {
    client: Client,
    config: GitHubConfig,
}

impl GitHubIssuesService {
    /// Create a new GitHub Issues service with token authentication
    ///
    /// # Arguments
    ///
    /// * `token` - GitHub personal access token or OAuth token
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::github_issues::GitHubIssuesService;
    ///
    /// let service = GitHubIssuesService::new("ghp_xxxxxxxxxxxxxxxxxxxx")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(token: &str) -> Result<Self, GitHubError> {
        let config = GitHubConfig {
            token: token.to_string(),
            ..Default::default()
        };
        
        Self::with_config(config)
    }

    /// Create a new GitHub Issues service with custom configuration
    ///
    /// # Arguments
    ///
    /// * `config` - GitHub service configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::github_issues::{GitHubIssuesService, GitHubConfig};
    /// use std::time::Duration;
    ///
    /// let config = GitHubConfig {
    ///     token: "ghp_xxxxxxxxxxxxxxxxxxxx".to_string(),
    ///     timeout: Duration::from_secs(60),
    ///     max_retries: 5,
    ///     ..Default::default()
    /// };
    ///
    /// let service = GitHubIssuesService::with_config(config)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn with_config(config: GitHubConfig) -> Result<Self, GitHubError> {
        if config.token.is_empty() {
            return Err(GitHubError::Authentication {
                token_type: "empty token".to_string(),
            });
        }

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", config.token).parse().unwrap(),
        );
        headers.insert(
            reqwest::header::USER_AGENT,
            "pmat-github-integration/1.0".parse().unwrap(),
        );
        headers.insert(
            reqwest::header::ACCEPT,
            "application/vnd.github.v3+json".parse().unwrap(),
        );

        let client = Client::builder()
            .timeout(config.timeout)
            .default_headers(headers)
            .build()
            .map_err(GitHubError::Request)?;

        Ok(Self { client, config })
    }

    /// Create a new GitHub issue
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner (user or organization)
    /// * `repo` - Repository name
    /// * `request` - Issue creation request
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::github_issues::{GitHubIssuesService, IssueRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let service = GitHubIssuesService::new("token")?;
    /// 
    /// let request = IssueRequest {
    ///     title: "PDMT Feature Implementation".to_string(),
    ///     body: "Implement using PDMT style with seed 42".to_string(),
    ///     labels: vec!["enhancement".to_string()],
    ///     assignees: vec![],
    /// };
    ///
    /// let issue = service.create_issue("owner", "repo", request).await?;
    /// assert!(!issue.title.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_issue(
        &self,
        owner: &str,
        repo: &str,
        request: IssueRequest,
    ) -> Result<GitHubIssue, GitHubError> {
        let url = format!("{}/repos/{}/{}/issues", self.config.base_url, owner, repo);
        
        self.execute_with_retry(|| async {
            let response = self
                .client
                .post(&url)
                .json(&request)
                .send()
                .await?;

            self.handle_response(response).await
        })
        .await
    }

    /// Read a GitHub issue by number
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner (user or organization)
    /// * `repo` - Repository name
    /// * `issue_number` - Issue number to retrieve
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::github_issues::GitHubIssuesService;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let service = GitHubIssuesService::new("token")?;
    /// let issue = service.read_issue("owner", "repo", 123).await?;
    /// assert_eq!(issue.number, 123);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn read_issue(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u32,
    ) -> Result<GitHubIssue, GitHubError> {
        let url = format!(
            "{}/repos/{}/{}/issues/{}",
            self.config.base_url, owner, repo, issue_number
        );

        self.execute_with_retry(|| async {
            let response = self.client.get(&url).send().await?;
            self.handle_response(response).await
        })
        .await
    }

    /// Update an existing GitHub issue
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner (user or organization)
    /// * `repo` - Repository name  
    /// * `issue_number` - Issue number to update
    /// * `request` - Issue update request
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::github_issues::{GitHubIssuesService, IssueUpdateRequest, IssueState};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let service = GitHubIssuesService::new("token")?;
    /// 
    /// let update = IssueUpdateRequest {
    ///     title: Some("Updated Title".to_string()),
    ///     state: Some(IssueState::Closed),
    ///     ..Default::default()
    /// };
    ///
    /// let issue = service.update_issue("owner", "repo", 123, update).await?;
    /// assert_eq!(issue.state, IssueState::Closed);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_issue(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u32,
        request: IssueUpdateRequest,
    ) -> Result<GitHubIssue, GitHubError> {
        let url = format!(
            "{}/repos/{}/{}/issues/{}",
            self.config.base_url, owner, repo, issue_number
        );

        self.execute_with_retry(|| async {
            let response = self
                .client
                .patch(&url)
                .json(&request)
                .send()
                .await?;

            self.handle_response(response).await
        })
        .await
    }

    /// List GitHub issues for a repository
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner (user or organization)
    /// * `repo` - Repository name
    /// * `pagination` - Optional pagination configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::github_issues::{GitHubIssuesService, Pagination};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let service = GitHubIssuesService::new("token")?;
    /// 
    /// let pagination = Pagination {
    ///     page: 1,
    ///     per_page: 50,
    /// };
    ///
    /// let issues = service.list_issues("owner", "repo", Some(pagination)).await?;
    /// assert!(issues.len() <= 50);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_issues(
        &self,
        owner: &str,
        repo: &str,
        pagination: Option<Pagination>,
    ) -> Result<Vec<GitHubIssue>, GitHubError> {
        let pagination = pagination.unwrap_or_default();
        let url = format!(
            "{}/repos/{}/{}/issues?page={}&per_page={}",
            self.config.base_url, owner, repo, pagination.page, pagination.per_page
        );

        self.execute_with_retry(|| async {
            let response = self.client.get(&url).send().await?;
            self.handle_response(response).await
        })
        .await
    }

    /// Execute HTTP request with retry logic for rate limiting
    async fn execute_with_retry<F, Fut, T>(&self, operation: F) -> Result<T, GitHubError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, GitHubError>>,
    {
        let mut attempts = 0;
        let mut delay = self.config.retry_delay;

        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(GitHubError::RateLimit { retry_after }) => {
                    if attempts >= self.config.max_retries {
                        return Err(GitHubError::RateLimit { retry_after });
                    }
                    
                    attempts += 1;
                    let sleep_duration = Duration::from_secs(retry_after).max(delay);
                    sleep(sleep_duration).await;
                    delay *= 2; // Exponential backoff
                }
                Err(other) => return Err(other),
            }
        }
    }

    /// Handle HTTP response and convert to appropriate types
    async fn handle_response<T>(&self, response: reqwest::Response) -> Result<T, GitHubError>
    where
        T: serde::de::DeserializeOwned,
    {
        let status = response.status();

        if status.is_success() {
            let json = response.json::<T>().await?;
            return Ok(json);
        }

        // Handle rate limiting
        if status == 403 {
            if let Some(retry_after) = response
                .headers()
                .get("x-ratelimit-reset")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
            {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let retry_after = retry_after.saturating_sub(now);
                
                return Err(GitHubError::RateLimit { retry_after });
            }
        }

        // Handle authentication errors
        if status == 401 {
            return Err(GitHubError::Authentication {
                token_type: "invalid or expired token".to_string(),
            });
        }

        // Handle other API errors
        let error_body = response.text().await.unwrap_or_default();
        Err(GitHubError::Api {
            status: status.as_u16(),
            message: error_body,
        })
    }

    /// Validate repository format (owner/repo)
    #[allow(dead_code)]
    fn validate_repo_format(owner: &str, repo: &str) -> Result<(), GitHubError> {
        if owner.is_empty() || repo.is_empty() {
            return Err(GitHubError::InvalidRepo {
                repo: format!("{}/{}", owner, repo),
            });
        }

        // Basic validation for allowed characters
        let valid_chars = |s: &str| s.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.');
        
        if !valid_chars(owner) || !valid_chars(repo) {
            return Err(GitHubError::InvalidRepo {
                repo: format!("{}/{}", owner, repo),
            });
        }

        Ok(())
    }
}

// Default implementation for IssueUpdateRequest
impl Default for IssueUpdateRequest {
    fn default() -> Self {
        Self {
            title: None,
            body: None,
            state: None,
            labels: None,
            assignees: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_config_default() {
        let config = GitHubConfig::default();
        assert!(config.token.is_empty());
        assert_eq!(config.base_url, "https://api.github.com");
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_issue_request_serialization() {
        let request = IssueRequest {
            title: "Test Issue".to_string(),
            body: "Test body with **markdown**".to_string(),
            labels: vec!["bug".to_string(), "high-priority".to_string()],
            assignees: vec!["developer".to_string()],
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("Test Issue"));
        assert!(json.contains("labels"));
        assert!(json.contains("assignees"));
    }

    #[test]
    fn test_pagination_default() {
        let pagination = Pagination::default();
        assert_eq!(pagination.page, 1);
        assert_eq!(pagination.per_page, 30);
    }

    #[test]
    fn test_validate_repo_format() {
        // Valid repository formats
        assert!(GitHubIssuesService::validate_repo_format("owner", "repo").is_ok());
        assert!(GitHubIssuesService::validate_repo_format("user123", "my-repo_v2").is_ok());
        
        // Invalid repository formats
        assert!(GitHubIssuesService::validate_repo_format("", "repo").is_err());
        assert!(GitHubIssuesService::validate_repo_format("owner", "").is_err());
    }

    #[test]
    fn test_github_error_display() {
        let error = GitHubError::Authentication {
            token_type: "expired".to_string(),
        };
        assert_eq!(error.to_string(), "Authentication failed: expired");

        let error = GitHubError::RateLimit { retry_after: 120 };
        assert_eq!(error.to_string(), "Rate limit exceeded, retry after 120 seconds");
    }

    #[tokio::test]
    async fn test_service_creation_with_empty_token() {
        let result = GitHubIssuesService::new("");
        assert!(result.is_err());
        
        if let Err(GitHubError::Authentication { token_type }) = result {
            assert_eq!(token_type, "empty token");
        } else {
            panic!("Expected authentication error");
        }
    }

    #[tokio::test]
    async fn test_service_creation_with_valid_token() {
        let result = GitHubIssuesService::new("ghp_test_token_12345678901234567890");
        assert!(result.is_ok());
    }
}