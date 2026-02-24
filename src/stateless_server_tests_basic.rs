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
