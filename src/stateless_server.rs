use anyhow::Result;
use lru::LruCache;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::models::template::TemplateResource;
use crate::services::renderer::TemplateRenderer;
use crate::{S3Client, TemplateServerTrait};

pub struct StatelessTemplateServer {
    pub renderer: TemplateRenderer,
}

impl StatelessTemplateServer {
    pub fn new() -> Result<Self> {
        Ok(Self {
            renderer: TemplateRenderer::new()?,
        })
    }

    pub async fn get_template_metadata(&self, uri: &str) -> Result<Arc<TemplateResource>> {
        // Parse URI and fetch from embedded templates
        let parts: Vec<&str> = uri
            .strip_prefix("template://")
            .ok_or_else(|| anyhow::anyhow!("Invalid URI: {uri}"))?
            .split('/')
            .collect();

        if parts.len() != 3 {
            return Err(anyhow::anyhow!("Invalid URI format: {uri}"));
        }

        // Fetch from embedded templates
        crate::services::embedded_templates::get_template_metadata(uri)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get template metadata: {e}"))
    }

    pub async fn get_template_content(&self, uri: &str) -> Result<Arc<str>> {
        // Fetch from embedded templates
        crate::services::embedded_templates::get_template_content(uri)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get template content: {e}"))
    }

    pub async fn list_templates(&self, prefix: &str) -> Result<Vec<Arc<TemplateResource>>> {
        crate::services::embedded_templates::list_templates(prefix)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list templates: {e}"))
    }
}

#[async_trait::async_trait]
impl TemplateServerTrait for StatelessTemplateServer {
    async fn get_template_metadata(&self, uri: &str) -> Result<Arc<TemplateResource>> {
        self.get_template_metadata(uri).await
    }

    async fn get_template_content(&self, s3_key: &str) -> Result<Arc<str>> {
        self.get_template_content(s3_key).await
    }

    async fn list_templates(&self, prefix: &str) -> Result<Vec<Arc<TemplateResource>>> {
        self.list_templates(prefix).await
    }

    fn get_renderer(&self) -> &TemplateRenderer {
        &self.renderer
    }

    fn get_metadata_cache(&self) -> Option<&Arc<RwLock<LruCache<String, Arc<TemplateResource>>>>> {
        None
    }

    fn get_content_cache(&self) -> Option<&Arc<RwLock<LruCache<String, Arc<str>>>>> {
        None
    }

    fn get_s3_client(&self) -> Option<&S3Client> {
        None
    }

    fn get_bucket_name(&self) -> Option<&str> {
        None
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use crate::TemplateServerTrait;

    // ============================================================================
    // Test Fixtures and Helpers
    // ============================================================================

    /// Creates a new StatelessTemplateServer for testing.
    fn create_test_server() -> StatelessTemplateServer {
        StatelessTemplateServer::new().expect("Failed to create StatelessTemplateServer")
    }

    /// Valid template URIs for testing
    fn valid_template_uris() -> Vec<&'static str> {
        vec![
            "template://makefile/rust/cli",
            "template://readme/rust/cli",
            "template://gitignore/rust/cli",
            "template://makefile/python-uv/cli",
            "template://makefile/deno/cli",
            "template://readme/deno/cli",
            "template://readme/python-uv/cli",
            "template://gitignore/deno/cli",
            "template://gitignore/python-uv/cli",
        ]
    }

    // ============================================================================
    // StatelessTemplateServer::new() Tests
    // ============================================================================

    #[test]
    fn test_new_creates_server_successfully() {
        let result = StatelessTemplateServer::new();
        assert!(
            result.is_ok(),
            "StatelessTemplateServer::new() should succeed"
        );
    }

    #[test]
    fn test_new_initializes_renderer() {
        let server = create_test_server();
        // Verify renderer is accessible
        let _renderer = &server.renderer;
        // If we get here without panic, renderer is properly initialized
    }

    #[test]
    fn test_multiple_servers_can_be_created() {
        let server1 = StatelessTemplateServer::new();
        let server2 = StatelessTemplateServer::new();
        let server3 = StatelessTemplateServer::new();

        assert!(server1.is_ok());
        assert!(server2.is_ok());
        assert!(server3.is_ok());
    }

