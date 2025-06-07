# Dead Code Analysis Redux Specification

## Abstract

This specification defines a deterministic, high-performance dead code elimination system leveraging graph-theoretic algorithms on the existing `CrossLangReferenceGraph` infrastructure. The system employs a multi-phase approach combining fixed-point liveness propagation with dynamic dispatch resolution, achieving O(V+E) amortized complexity while maintaining cross-language semantic accuracy.

## 1. System Architecture

### 1.1 Core Components

```rust
pub struct DeadCodeAnalysisEngine {
    reference_graph: Arc<CrossLangReferenceGraph>,
    vtable_resolver: Arc<VTableResolver>,
    liveness_set: HierarchicalBitSet,
    worklist: BinaryHeap<WorkItem>,
    entry_points: FxHashSet<NodeId>,
    analysis_config: DeadCodeConfig,
}
```

The engine operates on four primary data structures:

1. **Reference Graph**: Directed multigraph G = (V, E) where V represents code entities (functions, methods, classes) and E represents reference relationships (calls, instantiations, imports)
2. **Liveness BitVector**: Space-efficient representation using `HierarchicalBitSet` with O(1) amortized set/test operations
3. **Priority Worklist**: Deterministic processing order via reverse post-order numbering
4. **Dynamic Dispatch Table**: Bipartite graph B = (C ∪ I, E_dispatch) mapping call sites to implementations

### 1.2 Analysis Phases

```
Phase 1: Graph Construction [O(n log n)]
  ├── AST Parsing
  ├── Cross-language Entity Resolution  
  └── Dynamic Dispatch Edge Inference

Phase 2: Entry Point Discovery [O(n)]
  ├── Main Function Detection
  ├── Public API Enumeration
  └── Configuration-based Additions

Phase 3: Fixed-Point Iteration [O(V+E)]
  ├── Forward Reachability
  ├── Dynamic Call Resolution
  └── Convergence Detection

Phase 4: Post-Processing [O(V)]
  ├── SCC Decomposition (optional)
  ├── Dead Code Ranking
  └── Report Generation
```

## 2. Algorithm Specification

### 2.1 Fixed-Point Liveness Propagation

```rust
fn compute_liveness(&mut self) -> Result<LivenessResult> {
    // Initialize worklist with entry points
    let mut iteration = 0;
    let mut changed = true;
    
    // Pre-compute reverse post-order for determinism
    let rpo = self.reference_graph.reverse_post_order();
    
    while changed && iteration < self.config.max_iterations {
        changed = false;
        
        while let Some(work_item) = self.worklist.pop() {
            let node_id = work_item.node_id;
            
            // Skip if already processed in this iteration
            if !self.should_process(node_id, iteration) {
                continue;
            }
            
            // Process direct edges
            for edge in self.reference_graph.out_edges(node_id) {
                if self.propagate_liveness(edge.target) {
                    changed = true;
                    self.worklist.push(WorkItem::new(edge.target, rpo[edge.target]));
                }
            }
            
            // Process dynamic dispatch edges
            if let Some(dispatch_sites) = self.get_dispatch_sites(node_id) {
                for site in dispatch_sites {
                    let targets = self.vtable_resolver.resolve(site)?;
                    for target in targets {
                        if self.propagate_liveness(target) {
                            changed = true;
                            self.worklist.push(WorkItem::new(target, rpo[target]));
                        }
                    }
                }
            }
        }
        
        iteration += 1;
    }
    
    Ok(self.build_result())
}
```

**Complexity Analysis**:
- Each node processed at most O(d) times where d = max degree
- Total work: O(V × d) = O(E) for sparse graphs
- BitSet operations: O(1) amortized via hierarchical structure

### 2.2 Dynamic Dispatch Resolution

```rust
impl VTableResolver {
    pub fn resolve(&self, call_site: CallSiteId) -> Vec<NodeId> {
        // Build bipartite graph for this dispatch
        let constraints = self.extract_type_constraints(call_site);
        let candidates = self.find_conforming_implementations(&constraints);
        
        // Apply Hopcroft-Karp for maximum matching
        let matching = self.bipartite_matching(call_site, candidates);
        
        // Filter by additional semantic constraints
        matching.into_iter()
            .filter(|impl_id| self.is_semantically_valid(call_site, *impl_id))
            .collect()
    }
}
```

