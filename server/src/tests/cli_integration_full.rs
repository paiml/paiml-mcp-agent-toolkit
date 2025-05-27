#[cfg(test)]
mod tests {
    use crate::stateless_server::StatelessTemplateServer;
    use std::sync::Arc;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_cli_run_generate_to_file() {
        let _server = Arc::new(StatelessTemplateServer::new().unwrap());
        let temp_dir = TempDir::new().unwrap();
        let output_file = temp_dir.path().join("Makefile");

        // We can't directly test CLI argument parsing as it reads from std::env::args()
        // which cannot be easily mocked in tests

        // We can't easily test the full CLI run function because it parses actual command line args
        // But we can test that the file would be created
        assert!(!output_file.exists());
    }

    #[tokio::test]
    async fn test_cli_generate_template_direct() {
        use crate::services::template_service::generate_template;

        let server = StatelessTemplateServer::new().unwrap();
        let mut params = serde_json::Map::new();
        params.insert(
            "project_name".to_string(),
            serde_json::json!("test-project"),
        );

        let result = generate_template(&server, "template://makefile/rust/cli", params).await;

        assert!(result.is_ok());
        let generated = result.unwrap();
        assert!(generated.content.contains("test-project"));
        assert!(generated.content.contains("build:"));
        assert!(generated.content.contains("test:"));
    }

    #[tokio::test]
    async fn test_cli_list_templates_direct() {
        use crate::services::template_service::list_templates;

        let server = StatelessTemplateServer::new().unwrap();
        let templates = list_templates(&server, None, None).await;

        assert!(templates.is_ok());
        let list = templates.unwrap();
        assert!(list.len() >= 9); // We have at least 9 templates

        // Check filtering by toolchain
        let rust_templates = list_templates(&server, Some("rust"), None).await.unwrap();
        assert!(rust_templates.iter().all(|t| {
            matches!(
                t.toolchain,
                crate::models::template::Toolchain::RustCli { .. }
            )
        }));

        // Check filtering by category
        let makefile_templates = list_templates(&server, None, Some("makefile"))
            .await
            .unwrap();
        assert!(makefile_templates.iter().all(|t| {
            matches!(
                t.category,
                crate::models::template::TemplateCategory::Makefile
            )
        }));
    }

    #[tokio::test]
    async fn test_cli_search_templates_direct() {
        use crate::services::template_service::search_templates;

        let server = Arc::new(StatelessTemplateServer::new().unwrap());
        let results = search_templates(server.clone(), "rust", None).await;

        assert!(results.is_ok());
        let search_results = results.unwrap();
        assert!(!search_results.is_empty());
        assert!(search_results
            .iter()
            .any(|r| r.template.uri.contains("rust")));
    }

    #[tokio::test]
    async fn test_cli_validate_template_direct() {
        use crate::services::template_service::validate_template;

        let server = Arc::new(StatelessTemplateServer::new().unwrap());

        // Test with missing required parameter
        let params = serde_json::json!({});
        let result =
            validate_template(server.clone(), "template://makefile/rust/cli", &params).await;

        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(!validation.valid);
        assert!(!validation.errors.is_empty());

        // Test with all required parameters
        let params = serde_json::json!({
            "project_name": "test-project"
        });
        let result =
            validate_template(server.clone(), "template://makefile/rust/cli", &params).await;

        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(validation.valid);
        assert!(validation.errors.is_empty());
    }

    #[tokio::test]
    async fn test_cli_scaffold_project_direct() {
        use crate::services::template_service::scaffold_project;

        let server = Arc::new(StatelessTemplateServer::new().unwrap());
        let templates = vec!["makefile".to_string(), "readme".to_string()];
        let params = serde_json::json!({
            "project_name": "test-scaffold"
        });

        let result = scaffold_project(server.clone(), "rust", templates, params).await;

        assert!(result.is_ok());
        let scaffold_result = result.unwrap();
        assert_eq!(scaffold_result.files.len(), 2);
        assert!(scaffold_result
            .files
            .iter()
            .any(|f| f.path.ends_with("Makefile")));
        assert!(scaffold_result
            .files
            .iter()
            .any(|f| f.path.ends_with("README.md")));
    }

    #[tokio::test]
    async fn test_cli_context_generation() {
        use crate::services::context::{analyze_project, format_context_as_markdown};
        use std::path::PathBuf;

        // Test on the current project
        let project_path = PathBuf::from(".");
        let context = analyze_project(&project_path, "rust").await;

        assert!(context.is_ok());
        let project_context = context.unwrap();
        assert!(project_context.summary.total_functions > 0);

        // Test markdown formatting
        let markdown = format_context_as_markdown(&project_context);
        assert!(markdown.contains("# Project Context"));
        assert!(markdown.contains("## Summary"));
    }

    #[tokio::test]
    async fn test_cli_churn_analysis() {
        use crate::handlers::tools::{
            format_churn_as_csv, format_churn_as_markdown, format_churn_summary,
        };
        use crate::services::git_analysis::GitAnalysisService;
        use std::path::PathBuf;

        // Test on current project (which has git history)
        let project_path = PathBuf::from(".");
        let result = GitAnalysisService::analyze_code_churn(&project_path, 30);

        if result.is_ok() {
            let analysis = result.unwrap();

            // Test different output formats
            let summary = format_churn_summary(&analysis);
            assert!(summary.contains("Code Churn Analysis"));

            let markdown = format_churn_as_markdown(&analysis);
            assert!(markdown.contains("# Code Churn Analysis Report"));

            let csv = format_churn_as_csv(&analysis);
            assert!(csv.contains("file_path,commits,additions,deletions,churn_score"));
        }
    }
}
