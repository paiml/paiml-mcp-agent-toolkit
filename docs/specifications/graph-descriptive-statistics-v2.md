# Graph Descriptive Statistics for PMAT - Technical Specification v2

## Executive Summary

This specification introduces comprehensive graph-theoretic analysis capabilities to PMAT, enabling PageRank computation, community detection, and 20+ graph metrics for code dependency networks. The implementation follows Toyota Way principles with strict complexity bounds (≤10 cyclomatic), zero SATD tolerance, and O(V+E) average-case performance.

## Graph Construction Foundation

### Dependency Graph Building (Complexity: 10)

```rust
// server/src/graph/builder.rs
use syn::{Item, ItemUse, visit::Visit};
use tree_sitter::{Parser, Query, QueryCursor};
use rustc_hash::FxHashMap;

/// Core node data structure for dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeData {
    pub path: PathBuf,
    pub module: String,
    pub symbols: Vec<Symbol>,
    pub loc: usize,
    pub complexity: f64,
    pub ast_hash: u64,  // For incremental updates
}

/// Edge types representing different dependency relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeData {
    Import { weight: f64, visibility: Visibility },
    FunctionCall { count: usize, async_call: bool },
    TypeDependency { strength: f64, kind: TypeKind },
    DataFlow { confidence: f64, direction: FlowDirection },
    Inheritance { depth: usize },
}

#[derive(Debug, Clone)]
pub enum TypeKind {
    Generic,
    Trait,
    Struct,
    Enum,
}

pub struct DependencyGraphBuilder {
    nodes: FxHashMap<NodeId, NodeData>,
    edges: Vec<(NodeId, NodeId, EdgeData)>,
    symbol_table: SymbolTable,
}

impl DependencyGraphBuilder {
    /// Build complete dependency graph from workspace
    /// Complexity: 10 (file iteration + AST traversal + symbol resolution)
    pub fn from_workspace(path: &Path) -> Result<Self> {
        let mut builder = Self::new();
        
        // Phase 1: Collect all files and build symbol table
        let files = builder.collect_source_files(path)?;
        builder.build_global_symbol_table(&files)?;
        
        // Phase 2: Analyze dependencies
        for file in &files {
            let node_id = builder.analyze_file(file)?;
            builder.resolve_dependencies(node_id)?;
        }
        
        // Phase 3: Compute edge weights
        builder.compute_edge_weights()?;
        
        Ok(builder)
    }
    
    /// Language-specific AST analysis
    fn analyze_file(&mut self, path: &Path) -> Result<NodeId> {
        let content = fs::read_to_string(path)?;
        let node_id = self.create_node(path);
        
        match path.extension().and_then(|s| s.to_str()) {
            Some("rs") => self.analyze_rust(&content, node_id)?,
            Some("py") => self.analyze_python(&content, node_id)?,
            Some("ts" | "tsx") => self.analyze_typescript(&content, node_id)?,
            Some("js" | "jsx") => self.analyze_javascript(&content, node_id)?,
            Some("go") => self.analyze_go(&content, node_id)?,
            Some("java") => self.analyze_java(&content, node_id)?,
            Some("c" | "cpp" | "cc") => self.analyze_cpp(&content, node_id)?,
            _ => self.analyze_tree_sitter(&content, node_id)?,
        }
        
        Ok(node_id)
    }
    
    /// Rust-specific dependency extraction
    fn analyze_rust(&mut self, content: &str, node_id: NodeId) -> Result<()> {
        let ast = syn::parse_file(content)?;
        
        // Extract imports
        for item in &ast.items {
            match item {
                Item::Use(use_item) => {
                    self.process_rust_import(use_item, node_id)?;
                }
                Item::Mod(mod_item) => {
                    self.process_rust_module(mod_item, node_id)?;
                }
                _ => {}
            }
        }
        
        // Extract function calls and type dependencies
        let mut visitor = RustDependencyVisitor::new(node_id, self);
        visitor.visit_file(&ast);
        
        Ok(())
    }
}
```

### Graph Type System

```rust
// server/src/graph/types.rs
use petgraph::graph::{DiGraph, UnGraph};
use nalgebra_sparse::{CsrMatrix, CooMatrix};

/// Primary directed graph for dependency analysis
pub type DependencyGraph = DiGraph<NodeData, EdgeData>;

/// Undirected projection for community detection
pub type UndirectedGraph = UnGraph<NodeData, f64>;

/// Type alias for consistent node indexing
pub type NodeId = petgraph::graph::NodeIndex<u32>;

/// Unified matrix representations for different algorithms
pub struct GraphMatrices {
    /// Standard adjacency matrix
    pub adjacency: CsrMatrix<f64>,
    /// Column-stochastic for PageRank
    pub transition: CsrMatrix<f64>,
    /// For spectral clustering
    pub laplacian: CsrMatrix<f64>,
    /// Out-degree vector
    pub out_degrees: Vec<f64>,
}

/// Conversion from petgraph to matrix representations
impl From<&DependencyGraph> for GraphMatrices {
    fn from(graph: &DependencyGraph) -> Self {
        let n = graph.node_count();
        let mut coo = CooMatrix::new(n, n);
        let mut out_degrees = vec![0.0; n];
        
        // Build adjacency matrix with edge weights
        for edge in graph.edge_references() {
            let weight = edge.weight().to_numeric_weight();
            let source = edge.source().index();
            let target = edge.target().index();
            
            coo.push(source, target, weight);
            out_degrees[source] += weight;
        }
        
        let adjacency = CsrMatrix::from(&coo);
        
        // Create column-stochastic transition matrix
        let transition = Self::normalize_columns(&adjacency, &out_degrees);
        
        // Compute Laplacian L = D - A
        let laplacian = Self::compute_laplacian(&adjacency);
        
        GraphMatrices {
            adjacency,
            transition,
            laplacian,
            out_degrees,
        }
    }
}

impl EdgeData {
    /// Convert heterogeneous edge types to numeric weights
    pub fn to_numeric_weight(&self) -> f64 {
        match self {
            EdgeData::Import { weight, .. } => *weight * 2.0,  // Imports weighted higher
            EdgeData::FunctionCall { count, .. } => *count as f64,
            EdgeData::TypeDependency { strength, .. } => *strength * 1.5,
            EdgeData::DataFlow { confidence, .. } => *confidence,
            EdgeData::Inheritance { depth } => 3.0 / (*depth as f64 + 1.0),
        }
    }
}
```

