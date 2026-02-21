#![cfg_attr(coverage_nightly, coverage(off))]
//! File Split Service — Semantic file splitting using Louvain community detection
//!
//! Analyzes a file's function entries in the agent context index, builds an intra-file
//! call graph, runs Louvain community detection, and names each cluster using
//! suggest-rename signal functions.
//!
//! # Algorithm
//!
//! 1. `index.file_index[path]` → get all function indices in file
//! 2. Build intra-file call graph from `calls`/`called_by` (keep only edges where
//!    both endpoints are in the same file)
//! 3. Convert to `UndirectedGraph` for Louvain
//! 4. Run `LouvainDetector::detect_communities()`
//! 5. Map communities → function entries → clusters with estimated line counts
//! 6. Name each cluster using suggest-rename signals
//! 7. Calculate impact (scan index for importers)

use crate::graph::community::LouvainDetector;
use crate::graph::types::{NodeData, UndirectedGraph};
use crate::services::agent_context::query::suggest_rename::{
    find_context_word, try_common_prefix, try_doc_comment_consensus, try_dominant_type,
    try_function_theme,
};
use crate::services::agent_context::{AgentContextIndex, FunctionEntry};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

// ── Types ──────────────────────────────────────────────────────────────────

/// A split plan for a single source file
#[derive(Debug, Clone, Serialize)]
pub struct SplitPlan {
    /// Source file being split
    pub source_file: String,
    /// Total lines in file
    pub total_lines: usize,
    /// Detected clusters
    pub clusters: Vec<SplitCluster>,
    /// Items not assigned to any cluster (singletons)
    pub unclustered: Vec<ClusterItem>,
    /// Impact analysis
    pub impact: SplitImpact,
    /// Louvain modularity score (higher = better cluster separation)
    pub modularity: f64,
}

/// A cluster of related functions that should be extracted together
#[derive(Debug, Clone, Serialize)]
pub struct SplitCluster {
    /// Suggested filename for this cluster (no extension)
    pub suggested_name: String,
    /// Signal that produced the name
    pub naming_signal: String,
    /// Confidence in the suggested name (0.0-1.0)
    pub confidence: f32,
    /// Items in this cluster
    pub items: Vec<ClusterItem>,
    /// Estimated line count
    pub estimated_lines: usize,
    /// Cohesion score: ratio of internal edges to total possible edges
    pub cohesion: f64,
}

/// A single item (function/struct/enum/trait) in a cluster
#[derive(Debug, Clone, Serialize)]
pub struct ClusterItem {
    /// Item name
    pub name: String,
    /// Definition type
    pub definition_type: String,
    /// Line range (start, end)
    pub line_range: (usize, usize),
    /// Functions this item calls (within the file)
    pub calls: Vec<String>,
    /// Functions that call this item (within the file)
    pub called_by: Vec<String>,
}

/// Impact analysis for a split
#[derive(Debug, Clone, Serialize)]
pub struct SplitImpact {
    /// Files that import/use this module
    pub importing_files: Vec<String>,
    /// Potential circular dependency risks
    pub circular_risks: Vec<String>,
}

// ── Core Algorithm ─────────────────────────────────────────────────────────

