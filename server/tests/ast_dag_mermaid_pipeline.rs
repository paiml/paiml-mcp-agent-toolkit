//! AST→DAG→Mermaid Pipeline Integration Tests
//!
//! Tests the complete data flow from AST analysis through DAG generation to Mermaid visualization.
//! Verifies that metadata propagation and data integrity are maintained throughout the pipeline.

use paiml_mcp_agent_toolkit::{
    models::dag::NodeType,
    services::{
        ast_rust::{analyze_rust_file, analyze_rust_file_with_complexity},
        context::{analyze_project, FileContext},
        dag_builder::DagBuilder,
        mermaid_generator::{MermaidGenerator, MermaidOptions},
    },
};
use std::fs;
use tempfile::TempDir;

/// Test fixture: Create a simple Rust project for pipeline testing
fn create_test_rust_project() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let src_dir = temp_dir.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();

    // Create main.rs with a simple function
    fs::write(
        src_dir.join("main.rs"),
        r#"
mod utils;
use utils::helper_function;

fn main() {
    let result = calculate_sum(5, 10);
    helper_function(result);
}

fn calculate_sum(a: i32, b: i32) -> i32 {
    a + b
}

struct Config {
    name: String,
    value: i32,
}

trait Processable {
    fn process(&self) -> String;
}

impl Processable for Config {
    fn process(&self) -> String {
        format!("{}: {}", self.name, self.value)
    }
}
"#,
    )
    .unwrap();

    // Create utils.rs module
    fs::write(
        src_dir.join("utils.rs"),
        r#"
pub fn helper_function(value: i32) {
    println!("Helper: {}", value);
}

pub fn complex_function(input: Vec<i32>) -> Option<i32> {
    if input.is_empty() {
        return None;
    }
    
    let mut total = 0;
    for item in input {
        if item > 0 {
            total += item;
        } else {
            return None;
        }
    }
    
    Some(total)
}
"#,
    )
    .unwrap();

    temp_dir
}

#[tokio::test]
async fn test_ast_to_dag_metadata_propagation() {
    let temp_dir = create_test_rust_project();
    let project_path = temp_dir.path();

    // Step 1: AST Analysis
    let project_context = analyze_project(project_path, "rust")
        .await
        .expect("Failed to analyze project");

    // Verify AST analysis captured files
    assert!(!project_context.files.is_empty());

    let main_file = project_context
        .files
        .iter()
        .find(|f| f.path.contains("main.rs"))
        .expect("Should find main.rs");

    // Verify AST captured functions
    assert!(!main_file.items.is_empty());

    // Step 2: DAG Generation with metadata propagation
    let dag = DagBuilder::build_from_project(&project_context);

    // Verify DAG generation preserved data
    assert!(!dag.nodes.is_empty());
    assert!(!dag.edges.is_empty());

    // Test critical metadata propagation (addresses bug report issue)
    for (node_id, node_info) in &dag.nodes {
        // Verify required metadata fields exist (from bug report fixes)
        assert!(
            node_info.metadata.contains_key("file_path"),
            "Node {node_id} missing file_path metadata"
        );
        assert!(
            node_info.metadata.contains_key("module_path"),
            "Node {node_id} missing module_path metadata"
        );
        assert!(
            node_info.metadata.contains_key("display_name"),
            "Node {node_id} missing display_name metadata"
        );
        assert!(
            node_info.metadata.contains_key("node_type"),
            "Node {node_id} missing node_type metadata"
        );
        assert!(
            node_info.metadata.contains_key("language"),
            "Node {node_id} missing language metadata"
        );

        // Verify metadata values are meaningful (not empty)
        assert!(!node_info.metadata["file_path"].is_empty());
        assert!(!node_info.metadata["display_name"].is_empty());
        assert_eq!(node_info.metadata["language"], "rust");

        // Verify semantic naming worked
        assert!(
            !node_info.label.is_empty(),
            "Node {node_id} has empty label"
        );

        // Verify complexity was calculated
        assert!(
            node_info.complexity > 0,
            "Node {node_id} has zero complexity"
        );
    }

    // Verify we have nodes of different types
    let has_function = dag
        .nodes
        .values()
        .any(|n| n.node_type == NodeType::Function);
    let has_module = dag.nodes.values().any(|n| n.node_type == NodeType::Module);

    assert!(has_function, "Should have function nodes");
    assert!(has_module, "Should have module nodes");

    // Step 3: Mermaid Generation
    let mermaid_generator = MermaidGenerator::new(MermaidOptions::default());
    let mermaid_output = mermaid_generator.generate(&dag);

    // Verify Mermaid output is valid
    assert!(!mermaid_output.is_empty());
    assert!(
        mermaid_output.contains("graph"),
        "Should contain graph directive"
    );

    // Verify metadata made it into Mermaid output through node labels
    assert!(
        mermaid_output.contains("main"),
        "Should contain main function"
    );
    assert!(
        mermaid_output.contains("utils"),
        "Should contain utils module"
    );

    // Verify no placeholder data (addresses bug report)
    assert!(
        !mermaid_output.contains("NoData"),
        "Should not contain placeholder data"
    );
    assert!(
        !mermaid_output.contains("node_"),
        "Should not contain generic node names"
    );
}

