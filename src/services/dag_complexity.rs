//! Fill a dependency graph's function nodes with the complexity that
//! `analyze complexity` reports for the same function.
//!
//! #1020: `NodeInfo::complexity` was populated from `FileContext::complexity_metrics`,
//! which the project-AST path the DAG is built from leaves `None` — so every
//! function node in every graph came back `complexity: 1`, and the MCP
//! `analyze_dag` tool reported 1 for a function `analyze_complexity` scored 7
//! **in the same server process**. Worse, on the rare path where the metrics
//! *were* present the builder stored `cognitive` under a field the consumer
//! reads as cyclomatic, so agreement was impossible by construction.
//!
//! The fix is not another complexity counter: it is to ask the one this project
//! already treats as authoritative. [`analyze_file_complexity_uncached`] is the
//! exact entry point behind both `pmat analyze complexity` and the MCP
//! `analyze_complexity` tool, so a node annotated here cannot disagree with them.
//!
//! Nodes whose function cannot be located unambiguously are left alone and
//! marked `complexity_source: not-measured`: a graph must be able to say "nobody
//! measured this" instead of quoting a placeholder as a measurement.

use crate::models::dag::{DependencyGraph, NodeType};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Metadata key recording where a node's `complexity` came from.
pub const COMPLEXITY_SOURCE_KEY: &str = "complexity_source";
/// `complexity_source` value for a node carrying a real McCabe measurement.
pub const SOURCE_CYCLOMATIC: &str = "cyclomatic";
/// `complexity_source` value for a node nobody could measure.
pub const SOURCE_NOT_MEASURED: &str = "not-measured";

/// Where this node's `complexity` came from, defaulting to "nobody measured it".
///
/// A node with no provenance recorded has not been through
/// [`annotate_function_complexity`] — trait, module and type nodes never are —
/// so the honest answer for it is [`SOURCE_NOT_MEASURED`], never silence.
#[must_use]
pub fn complexity_source(node: &crate::models::dag::NodeInfo) -> &str {
    node.metadata
        .get(COMPLEXITY_SOURCE_KEY)
        .map_or(SOURCE_NOT_MEASURED, String::as_str)
}

/// The complexity a node can honestly report: `None` when nobody measured it.
///
/// `NodeInfo::complexity` is a non-optional `u32` that every construction site
/// has to fill in, so an unmeasured node carries the neutral weight 1 there.
/// Serialising that number next to `complexity_source: "not-measured"` puts a
/// measurement and a denial that any measurement exists in one object, and a
/// consumer reading `complexity` alone gets the value 1 — indistinguishable
/// from a function the analyzer really scored 1. This is the same shape #928
/// fixed: absence has to be REPRESENTABLE, so it is serialised as JSON `null`
/// rather than disguised as a placeholder.
#[must_use]
pub fn reported_complexity(node: &crate::models::dag::NodeInfo) -> Option<u32> {
    (complexity_source(node) != SOURCE_NOT_MEASURED).then_some(node.complexity)
}

/// Annotate every `Function` node in `graph` with its cyclomatic complexity.
///
/// `root` locates sources whose recorded path is relative. Returns the number of
/// nodes that received a measurement.
pub async fn annotate_function_complexity(graph: &mut DependencyGraph, root: &Path) -> usize {
    // No early return on an empty file list: a node whose source cannot be read
    // still has to be MARKED unmeasured, or it keeps quoting its placeholder.
    let measurements = measure_files(function_files(graph, root)).await;

    let mut annotated = 0;
    for node in graph.nodes.values_mut() {
        if node.node_type != NodeType::Function {
            continue;
        }
        let measured = resolve_path(&node.file_path, root)
            .and_then(|path| measurements.get(&path))
            .and_then(|functions| lookup(functions, &declared_name(node), node.line_number));

        match measured {
            Some(cyclomatic) => {
                node.complexity = u32::from(cyclomatic);
                node.metadata
                    .insert("complexity".to_string(), node.complexity.to_string());
                node.metadata.insert(
                    COMPLEXITY_SOURCE_KEY.to_string(),
                    SOURCE_CYCLOMATIC.to_string(),
                );
                annotated += 1;
            }
            None => {
                node.metadata.insert(
                    COMPLEXITY_SOURCE_KEY.to_string(),
                    SOURCE_NOT_MEASURED.to_string(),
                );
            }
        }
    }

    annotated
}

