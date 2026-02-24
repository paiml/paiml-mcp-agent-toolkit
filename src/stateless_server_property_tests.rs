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