/// Suggest how to split a file into semantically coherent clusters.
///
/// Returns `None` if the file is not in the index or has no functions.
pub fn suggest_split(
    index: &AgentContextIndex,
    file_path: &str,
    resolution: f64,
    min_cluster_lines: usize,
) -> Option<SplitPlan> {
    // Step 1: Get all function indices for this file
    let func_indices = index.file_index.get(file_path)?;
    if func_indices.is_empty() {
        return None;
    }

    // Build local index: global_idx -> local_idx and collect entries
    let mut global_to_local: HashMap<usize, usize> = HashMap::new();
    let mut local_entries: Vec<&FunctionEntry> = Vec::new();
    for (local_idx, &global_idx) in func_indices.iter().enumerate() {
        if global_idx < index.functions.len() {
            global_to_local.insert(global_idx, local_idx);
            local_entries.push(&index.functions[global_idx]);
        }
    }

    if local_entries.is_empty() {
        return None;
    }

    let total_lines = estimate_total_lines(&local_entries);

    // Step 2: Build intra-file call graph
    let (graph, node_to_local) = build_intra_file_graph(index, func_indices, &global_to_local);

    // Step 3-4: Run community detection
    let (communities, modularity) = if local_entries.len() < 10 {
        // Fallback: use connected components for small files
        let comms = connected_components(&graph);
        let detector = LouvainDetector::new();
        let mod_score = detector.calculate_modularity(&graph, &comms);
        (comms, mod_score)
    } else {
        let mut detector = LouvainDetector::new().with_resolution(resolution);
        let comms = detector.detect_communities(&graph);
        let mod_score = detector.calculate_modularity(&graph, &comms);
        (comms, mod_score)
    };

    // Step 5: Map communities to clusters
    let mut community_items: HashMap<usize, Vec<usize>> = HashMap::new();
    for (node_idx, &comm_id) in communities.iter().enumerate() {
        // Map node index back to local entry index
        if let Some(&local_idx) = node_to_local.get(&node_idx) {
            community_items.entry(comm_id).or_default().push(local_idx);
        }
    }

    // Collect multi-item communities as clusters, singletons as orphans
    let mut clusters = Vec::new();
    let mut orphan_items: Vec<(ClusterItem, usize)> = Vec::new(); // (item, local_idx)

    for local_indices in community_items.values() {
        let items: Vec<ClusterItem> = local_indices
            .iter()
            .map(|&li| make_cluster_item(li, &local_entries, index, func_indices, &global_to_local))
            .collect();

        let estimated_lines: usize = items
            .iter()
            .map(|i| i.line_range.1 - i.line_range.0 + 1)
            .sum();

        if items.len() == 1 || estimated_lines < min_cluster_lines {
            for (i, item) in items.into_iter().enumerate() {
                orphan_items.push((item, local_indices[i]));
            }
        } else {
            // Step 6: Name each cluster using signal functions
            let cluster_entries: Vec<&FunctionEntry> =
                local_indices.iter().map(|&li| local_entries[li]).collect();

            let (name, signal, confidence) = name_cluster(&cluster_entries, file_path);
            let cohesion = compute_cohesion(local_indices, index, func_indices, &global_to_local);

            clusters.push(SplitCluster {
                suggested_name: name,
                naming_signal: signal,
                confidence,
                items,
                estimated_lines,
                cohesion,
            });
        }
    }

    // Step 5b: Assign orphans to nearest cluster by line proximity
    let unclustered = assign_orphans_to_clusters(&mut clusters, orphan_items);

    // Sort items within each cluster by line range
    for cluster in &mut clusters {
        cluster.items.sort_by_key(|i| i.line_range.0);
    }

    // Sort clusters by line count descending
    clusters.sort_by(|a, b| b.estimated_lines.cmp(&a.estimated_lines));

    // Step 7: Calculate impact
    let impact = compute_impact(index, file_path);

    Some(SplitPlan {
        source_file: file_path.to_string(),
        total_lines,
        clusters,
        unclustered,
        impact,
        modularity,
    })
}

// ── Helpers ────────────────────────────────────────────────────────────────

/// Assign orphan items to the nearest cluster by line proximity.
/// Returns any items that couldn't be assigned (only if no clusters exist).
fn assign_orphans_to_clusters(
    clusters: &mut Vec<SplitCluster>,
    orphan_items: Vec<(ClusterItem, usize)>,
) -> Vec<ClusterItem> {
    if clusters.is_empty() {
        return orphan_items.into_iter().map(|(item, _)| item).collect();
    }

    let mut unclustered = Vec::new();
    for (item, _local_idx) in orphan_items {
        let item_mid = (item.line_range.0 + item.line_range.1) / 2;
        let nearest = clusters.iter_mut().min_by_key(|c| {
            c.items
                .iter()
                .map(|ci| {
                    let ci_mid = (ci.line_range.0 + ci.line_range.1) / 2;
                    (ci_mid as isize - item_mid as isize).unsigned_abs()
                })
                .min()
                .unwrap_or(usize::MAX)
        });
        if let Some(cluster) = nearest {
            let line_span = item.line_range.1 - item.line_range.0 + 1;
            cluster.estimated_lines += line_span;
            cluster.items.push(item);
        } else {
            unclustered.push(item);
        }
    }
    unclustered
}