    // ============================================================================
    // get_template_metadata() Tests
    // ============================================================================

    #[tokio::test]
    async fn test_get_template_metadata_valid_rust_makefile() {
        let server = create_test_server();
        let result = server
            .get_template_metadata("template://makefile/rust/cli")
            .await;

        assert!(result.is_ok(), "Should fetch rust makefile metadata");
        let metadata = result.unwrap();
        assert!(metadata.uri.contains("makefile/rust/cli"));
        assert!(!metadata.name.is_empty());
        assert!(!metadata.description.is_empty());
    }

    #[tokio::test]
    async fn test_get_template_metadata_valid_readme() {
        let server = create_test_server();
        let result = server
            .get_template_metadata("template://readme/rust/cli")
            .await;

        assert!(result.is_ok(), "Should fetch rust readme metadata");
        let metadata = result.unwrap();
        assert!(metadata.uri.contains("readme/rust/cli"));
    }

    #[tokio::test]
    async fn test_get_template_metadata_valid_gitignore() {
        let server = create_test_server();
        let result = server
            .get_template_metadata("template://gitignore/rust/cli")
            .await;

        assert!(result.is_ok(), "Should fetch rust gitignore metadata");
        let metadata = result.unwrap();
        assert!(metadata.uri.contains("gitignore/rust/cli"));
    }

    #[tokio::test]
    async fn test_get_template_metadata_valid_python_uv() {
        let server = create_test_server();
        let result = server
            .get_template_metadata("template://makefile/python-uv/cli")
            .await;

        assert!(result.is_ok(), "Should fetch python-uv makefile metadata");
    }

    #[tokio::test]
    async fn test_get_template_metadata_valid_deno() {
        let server = create_test_server();
        let result = server
            .get_template_metadata("template://makefile/deno/cli")
            .await;

        assert!(result.is_ok(), "Should fetch deno makefile metadata");
    }

    #[tokio::test]
    async fn test_get_template_metadata_invalid_uri_no_prefix() {
        let server = create_test_server();
        let result = server.get_template_metadata("makefile/rust/cli").await;

        assert!(result.is_err(), "Should fail without template:// prefix");
        let err = result.err().unwrap();
        assert!(err.to_string().contains("Invalid URI"));
    }

    #[tokio::test]
    async fn test_get_template_metadata_invalid_uri_wrong_format() {
        let server = create_test_server();
        let result = server.get_template_metadata("template://only/two").await;

        assert!(result.is_err(), "Should fail with only 2 parts");
        let err = result.err().unwrap();
        assert!(err.to_string().contains("Invalid URI format"));
    }

    #[tokio::test]
    async fn test_get_template_metadata_invalid_uri_single_part() {
        let server = create_test_server();
        let result = server.get_template_metadata("template://single").await;

        assert!(result.is_err(), "Should fail with single part");
    }

    #[tokio::test]
    async fn test_get_template_metadata_nonexistent_template() {
        let server = create_test_server();
        let result = server
            .get_template_metadata("template://nonexistent/template/type")
            .await;

        assert!(result.is_err(), "Should fail for nonexistent template");
    }

    #[tokio::test]
    async fn test_get_template_metadata_empty_uri() {
        let server = create_test_server();
        let result = server.get_template_metadata("").await;

        assert!(result.is_err(), "Should fail with empty URI");
    }

    #[tokio::test]
    async fn test_get_template_metadata_all_valid_templates() {
        let server = create_test_server();

        for uri in valid_template_uris() {
            let result = server.get_template_metadata(uri).await;
            assert!(result.is_ok(), "Should fetch metadata for: {}", uri);
        }
    }

    // ============================================================================
    // get_template_content() Tests
    // ============================================================================

    #[tokio::test]
    async fn test_get_template_content_valid_rust_makefile() {
        let server = create_test_server();
        let result = server
            .get_template_content("template://makefile/rust/cli")
            .await;

        assert!(result.is_ok(), "Should fetch rust makefile content");
        let content = result.unwrap();
        assert!(!content.is_empty(), "Content should not be empty");
        // Rust makefiles typically have cargo commands
        assert!(
            content.contains("cargo") || content.contains("{{"),
            "Content should have expected patterns"
        );
    }

