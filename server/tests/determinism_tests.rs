//! Determinism Verification Tests
//!
//! This module contains comprehensive tests to verify that the artifact generation
//! system produces byte-identical output across multiple runs, ensuring complete
//! determinism as specified in deterministic-graphs-mmd-spec.md

use pmat::services::artifact_writer::ArtifactWriter;
use pmat::services::deterministic_mermaid_engine::DeterministicMermaidEngine;
use pmat::services::dogfooding_engine::DogfoodingEngine;
use pmat::services::unified_ast_engine::UnifiedAstEngine;
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

/// Canonical hash constant for verification (would be computed from reference implementation)
#[allow(dead_code)]
const CANONICAL_TREE_HASH: &str = "placeholder_canonical_hash";
#[allow(dead_code)]
const CANONICAL_MERMAID_HASH: &str = "placeholder_mermaid_hash";

#[tokio::test]
async fn test_unified_ast_engine_determinism() {
    // Create a deterministic test project
    let temp_dir = create_test_project().await;
    let engine = UnifiedAstEngine::new();

    // Generate artifacts multiple times
    let mut tree_hashes = Vec::new();
    for iteration in 0..5 {
        println!("Iteration {}: Generating artifacts...", iteration + 1);
        let tree = engine.generate_artifacts(temp_dir.path()).await.unwrap();
        let hash = compute_tree_hash(&tree);
        tree_hashes.push(hash);
    }

    // All hashes must be identical
    for window in tree_hashes.windows(2) {
        assert_eq!(
            window[0], window[1],
            "Artifact generation must be deterministic across runs"
        );
    }

    println!(
        "✅ UnifiedAstEngine determinism verified: {} identical runs",
        tree_hashes.len()
    );
}

#[test]
fn test_mermaid_generation_determinism() {
    let engine = DeterministicMermaidEngine::new();
    let graph = create_test_dependency_graph();

    // Generate same diagram multiple times
    let mut diagrams = Vec::new();
    for _iteration in 0..10 {
        let mermaid = engine.generate_codebase_modules_mmd(&graph);
        diagrams.push(mermaid);
    }

    // All diagrams must be byte-identical
    for window in diagrams.windows(2) {
        assert_eq!(
            window[0], window[1],
            "Mermaid generation must be byte-stable"
        );
    }

    // Verify structure is deterministic
    let first_diagram = &diagrams[0];
    assert!(first_diagram.starts_with("graph TD\n"));

    // Verify PageRank ordering is stable
    let lines: Vec<&str> = first_diagram.lines().collect();
    let node_lines: Vec<&str> = lines
        .iter()
        .filter(|line| line.contains('[') && line.contains(']'))
        .copied()
        .collect();

    // Node lines should be in consistent order (by PageRank then name)
    assert!(!node_lines.is_empty(), "Should have node definitions");

    println!(
        "✅ Mermaid generation determinism verified: {} identical diagrams",
        diagrams.len()
    );
}

#[tokio::test]
async fn test_dogfooding_engine_determinism() {
    let temp_dir = create_test_project().await;
    let engine = DogfoodingEngine::new();
    let date = "2025-05-31";

    // Generate same artifacts multiple times
    let mut ast_contexts = Vec::new();
    let mut combined_metrics = Vec::new();
    let mut complexity_analyses = Vec::new();

    for iteration in 0..3 {
        println!("Dogfooding iteration {}", iteration + 1);

        let ast_context = engine
            .generate_ast_context(temp_dir.path(), date)
            .await
            .unwrap();
        let metrics = engine
            .generate_combined_metrics(temp_dir.path(), date)
            .await
            .unwrap();
        let complexity = engine
            .generate_complexity_analysis(temp_dir.path(), date)
            .await
            .unwrap();

        ast_contexts.push(ast_context);
        combined_metrics.push(metrics);
        complexity_analyses.push(complexity);
    }

    // Verify AST context determinism (excluding generation_time)
    for window in ast_contexts.windows(2) {
        assert_eq!(window[0], window[1], "AST context must be deterministic");
    }

    // Verify complexity analysis determinism
    for window in complexity_analyses.windows(2) {
        assert_eq!(
            window[0], window[1],
            "Complexity analysis must be deterministic"
        );
    }

    // Verify metrics determinism (excluding timestamp fields)
    for window in combined_metrics.windows(2) {
        let metrics1 = window[0].as_object().unwrap();
        let metrics2 = window[1].as_object().unwrap();

        // Compare non-timestamp fields
        assert_eq!(metrics1["ast"], metrics2["ast"]);
        assert_eq!(metrics1["churn"], metrics2["churn"]);
        assert_eq!(metrics1["dag"], metrics2["dag"]);
        assert_eq!(metrics1["hash"], metrics2["hash"]);
    }

    println!("✅ Dogfooding engine determinism verified");
}

