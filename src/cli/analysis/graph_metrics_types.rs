// Graph types: NodeIndex, SimpleGraph, NodeMetrics, GraphMetricsResult

/// Node index for the simple graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct NodeIndex(usize);

impl NodeIndex {
    
    fn index(self) -> usize {
        self.0
    }
}

/// A simple directed graph with String nodes and unit edges
struct SimpleGraph {
    nodes: Vec<String>,
    /// Adjacency list: outgoing edges
    outgoing: Vec<Vec<usize>>,
    /// Adjacency list: incoming edges
    incoming: Vec<Vec<usize>>,
}

impl SimpleGraph {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            outgoing: Vec::new(),
            incoming: Vec::new(),
        }
    }

    fn add_node(&mut self, name: String) -> NodeIndex {
        let idx = self.nodes.len();
        self.nodes.push(name);
        self.outgoing.push(Vec::new());
        self.incoming.push(Vec::new());
        NodeIndex(idx)
    }

    fn add_edge(&mut self, from: NodeIndex, to: NodeIndex) {
        self.outgoing[from.0].push(to.0);
        self.incoming[to.0].push(from.0);
    }

    fn node_count(&self) -> usize {
        self.nodes.len()
    }

    fn edge_count(&self) -> usize {
        self.outgoing.iter().map(|v| v.len()).sum()
    }

    fn node_indices(&self) -> impl Iterator<Item = NodeIndex> {
        (0..self.nodes.len()).map(NodeIndex)
    }

    fn get_node(&self, idx: NodeIndex) -> &String {
        &self.nodes[idx.0]
    }

    fn out_degree(&self, idx: NodeIndex) -> usize {
        self.outgoing[idx.0].len()
    }

    fn in_degree(&self, idx: NodeIndex) -> usize {
        self.incoming[idx.0].len()
    }

    fn outgoing_edges(&self, idx: NodeIndex) -> &[usize] {
        &self.outgoing[idx.0]
    }

    /// Distinct neighbours of `idx` with edge direction ignored, self-loops
    /// dropped. This is the neighbourhood a local clustering coefficient is
    /// defined over.
    fn undirected_neighbors(&self, idx: NodeIndex) -> Vec<usize> {
        let mut neighbors: Vec<usize> = self.outgoing[idx.0]
            .iter()
            .chain(self.incoming[idx.0].iter())
            .copied()
            .filter(|&n| n != idx.0)
            .collect();
        neighbors.sort_unstable();
        neighbors.dedup();
        neighbors
    }

    /// Every edge as an unordered pair, so `(a,b)` and `(b,a)` are one entry.
    fn undirected_edge_set(&self) -> std::collections::HashSet<(usize, usize)> {
        let mut edges = std::collections::HashSet::new();
        for (from, targets) in self.outgoing.iter().enumerate() {
            for &to in targets {
                if from != to {
                    edges.insert((from.min(to), from.max(to)));
                }
            }
        }
        edges
    }

    /// Connected-component id for every node, in the same numbering
    /// `connected_components` counts.
    ///
    /// Iterative on purpose: `dfs_undirected` recurses once per node, and this
    /// runs over whole repositories (4,000+ nodes in one component).
    fn component_ids(&self) -> Vec<usize> {
        let n = self.node_count();
        let mut ids = vec![usize::MAX; n];
        let mut next_id = 0;

        for start in 0..n {
            if ids[start] != usize::MAX {
                continue;
            }
            let mut stack = vec![start];
            ids[start] = next_id;
            while let Some(node) = stack.pop() {
                for &neighbor in self.outgoing[node].iter().chain(self.incoming[node].iter()) {
                    if ids[neighbor] == usize::MAX {
                        ids[neighbor] = next_id;
                        stack.push(neighbor);
                    }
                }
            }
            next_id += 1;
        }

        ids
    }

    /// Dijkstra's algorithm for shortest paths
    fn dijkstra(&self, source: NodeIndex, target: Option<NodeIndex>) -> HashMap<NodeIndex, i32> {
        use std::collections::BinaryHeap;

        let mut distances: HashMap<NodeIndex, i32> = HashMap::new();
        let mut heap = BinaryHeap::new();

        distances.insert(source, 0);
        heap.push(std::cmp::Reverse((0, source)));

        while let Some(std::cmp::Reverse((dist, node))) = heap.pop() {
            if let Some(&best) = distances.get(&node) {
                if dist > best {
                    continue;
                }
            }

            // Early exit if we found the target
            if let Some(t) = target {
                if node == t {
                    return distances;
                }
            }

            for &neighbor_idx in &self.outgoing[node.0] {
                let neighbor = NodeIndex(neighbor_idx);
                let new_dist = dist + 1;

                let is_better = distances.get(&neighbor).is_none_or(|&d| new_dist < d);

                if is_better {
                    distances.insert(neighbor, new_dist);
                    heap.push(std::cmp::Reverse((new_dist, neighbor)));
                }
            }
        }

        distances
    }

    /// Connected components using BFS/DFS (treats graph as undirected)
    fn connected_components(&self) -> usize {
        let n = self.node_count();
        if n == 0 {
            return 0;
        }

        let mut visited = vec![false; n];
        let mut count = 0;

        for start in 0..n {
            if !visited[start] {
                self.dfs_undirected(start, &mut visited);
                count += 1;
            }
        }

        count
    }

    fn dfs_undirected(&self, node: usize, visited: &mut [bool]) {
        if visited[node] {
            return;
        }
        visited[node] = true;

        // Follow outgoing edges
        for &neighbor in &self.outgoing[node] {
            if !visited[neighbor] {
                self.dfs_undirected(neighbor, visited);
            }
        }

        // Follow incoming edges (treat as undirected)
        for &neighbor in &self.incoming[node] {
            if !visited[neighbor] {
                self.dfs_undirected(neighbor, visited);
            }
        }
    }

    /// Get edge endpoints for GraphML export
    fn edge_endpoints(&self) -> Vec<(NodeIndex, NodeIndex)> {
        let mut edges = Vec::new();
        for (from_idx, targets) in self.outgoing.iter().enumerate() {
            for &to_idx in targets {
                edges.push((NodeIndex(from_idx), NodeIndex(to_idx)));
            }
        }
        edges
    }
}

