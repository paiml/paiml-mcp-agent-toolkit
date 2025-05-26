use crate::{TemplateServer, TemplateServerTrait};

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
