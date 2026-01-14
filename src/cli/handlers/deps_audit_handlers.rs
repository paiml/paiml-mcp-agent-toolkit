//! CLI handler for `pmat deps-audit` command
//!
//! Analyzes dependencies and suggests removals/replacements with
//! Sovereign AI stack (trueno ecosystem) alternatives.
//!
//! Uses trueno-graph for dependency graph analysis:
//! - PageRank for criticality scoring
//! - Transitive dependency counts
//! - Bridge/orphan detection

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use trueno_graph::{CsrGraph, NodeId, pagerank};

/// Dependency category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DepCategory {
    /// Core - essential, cannot be removed
    Core,
    /// Sovereign - already part of trueno/paiml ecosystem
    Sovereign,
    /// Replaceable - can be replaced with Sovereign stack
    Replaceable,
    /// Heavy - large dependency that adds significant bloat
    Heavy,
    /// DevOnly - only needed for development/testing
    DevOnly,
    /// Removable - can likely be removed entirely
    Removable,
}

/// Dependency analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepAnalysis {
    pub name: String,
    pub version: String,
    pub category: DepCategory,
    pub replacement: Option<String>,
    pub reason: String,
    pub transitive_count: usize,
    pub estimated_size_kb: usize,
    // Graph metrics
    pub pagerank_score: f32,
    pub in_degree: usize,  // How many deps depend on this
    pub out_degree: usize, // How many deps this brings in
    pub is_bridge: bool,   // Connects otherwise disconnected clusters
    pub is_orphan: bool,   // Nothing depends on this (easy to remove)
}

/// Full audit report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepsAuditReport {
    pub total_deps: usize,
    pub direct_deps: usize,
    pub transitive_deps: usize,
    pub sovereign_deps: usize,
    pub replaceable_deps: usize,
    pub removable_deps: usize,
    pub heavy_deps: usize,
    pub orphan_deps: usize,
    pub bridge_deps: usize,
    pub estimated_savings_kb: usize,
    pub dependencies: Vec<DepAnalysis>,
    pub recommendations: Vec<String>,
    // Graph analysis
    pub top_critical: Vec<(String, f32)>, // Top deps by PageRank
    pub removal_candidates: Vec<String>,   // Orphans that are safe to remove
}

/// Known Sovereign AI stack packages
const SOVEREIGN_PACKAGES: &[&str] = &[
    "trueno",
    "trueno-graph",
    "trueno-rag",
    "trueno-viz",
    "aprender",
    "pmcp",
    "presentar-terminal",
    "ruchy",
    "batuta",
    "realizár",
    "renacer",
    "certeza",
];

/// Replaceable dependencies with Sovereign alternatives
fn get_replacements() -> HashMap<&'static str, (&'static str, &'static str)> {
    let mut map = HashMap::new();
    // (dependency, (replacement, reason))
    map.insert("petgraph", ("trueno-graph", "Graph algorithms with O(1) lookups"));
    map.insert("ratatui", ("presentar-terminal", "TUI with ComputeBrick profiling"));
    map.insert("tui", ("presentar-terminal", "TUI with ComputeBrick profiling"));
    map.insert("crossterm", ("presentar-terminal", "Included in presentar-terminal"));
    map.insert("ndarray", ("trueno", "SIMD-accelerated tensors"));
    map.insert("nalgebra", ("trueno", "SIMD-accelerated linear algebra"));
    map.insert("arrow", ("trueno", "Use trueno for columnar data"));
    map.insert("parquet", ("trueno-rag", "Integrated in trueno-rag"));
    map.insert("polars", ("trueno", "Use trueno for dataframes"));
    map
}

/// Heavy dependencies that add significant compile time/binary size
fn get_heavy_deps() -> HashMap<&'static str, (&'static str, usize)> {
    let mut map = HashMap::new();
    // (dependency, (reason, estimated_kb))
    map.insert("swc_ecma_parser", ("JS/TS parsing - consider tree-sitter only", 8000));
    map.insert("swc_common", ("SWC common - heavy TypeScript support", 3000));
    map.insert("swc_ecma_ast", ("SWC AST - heavy TypeScript support", 2000));
    map.insert("swc_ecma_visit", ("SWC visitor - heavy TypeScript support", 1500));
    map.insert("octocrab", ("GitHub API - consider lighter ureq-based", 5000));
    map.insert("reqwest", ("HTTP client - consider ureq for sync", 4000));
    map.insert("rusqlite", ("SQLite - consider removing if unused", 2500));
    map.insert("git2", ("libgit2 bindings - shell out to git instead", 6000));
    map.insert("criterion", ("Benchmarking - dev only", 3000));
    map.insert("proptest", ("Property testing - dev only", 2000));
    map
}