### 2.3 Entry Point Discovery

```rust
#[derive(Debug, Clone)]
pub enum EntryPointKind {
    MainFunction,
    PublicApi { visibility: Visibility },
    ServerEndpoint { route: String },
    CliCommand { name: String },
    TestFunction { module: String },
    ConfigDriven { source: String },
}

impl EntryPointDiscovery {
    pub fn discover(&self, graph: &CrossLangReferenceGraph) -> FxHashSet<NodeId> {
        let mut entry_points = FxHashSet::default();
        
        // Phase 1: Syntactic discovery
        entry_points.extend(self.find_main_functions(graph));
        entry_points.extend(self.find_public_apis(graph));
        entry_points.extend(self.find_server_handlers(graph));
        
        // Phase 2: Semantic discovery
        entry_points.extend(self.find_cli_handlers(graph));
        entry_points.extend(self.find_exported_symbols(graph));
        
        // Phase 3: Configuration-based
        entry_points.extend(self.load_configured_entries());
        
        entry_points
    }
}
```

## 3. Determinism Guarantees

### 3.1 Node Ordering

```rust
#[derive(Eq, PartialEq)]
struct WorkItem {
    node_id: NodeId,
    rpo_number: u32,  // Reverse post-order number
    discovery_time: u64,
}

impl Ord for WorkItem {
    fn cmp(&self, other: &Self) -> Ordering {
        // Primary: RPO number (smaller = higher priority)
        self.rpo_number.cmp(&other.rpo_number)
            // Secondary: Discovery time (earlier = higher priority)
            .then_with(|| self.discovery_time.cmp(&other.discovery_time))
            // Tertiary: Node ID (for absolute determinism)
            .then_with(|| self.node_id.cmp(&other.node_id))
    }
}
```

### 3.2 Hash Function Stability

```rust
// Use FxHash for performance with deterministic seed
type NodeIdHasher = FxHasher;
type NodeIdMap<V> = HashMap<NodeId, V, BuildHasherDefault<NodeIdHasher>>;

// Ensure stable iteration order when needed
let sorted_nodes: Vec<_> = node_map.keys()
    .copied()
    .sorted_unstable()
    .collect();
```

## 4. Cross-Language Semantics

### 4.1 Rust-Specific Handling

```rust
impl RustSemantics {
    fn is_reachable(&self, item: &RustAstItem) -> bool {
        match item {
            // Trait implementations always live if trait is used
            RustAstItem::Impl { trait_ref: Some(_), .. } => true,
            
            // Generic functions live if any instantiation is live
            RustAstItem::Function { generics, .. } if !generics.is_empty() => {
                self.has_live_instantiation(item)
            }
            
            // #[test] functions based on configuration
            RustAstItem::Function { attrs, .. } => {
                !attrs.iter().any(|a| a.name == "test") || self.config.include_tests
            }
            
            _ => false,
        }
    }
}
```

### 4.2 TypeScript/JavaScript Handling

```rust
impl TypeScriptSemantics {
    fn resolve_dynamic_import(&self, import: &DynamicImport) -> Vec<NodeId> {
        // Conservative: mark all exports from dynamically imported modules
        if import.is_computed() {
            vec![] // Cannot statically resolve
        } else {
            self.resolve_module(&import.module_specifier)
                .map(|m| m.exported_symbols())
                .unwrap_or_default()
        }
    }
}
```

### 4.3 Python Handling

```rust
impl PythonSemantics {
    fn handle_getattr(&self, obj: &PyObject, attr: &str) -> Option<NodeId> {
        // Conservative handling of dynamic attribute access
        match self.infer_type(obj) {
            Some(TypeInfo::Class(class_id)) => {
                self.lookup_attribute(class_id, attr)
            }
            _ => None, // Cannot resolve statically
        }
    }
}
```

## 5. Performance Optimizations

### 5.1 Incremental Analysis

