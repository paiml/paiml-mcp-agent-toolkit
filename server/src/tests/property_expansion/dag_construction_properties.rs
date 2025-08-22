//! Property-based tests for DAG construction
//! 
//! This module verifies that dependency graphs maintain acyclicity
//! and other structural invariants.

use proptest::prelude::*;
use std::collections::{HashMap, HashSet};

/// Module information for the dependency graph
#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub name: String,
    pub path: String,
    pub size: usize,
    pub complexity: u32,
}

/// Edge in the dependency graph
#[derive(Debug, Clone)]
pub struct DagEdge {
    pub from: String,
    pub to: String,
    pub weight: f64,
}

/// Dependency graph structure
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    pub nodes: HashMap<String, ModuleInfo>,
    pub edges: Vec<DagEdge>,
    pub ranks: HashMap<String, f64>,
}

impl DependencyGraph {
    /// Create a new dependency graph
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            ranks: HashMap::new(),
        }
    }
    
    /// Detect cycles in the graph using DFS
    pub fn detect_cycles(&self) -> Vec<Vec<String>> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut cycles = Vec::new();
        
        // Build adjacency list
        let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();
        for edge in &self.edges {
            adj_list.entry(edge.from.clone())
                .or_default()
                .push(edge.to.clone());
        }
        
        // DFS for each unvisited node
        for node in self.nodes.keys() {
            if !visited.contains(node) {
                let mut path = Vec::new();
                self.dfs_cycle_detect(
                    node,
                    &adj_list,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                    &mut cycles,
                );
            }
        }
        
        cycles
    }
    
    fn dfs_cycle_detect(
        &self,
        node: &str,
        adj_list: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());
        
        if let Some(neighbors) = adj_list.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    self.dfs_cycle_detect(
                        neighbor,
                        adj_list,
                        visited,
                        rec_stack,
                        path,
                        cycles,
                    );
                } else if rec_stack.contains(neighbor) {
                    // Found a cycle
                    if let Some(start_idx) = path.iter().position(|n| n == neighbor) {
                        cycles.push(path[start_idx..].to_vec());
                    }
                }
            }
        }
        
        path.pop();
        rec_stack.remove(node);
    }
    
    /// Compute PageRank for all nodes
    pub fn compute_pagerank(&mut self, iterations: usize) -> &HashMap<String, f64> {
        let n = self.nodes.len() as f64;
        if n == 0.0 {
            return &self.ranks;
        }
        
        // Initialize ranks
        for node in self.nodes.keys() {
            self.ranks.insert(node.clone(), 1.0 / n);
        }
        
        // Build incoming edges map
        let mut incoming: HashMap<String, Vec<String>> = HashMap::new();
        let mut outgoing_count: HashMap<String, usize> = HashMap::new();
        
        for edge in &self.edges {
            incoming.entry(edge.to.clone())
                .or_default()
                .push(edge.from.clone());
            *outgoing_count.entry(edge.from.clone()).or_default() += 1;
        }
        
        // PageRank iterations
        let damping = 0.85;
        for _ in 0..iterations {
            let mut new_ranks = HashMap::new();
            
            for node in self.nodes.keys() {
                let mut rank = (1.0 - damping) / n;
                
                if let Some(incomers) = incoming.get(node) {
                    for incomer in incomers {
                        let incomer_rank = self.ranks.get(incomer).unwrap_or(&0.0);
                        let outgoing = outgoing_count.get(incomer).unwrap_or(&1) as f64;
                        rank += damping * incomer_rank / outgoing;
                    }
                }
                
                new_ranks.insert(node.clone(), rank);
            }
            
            self.ranks = new_ranks;
        }
        
        &self.ranks
    }
}

/// Builder for constructing DAGs
pub struct DagBuilder {
    modules: Vec<ModuleInfo>,
    edges: Vec<(usize, usize)>,
}

impl DagBuilder {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
            edges: Vec::new(),
        }
    }
    
    pub fn add_modules(mut self, modules: Vec<ModuleInfo>) -> Self {
        self.modules = modules;
        self
    }
    
    pub fn add_edges(mut self, edges: Vec<(usize, usize)>) -> Self {
        self.edges = edges;
        self
    }
    
    pub fn build(self) -> DependencyGraph {
        let mut graph = DependencyGraph::new();
        
        // Add nodes
        for module in self.modules {
            graph.nodes.insert(module.name.clone(), module);
        }
        
        // Add edges (only if both nodes exist)
        for (from_idx, to_idx) in self.edges {
            if let (Some(from_module), Some(to_module)) = 
                (self.modules.get(from_idx), self.modules.get(to_idx)) {
                graph.edges.push(DagEdge {
                    from: from_module.name.clone(),
                    to: to_module.name.clone(),
                    weight: 1.0,
                });
            }
        }
        
        graph
    }
}

