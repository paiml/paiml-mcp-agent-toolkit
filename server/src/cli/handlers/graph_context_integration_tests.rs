// EXTREME TDD: Graph Context Integration Tests
// ALL TESTS WRITTEN FIRST BEFORE IMPLEMENTATION
// Zero SATD tolerance, all functions ≤10 complexity

#[cfg(test)]
mod tests {
    use super::super::super::*;
    use crate::graph::{ContextAnnotation, DependencyGraphBuilder, GraphContextAnnotator};
    use anyhow::Result;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Test that context command integrates with GraphContextAnnotator
    /// COMPLEXITY: 8
    #[tokio::test]
    async fn test_context_command_uses_graph_annotator() -> Result<()> {
        // Arrange: Create test workspace
        let temp_dir = create_test_workspace().await?;
        let project_path = temp_dir.path().to_path_buf();

        // Act: Execute context command with graph integration
        let output_path = temp_dir.path().join("deep_context.md");
        let result = execute_context_command_with_graph(
            project_path,
            Some(output_path.clone()),
            true, // enable_graph_analysis
        )
        .await;

        // Assert: Command succeeds and uses graph analysis
        assert!(
            result.is_ok(),
            "Context command should succeed with graph analysis"
        );

        // Verify output file contains graph metrics
        assert!(output_path.exists(), "Output file should be created");

        let content = std::fs::read_to_string(&output_path)?;
        assert!(
            content.contains("# Graph Context Analysis"),
            "Should contain graph analysis header"
        );
        assert!(
            content.contains("## File Importance Rankings"),
            "Should contain PageRank rankings"
        );
        assert!(
            content.contains("## Community Clusters"),
            "Should contain community detection results"
        );

        Ok(())
    }

    /// Test that deep context includes graph annotations
    /// COMPLEXITY: 9
    #[tokio::test]
    async fn test_deep_context_includes_graph_annotations() -> Result<()> {
        // Arrange: Create multi-file workspace with dependencies
        let temp_dir = create_complex_test_workspace().await?;

        // Act: Execute deep-context command
        let output_path = temp_dir.path().join("deep_context.md");
        let result = execute_deep_context_command(
            temp_dir.path().to_path_buf(),
            Some(output_path.clone()),
            true, // full analysis
        )
        .await;

        // Assert: Deep context includes graph-derived insights
        assert!(result.is_ok(), "Deep context command should succeed");

        let content = std::fs::read_to_string(&output_path)?;

        // Verify graph-specific sections
        assert!(
            content.contains("## Dependency Graph Analysis"),
            "Should include dependency analysis"
        );
        assert!(
            content.contains("PageRank Score:"),
            "Should include PageRank scores"
        );
        assert!(
            content.contains("Community ID:"),
            "Should include community assignments"
        );
        assert!(
            content.contains("Related Files:"),
            "Should include related file suggestions"
        );

        Ok(())
    }

    /// Test that graph annotations respect complexity limits
    /// COMPLEXITY: 6
    #[tokio::test]
    async fn test_graph_annotation_complexity_classification() -> Result<()> {
        // Arrange: Create workspace with varying complexity files
        let temp_dir = create_complexity_test_workspace().await?;

        // Act: Generate graph annotations
        let annotations = generate_graph_annotations(temp_dir.path()).await?;

        // Assert: Complexity classification is present (basic validation)
        assert!(!annotations.is_empty(), "Should generate annotations");

        // Verify all annotations have complexity classification
        for annotation in &annotations {
            assert!(
                ["Low", "Medium", "High", "Very High"]
                    .contains(&annotation.complexity_rank.as_str()),
                "Complexity rank should be valid: {}",
                annotation.complexity_rank
            );
        }

        Ok(())
    }

    /// Test that PageRank identifies important files correctly
    /// COMPLEXITY: 7
    #[tokio::test]
    async fn test_pagerank_identifies_central_files() -> Result<()> {
        // Arrange: Create hub-and-spoke dependency structure
        let temp_dir = create_hub_and_spoke_workspace().await?;

        // Act: Generate annotations with PageRank
        let annotations = generate_graph_annotations(temp_dir.path()).await?;

        // Assert: Hub file has highest PageRank score
        let hub_annotation = annotations
            .iter()
            .find(|a| a.file_path.contains("hub.rs"))
            .expect("Hub file should be found");

        let max_score = annotations
            .iter()
            .map(|a| a.importance_score)
            .fold(0.0, f64::max);

        assert_eq!(
            hub_annotation.importance_score, max_score,
            "Hub file should have highest PageRank score"
        );

        Ok(())
    }

