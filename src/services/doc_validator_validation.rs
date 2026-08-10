impl DocValidator {
    /// Creates a new validator with default configuration
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(config: ValidatorConfig) -> Self {
        #[cfg(feature = "http-client")]
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
            #[cfg(feature = "http-client")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
        let target = link
            .target
            .split('#')
            .next()
            .expect("split should have at least one element");

        // Skip empty targets (pure anchors)
        if target.is_empty() {
            return (ValidationStatus::Valid, None, None);
        }

        // Resolve relative path
        let base_dir = link.source_file.parent().unwrap_or_else(|| Path::new("."));
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
    #[cfg(feature = "http-client")]
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

        // Issue #101: Handle crates.io URLs specially - they block programmatic access
        // Use the crates.io API instead of scraping the web page
        if let Some(crate_name) = Self::extract_crates_io_crate_name(&link.target) {
            return self.validate_crates_io_crate(client, &crate_name).await;
        }

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

    /// Extract crate name from crates.io URL (Issue #101)
    /// Handles: https://crates.io/crates/{crate_name}
    #[cfg(feature = "http-client")]
    fn extract_crates_io_crate_name(url: &str) -> Option<String> {
        // Match patterns like:
        // - https://crates.io/crates/trueno
        // - http://crates.io/crates/trueno
        // - https://crates.io/crates/trueno/versions
        let patterns = ["https://crates.io/crates/", "http://crates.io/crates/"];

        for pattern in patterns {
            if let Some(rest) = url.strip_prefix(pattern) {
                // Get the crate name (up to the next / or end)
                let crate_name = rest.split('/').next()?;
                if !crate_name.is_empty() {
                    return Some(crate_name.to_string());
                }
            }
        }
        None
    }

    /// Fallback when http-client feature is disabled
    #[cfg(not(feature = "http-client"))]
    async fn validate_http_link(
        &self,
        _link: &Link,
    ) -> (ValidationStatus, Option<String>, Option<u16>) {
        (
            ValidationStatus::Skipped,
            Some("HTTP validation requires http-client feature".to_string()),
            None,
        )
    }

    /// Validate a crate exists using the crates.io API (Issue #101)
    #[cfg(feature = "http-client")]
    async fn validate_crates_io_crate(
        &self,
        client: &reqwest::Client,
        crate_name: &str,
    ) -> (ValidationStatus, Option<String>, Option<u16>) {
        // Use the crates.io API which accepts programmatic access
        let api_url = format!("https://crates.io/api/v1/crates/{}", crate_name);

        match client
            .get(&api_url)
            .header("User-Agent", "pmat-doc-validator/1.0")
            .send()
            .await
        {
            Ok(response) => {
                let status_code = response.status().as_u16();
                if status_code == 200 {
                    (ValidationStatus::Valid, None, Some(status_code))
                } else if status_code == 404 {
                    (
                        ValidationStatus::NotFound,
                        Some(format!("Crate not found on crates.io: {}", crate_name)),
                        Some(status_code),
                    )
                } else {
                    (
                        ValidationStatus::HttpError(status_code),
                        Some(format!(
                            "crates.io API error {}: {}",
                            status_code, crate_name
                        )),
                        Some(status_code),
                    )
                }
            }
            Err(e) => (
                ValidationStatus::NetworkError,
                Some(format!("crates.io API error: {}", e)),
                None,
            ),
        }
    }

    /// Validates an anchor link
    async fn validate_anchor_link(
        &self,
        _link: &Link,
    ) -> (ValidationStatus, Option<String>, Option<u16>) {
        // Anchor validation not yet implemented; assumes valid
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn validate_directory(&self, root: &Path) -> Result<ValidationSummary> {
        let start = Instant::now();
        let mut all_links = Vec::new();
        let mut file_count = 0;

        // Find all markdown files, skipping excluded directories
        for entry in WalkDir::new(root)
            .into_iter()
            .filter_entry(|e| !self.should_exclude(e.path()))
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file() && e.path().extension().is_some_and(|ext| ext == "md"))
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
        let (valid_count, broken_count, skipped_count) = bucket_statuses(&results);

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

/// Split results into (valid, broken, skipped).
///
/// This used to be three inline filters, and the broken filter listed only
/// `NotFound | HttpError(_)`. `NetworkError` and `InvalidLink` therefore landed
/// in no bucket: running `validate-docs` with the network unreachable reported
/// "Links found: 4 / Valid links: 0 / Broken links: 0 / Skipped links: 0",
/// printed "All documentation links are valid!" and exited 0 even under
/// `--fail-on-error`. A link that could not be reached is not a link that was
/// validated, so it is broken. The `match` is exhaustive on purpose — adding a
/// status now forces a bucket decision at compile time instead of silently
/// dropping it from the totals.
pub(crate) fn bucket_statuses(results: &[ValidationResult]) -> (usize, usize, usize) {
    let mut valid = 0;
    let mut broken = 0;
    let mut skipped = 0;

    for result in results {
        match result.status {
            ValidationStatus::Valid => valid += 1,
            ValidationStatus::NotFound
            | ValidationStatus::HttpError(_)
            | ValidationStatus::NetworkError
            | ValidationStatus::InvalidLink => broken += 1,
            ValidationStatus::Skipped => skipped += 1,
        }
    }

    (valid, broken, skipped)
}

#[cfg(test)]
mod bucket_status_tests {
    use super::*;

    fn result_with(status: ValidationStatus) -> ValidationResult {
        ValidationResult {
            link: Link {
                text: "a".to_string(),
                target: "https://example.com".to_string(),
                source_file: PathBuf::from("README.md"),
                line_number: 1,
                link_type: LinkType::ExternalHttp,
            },
            status,
            error_message: None,
            http_status_code: None,
            response_time_ms: None,
        }
    }

    #[test]
    fn test_network_error_counts_as_broken() {
        let results = vec![result_with(ValidationStatus::NetworkError)];
        let (valid, broken, skipped) = bucket_statuses(&results);
        assert_eq!(
            (valid, broken, skipped),
            (0, 1, 0),
            "an unreachable link is not a validated link"
        );
    }

    #[test]
    fn test_invalid_link_counts_as_broken() {
        let results = vec![result_with(ValidationStatus::InvalidLink)];
        assert_eq!(bucket_statuses(&results), (0, 1, 0));
    }

    #[test]
    fn test_every_status_lands_in_exactly_one_bucket() {
        let results = vec![
            result_with(ValidationStatus::Valid),
            result_with(ValidationStatus::NotFound),
            result_with(ValidationStatus::HttpError(500)),
            result_with(ValidationStatus::NetworkError),
            result_with(ValidationStatus::InvalidLink),
            result_with(ValidationStatus::Skipped),
        ];
        let (valid, broken, skipped) = bucket_statuses(&results);
        assert_eq!(
            valid + broken + skipped,
            results.len(),
            "valid + broken + skipped must equal the total; a status in no bucket \
             is what let an offline run report zero broken links"
        );
        assert_eq!((valid, broken, skipped), (1, 4, 1));
    }
}