/// Generate a random DAG with specified parameters
fn generate_random_dag(num_modules: usize, edge_probability: f64) -> (Vec<ModuleInfo>, Vec<(usize, usize)>) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    // Generate modules
    let modules: Vec<ModuleInfo> = (0..num_modules)
        .map(|i| ModuleInfo {
            name: format!("module_{}", i),
            path: format!("src/module_{}.rs", i),
            size: rng.gen_range(100..10000),
            complexity: rng.gen_range(1..100),
        })
        .collect();
    
    // Generate edges (ensuring no cycles by only allowing edges from lower to higher indices)
    let mut edges = Vec::new();
    for i in 0..num_modules {
        for j in (i + 1)..num_modules {
            if rng.gen::<f64>() < edge_probability {
                edges.push((i, j));
            }
        }
    }
    
    (modules, edges)
}

// Arbitrary implementations for property testing
impl Arbitrary for ModuleInfo {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;
    
    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        (
            "[a-z][a-z0-9_]{2,20}",
            "[a-z0-9_/]+\\.rs",
            100usize..10000,
            1u32..100,
        ).prop_map(|(name, path, size, complexity)| {
            ModuleInfo {
                name,
                path,
                size,
                complexity,
            }
        }).boxed()
    }
}

prop_compose! {
    /// Strategy for generating module graphs
    fn arb_module_graph()
        (num_modules in 1..50usize)
        (
            num_modules in Just(num_modules),
            edge_probability in 0.0..0.3f64
        ) -> (Vec<ModuleInfo>, Vec<(usize, usize)>)
    {
        generate_random_dag(num_modules, edge_probability)
    }
}