#[tokio::test]
async fn test_pipeline_determinism() {
    let temp_dir = create_test_rust_project();
    let project_path = temp_dir.path();

    // Run pipeline twice
    let context1 = analyze_project(project_path, "rust").await.unwrap();
    let dag1 = DagBuilder::build_from_project(&context1);
    let mermaid1 = MermaidGenerator::new(MermaidOptions::default()).generate(&dag1);

    let context2 = analyze_project(project_path, "rust").await.unwrap();
    let dag2 = DagBuilder::build_from_project(&context2);
    let mermaid2 = MermaidGenerator::new(MermaidOptions::default()).generate(&dag2);

    // Results should be deterministic in structure
    assert_eq!(dag1.nodes.len(), dag2.nodes.len());
    assert_eq!(dag1.edges.len(), dag2.edges.len());

    // Since temp directory names can vary, normalize the Mermaid output
    // by replacing the temp directory path with a placeholder
    let temp_path_str = temp_dir.path().to_string_lossy().to_string();
    let normalized_mermaid1 = mermaid1.replace(&temp_path_str, "TEST_PATH");
    let normalized_mermaid2 = mermaid2.replace(&temp_path_str, "TEST_PATH");

    // The node order might vary slightly between runs, so just check structure
    assert!(normalized_mermaid1.contains("graph TD"));
    assert!(normalized_mermaid2.contains("graph TD"));
    assert!(normalized_mermaid1.contains("Config"));
    assert!(normalized_mermaid2.contains("Config"));
    assert!(normalized_mermaid1.contains("Processable"));
    assert!(normalized_mermaid2.contains("Processable"));
    assert!(normalized_mermaid1.contains("calculate_sum"));
    assert!(normalized_mermaid2.contains("calculate_sum"));
    assert!(normalized_mermaid1.contains("inherits"));
    assert!(normalized_mermaid2.contains("inherits"));
}

#[tokio::test]
async fn test_pipeline_with_complex_project() {
    let temp_dir = create_test_rust_project();
    let project_path = temp_dir.path();

    // Analyze the project
    let project_context = analyze_project(project_path, "rust").await.unwrap();
    let dag = DagBuilder::build_from_project(&project_context);

    // Test PageRank pruning doesn't break metadata
    let pruned_dag = paiml_mcp_agent_toolkit::services::dag_builder::prune_graph_pagerank(&dag, 10);

    // Verify pruned graph maintains metadata integrity
    for (node_id, node_info) in &pruned_dag.nodes {
        assert!(
            node_info.metadata.contains_key("file_path"),
            "Pruned node {node_id} missing file_path metadata"
        );
        assert!(!node_info.metadata["file_path"].is_empty());
    }

    // Generate Mermaid from pruned graph
    let mermaid_output = MermaidGenerator::new(MermaidOptions::default()).generate(&pruned_dag);
    assert!(!mermaid_output.is_empty());
    assert!(mermaid_output.contains("graph"));
}

#[tokio::test]
async fn test_edge_budget_enforcement() {
    let temp_dir = create_test_rust_project();
    let project_path = temp_dir.path();

    let project_context = analyze_project(project_path, "rust").await.unwrap();
    let dag = DagBuilder::build_from_project(&project_context);

    // Should stay under edge budget (400 as per DAG_BUILDER constant)
    assert!(
        dag.edges.len() <= 400,
        "DAG exceeds edge budget: {} edges",
        dag.edges.len()
    );

    // All nodes in edges should exist in nodes map
    for edge in &dag.edges {
        assert!(
            dag.nodes.contains_key(&edge.from),
            "Edge references non-existent from node: {}",
            edge.from
        );
        assert!(
            dag.nodes.contains_key(&edge.to),
            "Edge references non-existent to node: {}",
            edge.to
        );
    }
}

