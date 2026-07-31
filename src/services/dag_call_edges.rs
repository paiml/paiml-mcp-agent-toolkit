#![cfg_attr(coverage_nightly, coverage(off))]
//! Call-edge extraction for dependency graphs.
//!
//! Defect #653: `pmat analyze dag --dag-type call-graph` emitted a graph with zero
//! edges for every project, because nothing in the pipeline ever produced an
//! `EdgeType::Calls` edge — `DagBuilder` only ever looked at `use`/`impl` items, and
//! `build_context_graph` left edge extraction as a TODO. This module walks the Rust
//! sources that back the graph's function nodes and records the calls it can resolve
//! to a function node that actually exists.
//!
//! Resolution is deliberately conservative: a callee that matches several project
//! functions and cannot be disambiguated by file produces **no** edge, because a
//! guessed edge is worse than a missing one.

use crate::models::dag::{DependencyGraph, Edge, EdgeType, NodeType};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use syn::visit::Visit;

/// Add `EdgeType::Calls` edges to `graph` for every resolvable Rust call.
///
/// `root` is the analyzed project root, used to locate sources whose recorded
/// path is relative. Returns the number of edges added.
pub fn add_call_edges(graph: &mut DependencyGraph, root: &Path) -> usize {
    let functions_by_file = index_functions_by_file(graph);
    let functions_by_name = index_functions_by_name(graph);

    let existing: HashSet<(String, String)> = graph
        .edges
        .iter()
        .filter(|e| e.edge_type == EdgeType::Calls)
        .map(|e| (e.from.clone(), e.to.clone()))
        .collect();
    let mut seen = existing;
    let mut added = 0;

    for (file_path, local_functions) in &functions_by_file {
        let Some(source) = read_source(file_path, root) else {
            continue;
        };
        for (caller, callee) in collect_calls(&source) {
            let Some(from) = local_functions.get(&caller) else {
                continue;
            };
            let Some(to) = resolve_callee(&callee, local_functions, &functions_by_name) else {
                continue;
            };
            if from == &to || !seen.insert((from.clone(), to.clone())) {
                continue;
            }
            graph.add_edge(Edge {
                from: from.clone(),
                to,
                edge_type: EdgeType::Calls,
                weight: 1,
            });
            added += 1;
        }
    }

    added
}

/// file path -> (function label -> node id), ordered for deterministic output
fn index_functions_by_file(graph: &DependencyGraph) -> BTreeMap<String, HashMap<String, String>> {
    let mut by_file: BTreeMap<String, HashMap<String, String>> = BTreeMap::new();

    for node in graph.nodes.values() {
        if node.node_type != NodeType::Function || !node.file_path.ends_with(".rs") {
            continue;
        }
        by_file
            .entry(node.file_path.clone())
            .or_default()
            .insert(node.label.clone(), node.id.clone());
    }

    by_file
}

/// function label -> every node id declaring it (used to detect ambiguity)
fn index_functions_by_name(graph: &DependencyGraph) -> HashMap<String, Vec<String>> {
    let mut by_name: HashMap<String, Vec<String>> = HashMap::new();

    for node in graph.nodes.values() {
        if node.node_type == NodeType::Function {
            by_name
                .entry(node.label.clone())
                .or_default()
                .push(node.id.clone());
        }
    }

    by_name
}

fn read_source(file_path: &str, root: &Path) -> Option<String> {
    let direct = PathBuf::from(file_path);
    let candidate = if direct.exists() {
        direct
    } else {
        root.join(file_path)
    };
    std::fs::read_to_string(candidate).ok()
}

/// Prefer a callee declared in the same file; otherwise accept a project-wide
/// match only when it is unique.
fn resolve_callee(
    callee: &str,
    local_functions: &HashMap<String, String>,
    functions_by_name: &HashMap<String, Vec<String>>,
) -> Option<String> {
    if let Some(id) = local_functions.get(callee) {
        return Some(id.clone());
    }
    match functions_by_name.get(callee) {
        Some(candidates) if candidates.len() == 1 => Some(candidates[0].clone()),
        _ => None,
    }
}

/// Extract `(enclosing function, called function)` pairs from Rust source.
fn collect_calls(source: &str) -> Vec<(String, String)> {
    let Ok(file) = syn::parse_file(source) else {
        return Vec::new();
    };
    let mut collector = CallCollector::default();
    collector.visit_file(&file);
    collector.calls
}

#[derive(Default)]
struct CallCollector {
    enclosing: Vec<String>,
    calls: Vec<(String, String)>,
}

