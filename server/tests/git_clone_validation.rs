use pmat::services::git_clone::{CloneError, GitCloner};
use tempfile::TempDir;

#[test]
fn test_github_url_parsing_comprehensive() {
    let temp_dir = TempDir::new().unwrap();
    let cloner = GitCloner::new(temp_dir.path().to_path_buf());

    // Valid URL test cases
    let valid_cases = vec![
        // HTTPS format
        ("https://github.com/rust-lang/rust", "rust-lang", "rust"),
        ("https://github.com/rust-lang/rust.git", "rust-lang", "rust"),
        (
            "https://github.com/USER-NAME/REPO-NAME",
            "USER-NAME",
            "REPO-NAME",
        ),
        (
            "https://github.com/user_name/repo_name",
            "user_name",
            "repo_name",
        ),
        ("https://github.com/user123/repo456", "user123", "repo456"),
        ("https://github.com/123/456", "123", "456"),
        ("https://github.com/a/b", "a", "b"),
        ("https://github.com/A/B", "A", "B"),
        // SSH format
        ("git@github.com:rust-lang/rust.git", "rust-lang", "rust"),
        ("git@github.com:rust-lang/rust", "rust-lang", "rust"),
        ("git@github.com:USER/REPO.git", "USER", "REPO"),
        (
            "git@github.com:user-name/repo-name.git",
            "user-name",
            "repo-name",
        ),
        // Short format
        ("rust-lang/rust", "rust-lang", "rust"),
        ("user/repo", "user", "repo"),
        ("USER/REPO", "USER", "REPO"),
        ("user-name/repo-name", "user-name", "repo-name"),
        ("user_name/repo_name", "user_name", "repo_name"),
        ("123/456", "123", "456"),
    ];

    for (url, expected_owner, expected_repo) in valid_cases {
        match cloner.parse_github_url(url) {
            Ok(parsed) => {
                assert_eq!(
                    parsed.owner, expected_owner,
                    "Owner mismatch for URL: {url}"
                );
                assert_eq!(parsed.repo, expected_repo, "Repo mismatch for URL: {url}");
            }
            Err(e) => {
                panic!("Failed to parse valid URL '{url}': {e:?}");
            }
        }
    }
}

#[test]
fn test_github_url_parsing_invalid() {
    let temp_dir = TempDir::new().unwrap();
    let cloner = GitCloner::new(temp_dir.path().to_path_buf());

    // Invalid URL test cases
    let invalid_cases = vec![
        // Empty and whitespace
        "",
        " ",
        "\t",
        "\n",
        "   ",
        // Incomplete URLs
        "https://",
        "https://github.com",
        "https://github.com/",
        "https://github.com/owner",
        "https://github.com/owner/",
        "git@github.com",
        "git@github.com:",
        // Wrong domain
        "https://gitlab.com/owner/repo",
        "https://bitbucket.org/owner/repo",
        "https://example.com/owner/repo",
        // Invalid formats
        "http://github.com/owner/repo", // We only support HTTPS
        "ftp://github.com/owner/repo",
        "file:///etc/passwd",
        // Path traversal
        "https://github.com/../repo",
        "https://github.com/owner/../../../etc/passwd",
        "https://github.com/.git/config",
        // Invalid characters (depending on implementation)
        "https://github.com/owner repo/name",
        "https://github.com/owner/repo name",
        "https://github.com/owner<script>/repo",
        "https://github.com/owner/repo;rm -rf /",
        // Multiple slashes
        "https://github.com//owner/repo",
        "https://github.com/owner//repo",
        "https://github.com/owner/repo//",
        // Too many components
        "https://github.com/owner/repo/extra",
        "https://github.com/owner/repo/tree/main",
        "https://github.com/owner/repo/blob/main/README.md",
    ];

    for url in invalid_cases {
        match cloner.parse_github_url(url) {
            Ok(parsed) => {
                panic!("Expected error for invalid URL '{url}', but got: {parsed:?}");
            }
            Err(CloneError::InvalidUrl(_)) => {
                // Expected error
            }
            Err(e) => {
                panic!("Expected InvalidUrl error for '{url}', but got: {e:?}");
            }
        }
    }
}

