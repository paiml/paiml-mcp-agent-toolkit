#![cfg_attr(coverage_nightly, coverage(off))]
//! PTX Dataflow Tracing
//!
//! Classifies functions by PTX role (Emitter, Loader, Analyzer, Consumer)
//! and traces dataflow across project boundaries.

use crate::services::agent_context::AgentContextIndex;

/// PTX role classification for a function
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PtxRole {
    /// Generates PTX code (inline asm, .ptx/.cu files, PTX string literals)
    Emitter,
    /// Loads/parses PTX modules (cuModuleLoad, create_shader_module)
    Loader,
    /// Analyzes PTX metrics (barrier checks, register counts)
    Analyzer,
    /// Consumes PTX results transitively (calls Loaders/Emitters)
    Consumer,
}

impl std::fmt::Display for PtxRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PtxRole::Emitter => write!(f, "emitter"),
            PtxRole::Loader => write!(f, "loader"),
            PtxRole::Analyzer => write!(f, "analyzer"),
            PtxRole::Consumer => write!(f, "consumer"),
        }
    }
}

/// A node in the PTX dataflow DAG
#[derive(Debug, Clone)]
pub struct PtxFlowNode {
    pub project: String,
    pub function_name: String,
    pub file_path: String,
    pub role: PtxRole,
    pub func_idx: usize,
}

/// An edge in the PTX dataflow DAG
#[derive(Debug, Clone)]
pub struct PtxFlowEdge {
    pub from_idx: usize,
    pub to_idx: usize,
}

/// Result of PTX dataflow analysis
pub struct PtxFlowResult {
    pub nodes: Vec<PtxFlowNode>,
    pub edges: Vec<PtxFlowEdge>,
}

const EMITTER_KEYWORDS: &[&str] = &[
    "asm!(", "asm volatile", "global_asm!", ".version ", ".target sm_",
    "__global__", "__device__", "emit_ptx", "ptx_builder",
];
const LOADER_KEYWORDS: &[&str] = &[
    "cuModuleLoad", "cuModuleLoadData", "create_shader_module",
    "load_ptx", "ptx_module", "load_module", "compile_ptx",
];
const ANALYZER_KEYWORDS: &[&str] = &[
    "barrier_count", "register_pressure", "shared_memory_size",
    "ptx_analysis", "detect_ptx_barrier", "detect_shared_memory", "ptx_diagnostic",
];

fn source_matches_any(source: &str, keywords: &[&str]) -> bool {
    keywords.iter().any(|kw| source.contains(kw))
}

/// Classify a function's PTX role based on source content and file extension.
pub fn classify_ptx_role(source: &str, file_path: &str) -> Option<PtxRole> {
    let is_ptx_file = file_path.ends_with(".ptx") || file_path.ends_with(".cu");
    if is_ptx_file || source_matches_any(source, EMITTER_KEYWORDS) {
        return Some(PtxRole::Emitter);
    }
    if source_matches_any(source, LOADER_KEYWORDS) {
        return Some(PtxRole::Loader);
    }
    if source_matches_any(source, ANALYZER_KEYWORDS) {
        return Some(PtxRole::Analyzer);
    }
    None
}

fn make_node(func: &crate::services::agent_context::FunctionEntry, idx: usize, role: PtxRole) -> PtxFlowNode {
    PtxFlowNode {
        project: func.file_path.split('/').next().unwrap_or("local").to_string(),
        function_name: func.function_name.clone(),
        file_path: func.file_path.clone(),
        role,
        func_idx: idx,
    }
}

/// Phase 1: classify all functions with a PTX role
fn classify_ptx_nodes(index: &AgentContextIndex) -> (Vec<PtxFlowNode>, Vec<usize>) {
    let mut nodes = Vec::new();
    let mut node_func_idx = Vec::new();
    for (i, func) in index.all_functions().iter().enumerate() {
        if let Some(role) = classify_ptx_role(&func.source, &func.file_path) {
            nodes.push(make_node(func, i, role));
            node_func_idx.push(i);
        }
    }
    (nodes, node_func_idx)
}

/// Phase 2: find Consumer nodes (callers of PTX nodes) via inverted lookup
fn find_consumer_nodes(
    index: &AgentContextIndex,
    nodes: &mut Vec<PtxFlowNode>,
    node_func_idx: &[usize],
) {
    let ptx_indices: std::collections::HashSet<usize> = node_func_idx.iter().copied().collect();
    let mut seen = std::collections::HashSet::new();
    for &ptx_idx in node_func_idx {
        let Some(caller_indices) = index.called_by_indices(ptx_idx) else { continue };
        for &caller_idx in caller_indices {
            if !ptx_indices.contains(&caller_idx) && seen.insert(caller_idx) {
                nodes.push(make_node(&index.all_functions()[caller_idx], caller_idx, PtxRole::Consumer));
            }
        }
    }
}