/// Dev-only dependencies
const DEV_ONLY: &[&str] = &[
    "criterion",
    "proptest",
    "quickcheck",
    "quickcheck_macros",
    "assert_cmd",
    "predicates",
    "pretty_assertions",
    "env_logger",
    "futures-test",
    "tokio-test",
    "serial_test",
];

/// Potentially removable dependencies
fn get_removable() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();
    map.insert("prettytable-rs", "Use simple formatting instead");
    map.insert("dialoguer", "Use simple stdin/stdout");
    map.insert("console", "Minimal terminal handling needed");
    map.insert("indicatif", "Progress bars may not be needed");
    map.insert("webbrowser", "Shell out to xdg-open/open instead");
    map.insert("sourcemap", "Only needed if debugging JS");
    map.insert("pulldown-cmark", "Use simple markdown or none");
    map.insert("pest", "Consider tree-sitter for all parsing");
    map.insert("pest_derive", "Consider tree-sitter for all parsing");
    map.insert("rmp-serde", "Use JSON or bincode only");
    map.insert("bincode", "Use JSON only for simplicity");
    map
}

/// Analyze a single dependency (graph metrics populated later)
fn analyze_dep(name: &str, version: &str, is_dev: bool) -> DepAnalysis {
    let replacements = get_replacements();
    let heavy = get_heavy_deps();
    let removable = get_removable();

    let base = DepAnalysis {
        name: name.to_string(),
        version: version.to_string(),
        category: DepCategory::Core,
        replacement: None,
        reason: String::new(),
        transitive_count: 0,
        estimated_size_kb: 100,
        // Graph metrics initialized to defaults, populated by analyze_graph()
        pagerank_score: 0.0,
        in_degree: 0,
        out_degree: 0,
        is_bridge: false,
        is_orphan: false,
    };

    // Check if it's a Sovereign package
    if SOVEREIGN_PACKAGES.contains(&name) {
        return DepAnalysis {
            category: DepCategory::Sovereign,
            reason: "Part of Sovereign AI stack".to_string(),
            estimated_size_kb: 0,
            ..base
        };
    }

    // Check if dev-only
    if is_dev || DEV_ONLY.contains(&name) {
        let (reason, size) = heavy.get(name).map(|(r, s)| (*r, *s)).unwrap_or(("Development dependency", 500));
        return DepAnalysis {
            category: DepCategory::DevOnly,
            reason: reason.to_string(),
            estimated_size_kb: size,
            ..base
        };
    }

    // Check if replaceable
    if let Some((replacement, reason)) = replacements.get(name) {
        return DepAnalysis {
            category: DepCategory::Replaceable,
            replacement: Some(replacement.to_string()),
            reason: reason.to_string(),
            estimated_size_kb: 2000,
            ..base
        };
    }

    // Check if heavy
    if let Some((reason, size)) = heavy.get(name) {
        return DepAnalysis {
            category: DepCategory::Heavy,
            reason: reason.to_string(),
            estimated_size_kb: *size,
            ..base
        };
    }

    // Check if removable
    if let Some(reason) = removable.get(name) {
        return DepAnalysis {
            category: DepCategory::Removable,
            reason: reason.to_string(),
            estimated_size_kb: 500,
            ..base
        };
    }

    // Default: Core dependency
    DepAnalysis {
        reason: "Essential dependency".to_string(),
        ..base
    }
}

/// Parse Cargo.toml and extract dependencies
#[allow(clippy::type_complexity)]
fn parse_cargo_toml(path: &Path) -> Result<(Vec<(String, String, bool)>, Vec<(String, String, bool)>)> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;

    let toml: toml::Value = toml::from_str(&content)
        .with_context(|| "Failed to parse Cargo.toml")?;

    let mut deps = Vec::new();
    let mut dev_deps = Vec::new();

    // Regular dependencies
    if let Some(dependencies) = toml.get("dependencies").and_then(|d| d.as_table()) {
        for (name, value) in dependencies {
            let version = match value {
                toml::Value::String(v) => v.clone(),
                toml::Value::Table(t) => t.get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("*")
                    .to_string(),
                _ => "*".to_string(),
            };
            deps.push((name.clone(), version, false));
        }
    }

    // Dev dependencies
    if let Some(dependencies) = toml.get("dev-dependencies").and_then(|d| d.as_table()) {
        for (name, value) in dependencies {
            let version = match value {
                toml::Value::String(v) => v.clone(),
                toml::Value::Table(t) => t.get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("*")
                    .to_string(),
                _ => "*".to_string(),
            };
            dev_deps.push((name.clone(), version, true));
        }
    }

    Ok((deps, dev_deps))
}