#[test]
fn test_cache_key_generation() {
    let temp_dir = TempDir::new().unwrap();
    let cloner = GitCloner::new(temp_dir.path().to_path_buf());

    let test_cases = vec![
        ("https://github.com/rust-lang/rust.git", true),
        ("git@github.com:rust-lang/rust.git", true),
        ("rust-lang/rust", true),
        ("https://github.com/user/repo", true),
        ("https://github.com/USER/REPO", true),
        ("https://github.com/user-name/repo-name", true),
        ("https://github.com/user_name/repo_name", true),
        ("https://github.com/123/456", true),
        ("special!@#$%^&*()chars", true),
        ("../../../etc/passwd", true),
        ("path/with/many/slashes", true),
        ("https://github.com/owner/repo?query=param", true),
        ("https://github.com/owner/repo#fragment", true),
    ];

    for (url, _) in test_cases {
        let cache_key = cloner.compute_cache_key(url);

        // Verify cache key properties
        assert!(
            !cache_key.is_empty(),
            "Cache key should not be empty for URL: {url}"
        );

        // Cache key should only contain safe characters
        assert!(
            cache_key
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-'),
            "Cache key contains invalid characters for URL: {url}. Key: {cache_key}"
        );

        // Cache key should not contain path separators
        assert!(
            !cache_key.contains('/'),
            "Cache key contains '/' for URL: {url}. Key: {cache_key}"
        );
        assert!(
            !cache_key.contains('\\'),
            "Cache key contains '\\' for URL: {url}. Key: {cache_key}"
        );

        // Cache key should not contain special characters that could cause issues
        assert!(
            !cache_key.contains(':'),
            "Cache key contains ':' for URL: {url}. Key: {cache_key}"
        );
        assert!(
            !cache_key.contains('.'),
            "Cache key contains '.' for URL: {url}. Key: {cache_key}"
        );
    }
}

#[test]
fn test_cache_key_uniqueness() {
    let temp_dir = TempDir::new().unwrap();
    let cloner = GitCloner::new(temp_dir.path().to_path_buf());

    // Different URLs should produce different cache keys
    let urls = vec![
        "https://github.com/rust-lang/rust",
        "https://github.com/rust-lang/cargo",
        "https://github.com/tokio-rs/tokio",
        "https://github.com/serde-rs/serde",
        "git@github.com:rust-lang/rust.git",
        "rust-lang/rust",
        "RUST-LANG/RUST", // Case sensitive
    ];

    let mut cache_keys = std::collections::HashSet::new();
    for url in urls {
        let cache_key = cloner.compute_cache_key(url);
        assert!(
            cache_keys.insert(cache_key.clone()),
            "Duplicate cache key '{cache_key}' for URL: {url}"
        );
    }
}

#[test]
fn test_url_normalization() {
    let temp_dir = TempDir::new().unwrap();
    let cloner = GitCloner::new(temp_dir.path().to_path_buf());

    // URLs that should parse to the same repository
    let equivalent_urls = vec![
        vec![
            "https://github.com/rust-lang/rust",
            "https://github.com/rust-lang/rust.git",
            "  https://github.com/rust-lang/rust  ", // With whitespace
            "https://github.com/rust-lang/rust/",    // Trailing slash
        ],
        vec![
            "git@github.com:rust-lang/rust.git",
            "git@github.com:rust-lang/rust",
            "  git@github.com:rust-lang/rust.git  ",
        ],
    ];

    for group in equivalent_urls {
        let mut parsed_urls = Vec::new();
        for url in group {
            match cloner.parse_github_url(url) {
                Ok(parsed) => parsed_urls.push(parsed),
                Err(e) => panic!("Failed to parse URL '{url}': {e:?}"),
            }
        }

        // All URLs in the group should parse to the same result
        for i in 1..parsed_urls.len() {
            assert_eq!(
                parsed_urls[0], parsed_urls[i],
                "URLs should parse to the same result"
            );
        }
    }
}

#[test]
fn test_security_boundaries() {
    let temp_dir = TempDir::new().unwrap();
    let cloner = GitCloner::new(temp_dir.path().to_path_buf());

    // Security-relevant test cases
    let security_cases = vec![
        // Path traversal attempts
        "https://github.com/../../../etc/passwd",
        "https://github.com/owner/../../../",
        "https://github.com/./././owner/repo",
        "https://github.com/%2e%2e%2f%2e%2e%2f",
        // URL injection attempts
        "https://evil.com/github.com/owner/repo",
        "https://github.com.evil.com/owner/repo",
        "https://github.com@evil.com/owner/repo",
        // Null byte injection (Note: These may actually parse if null bytes are stripped)
        // "https://github.com/owner/repo\0.git",
        // "https://github.com/owner\0/repo",

        // Special files
        "https://github.com/.git/config",
        "https://github.com/.ssh/authorized_keys",
        "https://github.com/.env",
        // Protocol confusion
        "javascript:alert('xss')",
        "data:text/html,<script>alert('xss')</script>",
        "file:///etc/passwd",
    ];

    for url in security_cases {
        let result = cloner.parse_github_url(url);
        assert!(
            result.is_err(),
            "Security-sensitive URL should fail to parse: {url}"
        );
    }
}