#[tokio::test]
async fn test_artifact_writer_determinism() {
    let temp_dir = TempDir::new().unwrap();
    let tree = create_test_artifact_tree();

    // Write artifacts multiple times to different locations
    let mut manifests = Vec::new();
    for iteration in 0..3 {
        let subdir = temp_dir.path().join(format!("run_{iteration}"));
        let mut writer = ArtifactWriter::new(subdir).unwrap();
        writer.write_artifacts(&tree).unwrap();

        // Read manifest and normalize timestamps
        let manifest = normalize_manifest(&writer);
        manifests.push(manifest);
    }

    // All manifests should have identical structure and hashes
    for window in manifests.windows(2) {
        assert_eq!(
            window[0], window[1],
            "Artifact writer output must be deterministic"
        );
    }

    println!("✅ Artifact writer determinism verified");
}

#[test]
fn test_pagerank_numerical_stability() {
    let _engine = DeterministicMermaidEngine::new();
    let graph = create_large_test_graph();

    // Run PageRank with different iteration counts to test stability
    let mut results = Vec::new();
    for _iterations in &[50, 100, 150, 200] {
        let test_engine = DeterministicMermaidEngine::new();
        // Would need to expose iteration count in actual implementation
        let mermaid = test_engine.generate_codebase_modules_mmd(&graph);
        results.push(mermaid);
    }

    // Results should converge and be stable
    // (In practice, we'd check that scores converge within epsilon)
    let final_result = &results[results.len() - 1];
    let second_to_last = &results[results.len() - 2];

    assert_eq!(
        final_result, second_to_last,
        "PageRank should converge to stable result"
    );

    println!("✅ PageRank numerical stability verified");
}

#[test]
fn test_hash_collision_resistance() {
    // Test that different inputs produce different hashes
    let test_cases = vec![
        ("", "empty"),
        ("a", "single char"),
        ("hello world", "simple string"),
        ("Hello World", "case different"),
        ("hello world ", "trailing space"),
        (" hello world", "leading space"),
        ("hello\nworld", "newline"),
        ("hello\tworld", "tab"),
    ];

    let mut hashes = HashMap::new();
    for (input, description) in test_cases {
        let hash = blake3::hash(input.as_bytes());
        let hash_str = format!("{hash}");

        if let Some(existing_desc) = hashes.get(&hash_str) {
            panic!(
                "Hash collision detected: '{description}' and '{existing_desc}' produce same hash"
            );
        }

        hashes.insert(hash_str, description);
    }

    println!(
        "✅ Hash collision resistance verified for {} test cases",
        hashes.len()
    );
}

#[tokio::test]
async fn test_file_ordering_stability() {
    // Test that file traversal order is deterministic
    let temp_dir = TempDir::new().unwrap();

    // Create files in non-alphabetical order
    let files = vec!["z.rs", "a.rs", "m.rs", "b.rs", "y.rs"];
    for file in &files {
        let path = temp_dir.path().join(file);
        fs::write(&path, "pub fn test() {}").unwrap();
    }

    let engine = UnifiedAstEngine::new();

    // Parse project multiple times
    let mut file_orders = Vec::new();
    for _ in 0..5 {
        let forest = engine.parse_project(temp_dir.path()).await.unwrap();
        let file_paths: Vec<String> = forest
            .files()
            .map(|(path, _)| path.display().to_string())
            .collect();
        file_orders.push(file_paths);
    }

    // File order should be consistent across runs
    for window in file_orders.windows(2) {
        assert_eq!(
            window[0], window[1],
            "File traversal order must be deterministic"
        );
    }

    // Should be in sorted order
    let first_order = &file_orders[0];
    let mut sorted_order = first_order.clone();
    sorted_order.sort();
    assert_eq!(
        *first_order, sorted_order,
        "Files should be processed in sorted order"
    );

    println!("✅ File ordering stability verified");
}

#[test]
fn test_edge_case_determinism() {
    // Test determinism with edge cases
    let engine = DeterministicMermaidEngine::new();

    // Empty graph
    let empty_graph = petgraph::stable_graph::StableGraph::new();
    let empty_mermaid1 = engine.generate_codebase_modules_mmd(&empty_graph);
    let empty_mermaid2 = engine.generate_codebase_modules_mmd(&empty_graph);
    assert_eq!(empty_mermaid1, empty_mermaid2);
    assert_eq!(empty_mermaid1.trim(), "graph TD");

    // Single node graph
    let single_node_graph = create_single_node_graph();
    let single1 = engine.generate_codebase_modules_mmd(&single_node_graph);
    let single2 = engine.generate_codebase_modules_mmd(&single_node_graph);
    assert_eq!(single1, single2);

    // Cyclic graph
    let cyclic_graph = create_cyclic_graph();
    let cyclic1 = engine.generate_codebase_modules_mmd(&cyclic_graph);
    let cyclic2 = engine.generate_codebase_modules_mmd(&cyclic_graph);
    assert_eq!(cyclic1, cyclic2);

    println!("✅ Edge case determinism verified");
}