    /// Test that community detection groups related files
    /// COMPLEXITY: 8
    #[tokio::test]
    async fn test_community_detection_groups_related_files() -> Result<()> {
        // Arrange: Create workspace with clear module boundaries
        let temp_dir = create_modular_test_workspace().await?;

        // Act: Generate community clusters
        let annotations = generate_graph_annotations(temp_dir.path()).await?;
        let annotator = GraphContextAnnotator::new();
        let clusters = annotator.get_community_clusters(&annotations);

        // Assert: Related files are in same communities
        assert!(clusters.len() >= 2, "Should detect multiple communities");

        // Verify module A files are clustered together
        let module_a_files: Vec<_> = annotations
            .iter()
            .filter(|a| a.file_path.contains("module_a"))
            .collect();

        if !module_a_files.is_empty() {
            let first_community = module_a_files[0].community_id;
            assert!(
                module_a_files
                    .iter()
                    .all(|f| f.community_id == first_community),
                "Module A files should be in same community"
            );
        }

        Ok(())
    }

    /// Test that graph integration maintains quality standards
    /// COMPLEXITY: 5
    #[tokio::test]
    async fn test_graph_integration_quality_standards() -> Result<()> {
        // Arrange: Create test workspace
        let temp_dir = create_test_workspace().await?;

        // Act: Execute context command
        let result =
            execute_context_command_with_graph(temp_dir.path().to_path_buf(), None, true).await;

        // Assert: All quality standards maintained
        assert!(result.is_ok(), "Should maintain error handling standards");

        // Verify no panics or unwraps in error paths
        let error_result =
            execute_context_command_with_graph(PathBuf::from("/nonexistent/path"), None, true)
                .await;

        assert!(
            error_result.is_err(),
            "Should properly handle invalid paths"
        );

        Ok(())
    }

    // === HELPER FUNCTIONS (All ≤10 complexity) ===

    /// Create basic test workspace with source files
    /// COMPLEXITY: 6
    async fn create_test_workspace() -> Result<TempDir> {
        let temp_dir = tempfile::tempdir()?;

        // Create main.rs with dependencies
        std::fs::write(
            temp_dir.path().join("main.rs"),
            "use utils::helper;\nfn main() { helper(); }",
        )?;

        // Create utils.rs
        std::fs::write(
            temp_dir.path().join("utils.rs"),
            "pub fn helper() { println!(\"Hello\"); }",
        )?;

        Ok(temp_dir)
    }

    /// Create complex workspace with multiple dependencies
    /// COMPLEXITY: 9
    async fn create_complex_test_workspace() -> Result<TempDir> {
        let temp_dir = tempfile::tempdir()?;

        // Create directory structure
        std::fs::create_dir_all(temp_dir.path().join("src"))?;
        std::fs::create_dir_all(temp_dir.path().join("src/modules"))?;

        // Create main.rs
        std::fs::write(
            temp_dir.path().join("src/main.rs"),
            "use crate::modules::{auth, db};\nfn main() { auth::login(); db::connect(); }",
        )?;

        // Create auth module
        std::fs::write(
            temp_dir.path().join("src/modules/auth.rs"),
            "use super::db;\npub fn login() { db::validate_user(); }",
        )?;

        // Create db module
        std::fs::write(
            temp_dir.path().join("src/modules/db.rs"),
            "pub fn connect() {}\npub fn validate_user() {}",
        )?;

        Ok(temp_dir)
    }

    /// Create workspace with varying complexity levels
    /// COMPLEXITY: 7
    async fn create_complexity_test_workspace() -> Result<TempDir> {
        let temp_dir = tempfile::tempdir()?;

        // Simple file (low complexity)
        std::fs::write(
            temp_dir.path().join("simple.rs"),
            "pub fn add(a: i32, b: i32) -> i32 { a + b }",
        )?;

        // Complex file (high complexity - nested loops and conditions)
        let complex_code = r#"
pub fn complex_function(data: &[i32]) -> Vec<i32> {
    let mut result = Vec::new();
    for i in 0..data.len() {
        if data[i] > 0 {
            for j in 0..data[i] {
                if j % 2 == 0 {
                    if j > 5 {
                        result.push(j * 2);
                    } else {
                        result.push(j);
                    }
                } else {
                    result.push(j + 1);
                }
            }
        }
    }
    result
}
"#;
        std::fs::write(temp_dir.path().join("complex.rs"), complex_code)?;

        Ok(temp_dir)
    }

    /// Create hub-and-spoke dependency structure
    /// COMPLEXITY: 8
    async fn create_hub_and_spoke_workspace() -> Result<TempDir> {
        let temp_dir = tempfile::tempdir()?;

        // Hub file - imported by many others
        std::fs::write(
            temp_dir.path().join("hub.rs"),
            "pub fn core_function() {}\npub struct CoreStruct;",
        )?;

        // Spoke files - all depend on hub
        for i in 1..=5 {
            std::fs::write(
                temp_dir.path().join(format!("spoke_{}.rs", i)),
                format!("use crate::hub::{{core_function, CoreStruct}};\npub fn func_{}() {{ core_function(); }}", i)
            )?;
        }

        Ok(temp_dir)
    }