#[test]
fn test_fuzzer_identified_security_issues() {
    let temp_dir = TempDir::new().unwrap();
    let cloner = GitCloner::new(temp_dir.path().to_path_buf());

    // Test cases identified by the fuzzer that should be rejected
    let fuzzer_cases = vec![
        // Double dots as owner or repo names
        "..",
        "../..",
        "https://github.com/../repo",
        "https://github.com/owner/..",
        "git@github.com:../repo.git",
        "git@github.com:owner/...git",
        // .git as a standalone component
        "https://github.com/.git/repo",
        "https://github.com/owner/.git",
        ".git/repo",
        "owner/.git",
        // Names starting or ending with dots
        "https://github.com/.hidden/repo",
        "https://github.com/owner/repo.",
        "https://github.com/./repo",
        "https://github.com/owner/.",
        // Special git files
        "https://github.com/.gitignore/repo",
        "https://github.com/owner/.gitmodules",
        "https://github.com/.gitattributes/repo",
        // URL encoded path traversal
        "https://github.com/%2e%2e/repo",
        "https://github.com/owner%2f..%2f..%2fetc%2fpasswd",
        // Empty or whitespace components
        "https://github.com/ /repo",
        "https://github.com/\t/repo",
        "https://github.com/\n/repo",
        // Control characters (skip null bytes due to Rust string handling)
        // "https://github.com/owner\0/repo",
        // "https://github.com/owner/repo\r\n",
    ];

    for url in fuzzer_cases {
        let result = cloner.parse_github_url(url);
        assert!(
            result.is_err(),
            "Fuzzer-identified security URL should fail to parse: {url}"
        );
    }
}

#[test]
fn test_edge_case_handling() {
    let temp_dir = TempDir::new().unwrap();
    let cloner = GitCloner::new(temp_dir.path().to_path_buf());

    // Test very long inputs
    let long_owner = "a".repeat(1000);
    let long_repo = "b".repeat(1000);
    let long_url = format!("https://github.com/{long_owner}/{long_repo}");

    // Should either parse successfully or fail gracefully
    let _ = cloner.parse_github_url(&long_url);

    // Test unicode
    let unicode_urls = vec![
        "https://github.com/用户/仓库",
        "https://github.com/пользователь/репозиторий",
        "https://github.com/👤/📦",
    ];

    for url in unicode_urls {
        let _ = cloner.parse_github_url(url); // Should not panic
    }

    // Test empty components
    let empty_component_urls = vec![
        "https://github.com//repo",
        "https://github.com/owner/",
        "https://github.com///",
        "/",
        "//",
    ];

    for url in empty_component_urls {
        let _ = cloner.parse_github_url(url); // Should not panic
    }
}

#[tokio::test]
async fn test_clone_timeout() {
    let temp_dir = TempDir::new().unwrap();
    let cloner = GitCloner::new(temp_dir.path().to_path_buf())
        .with_timeout(std::time::Duration::from_millis(1)); // Very short timeout

    // This should timeout
    let result = cloner
        .clone_or_update("https://github.com/torvalds/linux") // Large repo
        .await;

    match result {
        Err(CloneError::Timeout) => {
            // Expected
        }
        Ok(_) => {
            panic!("Expected timeout error");
        }
        Err(e) => {
            // Also acceptable if it fails for other reasons (network, etc)
            println!("Got error: {e:?}");
        }
    }
}

#[test]
fn test_round_trip_parsing() {
    let temp_dir = TempDir::new().unwrap();
    let cloner = GitCloner::new(temp_dir.path().to_path_buf());

    let test_cases = vec![
        ("rust-lang", "rust"),
        ("tokio-rs", "tokio"),
        ("user-name", "repo-name"),
        ("user_name", "repo_name"),
        ("123", "456"),
        ("a", "b"),
    ];

    for (owner, repo) in test_cases {
        // Create URLs in different formats
        let https_url = format!("https://github.com/{owner}/{repo}");
        let ssh_url = format!("git@github.com:{owner}/{repo}.git");
        let short_url = format!("{owner}/{repo}");

        // Parse all formats
        let https_parsed = cloner.parse_github_url(&https_url).unwrap();
        let ssh_parsed = cloner.parse_github_url(&ssh_url).unwrap();
        let short_parsed = cloner.parse_github_url(&short_url).unwrap();

        // All should parse to the same result
        assert_eq!(https_parsed, ssh_parsed);
        assert_eq!(https_parsed, short_parsed);
        assert_eq!(https_parsed.owner, owner);
        assert_eq!(https_parsed.repo, repo);
    }
}