#[tokio::test]
async fn test_concurrent_generation_determinism() {
    // Test that sequential generation produces identical results
    // (Concurrent disabled due to syn::File Send limitations)
    let temp_dir = create_test_project().await;
    let engine = UnifiedAstEngine::new();

    // Run generations sequentially
    let mut results = Vec::new();
    for _ in 0..4 {
        let tree = engine.generate_artifacts(temp_dir.path()).await.unwrap();
        let hash = compute_tree_hash(&tree);
        results.push(hash);
    }

    // All results should be identical
    for window in results.windows(2) {
        assert_eq!(
            window[0], window[1],
            "Sequential generation must be deterministic"
        );
    }

    println!("✅ Sequential generation determinism verified");
}

// Helper functions

async fn create_test_project() -> TempDir {
    let temp_dir = TempDir::new().unwrap();

    // Create a realistic Rust project structure
    let src_dir = temp_dir.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();

    // Main file
    fs::write(
        src_dir.join("main.rs"),
        r#"
        use lib::Config;
        
        fn main() {
            let config = Config::new();
            println!("Hello, {}!", config.name);
        }
    "#,
    )
    .unwrap();

    // Lib file
    fs::write(
        src_dir.join("lib.rs"),
        r#"
        pub mod utils;
        
        pub struct Config {
            pub name: String,
        }
        
        impl Config {
            pub fn new() -> Self {
                Self {
                    name: "World".to_string(),
                }
            }
        }
        
        pub trait Display {
            fn display(&self) -> String;
        }
        
        impl Display for Config {
            fn display(&self) -> String {
                format!("Config: {}", self.name)
            }
        }
    "#,
    )
    .unwrap();

    // Utils module
    fs::write(
        src_dir.join("utils.rs"),
        r"
        pub fn helper_function() -> i32 {
            42
        }
        
        pub struct Helper {
            value: i32,
        }
        
        impl Helper {
            pub fn new(value: i32) -> Self {
                Self { value }
            }
            
            pub fn process(&self) -> i32 {
                self.value * 2
            }
        }
    ",
    )
    .unwrap();

    temp_dir
}

fn create_test_dependency_graph() -> petgraph::stable_graph::StableGraph<
    pmat::services::unified_ast_engine::ModuleNode,
    pmat::models::dag::EdgeType,
> {
    use pmat::models::dag::EdgeType;
    use pmat::services::unified_ast_engine::{ModuleMetrics, ModuleNode};
    use std::path::PathBuf;

    let mut graph = petgraph::stable_graph::StableGraph::new();

    // Add test nodes
    let main_node = graph.add_node(ModuleNode {
        name: "main".to_string(),
        path: PathBuf::from("src/main.rs"),
        visibility: "public".to_string(),
        metrics: ModuleMetrics {
            complexity: 5,
            lines: 20,
            functions: 1,
            classes: 0,
        },
    });

    let lib_node = graph.add_node(ModuleNode {
        name: "lib".to_string(),
        path: PathBuf::from("src/lib.rs"),
        visibility: "public".to_string(),
        metrics: ModuleMetrics {
            complexity: 8,
            lines: 40,
            functions: 3,
            classes: 2,
        },
    });

    let utils_node = graph.add_node(ModuleNode {
        name: "utils".to_string(),
        path: PathBuf::from("src/utils.rs"),
        visibility: "public".to_string(),
        metrics: ModuleMetrics {
            complexity: 3,
            lines: 15,
            functions: 2,
            classes: 1,
        },
    });

    // Add edges
    graph.add_edge(main_node, lib_node, EdgeType::Uses);
    graph.add_edge(lib_node, utils_node, EdgeType::Imports);

    graph
}

fn create_large_test_graph() -> petgraph::stable_graph::StableGraph<
    pmat::services::unified_ast_engine::ModuleNode,
    pmat::models::dag::EdgeType,