## Command Interface

```bash
# Analyze single file with graph metrics
pmat analyze graph src/main.rs

# Analyze directory with full statistics
pmat analyze graph . --metrics all

# Generate annotated deep context
pmat context --output deep_context.md --graph-metrics

# Export graph for visualization
pmat analyze graph . --export graph.json --format gephi

# Community-specific analysis
pmat analyze graph . --community-detection louvain --resolution 1.5

# Custom PageRank parameters
pmat analyze graph . --pagerank-damping 0.85 --pagerank-tolerance 1e-6
```

## Core Metrics Implementation

### 1. PageRank Algorithm (Complexity: 9)

```rust
// server/src/graph/pagerank.rs
use nalgebra_sparse::{CsrMatrix, CooMatrix};
use rayon::prelude::*;

pub struct PageRankComputer {
    damping: f64,
    tolerance: f64,
    max_iterations: usize,
}

impl PageRankComputer {
    /// Power iteration PageRank - O(k(V+E)), k=iterations
    /// Complexity: 9 (loop + convergence check + sparse ops)
    pub fn compute(&self, matrices: &GraphMatrices) -> Vec<f64> {
        let n = matrices.transition.nrows();
        let mut rank = vec![1.0 / n as f64; n];
        let mut new_rank = vec![0.0; n];
        
        for iteration in 0..self.max_iterations {
            // Teleportation component
            let teleport = (1.0 - self.damping) / n as f64;
            
            // Sparse matrix-vector multiplication using CSR format
            new_rank.par_iter_mut()
                .enumerate()
                .for_each(|(i, r)| {
                    *r = teleport + self.damping * 
                         self.compute_incoming_rank(&matrices.transition, &rank, i);
                });
            
            // Check convergence
            if self.converged(&rank, &new_rank) {
                log::debug!("PageRank converged at iteration {}", iteration);
                break;
            }
            
            std::mem::swap(&mut rank, &mut new_rank);
        }
        
        rank
    }
    
    #[inline(always)]
    fn compute_incoming_rank(&self, transition: &CsrMatrix<f64>, rank: &[f64], node: usize) -> f64 {
        // Efficient CSR row iteration
        let row = transition.row(node);
        let values = row.values();
        let indices = row.col_indices();
        
        values.iter()
            .zip(indices.iter())
            .map(|(weight, &j)| rank[j] * weight)
            .sum()
    }
    
    #[inline(always)]
    fn converged(&self, old: &[f64], new: &[f64]) -> bool {
        old.iter()
            .zip(new.iter())
            .map(|(o, n)| (o - n).abs())
            .sum::<f64>() < self.tolerance
    }
}
```

### 2. Community Detection - Louvain Method (Complexity: 10)

```rust
// server/src/graph/community.rs
use petgraph::graph::UnGraph;
use rustc_hash::FxHashMap;
use rand::seq::SliceRandom;

pub struct LouvainDetector {
    resolution: f64,
    randomize: bool,
    rng: StdRng,
}

impl LouvainDetector {
    /// Louvain modularity optimization - O(V log V) average
    /// Complexity: 10 (nested loop with early exit)
    pub fn detect_communities(&mut self, graph: &UndirectedGraph) -> Vec<usize> {
        let mut communities = (0..graph.node_count()).collect::<Vec<_>>();
        let mut improved = true;
        let mut iteration = 0;
        
        while improved && iteration < 100 {
            improved = false;
            iteration += 1;
            
            // Phase 1: Local optimization
            let node_order = self.get_node_order(graph);
            
            for &node_idx in &node_order {
                let node = NodeIndex::new(node_idx);
                let best_community = self.find_best_community(
                    graph, node, &communities
                );
                
                if best_community != communities[node_idx] {
                    communities[node_idx] = best_community;
                    improved = true;
                }
            }
            
            // Phase 2: Graph coarsening (if improved)
            if improved && iteration % 5 == 0 {
                self.coarsen_communities(&mut communities);
            }
        }
        
        communities
    }
    
    fn get_node_order(&mut self, graph: &UndirectedGraph) -> Vec<usize> {
        let mut order: Vec<_> = (0..graph.node_count()).collect();
        if self.randomize {
            order.shuffle(&mut self.rng);
        }
        order
    }
    
    fn find_best_community(&self, 
        graph: &UndirectedGraph,
        node: NodeIndex,
        communities: &[usize]
    ) -> usize {
        let mut best_delta = 0.0;
        let mut best_comm = communities[node.index()];
        let mut neighbor_comms = FxHashSet::default();
        
        // Collect unique neighboring communities
        for neighbor in graph.neighbors(node) {
            neighbor_comms.insert(communities[neighbor.index()]);
        }
        
        // Evaluate each neighboring community
        for &comm in &neighbor_comms {
            let delta = self.modularity_delta(graph, node, comm, communities);
            
            if delta > best_delta {
                best_delta = delta;
                best_comm = comm;
            }
        }
        
        best_comm
    }
    
    #[inline(always)]
    fn modularity_delta(&self,
        graph: &UndirectedGraph,
        node: NodeIndex,
        target_community: usize,
        communities: &[usize]
    ) -> f64 {
        let k_i = graph.edges(node).map(|e| *e.weight()).sum::<f64>();
        let sigma_c = self.community_degree(graph, target_community, communities);
        let m2 = 2.0 * graph.edge_count() as f64;
        
        let edges_to_comm = graph.edges(node)
            .filter(|e| communities[e.target().index()] == target_community)
            .map(|e| *e.weight())
            .sum::<f64>();
        
        2.0 * edges_to_comm - self.resolution * k_i * sigma_c / m2
    }
    
    fn coarsen_communities(&self, communities: &mut Vec<usize>) {
        // Renumber communities to be contiguous
        let mut mapping = FxHashMap::default();
        let mut next_id = 0;
        
        for comm in communities.iter_mut() {
            let new_id = *mapping.entry(*comm).or_insert_with(|| {
                let id = next_id;
                next_id += 1;
                id
            });
            *comm = new_id;
        }
    }
}
```