/// The name the function was DECLARED with.
///
/// `NodeInfo::label` is not it: `DagBuilder::enrich_node` overwrites the label
/// with `SemanticNamer`'s display name. The declared name survives as the tail
/// of the node id (`services_foo::branchy`), which is how the id was built.
fn declared_name(node: &crate::models::dag::NodeInfo) -> String {
    node.id
        .rsplit("::")
        .next()
        .unwrap_or(&node.label)
        .to_string()
}

/// Distinct on-disk files that back the graph's function nodes.
fn function_files(graph: &DependencyGraph, root: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = graph
        .nodes
        .values()
        .filter(|n| n.node_type == NodeType::Function)
        .filter_map(|n| resolve_path(&n.file_path, root))
        .collect();
    files.sort();
    files.dedup();
    files
}

fn resolve_path(file_path: &str, root: &Path) -> Option<PathBuf> {
    let direct = PathBuf::from(file_path);
    if direct.is_file() {
        return Some(direct);
    }
    let joined = root.join(file_path);
    joined.is_file().then_some(joined)
}

/// Run the project's complexity analyzer over each file, concurrently.
async fn measure_files(
    files: Vec<PathBuf>,
) -> HashMap<PathBuf, Vec<crate::services::complexity::FunctionComplexity>> {
    use crate::services::complexity::analyze_file_complexity_uncached;
    use futures::stream::{self, StreamExt};

    stream::iter(files)
        .map(|path| async move {
            let metrics = analyze_file_complexity_uncached(&path, None).await.ok()?;
            Some((path, metrics.functions))
        })
        .buffer_unordered(num_cpus::get())
        .filter_map(|entry| async move { entry })
        .collect::<HashMap<_, _>>()
        .await
}

