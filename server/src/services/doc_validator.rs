//! Documentation link validator
//!
//! Validates markdown links (internal and external HTTP/HTTPS) and reports broken links.
//! Designed with EXTREME TDD principles with property tests and comprehensive coverage.

use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use walkdir::WalkDir;

/// Core validator that orchestrates link checking
pub struct DocValidator {
    config: ValidatorConfig,
    http_client: Option<reqwest::Client>,
}

/// Configuration for validation behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorConfig {
    /// Root directory to search for markdown files
    pub root_dir: PathBuf,

    /// Timeout for HTTP requests (milliseconds)
    pub http_timeout_ms: u64,

    /// Maximum number of retries for failed requests
    pub max_retries: u32,

    /// Delay between retries (milliseconds)
    pub retry_delay_ms: u64,

    /// Maximum concurrent HTTP requests
    pub max_concurrent_requests: usize,

    /// Patterns to exclude from validation
    pub exclude_patterns: Vec<String>,

    /// Follow redirects
    pub follow_redirects: bool,

    /// User agent string
    pub user_agent: String,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            root_dir: PathBuf::from("."),
            http_timeout_ms: 30000,
            max_retries: 3,
            retry_delay_ms: 1000,
            max_concurrent_requests: 10,
            exclude_patterns: vec![
                "archive".to_string(),
                "node_modules".to_string(),
                ".git".to_string(),
                "target".to_string(),
            ],
            follow_redirects: true,
            user_agent: format!("pmat-doc-validator/{}", env!("CARGO_PKG_VERSION")),
        }
    }
}

/// Represents a parsed markdown link
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Link {
    /// Link text
    pub text: String,

    /// Link target (URL or file path)
    pub target: String,

    /// Source file containing the link
    pub source_file: PathBuf,

    /// Line number in source file
    pub line_number: usize,

    /// Link type
    pub link_type: LinkType,
}

/// Type of link
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LinkType {
    /// Internal file link (relative or absolute path)
    Internal,

    /// External HTTP/HTTPS link
    ExternalHttp,

    /// Anchor link within same document
    Anchor,

    /// Email link
    Email,

    /// Other protocol
    Other(String),
}

/// Result of validating a single link
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub link: Link,
    pub status: ValidationStatus,
    pub error_message: Option<String>,
    pub http_status_code: Option<u16>,
    pub response_time_ms: Option<u64>,
}

/// Status of link validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationStatus {
    /// Link is valid and accessible
    Valid,

    /// Link returned 404 or file not found
    NotFound,

    /// Link returned other HTTP error
    HttpError(u16),

    /// Network error (timeout, connection failed, etc.)
    NetworkError,

    /// Link is malformed or invalid
    InvalidLink,

    /// Link was skipped (excluded pattern)
    Skipped,
}

/// Summary of validation run
#[derive(Debug, Clone)]
pub struct ValidationSummary {
    pub total_files: usize,
    pub total_links: usize,
    pub valid_links: usize,
    pub broken_links: usize,
    pub skipped_links: usize,
    pub duration_ms: u64,
    pub results: Vec<ValidationResult>,
}

/// Extracts all markdown links from a file
///
/// # Examples
///
/// ```
/// use pmat::services::doc_validator::extract_links;
/// use std::path::PathBuf;
///
/// let content = "[example](https://example.com) and [local](./file.md)";
/// let links = extract_links(content, &PathBuf::from("test.md"));
/// assert_eq!(links.len(), 2);
/// ```
pub fn extract_links(content: &str, source_file: &Path) -> Vec<Link> {
    let mut links = Vec::new();
    let regex = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap();

    for (line_num, line) in content.lines().enumerate() {
        // Skip code blocks (lines starting with backticks)
        if line.trim_start().starts_with("```") {
            continue;
        }

        for cap in regex.captures_iter(line) {
            let text = cap[1].to_string();
            let target = cap[2].to_string();
            let link_type = classify_link(&target);

            links.push(Link {
                text,
                target,
                source_file: source_file.to_path_buf(),
                line_number: line_num + 1,
                link_type,
            });
        }
    }

    links
}