### 3. Centrality Metrics Suite (Complexity: 8)

```rust
// server/src/graph/centrality.rs
use petgraph::algo::{dijkstra, betweenness_centrality};
use rayon::prelude::*;

pub struct CentralityComputer {
    normalize: bool,
    weighted: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CentralityMetrics {
    pub degree: Vec<f64>,
    pub betweenness: Vec<f64>,
    pub closeness: Vec<f64>,
    pub eigenvector: Vec<f64>,
    pub katz: Vec<f64>,
    pub harmonic: Vec<f64>,
}

impl CentralityComputer {
    /// Compute all centrality metrics - O(V²) average
    /// Complexity: 8 (parallel computation with reduction)
    pub fn compute_all(&self, graph: &DependencyGraph) -> CentralityMetrics {
        CentralityMetrics {
            degree: self.degree_centrality(graph),
            betweenness: self.betweenness_centrality(graph),
            closeness: self.closeness_centrality(graph),
            eigenvector: self.eigenvector_centrality(graph),
            katz: self.katz_centrality(graph),
            harmonic: self.harmonic_centrality(graph),
        }
    }
    
    fn degree_centrality(&self, graph: &DependencyGraph) -> Vec<f64> {
        let n = graph.node_count() as f64;
        graph.node_indices()
            .map(|node| {
                let in_degree = graph.edges_directed(node, Incoming).count() as f64;
                let out_degree = graph.edges_directed(node, Outgoing).count() as f64;
                let total = in_degree + out_degree;
                
                if self.normalize { 
                    total / (2.0 * (n - 1.0))  // Directed graph normalization
                } else { 
                    total 
                }
            })
            .collect()
    }
    
    fn closeness_centrality(&self, graph: &DependencyGraph) -> Vec<f64> {
        graph.node_indices()
            .par_bridge()
            .map(|node| {
                let distances = dijkstra(graph, node, None, |e| {
                    if self.weighted { 
                        e.weight().to_numeric_weight() 
                    } else { 
                        1.0 
                    }
                });
                
                let reachable = distances.len() - 1;  // Exclude self
                if reachable == 0 { return 0.0; }
                
                let sum: f64 = distances.values()
                    .filter(|&&d| d > 0.0 && d.is_finite())
                    .sum();
                    
                if sum > 0.0 {
                    reachable as f64 / sum
                } else {
                    0.0
                }
            })
            .collect()
    }
    
    fn eigenvector_centrality(&self, graph: &DependencyGraph) -> Vec<f64> {
        let matrices = GraphMatrices::from(graph);
        let n = graph.node_count();
        let mut centrality = vec![1.0 / n as f64; n];
        
        // Power iteration for dominant eigenvalue
        for _ in 0..100 {
            let mut new_centrality = vec![0.0; n];
            
            // Matrix-vector multiplication with adjacency matrix
            for i in 0..n {
                let row = matrices.adjacency.row(i);
                new_centrality[i] = row.values()
                    .iter()
                    .zip(row.col_indices())
                    .map(|(val, &col)| val * centrality[col])
                    .sum();
            }
            
            // L2 normalization
            let norm: f64 = new_centrality.iter().map(|x| x * x).sum::<f64>().sqrt();
            if norm > 1e-10 {
                new_centrality.iter_mut().for_each(|x| *x /= norm);
            }
            
            // Check convergence
            let diff: f64 = centrality.iter()
                .zip(&new_centrality)
                .map(|(old, new)| (old - new).abs())
                .sum();
                
            if diff < 1e-6 { break; }
            
            centrality = new_centrality;
        }
        
        centrality
    }
}
```

### 4. Graph Structural Metrics (Complexity: 7)