/// Phase 3: build edges between PTX nodes using the call graph
fn build_flow_edges(index: &AgentContextIndex, nodes: &[PtxFlowNode]) -> Vec<PtxFlowEdge> {
    let func_idx_to_node: std::collections::HashMap<usize, usize> = nodes
        .iter()
        .enumerate()
        .map(|(node_idx, node)| (node.func_idx, node_idx))
        .collect();

    let mut edges = Vec::new();
    for (from_node_idx, from_node) in nodes.iter().enumerate() {
        let Some(callee_indices) = index.calls_indices(from_node.func_idx) else { continue };
        for &callee_idx in callee_indices {
            if let Some(&to_node_idx) = func_idx_to_node.get(&callee_idx) {
                if from_node_idx != to_node_idx {
                    edges.push(PtxFlowEdge { from_idx: from_node_idx, to_idx: to_node_idx });
                }
            }
        }
    }
    edges
}

/// Trace PTX dataflow across the merged index.
///
/// Builds a DAG of Emitter → Loader → Consumer chains.
pub fn trace_ptx_dataflow(index: &AgentContextIndex) -> PtxFlowResult {
    let (mut nodes, node_func_idx) = classify_ptx_nodes(index);
    find_consumer_nodes(index, &mut nodes, &node_func_idx);
    let edges = build_flow_edges(index, &nodes);
    PtxFlowResult { nodes, edges }
}

/// Format PTX flow result as a human-readable table
pub fn format_ptx_flow_text(result: &PtxFlowResult) -> String {
    let mut out = String::new();
    out.push_str(&format!("\x1b[1;4mPTX Dataflow\x1b[0m ({} nodes, {} edges)\n\n", result.nodes.len(), result.edges.len()));

    if result.nodes.is_empty() {
        out.push_str("  No PTX-related functions found in workspace.\n");
        return out;
    }

    // Group by role
    for role in &[PtxRole::Emitter, PtxRole::Loader, PtxRole::Analyzer, PtxRole::Consumer] {
        let role_nodes: Vec<_> = result.nodes.iter().filter(|n| &n.role == role).collect();
        if role_nodes.is_empty() {
            continue;
        }
        let role_color = match role {
            PtxRole::Emitter => "\x1b[1;31m",
            PtxRole::Loader => "\x1b[1;33m",
            PtxRole::Analyzer => "\x1b[1;36m",
            PtxRole::Consumer => "\x1b[1;32m",
        };
        out.push_str(&format!("  {role_color}{role}\x1b[0m ({}):\n", role_nodes.len()));
        for node in &role_nodes {
            out.push_str(&format!("    [{project}] {name}  \x1b[2m{path}\x1b[0m\n",
                project = node.project, name = node.function_name, path = node.file_path));
        }
        out.push('\n');
    }

    // Show edges
    if !result.edges.is_empty() {
        out.push_str("  \x1b[1mDataflow chains:\x1b[0m\n");
        for edge in &result.edges {
            let from = &result.nodes[edge.from_idx];
            let to = &result.nodes[edge.to_idx];
            out.push_str(&format!("    [{src}] {src_fn} \x1b[2m({src_role})\x1b[0m → [{dst}] {dst_fn} \x1b[2m({dst_role})\x1b[0m\n",
                src = from.project, src_fn = from.function_name, src_role = from.role,
                dst = to.project, dst_fn = to.function_name, dst_role = to.role));
        }
        out.push('\n');
    }

    out
}

/// Format PTX flow result as JSON
pub fn format_ptx_flow_json(result: &PtxFlowResult) -> String {
    let nodes: Vec<serde_json::Value> = result.nodes.iter().map(|n| {
        serde_json::json!({
            "project": n.project,
            "function_name": n.function_name,
            "file_path": n.file_path,
            "role": n.role.to_string(),
        })
    }).collect();
    let edges: Vec<serde_json::Value> = result.edges.iter().map(|e| {
        serde_json::json!({
            "from": result.nodes[e.from_idx].function_name,
            "to": result.nodes[e.to_idx].function_name,
            "from_project": result.nodes[e.from_idx].project,
            "to_project": result.nodes[e.to_idx].project,
        })
    }).collect();
    serde_json::json!({
        "ptx_flow": { "nodes": nodes, "edges": edges }
    }).to_string()
}