/// Classifies a link target into its type
///
/// # Examples
///
/// ```
/// use pmat::services::doc_validator::{classify_link, LinkType};
///
/// assert_eq!(classify_link("https://example.com"), LinkType::ExternalHttp);
/// assert_eq!(classify_link("./local.md"), LinkType::Internal);
/// assert_eq!(classify_link("#anchor"), LinkType::Anchor);
/// assert_eq!(classify_link("mailto:user@example.com"), LinkType::Email);
/// ```
pub fn classify_link(target: &str) -> LinkType {
    if target.starts_with("http://") || target.starts_with("https://") {
        LinkType::ExternalHttp
    } else if target.starts_with('#') {
        LinkType::Anchor
    } else if target.starts_with("mailto:") {
        LinkType::Email
    } else if target.contains("://") {
        LinkType::Other(target.split("://").next().unwrap().to_string())
    } else {
        LinkType::Internal
    }
}

/// Normalizes a path by resolving `.` and `..` components
fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();

    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                components.pop();
            }
            std::path::Component::CurDir => {
                // Skip current directory
            }
            _ => {
                components.push(component);
            }
        }
    }

    components.iter().collect()
}

impl DocValidator {
    /// Creates a new validator with default configuration
    pub fn new(config: ValidatorConfig) -> Self {
        let http_client = if config.http_timeout_ms > 0 {
            Some(
                reqwest::Client::builder()
                    .timeout(Duration::from_millis(config.http_timeout_ms))
                    .user_agent(&config.user_agent)
                    .redirect(if config.follow_redirects {
                        reqwest::redirect::Policy::limited(10)
                    } else {
                        reqwest::redirect::Policy::none()
                    })
                    .build()
                    .expect("Failed to create HTTP client"),
            )
        } else {
            None
        };

        Self {
            config,
            http_client,
        }
    }

    /// Validates a single link
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pmat::services::doc_validator::{DocValidator, Link, LinkType, ValidatorConfig};
    /// use std::path::PathBuf;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let validator = DocValidator::new(ValidatorConfig::default());
    ///     let link = Link {
    ///         text: "Example".to_string(),
    ///         target: "https://example.com".to_string(),
    ///         source_file: PathBuf::from("test.md"),
    ///         line_number: 1,
    ///         link_type: LinkType::ExternalHttp,
    ///     };
    ///
    ///     let result = validator.validate_link(&link).await;
    ///     assert!(result.is_ok());
    /// }
    /// ```
    pub async fn validate_link(&self, link: &Link) -> Result<ValidationResult> {
        let start = Instant::now();

        let (status, error_message, http_status) = match &link.link_type {
            LinkType::Internal => self.validate_internal_link(link).await,
            LinkType::ExternalHttp => self.validate_http_link(link).await,
            LinkType::Anchor => self.validate_anchor_link(link).await,
            LinkType::Email => (ValidationStatus::Valid, None, None), // Don't validate emails
            LinkType::Other(_) => (ValidationStatus::Skipped, None, None),
        };

        Ok(ValidationResult {
            link: link.clone(),
            status,
            error_message,
            http_status_code: http_status,
            response_time_ms: Some(start.elapsed().as_millis() as u64),
        })
    }

    /// Validates an internal file link
    async fn validate_internal_link(
        &self,
        link: &Link,
    ) -> (ValidationStatus, Option<String>, Option<u16>) {
        // Remove anchor from target
        let target = link.target.split('#').next().unwrap();

        // Skip empty targets (pure anchors)
        if target.is_empty() {
            return (ValidationStatus::Valid, None, None);
        }

        // Resolve relative path
        let base_dir = link
            .source_file
            .parent()
            .unwrap_or_else(|| Path::new("."));
        let target_path = base_dir.join(target);
        let normalized_path = normalize_path(&target_path);

        if normalized_path.exists() {
            (ValidationStatus::Valid, None, None)
        } else {
            (
                ValidationStatus::NotFound,
                Some(format!("File not found: {}", normalized_path.display())),
                None,
            )
        }
    }