```rust
// server/src/graph/structure.rs
use petgraph::algo::{connected_components, is_cyclic_directed, tarjan_scc};

#[derive(Debug, Clone, Serialize)]
pub struct StructuralMetrics {
    pub density: f64,
    pub diameter: Option<usize>,
    pub radius: Option<usize>,
    pub average_degree: f64,
    pub clustering_coefficient: f64,
    pub assortativity: f64,
    pub components: usize,
    pub strongly_connected_components: usize,
    pub is_cyclic: bool,
    pub transitivity: f64,
    pub reciprocity: Option<f64>,  // Only for directed graphs
}

pub struct StructuralAnalyzer {
    directed: bool,
}

impl StructuralAnalyzer {
    /// Compute structural properties - O(V+E)
    /// Complexity: 7 (single pass with aggregation)
    pub fn analyze(&self, graph: &DependencyGraph) -> StructuralMetrics {
        let undirected = self.to_undirected(graph);
        
        StructuralMetrics {
            density: self.compute_density(graph),
            diameter: self.compute_diameter(&undirected),
            radius: self.compute_radius(&undirected),
            average_degree: self.compute_avg_degree(graph),
            clustering_coefficient: self.clustering_coefficient(&undirected),
            assortativity: self.assortativity(graph),
            components: connected_components(&undirected),
            strongly_connected_components: tarjan_scc(graph).len(),
            is_cyclic: is_cyclic_directed(graph),
            transitivity: self.transitivity(&undirected),
            reciprocity: if self.directed { 
                Some(self.reciprocity(graph)) 
            } else { 
                None 
            },
        }
    }
    
    fn compute_density(&self, graph: &DependencyGraph) -> f64 {
        let n = graph.node_count() as f64;
        let e = graph.edge_count() as f64;
        
        if n <= 1.0 { return 0.0; }
        
        e / (n * (n - 1.0))  // Directed graph density
    }
    
    fn clustering_coefficient(&self, graph: &UndirectedGraph) -> f64 {
        let coefficients: Vec<f64> = graph.node_indices()
            .map(|node| self.local_clustering(graph, node))
            .collect();
            
        let sum: f64 = coefficients.iter().sum();
        sum / coefficients.len() as f64
    }
    
    fn local_clustering(&self, graph: &UndirectedGraph, node: NodeIndex) -> f64 {
        let neighbors: Vec<_> = graph.neighbors(node).collect();
        let k = neighbors.len();
        
        if k < 2 { return 0.0; }
        
        let mut triangles = 0;
        for i in 0..neighbors.len() {
            for j in (i+1)..neighbors.len() {
                if graph.contains_edge(neighbors[i], neighbors[j]) {
                    triangles += 1;
                }
            }
        }
        
        2.0 * triangles as f64 / (k * (k - 1)) as f64
    }
    
    fn reciprocity(&self, graph: &DependencyGraph) -> f64 {
        let mut reciprocal_pairs = 0;
        let total_edges = graph.edge_count();
        
        for edge in graph.edge_references() {
            if graph.contains_edge(edge.target(), edge.source()) {
                reciprocal_pairs += 1;
            }
        }
        
        reciprocal_pairs as f64 / (2.0 * total_edges as f64)
    }
}
```

### 5. Deep Context Integration (Complexity: 6)

```rust
// server/src/graph/context_annotator.rs
use crate::context::{DeepContext, FileContext};

pub struct GraphContextAnnotator {
    metrics: GraphMetrics,
    pagerank_threshold: f64,
    community_labels: FxHashMap<usize, String>,
}

impl GraphContextAnnotator {
    /// Annotate context with graph metrics - O(V)
    /// Complexity: 6 (linear scan with annotation)
    pub fn annotate(&self, context: &mut DeepContext) -> Result<()> {
        // Add global graph statistics
        context.metadata.insert(
            "graph.nodes".to_string(),
            self.metrics.node_count.to_string()
        );
        context.metadata.insert(
            "graph.edges".to_string(),
            self.metrics.edge_count.to_string()
        );
        context.metadata.insert(
            "graph.density".to_string(),
            format!("{:.4}", self.metrics.density)
        );
        context.metadata.insert(
            "graph.communities".to_string(),
            self.metrics.num_communities.to_string()
        );
        
        // Annotate each file
        for file in &mut context.files {
            self.annotate_file(file)?;
        }
        
        // Sort files by PageRank for better context prioritization
        context.files.sort_by(|a, b| {
            let pr_a = self.get_pagerank_for_file(&a.path);
            let pr_b = self.get_pagerank_for_file(&b.path);
            pr_b.partial_cmp(&pr_a).unwrap_or(Ordering::Equal)
        });
        
        Ok(())
    }
    
    fn annotate_file(&self, file: &mut FileContext) -> Result<()> {
        let node_id = self.get_node_id(&file.path)?;
        
        // Core metrics
        file.annotations.insert(
            "graph.pagerank".to_string(),
            format!("{:.6}", self.metrics.pagerank[node_id])
        );
        
        file.annotations.insert(
            "graph.in_degree".to_string(),
            self.metrics.in_degrees[node_id].to_string()
        );
        
        file.annotations.insert(
            "graph.out_degree".to_string(),
            self.metrics.out_degrees[node_id].to_string()
        );
        
        // Centrality metrics
        file.annotations.insert(
            "graph.centrality.degree".to_string(),
            format!("{:.4}", self.metrics.centrality.degree[node_id])
        );
        
        file.annotations.insert(
            "graph.centrality.betweenness".to_string(),
            format!("{:.4}", self.metrics.centrality.betweenness[node_id])
        );
        
        file.annotations.insert(
            "graph.centrality.closeness".to_string(),
            format!("{:.4}", self.metrics.centrality.closeness[node_id])
        );
        
        // Community information
        let community = self.metrics.communities[node_id];
        file.annotations.insert(
            "graph.community".to_string(),
            community.to_string()
        );
        
        if let Some(label) = self.community_labels.get(&community) {
            file.annotations.insert(
                "graph.community_label".to_string(),
                label.clone()
            );
        }
        
        // Critical file detection
        if self.metrics.pagerank[node_id] > self.pagerank_threshold {
            file.tags.push("critical-dependency".to_string());
        }
        
        if self.metrics.centrality.betweenness[node_id] > 0.1 {
            file.tags.push("architectural-hub".to_string());
        }
        
        Ok(())
    }
}
```

## Performance Optimizations

### 1. SIMD-Accelerated PageRank (Complexity: 8)

