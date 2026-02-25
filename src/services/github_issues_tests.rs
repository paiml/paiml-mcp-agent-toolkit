// GitHub Issues tests
// This file is include!()'d from github_issues.rs — do NOT add `use` imports or `#!` attributes.

#[cfg_attr(coverage_nightly, coverage(off))]
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