impl CallCollector {
    fn record(&mut self, callee: String) {
        if let Some(caller) = self.enclosing.last() {
            self.calls.push((caller.clone(), callee));
        }
    }
}

impl<'ast> Visit<'ast> for CallCollector {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.enclosing.push(node.sig.ident.to_string());
        syn::visit::visit_item_fn(self, node);
        self.enclosing.pop();
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.enclosing.push(node.sig.ident.to_string());
        syn::visit::visit_impl_item_fn(self, node);
        self.enclosing.pop();
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        // Only path calls (`helper_one()`, `helper::helper_one()`, `Self::new()`).
        // Method calls are intentionally ignored: receivers are untyped here, so
        // matching `x.build()` to a free function named `build` would fabricate edges.
        if let syn::Expr::Path(path) = node.func.as_ref() {
            if let Some(segment) = path.path.segments.last() {
                self.record(segment.ident.to_string());
            }
        }
        syn::visit::visit_expr_call(self, node);
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::dag::NodeInfo;
    use rustc_hash::FxHashMap;

    fn function_node(id: &str, label: &str, file: &str) -> NodeInfo {
        NodeInfo {
            id: id.to_string(),
            label: label.to_string(),
            node_type: NodeType::Function,
            file_path: file.to_string(),
            line_number: 1,
            complexity: 1,
            metadata: FxHashMap::default(),
        }
    }

    #[test]
    fn test_collect_calls_finds_direct_calls() {
        let calls = collect_calls("fn main() { helper_one(); helper_two(); }");
        assert_eq!(
            calls,
            vec![
                ("main".to_string(), "helper_one".to_string()),
                ("main".to_string(), "helper_two".to_string()),
            ]
        );
    }

    #[test]
    fn test_collect_calls_uses_last_path_segment() {
        let calls = collect_calls("fn main() { helper::helper_one(); }");
        assert_eq!(calls, vec![("main".to_string(), "helper_one".to_string())]);
    }

    #[test]
    fn test_collect_calls_ignores_method_calls() {
        let calls = collect_calls("fn main() { let x = 1; x.build(); }");
        assert!(calls.is_empty());
    }

    /// Regression test for #653: a project where main() calls helper_one() must
    /// produce a Calls edge, not the observed "0 edges".
    #[test]
    fn test_add_call_edges_produces_edges() {
        let dir = tempfile::tempdir().expect("tempdir");
        let main_rs = dir.path().join("main.rs");
        std::fs::write(&main_rs, "fn main() { helper_one(); }\n").expect("write");
        let helper_rs = dir.path().join("helper.rs");
        std::fs::write(&helper_rs, "pub fn helper_one() {}\n").expect("write");

        let mut graph = DependencyGraph::new();
        graph.add_node(function_node(
            "main_main",
            "main",
            &main_rs.display().to_string(),
        ));
        graph.add_node(function_node(
            "helper_helper_one",
            "helper_one",
            &helper_rs.display().to_string(),
        ));

        let added = add_call_edges(&mut graph, dir.path());

        assert_eq!(added, 1);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].from, "main_main");
        assert_eq!(graph.edges[0].to, "helper_helper_one");
        assert_eq!(graph.edges[0].edge_type, EdgeType::Calls);
    }

    /// output_derived_from_input: no sources, no edges.
    #[test]
    fn test_add_call_edges_empty_graph() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut graph = DependencyGraph::new();
        assert_eq!(add_call_edges(&mut graph, dir.path()), 0);
        assert!(graph.edges.is_empty());
    }

    /// An ambiguous callee must produce no edge rather than a guessed one.
    #[test]
    fn test_add_call_edges_skips_ambiguous_callees() {
        let dir = tempfile::tempdir().expect("tempdir");
        let caller_rs = dir.path().join("caller.rs");
        std::fs::write(&caller_rs, "fn go() { shared(); }\n").expect("write");
        let a_rs = dir.path().join("a.rs");
        std::fs::write(&a_rs, "pub fn shared() {}\n").expect("write");
        let b_rs = dir.path().join("b.rs");
        std::fs::write(&b_rs, "pub fn shared() {}\n").expect("write");

        let mut graph = DependencyGraph::new();
        graph.add_node(function_node(
            "caller_go",
            "go",
            &caller_rs.display().to_string(),
        ));
        graph.add_node(function_node(
            "a_shared",
            "shared",
            &a_rs.display().to_string(),
        ));
        graph.add_node(function_node(
            "b_shared",
            "shared",
            &b_rs.display().to_string(),
        ));

        assert_eq!(add_call_edges(&mut graph, dir.path()), 0);
        assert!(graph.edges.is_empty());
    }
}