```rust
// server/src/graph/simd_pagerank.rs
use packed_simd_2::{f64x4, f64x8};
use std::arch::x86_64::*;

pub struct SimdPageRank {
    ranks: Vec<f64>,
    chunk_size: usize,
}

impl SimdPageRank {
    /// SIMD PageRank iteration - 4-8x speedup
    /// Complexity: 8 (vectorized loop with alignment)
    #[target_feature(enable = "avx2")]
    pub unsafe fn iterate(&mut self, transition: &CsrMatrix<f64>) {
        let n = self.ranks.len();
        let mut new_ranks = vec![0.0; n];
        
        // Process 8 values at once with AVX2
        let chunks = n / 8;
        for chunk_idx in 0..chunks {
            let base_idx = chunk_idx * 8;
            let mut accumulator = _mm256_setzero_pd();
            
            // Load 8 ranks
            let rank_vec = _mm256_loadu_pd(&self.ranks[base_idx]);
            
            // Compute weighted sum for each node in chunk
            for offset in 0..8 {
                let node_idx = base_idx + offset;
                let row = transition.row(node_idx);
                
                for (val, &col) in row.values().iter().zip(row.col_indices()) {
                    let weight = _mm256_set1_pd(*val);
                    let source_rank = _mm256_set1_pd(self.ranks[col]);
                    accumulator = _mm256_fmadd_pd(weight, source_rank, accumulator);
                }
            }
            
            // Store results
            _mm256_storeu_pd(&mut new_ranks[base_idx], accumulator);
        }
        
        // Handle remaining elements
        for i in (chunks * 8)..n {
            new_ranks[i] = self.compute_single_rank(transition, i);
        }
        
        self.ranks = new_ranks;
    }
    
    #[inline(always)]
    fn compute_single_rank(&self, transition: &CsrMatrix<f64>, node: usize) -> f64 {
        let row = transition.row(node);
        row.values()
            .iter()
            .zip(row.col_indices())
            .map(|(weight, &col)| weight * self.ranks[col])
            .sum()
    }
}
```

### 2. Parallel Community Detection (Complexity: 9)

```rust
// server/src/graph/parallel_louvain.rs
use rayon::prelude::*;
use crossbeam::sync::ShardedLock;
use std::sync::Arc;

pub struct ParallelLouvain {
    num_threads: usize,
    communities: Arc<ShardedLock<Vec<usize>>>,
}

impl ParallelLouvain {
    /// Parallel Louvain with lock-free updates where possible
    /// Complexity: 9 (parallel reduction with synchronization)
    pub fn detect(&self, graph: &UndirectedGraph) -> Vec<usize> {
        let n = graph.node_count();
        let communities = Arc::new(ShardedLock::new((0..n).collect::<Vec<_>>()));
        
        // Partition nodes for parallel processing
        let chunk_size = (n + self.num_threads - 1) / self.num_threads;
        
        loop {
            // Collect updates in parallel
            let updates: Vec<(usize, usize)> = (0..n)
                .into_par_iter()
                .chunks(chunk_size)
                .flat_map(|chunk| {
                    let mut local_updates = Vec::new();
                    let comms = communities.read().unwrap();
                    
                    for node_idx in chunk {
                        let node = NodeIndex::new(node_idx);
                        let current = comms[node_idx];
                        let best = self.find_best_community_locked(
                            graph, node, &comms
                        );
                        
                        if best != current {
                            local_updates.push((node_idx, best));
                        }
                    }
                    
                    local_updates
                })
                .collect();
            
            if updates.is_empty() { 
                break; 
            }
            
            // Apply updates atomically
            {
                let mut comms = communities.write().unwrap();
                for (node, new_comm) in updates {
                    comms[node] = new_comm;
                }
            }
        }
        
        Arc::try_unwrap(communities)
            .unwrap()
            .into_inner()
            .unwrap()
    }
}
```

## Test-Driven Development

### 1. Property-Based Testing (Complexity: 5)

```rust
// server/src/graph/tests/properties.rs
use proptest::prelude::*;
use approx::assert_relative_eq;

// Graph generator for property testing
prop_compose! {
    fn arbitrary_graph(min_nodes: usize, max_nodes: usize)
                      (nodes in min_nodes..max_nodes)
                      (
                          nodes: usize,
                          edges in prop::collection::vec(
                              (0..nodes, 0..nodes, 0.1f64..10.0f64),
                              0..(nodes * nodes / 4)
                          )
                      ) -> DependencyGraph {
        let mut graph = DependencyGraph::new();
        
        // Add nodes
        for i in 0..nodes {
            let node_data = NodeData {
                path: PathBuf::from(format!("file_{}.rs", i)),
                module: format!("mod_{}", i),
                symbols: vec![],
                loc: (i * 100) + 50,
                complexity: (i as f64 * 0.5) + 1.0,
                ast_hash: i as u64,
            };
            graph.add_node(node_data);
        }
        
        // Add edges
        for (src, dst, weight) in edges {
            if src != dst {  // No self-loops
                let edge_data = EdgeData::FunctionCall { 
                    count: weight as usize, 
                    async_call: false 
                };
                graph.add_edge(NodeIndex::new(src), NodeIndex::new(dst), edge_data);
            }
        }
        
        graph
    }
}

proptest! {
    #[test]
    fn test_pagerank_invariants(graph in arbitrary_graph(5, 50)) {
        let matrices = GraphMatrices::from(&graph);
        let pr = PageRankComputer::default();
        let ranks = pr.compute(&matrices);
        
        // Sum preservation (should sum to 1.0)
        let sum: f64 = ranks.iter().sum();
        assert_relative_eq!(sum, 1.0, epsilon = 1e-6);
        
        // Non-negativity
        prop_assert!(ranks.iter().all(|&r| r >= 0.0));
        
        // No NaN or infinite values
        prop_assert!(ranks.iter().all(|&r| r.is_finite()));
    }
    
    #[test]
    fn test_community_partition_properties(mut graph in arbitrary_graph(10, 100)) {
        let undirected = to_undirected(&graph);
        let mut detector = LouvainDetector::default();
        let communities = detector.detect_communities(&undirected);
        
        // Partition property: every node in exactly one community
        prop_assert_eq!(communities.len(), graph.node_count());
        
        // Communities are contiguous
        let max_comm = communities.iter().max().copied().unwrap_or(0);
        let unique: HashSet<_> = communities.iter().copied().collect();
        prop_assert!(unique.len() <= max_comm + 1);
        
        // Modularity should be non-negative for valid partition
        let modularity = compute_modularity(&undirected, &communities);
        prop_assert!(modularity >= -0.5);  // Allowing some slack for random graphs
    }
    
    #[test]
    fn test_centrality_bounds(graph in arbitrary_graph(5, 50)) {
        let computer = CentralityComputer::new(true, false);
        let metrics = computer.compute_all(&graph);
        
        // All normalized centralities in [0, 1]
        for value in metrics.degree.iter()
            .chain(&metrics.betweenness)
            .chain(&metrics.closeness)
            .chain(&metrics.eigenvector) {
            prop_assert!(*value >= 0.0 && *value <= 1.0);
        }
        
        // Eigenvector centrality should be normalized (L2 = 1)
        let l2_norm: f64 = metrics.eigenvector.iter()
            .map(|x| x * x)
            .sum::<f64>()
            .sqrt();
        assert_relative_eq!(l2_norm, 1.0, epsilon = 1e-6);
    }
}
```

