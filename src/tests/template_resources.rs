use crate::stateless_server::StatelessTemplateServer;
use std::sync::Arc;

fn create_test_server() -> Arc<StatelessTemplateServer> {
    Arc::new(StatelessTemplateServer::new().unwrap())
}

#[tokio::test]
async fn test_list_all_templates() {
    let server = create_test_server();

    let templates = server.list_templates("").await.unwrap();

    // Should have templates for all file types (we have 3 templates embedded)
    assert!(templates.len() >= 3);

    // Verify all templates have required fields
    for template in &templates {
        assert!(!template.uri.is_empty());
        assert!(!template.name.is_empty());
        assert!(!template.description.is_empty());
        assert!(!template.s3_object_key.is_empty());
        assert_eq!(template.content_hash, "embedded");
    }
}

#[tokio::test]
async fn test_filter_templates_by_prefix() {
    let server = create_test_server();

    // Filter by makefile
    let makefiles = server.list_templates("makefile").await.unwrap();
    assert!(!makefiles.is_empty());
    for template in &makefiles {
        assert!(template.uri.contains("makefile/"));
    }

    // Filter by language
    let rust_templates = server.list_templates("rust").await.unwrap();
    assert!(!rust_templates.is_empty());
    for template in &rust_templates {
        assert!(template.uri.contains("/rust/"));
    }
}

#[tokio::test]
async fn test_get_template_metadata() {
    let server = create_test_server();

    // Test Rust Makefile metadata
    let rust_makefile = server
        .get_template_metadata("template://makefile/rust/cli")
        .await
        .unwrap();

    assert_eq!(rust_makefile.uri, "template://makefile/rust/cli");
    assert!(rust_makefile.name.contains("Makefile"));
    assert!(rust_makefile.description.contains("Rust"));

    // Check parameters
    assert!(!rust_makefile.parameters.is_empty());

    // Should have project_name parameter
    let project_name_param = rust_makefile
        .parameters
        .iter()
        .find(|p| p.name == "project_name")
        .expect("Should have project_name parameter");

    assert!(project_name_param.required);
    assert!(!project_name_param.description.is_empty());
}

#[tokio::test]
async fn test_get_template_content() {
    let server = create_test_server();

    // Get content directly by URI
    let content = server
        .get_template_content("template://readme/rust/cli")
        .await
        .unwrap();

    // Should be a valid Handlebars template
    assert!(content.contains("{{"));
    assert!(content.contains("}}"));
    assert!(content.contains("project_name"));
    assert!(content.contains("Pragmatic AI Labs MCP Agent Toolkit"));
}

#[tokio::test]
async fn test_invalid_template_uri() {
    let server = create_test_server();

    // Invalid URI format
    let result = server.get_template_metadata("invalid://uri").await;
    assert!(result.is_err());

    // Non-existent template
    let result = server
        .get_template_metadata("template://makefile/cobol/mainframe")
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_template_categories() {
    let server = create_test_server();

    let templates = server.list_templates("").await.unwrap();

    // Check we have all three categories
    let has_makefile = templates.iter().any(|t| {
        matches!(
            &t.category,
            crate::models::template::TemplateCategory::Makefile
        )
    });
    let has_readme = templates.iter().any(|t| {
        matches!(
            &t.category,
            crate::models::template::TemplateCategory::Readme
        )
    });
    let has_gitignore = templates.iter().any(|t| {
        matches!(
            &t.category,
            crate::models::template::TemplateCategory::Gitignore
        )
    });

    assert!(has_makefile, "Should have Makefile templates");
    assert!(has_readme, "Should have README templates");
    assert!(has_gitignore, "Should have .gitignore templates");
}

#[tokio::test]
async fn test_template_toolchains() {
    let server = create_test_server();

    let templates = server.list_templates("").await.unwrap();

    // Check for Rust toolchain
    let has_rust = templates.iter().any(|t| {
        matches!(
            &t.toolchain,
            crate::models::template::Toolchain::RustCli { .. }
        )
    });

    assert!(has_rust, "Should have Rust toolchain templates");
}

#[tokio::test]
async fn test_template_parameter_types() {
    let server = create_test_server();

    let templates = server.list_templates("").await.unwrap();

    for template in templates {
        // Check parameter types
        for param in &template.parameters {
            // Required parameters should have descriptions
            if param.required {
                assert!(
                    !param.description.is_empty(),
                    "Required parameter '{}' in template '{}' should have a description",
                    param.name,
                    template.uri
                );
            }
        }
    }
}

#[tokio::test]
async fn test_rust_template_parameters() {
    let server = create_test_server();

    let rust_makefile = server
        .get_template_metadata("template://makefile/rust/cli")
        .await
        .unwrap();

    // Check for expected Rust-specific parameters
    let param_names: Vec<_> = rust_makefile.parameters.iter().map(|p| &p.name).collect();

    assert!(param_names.contains(&&"project_name".to_string()));
    assert!(param_names.contains(&&"has_tests".to_string()));

    // Check parameter properties
    let has_tests = rust_makefile
        .parameters
        .iter()
        .find(|p| p.name == "has_tests")
        .unwrap();

    assert_eq!(has_tests.default_value, Some("true".to_string()));
}
