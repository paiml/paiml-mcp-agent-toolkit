//! GitHub integration for fetching and parsing issues
//!
//! This module provides functionality to fetch GitHub issues and extract
//! relevant information for the refactor auto command.

use anyhow::{anyhow, Result};
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::env;
use tracing::{debug, info, warn};

/// GitHub issue data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubIssue {
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub html_url: String,
    pub created_at: String,
    pub updated_at: String,
    pub labels: Vec<Label>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub name: String,
}

/// Parsed issue data with extracted information
#[derive(Debug, Clone)]
pub struct ParsedIssue {
    pub issue: GitHubIssue,
    pub file_paths: Vec<String>,
    pub keywords: HashMap<String, f32>, // keyword -> weight
    pub summary: String,
}

/// Keywords mapping to code quality categories
const KEYWORD_MAPPINGS: &[(&[&str], &str, f32)] = &[
    // Performance keywords
    (&["performance", "slow", "optimize", "speed", "latency", "throughput"], "Performance", 3.0),
    // Bug/Correctness keywords
    (&["bug", "error", "fix", "crash", "panic", "broken", "incorrect"], "Correctness", 3.0),
    // Readability/Complexity keywords
    (&["unreadable", "confusing", "cleanup", "refactor", "complex", "complicated", "simplify"], "Complexity", 2.5),
    // Security keywords
    (&["security", "vulnerability", "exploit", "injection", "unsafe"], "Security", 4.0),
    // Technical debt keywords
    (&["debt", "todo", "fixme", "hack", "workaround", "temporary"], "TechnicalDebt", 2.0),
    // Maintainability keywords
    (&["maintain", "maintenance", "coupling", "cohesion", "modular"], "Maintainability", 2.0),
];

/// GitHub API client
pub struct GitHubClient {
    client: reqwest::Client,
    _token: Option<String>,
}

impl GitHubClient {
    /// Create a new GitHub client, optionally with authentication
    pub fn new() -> Result<Self> {
        let token = env::var("GITHUB_TOKEN").ok()
            .or_else(|| env::var("GH_TOKEN").ok());
        
        if token.is_none() {
            warn!("No GitHub token found. API rate limits will be restrictive.");
        }
        
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/vnd.github.v3+json"));
        headers.insert(USER_AGENT, HeaderValue::from_static("pmat/0.1"));
        
        if let Some(ref token) = token {
            let auth_value = format!("Bearer {}", token);
            headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_value)?);
        }
        
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(30))
            .build()?;
        
        Ok(Self { client, _token: token })
    }
    
    /// Fetch a GitHub issue from a URL
    pub async fn fetch_issue(&self, url: &str) -> Result<GitHubIssue> {
        let (owner, repo, issue_number) = Self::parse_issue_url(url)?;
        
        info!("Fetching GitHub issue: {}/{} #{}", owner, repo, issue_number);
        
        let api_url = format!(
            "https://api.github.com/repos/{}/{}/issues/{}",
            owner, repo, issue_number
        );
        
        let response = self.client
            .get(&api_url)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("GitHub API error {}: {}", status, body));
        }
        
        let issue: GitHubIssue = response.json().await?;
        
        if issue.state == "closed" {
            warn!("Issue #{} is closed", issue.number);
        }
        
        Ok(issue)
    }
    
    /// Parse a GitHub issue URL to extract owner, repo, and issue number
    fn parse_issue_url(url: &str) -> Result<(String, String, u64)> {
        let re = Regex::new(r"github\.com/([^/]+)/([^/]+)/issues/(\d+)")?;
        
        let captures = re.captures(url)
            .ok_or_else(|| anyhow!("Invalid GitHub issue URL: {}", url))?;
        
        let owner = captures[1].to_string();
        let repo = captures[2].to_string();
        let issue_number = captures[3].parse::<u64>()?;
        
        Ok((owner, repo, issue_number))
    }
}

/// Parse a GitHub issue to extract relevant information
pub fn parse_issue(issue: GitHubIssue) -> ParsedIssue {
    let mut file_paths = Vec::new();
    let mut keywords = HashMap::new();
    
    // Combine title and body for analysis
    let text = format!(
        "{} {}",
        issue.title,
        issue.body.as_ref().unwrap_or(&String::new())
    );
    
    // Extract file paths
    file_paths.extend(extract_file_paths(&text));
    
    // Extract and weight keywords
    extract_keywords(&text, &mut keywords);
    
    // Generate summary
    let summary = generate_summary(&issue);
    
    ParsedIssue {
        issue,
        file_paths,
        keywords,
        summary,
    }
}

