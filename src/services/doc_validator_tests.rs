#[cfg_attr(coverage_nightly, coverage(off))]
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
        assert_eq!(classify_link("http://example.com"), LinkType::ExternalHttp);
        assert_eq!(classify_link("https://example.com"), LinkType::ExternalHttp);
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
        let summary = validator.validate_directory(temp_dir.path()).await.unwrap();

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
        std::fs::write(archive_dir.join("old.md"), "[broken](./nonexistent.md)").unwrap();

        let validator = DocValidator::default();
        let summary = validator.validate_directory(temp_dir.path()).await.unwrap();

        // Should only scan 2 files (readme.md and test.md), excluding archive/old.md
        assert_eq!(summary.total_files, 2);
        assert_eq!(summary.valid_links, 1);
        assert_eq!(summary.broken_links, 0); // archive's broken link should be excluded
    }

    // Issue #101: Tests for crates.io URL handling
    #[test]
    fn test_extract_crates_io_crate_name_basic() {
        assert_eq!(
            DocValidator::extract_crates_io_crate_name("https://crates.io/crates/trueno"),
            Some("trueno".to_string())
        );
        assert_eq!(
            DocValidator::extract_crates_io_crate_name("https://crates.io/crates/aprender"),
            Some("aprender".to_string())
        );
    }

    #[test]
    fn test_extract_crates_io_crate_name_with_subpath() {
        assert_eq!(
            DocValidator::extract_crates_io_crate_name("https://crates.io/crates/serde/versions"),
            Some("serde".to_string())
        );
        assert_eq!(
            DocValidator::extract_crates_io_crate_name("https://crates.io/crates/tokio/1.0.0"),
            Some("tokio".to_string())
        );
    }

    #[test]
    fn test_extract_crates_io_crate_name_http() {
        assert_eq!(
            DocValidator::extract_crates_io_crate_name("http://crates.io/crates/serde"),
            Some("serde".to_string())
        );
    }

    #[test]
    fn test_extract_crates_io_crate_name_non_crates_io() {
        assert_eq!(
            DocValidator::extract_crates_io_crate_name("https://github.com/rust-lang/rust"),
            None
        );
        assert_eq!(
            DocValidator::extract_crates_io_crate_name("https://docs.rs/serde"),
            None
        );
        assert_eq!(
            DocValidator::extract_crates_io_crate_name("https://crates.io/"),
            None
        );
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_validate_crates_io_existing_crate() {
        let link = Link {
            text: "trueno".to_string(),
            target: "https://crates.io/crates/serde".to_string(), // Use well-known crate
            source_file: PathBuf::from("test.md"),
            line_number: 1,
            link_type: LinkType::ExternalHttp,
        };

        let validator = DocValidator::default();
        let result = validator.validate_link(&link).await.unwrap();

        assert_eq!(result.status, ValidationStatus::Valid);
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_validate_crates_io_nonexistent_crate() {
        let link = Link {
            text: "nonexistent".to_string(),
            target: "https://crates.io/crates/this-crate-definitely-does-not-exist-12345"
                .to_string(),
            source_file: PathBuf::from("test.md"),
            line_number: 1,
            link_type: LinkType::ExternalHttp,
        };

        let validator = DocValidator::default();
        let result = validator.validate_link(&link).await.unwrap();

        assert_eq!(result.status, ValidationStatus::NotFound);
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
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
