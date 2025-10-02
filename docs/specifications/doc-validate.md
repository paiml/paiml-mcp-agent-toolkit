# Documentation Link Validation Specification

**Status**: Active
**Type**: Specification
**Created**: 2025-10-02
**Updated**: 2025-10-02
**Priority**: P0
**Complexity**: Medium

---

## Executive Summary

This specification defines a comprehensive markdown documentation link validator that integrates into the PMAT CLI. The validator will check all markdown files in a project for broken links (both internal and external HTTP/HTTPS), failing builds on 404 errors. Implementation follows EXTREME TDD principles with property tests, doctests, comprehensive examples, and full quality gate enforcement.

## 1. Problem Statement

### 1.1 Current State
- Python script `scripts/validate-docs.py` exists but is not integrated into PMAT
- No automated validation of external HTTP/HTTPS links
- No build failure on broken documentation links
- No property-based testing for link validation logic
- Manual process prone to documentation drift

### 1.2 Desired State
- Native Rust implementation integrated into PMAT CLI
- Validates both internal file links and external HTTP/HTTPS links
- Fails CI/CD builds on 404 errors
- Comprehensive test coverage with property tests
- Performance-optimized with concurrent HTTP checks
- Configurable timeouts and retry logic

## 2. Design Principles

- **EXTREME TDD**: Write failing tests first, implement to pass
- **Quality-First**: 100% test coverage, property tests, doctests
- **Performance**: Concurrent HTTP requests with connection pooling
- **Reliability**: Retry logic with exponential backoff
- **Configurability**: Timeout, retry, and exclusion settings
- **Determinism**: Stable, reproducible validation results

## 3. Architecture

### 3.1 Component Overview

```rust
/// Core validator that orchestrates link checking
pub struct DocValidator {
    config: ValidatorConfig,
    http_client: HttpClient,
    cache: LinkCache,
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
```

### 3.2 Core Algorithms

#### 3.2.1 Link Extraction