```rust
pub struct IncrementalState {
    previous_liveness: HierarchicalBitSet,
    changed_files: FxHashSet<PathBuf>,
    invalidated_nodes: FxHashSet<NodeId>,
}

impl DeadCodeAnalysisEngine {
    pub fn analyze_incremental(&mut self, state: &IncrementalState) -> Result<LivenessResult> {
        // Only reprocess affected subgraph
        let affected_scc = self.compute_affected_sccs(&state.changed_files);
        
        // Restore previous state for unaffected nodes
        self.liveness_set = state.previous_liveness.clone();
        
        // Run analysis only on affected components
        self.analyze_subgraph(affected_scc)
    }
}
```

### 5.2 Parallel SCC Processing

```rust
use rayon::prelude::*;

impl DeadCodeAnalysisEngine {
    fn analyze_sccs_parallel(&mut self) -> Result<()> {
        let sccs = self.compute_sccs();
        let scc_dag = self.build_scc_dag(&sccs);
        
        // Process SCCs in topological order, parallelizing within levels
        for level in scc_dag.topological_levels() {
            level.par_iter()
                .map(|scc_id| self.analyze_scc(&sccs[*scc_id]))
                .collect::<Result<Vec<_>>>()?;
        }
        
        Ok(())
    }
}
```

## 6. Integration Points

### 6.1 Configuration Schema

```toml
[dead_code]
# Analysis parameters
max_iterations = 100
include_tests = false
confidence_threshold = 0.8

# Entry points
[[dead_code.entry_points]]
kind = "function"
pattern = "src/bin/*/main"

[[dead_code.entry_points]]
kind = "public_api"
crate = "server"

# Language-specific
[dead_code.rust]
analyze_macros = true
trait_impl_heuristic = "conservative"

[dead_code.typescript]
strict_mode = true
ignore_ambient = true
```

### 6.2 Output Format

```rust
#[derive(Serialize)]
pub struct DeadCodeReport {
    pub summary: DeadCodeSummary,
    pub files: Vec<FileDeadCodeMetrics>,
    pub hotspots: Vec<DeadCodeHotspot>,
    pub suggestions: Vec<RemediationSuggestion>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scc_analysis: Option<SccDecomposition>,
}

#[derive(Serialize)]
pub struct DeadCodeHotspot {
    pub module: String,
    pub dead_lines: usize,
    pub confidence: f64,
    pub impact: ImpactLevel,
    pub connected_dead_code: Vec<NodeId>,
}
```

## 7. Validation & Testing

### 7.1 Property-Based Tests

```rust
#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn liveness_monotonic(graph in arb_reference_graph()) {
            let engine = DeadCodeAnalysisEngine::new(graph);
            let result1 = engine.analyze()?;
            
            // Adding edges should only increase liveness
            let augmented = graph.add_random_edges(10);
            let engine2 = DeadCodeAnalysisEngine::new(augmented);
            let result2 = engine2.analyze()?;
            
            prop_assert!(result1.live_nodes.is_subset(&result2.live_nodes));
        }
    }
}
```

### 7.2 Benchmarks

```rust
#[bench]
fn bench_linux_kernel_subset(b: &mut Bencher) {
    let graph = load_kernel_callgraph("vmlinux.graph");
    let engine = DeadCodeAnalysisEngine::new(graph);
    
    b.iter(|| {
        engine.analyze().unwrap()
    });
}

// Expected performance on 100K nodes, 500K edges:
// - Initial analysis: ~250ms
// - Incremental update (1% change): ~15ms
// - Memory usage: ~40MB
```

## 8. Future Extensions

1. **Speculative Devirtualization**: Use profile-guided optimization data to prune unlikely dynamic dispatch targets
2. **Escape Analysis Integration**: Leverage escape analysis to prove certain closures/objects never escape, enabling more precise liveness
3. **Distributed Analysis**: Shard the reference graph for massive codebases using consistent hashing on SCCs
4. **Machine Learning Heuristics**: Train models on historical dead code patterns to improve entry point discovery

## References

1. Tarjan, R. (1972). "Depth-First Search and Linear Graph Algorithms"
2. Lengauer & Tarjan (1979). "A Fast Algorithm for Finding Dominators"
3. Grove et al. (1997). "Call Graph Construction in Object-Oriented Languages"
4. Tip et al. (2011). "What's Dead May Never Die: An Empirical Study of Dead Code"