    #[tokio::test]
    async fn test_get_template_content_valid_readme() {
        let server = create_test_server();
        let result = server
            .get_template_content("template://readme/rust/cli")
            .await;

        assert!(result.is_ok(), "Should fetch rust readme content");
        let content = result.unwrap();
        assert!(!content.is_empty());
    }

    #[tokio::test]
    async fn test_get_template_content_valid_gitignore() {
        let server = create_test_server();
        let result = server
            .get_template_content("template://gitignore/rust/cli")
            .await;

        assert!(result.is_ok(), "Should fetch rust gitignore content");
        let content = result.unwrap();
        assert!(!content.is_empty());
    }

    #[tokio::test]
    async fn test_get_template_content_nonexistent_template() {
        let server = create_test_server();
        let result = server
            .get_template_content("template://nonexistent/template/type")
            .await;

        assert!(result.is_err(), "Should fail for nonexistent template");
    }

    #[tokio::test]
    async fn test_get_template_content_all_valid_templates() {
        let server = create_test_server();

        for uri in valid_template_uris() {
            let result = server.get_template_content(uri).await;
            assert!(result.is_ok(), "Should fetch content for: {}", uri);
            assert!(
                !result.unwrap().is_empty(),
                "Content should not be empty for: {}",
                uri
            );
        }
    }

    #[tokio::test]
    async fn test_get_template_content_is_handlebars_template() {
        let server = create_test_server();
        let result = server
            .get_template_content("template://makefile/rust/cli")
            .await;

        assert!(result.is_ok());
        let content = result.unwrap();
        // Handlebars templates often contain {{ }} syntax
        assert!(
            content.contains("{{") || content.len() > 10,
            "Content should be a valid template"
        );
    }

    // ============================================================================
    // list_templates() Tests
    // ============================================================================

    #[tokio::test]
    async fn test_list_templates_empty_prefix() {
        let server = create_test_server();
        let result = server.list_templates("").await;

        assert!(
            result.is_ok(),
            "Should list all templates with empty prefix"
        );
        let templates = result.unwrap();
        assert!(!templates.is_empty(), "Should have at least one template");
        // We know there are at least 9 embedded templates
        assert!(templates.len() >= 9, "Should have at least 9 templates");
    }

    #[tokio::test]
    async fn test_list_templates_makefile_prefix() {
        let server = create_test_server();
        let result = server.list_templates("makefile").await;

        assert!(result.is_ok());
        let templates = result.unwrap();
        assert!(!templates.is_empty());

        // All returned templates should contain "makefile" in URI
        for template in &templates {
            assert!(
                template.uri.contains("makefile"),
                "All templates should match prefix: {}",
                template.uri
            );
        }
    }

    #[tokio::test]
    async fn test_list_templates_readme_prefix() {
        let server = create_test_server();
        let result = server.list_templates("readme").await;

        assert!(result.is_ok());
        let templates = result.unwrap();
        assert!(!templates.is_empty());

        for template in &templates {
            assert!(
                template.uri.contains("readme"),
                "All templates should match readme prefix: {}",
                template.uri
            );
        }
    }

    #[tokio::test]
    async fn test_list_templates_gitignore_prefix() {
        let server = create_test_server();
        let result = server.list_templates("gitignore").await;

        assert!(result.is_ok());
        let templates = result.unwrap();
        assert!(!templates.is_empty());

        for template in &templates {
            assert!(
                template.uri.contains("gitignore"),
                "All templates should match gitignore prefix: {}",
                template.uri
            );
        }
    }

    #[tokio::test]
    async fn test_list_templates_rust_prefix() {
        let server = create_test_server();
        let result = server.list_templates("rust").await;

        assert!(result.is_ok());
        let templates = result.unwrap();
        assert!(!templates.is_empty());

        for template in &templates {
            assert!(
                template.uri.contains("rust"),
                "All templates should match rust prefix: {}",
                template.uri
            );
        }
    }

    #[tokio::test]
    async fn test_list_templates_python_uv_prefix() {
        let server = create_test_server();
        let result = server.list_templates("python-uv").await;

        assert!(result.is_ok());
        let templates = result.unwrap();
        assert!(!templates.is_empty());

        for template in &templates {
            assert!(
                template.uri.contains("python-uv"),
                "All templates should match python-uv prefix: {}",
                template.uri
            );
        }
    }