proptest! {
    /// Property: DAG construction preserves acyclicity
    #[test]
    fn dag_construction_preserves_acyclicity(
        (modules, edges) in arb_module_graph()
    ) {
        let graph = DagBuilder::new()
            .add_modules(modules)
            .add_edges(edges)
            .build();
        
        // Property: No cycles in dependency graph
        let cycles = graph.detect_cycles();
        prop_assert!(
            cycles.is_empty(),
            "DAG construction introduced cycles: {:?}",
            cycles
        );
    }
    
    /// Property: All edges reference valid nodes
    #[test]
    fn dag_edges_reference_valid_nodes(
        (modules, edges) in arb_module_graph()
    ) {
        let graph = DagBuilder::new()
            .add_modules(modules)
            .add_edges(edges)
            .build();
        
        for edge in &graph.edges {
            prop_assert!(
                graph.nodes.contains_key(&edge.from),
                "Edge references non-existent source: {}",
                edge.from
            );
            prop_assert!(
                graph.nodes.contains_key(&edge.to),
                "Edge references non-existent target: {}",
                edge.to
            );
        }
    }
    
    /// Property: PageRank converges to valid distribution
    #[test]
    fn pagerank_convergence(
        (modules, edges) in arb_module_graph()
    ) {
        let mut graph = DagBuilder::new()
            .add_modules(modules)
            .add_edges(edges)
            .build();
        
        if graph.nodes.is_empty() {
            return Ok(());
        }
        
        let ranks = graph.compute_pagerank(100);
        
        // Property: PageRank sums to 1.0 (within epsilon)
        let sum: f64 = ranks.values().sum();
        prop_assert!(
            (sum - 1.0).abs() < 1e-6,
            "PageRank sum {} != 1.0",
            sum
        );
        
        // Property: All ranks are non-negative
        for (&rank) in ranks.values() {
            prop_assert!(
                rank >= 0.0,
                "Negative PageRank detected: {}",
                rank
            );
        }
    }
    
    /// Property: Empty graph has no cycles
    #[test]
    fn empty_graph_no_cycles(_dummy in Just(())) {
        let graph = DependencyGraph::new();
        let cycles = graph.detect_cycles();
        prop_assert!(cycles.is_empty());
    }
    
    /// Property: Single node graph has no cycles
    #[test]
    fn single_node_no_cycles(module in any::<ModuleInfo>()) {
        let mut graph = DependencyGraph::new();
        graph.nodes.insert(module.name.clone(), module);
        
        let cycles = graph.detect_cycles();
        prop_assert!(cycles.is_empty());
    }
    
    /// Property: Linear chain has no cycles
    #[test]
    fn linear_chain_no_cycles(num_modules in 2..20usize) {
        let modules: Vec<ModuleInfo> = (0..num_modules)
            .map(|i| ModuleInfo {
                name: format!("module_{}", i),
                path: format!("src/module_{}.rs", i),
                size: 1000,
                complexity: 10,
            })
            .collect();
        
        // Create linear chain: 0 -> 1 -> 2 -> ... -> n-1
        let edges: Vec<(usize, usize)> = (0..num_modules - 1)
            .map(|i| (i, i + 1))
            .collect();
        
        let graph = DagBuilder::new()
            .add_modules(modules)
            .add_edges(edges)
            .build();
        
        let cycles = graph.detect_cycles();
        prop_assert!(
            cycles.is_empty(),
            "Linear chain should have no cycles"
        );
    }
    
    /// Property: Adding reverse edge creates cycle
    #[test]
    fn reverse_edge_creates_cycle(num_modules in 3..10usize) {
        let modules: Vec<ModuleInfo> = (0..num_modules)
            .map(|i| ModuleInfo {
                name: format!("module_{}", i),
                path: format!("src/module_{}.rs", i),
                size: 1000,
                complexity: 10,
            })
            .collect();
        
        // Create chain with cycle: 0 -> 1 -> 2, 2 -> 0
        let mut edges: Vec<(usize, usize)> = (0..num_modules - 1)
            .map(|i| (i, i + 1))
            .collect();
        
        // Add reverse edge to create cycle
        edges.push((num_modules - 1, 0));
        
        let mut graph = DependencyGraph::new();
        
        // Add nodes
        for module in &modules {
            graph.nodes.insert(module.name.clone(), module.clone());
        }
        
        // Add edges (including the cycle)
        for (from_idx, to_idx) in &edges {
            if let (Some(from_module), Some(to_module)) = 
                (modules.get(*from_idx), modules.get(*to_idx)) {
                graph.edges.push(DagEdge {
                    from: from_module.name.clone(),
                    to: to_module.name.clone(),
                    weight: 1.0,
                });
            }
        }
        
        let cycles = graph.detect_cycles();
        prop_assert!(
            !cycles.is_empty(),
            "Should detect cycle with reverse edge"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dag_builder() {
        let modules = vec![
            ModuleInfo {
                name: "module_a".to_string(),
                path: "src/a.rs".to_string(),
                size: 1000,
                complexity: 10,
            },
            ModuleInfo {
                name: "module_b".to_string(),
                path: "src/b.rs".to_string(),
                size: 2000,
                complexity: 20,
            },
        ];
        
        let graph = DagBuilder::new()
            .add_modules(modules)
            .add_edges(vec![(0, 1)])
            .build();
        
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].from, "module_a");
        assert_eq!(graph.edges[0].to, "module_b");
    }
    
    #[test]
    fn test_cycle_detection() {
        let mut graph = DependencyGraph::new();
        
        // Add nodes
        graph.nodes.insert("a".to_string(), ModuleInfo {
            name: "a".to_string(),
            path: "a.rs".to_string(),
            size: 100,
            complexity: 5,
        });
        graph.nodes.insert("b".to_string(), ModuleInfo {
            name: "b".to_string(),
            path: "b.rs".to_string(),
            size: 100,
            complexity: 5,
        });
        
        // Add edge creating a cycle
        graph.edges.push(DagEdge {
            from: "a".to_string(),
            to: "b".to_string(),
            weight: 1.0,
        });
        graph.edges.push(DagEdge {
            from: "b".to_string(),
            to: "a".to_string(),
            weight: 1.0,
        });
        
        let cycles = graph.detect_cycles();
        assert!(!cycles.is_empty(), "Should detect cycle");
    }
    
    #[test]
    fn test_pagerank() {
        let modules = vec![
            ModuleInfo {
                name: "a".to_string(),
                path: "a.rs".to_string(),
                size: 100,
                complexity: 5,
            },
            ModuleInfo {
                name: "b".to_string(),
                path: "b.rs".to_string(),
                size: 100,
                complexity: 5,
            },
            ModuleInfo {
                name: "c".to_string(),
                path: "c.rs".to_string(),
                size: 100,
                complexity: 5,
            },
        ];
        
        let mut graph = DagBuilder::new()
            .add_modules(modules)
            .add_edges(vec![(0, 1), (0, 2), (1, 2)])
            .build();
        
        let ranks = graph.compute_pagerank(100);
        
        // Check sum is approximately 1.0
        let sum: f64 = ranks.values().sum();
        assert!((sum - 1.0).abs() < 1e-6);
        
        // Node c should have highest rank (most incoming edges)
        let rank_c = ranks.get("c").unwrap();
        let rank_a = ranks.get("a").unwrap();
        assert!(rank_c > rank_a);
    }
}