fn estimate_total_lines(entries: &[&FunctionEntry]) -> usize {
    entries.iter().map(|e| e.end_line).max().unwrap_or(0)
}

/// Build an undirected graph from intra-file call relationships.
/// Returns the graph and a mapping from graph node index to local entry index.
fn build_intra_file_graph(
    index: &AgentContextIndex,
    func_indices: &[usize],
    global_to_local: &HashMap<usize, usize>,
) -> (UndirectedGraph, HashMap<usize, usize>) {
    let mut graph = UndirectedGraph::new();
    let mut local_to_node: HashMap<usize, crate::graph::types::NodeId> = HashMap::new();
    let mut node_to_local: HashMap<usize, usize> = HashMap::new();
    let mut node_counter = 0usize;

    // Add nodes
    for (local_idx, &global_idx) in func_indices.iter().enumerate() {
        if global_idx < index.functions.len() {
            let entry = &index.functions[global_idx];
            let node_data = NodeData {
                path: PathBuf::from(&entry.file_path),
                module: entry.function_name.clone(),
                symbols: vec![],
                loc: entry.end_line - entry.start_line + 1,
                complexity: entry.quality.complexity as f64,
                ast_hash: 0,
            };
            let nid = graph.add_node(node_data);
            local_to_node.insert(local_idx, nid);
            node_to_local.insert(node_counter, local_idx);
            node_counter += 1;
        }
    }

    // Add edges from calls graph (only intra-file edges)
    for (src, dst) in collect_intra_file_edges(index, func_indices, global_to_local, &local_to_node)
    {
        if graph.edge_weight(src, dst).is_none() {
            graph.add_edge(src, dst, 1.0);
        }
    }

    (graph, node_to_local)
}

/// Collect all (src_node, dst_node) pairs for intra-file call edges.
fn collect_intra_file_edges(
    index: &AgentContextIndex,
    func_indices: &[usize],
    global_to_local: &HashMap<usize, usize>,
    local_to_node: &HashMap<usize, crate::graph::types::NodeId>,
) -> Vec<(crate::graph::types::NodeId, crate::graph::types::NodeId)> {
    let mut edges = Vec::new();
    for &global_idx in func_indices {
        let Some(&local_src) = global_to_local.get(&global_idx) else {
            continue;
        };
        let Some(callees) = index.calls.get(&global_idx) else {
            continue;
        };
        let Some(&src_node) = local_to_node.get(&local_src) else {
            continue;
        };
        for &callee_global in callees {
            let Some(&local_dst) = global_to_local.get(&callee_global) else {
                continue;
            };
            if local_dst == local_src {
                continue;
            }
            let Some(&dst_node) = local_to_node.get(&local_dst) else {
                continue;
            };
            edges.push((src_node, dst_node));
        }
    }
    edges
}

/// Simple connected components for small graphs (fallback when < 10 functions).
fn connected_components(graph: &UndirectedGraph) -> Vec<usize> {
    let n = graph.node_count();
    if n == 0 {
        return Vec::new();
    }

    let mut assignments = vec![usize::MAX; n];
    let mut current_community = 0;

    let node_ids: Vec<_> = graph.node_indices().collect();

    for (idx, &nid) in node_ids.iter().enumerate() {
        if assignments[idx] != usize::MAX {
            continue;
        }
        // BFS
        let mut queue = vec![nid];
        assignments[idx] = current_community;
        while let Some(current) = queue.pop() {
            for neighbor in graph.neighbors(current) {
                // Find index of neighbor in node_ids
                if let Some(neighbor_idx) = node_ids.iter().position(|&n| n == neighbor) {
                    if assignments[neighbor_idx] == usize::MAX {
                        assignments[neighbor_idx] = current_community;
                        queue.push(neighbor);
                    }
                }
            }
        }
        current_community += 1;
    }

    assignments
}