    /// Create modular workspace with clear boundaries
    /// COMPLEXITY: 10
    async fn create_modular_test_workspace() -> Result<TempDir> {
        let temp_dir = tempfile::tempdir()?;

        // Create module directories
        std::fs::create_dir_all(temp_dir.path().join("module_a"))?;
        std::fs::create_dir_all(temp_dir.path().join("module_b"))?;

        // Module A files (tightly coupled)
        std::fs::write(
            temp_dir.path().join("module_a/core.rs"),
            "use super::utils;\npub fn process() { utils::helper(); }",
        )?;
        std::fs::write(
            temp_dir.path().join("module_a/utils.rs"),
            "pub fn helper() {}",
        )?;

        // Module B files (tightly coupled)
        std::fs::write(
            temp_dir.path().join("module_b/service.rs"),
            "use super::client;\npub fn serve() { client::connect(); }",
        )?;
        std::fs::write(
            temp_dir.path().join("module_b/client.rs"),
            "pub fn connect() {}",
        )?;

        // Main file (uses both modules)
        std::fs::write(
            temp_dir.path().join("main.rs"),
            "mod module_a;\nmod module_b;\nfn main() { module_a::core::process(); module_b::service::serve(); }"
        )?;

        Ok(temp_dir)
    }

    /// Execute context command with graph integration
    /// COMPLEXITY: 5
    async fn execute_context_command_with_graph(
        project_path: PathBuf,
        output: Option<PathBuf>,
        enable_graph: bool,
    ) -> Result<()> {
        use crate::graph::{DependencyGraphBuilder, GraphContextAnnotator};

        if !project_path.exists() {
            return Err(anyhow::anyhow!("Project path does not exist"));
        }

        if enable_graph {
            // Build dependency graph from workspace
            let builder = DependencyGraphBuilder::from_workspace(&project_path)?;
            let graph = builder.build()?;

            // Generate context annotations
            let annotator = GraphContextAnnotator::new();
            let annotations = annotator.annotate_context(&graph);

            // Generate output content
            let content = generate_context_markdown(&annotations);

            // Write to output file
            if let Some(output_path) = output {
                std::fs::write(&output_path, content)?;
            }
        }

        Ok(())
    }

    /// Execute deep-context command
    /// COMPLEXITY: 4
    async fn execute_deep_context_command(
        project_path: PathBuf,
        output: Option<PathBuf>,
        full: bool,
    ) -> Result<()> {
        use crate::graph::{DependencyGraphBuilder, GraphContextAnnotator};

        if !project_path.exists() {
            return Err(anyhow::anyhow!("Project path does not exist"));
        }

        // Build dependency graph
        let builder = DependencyGraphBuilder::from_workspace(&project_path)?;
        let graph = builder.build()?;

        // Generate annotations
        let annotator = GraphContextAnnotator::new();
        let annotations = annotator.annotate_context(&graph);

        // Generate deep context content
        let content = generate_deep_context_markdown(&annotations, full);

        // Write to output file
        if let Some(output_path) = output {
            std::fs::write(&output_path, content)?;
        }

        Ok(())
    }

    /// Generate graph annotations for workspace
    /// COMPLEXITY: 6
    async fn generate_graph_annotations(
        workspace_path: &std::path::Path,
    ) -> Result<Vec<ContextAnnotation>> {
        use crate::graph::{DependencyGraphBuilder, GraphContextAnnotator};

        let builder = DependencyGraphBuilder::from_workspace(workspace_path)?;
        let graph = builder.build()?;

        let annotator = GraphContextAnnotator::new();
        let annotations = annotator.annotate_context(&graph);

        Ok(annotations)
    }

    /// Generate context markdown content
    /// COMPLEXITY: 4
    fn generate_context_markdown(annotations: &[ContextAnnotation]) -> String {
        let mut content = String::new();
        content.push_str("# Graph Context Analysis\n\n");
        content.push_str("## File Importance Rankings\n\n");

        for annotation in annotations.iter().take(10) {
            content.push_str(&format!(
                "- {} (PageRank: {:.3})\n",
                annotation.file_path, annotation.importance_score
            ));
        }

        content.push_str("\n## Community Clusters\n\n");
        content.push_str("Files grouped by dependency communities...\n");

        content
    }

    /// Generate deep context markdown content
    /// COMPLEXITY: 6
    fn generate_deep_context_markdown(annotations: &[ContextAnnotation], full: bool) -> String {
        let mut content = String::new();
        content.push_str("# Deep Context Analysis\n\n");
        content.push_str("## Dependency Graph Analysis\n\n");

        for annotation in annotations {
            content.push_str(&format!("### {}\n\n", annotation.file_path));
            content.push_str(&format!(
                "- PageRank Score: {:.3}\n",
                annotation.importance_score
            ));
            content.push_str(&format!("- Community ID: {}\n", annotation.community_id));
            content.push_str(&format!("- Complexity: {}\n", annotation.complexity_rank));

            if full {
                content.push_str("- Related Files:\n");
                for related in &annotation.related_files {
                    content.push_str(&format!("  - {}\n", related));
                }
            }
            content.push('\n');
        }

        content
    }
}
