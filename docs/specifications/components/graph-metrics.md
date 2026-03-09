# Graph & Metrics

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 7

## DAG Construction

### Graph Types

| Type | Filter | Edges Included |
|------|--------|---------------|
| CallGraph | `filter_call_edges()` | Function calls only |
| ImportGraph | `filter_import_edges()` | Module imports only |
| Inheritance | `filter_inheritance_edges()` | Inherits edges only |
| FullDependency | No filter | All edge types |

### DagBuilder

Builds dependency graph from `ProjectContext`:
- `build_from_project(context)` — full graph
- `build_from_project_with_limit(context, 200)` — bounded to 200 nodes

### Edge Types

```rust
enum EdgeType {
    Calls,       // Function calls function
    Imports,     // Module imports module
    Inherits,    // Class extends class
    Implements,  // Class implements trait/interface
}
```

## PageRank Scoring

### Algorithm

10-iteration power method with damping factor 0.85:

```rust
// Precompute out-degrees for O(1) lookup
for edge in &graph.edges {
    out_degrees[from] += 1;
}

// 10 iterations sufficient for ranking
for _ in 0..10 {
    new_scores = vec![0.15; n];  // damping
    for edge in &graph.edges {
        new_scores[to] += 0.85 * scores[from] / out_degree[from];
    }
    scores = new_scores;
}
```

### In-Place Mutation

`add_pagerank_scores(mut graph)` takes ownership to avoid cloning:
```rust
pub fn add_pagerank_scores(mut graph: DependencyGraph) -> DependencyGraph {
    let scores = compute_pagerank_scores(&graph);
    for (idx, id) in node_ids.iter().enumerate() {
        graph.nodes.get_mut(id).unwrap()
            .metadata.insert("centrality", scores[idx].to_string());
    }
    graph
}
```

### Graph Pruning

`prune_graph_pagerank(graph, max_nodes)`:
1. Compute PageRank scores
2. Select top-k nodes by score
3. Remove edges referencing pruned nodes
4. Add centrality metadata to retained nodes

## Graph Descriptive Statistics

### Metrics Computed

| Metric | Description |
|--------|-------------|
| Node count | Total nodes in graph |
| Edge count | Total edges |
| Density | edges / (nodes * (nodes-1)) |
| Average degree | Mean in+out degree |
| Max in-degree | Most-depended-on node |
| Max out-degree | Most-dependent node |
| Connected components | Weakly connected subgraphs |
| Centrality scores | PageRank per node |

### Community Detection

Louvain algorithm via trueno-graph for identifying module clusters.

## Visualization

### Terminal Output

ASCII-based graph rendering:
- Node boxes with complexity coloring
- Edge arrows with type labels
- Legend for node types

### Interactive (Future)

Web-based D3.js visualization served via HTTP API.

## Key Files

| File | Purpose |
|------|---------|
| `src/services/dag_builder_core.rs` | DagBuilder implementation |
| `src/services/dag_builder_graph_algorithms.rs` | PageRank, filtering, pruning |
| `src/models/dag.rs` | DependencyGraph, NodeInfo, Edge types |

## References

- Consolidated from: graph-descriptive-statistics-v2, integrating-graph-visualizations-spec