// Public types

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Per-node graph metrics.
///
/// Every measure the `--metrics` selection can leave uncomputed is an
/// `Option<f64>`, and `None` means EXACTLY "this run did not compute it" — it
/// serialises as `null` and renders as `n/a`.
///
/// These were plain `f64` seeded with a struct initializer (`pagerank: 1.0/N`,
/// `closeness_centrality: 0.0`) that only the selected metrics overwrote. On a
/// 4,603-node graph `--metrics centrality` therefore published the pagerank
/// INITIALIZER — 0.00021724961981316532 == 1/4603 to the last digit, byte
/// identical for every node, hub and leaf alike — as a measured importance
/// score, and `filter_results` applied `--min-centrality` to a closeness that
/// had never been computed. An unmeasured metric is not a metric of 0.
pub struct NodeMetrics {
    pub name: String,
    /// Always computed: degree is read straight off the adjacency lists, so it
    /// costs nothing and needs no metric selection.
    pub degree_centrality: f64,
    /// `--metrics betweenness` (or `all`); `None` otherwise.
    pub betweenness_centrality: Option<f64>,
    /// `--metrics closeness` (or `all`); `None` otherwise.
    pub closeness_centrality: Option<f64>,
    /// `--metrics page-rank` (or `all`); `None` otherwise.
    pub pagerank: Option<f64>,
    /// Local clustering coefficient: `--metrics clustering` (or `all`).
    ///
    /// `Clustering` was an advertised `--metrics` value with no implementation
    /// and no output field, so `--metrics clustering` returned a document
    /// byte-identical to `--metrics centrality` and exit 0.
    pub clustering_coefficient: Option<f64>,
    /// Connected-component id: `--metrics components` (or `all`).
    ///
    /// The graph-wide `connected_components` count is emitted unconditionally;
    /// this is the per-node membership that makes the selection mean something.
    pub component_id: Option<usize>,
    pub in_degree: usize,
    pub out_degree: usize,
}

#[derive(Debug, Serialize)]
/// Result of graph metrics operation.
pub struct GraphMetricsResult {
    pub nodes: Vec<NodeMetrics>,
    pub total_nodes: usize,
    pub total_edges: usize,
    pub density: f64,
    pub average_degree: f64,
    pub max_degree: usize,
    pub connected_components: usize,
}
