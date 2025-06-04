use crate::{ContentCache, MetadataCache, S3Client, TemplateServer, TemplateServerTrait};
use std::sync::Arc;
use tokio::sync::RwLock;

// Include comprehensive code smell tests for README features
mod code_smell_comprehensive_tests;

// Include Clap command structure validation tests
mod clap_command_structure_tests;

// Include Clap argument parsing correctness tests
mod clap_argument_parsing_tests;

// Include Clap environment variable integration tests
mod clap_env_var_tests;

#[tokio::test]
async fn test_template_server_new() {
    let server = TemplateServer::new().await;
    assert!(server.is_ok());

    let server = server.unwrap();
    assert_eq!(server.bucket_name, "dummy");
}

#[tokio::test]
async fn test_template_server_trait_implementation() {
    let server = TemplateServer::new().await.unwrap();

    // Test get_renderer
    let renderer = server.get_renderer();
    assert!(std::ptr::eq(renderer, &server.renderer)); // They should be the same reference

    // Test get_metadata_cache
    assert!(server.get_metadata_cache().is_some());

    // Test get_content_cache
    assert!(server.get_content_cache().is_some());

    // Test get_s3_client
    assert!(server.get_s3_client().is_some());

    // Test get_bucket_name
    assert_eq!(server.get_bucket_name(), Some("dummy"));
}

#[tokio::test]
async fn test_template_server_deprecated_methods() {
    let server = TemplateServer::new().await.unwrap();

    // Test get_template_metadata returns error
    let result = server.get_template_metadata("test").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("deprecated"));

    // Test get_template_content returns error
    let result = server.get_template_content("test").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("deprecated"));

    // Test list_templates returns error (via trait)
    let result = TemplateServerTrait::list_templates(&server, "test").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("deprecated"));
}

#[tokio::test]
async fn test_warm_cache() {
    let server = TemplateServer::new().await.unwrap();

    // warm_cache should succeed even though individual template fetches fail
    let result = server.warm_cache().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_template_server_cache_initialization() {
    let server = TemplateServer::new().await.unwrap();

    // Check metadata cache is initialized and empty
    let metadata_cache = server.metadata_cache.read().await;
    assert_eq!(metadata_cache.len(), 0);
    drop(metadata_cache);

    // Check content cache is initialized and empty
    let content_cache = server.content_cache.read().await;
    assert_eq!(content_cache.len(), 0);
}

#[tokio::test]
async fn test_template_server_cache_sizes() {
    let server = TemplateServer::new().await.unwrap();

    // Verify cache sizes are set correctly (1024 total, split between metadata and content)
    let metadata_cache = server.metadata_cache.read().await;
    assert!(metadata_cache.cap().get() >= 512);
    drop(metadata_cache);

    let content_cache = server.content_cache.read().await;
    assert!(content_cache.cap().get() >= 1024);
}

#[tokio::test]
async fn test_warm_cache_templates() {
    let server = TemplateServer::new().await.unwrap();

    // Call warm_cache and ensure it tries to load expected templates
    let result = server.warm_cache().await;
    assert!(result.is_ok());

    // The cache should still be empty since get_template_metadata returns errors
    let metadata_cache = server.metadata_cache.read().await;
    assert_eq!(metadata_cache.len(), 0);
}

#[tokio::test]
async fn test_template_server_trait_via_methods() {
    let server = TemplateServer::new().await.unwrap();

    // Test calling trait methods directly
    let result = TemplateServerTrait::get_template_metadata(&server, "test").await;
    assert!(result.is_err());

    let result = TemplateServerTrait::get_template_content(&server, "test").await;
    assert!(result.is_err());

    // Test trait method accessors
    let _ = TemplateServerTrait::get_renderer(&server);
    let _ = TemplateServerTrait::get_metadata_cache(&server);
    let _ = TemplateServerTrait::get_content_cache(&server);
    let _ = TemplateServerTrait::get_s3_client(&server);
    let _ = TemplateServerTrait::get_bucket_name(&server);
}

#[test]
fn test_type_aliases() {
    use lru::LruCache;
    use std::num::NonZeroUsize;

    // Test that type aliases work correctly
    let metadata_cache: MetadataCache =
        Arc::new(RwLock::new(LruCache::new(NonZeroUsize::new(10).unwrap())));

    let content_cache: ContentCache =
        Arc::new(RwLock::new(LruCache::new(NonZeroUsize::new(10).unwrap())));

    // Basic sanity check
    assert!(Arc::strong_count(&metadata_cache) == 1);
    assert!(Arc::strong_count(&content_cache) == 1);
}

#[test]
fn test_s3_client_instantiation() {
    // Test S3Client can be instantiated
    let _client = S3Client;
}

#[tokio::test]
async fn test_run_mcp_server_basic() {
    use crate::run_mcp_server;
    use crate::stateless_server::StatelessTemplateServer;
    use std::sync::Arc;

    // Create a test server
    let server = Arc::new(StatelessTemplateServer::new().unwrap());

    // Create a simple invalid request that will cause the server to exit immediately
    std::thread::spawn(move || {
        // Simulate empty stdin which will cause the server to exit
        let _result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(run_mcp_server(server));
    });

    // Give the server a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Test passes if server starts without panic
}

#[test]
fn test_public_exports() {
    // Test that public exports are available
    use crate::{ParameterSpec, ParameterType, TemplateError};

    // Create dummy values to ensure types are used
    let _param_type = ParameterType::String;
    let _param_spec = ParameterSpec {
        name: "test".to_string(),
        description: "test".to_string(),
        param_type: ParameterType::String,
        required: true,
        default_value: None,
        validation_pattern: None,
    };

    // Test error type
    let _error = TemplateError::InvalidUri {
        uri: "test".to_string(),
    };
}