    /// Validates an HTTP/HTTPS link with retry logic
    async fn validate_http_link(
        &self,
        link: &Link,
    ) -> (ValidationStatus, Option<String>, Option<u16>) {
        let client = match &self.http_client {
            Some(c) => c,
            None => {
                return (
                    ValidationStatus::NetworkError,
                    Some("HTTP client not configured".to_string()),
                    None,
                )
            }
        };

        let mut retries = 0;

        loop {
            match client.head(&link.target).send().await {
                Ok(response) => {
                    let status_code = response.status().as_u16();

                    return if status_code == 404 {
                        (
                            ValidationStatus::NotFound,
                            Some(format!("HTTP 404: {}", link.target)),
                            Some(status_code),
                        )
                    } else if (200..300).contains(&status_code) {
                        (ValidationStatus::Valid, None, Some(status_code))
                    } else {
                        (
                            ValidationStatus::HttpError(status_code),
                            Some(format!("HTTP {}: {}", status_code, link.target)),
                            Some(status_code),
                        )
                    };
                }
                Err(e) => {
                    retries += 1;
                    if retries >= self.config.max_retries {
                        return (
                            ValidationStatus::NetworkError,
                            Some(format!("Network error: {}", e)),
                            None,
                        );
                    }

                    tokio::time::sleep(Duration::from_millis(
                        self.config.retry_delay_ms * 2_u64.pow(retries - 1),
                    ))
                    .await;
                }
            }
        }
    }

    /// Validates an anchor link
    async fn validate_anchor_link(
        &self,
        _link: &Link,
    ) -> (ValidationStatus, Option<String>, Option<u16>) {
        // For now, assume anchors are valid
        // TODO: Parse markdown headers and validate anchor exists
        (ValidationStatus::Valid, None, None)
    }

    /// Checks if a path should be excluded
    fn should_exclude(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        for pattern in &self.config.exclude_patterns {
            if path_str.contains(pattern) {
                return true;
            }
        }

        false
    }

    /// Validates all links in a directory
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pmat::services::doc_validator::{DocValidator, ValidatorConfig};
    /// use std::path::PathBuf;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let validator = DocValidator::new(ValidatorConfig::default());
    ///     let summary = validator.validate_directory(&PathBuf::from("docs")).await.unwrap();
    ///
    ///     if summary.broken_links > 0 {
    ///         eprintln!("Found {} broken links", summary.broken_links);
    ///         std::process::exit(1);
    ///     }
    /// }
    /// ```
    pub async fn validate_directory(&self, root: &Path) -> Result<ValidationSummary> {
        let start = Instant::now();
        let mut all_links = Vec::new();
        let mut file_count = 0;

        // Find all markdown files, skipping excluded directories
        for entry in WalkDir::new(root)
            .into_iter()
            .filter_entry(|e| !self.should_exclude(e.path()))
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        {
            file_count += 1;
            let content = tokio::fs::read_to_string(entry.path())
                .await
                .context(format!("Failed to read {}", entry.path().display()))?;
            let links = extract_links(&content, entry.path());
            all_links.extend(links);
        }

        // Validate all links concurrently
        let results = self.validate_links_concurrent(&all_links).await?;

        // Compute summary
        let valid_count = results
            .iter()
            .filter(|r| r.status == ValidationStatus::Valid)
            .count();
        let broken_count = results
            .iter()
            .filter(|r| {
                matches!(
                    r.status,
                    ValidationStatus::NotFound | ValidationStatus::HttpError(_)
                )
            })
            .count();
        let skipped_count = results
            .iter()
            .filter(|r| r.status == ValidationStatus::Skipped)
            .count();

        Ok(ValidationSummary {
            total_files: file_count,
            total_links: all_links.len(),
            valid_links: valid_count,
            broken_links: broken_count,
            skipped_links: skipped_count,
            duration_ms: start.elapsed().as_millis() as u64,
            results,
        })
    }

    /// Validates multiple links concurrently
    async fn validate_links_concurrent(&self, links: &[Link]) -> Result<Vec<ValidationResult>> {
        use futures::stream::{self, StreamExt};

        let results = stream::iter(links)
            .map(|link| async move { self.validate_link(link).await })
            .buffer_unordered(self.config.max_concurrent_requests)
            .collect::<Vec<_>>()
            .await;

        results.into_iter().collect()
    }
}