    #[tokio::test]
    async fn test_list_templates_deno_prefix() {
        let server = create_test_server();
        let result = server.list_templates("deno").await;

        assert!(result.is_ok());
        let templates = result.unwrap();
        assert!(!templates.is_empty());

        for template in &templates {
            assert!(
                template.uri.contains("deno"),
                "All templates should match deno prefix: {}",
                template.uri
            );
        }
    }

    #[tokio::test]
    async fn test_list_templates_nonexistent_prefix() {
        let server = create_test_server();
        let result = server.list_templates("nonexistent_prefix_xyz").await;

        assert!(result.is_ok());
        let templates = result.unwrap();
        assert!(
            templates.is_empty(),
            "Should return empty list for nonexistent prefix"
        );
    }

    #[tokio::test]
    async fn test_list_templates_partial_match() {
        let server = create_test_server();
        // "make" should match "makefile"
        let result = server.list_templates("make").await;

        assert!(result.is_ok());
        let templates = result.unwrap();
        // At least the makefile templates should match
        assert!(!templates.is_empty());
    }

    // ============================================================================
    // TemplateServerTrait Implementation Tests
    // ============================================================================

    #[tokio::test]
    async fn test_trait_get_template_metadata() {
        let server = create_test_server();
        let trait_server: &dyn TemplateServerTrait = &server;

        let result = trait_server
            .get_template_metadata("template://makefile/rust/cli")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_trait_get_template_content() {
        let server = create_test_server();
        let trait_server: &dyn TemplateServerTrait = &server;

        let result = trait_server
            .get_template_content("template://makefile/rust/cli")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_trait_list_templates() {
        let server = create_test_server();
        let trait_server: &dyn TemplateServerTrait = &server;

        let result = trait_server.list_templates("").await;
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_trait_get_renderer() {
        let server = create_test_server();
        let trait_server: &dyn TemplateServerTrait = &server;

        let renderer = trait_server.get_renderer();
        // Just verify we can access the renderer
        let _ = renderer;
    }

    #[test]
    fn test_trait_get_metadata_cache_returns_none() {
        let server = create_test_server();
        let trait_server: &dyn TemplateServerTrait = &server;

        let cache = trait_server.get_metadata_cache();
        assert!(
            cache.is_none(),
            "Stateless server should not have metadata cache"
        );
    }

    #[test]
    fn test_trait_get_content_cache_returns_none() {
        let server = create_test_server();
        let trait_server: &dyn TemplateServerTrait = &server;

        let cache = trait_server.get_content_cache();
        assert!(
            cache.is_none(),
            "Stateless server should not have content cache"
        );
    }

    #[test]
    fn test_trait_get_s3_client_returns_none() {
        let server = create_test_server();
        let trait_server: &dyn TemplateServerTrait = &server;

        let client = trait_server.get_s3_client();
        assert!(
            client.is_none(),
            "Stateless server should not have S3 client"
        );
    }

    #[test]
    fn test_trait_get_bucket_name_returns_none() {
        let server = create_test_server();
        let trait_server: &dyn TemplateServerTrait = &server;

        let bucket = trait_server.get_bucket_name();
        assert!(
            bucket.is_none(),
            "Stateless server should not have bucket name"
        );
    }

    // ============================================================================
    // Concurrent Access Tests
    // ============================================================================

    #[tokio::test]
    async fn test_concurrent_metadata_access() {
        let server = Arc::new(create_test_server());

        let handles: Vec<_> = valid_template_uris()
            .into_iter()
            .map(|uri| {
                let server = Arc::clone(&server);
                let uri = uri.to_string();
                tokio::spawn(async move { server.get_template_metadata(&uri).await })
            })
            .collect();

        for handle in handles {
            let result = handle.await.expect("Task panicked");
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_concurrent_content_access() {
        let server = Arc::new(create_test_server());

        let handles: Vec<_> = valid_template_uris()
            .into_iter()
            .map(|uri| {
                let server = Arc::clone(&server);
                let uri = uri.to_string();
                tokio::spawn(async move { server.get_template_content(&uri).await })
            })
            .collect();

        for handle in handles {
            let result = handle.await.expect("Task panicked");
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_concurrent_list_access() {
        let server = Arc::new(create_test_server());
        let prefixes = vec!["", "makefile", "readme", "gitignore", "rust", "deno"];

        let handles: Vec<_> = prefixes
            .into_iter()
            .map(|prefix| {
                let server = Arc::clone(&server);
                let prefix = prefix.to_string();
                tokio::spawn(async move { server.list_templates(&prefix).await })
            })
            .collect();

        for handle in handles {
            let result = handle.await.expect("Task panicked");
            assert!(result.is_ok());
        }
    }

    // ============================================================================
    // Edge Case Tests
    // ============================================================================

    #[tokio::test]
    async fn test_uri_with_extra_slashes() {
        let server = create_test_server();
        // This should still fail as the format check happens before embedded template lookup
        let result = server
            .get_template_metadata("template://makefile//rust//cli")
            .await;
        // The format validation should see 5 parts instead of 3
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_uri_with_special_characters() {
        let server = create_test_server();
        let result = server
            .get_template_metadata("template://test%20name/type/variant")
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_uri_case_sensitivity() {
        let server = create_test_server();
        // URI matching is case-sensitive
        let result = server
            .get_template_metadata("template://MAKEFILE/RUST/CLI")
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_templates_case_sensitivity() {
        let server = create_test_server();
        let result = server.list_templates("MAKEFILE").await;

        assert!(result.is_ok());
        let templates = result.unwrap();
        // Prefix matching should be case-sensitive
        assert!(templates.is_empty());
    }

    #[tokio::test]
    async fn test_template_metadata_has_valid_version() {
        let server = create_test_server();
        let result = server
            .get_template_metadata("template://makefile/rust/cli")
            .await;

        assert!(result.is_ok());
        let metadata = result.unwrap();
        // Semantic version should be valid (1.0.0 for embedded templates)
        assert!(metadata.semantic_version.major >= 1);
    }

    #[tokio::test]
    async fn test_template_metadata_has_parameters() {
        let server = create_test_server();
        let result = server
            .get_template_metadata("template://makefile/rust/cli")
            .await;

        assert!(result.is_ok());
        let metadata = result.unwrap();
        // Templates typically have parameters
        // Not all templates have required parameters, but structure should be valid
        let _ = &metadata.parameters;
    }

    // ============================================================================
    // Template Content Verification Tests
    // ============================================================================

    #[tokio::test]
    async fn test_makefile_content_structure() {
        let server = create_test_server();

        for toolchain in &["rust", "python-uv", "deno"] {
            let uri = format!("template://makefile/{}/cli", toolchain);
            let result = server.get_template_content(&uri).await;
            assert!(result.is_ok(), "Should get makefile for {}", toolchain);

            let content = result.unwrap();
            // Makefiles should have targets or handlebars syntax
            assert!(
                content.contains(':') || content.contains("{{"),
                "Makefile should have target syntax or templates for {}",
                toolchain
            );
        }
    }

    #[tokio::test]
    async fn test_readme_content_structure() {
        let server = create_test_server();

        for toolchain in &["rust", "python-uv", "deno"] {
            let uri = format!("template://readme/{}/cli", toolchain);
            let result = server.get_template_content(&uri).await;
            assert!(result.is_ok(), "Should get readme for {}", toolchain);

            let content = result.unwrap();
            // READMEs should have markdown headers or handlebars syntax
            assert!(
                content.contains('#') || content.contains("{{"),
                "README should have markdown headers or templates for {}",
                toolchain
            );
        }
    }

    #[tokio::test]
    async fn test_gitignore_content_structure() {
        let server = create_test_server();

        for toolchain in &["rust", "deno", "python-uv"] {
            let uri = format!("template://gitignore/{}/cli", toolchain);
            let result = server.get_template_content(&uri).await;
            assert!(result.is_ok(), "Should get gitignore for {}", toolchain);

            let content = result.unwrap();
            // Gitignores should have patterns (usually start with . or * or /)
            assert!(
                content.contains('*')
                    || content.contains('.')
                    || content.contains('/')
                    || content.contains("{{"),
                "Gitignore should have ignore patterns for {}",
                toolchain
            );
        }
    }

    // ============================================================================
    // Consistency Tests
    // ============================================================================

    #[tokio::test]
    async fn test_metadata_content_consistency() {
        let server = create_test_server();

        for uri in valid_template_uris() {
            // Both metadata and content should be available for same URI
            let metadata_result = server.get_template_metadata(uri).await;
            let content_result = server.get_template_content(uri).await;

            assert!(
                metadata_result.is_ok(),
                "Metadata should be available for: {}",
                uri
            );
            assert!(
                content_result.is_ok(),
                "Content should be available for: {}",
                uri
            );
        }
    }

    #[tokio::test]
    async fn test_listed_templates_are_fetchable() {
        let server = create_test_server();
        let templates = server
            .list_templates("")
            .await
            .expect("Should list templates");

        for template in templates {
            let metadata_result = server.get_template_metadata(&template.uri).await;
            assert!(
                metadata_result.is_ok(),
                "Should fetch metadata for listed template: {}",
                template.uri
            );

            let content_result = server.get_template_content(&template.uri).await;
            assert!(
                content_result.is_ok(),
                "Should fetch content for listed template: {}",
                template.uri
            );
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // ============================================================================
    // Property-Based Test Strategies
    // ============================================================================

    /// Strategy for generating valid template prefixes
    fn valid_prefix_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("".to_string()),
            Just("makefile".to_string()),
            Just("readme".to_string()),
            Just("gitignore".to_string()),
            Just("rust".to_string()),
            Just("deno".to_string()),
            Just("python-uv".to_string()),
            Just("cli".to_string()),
        ]
    }

    /// Strategy for generating valid template categories
    fn valid_category_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("makefile".to_string()),
            Just("readme".to_string()),
            Just("gitignore".to_string()),
        ]
    }

    /// Strategy for generating valid toolchains
    fn valid_toolchain_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("rust".to_string()),
            Just("deno".to_string()),
            Just("python-uv".to_string()),
        ]
    }

    /// Strategy for generating valid variant
    fn valid_variant_strategy() -> impl Strategy<Value = String> {
        Just("cli".to_string())
    }

    /// Strategy for generating valid URIs
    fn valid_uri_strategy() -> impl Strategy<Value = String> {
        (
            valid_category_strategy(),
            valid_toolchain_strategy(),
            valid_variant_strategy(),
        )
            .prop_map(|(category, toolchain, variant)| {
                format!("template://{}/{}/{}", category, toolchain, variant)
            })
    }

    /// Strategy for generating invalid URIs
    fn invalid_uri_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            // Missing template:// prefix
            "[a-z]{3,10}/[a-z]{3,10}/[a-z]{3,10}".prop_map(|s| s),
            // Wrong prefix
            "http://[a-z]{3,10}/[a-z]{3,10}/[a-z]{3,10}".prop_map(|s| s),
            // Too few parts
            Just("template://only/two".to_string()),
            Just("template://single".to_string()),
            // Empty
            Just("".to_string()),
            // Extra slashes
            Just("template:///extra//slashes///".to_string()),
        ]
    }

    // ============================================================================
    // Property Tests
    // ============================================================================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]

        #[test]
        fn prop_server_creation_always_succeeds(_seed in 0u32..1000) {
            let result = StatelessTemplateServer::new();
            prop_assert!(result.is_ok());
        }

        #[test]
        fn prop_list_templates_never_panics(prefix in ".*") {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(async {
                let server = StatelessTemplateServer::new().unwrap();
                server.list_templates(&prefix).await
            });
            // Should always return Ok (possibly with empty vec)
            prop_assert!(result.is_ok());
        }

        #[test]
        fn prop_valid_prefixes_return_results(prefix in valid_prefix_strategy()) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let templates = rt.block_on(async {
                let server = StatelessTemplateServer::new().unwrap();
                server.list_templates(&prefix).await.unwrap()
            });

            // Valid prefixes that match templates should return non-empty lists
            // (except for very specific prefixes that might not match anything)
            if !prefix.is_empty() && ["makefile", "readme", "gitignore", "rust", "deno", "python-uv"].contains(&prefix.as_str()) {
                prop_assert!(!templates.is_empty(), "Prefix '{}' should match templates", prefix);
            }
        }

        #[test]
        fn prop_valid_uri_metadata_succeeds(uri in valid_uri_strategy()) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(async {
                let server = StatelessTemplateServer::new().unwrap();
                server.get_template_metadata(&uri).await
            });

            prop_assert!(result.is_ok(), "Valid URI should succeed: {}", uri);
        }

        #[test]
        fn prop_valid_uri_content_succeeds(uri in valid_uri_strategy()) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(async {
                let server = StatelessTemplateServer::new().unwrap();
                server.get_template_content(&uri).await
            });

            prop_assert!(result.is_ok(), "Valid URI should have content: {}", uri);
            prop_assert!(!result.unwrap().is_empty(), "Content should not be empty: {}", uri);
        }

        #[test]
        fn prop_invalid_uri_metadata_fails(uri in invalid_uri_strategy()) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(async {
                let server = StatelessTemplateServer::new().unwrap();
                server.get_template_metadata(&uri).await
            });

            prop_assert!(result.is_err(), "Invalid URI should fail: {}", uri);
        }

