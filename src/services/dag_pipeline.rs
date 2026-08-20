//! The order in which a `--dag-type` graph has to be assembled.
//!
//! #1020: every surface used to build the graph like this —
//!
//! 1. build the graph AND immediately cut it down to the 400-edge Mermaid budget
//!    (which also deletes every node the surviving edges do not touch),
//! 2. walk the sources for `Calls` edges,
//! 3. filter to the requested edge type.
//!
//! Step 1 deletes all `Function` nodes on any tree with more than 400
//! import/inheritance edges, so step 2 had nothing to start from and step 3
//! returned an empty graph. `analyze_dag {dag_type: "call-graph"}` over
//! `src/services` answered `0 nodes, 0 edges` while `full-dependency` over the
//! identical path answered `369 nodes, 400 edges`, and `pmat analyze dag
//! --dag-type call-graph` over the same directory drew nothing either. A
//! 10-file fixture stayed under the budget, which is why the bug looked like it
//! did not exist.
//!
//! The order here is: complete graph -> enrich -> select -> *then* budget. The
//! budget is a rendering concern and must be the last thing that happens, to the
//! graph the caller actually asked for.

use crate::models::dag::{DependencyGraph, EdgeType, NodeType};
use crate::services::context::ProjectContext;
use std::path::Path;

/// What the complete (pre-budget, pre-filter) graph contained.
///
/// An empty answer has to be explainable: these are the numbers that say whether
/// "no call edges" means "no functions were found", "no calls resolved", or
/// "this tree is not Rust".
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DagBuildStats {
    /// Files the AST phase produced a context for.
    pub files_analyzed: usize,
    /// `Function` nodes in the complete graph.
    pub function_nodes: usize,
    /// `Function` nodes backed by a Rust source — the only ones call-edge
    /// extraction can currently walk.
    pub rust_function_nodes: usize,
    /// `Calls` edges resolved over the complete graph.
    pub call_edges: usize,
    /// Nodes/edges of the complete graph, before filtering and budgeting.
    pub total_nodes: usize,
    /// Edges of the complete graph, before filtering and budgeting.
    pub total_edges: usize,
}

impl DagBuildStats {
    /// Explain, in one sentence, why the requested graph came back empty.
    ///
    /// Returns `None` when the graph is not empty — there is nothing to excuse.
    #[must_use]
    pub fn explain_empty(
        &self,
        graph: &DependencyGraph,
        edge_types: Option<&[EdgeType]>,
    ) -> Option<String> {
        if !graph.nodes.is_empty() || !graph.edges.is_empty() {
            return None;
        }

        let wanted = match edge_types {
            None => "full-dependency".to_string(),
            Some(types) => types
                .iter()
                .map(|t| format!("{t:?}"))
                .collect::<Vec<_>>()
                .join("/"),
        };

        Some(if self.files_analyzed == 0 {
            format!("no source file under this path could be parsed, so no {wanted} graph exists")
        } else if self.function_nodes == 0 {
            format!(
                "{} files parsed but no function declarations were found, so there is nothing a {wanted} graph could connect",
                self.files_analyzed
            )
        } else if edge_types.is_some_and(|t| t.contains(&EdgeType::Calls))
            && self.rust_function_nodes == 0
        {
            format!(
                "{} functions found across {} files, but none are Rust: call-edge extraction only supports Rust sources, so no call graph can be produced for this tree",
                self.function_nodes, self.files_analyzed
            )
        } else {
            format!(
                "{} functions across {} files produced {} edges in total, none of them {wanted}",
                self.function_nodes, self.files_analyzed, self.total_edges
            )
        })
    }
}

/// Build the dependency graph the caller asked for.
///
/// `edge_types` is `None` for a full-dependency graph, or the edge types to keep.
/// `root` locates sources whose recorded path is relative.
pub async fn build_typed_dag(
    project: &ProjectContext,
    root: &Path,
    edge_types: Option<&[EdgeType]>,
) -> (DependencyGraph, DagBuildStats) {
    use crate::services::dag_builder::{add_pagerank_scores, enforce_edge_budget, DagBuilder};

    // 1. The COMPLETE graph. Budgeting here is what destroyed the call graph.
    let mut graph = DagBuilder::build_from_project_unbudgeted(project);

    // 2. Enrich. Call edges need the function bodies, which only the sources
    //    carry, and they need every function node to still be present.
    let wants_calls = edge_types.is_none_or(|types| types.contains(&EdgeType::Calls));
    let call_edges = if wants_calls {
        crate::services::dag_call_edges::add_call_edges(&mut graph, root)
    } else {
        0
    };

    let stats = DagBuildStats {
        files_analyzed: project.files.len(),
        function_nodes: graph
            .nodes
            .values()
            .filter(|n| n.node_type == NodeType::Function)
            .count(),
        rust_function_nodes: graph
            .nodes
            .values()
            .filter(|n| n.node_type == NodeType::Function && n.file_path.ends_with(".rs"))
            .count(),
        call_edges,
        total_nodes: graph.nodes.len(),
        total_edges: graph.edges.len(),
    };

    // 3. Select what was asked for.
    let graph = match edge_types {
        Some(types) => graph.filter_by_edge_types(types),
        None => graph,
    };

    // 4. Only now apply the rendering budget, and score what survived.
    let mut graph = add_pagerank_scores(enforce_edge_budget(graph));

    // 5. Give the surviving function nodes the complexity the complexity
    //    analyzer reports for them, rather than a placeholder.
    crate::services::dag_complexity::annotate_function_complexity(&mut graph, root).await;

    (graph, stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_graph() -> DependencyGraph {
        DependencyGraph::new()
    }

    #[test]
    fn test_explain_empty_is_silent_for_a_non_empty_graph() {
        use crate::models::dag::NodeInfo;
        use rustc_hash::FxHashMap;

        let mut graph = empty_graph();
        graph.add_node(NodeInfo {
            id: "a".to_string(),
            label: "a".to_string(),
            node_type: NodeType::Function,
            file_path: "a.rs".to_string(),
            line_number: 1,
            complexity: 1,
            metadata: FxHashMap::default(),
        });
        let stats = DagBuildStats::default();
        assert!(stats
            .explain_empty(&graph, Some(&[EdgeType::Calls]))
            .is_none());
    }

    #[test]
    fn test_explain_empty_names_unparsed_tree() {
        let stats = DagBuildStats::default();
        let message = stats
            .explain_empty(&empty_graph(), Some(&[EdgeType::Calls]))
            .expect("an empty graph must be explained");
        assert!(message.contains("no source file"), "{message}");
    }

    #[test]
    fn test_explain_empty_names_non_rust_tree() {
        let stats = DagBuildStats {
            files_analyzed: 12,
            function_nodes: 30,
            rust_function_nodes: 0,
            ..DagBuildStats::default()
        };
        let message = stats
            .explain_empty(&empty_graph(), Some(&[EdgeType::Calls]))
            .expect("an empty graph must be explained");
        assert!(message.contains("only supports Rust sources"), "{message}");
    }

    #[test]
    fn test_explain_empty_reports_edge_shortfall() {
        let stats = DagBuildStats {
            files_analyzed: 12,
            function_nodes: 30,
            rust_function_nodes: 30,
            total_edges: 44,
            ..DagBuildStats::default()
        };
        let message = stats
            .explain_empty(&empty_graph(), Some(&[EdgeType::Inherits]))
            .expect("an empty graph must be explained");
        assert!(message.contains("44 edges in total"), "{message}");
    }
}