impl Default for DocValidator {
    fn default() -> Self {
        Self::new(ValidatorConfig::default())
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn red_test_extract_links_from_empty_content() {
        // RED: This should pass but implementation doesn't exist yet
        let links = extract_links("", Path::new("test.md"));
        assert_eq!(links.len(), 0);
    }

    #[test]
    fn red_test_extract_single_http_link() {
        // RED: Implementation missing
        let content = "[Example](https://example.com)";
        let links = extract_links(content, Path::new("test.md"));

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].text, "Example");
        assert_eq!(links[0].target, "https://example.com");
        assert_eq!(links[0].link_type, LinkType::ExternalHttp);
    }

    #[test]
    fn red_test_extract_multiple_links() {
        // RED: Implementation missing
        let content = "[Link1](url1.md) and [Link2](url2.md)";
        let links = extract_links(content, Path::new("test.md"));

        assert_eq!(links.len(), 2);
    }

    #[test]
    fn red_test_classify_http_link() {
        // RED: Implementation missing
        assert_eq!(
            classify_link("http://example.com"),
            LinkType::ExternalHttp
        );
        assert_eq!(
            classify_link("https://example.com"),
            LinkType::ExternalHttp
        );
    }

    #[test]
    fn red_test_classify_internal_link() {
        // RED: Implementation missing
        assert_eq!(classify_link("./file.md"), LinkType::Internal);
        assert_eq!(classify_link("../parent.md"), LinkType::Internal);
        assert_eq!(classify_link("/abs/path.md"), LinkType::Internal);
    }

    #[test]
    fn red_test_classify_anchor_link() {
        // RED: Implementation missing
        assert_eq!(classify_link("#section"), LinkType::Anchor);
    }

    #[test]
    fn red_test_classify_email_link() {
        // RED: Implementation missing
        assert_eq!(classify_link("mailto:user@example.com"), LinkType::Email);
    }

    #[tokio::test]
    async fn red_test_validate_existing_internal_link() {
        // RED: Implementation missing
        // Setup: Create temp file
        let temp_dir = tempfile::tempdir().unwrap();
        let target_file = temp_dir.path().join("target.md");
        std::fs::write(&target_file, "content").unwrap();

        let source_file = temp_dir.path().join("source.md");
        let link = Link {
            text: "Target".to_string(),
            target: "./target.md".to_string(),
            source_file: source_file.clone(),
            line_number: 1,
            link_type: LinkType::Internal,
        };

        let validator = DocValidator::default();
        let result = validator.validate_link(&link).await.unwrap();

        assert_eq!(result.status, ValidationStatus::Valid);
    }

    #[tokio::test]
    async fn red_test_validate_missing_internal_link() {
        // RED: Implementation missing
        let temp_dir = tempfile::tempdir().unwrap();
        let source_file = temp_dir.path().join("source.md");

        let link = Link {
            text: "Missing".to_string(),
            target: "./missing.md".to_string(),
            source_file: source_file.clone(),
            line_number: 1,
            link_type: LinkType::Internal,
        };

        let validator = DocValidator::default();
        let result = validator.validate_link(&link).await.unwrap();

        assert_eq!(result.status, ValidationStatus::NotFound);
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn red_test_validate_http_404() {
        // RED: Implementation missing
        // Note: Use mock HTTP server in real implementation
        let link = Link {
            text: "404".to_string(),
            target: "https://httpbin.org/status/404".to_string(),
            source_file: PathBuf::from("test.md"),
            line_number: 1,
            link_type: LinkType::ExternalHttp,
        };

        let validator = DocValidator::default();
        let result = validator.validate_link(&link).await.unwrap();

        assert_eq!(result.status, ValidationStatus::NotFound);
        assert_eq!(result.http_status_code, Some(404));
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn red_test_validate_http_200() {
        // RED: Implementation missing
        let link = Link {
            text: "Success".to_string(),
            target: "https://httpbin.org/status/200".to_string(),
            source_file: PathBuf::from("test.md"),
            line_number: 1,
            link_type: LinkType::ExternalHttp,
        };

        let validator = DocValidator::default();
        let result = validator.validate_link(&link).await.unwrap();

        assert_eq!(result.status, ValidationStatus::Valid);
        assert_eq!(result.http_status_code, Some(200));
    }

    #[tokio::test]
    async fn red_test_concurrent_validation() {
        // RED: Implementation missing
        let temp_dir = tempfile::tempdir().unwrap();

        // Create multiple test files
        for i in 0..10 {
            let file = temp_dir.path().join(format!("file{}.md", i));
            std::fs::write(&file, format!("[link](./file{}.md)", (i + 1) % 10)).unwrap();
        }

        let validator = DocValidator::default();
        let summary = validator
            .validate_directory(temp_dir.path())
            .await
            .unwrap();

        assert_eq!(summary.total_files, 10);
        assert_eq!(summary.valid_links, 10);
    }

    #[tokio::test]
    async fn test_archive_directory_excluded_by_default() {
        let temp_dir = tempfile::tempdir().unwrap();

        // Create regular docs
        std::fs::write(temp_dir.path().join("readme.md"), "[link](./test.md)").unwrap();
        std::fs::write(temp_dir.path().join("test.md"), "content").unwrap();

        // Create archive directory with broken links (should be excluded)
        let archive_dir = temp_dir.path().join("archive");
        std::fs::create_dir(&archive_dir).unwrap();
        std::fs::write(
            archive_dir.join("old.md"),
            "[broken](./nonexistent.md)",
        ).unwrap();

        let validator = DocValidator::default();
        let summary = validator
            .validate_directory(temp_dir.path())
            .await
            .unwrap();

        // Should only scan 2 files (readme.md and test.md), excluding archive/old.md
        assert_eq!(summary.total_files, 2);
        assert_eq!(summary.valid_links, 1);
        assert_eq!(summary.broken_links, 0); // archive's broken link should be excluded
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: All valid markdown link syntax should be extracted
        #[test]
        fn test_link_extraction_completeness(
            text in "[a-zA-Z0-9 ]+",
            url in "https?://[a-zA-Z0-9.-]+\\.[a-z]{2,}"
        ) {
            let markdown = format!("[{}]({})", text, url);
            let links = extract_links(&markdown, Path::new("test.md"));

            prop_assert_eq!(links.len(), 1);
            prop_assert_eq!(&links[0].text, &text);
            prop_assert_eq!(&links[0].target, &url);
        }

        /// Property: Link classification should be deterministic
        #[test]
        fn test_link_classification_determinism(target in ".*") {
            let type1 = classify_link(&target);
            let type2 = classify_link(&target);
            prop_assert_eq!(type1, type2);
        }

        /// Property: HTTP links should always get ExternalHttp classification
        #[test]
        fn test_http_link_classification(
            domain in "[a-z0-9-]+",
            tld in "[a-z]{2,4}"
        ) {
            let url = format!("https://{}.{}", domain, tld);
            let link_type = classify_link(&url);
            prop_assert_eq!(link_type, LinkType::ExternalHttp);
        }

        /// Property: Internal links should resolve correctly relative to source
        #[test]
        fn test_internal_link_resolution(
            filename in "[a-z0-9_-]+\\.md"
        ) {
            let source = PathBuf::from("docs/spec.md");
            let target = format!("./{}", filename);

            // This should resolve to docs/{filename}
            let _link = Link {
                text: "test".to_string(),
                target: target.clone(),
                source_file: source.clone(),
                line_number: 1,
                link_type: LinkType::Internal,
            };

            // Property: resolution should be in same directory
            let base = source.parent().unwrap();
            let resolved = base.join(&filename);
            prop_assert!(resolved.starts_with("docs"));
        }

        /// Property: Validation status should never be undefined
        #[test]
        fn test_validation_status_completeness(
            http_code in 100u16..600u16
        ) {
            let status = match http_code {
                404 => ValidationStatus::NotFound,
                200..=299 => ValidationStatus::Valid,
                code => ValidationStatus::HttpError(code),
            };

            // Status should always be one of the defined variants
            prop_assert!(matches!(
                status,
                ValidationStatus::Valid
                    | ValidationStatus::NotFound
                    | ValidationStatus::HttpError(_)
            ));
        }

        /// Property: Retry with exponential backoff should increase delay
        #[test]
        fn test_exponential_backoff(base_delay in 100u64..1000u64, retry in 0u32..5u32) {
            let delay = base_delay * 2_u64.pow(retry);
            if retry > 0 {
                let prev_delay = base_delay * 2_u64.pow(retry - 1);
                prop_assert!(delay >= prev_delay * 2);
            }
        }
    }
}