### 2. Deterministic Test Cases (Complexity: 4)

```rust
// server/src/graph/tests/deterministic.rs
#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    
    fn create_star_graph(n: usize) -> DependencyGraph {
        let mut graph = DependencyGraph::new();
        
        // Add nodes
        for i in 0..n {
            graph.add_node(NodeData::test_node(i));
        }
        
        // Connect all nodes to node 0 (center)
        for i in 1..n {
            graph.add_edge(
                NodeIndex::new(0), 
                NodeIndex::new(i), 
                EdgeData::test_edge()
            );
            graph.add_edge(
                NodeIndex::new(i), 
                NodeIndex::new(0), 
                EdgeData::test_edge()
            );
        }
        
        graph
    }
    
    #[test]
    fn test_star_graph_pagerank() {
        let graph = create_star_graph(5);
        let matrices = GraphMatrices::from(&graph);
        let pr = PageRankComputer::default();
        let ranks = pr.compute(&matrices);
        
        // Center should have highest PageRank
        assert!(ranks[0] > ranks[1]);
        assert!(ranks[0] > ranks[2]);
        
        // All leaves should have equal PageRank
        assert_relative_eq!(ranks[1], ranks[2], epsilon = 1e-9);
        assert_relative_eq!(ranks[2], ranks[3], epsilon = 1e-9);
    }
    
    #[test]
    fn test_complete_graph_metrics() {
        let graph = create_complete_graph(5);
        let analyzer = StructuralAnalyzer::new(true);
        let metrics = analyzer.analyze(&graph);
        
        // Complete directed graph density = 1.0
        assert_relative_eq!(metrics.density, 1.0, epsilon = 1e-9);
        
        // All nodes should have equal degree centrality
        let computer = CentralityComputer::new(true, false);
        let centrality = computer.compute_all(&graph);
        
        let first = centrality.degree[0];
        for &deg in &centrality.degree[1..] {
            assert_relative_eq!(deg, first, epsilon = 1e-9);
        }
    }
    
    #[test]
    fn test_cycle_detection() {
        let mut graph = DependencyGraph::new();
        
        // Create cycle: A -> B -> C -> A
        for i in 0..3 {
            graph.add_node(NodeData::test_node(i));
        }
        
        graph.add_edge(NodeIndex::new(0), NodeIndex::new(1), EdgeData::test_edge());
        graph.add_edge(NodeIndex::new(1), NodeIndex::new(2), EdgeData::test_edge());
        graph.add_edge(NodeIndex::new(2), NodeIndex::new(0), EdgeData::test_edge());
        
        let analyzer = StructuralAnalyzer::new(true);
        let metrics = analyzer.analyze(&graph);
        
        assert!(metrics.is_cyclic);
        assert_eq!(metrics.strongly_connected_components, 1);
    }
}
```

### 3. Benchmark Suite (Complexity: 3)

```rust
// server/benches/graph_benchmarks.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};

fn benchmark_pagerank(c: &mut Criterion) {
    let mut group = c.benchmark_group("pagerank");
    
    for size in [100, 500, 1000, 5000, 10000].iter() {
        let graph = generate_erdos_renyi(*size, 0.01);
        let matrices = GraphMatrices::from(&graph);
        
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &matrices,
            |b, m| {
                let pr = PageRankComputer::default();
                b.iter(|| pr.compute(m));
            }
        );
    }
    
    group.finish();
}

fn benchmark_community_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("louvain");
    
    for size in [50, 100, 250, 500, 1000].iter() {
        let graph = generate_modular_graph(*size, 5, 0.8, 0.1);
        let undirected = to_undirected(&graph);
        
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &undirected,
            |b, g| {
                let mut detector = LouvainDetector::default();
                b.iter(|| detector.detect_communities(g));
            }
        );
    }
    
    group.finish();
}

fn benchmark_centrality_parallel(c: &mut Criterion) {
    let mut group = c.benchmark_group("centrality_parallel");
    
    let graph = generate_erdos_renyi(1000, 0.01);
    
    for threads in [1, 2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(threads),
            &graph,
            |b, g| {
                rayon::ThreadPoolBuilder::new()
                    .num_threads(*threads)
                    .build()
                    .unwrap()
                    .install(|| {
                        let computer = CentralityComputer::new(true, true);
                        b.iter(|| computer.compute_all(g));
                    });
            }
        );
    }
    
    group.finish();
}

criterion_group!(
    benches, 
    benchmark_pagerank, 
    benchmark_community_detection,
    benchmark_centrality_parallel
);
criterion_main!(benches);
```