/// Find `name`'s measurement, refusing to guess when the file declares the name
/// more than once and the node's line does not pick one out.
fn lookup(
    functions: &[crate::services::complexity::FunctionComplexity],
    name: &str,
    line_number: usize,
) -> Option<u16> {
    let candidates: Vec<&crate::services::complexity::FunctionComplexity> =
        functions.iter().filter(|f| f.name == name).collect();

    match candidates.as_slice() {
        [] => None,
        [only] => Some(only.metrics.cyclomatic),
        many => {
            let line = u32::try_from(line_number).ok()?;
            let mut spanning = many
                .iter()
                .filter(|f| f.line_start <= line && line <= f.line_end);
            let first = spanning.next()?;
            // Two overlapping declarations of the same name: no answer beats a
            // coin flip between them.
            spanning
                .next()
                .is_none()
                .then_some(first.metrics.cyclomatic)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::dag::NodeInfo;
    use crate::services::complexity::{ComplexityMetrics, FunctionComplexity};
    use rustc_hash::FxHashMap;

    fn metrics(cyclomatic: u16) -> ComplexityMetrics {
        ComplexityMetrics {
            cyclomatic,
            ..Default::default()
        }
    }

    fn func(name: &str, start: u32, end: u32, cyclomatic: u16) -> FunctionComplexity {
        FunctionComplexity {
            name: name.to_string(),
            line_start: start,
            line_end: end,
            metrics: metrics(cyclomatic),
        }
    }

    fn function_node(id: &str, label: &str, file: &str, line: usize) -> NodeInfo {
        NodeInfo {
            id: id.to_string(),
            label: label.to_string(),
            node_type: NodeType::Function,
            file_path: file.to_string(),
            line_number: line,
            complexity: 1,
            metadata: FxHashMap::default(),
        }
    }

    #[test]
    fn test_lookup_unique_name() {
        assert_eq!(lookup(&[func("a", 1, 9, 7)], "a", 3), Some(7));
    }

    #[test]
    fn test_lookup_unknown_name_is_none() {
        assert_eq!(lookup(&[func("a", 1, 9, 7)], "b", 3), None);
    }

    #[test]
    fn test_lookup_disambiguates_overloads_by_line() {
        let functions = vec![func("new", 1, 9, 2), func("new", 20, 29, 5)];
        assert_eq!(lookup(&functions, "new", 22), Some(5));
        assert_eq!(lookup(&functions, "new", 3), Some(2));
        // A line outside both spans picks neither.
        assert_eq!(lookup(&functions, "new", 50), None);
    }

    #[test]
    fn test_declared_name_survives_semantic_relabelling() {
        let mut node = function_node("services_foo::branchy", "branchy", "src/foo.rs", 1);
        node.label = "foo::branchy (renamed)".to_string();
        assert_eq!(declared_name(&node), "branchy");
    }

    /// The whole point: the number on the node is the number the complexity
    /// analyzer reports, not a constant.
    #[tokio::test]
    async fn test_annotate_matches_the_complexity_analyzer() {
        use crate::services::complexity::analyze_file_complexity_uncached;

        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("branchy.rs");
        std::fs::write(
            &file,
            r"
pub fn branchy(a: i32, b: i32) -> i32 {
    if a > 0 && b > 0 {
        return 1;
    }
    if a < 0 || b < 0 {
        return 2;
    }
    for i in 0..a {
        if i == b {
            return 3;
        }
    }
    match a {
        0 => 4,
        _ => 5,
    }
}
",
        )
        .expect("write");

        let expected = analyze_file_complexity_uncached(&file, None)
            .await
            .expect("analyzer")
            .functions
            .iter()
            .find(|f| f.name == "branchy")
            .expect("analyzer must see branchy")
            .metrics
            .cyclomatic;
        assert!(
            expected > 1,
            "fixture must be branchy enough to distinguish a measurement from the placeholder"
        );

        let mut graph = DependencyGraph::new();
        graph.add_node(function_node(
            "branchy_rs::branchy",
            "branchy",
            &file.display().to_string(),
            2,
        ));

        let annotated = annotate_function_complexity(&mut graph, dir.path()).await;

        assert_eq!(annotated, 1);
        let node = graph.nodes.values().next().expect("node");
        assert_eq!(u32::from(expected), node.complexity);
        assert_eq!(
            node.metadata.get(COMPLEXITY_SOURCE_KEY).map(String::as_str),
            Some(SOURCE_CYCLOMATIC)
        );
        assert_eq!(
            node.metadata.get("complexity").map(String::as_str),
            Some(node.complexity.to_string().as_str()),
            "metadata must not contradict the field"
        );
    }

    /// A node whose source is gone must SAY it was not measured, not report 1.
    #[tokio::test]
    async fn test_unmeasurable_node_is_labelled_not_measured() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut graph = DependencyGraph::new();
        graph.add_node(function_node(
            "missing::ghost",
            "ghost",
            "definitely/not/here.rs",
            1,
        ));

        assert_eq!(
            annotate_function_complexity(&mut graph, dir.path()).await,
            0
        );
        let node = graph.nodes.values().next().expect("node");
        assert_eq!(
            node.metadata.get(COMPLEXITY_SOURCE_KEY).map(String::as_str),
            Some(SOURCE_NOT_MEASURED)
        );
        // …and it must not be able to QUOTE that placeholder as a number.
        assert_eq!(reported_complexity(node), None);
    }

    /// A node with no provenance at all — trait, module and type nodes are
    /// never annotated — must read as unmeasured, not as silence.
    #[test]
    fn test_missing_provenance_reads_as_not_measured() {
        let mut node = function_node("m::t", "t", "src/t.rs", 1);
        node.node_type = NodeType::Trait;
        node.complexity = 1;

        assert_eq!(complexity_source(&node), SOURCE_NOT_MEASURED);
        assert_eq!(
            reported_complexity(&node),
            None,
            "complexity 1 with no provenance is the placeholder, not a measurement"
        );
    }

    /// The other direction: a node that WAS measured reports its number, so
    /// nulling everything is not a way to pass the test above.
    #[test]
    fn test_measured_node_reports_its_number() {
        let mut node = function_node("m::f", "f", "src/f.rs", 1);
        node.complexity = 7;
        node.metadata.insert(
            COMPLEXITY_SOURCE_KEY.to_string(),
            SOURCE_CYCLOMATIC.to_string(),
        );

        assert_eq!(complexity_source(&node), SOURCE_CYCLOMATIC);
        assert_eq!(reported_complexity(&node), Some(7));
    }

    /// `Option<u32>` is what makes the absence representable in JSON at all:
    /// `None` has to serialise as `null`, and `Some(1)` must stay a `1` that a
    /// consumer can tell apart from it.
    #[test]
    fn test_absence_serialises_as_null_and_one_stays_one() {
        let mut unmeasured = function_node("m::t", "t", "src/t.rs", 1);
        unmeasured.complexity = 1;
        let mut measured_one = function_node("m::g", "g", "src/g.rs", 1);
        measured_one.complexity = 1;
        measured_one.metadata.insert(
            COMPLEXITY_SOURCE_KEY.to_string(),
            SOURCE_CYCLOMATIC.to_string(),
        );

        assert_eq!(
            serde_json::json!(reported_complexity(&unmeasured)),
            serde_json::Value::Null
        );
        assert_eq!(
            serde_json::json!(reported_complexity(&measured_one)),
            serde_json::json!(1),
            "a function the analyzer really scored 1 must still report 1"
        );
    }
}