/// Extract file paths from issue text
fn extract_file_paths(text: &str) -> Vec<String> {
    let mut paths = HashSet::new();
    
    // Match common file path patterns
    let patterns = vec![
        // Backtick-quoted paths: `src/file.rs`
        Regex::new(r"`([a-zA-Z0-9_\-./]+\.[a-zA-Z0-9]+)`").unwrap(),
        // Explicit paths: src/services/file.rs or server/src/handlers/mod.rs
        Regex::new(r"\b(?:[a-zA-Z0-9_\-]+/)*[a-zA-Z0-9_\-]+\.[a-zA-Z0-9]+\b").unwrap(),
        // Module paths: services::complexity::analyze
        Regex::new(r"\b[a-zA-Z0-9_]+(?:::[a-zA-Z0-9_]+)+\b").unwrap(),
    ];
    
    for pattern in patterns {
        for capture in pattern.captures_iter(text) {
            if let Some(path) = capture.get(1).or(capture.get(0)) {
                let path_str = path.as_str();
                
                // Convert module paths to file paths
                if path_str.contains("::") {
                    let file_path = path_str.replace("::", "/");
                    paths.insert(format!("src/{}.rs", file_path));
                    paths.insert(format!("server/src/{}.rs", file_path));
                } else {
                    paths.insert(path_str.to_string());
                }
            }
        }
    }
    
    let mut sorted_paths: Vec<_> = paths.into_iter().collect();
    sorted_paths.sort();
    
    debug!("Extracted {} file paths from issue", sorted_paths.len());
    sorted_paths
}

/// Extract keywords and assign weights based on predefined mappings
fn extract_keywords(text: &str, keywords: &mut HashMap<String, f32>) {
    let text_lower = text.to_lowercase();
    
    for (keyword_list, category, weight) in KEYWORD_MAPPINGS {
        for keyword in *keyword_list {
            if text_lower.contains(keyword) {
                // Count occurrences
                let count = text_lower.matches(keyword).count() as f32;
                let adjusted_weight = weight * (1.0 + (count - 1.0) * 0.2).min(2.0);
                
                // Update category weight
                let entry = keywords.entry(category.to_string()).or_insert(0.0);
                *entry = (*entry + adjusted_weight).min(weight * 2.0);
                
                debug!("Found keyword '{}' in category {} (weight: {:.1})", 
                    keyword, category, adjusted_weight);
            }
        }
    }
    
    // Normalize weights
    if !keywords.is_empty() {
        let max_weight = keywords.values().fold(0.0f32, |a, &b| a.max(b));
        if max_weight > 0.0 {
            for weight in keywords.values_mut() {
                *weight /= max_weight;
            }
        }
    }
}

/// Generate a concise summary of the issue
fn generate_summary(issue: &GitHubIssue) -> String {
    let body = issue.body.as_ref().map(|b| {
        // Take first paragraph or first 200 characters
        let first_paragraph = b.split("\n\n").next().unwrap_or(b);
        if first_paragraph.len() > 200 {
            format!("{}...", &first_paragraph[..200])
        } else {
            first_paragraph.to_string()
        }
    }).unwrap_or_default();
    
    if body.is_empty() {
        issue.title.clone()
    } else {
        format!("{}\n\n{}", issue.title, body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_file_paths() {
        let text = r#"
        The issue is in `src/services/complexity.rs` and also affects
        the module services::ast_rust::analyze. Additionally, check
        server/src/handlers/mod.rs for related code.
        "#;
        
        let paths = extract_file_paths(text);
        
        assert!(paths.contains(&"src/services/complexity.rs".to_string()));
        assert!(paths.contains(&"server/src/handlers/mod.rs".to_string()));
        assert!(paths.iter().any(|p| p.contains("services/ast_rust")));
    }
    
    #[test]
    fn test_extract_keywords() {
        let text = "This function has terrible performance and is very slow. 
                   It's also confusing and needs optimization.";
        
        let mut keywords = HashMap::new();
        extract_keywords(text, &mut keywords);
        
        assert!(keywords.contains_key("Performance"));
        assert!(keywords.contains_key("Complexity"));
        assert!(keywords["Performance"] > 0.5); // Should be high weight
    }
    
    #[test]
    fn test_parse_issue_url() {
        let url = "https://github.com/owner/repo/issues/123";
        let (owner, repo, number) = GitHubClient::parse_issue_url(url).unwrap();
        
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
        assert_eq!(number, 123);
    }
}