        #[test]
        fn prop_template_count_is_stable(prefix in valid_prefix_strategy()) {
            let rt = tokio::runtime::Runtime::new().unwrap();

            // Call list_templates twice and verify same count
            let (count1, count2) = rt.block_on(async {
                let server = StatelessTemplateServer::new().unwrap();
                let templates1 = server.list_templates(&prefix).await.unwrap();
                let templates2 = server.list_templates(&prefix).await.unwrap();
                (templates1.len(), templates2.len())
            });

            prop_assert_eq!(count1, count2, "Template count should be stable for prefix: {}", prefix);
        }

        #[test]
        fn prop_metadata_uri_matches_request(
            category in valid_category_strategy(),
            toolchain in valid_toolchain_strategy()
        ) {
            let uri = format!("template://{}/{}/cli", category, toolchain);
            let rt = tokio::runtime::Runtime::new().unwrap();

            let result = rt.block_on(async {
                let server = StatelessTemplateServer::new().unwrap();
                server.get_template_metadata(&uri).await
            });

            prop_assert!(result.is_ok());
            let metadata = result.unwrap();
            prop_assert!(
                metadata.uri.contains(&category),
                "Metadata URI should contain category: {} not in {}",
                category,
                metadata.uri
            );
            prop_assert!(
                metadata.uri.contains(&toolchain),
                "Metadata URI should contain toolchain: {} not in {}",
                toolchain,
                metadata.uri
            );
        }

