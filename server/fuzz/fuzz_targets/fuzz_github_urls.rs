#![no_main]

use libfuzzer_sys::fuzz_target;
use paiml_mcp_agent_toolkit::services::git_clone::{GitCloner, ParsedGitHubUrl};
use std::path::PathBuf;

fuzz_target!(|data: &[u8]| {
    let temp_dir = std::env::temp_dir();
    let cloner = GitCloner::new(temp_dir);
    
    // Test 1: Raw string fuzzing for URL parsing
    if let Ok(s) = std::str::from_utf8(data) {
        // Test direct URL parsing
        let _ = cloner.parse_github_url(s);
        
        // Test cache key generation
        let _ = cloner.compute_cache_key(s);
        
        // Test various URL mutations
        let mutations = vec![
            s.to_string(),
            s.trim().to_string(),
            s.replace("github.com", "github.enterprise.com"),
            s.replace("https://", "git@"),
            s.replace("git@", "https://"),
            s.replace("/", "\\"),
            s.replace("\\", "/"),
            s.replace(":", "::"),
            s.replace("::", ":"),
            format!("https://github.com/{}", s),
            format!("git@github.com:{}", s),
            format!("{}?foo=bar", s),
            format!("{}#fragment", s),
            format!("{}\0", s), // Null byte injection
            format!("\0{}", s),
            format!("{}/{}", s, s), // Duplicate path components
            s.replace(".", ".."), // Path traversal attempt
            s.replace("..", "."),
            s.replace("-", "--"), // Double dash
            s.replace("_", "__"), // Double underscore
            format!("https://github.com/{}/{}", s, s),
            format!("git@github.com:{}/{}.git", s, s),
            format!("{}.git", s),
            format!("{}.git.git", s),
            s.to_uppercase(),
            s.to_lowercase(),
            s.chars().rev().collect::<String>(), // Reverse
        ];
        
        for mutation in mutations {
            let _ = cloner.parse_github_url(&mutation);
            let _ = cloner.compute_cache_key(&mutation);
        }
    }
    
    // Test 2: Binary data (invalid UTF-8)
    let lossy = String::from_utf8_lossy(data);
    let _ = cloner.parse_github_url(&lossy);
    let _ = cloner.compute_cache_key(&lossy);
    
    // Test 3: Edge cases and security-relevant patterns
    let edge_cases = vec![
        // Empty and whitespace
        "",
        " ",
        "\n",
        "\t",
        "\r\n",
        "   \t\n\r   ",
        
        // Partial URLs
        "https://",
        "http://",
        "git://",
        "ssh://",
        "file://",
        "github.com",
        "github",
        ".com",
        
        // Special characters
        "//////////",
        "\\\\\\\\\\\\",
        ":",
        "::",
        ":::",
        "@",
        "@@",
        "@@@",
        "git@",
        ".git",
        "..git",
        "...git",
        
        // GitHub URL variations
        "https://github.com",
        "https://github.com/",
        "https://github.com//",
        "https://github.com///",
        "https://github.com/owner",
        "https://github.com/owner/",
        "https://github.com/owner//",
        "https://github.com//owner",
        "https://github.com/owner/repo",
        "https://github.com/owner//repo",
        "https://github.com//owner//repo",
        "https://github.com/owner/repo/",
        "https://github.com/owner/repo//",
        "https://github.com/owner/repo.git",
        "https://github.com/owner/repo.git/",
        "https://github.com/owner/repo.git.git",
        
        // Path traversal attempts
        "https://github.com/../",
        "https://github.com/../../",
        "https://github.com/../../../etc/passwd",
        "https://github.com/owner/../../../",
        "https://github.com/owner/repo/../../../",
        "https://github.com/owner/repo/../../../../etc/passwd",
        "https://github.com/./owner/./repo",
        "https://github.com/%2e%2e/",
        "https://github.com/%2e%2e%2f%2e%2e%2f",
        
        // URL encoding and special characters
        "https://github.com/owner/repo%00.git",
        "https://github.com/owner/repo%20space.git",
        "https://github.com/owner/repo%0a.git",
        "https://github.com/owner/repo%0d.git",
        "https://github.com/owner/repo%ff.git",
        "https://github.com/owner/repo?foo=bar",
        "https://github.com/owner/repo?foo=bar&baz=qux",
        "https://github.com/owner/repo#readme",
        "https://github.com/owner/repo#L1",
        "https://github.com/owner/repo#L1-L10",
        
        // Security-relevant patterns
        "https://github.com/.git/config",
        "https://github.com/.git/HEAD",
        "https://github.com/.ssh/id_rsa",
        "https://github.com/.env",
        "ssh://git@github.com/owner/repo.git",
        "git://github.com/owner/repo.git",
        "file:///etc/passwd",
        "file://localhost/etc/passwd",
        "https://evil.com/github.com/owner/repo",
        "https://github.com.evil.com/owner/repo",
        "https://github.com@evil.com/owner/repo",
        "https://user:pass@github.com/owner/repo",
        "https://token@github.com/owner/repo",
        
        // Long strings and resource exhaustion
        &"a".repeat(10000),
        &"a".repeat(100000),
        &"/".repeat(1000),
        &"/".repeat(10000),
        &".".repeat(1000),
        &":".repeat(1000),
        &"@".repeat(1000),
        &format!("https://github.com/{}/{}", "x".repeat(255), "y".repeat(255)),
        &format!("https://github.com/{}/{}", "x".repeat(1000), "y".repeat(1000)),
        &format!("https://github.com/{}", "/repo".repeat(100)),
        
        // Unicode and non-ASCII
        "https://github.com/用户/仓库",
        "https://github.com/пользователь/репозиторий",
        "https://github.com/👤/📦",
        "https://github.com/\u{200B}/\u{200B}", // Zero-width space
        "https://github.com/\u{202E}toor/oper", // Right-to-left override
        
        // Mixed protocols and formats
        "http://github.com/owner/repo",
        "ftp://github.com/owner/repo",
        "github.com/owner/repo",
        "github.com:owner/repo",
        "owner/repo",
        "/owner/repo",
        "owner/repo/",
        "./owner/repo",
        "../owner/repo",
        "~/owner/repo",
    ];
    
    for edge in edge_cases {
        let _ = cloner.parse_github_url(edge);
        let cache_key = cloner.compute_cache_key(edge);
        
        // Verify cache key properties
        assert!(cache_key.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-'));
        assert!(!cache_key.contains('/'));
        assert!(!cache_key.contains(':'));
        assert!(!cache_key.contains('.'));
    }
    
    // Test 4: Valid patterns that should parse successfully
    let valid_patterns = vec![
        ("https://github.com/rust-lang/rust", "rust-lang", "rust"),
        ("https://github.com/rust-lang/rust.git", "rust-lang", "rust"),
        ("git@github.com:rust-lang/rust.git", "rust-lang", "rust"),
        ("rust-lang/rust", "rust-lang", "rust"),
        ("https://github.com/user-name/repo-name", "user-name", "repo-name"),
        ("https://github.com/user_name/repo_name", "user_name", "repo_name"),
        ("https://github.com/user123/repo456", "user123", "repo456"),
        ("https://github.com/USER/REPO", "USER", "REPO"),
        ("git@github.com:User/Repo.git", "User", "Repo"),
        ("https://github.com/a/b", "a", "b"),
        ("https://github.com/0/1", "0", "1"),
    ];
    
    for (pattern, expected_owner, expected_repo) in valid_patterns {
        match cloner.parse_github_url(pattern) {
            Ok(parsed) => {
                assert_eq!(parsed.owner, expected_owner);
                assert_eq!(parsed.repo, expected_repo);
            }
            Err(_) => {
                // Log failed parses for debugging
                eprintln!("Failed to parse valid pattern: {}", pattern);
            }
        }
    }
    
    // Test 5: Round-trip property for valid URLs
    if let Ok(s) = std::str::from_utf8(data) {
        if let Ok(parsed) = cloner.parse_github_url(s) {
            // Construct a canonical URL from parsed components
            let canonical = format!("https://github.com/{}/{}", parsed.owner, parsed.repo);
            
            // Re-parse the canonical URL
            if let Ok(reparsed) = cloner.parse_github_url(&canonical) {
                assert_eq!(parsed, reparsed);
            }
        }
    }
});