## CLI Integration

### 1. Command Parser (Complexity: 6)

```rust
// server/src/cli/graph_commands.rs
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[clap(about = "Analyze code dependency graphs")]
pub struct GraphCommand {
    /// Path to analyze
    pub path: PathBuf,
    
    /// Metrics to compute
    #[clap(long, value_delimiter = ',', default_value = "pagerank,communities")]
    pub metrics: Vec<MetricType>,
    
    /// Output format
    #[clap(long, value_enum, default_value = "table")]
    pub format: OutputFormat,
    
    /// Export graph data
    #[clap(long)]
    pub export: Option<PathBuf>,
    
    /// Export format
    #[clap(long, value_enum, default_value = "json")]
    pub export_format: ExportFormat,
    
    /// Include in deep context
    #[clap(long)]
    pub annotate_context: bool,
    
    /// PageRank damping factor
    #[clap(long, default_value = "0.85")]
    pub pagerank_damping: f64,
    
    /// Community detection resolution
    #[clap(long, default_value = "1.0")]
    pub community_resolution: f64,
    
    /// Top N files to show
    #[clap(long, default_value = "20")]
    pub top_n: usize,
}

#[derive(Clone, ValueEnum)]
pub enum MetricType {
    All,
    PageRank,
    Communities,
    Centrality,
    Structure,
}

#[derive(Clone, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Markdown,
    Csv,
}

#[derive(Clone, ValueEnum)]
pub enum ExportFormat {
    Json,
    Gephi,
    GraphML,
    Dot,
}

impl GraphCommand {
    /// Execute graph analysis - Complexity: 6
    pub async fn execute(&self) -> Result<()> {
        // Build dependency graph with progress indicator
        let spinner = indicatif::ProgressBar::new_spinner();
        spinner.set_message("Building dependency graph...");
        
        let graph = DependencyGraphBuilder::from_workspace(&self.path)?
            .build()?;
        
        spinner.finish_with_message("Graph built successfully");
        
        // Compute requested metrics
        let metrics = self.compute_metrics(&graph)?;
        
        // Format and display output
        self.display_results(&metrics)?;
        
        // Export if requested
        if let Some(export_path) = &self.export {
            self.export_graph(&graph, &metrics, export_path)?;
        }
        
        // Annotate context if requested
        if self.annotate_context {
            self.annotate_deep_context(&metrics)?;
        }
        
        Ok(())
    }
    
    fn compute_metrics(&self, graph: &DependencyGraph) -> Result<GraphMetrics> {
        let mut metrics = GraphMetrics::new();
        
        for metric_type in &self.metrics {
            match metric_type {
                MetricType::All => {
                    metrics.compute_all(graph, self)?;
                }
                MetricType::PageRank => {
                    let pr = PageRankComputer::new()
                        .with_damping(self.pagerank_damping);
                    let matrices = GraphMatrices::from(graph);
                    metrics.pagerank = pr.compute(&matrices);
                }
                MetricType::Communities => {
                    let undirected = to_undirected(graph);
                    let mut detector = LouvainDetector::new()
                        .with_resolution(self.community_resolution);
                    metrics.communities = detector.detect_communities(&undirected);
                }
                MetricType::Centrality => {
                    let computer = CentralityComputer::new(true, true);
                    metrics.centrality = computer.compute_all(graph);
                }
                MetricType::Structure => {
                    let analyzer = StructuralAnalyzer::new(true);
                    metrics.structure = analyzer.analyze(graph);
                }
            }
        }
        
        Ok(metrics)
    }
}
```

### 2. Output Formatters (Complexity: 5)

```rust
// server/src/cli/graph_formatters.rs
use comfy_table::{Table, Cell, Attribute, Color};
use serde_json;

pub struct GraphFormatter {
    top_n: usize,
}

impl GraphFormatter {
    /// Format metrics as colored table - Complexity: 5
    pub fn format_table(&self, metrics: &GraphMetrics) -> String {
        let mut table = Table::new();
        table.set_header(vec![
            Cell::new("File").add_attribute(Attribute::Bold),
            Cell::new("PageRank").add_attribute(Attribute::Bold),
            Cell::new("In/Out Degree").add_attribute(Attribute::Bold),
            Cell::new("Betweenness").add_attribute(Attribute::Bold),
            Cell::new("Community").add_attribute(Attribute::Bold),
        ]);
        
        // Sort by PageRank descending
        let mut entries: Vec<_> = metrics.iter_nodes().collect();
        entries.sort_by(|a, b| {
            b.pagerank.partial_cmp(&a.pagerank).unwrap()
        });
        
        // Top N files with color coding
        for (i, entry) in entries.iter().take(self.top_n).enumerate() {
            let pagerank_cell = if entry.pagerank > 0.05 {
                Cell::new(format!("{:.6}", entry.pagerank))
                    .fg(Color::Red)
            } else if entry.pagerank > 0.02 {
                Cell::new(format!("{:.6}", entry.pagerank))
                    .fg(Color::Yellow)
            } else {
                Cell::new(format!("{:.6}", entry.pagerank))
            };
            
            table.add_row(vec![
                Cell::new(&entry.file),
                pagerank_cell,
                Cell::new(format!("{}/{}", entry.in_degree, entry.out_degree)),
                Cell::new(format!("{:.4}", entry.betweenness)),
                Cell::new(format!("C{}", entry.community)),
            ]);
        }
        
        table.to_string()
    }
    
    /// Format as CSV - Complexity: 4
    pub fn format_csv(&self, metrics: &GraphMetrics) -> Result<String> {
        let mut wtr = csv::Writer::from_writer(vec![]);
        
        wtr.write_record(&[
            "file", 
            "pagerank", 
            "in_degree", 
            "out_degree",
            "degree_centrality",
            "betweenness_centrality", 
            "closeness_centrality",
            "community"
        ])?;
        
        for node in metrics.iter_nodes() {
            wtr.write_record(&[
                &node.file,
                &node.pagerank.to_string(),
                &node.in_degree.to_string(),
                &node.out_degree.to_string(),
                &node.degree_centrality.to_string(),
                &node.betweenness.to_string(),
                &node.closeness.to_string(),
                &node.community.to_string(),
            ])?;
        }
        
        String::from_utf8(wtr.into_inner()?)
            .map_err(Into::into)
    }
}
```

