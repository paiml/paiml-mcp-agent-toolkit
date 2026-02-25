#![cfg_attr(coverage_nightly, coverage(off))]
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
//! See [`GitHubIssuesService`] for usage examples.

use reqwest::{Client, Error as ReqwestError};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use tokio::time::sleep;

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
/// ```rust,no_run
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

// Client implementation (constructors, CRUD, retry logic, response handling)
include!("github_issues_client.rs");

// Unit tests and property tests
include!("github_issues_tests.rs");