/// Dependency edge from Cargo.lock
#[derive(Debug, Clone)]
struct DepEdge {
    from: String,
    to: String,
}

/// Parse Cargo.lock and extract dependency graph
/// Looks in path, parent, and grandparent (for workspace roots)
fn parse_cargo_lock(path: &Path) -> Result<(Vec<String>, Vec<DepEdge>)> {
    // Try path, parent, grandparent (workspace root)
    let candidates = [
        path.join("Cargo.lock"),
        path.parent().map(|p| p.join("Cargo.lock")).unwrap_or_default(),
        path.parent().and_then(|p| p.parent()).map(|p| p.join("Cargo.lock")).unwrap_or_default(),
    ];

    let lock_path = candidates.iter().find(|p| p.exists());
    let Some(lock_path) = lock_path else {
        return Ok((Vec::new(), Vec::new()));
    };

    let content = fs::read_to_string(lock_path)
        .with_context(|| format!("Failed to read {}", lock_path.display()))?;

    let toml: toml::Value = toml::from_str(&content)
        .with_context(|| "Failed to parse Cargo.lock")?;

    let mut all_packages = Vec::new();
    let mut edges = Vec::new();

    if let Some(packages) = toml.get("package").and_then(|p| p.as_array()) {
        for pkg in packages {
            if let Some(name) = pkg.get("name").and_then(|n| n.as_str()) {
                all_packages.push(name.to_string());

                // Extract dependencies for this package
                if let Some(deps) = pkg.get("dependencies").and_then(|d| d.as_array()) {
                    for dep in deps {
                        if let Some(dep_str) = dep.as_str() {
                            // Format: "name version" or "name version (source)"
                            let dep_name = dep_str.split_whitespace().next().unwrap_or(dep_str);
                            edges.push(DepEdge {
                                from: name.to_string(),
                                to: dep_name.to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    Ok((all_packages, edges))
}

/// Graph analysis results
struct GraphAnalysis {
    pagerank_scores: HashMap<String, f32>,
    in_degrees: HashMap<String, usize>,
    out_degrees: HashMap<String, usize>,
    bridges: HashSet<String>,
    orphans: HashSet<String>,
    transitive_counts: HashMap<String, usize>,
}

/// Build and analyze dependency graph using trueno-graph
fn analyze_dependency_graph(
    direct_deps: &[String],
    all_packages: &[String],
    edges: &[DepEdge],
) -> GraphAnalysis {
    let mut name_to_id: HashMap<String, NodeId> = HashMap::new();
    let mut id_to_name: HashMap<NodeId, String> = HashMap::new();

    // Assign node IDs
    for (i, name) in all_packages.iter().enumerate() {
        let node_id = NodeId(i as u32);
        name_to_id.insert(name.clone(), node_id);
        id_to_name.insert(node_id, name.clone());
    }

    // Build edge list for CSR graph (with weights = 1.0)
    let mut edge_list: Vec<(NodeId, NodeId, f32)> = Vec::new();
    let mut in_degrees: HashMap<String, usize> = HashMap::new();
    let mut out_degrees: HashMap<String, usize> = HashMap::new();

    for edge in edges {
        if let (Some(&from_id), Some(&to_id)) = (name_to_id.get(&edge.from), name_to_id.get(&edge.to)) {
            edge_list.push((from_id, to_id, 1.0)); // Default weight = 1.0
            *out_degrees.entry(edge.from.clone()).or_insert(0) += 1;
            *in_degrees.entry(edge.to.clone()).or_insert(0) += 1;
        }
    }

    // Build CSR graph
    let num_nodes = all_packages.len();
    let graph = CsrGraph::from_edge_list(&edge_list).unwrap_or_else(|_| CsrGraph::new());

    // Calculate PageRank
    let pagerank_vec = pagerank(&graph, 20, 1e-6).unwrap_or_else(|_| vec![1.0 / num_nodes.max(1) as f32; num_nodes]);
    let mut pagerank_scores: HashMap<String, f32> = HashMap::new();
    for (i, &score) in pagerank_vec.iter().enumerate() {
        if let Some(name) = id_to_name.get(&NodeId(i as u32)) {
            pagerank_scores.insert(name.clone(), score);
        }
    }

    // Find orphans (direct deps that nothing else depends on)
    let mut orphans = HashSet::new();
    let direct_set: HashSet<_> = direct_deps.iter().cloned().collect();
    for dep in direct_deps {
        if in_degrees.get(dep).copied().unwrap_or(0) == 0 {
            // No other package depends on this
            if direct_set.contains(dep) {
                orphans.insert(dep.clone());
            }
        }
    }

    // Find bridges (deps that connect otherwise unconnected parts)
    // Simple heuristic: deps with high betweenness = both in and out degree > 0
    // and are the only path between their dependents and dependencies
    let mut bridges = HashSet::new();
    for (name, &out_deg) in &out_degrees {
        let in_deg = in_degrees.get(name).copied().unwrap_or(0);
        // High connectivity on both sides suggests bridge
        if in_deg >= 2 && out_deg >= 3 {
            bridges.insert(name.clone());
        }
    }

    // Calculate transitive dependency counts (how many deps does each direct dep bring)
    let mut transitive_counts: HashMap<String, usize> = HashMap::new();
    for dep in direct_deps {
        let count = count_transitive_deps(dep, &edge_list, &name_to_id, &id_to_name);
        transitive_counts.insert(dep.clone(), count);
    }

    GraphAnalysis {
        pagerank_scores,
        in_degrees,
        out_degrees,
        bridges,
        orphans,
        transitive_counts,
    }
}

/// Count transitive dependencies via BFS
fn count_transitive_deps(
    start: &str,
    edges: &[(NodeId, NodeId, f32)],
    name_to_id: &HashMap<String, NodeId>,
    _id_to_name: &HashMap<NodeId, String>,
) -> usize {
    let Some(&start_id) = name_to_id.get(start) else {
        return 0;
    };

    // Build adjacency list (ignoring weights for traversal)
    let mut adj: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    for &(from, to, _weight) in edges {
        adj.entry(from).or_default().push(to);
    }

    // BFS to count reachable nodes
    let mut visited = HashSet::new();
    let mut queue = vec![start_id];
    visited.insert(start_id);

    while let Some(node) = queue.pop() {
        if let Some(neighbors) = adj.get(&node) {
            for &neighbor in neighbors {
                if visited.insert(neighbor) {
                    queue.push(neighbor);
                }
            }
        }
    }

    // Don't count self
    visited.len().saturating_sub(1)
}

/// Apply graph analysis to dependency list
fn apply_graph_analysis(deps: &mut [DepAnalysis], analysis: &GraphAnalysis) {
    for dep in deps.iter_mut() {
        dep.pagerank_score = *analysis.pagerank_scores.get(&dep.name).unwrap_or(&0.0);
        dep.in_degree = *analysis.in_degrees.get(&dep.name).unwrap_or(&0);
        dep.out_degree = *analysis.out_degrees.get(&dep.name).unwrap_or(&0);
        dep.is_bridge = analysis.bridges.contains(&dep.name);
        dep.is_orphan = analysis.orphans.contains(&dep.name);
        dep.transitive_count = *analysis.transitive_counts.get(&dep.name).unwrap_or(&0);
    }
}

/// Pareto analysis result - ROI for removing a dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParetoEntry {
    pub name: String,
    pub transitive_deps: usize,
    pub effort: ParetoEffort,
    pub roi: f32, // transitive_deps / effort_multiplier
    pub reason: String,
    pub category: DepCategory,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ParetoEffort {
    /// Easy: Just remove from Cargo.toml
    Low = 1,
    /// Medium: Requires small code changes
    Medium = 2,
    /// Hard: Requires significant refactoring
    High = 3,
}

impl ParetoEffort {
    fn multiplier(&self) -> f32 {
        match self {
            ParetoEffort::Low => 1.0,
            ParetoEffort::Medium => 2.0,
            ParetoEffort::High => 3.0,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            ParetoEffort::Low => "Low",
            ParetoEffort::Medium => "Medium",
            ParetoEffort::High => "High",
        }
    }
}

/// Calculate effort to remove a dependency based on its usage
fn estimate_effort(name: &str, category: DepCategory) -> ParetoEffort {
    // High effort: deeply integrated deps
    let high_effort = ["tokio", "serde", "clap", "anyhow", "thiserror", "tracing"];
    if high_effort.contains(&name) {
        return ParetoEffort::High;
    }

    // Medium effort: used in multiple places but replaceable
    let medium_effort = [
        "git2", "octocrab", "reqwest", "swc_ecma_parser", "swc_common",
        "swc_ecma_ast", "swc_ecma_visit", "rusqlite", "pest", "pest_derive",
    ];
    if medium_effort.contains(&name) {
        return ParetoEffort::Medium;
    }

    // Low effort if removable category or simple utility
    if matches!(category, DepCategory::Removable | DepCategory::DevOnly) {
        return ParetoEffort::Low;
    }

    // Default based on category
    match category {
        DepCategory::Heavy => ParetoEffort::Medium,
        DepCategory::Replaceable => ParetoEffort::Medium,
        _ => ParetoEffort::High,
    }
}

/// Run Pareto analysis using cargo tree for accurate transitive counts
fn run_pareto_analysis(deps: &[DepAnalysis], path: &Path) -> Vec<ParetoEntry> {
    let mut entries = Vec::new();

    // Only analyze removable, heavy, and replaceable deps
    let candidates: Vec<_> = deps
        .iter()
        .filter(|d| matches!(d.category, DepCategory::Removable | DepCategory::Heavy | DepCategory::Replaceable))
        .collect();

    for dep in candidates {
        // Get actual transitive count from cargo tree
        let transitive = get_transitive_count(&dep.name, path);

        let effort = estimate_effort(&dep.name, dep.category);
        let roi = transitive as f32 / effort.multiplier();

        entries.push(ParetoEntry {
            name: dep.name.clone(),
            transitive_deps: transitive,
            effort,
            roi,
            reason: dep.reason.clone(),
            category: dep.category,
        });
    }

    // Sort by ROI (highest first)
    entries.sort_by(|a, b| b.roi.partial_cmp(&a.roi).unwrap_or(std::cmp::Ordering::Equal));

    entries
}

/// Get transitive dependency count using cargo tree
fn get_transitive_count(dep_name: &str, path: &Path) -> usize {
    use std::process::Command;

    let output = Command::new("cargo")
        .args(["tree", "-p", dep_name, "--prefix", "none", "-e", "no-dev"])
        .current_dir(path)
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            // Count unique lines (each is a transitive dep)
            let count: HashSet<_> = stdout.lines().collect();
            count.len().saturating_sub(1) // Don't count self
        }
        _ => 0,
    }
}

/// Print Pareto analysis report
fn print_pareto_report(entries: &[ParetoEntry]) {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊  Pareto Analysis: 80/20 Dependency Removal");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("ROI = Transitive Deps Saved / Effort");
    println!("Higher ROI = Better bang for buck");
    println!();

    if entries.is_empty() {
        println!("No removable dependencies found.");
        return;
    }

    // Calculate 80% of total savings
    let total_transitive: usize = entries.iter().map(|e| e.transitive_deps).sum();
    let target_80 = (total_transitive as f32 * 0.8) as usize;

    println!("┌─────────────────────┬───────────┬────────┬────────┬─────────────────────────────┐");
    println!("│ Dependency          │ Trans.Deps│ Effort │ ROI    │ Reason                      │");
    println!("├─────────────────────┼───────────┼────────┼────────┼─────────────────────────────┤");

    let mut cumulative = 0;
    let mut marked_80 = false;
    for entry in entries.iter().take(20) {
        cumulative += entry.transitive_deps;
        let marker = if !marked_80 && cumulative >= target_80 {
            marked_80 = true;
            "← 80%"
        } else {
            ""
        };

        println!("│ {:<19} │ {:>9} │ {:>6} │ {:>6.1} │ {:<21} {:>5} │",
                 &entry.name[..entry.name.len().min(19)],
                 entry.transitive_deps,
                 entry.effort.label(),
                 entry.roi,
                 &entry.reason[..entry.reason.len().min(21)],
                 marker);
    }
    println!("└─────────────────────┴───────────┴────────┴────────┴─────────────────────────────┘");
    println!();

    // Summary
    let top_5_savings: usize = entries.iter().take(5).map(|e| e.transitive_deps).sum();
    let top_5_pct = if total_transitive > 0 {
        (top_5_savings as f32 / total_transitive as f32 * 100.0) as usize
    } else {
        0
    };

    println!("💡 Summary:");
    println!("   Total transitive deps from candidates: {}", total_transitive);
    println!("   Top 5 removals save: {} deps ({}% of total)", top_5_savings, top_5_pct);
    println!();

    // Actionable commands
    println!("🔧 Quick Wins (Low Effort, High ROI):");
    for entry in entries.iter().filter(|e| matches!(e.effort, ParetoEffort::Low) && e.roi > 10.0).take(5) {
        println!("   cargo rm {} # saves {} transitive deps", entry.name, entry.transitive_deps);
    }
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}

/// Sort mode for deps-audit output
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    /// Sort by transitive dependency count (most deps first)
    Transitive,
    /// Sort by estimated binary size (largest first)
    Size,
    /// Sort by PageRank criticality score (most critical first)
    PageRank,
    /// Sort alphabetically by name
    Name,
    /// Sort by category priority (removable first)
    Category,
}

impl SortMode {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "size" | "binary" | "kb" => SortMode::Size,
            "pagerank" | "rank" | "critical" => SortMode::PageRank,
            "name" | "alpha" | "alphabetical" => SortMode::Name,
            "category" | "cat" => SortMode::Category,
            _ => SortMode::Transitive, // default
        }
    }
}

/// Handle the deps-audit command
pub fn handle_deps_audit(
    path: &Path,
    format: &str,
    show_all: bool,
    pareto: bool,
    sort_by: &str,
) -> Result<()> {
    let cargo_toml = path.join("Cargo.toml");
    if !cargo_toml.exists() {
        anyhow::bail!("No Cargo.toml found at {}", path.display());
    }

    let (deps, dev_deps) = parse_cargo_toml(&cargo_toml)?;

    // Parse Cargo.lock for graph analysis
    let (all_packages, edges) = parse_cargo_lock(path)?;
    let direct_dep_names: Vec<String> = deps.iter().map(|(n, _, _)| n.clone()).collect();

    // Build and analyze dependency graph
    let graph_analysis = analyze_dependency_graph(&direct_dep_names, &all_packages, &edges);

    let mut all_deps: Vec<DepAnalysis> = deps
        .iter()
        .map(|(name, version, is_dev)| analyze_dep(name, version, *is_dev))
        .collect();

    let dev_analyses: Vec<DepAnalysis> = dev_deps
        .iter()
        .map(|(name, version, _)| analyze_dep(name, version, true))
        .collect();

    all_deps.extend(dev_analyses);

    // Apply graph analysis to populate graph metrics
    apply_graph_analysis(&mut all_deps, &graph_analysis);

    // Sort based on user preference
    let sort_mode = SortMode::from_str(sort_by);
    all_deps.sort_by(|a, b| {
        match sort_mode {
            SortMode::Transitive => b.transitive_count.cmp(&a.transitive_count),
            SortMode::Size => b.estimated_size_kb.cmp(&a.estimated_size_kb),
            SortMode::PageRank => b.pagerank_score.partial_cmp(&a.pagerank_score).unwrap_or(std::cmp::Ordering::Equal),
            SortMode::Name => a.name.cmp(&b.name),
            SortMode::Category => {
                let priority = |cat: DepCategory| match cat {
                    DepCategory::Removable => 0,
                    DepCategory::Heavy => 1,
                    DepCategory::Replaceable => 2,
                    DepCategory::DevOnly => 3,
                    DepCategory::Core => 4,
                    DepCategory::Sovereign => 5,
                };
                priority(a.category).cmp(&priority(b.category))
            }
        }
    });

    // Calculate stats
    let sovereign_count = all_deps.iter().filter(|d| d.category == DepCategory::Sovereign).count();
    let replaceable_count = all_deps.iter().filter(|d| d.category == DepCategory::Replaceable).count();
    let removable_count = all_deps.iter().filter(|d| d.category == DepCategory::Removable).count();
    let heavy_count = all_deps.iter().filter(|d| d.category == DepCategory::Heavy).count();
    let orphan_count = all_deps.iter().filter(|d| d.is_orphan).count();
    let bridge_count = all_deps.iter().filter(|d| d.is_bridge).count();
    let estimated_savings: usize = all_deps
        .iter()
        .filter(|d| matches!(d.category, DepCategory::Removable | DepCategory::Heavy | DepCategory::Replaceable))
        .map(|d| d.estimated_size_kb)
        .sum();

    // Top critical deps by PageRank
    let mut top_critical: Vec<(String, f32)> = all_deps
        .iter()
        .filter(|d| d.pagerank_score > 0.0)
        .map(|d| (d.name.clone(), d.pagerank_score))
        .collect();
    top_critical.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    top_critical.truncate(10);

    // Removal candidates: orphan deps that are removable or heavy
    let removal_candidates: Vec<String> = all_deps
        .iter()
        .filter(|d| d.is_orphan && matches!(d.category, DepCategory::Removable | DepCategory::Heavy))
        .map(|d| d.name.clone())
        .collect();

    // Generate recommendations
    let mut recommendations = Vec::new();

    if heavy_count > 0 {
        recommendations.push(format!(
            "Consider removing/replacing {} heavy dependencies to reduce binary size by ~{}KB",
            heavy_count, estimated_savings
        ));
    }

    // SWC recommendation
    let swc_deps: Vec<_> = all_deps.iter().filter(|d| d.name.starts_with("swc_")).collect();
    if !swc_deps.is_empty() {
        recommendations.push(
            "SWC dependencies add ~15MB. Consider using tree-sitter-typescript only.".to_string()
        );
    }

    // Git2 recommendation
    if all_deps.iter().any(|d| d.name == "git2") {
        recommendations.push(
            "git2 (libgit2) adds ~6MB. Consider shelling out to `git` CLI instead.".to_string()
        );
    }

    // Octocrab recommendation
    if all_deps.iter().any(|d| d.name == "octocrab") {
        recommendations.push(
            "octocrab adds ~5MB. Consider using ureq + serde_json for GitHub API.".to_string()
        );
    }

    // Graph-based recommendations
    if !removal_candidates.is_empty() {
        recommendations.push(format!(
            "Graph analysis: {} orphan deps are safe to remove: {}",
            removal_candidates.len(),
            removal_candidates.iter().take(5).cloned().collect::<Vec<_>>().join(", ")
        ));
    }

    // High transitive count warning
    let high_transitive: Vec<_> = all_deps
        .iter()
        .filter(|d| d.transitive_count > 50)
        .collect();
    if !high_transitive.is_empty() {
        let names: Vec<_> = high_transitive.iter().map(|d| format!("{}({})", d.name, d.transitive_count)).collect();
        recommendations.push(format!(
            "High transitive deps (each brings 50+ deps): {}",
            names.join(", ")
        ));
    }

    // Run Pareto analysis if requested (before consuming all_deps)
    if pareto {
        let pareto_entries = run_pareto_analysis(&all_deps, path);
        print_pareto_report(&pareto_entries);
        return Ok(());
    }

    let report = DepsAuditReport {
        total_deps: all_deps.len(),
        direct_deps: deps.len() + dev_deps.len(),
        transitive_deps: all_packages.len().saturating_sub(deps.len() + dev_deps.len()),
        sovereign_deps: sovereign_count,
        replaceable_deps: replaceable_count,
        removable_deps: removable_count,
        heavy_deps: heavy_count,
        orphan_deps: orphan_count,
        bridge_deps: bridge_count,
        estimated_savings_kb: estimated_savings,
        dependencies: if show_all {
            all_deps
        } else {
            all_deps.into_iter()
                .filter(|d| !matches!(d.category, DepCategory::Core | DepCategory::Sovereign))
                .collect()
        },
        recommendations,
        top_critical,
        removal_candidates,
    };

    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        "yaml" => {
            println!("{}", serde_yaml::to_string(&report)?);
        }
        _ => {
            print_text_report(&report);
        }
    }

    Ok(())
}

fn print_text_report(report: &DepsAuditReport) {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔍  Dependency Audit Report (with Graph Analysis)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("📊  Summary");
    println!("  Direct Dependencies:   {}", report.direct_deps);
    println!("  Transitive Deps:       {}", report.transitive_deps);
    println!("  Total (graph nodes):   {}", report.total_deps);
    println!("  Sovereign Stack:       {} ✅", report.sovereign_deps);
    println!("  Replaceable:           {} 🔄", report.replaceable_deps);
    println!("  Removable:             {} ❌", report.removable_deps);
    println!("  Heavy (bloat):         {} ⚠️", report.heavy_deps);
    println!("  Orphans (easy remove): {} 🎯", report.orphan_deps);
    println!("  Bridges (connectors):  {} 🌉", report.bridge_deps);
    println!("  Est. Savings:          ~{}KB (~{}MB)",
             report.estimated_savings_kb,
             report.estimated_savings_kb / 1024);
    println!();

    // Top critical deps by PageRank
    if !report.top_critical.is_empty() {
        println!("📈  Critical Dependencies (by PageRank)");
        println!("  ┌─────────────────────┬──────────┐");
        println!("  │ Dependency          │ Score    │");
        println!("  ├─────────────────────┼──────────┤");
        for (name, score) in report.top_critical.iter().take(5) {
            println!("  │ {:<19} │ {:.6} │", &name[..name.len().min(19)], score);
        }
        println!("  └─────────────────────┴──────────┘");
        println!("  (Higher = more deps depend on it, harder to remove)");
        println!();
    }

    // Group by category
    let removable: Vec<_> = report.dependencies.iter()
        .filter(|d| d.category == DepCategory::Removable)
        .collect();
    let heavy: Vec<_> = report.dependencies.iter()
        .filter(|d| d.category == DepCategory::Heavy)
        .collect();
    let replaceable: Vec<_> = report.dependencies.iter()
        .filter(|d| d.category == DepCategory::Replaceable)
        .collect();
    let dev_only: Vec<_> = report.dependencies.iter()
        .filter(|d| d.category == DepCategory::DevOnly)
        .collect();

    if !removable.is_empty() {
        println!("❌  Removable Dependencies");
        println!("  ┌─────────────────────┬────────────────────────────────────────┐");
        println!("  │ Dependency          │ Reason                                 │");
        println!("  ├─────────────────────┼────────────────────────────────────────┤");
        for dep in &removable {
            println!("  │ {:<19} │ {:<38} │",
                     &dep.name[..dep.name.len().min(19)],
                     &dep.reason[..dep.reason.len().min(38)]);
        }
        println!("  └─────────────────────┴────────────────────────────────────────┘");
        println!();
    }

    if !heavy.is_empty() {
        println!("⚠️   Heavy Dependencies (Bloat)");
        println!("  ┌─────────────────────┬──────────┬─────────────────────────────┐");
        println!("  │ Dependency          │ Size KB  │ Reason                      │");
        println!("  ├─────────────────────┼──────────┼─────────────────────────────┤");
        for dep in &heavy {
            println!("  │ {:<19} │ {:>8} │ {:<27} │",
                     &dep.name[..dep.name.len().min(19)],
                     dep.estimated_size_kb,
                     &dep.reason[..dep.reason.len().min(27)]);
        }
        println!("  └─────────────────────┴──────────┴─────────────────────────────┘");
        println!();
    }

    if !replaceable.is_empty() {
        println!("🔄  Replaceable with Sovereign Stack");
        println!("  ┌─────────────────────┬─────────────────────┬───────────────────┐");
        println!("  │ Dependency          │ Replacement         │ Benefit           │");
        println!("  ├─────────────────────┼─────────────────────┼───────────────────┤");
        for dep in &replaceable {
            let replacement = dep.replacement.as_deref().unwrap_or("-");
            println!("  │ {:<19} │ {:<19} │ {:<17} │",
                     &dep.name[..dep.name.len().min(19)],
                     &replacement[..replacement.len().min(19)],
                     &dep.reason[..dep.reason.len().min(17)]);
        }
        println!("  └─────────────────────┴─────────────────────┴───────────────────┘");
        println!();
    }

    if !dev_only.is_empty() {
        println!("🧪  Dev-Only Dependencies ({})", dev_only.len());
        let names: Vec<_> = dev_only.iter().map(|d| d.name.as_str()).collect();
        println!("  {}", names.join(", "));
        println!();
    }

    if !report.recommendations.is_empty() {
        println!("💡  Recommendations");
        for (i, rec) in report.recommendations.iter().enumerate() {
            println!("  {}. {}", i + 1, rec);
        }
        println!();
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Run with --all to see Core and Sovereign deps");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_sovereign_dep() {
        let dep = analyze_dep("trueno-graph", "0.1.10", false);
        assert_eq!(dep.category, DepCategory::Sovereign);
    }

    #[test]
    fn test_analyze_replaceable_dep() {
        let dep = analyze_dep("petgraph", "0.6.0", false);
        assert_eq!(dep.category, DepCategory::Replaceable);
        assert_eq!(dep.replacement, Some("trueno-graph".to_string()));
    }

    #[test]
    fn test_analyze_heavy_dep() {
        let dep = analyze_dep("swc_ecma_parser", "24.0.0", false);
        assert_eq!(dep.category, DepCategory::Heavy);
        assert!(dep.estimated_size_kb > 5000);
    }

    #[test]
    fn test_analyze_dev_dep() {
        let dep = analyze_dep("criterion", "0.5.0", true);
        assert_eq!(dep.category, DepCategory::DevOnly);
    }

    #[test]
    fn test_analyze_removable_dep() {
        let dep = analyze_dep("prettytable-rs", "0.10.0", false);
        assert_eq!(dep.category, DepCategory::Removable);
    }
}