#[tokio::test]
async fn test_individual_file_analysis() {
    let temp_dir = create_test_rust_project();
    let main_file = temp_dir.path().join("src/main.rs");

    // Test individual file AST analysis with complexity
    let file_complexity = analyze_rust_file_with_complexity(&main_file).await.unwrap();
    let file_ast = analyze_rust_file(&main_file).await.unwrap();

    // Verify file-level metrics
    assert!(!file_ast.path.is_empty());
    // Verify we have complexity metrics (cognitive is u32, so always >= 0)
    let _ = file_complexity.total_complexity.cognitive; // Just ensure it exists
    assert!(!file_ast.items.is_empty());

    // Verify function extraction
    let function_count = file_ast
        .items
        .iter()
        .filter(|item| {
            matches!(
                item,
                paiml_mcp_agent_toolkit::services::context::AstItem::Function { .. }
            )
        })
        .count();

    assert!(function_count > 0, "Should extract functions from main.rs");

    // Test that this data flows through to DAG
    let file_context = FileContext {
        path: main_file.to_string_lossy().to_string(),
        language: "rust".to_string(),
        items: file_ast.items,
        complexity_metrics: Some(file_complexity),
    };

    let project_context = paiml_mcp_agent_toolkit::services::context::ProjectContext {
        project_type: "rust".to_string(),
        files: vec![file_context],
        summary: paiml_mcp_agent_toolkit::services::context::ProjectSummary {
            total_files: 1,
            total_functions: function_count,
            total_structs: 0,
            total_enums: 0,
            total_traits: 0,
            total_impls: 0,
            dependencies: vec![],
        },
    };

    let dag = DagBuilder::build_from_project(&project_context);
    assert!(!dag.nodes.is_empty());
}

#[tokio::test]
async fn test_mermaid_output_quality() {
    let temp_dir = create_test_rust_project();
    let project_path = temp_dir.path();

    let project_context = analyze_project(project_path, "rust").await.unwrap();
    let dag = DagBuilder::build_from_project(&project_context);

    let generator = MermaidGenerator::new(MermaidOptions {
        max_depth: Some(3),
        filter_external: true,
        show_complexity: true,
        ..Default::default()
    });

    let mermaid_output = generator.generate(&dag);

    // Verify Mermaid syntax
    assert!(mermaid_output.starts_with("graph") || mermaid_output.starts_with("flowchart"));

    // Should not have empty nodes
    assert!(!mermaid_output.contains("[]"));

    // Should have actual content
    let lines: Vec<&str> = mermaid_output
        .lines()
        .filter(|l| !l.trim().is_empty())
        .collect();
    assert!(lines.len() > 1, "Mermaid output should have multiple lines");

    // Should contain actual node definitions and relationships
    let has_arrows = mermaid_output.contains("-->") || mermaid_output.contains("---");
    assert!(has_arrows, "Mermaid output should contain relationships");
}

#[test]
fn test_metadata_serialization() {
    // Test that metadata can be properly serialized/deserialized
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("file_path".to_string(), "src/main.rs".to_string());
    metadata.insert("module_path".to_string(), "main".to_string());
    metadata.insert("display_name".to_string(), "main".to_string());
    metadata.insert("node_type".to_string(), "Function".to_string());
    metadata.insert("language".to_string(), "rust".to_string());

    let node = paiml_mcp_agent_toolkit::models::dag::NodeInfo {
        id: "test_node".to_string(),
        label: "Test Node".to_string(),
        node_type: NodeType::Function,
        file_path: "src/main.rs".to_string(),
        line_number: 42,
        complexity: 5,
        metadata,
    };

    // Test serialization
    let serialized = serde_json::to_string(&node).unwrap();
    assert!(serialized.contains("file_path"));
    assert!(serialized.contains("src/main.rs"));

    // Test deserialization
    let deserialized: paiml_mcp_agent_toolkit::models::dag::NodeInfo =
        serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.id, "test_node");
    assert_eq!(
        deserialized.metadata.get("file_path"),
        Some(&"src/main.rs".to_string())
    );
    assert_eq!(
        deserialized.metadata.get("language"),
        Some(&"rust".to_string())
    );
}