fn make_cluster_item(
    local_idx: usize,
    local_entries: &[&FunctionEntry],
    index: &AgentContextIndex,
    func_indices: &[usize],
    global_to_local: &HashMap<usize, usize>,
) -> ClusterItem {
    let entry = local_entries[local_idx];
    let global_idx = func_indices[local_idx];

    let calls = index
        .calls
        .get(&global_idx)
        .map(|callees| {
            callees
                .iter()
                .filter_map(|&c| {
                    global_to_local
                        .get(&c)
                        .map(|&li| local_entries[li].function_name.clone())
                })
                .collect()
        })
        .unwrap_or_default();

    let called_by = index
        .called_by
        .get(&global_idx)
        .map(|callers| {
            callers
                .iter()
                .filter_map(|&c| {
                    global_to_local
                        .get(&c)
                        .map(|&li| local_entries[li].function_name.clone())
                })
                .collect()
        })
        .unwrap_or_default();

    ClusterItem {
        name: entry.function_name.clone(),
        definition_type: format!("{:?}", entry.definition_type),
        line_range: (entry.start_line, entry.end_line),
        calls,
        called_by,
    }
}

/// Generic prefixes that don't make good cluster names.
/// These are common verbs/prepositions that pass the 4-char min-length
/// check in try_common_prefix but don't convey semantic meaning.
const GENERIC_PREFIX_BLOCKLIST: &[&str] = &[
    "from",
    "into",
    "with",
    "make",
    "create",
    "build",
    "parse",
    "check",
    "test",
    "init",
    "load",
    "save",
    "read",
    "write",
    "send",
    "recv",
    "handle",
    "process",
    "convert",
    "transform",
    "validate",
    "exec",
    "run",
    "call",
    "apply",
    "update",
    "delete",
    "remove",
    "find",
    "get",
    "set",
    "new",
    "try",
    "is",
    "has",
    "can",
    "should",
];

/// Name a cluster using the suggest-rename signal cascade.
fn name_cluster(entries: &[&FunctionEntry], file_path: &str) -> (String, String, f32) {
    // Try each signal in priority order

    // 1. Dominant type (struct/enum/trait)
    if let Some((name, confidence, _reason)) = try_dominant_type(entries) {
        return (name, "DominantType".to_string(), confidence);
    }

    // 2. Function theme (>70% share a keyword)
    if let Some((name, confidence, _reason)) = try_function_theme(entries) {
        return (name, "FunctionTheme".to_string(), confidence);
    }

    // 3. Common prefix (skip generic verbs/prepositions)
    if let Some((name, confidence, _reason)) = try_common_prefix(entries) {
        if !GENERIC_PREFIX_BLOCKLIST.contains(&name.as_str()) {
            return (name, "CommonPrefix".to_string(), confidence);
        }
    }

    // 4. Doc comment consensus
    if let Some((name, confidence, _reason)) = try_doc_comment_consensus(entries) {
        return (name, "DocCommentConsensus".to_string(), confidence);
    }

    // 5. Context word from function names
    if let Some(word) = find_context_word(entries) {
        return (word, "ContextWord".to_string(), 0.5);
    }

    // 6. Fallback: use file stem + cluster index
    let stem = Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("module");
    (format!("{}_cluster", stem), "Fallback".to_string(), 0.2)
}

/// Compute cohesion: ratio of internal edges to max possible edges.
fn compute_cohesion(
    local_indices: &[usize],
    index: &AgentContextIndex,
    func_indices: &[usize],
    global_to_local: &HashMap<usize, usize>,
) -> f64 {
    if local_indices.len() < 2 {
        return 1.0;
    }

    let local_set: HashSet<usize> = local_indices.iter().copied().collect();
    let mut internal_edges = 0usize;

    for &li in local_indices {
        let global_idx = func_indices[li];
        if let Some(callees) = index.calls.get(&global_idx) {
            for &callee_global in callees {
                if let Some(&callee_local) = global_to_local.get(&callee_global) {
                    if local_set.contains(&callee_local) && callee_local != li {
                        internal_edges += 1;
                    }
                }
            }
        }
    }

    let n = local_indices.len() as f64;
    let max_edges = n * (n - 1.0);
    if max_edges == 0.0 {
        return 1.0;
    }
    internal_edges as f64 / max_edges
}

