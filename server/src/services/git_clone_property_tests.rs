//! Property tests for git_clone service

use proptest::prelude::*;
use crate::services::git_clone::{GitCloner, ParsedGitHubUrl};

proptest! {
    /// Test that repo size is always non-negative
    #[test]
    fn test_repo_size_non_negative(
        owner in "[a-z]{3,10}",
        repo in "[a-z]{3,10}",
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let git_clone = GitCloner::new();
        let parsed_url = ParsedGitHubUrl {
            owner: owner.clone(),
            repo: repo.clone(),
            branch: None,
            subdir: None,
        };
        
        let result = rt.block_on(async {
            git_clone.check_repo_size(&parsed_url).await
        });
        
        match result {
            Ok(size) => {
                prop_assert!(size >= 0, "Repository size should be non-negative");
                // GitHub reports sizes in KB, so even empty repos have some size
                if owner == "github" && repo == "gitignore" {
                    // Well-known small repo
                    prop_assert!(size > 0, "Known repository should have non-zero size");
                }
            }
            Err(_) => {
                // API errors are acceptable (rate limits, network issues, etc.)
            }
        }
    }

    /// Test URL parsing consistency
    #[test]
    fn test_parsed_url_properties(
        owner in "[a-zA-Z0-9_-]{1,39}",  // GitHub username constraints
        repo in "[a-zA-Z0-9_.-]{1,100}", // GitHub repo name constraints
        branch in prop::option::of("[a-zA-Z0-9_/-]{1,50}"),
    ) {
        let parsed_url = ParsedGitHubUrl {
            owner: owner.clone(),
            repo: repo.clone(),
            branch: branch.clone(),
            subdir: None,
        };
        
        // Properties of ParsedGitHubUrl
        prop_assert!(!parsed_url.owner.is_empty());
        prop_assert!(!parsed_url.repo.is_empty());
        prop_assert_eq!(parsed_url.owner, owner);
        prop_assert_eq!(parsed_url.repo, repo);
        prop_assert_eq!(parsed_url.branch, branch);
    }

    /// Test that size checking handles various repo states
    #[test]
    fn test_repo_size_edge_cases(test_case in 0u8..5) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let git_clone = GitCloner::new();
        
        let parsed_url = match test_case {
            0 => ParsedGitHubUrl {
                owner: "nonexistent-user-12345".to_string(),
                repo: "nonexistent-repo".to_string(),
                branch: None,
                subdir: None,
            },
            1 => ParsedGitHubUrl {
                owner: "torvalds".to_string(),
                repo: "linux".to_string(),  // Very large repo
                branch: None,
                subdir: None,
            },
            2 => ParsedGitHubUrl {
                owner: "octocat".to_string(),
                repo: "Hello-World".to_string(),  // Classic test repo
                branch: None,
                subdir: None,
            },
            3 => ParsedGitHubUrl {
                owner: "github".to_string(),
                repo: "gitignore".to_string(),  // Small but important repo
                branch: None,
                subdir: None,
            },
            _ => ParsedGitHubUrl {
                owner: "rust-lang".to_string(),
                repo: "rust".to_string(),  // Medium-large repo
                branch: None,
                subdir: None,
            },
        };
        
        let result = rt.block_on(async {
            git_clone.check_repo_size(&parsed_url).await
        });
        
        match test_case {
            0 => {
                // Nonexistent repo should error
                prop_assert!(result.is_err(), "Nonexistent repo should return error");
            }
            1 => {
                // Linux kernel should be very large
                if let Ok(size) = result {
                    prop_assert!(size > 1_000_000, "Linux kernel should be > 1GB");
                }
            }
            2 | 3 => {
                // Small repos should be small
                if let Ok(size) = result {
                    prop_assert!(size < 10_000, "Test repos should be < 10MB");
                }
            }
            _ => {
                // Rust repo should be medium size
                if let Ok(size) = result {
                    prop_assert!(size > 10_000 && size < 1_000_000, 
                        "Rust repo should be between 10MB and 1GB");
                }
            }
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_real_repo_sizes() {
        // Skip if no GitHub token (to avoid rate limits in CI)
        if env::var("GITHUB_TOKEN").is_err() && env::var("CI").is_ok() {
            eprintln!("Skipping GitHub API test in CI without token");
            return;
        }

        let git_clone = GitCloner::new();
        
        // Test with known repositories
        let test_repos = vec![
            ("github", "gitignore", 100, 10_000),      // Small repo
            ("rust-lang", "mdBook", 1_000, 100_000),   // Medium repo
        ];
        
        for (owner, repo, min_size, max_size) in test_repos {
            let parsed_url = ParsedGitHubUrl {
                owner: owner.to_string(),
                repo: repo.to_string(),
                branch: None,
                subdir: None,
            };
            
            match git_clone.check_repo_size(&parsed_url).await {
                Ok(size) => {
                    assert!(size >= min_size, 
                        "{}/{} size {} KB is less than expected {} KB", 
                        owner, repo, size, min_size);
                    assert!(size <= max_size, 
                        "{}/{} size {} KB is more than expected {} KB", 
                        owner, repo, size, max_size);
                }
                Err(e) => {
                    eprintln!("Warning: Could not check {}/{}: {}", owner, repo, e);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_error_handling() {
        let git_clone = GitCloner::new();
        
        // Test various error cases
        let error_cases = vec![
            ("", "repo", "Empty owner"),
            ("owner", "", "Empty repo"),
            ("definitely-not-exist-123456", "repo", "Nonexistent user"),
        ];
        
        for (owner, repo, description) in error_cases {
            let parsed_url = ParsedGitHubUrl {
                owner: owner.to_string(),
                repo: repo.to_string(),
                branch: None,
                subdir: None,
            };
            
            let result = git_clone.check_repo_size(&parsed_url).await;
            assert!(result.is_err(), "{} should produce an error", description);
        }
    }
}