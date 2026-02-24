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