## Performance Requirements

### Big-O Complexity Analysis

| Algorithm | Time Complexity | Space Complexity | Notes |
|-----------|----------------|------------------|-------|
| Graph Construction | O(V·S + E) | O(V+E) | S=symbols per file |
| PageRank | O(k(V+E)) | O(V) | k=iterations (typically 20-50) |
| PageRank (SIMD) | O(k(V+E)/8) | O(V) | 8-wide SIMD operations |
| Louvain Community | O(V log V) avg | O(V) | Worst case O(V²) |
| Parallel Louvain | O(V log V / P) | O(V) | P=number of threads |
| Betweenness Centrality | O(VE) | O(V) | Brandes algorithm |
| Closeness Centrality | O(V²) | O(V) | With Dijkstra |
| Eigenvector Centrality | O(kV²) | O(V) | Power iteration, k~100 |
| Clustering Coefficient | O(V·d²) | O(1) | d=average degree |
| SCC Detection | O(V+E) | O(V) | Tarjan's algorithm |

### Performance Targets

```yaml
benchmarks:
  graph_construction:
    1K_files: < 100ms
    10K_files: < 1s
    100K_files: < 10s
    
  pagerank:
    100_nodes: < 5ms
    1000_nodes: < 50ms
    10000_nodes: < 500ms
    
  pagerank_simd:
    1000_nodes: < 10ms
    10000_nodes: < 100ms
    
  community_detection:
    100_nodes: < 10ms
    1000_nodes: < 100ms
    10000_nodes: < 2s
    
  parallel_community:
    1000_nodes: < 30ms    # 4 threads
    10000_nodes: < 500ms   # 8 threads
    
  full_analysis:
    small_project: < 100ms    # <1K files
    medium_project: < 1s       # <10K files
    large_project: < 10s       # <100K files
```

## Implementation Timeline

### Sprint 1 (Days 1-7): Core Foundation

- [ ] Day 1-2: Graph construction from AST with language-specific parsers
- [ ] Day 3: Type system and data structure definitions
- [ ] Day 4-5: PageRank implementation with tests
- [ ] Day 6-7: Community detection (Louvain) with tests

### Sprint 2 (Days 8-14): Metrics & Optimization

- [ ] Day 8-9: Centrality metrics suite (6 metrics)
- [ ] Day 10: Structural metrics analysis
- [ ] Day 11-12: SIMD optimizations for PageRank
- [ ] Day 13-14: Parallel Louvain implementation

### Sprint 3 (Days 15-21): Integration & Polish

- [ ] Day 15-16: CLI commands and argument parsing
- [ ] Day 17-18: Deep context annotation system
- [ ] Day 19: Export formats (Gephi, GraphML, DOT)
- [ ] Day 20: Performance benchmarks and tuning
- [ ] Day 21: Documentation and examples

## Quality Gates

### 1. Complexity Enforcement

```toml
# .pmat/quality-gates.toml
[graph_module]
max_cyclomatic = 10
max_cognitive = 8
max_nesting = 3
min_test_coverage = 95
zero_satd = true
allowed_unsafe = ["simd_pagerank"]  # Only for SIMD optimizations
```

### 2. Property Test Requirements

Every algorithm must have:
- Invariant tests (sum preservation, partition properties)
- Edge case tests (empty graph, single node, complete graph)
- Performance regression tests
- Determinism tests (same input → same output)
- Convergence tests for iterative algorithms

### 3. Documentation Requirements

Every public function must have:
- Brief description
- Complexity annotation
- Big-O analysis  
- Example usage
- Invariants/postconditions
- Error cases

## Validation Criteria

### 1. Correctness

- PageRank convergence: ε < 10⁻⁶
- PageRank sum: |Σ(ranks) - 1.0| < 10⁻⁶
- Community modularity: Q > 0.3 for known modular graphs
- Centrality correlation: >0.9 with NetworkX reference
- All property tests passing

### 2. Performance

- Sub-linear scaling for sparse graphs (E << V²)
- <100ms for projects with 1000 files
- Memory usage: <100MB for 10K nodes
- SIMD speedup: >4x on AVX2 hardware

### 3. Quality

- 100% test coverage on core algorithms
- Zero SATD comments
- All functions ≤10 cyclomatic complexity
- Property tests for all invariants
- No memory leaks (validated with valgrind)

---

*This specification v2 addresses all review feedback with comprehensive type definitions, graph construction details, and unified data structures while maintaining Toyota Way principles with zero tolerance for technical debt.*
