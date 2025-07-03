#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::{Arbitrary, Unstructured};
use pmat::services::context::{ProjectContext, FileContext, AstItem, ProjectSummary};
use pmat::services::dag_builder::DagBuilder;
use pmat::models::dag::EdgeType;

#[derive(Arbitrary, Debug)]
struct FuzzProject {
    files: Vec<FuzzFile>,
}

#[derive(Arbitrary, Debug)]
struct FuzzFile {
    path: String,
    items: Vec<FuzzAstItem>,
}

#[derive(Arbitrary, Debug)]
enum FuzzAstItem {
    Function {
        name: String,
        visibility: FuzzVisibility,
        is_async: bool,
    },
    Struct {
        name: String,
        visibility: FuzzVisibility,
        fields_count: u8,
    },
    Trait {
        name: String,
        visibility: FuzzVisibility,
    },
    Module {
        name: String,
        visibility: FuzzVisibility,
    },
    Impl {
        type_name: String,
        trait_name: Option<String>,
    },
    Use {
        path: String,
    },
}

#[derive(Arbitrary, Debug)]
enum FuzzVisibility {
    Public,
    Private,
    Crate,
}

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    if let Ok(input) = FuzzProject::arbitrary(&mut u) {
        let project = convert_to_project_context(input);
        
        // This should never panic
        let graph = DagBuilder::build_from_project(&project);
        
        // Verify invariants
        assert_dag_invariants(&graph);
    }
});

fn convert_to_project_context(fuzz_project: FuzzProject) -> ProjectContext {
    let files = fuzz_project.files.into_iter()
        .map(convert_file)
        .collect::<Vec<_>>();
    
    let mut summary = ProjectSummary {
        total_files: files.len(),
        total_functions: 0,
        total_structs: 0,
        total_enums: 0,
        total_traits: 0,
        total_impls: 0,
        dependencies: vec![],
    };
    
    // Count items for summary
    for file in &files {
        for item in &file.items {
            match item {
                AstItem::Function { .. } => summary.total_functions += 1,
                AstItem::Struct { .. } => summary.total_structs += 1,
                AstItem::Trait { .. } => summary.total_traits += 1,
                AstItem::Impl { .. } => summary.total_impls += 1,
                _ => {}
            }
        }
    }
    
    ProjectContext {
        project_type: "rust".to_string(),
        files,
        summary,
    }
}

fn convert_file(fuzz_file: FuzzFile) -> FileContext {
    let items = fuzz_file.items.into_iter()
        .enumerate()
        .map(|(line, item)| convert_ast_item(item, line))
        .collect();
    
    FileContext {
        path: sanitize_path(&fuzz_file.path),
        language: "rust".to_string(),
        items,
        complexity_metrics: None,
    }
}

fn sanitize_path(path: &str) -> String {
    if path.is_empty() {
        "empty.rs".to_string()
    } else {
        // Ensure valid file path
        let sanitized = path.chars()
            .take(200) // Limit length
            .filter(|c| c.is_alphanumeric() || *c == '/' || *c == '.' || *c == '_' || *c == '-')
            .collect::<String>();
        
        if sanitized.is_empty() {
            "file.rs".to_string()
        } else if !sanitized.ends_with(".rs") {
            format!("{}.rs", sanitized)
        } else {
            sanitized
        }
    }
}

fn convert_ast_item(fuzz_item: FuzzAstItem, line: usize) -> AstItem {
    match fuzz_item {
        FuzzAstItem::Function { name, visibility, is_async } => {
            AstItem::Function {
                name: sanitize_name(&name),
                visibility: convert_visibility(visibility),
                is_async,
                line: line + 1,
            }
        }
        FuzzAstItem::Struct { name, visibility, fields_count } => {
            AstItem::Struct {
                name: sanitize_name(&name),
                visibility: convert_visibility(visibility),
                fields_count: fields_count as usize,
                derives: vec![],
                line: line + 1,
            }
        }
        FuzzAstItem::Trait { name, visibility } => {
            AstItem::Trait {
                name: sanitize_name(&name),
                visibility: convert_visibility(visibility),
                line: line + 1,
            }
        }
        FuzzAstItem::Module { name, visibility } => {
            AstItem::Module {
                name: sanitize_name(&name),
                visibility: convert_visibility(visibility),
                line: line + 1,
            }
        }
        FuzzAstItem::Impl { type_name, trait_name } => {
            AstItem::Impl {
                type_name: sanitize_name(&type_name),
                trait_name: trait_name.map(|n| sanitize_name(&n)),
                line: line + 1,
            }
        }
        FuzzAstItem::Use { path } => {
            AstItem::Use {
                path: sanitize_path_import(&path),
                line: line + 1,
            }
        }
    }
}

fn sanitize_name(name: &str) -> String {
    if name.is_empty() {
        "empty".to_string()
    } else {
        name.chars()
            .take(100)
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>()
            .trim_start_matches(|c: char| c.is_numeric())
            .to_string()
    }
}

fn sanitize_path_import(path: &str) -> String {
    if path.is_empty() {
        "empty".to_string()
    } else {
        path.chars()
            .take(200)
            .filter(|c| c.is_alphanumeric() || *c == ':' || *c == '_')
            .collect()
    }
}

fn convert_visibility(vis: FuzzVisibility) -> String {
    match vis {
        FuzzVisibility::Public => "pub".to_string(),
        FuzzVisibility::Private => "private".to_string(),
        FuzzVisibility::Crate => "pub(crate)".to_string(),
    }
}

fn assert_dag_invariants(graph: &pmat::models::dag::DependencyGraph) {
    // All edges should reference existing nodes
    for edge in &graph.edges {
        assert!(
            graph.nodes.contains_key(&edge.from) || edge.edge_type == EdgeType::Imports,
            "Edge references non-existent 'from' node: {}",
            edge.from
        );
        
        // Import edges might reference external modules
        if edge.edge_type != EdgeType::Imports {
            assert!(
                graph.nodes.contains_key(&edge.to),
                "Edge references non-existent 'to' node: {}",
                edge.to
            );
        }
    }
    
    // All node IDs should be properly formatted
    for (id, node) in &graph.nodes {
        assert_eq!(id, &node.id, "Node ID mismatch");
        assert!(!node.label.is_empty(), "Empty node label");
    }
    
    // No self-loops (except in special cases)
    for edge in &graph.edges {
        if edge.from == edge.to {
            // Self-loops might be valid for certain edge types
            assert!(
                edge.edge_type == EdgeType::Calls,
                "Invalid self-loop with edge type: {:?}",
                edge.edge_type
            );
        }
    }
}