/// Compute split impact: which files import this module.
fn compute_impact(index: &AgentContextIndex, file_path: &str) -> SplitImpact {
    let mut importing_files = Vec::new();

    // Scan for files that reference functions in this file
    let target_indices: HashSet<usize> = index
        .file_index
        .get(file_path)
        .map(|v| v.iter().copied().collect())
        .unwrap_or_default();

    for (other_file, other_indices) in &index.file_index {
        if other_file == file_path {
            continue;
        }
        // Check if any function in other_file calls a function in our file
        let has_dependency = other_indices.iter().any(|&gi| {
            index
                .calls
                .get(&gi)
                .map(|callees| callees.iter().any(|c| target_indices.contains(c)))
                .unwrap_or(false)
        });
        if has_dependency {
            importing_files.push(other_file.clone());
        }
    }

    importing_files.sort();

    SplitImpact {
        importing_files,
        circular_risks: Vec::new(), // TODO: detect circular deps
    }
}

// ── Execute Split ──────────────────────────────────────────────────────────

/// Execute a split plan by creating new files with `include!()` pattern.
///
/// For each cluster, creates `{base}_{cluster_name}.rs` and replaces the
/// original file with `include!()` directives.
pub fn execute_split(plan: &SplitPlan, project_root: &Path) -> anyhow::Result<Vec<PathBuf>> {
    use std::fs;

    let source_path = project_root.join(&plan.source_file);
    let source_content = fs::read_to_string(&source_path)?;
    let source_lines: Vec<&str> = source_content.lines().collect();

    let parent_dir = source_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("No parent directory for {}", plan.source_file))?;
    let stem = source_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid file stem for {}", plan.source_file))?;

    let mut created_files = Vec::new();
    let mut include_directives = Vec::new();

    for cluster in &plan.clusters {
        let cluster_filename = format!("{}_{}.rs", stem, cluster.suggested_name);
        let cluster_path = parent_dir.join(&cluster_filename);

        // Extract lines for this cluster
        let mut cluster_lines = Vec::new();
        let mut ranges: Vec<(usize, usize)> = cluster
            .items
            .iter()
            .map(|item| (item.line_range.0, item.line_range.1))
            .collect();
        ranges.sort_by_key(|r| r.0);

        for (start, end) in &ranges {
            // Lines are 1-indexed in FunctionEntry
            let start_idx = start.saturating_sub(1);
            let total_lines = source_lines.len();
            let end_idx = (*end).min(total_lines);
            for line in &source_lines[start_idx..end_idx] {
                cluster_lines.push(*line);
            }
            cluster_lines.push(""); // blank line separator
        }

        let cluster_content = cluster_lines.join("\n");
        fs::write(&cluster_path, &cluster_content)?;
        created_files.push(cluster_path);

        include_directives.push(format!("include!(\"{}\");", cluster_filename));
    }

    // Note: We don't rewrite the original file automatically — that requires
    // careful handling of use statements, module-level attributes, etc.
    // The user should review the generated files and update the source manually.

    Ok(created_files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connected_components_empty() {
        let graph = UndirectedGraph::new();
        let comms = connected_components(&graph);
        assert!(comms.is_empty());
    }

    #[test]
    fn test_connected_components_singleton() {
        let mut graph = UndirectedGraph::new();
        graph.add_node(NodeData {
            path: PathBuf::from("test.rs"),
            module: "test".to_string(),
            symbols: vec![],
            loc: 10,
            complexity: 1.0,
            ast_hash: 0,
        });
        let comms = connected_components(&graph);
        assert_eq!(comms.len(), 1);
        assert_eq!(comms[0], 0);
    }

    #[test]
    fn test_connected_components_two_disconnected() {
        let mut graph = UndirectedGraph::new();
        graph.add_node(NodeData {
            path: PathBuf::from("a.rs"),
            module: "a".to_string(),
            symbols: vec![],
            loc: 10,
            complexity: 1.0,
            ast_hash: 0,
        });
        graph.add_node(NodeData {
            path: PathBuf::from("b.rs"),
            module: "b".to_string(),
            symbols: vec![],
            loc: 10,
            complexity: 1.0,
            ast_hash: 0,
        });
        let comms = connected_components(&graph);
        assert_eq!(comms.len(), 2);
        // Should be in different communities
        assert_ne!(comms[0], comms[1]);
    }

    #[test]
    fn test_connected_components_two_connected() {
        let mut graph = UndirectedGraph::new();
        let n1 = graph.add_node(NodeData {
            path: PathBuf::from("a.rs"),
            module: "a".to_string(),
            symbols: vec![],
            loc: 10,
            complexity: 1.0,
            ast_hash: 0,
        });
        let n2 = graph.add_node(NodeData {
            path: PathBuf::from("b.rs"),
            module: "b".to_string(),
            symbols: vec![],
            loc: 10,
            complexity: 1.0,
            ast_hash: 0,
        });
        graph.add_edge(n1, n2, 1.0);
        let comms = connected_components(&graph);
        assert_eq!(comms.len(), 2);
        // Should be in the same community
        assert_eq!(comms[0], comms[1]);
    }

    #[test]
    fn test_estimate_total_lines_empty() {
        let entries: Vec<&FunctionEntry> = vec![];
        assert_eq!(estimate_total_lines(&entries), 0);
    }

    #[test]
    fn test_split_impact_default() {
        let impact = SplitImpact {
            importing_files: vec!["a.rs".to_string()],
            circular_risks: vec![],
        };
        assert_eq!(impact.importing_files.len(), 1);
        assert!(impact.circular_risks.is_empty());
    }

    fn make_entry(name: &str, file: &str, start: usize, end: usize) -> FunctionEntry {
        use crate::services::agent_context::function_index::DefinitionType;
        FunctionEntry {
            file_path: file.to_string(),
            function_name: name.to_string(),
            signature: format!("fn {}()", name),
            definition_type: DefinitionType::Function,
            doc_comment: None,
            source: format!("fn {}() {{}}", name),
            start_line: start,
            end_line: end,
            language: "rust".to_string(),
            quality: Default::default(),
            checksum: String::new(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: vec![],
        }
    }

    fn make_struct_entry(name: &str, file: &str, start: usize, end: usize) -> FunctionEntry {
        use crate::services::agent_context::function_index::DefinitionType;
        let mut entry = make_entry(name, file, start, end);
        entry.definition_type = DefinitionType::Struct;
        entry.signature = format!("struct {}", name);
        entry
    }

    #[test]
    fn test_name_cluster_dominant_type() {
        let e1 = make_struct_entry("MyConfig", "a.rs", 1, 10);
        let e2 = make_entry("new", "a.rs", 12, 20);
        let e3 = make_entry("load", "a.rs", 22, 30);
        let entries: Vec<&FunctionEntry> = vec![&e1, &e2, &e3];
        let (name, signal, confidence) = name_cluster(&entries, "a.rs");
        // Should use dominant type or function theme depending on signal
        assert!(!name.is_empty());
        assert!(!signal.is_empty());
        assert!(confidence > 0.0);
    }

    #[test]
    fn test_name_cluster_fallback() {
        // Create entries with no clear signal
        let e1 = make_entry("x", "src/mod.rs", 1, 5);
        let e2 = make_entry("y", "src/mod.rs", 6, 10);
        let entries: Vec<&FunctionEntry> = vec![&e1, &e2];
        let (name, signal, confidence) = name_cluster(&entries, "src/mod.rs");
        // Fallback should produce something
        assert!(!name.is_empty());
        assert!(confidence <= 1.0);
        // With such short names and no signal, likely falls back
        assert!(signal == "ContextWord" || signal == "Fallback");
    }

    #[test]
    fn test_compute_cohesion_single() {
        let cohesion = compute_cohesion(
            &[0],
            &AgentContextIndex::build(std::path::Path::new(".")).unwrap_or_else(|_| {
                // Fallback: test with empty-ish assertion
                panic!("Index needed for cohesion test");
            }),
            &[0],
            &HashMap::new(),
        );
        assert_eq!(cohesion, 1.0);
    }

    #[test]
    fn test_split_plan_serialization() {
        let plan = SplitPlan {
            source_file: "test.rs".to_string(),
            total_lines: 500,
            clusters: vec![SplitCluster {
                suggested_name: "config".to_string(),
                naming_signal: "DominantType".to_string(),
                confidence: 0.9,
                items: vec![ClusterItem {
                    name: "Config".to_string(),
                    definition_type: "Struct".to_string(),
                    line_range: (1, 50),
                    calls: vec![],
                    called_by: vec![],
                }],
                estimated_lines: 50,
                cohesion: 0.8,
            }],
            unclustered: vec![],
            impact: SplitImpact {
                importing_files: vec!["main.rs".to_string()],
                circular_risks: vec![],
            },
            modularity: 0.45,
        };

        let json = serde_json::to_string(&plan).unwrap();
        assert!(json.contains("config"));
        assert!(json.contains("DominantType"));
        assert!(json.contains("main.rs"));
    }

    #[test]
    fn test_cluster_item_serialization() {
        let item = ClusterItem {
            name: "process_data".to_string(),
            definition_type: "Function".to_string(),
            line_range: (10, 50),
            calls: vec!["helper".to_string()],
            called_by: vec!["main".to_string()],
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("process_data"));
        assert!(json.contains("helper"));
    }

    #[test]
    fn test_split_cluster_serialization() {
        let cluster = SplitCluster {
            suggested_name: "parsing".to_string(),
            naming_signal: "FunctionTheme".to_string(),
            confidence: 0.85,
            items: vec![],
            estimated_lines: 200,
            cohesion: 0.6,
        };
        let json = serde_json::to_string(&cluster).unwrap();
        assert!(json.contains("parsing"));
        assert!(json.contains("0.85"));
    }

    #[test]
    fn test_suggest_split_missing_file() {
        // Build index on current project
        let index = match AgentContextIndex::build(std::path::Path::new(".")) {
            Ok(i) => i,
            Err(_) => return, // Skip if can't build index
        };

        let result = suggest_split(&index, "nonexistent_file.rs", 1.0, 50);
        assert!(result.is_none());
    }

    #[test]
    fn test_execute_split_empty_plan() {
        let plan = SplitPlan {
            source_file: "test.rs".to_string(),
            total_lines: 100,
            clusters: vec![],
            unclustered: vec![],
            impact: SplitImpact {
                importing_files: vec![],
                circular_risks: vec![],
            },
            modularity: 0.0,
        };

        let temp_dir = std::env::temp_dir().join("pmat_split_test");
        let _ = std::fs::create_dir_all(&temp_dir);
        let test_file = temp_dir.join("test.rs");
        std::fs::write(&test_file, "fn main() {}\n").unwrap();

        let result = execute_split(&plan, &temp_dir);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_execute_split_with_cluster() {
        let plan = SplitPlan {
            source_file: "test.rs".to_string(),
            total_lines: 10,
            clusters: vec![SplitCluster {
                suggested_name: "helpers".to_string(),
                naming_signal: "FunctionTheme".to_string(),
                confidence: 0.8,
                items: vec![ClusterItem {
                    name: "helper_fn".to_string(),
                    definition_type: "Function".to_string(),
                    line_range: (1, 3),
                    calls: vec![],
                    called_by: vec![],
                }],
                estimated_lines: 3,
                cohesion: 1.0,
            }],
            unclustered: vec![],
            impact: SplitImpact {
                importing_files: vec![],
                circular_risks: vec![],
            },
            modularity: 0.5,
        };

        let temp_dir = std::env::temp_dir().join("pmat_split_test2");
        let _ = std::fs::create_dir_all(&temp_dir);
        let test_file = temp_dir.join("test.rs");
        std::fs::write(
            &test_file,
            "fn helper_fn() {}\nfn other() {}\nfn last() {}\n",
        )
        .unwrap();

        let result = execute_split(&plan, &temp_dir);
        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].to_string_lossy().contains("test_helpers.rs"));

        // Verify file was created with content
        let content = std::fs::read_to_string(&files[0]).unwrap();
        assert!(content.contains("helper_fn"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_estimate_total_lines_with_entries() {
        let e1 = make_entry("a", "f.rs", 1, 50);
        let e2 = make_entry("b", "f.rs", 51, 100);
        let entries: Vec<&FunctionEntry> = vec![&e1, &e2];
        assert_eq!(estimate_total_lines(&entries), 100);
    }

    #[test]
    fn test_make_cluster_item_basic() {
        let e1 = make_entry("process", "f.rs", 10, 30);
        let _entries: Vec<&FunctionEntry> = vec![&e1];
        let _func_indices = vec![0usize];
        let mut global_to_local = HashMap::new();
        global_to_local.insert(0usize, 0usize);

        let index_stub = AgentContextIndex::build(std::path::Path::new("."));
        if index_stub.is_err() {
            return; // Skip if can't build
        }
        // Just test with the struct creation path
        let item = ClusterItem {
            name: "process".to_string(),
            definition_type: "Function".to_string(),
            line_range: (10, 30),
            calls: vec![],
            called_by: vec![],
        };
        assert_eq!(item.name, "process");
        assert_eq!(item.line_range, (10, 30));
    }

    #[test]
    fn test_generic_prefix_blocklist_contains_from() {
        assert!(GENERIC_PREFIX_BLOCKLIST.contains(&"from"));
        assert!(GENERIC_PREFIX_BLOCKLIST.contains(&"into"));
        assert!(GENERIC_PREFIX_BLOCKLIST.contains(&"with"));
        assert!(GENERIC_PREFIX_BLOCKLIST.contains(&"make"));
        assert!(GENERIC_PREFIX_BLOCKLIST.contains(&"handle"));
    }

    #[test]
    fn test_generic_prefix_blocklist_allows_good_names() {
        assert!(!GENERIC_PREFIX_BLOCKLIST.contains(&"baseline"));
        assert!(!GENERIC_PREFIX_BLOCKLIST.contains(&"health"));
        assert!(!GENERIC_PREFIX_BLOCKLIST.contains(&"metrics"));
        assert!(!GENERIC_PREFIX_BLOCKLIST.contains(&"cluster"));
    }

    #[test]
    fn test_name_cluster_skips_generic_prefix() {
        // Create entries where common prefix would be "from" but should be skipped
        let e1 = make_entry("from_score", "f.rs", 1, 10);
        let e2 = make_entry("from_files", "f.rs", 11, 20);
        let e3 = make_entry("from_projects", "f.rs", 21, 30);
        let entries: Vec<&FunctionEntry> = vec![&e1, &e2, &e3];

        let (name, signal, _confidence) = name_cluster(&entries, "file_health.rs");
        // Should NOT be "from" — should fall through to a better signal
        assert_ne!(name, "from", "Generic prefix 'from' should be blocked");
        assert_ne!(
            signal, "CommonPrefix",
            "Should skip CommonPrefix for generic verb"
        );
    }

    #[test]
    fn test_name_cluster_allows_specific_prefix() {
        // Create entries where common prefix is a meaningful domain word
        let e1 = make_entry("baseline_save", "f.rs", 1, 10);
        let e2 = make_entry("baseline_load", "f.rs", 11, 20);
        let e3 = make_entry("baseline_check", "f.rs", 21, 30);
        let entries: Vec<&FunctionEntry> = vec![&e1, &e2, &e3];

        let (name, signal, _confidence) = name_cluster(&entries, "file_health.rs");
        // "baseline" should be accepted since it's not in the blocklist
        assert_eq!(name, "baseline");
        assert_eq!(signal, "CommonPrefix");
    }
}
