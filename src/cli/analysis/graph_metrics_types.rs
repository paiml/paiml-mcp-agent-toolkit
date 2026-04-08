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

                let is_better = distances.get(&neighbor).map_or(true, |&d| new_dist < d);

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
/// Node metrics.
pub struct NodeMetrics {
    pub name: String,
    pub degree_centrality: f64,
    pub betweenness_centrality: f64,
    pub closeness_centrality: f64,
    pub pagerank: f64,
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