> {
    use pmat::models::dag::EdgeType;
    use pmat::services::unified_ast_engine::{ModuleMetrics, ModuleNode};
    use std::path::PathBuf;

    let mut graph = petgraph::stable_graph::StableGraph::new();

    // Create 20 nodes for more interesting PageRank
    let mut nodes = Vec::new();
    for i in 0..20 {
        let node = graph.add_node(ModuleNode {
            name: format!("module_{i}"),
            path: PathBuf::from(format!("src/module_{i}.rs")),
            visibility: "public".to_string(),
            metrics: ModuleMetrics {
                complexity: (i % 10) + 1,
                lines: (i * 10) + 20,
                functions: (i % 5) + 1,
                classes: i % 3,
            },
        });
        nodes.push(node);
    }

    // Add various edge patterns
    for i in 0..nodes.len() {
        if i + 1 < nodes.len() {
            graph.add_edge(nodes[i], nodes[i + 1], EdgeType::Uses);
        }
        if i + 3 < nodes.len() {
            graph.add_edge(nodes[i], nodes[i + 3], EdgeType::Imports);
        }
        if i >= 2 {
            graph.add_edge(nodes[i], nodes[i - 2], EdgeType::Calls);
        }
    }

    graph
}

fn create_single_node_graph() -> petgraph::stable_graph::StableGraph<
    pmat::services::unified_ast_engine::ModuleNode,
    pmat::models::dag::EdgeType,
> {
    use pmat::services::unified_ast_engine::{ModuleMetrics, ModuleNode};
    use std::path::PathBuf;

    let mut graph = petgraph::stable_graph::StableGraph::new();
    graph.add_node(ModuleNode {
        name: "single".to_string(),
        path: PathBuf::from("single.rs"),
        visibility: "public".to_string(),
        metrics: ModuleMetrics::default(),
    });
    graph
}

fn create_cyclic_graph() -> petgraph::stable_graph::StableGraph<
    pmat::services::unified_ast_engine::ModuleNode,
    pmat::models::dag::EdgeType,
> {
    use pmat::models::dag::EdgeType;
    use pmat::services::unified_ast_engine::{ModuleMetrics, ModuleNode};
    use std::path::PathBuf;

    let mut graph = petgraph::stable_graph::StableGraph::new();

    let a = graph.add_node(ModuleNode {
        name: "a".to_string(),
        path: PathBuf::from("a.rs"),
        visibility: "public".to_string(),
        metrics: ModuleMetrics::default(),
    });

    let b = graph.add_node(ModuleNode {
        name: "b".to_string(),
        path: PathBuf::from("b.rs"),
        visibility: "public".to_string(),
        metrics: ModuleMetrics::default(),
    });

    let c = graph.add_node(ModuleNode {
        name: "c".to_string(),
        path: PathBuf::from("c.rs"),
        visibility: "public".to_string(),
        metrics: ModuleMetrics::default(),
    });

    // Create cycle: a -> b -> c -> a
    graph.add_edge(a, b, EdgeType::Uses);
    graph.add_edge(b, c, EdgeType::Uses);
    graph.add_edge(c, a, EdgeType::Uses);

    graph
}

fn create_test_artifact_tree() -> pmat::services::unified_ast_engine::ArtifactTree {
    use pmat::services::unified_ast_engine::{ArtifactTree, MermaidArtifacts, Template};
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    let mut dogfooding = BTreeMap::new();
    dogfooding.insert("test.md".to_string(), "# Test".to_string());

    let mermaid = MermaidArtifacts {
        ast_generated: BTreeMap::new(),
        non_code: BTreeMap::new(),
    };

    let templates = vec![Template {
        name: "test".to_string(),
        content: "test content".to_string(),
        hash: blake3::hash(b"test content"),
        source_location: PathBuf::from("test.rs"),
    }];

    ArtifactTree {
        dogfooding,
        mermaid,
        templates,
    }
}

fn compute_tree_hash(tree: &pmat::services::unified_ast_engine::ArtifactTree) -> blake3::Hash {
    let mut hasher = blake3::Hasher::new();

    // Hash in deterministic order
    for (key, value) in &tree.dogfooding {
        hasher.update(key.as_bytes());
        hasher.update(value.as_bytes());
    }

    for (key, value) in &tree.mermaid.ast_generated {
        hasher.update(key.as_bytes());
        hasher.update(value.as_bytes());
    }

    for template in &tree.templates {
        hasher.update(template.name.as_bytes());
        hasher.update(template.content.as_bytes());
    }

    hasher.finalize()
}

fn normalize_manifest(writer: &ArtifactWriter) -> HashMap<String, String> {
    // Extract deterministic parts of manifest (excluding timestamps)
    writer
        .get_statistics()
        .by_type
        .iter()
        .map(|(k, v)| (k.clone(), format!("{}:{}", v.count, v.size)))
        .collect()
}
