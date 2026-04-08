// GitHub Issues client implementation methods
// This file is include!()'d from github_issues.rs — do NOT add `use` imports or `#!` attributes.

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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_config(config: GitHubConfig) -> Result<Self, GitHubError> {
        if config.token.is_empty() {
            return Err(GitHubError::Authentication {
                token_type: "empty token".to_string(),
            });
        }

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", config.token).parse().expect("valid authorization header"),
        );
        headers.insert(
            reqwest::header::USER_AGENT,
            "pmat-github-integration/1.0".parse().expect("valid user-agent header"),
        );
        headers.insert(
            reqwest::header::ACCEPT,
            "application/vnd.github.v3+json".parse().expect("valid accept header"),
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
                    .expect("system time after UNIX epoch")
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