        #[test]
        fn prop_trait_methods_return_none_for_cache_and_s3(_seed in 0u32..100) {
            let server = StatelessTemplateServer::new().unwrap();
            let trait_server: &dyn TemplateServerTrait = &server;

            prop_assert!(trait_server.get_metadata_cache().is_none());
            prop_assert!(trait_server.get_content_cache().is_none());
            prop_assert!(trait_server.get_s3_client().is_none());
            prop_assert!(trait_server.get_bucket_name().is_none());
        }

        #[test]
        fn prop_content_is_deterministic(uri in valid_uri_strategy()) {
            let rt = tokio::runtime::Runtime::new().unwrap();

            let (content1, content2) = rt.block_on(async {
                let server = StatelessTemplateServer::new().unwrap();
                let c1 = server.get_template_content(&uri).await.unwrap();
                let c2 = server.get_template_content(&uri).await.unwrap();
                (c1, c2)
            });

            prop_assert_eq!(content1.as_ref(), content2.as_ref(), "Content should be deterministic for: {}", uri);
        }

        #[test]
        fn prop_all_listed_templates_have_valid_structure(_seed in 0u32..10) {
            let rt = tokio::runtime::Runtime::new().unwrap();

            let templates = rt.block_on(async {
                let server = StatelessTemplateServer::new().unwrap();
                server.list_templates("").await.unwrap()
            });

            for template in templates {
                prop_assert!(!template.uri.is_empty(), "URI should not be empty");
                prop_assert!(!template.name.is_empty(), "Name should not be empty");
                prop_assert!(!template.description.is_empty(), "Description should not be empty");
                prop_assert!(template.semantic_version.major >= 1, "Version should be >= 1.0.0");
            }
        }
    }
}