```rust
/// Extracts all markdown links from a file
///
/// # Examples
///
/// ```
/// use pmat::doc_validator::extract_links;
/// use std::path::PathBuf;
///
/// let content = "[example](https://example.com) and [local](./file.md)";
/// let links = extract_links(content, &PathBuf::from("test.md"));
/// assert_eq!(links.len(), 2);
/// ```
pub fun extract_links(content: &str, source_file: &Path) -> Vec<Link> {
    let mut links = Vec::new();
    let regex = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap();

    for (line_num, line) in content.lines().enumerate() {
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
/// use pmat::doc_validator::{classify_link, LinkType};
///
/// assert_eq!(classify_link("https://example.com"), LinkType::ExternalHttp);
/// assert_eq!(classify_link("./local.md"), LinkType::Internal);
/// assert_eq!(classify_link("#anchor"), LinkType::Anchor);
/// assert_eq!(classify_link("mailto:user@example.com"), LinkType::Email);
/// ```
pub fun classify_link(target: &str) -> LinkType {
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
```

#### 3.2.2 Link Validation

```rust
impl DocValidator {
    /// Validates a single link
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pmat::doc_validator::{DocValidator, Link, LinkType};
    /// use std::path::PathBuf;
    ///
    /// let validator = DocValidator::default();
    /// let link = Link {
    ///     text: "Example".to_string(),
    ///     target: "https://example.com".to_string(),
    ///     source_file: PathBuf::from("test.md"),
    ///     line_number: 1,
    ///     link_type: LinkType::ExternalHttp,
    /// };
    ///
    /// let result = validator.validate_link(&link).await;
    /// assert!(result.is_ok());
    /// ```
    pub async fun validate_link(&self, link: &Link) -> Result<ValidationResult> {
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
    async fun validate_internal_link(&self, link: &Link) -> (ValidationStatus, Option<String>, Option<u16>) {
        // Remove anchor from target
        let target = link.target.split('#').next().unwrap();

        // Resolve relative path
        let base_dir = link.source_file.parent().unwrap();
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
    async fun validate_http_link(&self, link: &Link) -> (ValidationStatus, Option<String>, Option<u16>) {
        let mut retries = 0;

        loop {
            match self.http_client.head(&link.target).await {
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
                        self.config.retry_delay_ms * 2_u64.pow(retries - 1)
                    )).await;
                }
            }
        }
    }

    /// Validates all links in a directory
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pmat::doc_validator::DocValidator;
    /// use std::path::PathBuf;
    ///
    /// #[tokio::main]
    /// async fun main() {
    ///     let validator = DocValidator::default();
    ///     let summary = validator.validate_directory(&PathBuf::from("docs")).await.unwrap();
    ///
    ///     if summary.broken_links > 0 {
    ///         eprintln!("Found {} broken links", summary.broken_links);
    ///         std::process::exit(1);
    ///     }
    /// }
    /// ```
    pub async fun validate_directory(&self, root: &Path) -> Result<ValidationSummary> {
        let start = Instant::now();
        let mut all_links = Vec::new();
        let mut file_count = 0;

        // Find all markdown files
        for entry in WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
        {
            if self.should_exclude(entry.path()) {
                continue;
            }

            file_count += 1;
            let content = tokio::fs::read_to_string(entry.path()).await?;
            let links = extract_links(&content, entry.path());
            all_links.extend(links);
        }

        // Validate all links concurrently
        let results = self.validate_links_concurrent(&all_links).await?;

        // Compute summary
        let valid_count = results.iter().filter(|r| r.status == ValidationStatus::Valid).count();
        let broken_count = results.iter().filter(|r| {
            matches!(r.status, ValidationStatus::NotFound | ValidationStatus::HttpError(_))
        }).count();
        let skipped_count = results.iter().filter(|r| r.status == ValidationStatus::Skipped).count();

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
}
```

### 3.3 CLI Integration

```rust
/// CLI command for document validation
#[derive(Parser, Debug)]
pub struct ValidateDocsCmd {
    /// Root directory to validate (defaults to current directory)
    #[arg(short, long)]
    root: Option<PathBuf>,

    /// Configuration file path
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Fail on broken links
    #[arg(short, long, default_value = "true")]
    fail_on_error: bool,

    /// Output format (text, json, junit)
    #[arg(short, long, default_value = "text")]
    output: OutputFormat,

    /// Maximum concurrent HTTP requests
    #[arg(long, default_value = "10")]
    max_concurrent: usize,

    /// HTTP request timeout in seconds
    #[arg(long, default_value = "30")]
    timeout: u64,
}

impl ValidateDocsCmd {
    pub async fun execute(&self) -> Result<ExitCode> {
        let config = if let Some(config_path) = &self.config {
            ValidatorConfig::from_file(config_path)?
        } else {
            ValidatorConfig {
                root_dir: self.root.clone().unwrap_or_else(|| PathBuf::from(".")),
                http_timeout_ms: self.timeout * 1000,
                max_retries: 3,
                retry_delay_ms: 1000,
                max_concurrent_requests: self.max_concurrent,
                exclude_patterns: vec![],
                follow_redirects: true,
                user_agent: format!("pmat-doc-validator/{}", env!("CARGO_PKG_VERSION")),
            }
        };

        let validator = DocValidator::new(config);
        let summary = validator.validate_directory(&validator.config.root_dir).await?;

        // Output results
        match self.output {
            OutputFormat::Text => print_text_summary(&summary),
            OutputFormat::Json => print_json_summary(&summary)?,
            OutputFormat::Junit => print_junit_summary(&summary)?,
        }

        // Exit with error code if broken links found
        if self.fail_on_error && summary.broken_links > 0 {
            Ok(ExitCode::FAILURE)
        } else {
            Ok(ExitCode::SUCCESS)
        }
    }
}
```

## 4. Test-Driven Development Plan

### 4.1 Property Tests

```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: All valid markdown link syntax should be extracted
        #[test]
        fun test_link_extraction_completeness(
            text in "[a-zA-Z0-9 ]+",
            url in "https?://[a-zA-Z0-9.-]+\\.[a-z]{2,}"
        ) {
            let markdown = format!("[{}]({})", text, url);
            let links = extract_links(&markdown, Path::new("test.md"));

            prop_assert_eq!(links.len(), 1);
            prop_assert_eq!(links[0].text, text);
            prop_assert_eq!(links[0].target, url);
        }

        /// Property: Link classification should be deterministic
        #[test]
        fun test_link_classification_determinism(target in ".*") {
            let type1 = classify_link(&target);
            let type2 = classify_link(&target);
            prop_assert_eq!(type1, type2);
        }

        /// Property: HTTP links should always get ExternalHttp classification
        #[test]
        fun test_http_link_classification(
            domain in "[a-z0-9-]+",
            tld in "[a-z]{2,4}"
        ) {
            let url = format!("https://{}.{}", domain, tld);
            let link_type = classify_link(&url);
            prop_assert_eq!(link_type, LinkType::ExternalHttp);
        }

        /// Property: Internal links should resolve correctly relative to source
        #[test]
        fun test_internal_link_resolution(
            filename in "[a-z0-9_-]+\\.md"
        ) {
            let source = PathBuf::from("docs/spec.md");
            let target = format!("./{}", filename);

            // This should resolve to docs/{filename}
            let link = Link {
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
        fun test_validation_status_completeness(
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
        fun test_exponential_backoff(base_delay in 100u64..1000u64, retry in 0u32..5u32) {
            let delay = base_delay * 2_u64.pow(retry);
            if retry > 0 {
                let prev_delay = base_delay * 2_u64.pow(retry - 1);
                prop_assert!(delay >= prev_delay * 2);
            }
        }
    }
}
```

### 4.2 Unit Tests (TDD Style)

```rust
#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fun red_test_extract_links_from_empty_content() {
        // RED: This should pass but implementation doesn't exist yet
        let links = extract_links("", Path::new("test.md"));
        assert_eq!(links.len(), 0);
    }

    #[test]
    fun red_test_extract_single_http_link() {
        // RED: Implementation missing
        let content = "[Example](https://example.com)";
        let links = extract_links(content, Path::new("test.md"));

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].text, "Example");
        assert_eq!(links[0].target, "https://example.com");
        assert_eq!(links[0].link_type, LinkType::ExternalHttp);
    }

    #[test]
    fun red_test_extract_multiple_links() {
        // RED: Implementation missing
        let content = "[Link1](url1.md) and [Link2](url2.md)";
        let links = extract_links(content, Path::new("test.md"));

        assert_eq!(links.len(), 2);
    }

    #[test]
    fun red_test_classify_http_link() {
        // RED: Implementation missing
        assert_eq!(classify_link("http://example.com"), LinkType::ExternalHttp);
        assert_eq!(classify_link("https://example.com"), LinkType::ExternalHttp);
    }

    #[test]
    fun red_test_classify_internal_link() {
        // RED: Implementation missing
        assert_eq!(classify_link("./file.md"), LinkType::Internal);
        assert_eq!(classify_link("../parent.md"), LinkType::Internal);
        assert_eq!(classify_link("/abs/path.md"), LinkType::Internal);
    }

    #[test]
    fun red_test_classify_anchor_link() {
        // RED: Implementation missing
        assert_eq!(classify_link("#section"), LinkType::Anchor);
    }

    #[test]
    fun red_test_classify_email_link() {
        // RED: Implementation missing
        assert_eq!(classify_link("mailto:user@example.com"), LinkType::Email);
    }

    #[tokio::test]
    async fun red_test_validate_existing_internal_link() {
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
    async fun red_test_validate_missing_internal_link() {
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
    async fun red_test_validate_http_404() {
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
    async fun red_test_validate_http_200() {
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
    async fun red_test_http_retry_logic() {
        // RED: Implementation missing
        // Should retry on network errors up to max_retries times
        let config = ValidatorConfig {
            max_retries: 3,
            retry_delay_ms: 10,
            ..Default::default()
        };

        let validator = DocValidator::new(config);

        // Use unreachable IP to trigger network error
        let link = Link {
            text: "Unreachable".to_string(),
            target: "http://192.0.2.1".to_string(),
            source_file: PathBuf::from("test.md"),
            line_number: 1,
            link_type: LinkType::ExternalHttp,
        };

        let result = validator.validate_link(&link).await.unwrap();
        assert_eq!(result.status, ValidationStatus::NetworkError);
    }

    #[tokio::test]
    async fun red_test_concurrent_validation() {
        // RED: Implementation missing
        let temp_dir = tempfile::tempdir().unwrap();

        // Create multiple test files
        for i in 0..10 {
            let file = temp_dir.path().join(format!("file{}.md", i));
            std::fs::write(&file, format!("[link](./file{}.md)", (i + 1) % 10)).unwrap();
        }

        let validator = DocValidator::default();
        let summary = validator.validate_directory(temp_dir.path()).await.unwrap();

        assert_eq!(summary.total_files, 10);
        assert_eq!(summary.valid_links, 10);
    }
}
```

### 4.3 Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fun test_validate_real_docs_directory() {
        // Integration test using actual docs directory
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs");

        let validator = DocValidator::default();
        let summary = validator.validate_directory(&root).await.unwrap();

        println!("Validated {} files", summary.total_files);
        println!("Found {} links", summary.total_links);
        println!("Valid: {}, Broken: {}", summary.valid_links, summary.broken_links);

        // Report any broken links
        for result in &summary.results {
            if result.status == ValidationStatus::NotFound {
                eprintln!(
                    "Broken link in {}:{} -> {}",
                    result.link.source_file.display(),
                    result.link.line_number,
                    result.link.target
                );
            }
        }
    }

    #[test]
    fun test_cli_validate_docs_command() {
        // Integration test for CLI
        let output = std::process::Command::new("cargo")
            .args(&["run", "--", "validate-docs", "--root", "docs"])
            .output()
            .expect("Failed to execute command");

        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
}
```

## 5. Quality Gates

### 5.1 Automated Checks

All code must pass:
- ✅ `cargo test` - All unit, property, and integration tests
- ✅ `cargo test --doc` - All doctests
- ✅ `cargo clippy -- -D warnings` - No clippy warnings
- ✅ `cargo fmt -- --check` - Proper formatting
- ✅ `pmat quality-gate` - Quality gate checks
- ✅ `cargo llvm-cov --lcov --output-path lcov.info` - Coverage report
- ✅ Coverage ≥ 80% for new code

### 5.2 Manual Review Checklist

- [ ] Code follows Rust best practices
- [ ] Error handling is comprehensive
- [ ] Documentation is complete and accurate
- [ ] Examples are runnable and correct
- [ ] Performance is acceptable (benchmark results)
- [ ] Security considerations addressed
- [ ] Edge cases handled

## 6. Performance Requirements

- **Throughput**: Validate 1000+ links per minute
- **Concurrency**: Support 10+ concurrent HTTP requests
- **Memory**: Maximum 100MB heap usage for 10,000 links
- **Latency**: HTTP timeout configurable (default 30s)
- **Reliability**: Exponential backoff retry for transient failures

## 7. Configuration

### 7.1 Configuration File Format

```toml
# .pmat/doc-validator.toml

[validator]
# Root directory to validate
root_dir = "."

# HTTP request timeout (milliseconds)
http_timeout_ms = 30000

# Maximum retries for failed requests
max_retries = 3

# Retry delay (milliseconds)
retry_delay_ms = 1000

# Maximum concurrent HTTP requests
max_concurrent_requests = 10

# Follow HTTP redirects
follow_redirects = true

# User agent string
user_agent = "pmat-doc-validator/1.0"

# Patterns to exclude (glob patterns)
exclude_patterns = [
    "**/node_modules/**",
    "**/target/**",
    "**/.git/**",
    "**/archive/**"
]

# Domains to skip validation (e.g., localhost, private networks)
skip_domains = [
    "localhost",
    "127.0.0.1",
    "*.internal",
]
```

## 8. Dependencies

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
regex = "1.10"
walkdir = "2.4"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
thiserror = "1.0"
clap = { version = "4.5", features = ["derive"] }

[dev-dependencies]
proptest = "1.4"
tempfile = "3.8"
mockito = "1.2"
criterion = "0.5"
```

## 9. Implementation Roadmap

See: `docs/execution/doc-validate-roadmap.md`

### Phase 1: Core Link Extraction (Week 1)
- Implement link parsing with regex
- Add link classification logic
- Write property tests for extraction
- Write unit tests for classification

### Phase 2: Internal Link Validation (Week 1)
- Implement file existence checking
- Handle relative path resolution
- Add anchor validation
- Write tests for internal links

### Phase 3: HTTP Link Validation (Week 2)
- Implement HTTP client with retry logic
- Add concurrent request handling
- Implement exponential backoff
- Write tests for HTTP validation

### Phase 4: CLI Integration (Week 2)
- Add CLI command and arguments
- Implement output formatters (text, JSON, JUnit)
- Add configuration file support
- Write integration tests

### Phase 5: Quality & Performance (Week 3)
- Run all quality gates
- Optimize performance with benchmarks
- Achieve 80%+ test coverage
- Complete documentation

### Phase 6: Release (Week 3)
- Version bump
- Update changelog
- Publish to crates.io
- Create GitHub release

## 10. Success Criteria

- [ ] All property tests passing
- [ ] All unit tests passing
- [ ] All integration tests passing
- [ ] All doctests passing
- [ ] Test coverage ≥ 80%
- [ ] `pmat quality-gate` passing
- [ ] CLI command functional
- [ ] Validates both internal and external links
- [ ] Fails build on 404 errors
- [ ] Performance benchmarks met
- [ ] Documentation complete
- [ ] Published to crates.io
- [ ] GitHub release created

## 11. References

- [RFC 3986 - URI Generic Syntax](https://www.rfc-editor.org/rfc/rfc3986)
- [CommonMark Spec](https://spec.commonmark.org/)
- [HTTP Status Codes](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status)
- [Tokio Documentation](https://tokio.rs/)
- [Reqwest Documentation](https://docs.rs/reqwest/)
- [Property-Based Testing in Rust](https://proptest-rs.github.io/proptest/)

---

**Next Steps**:
1. Create detailed roadmap in `docs/execution/doc-validate-roadmap.md`
2. Create GitHub issues for each phase
3. Begin TDD implementation starting with RED tests
