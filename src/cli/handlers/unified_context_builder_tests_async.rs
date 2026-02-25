// Async and integration tests for UnifiedContextBuilder
// Included via include!() in unified_context_builder.rs mod tests

// ============================================================================
// Async analysis function tests
// ============================================================================

#[tokio::test]
async fn test_run_entropy_analysis_returns_ok() {
    let temp = create_temp_project();
    let result = run_entropy_analysis(temp.path()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_provability_analysis_returns_ok() {
    let temp = create_temp_project();
    let result = run_provability_analysis(temp.path()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_graph_metrics_analysis_returns_ok() {
    let temp = create_temp_project();
    let result = run_graph_metrics_analysis(temp.path()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_tdg_analysis_returns_ok() {
    let temp = create_temp_project();
    let result = run_tdg_analysis(temp.path()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_dead_code_analysis_returns_ok() {
    let temp = create_temp_project();
    let result = run_dead_code_analysis(temp.path()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_satd_analysis_works() {
    let temp = create_temp_project();
    let result = run_satd_analysis(temp.path()).await;
    assert!(result.is_ok());
}

// ============================================================================
// Async builder methods tests
// ============================================================================

#[tokio::test]
async fn test_builder_add_entropy_analysis_async() {
    let temp = create_temp_project();
    let mut builder = UnifiedContextBuilder::new(temp.path());
    builder.add_entropy_analysis_async().await;

    let output = builder.build();
    assert!(output.contains("## Entropy Analysis"));
}

#[tokio::test]
async fn test_builder_add_provability_analysis() {
    let temp = create_temp_project();
    let mut builder = UnifiedContextBuilder::new(temp.path());
    builder.add_provability_analysis().await;

    let output = builder.build();
    assert!(output.contains("## Provability Analysis"));
}

#[tokio::test]
async fn test_builder_add_graph_metrics() {
    let temp = create_temp_project();
    let mut builder = UnifiedContextBuilder::new(temp.path());
    builder.add_graph_metrics().await;

    let output = builder.build();
    assert!(output.contains("## Graph Metrics"));
}

#[tokio::test]
async fn test_builder_add_tdg_analysis_async() {
    let temp = create_temp_project();
    let mut builder = UnifiedContextBuilder::new(temp.path());
    builder.add_tdg_analysis_async().await;

    let output = builder.build();
    assert!(output.contains("## Technical Debt Gradient"));
}

#[tokio::test]
async fn test_builder_add_dead_code_analysis() {
    let temp = create_temp_project();
    let mut builder = UnifiedContextBuilder::new(temp.path());
    builder.add_dead_code_analysis().await;

    let output = builder.build();
    assert!(output.contains("## Dead Code Analysis"));
}

#[tokio::test]
async fn test_builder_add_satd_analysis() {
    let temp = create_temp_project();
    let mut builder = UnifiedContextBuilder::new(temp.path());
    builder.add_satd_analysis().await;

    let output = builder.build();
    assert!(output.contains("## Self-Admitted Technical Debt"));
}

// ============================================================================
// ProjectContext integration tests
// ============================================================================

#[test]
fn test_builder_add_quality_insights() {
    let temp = create_temp_project();
    let mut builder = UnifiedContextBuilder::new(temp.path());

    // Create a minimal ProjectContext for testing
    let context = ProjectContext {
        project_type: "rust".to_string(),
        files: vec![],
        summary: crate::services::context::ProjectSummary {
            total_files: 10,
            total_functions: 50,
            total_structs: 5,
            total_enums: 3,
            total_traits: 2,
            total_impls: 8,
            dependencies: vec![],
        },
        graph: None,
    };

    builder.add_quality_insights(&context);

    let output = builder.build();
    assert!(output.contains("## Quality Insights"));
    assert!(output.contains("50 functions"));
}

#[test]
fn test_builder_add_recommendations() {
    let temp = create_temp_project();
    let mut builder = UnifiedContextBuilder::new(temp.path());

    let context = ProjectContext {
        project_type: "rust".to_string(),
        files: vec![],
        summary: crate::services::context::ProjectSummary {
            total_files: 5,
            total_functions: 10,
            total_structs: 2,
            total_enums: 1,
            total_traits: 1,
            total_impls: 3,
            dependencies: vec![],
        },
        graph: None,
    };

    builder.add_recommendations(&context);

    let output = builder.build();
    assert!(output.contains("## Recommendations"));
    assert!(output.contains("modularizing"));
}

#[test]
fn test_builder_add_key_components_empty() {
    let temp = create_temp_project();
    let mut builder = UnifiedContextBuilder::new(temp.path());

    let context = ProjectContext {
        project_type: "rust".to_string(),
        files: vec![],
        summary: crate::services::context::ProjectSummary {
            total_files: 0,
            total_functions: 0,
            total_structs: 0,
            total_enums: 0,
            total_traits: 0,
            total_impls: 0,
            dependencies: vec![],
        },
        graph: None,
    };

    builder.add_key_components(&context);

    let output = builder.build();
    assert!(output.contains("## Key Components"));
    assert!(output.contains("No files analyzed"));
}

#[test]
fn test_builder_add_basic_structure_with_context() {
    let temp = create_temp_project();
    let mut builder = UnifiedContextBuilder::new(temp.path());

    let context = ProjectContext {
        project_type: "typescript".to_string(),
        files: vec![],
        summary: crate::services::context::ProjectSummary {
            total_files: 20,
            total_functions: 100,
            total_structs: 10,
            total_enums: 5,
            total_traits: 0,
            total_impls: 15,
            dependencies: vec![],
        },
        graph: None,
    };

    builder.add_basic_structure_with_context(&context);

    let output = builder.build();
    assert!(output.contains("typescript"));
    assert!(output.contains("20"));
    assert!(output.contains("100"));
}
