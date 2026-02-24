// ParallelLouvain core algorithm methods
// Included by parallel_louvain.rs - shares parent scope

impl ParallelLouvain {
    /// Create a new ParallelLouvain detector with default parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the resolution parameter.
    pub fn with_resolution(mut self, resolution: f64) -> Self {
        self.resolution = resolution;
        self
    }

    /// Set maximum iterations.
    pub fn with_max_iterations(mut self, max_iterations: usize) -> Self {
        self.max_iterations = max_iterations;
        self
    }

    /// Set minimum improvement threshold.
    pub fn with_min_improvement(mut self, min_improvement: f64) -> Self {
        self.min_improvement = min_improvement;
        self
    }

    /// Set number of threads (0 = use all available).
    pub fn with_num_threads(mut self, num_threads: usize) -> Self {
        self.num_threads = num_threads;
        self
    }

    /// Detect communities in an undirected graph.
    ///
    /// # Arguments
    /// * `graph` - Undirected graph to analyze
    ///
    /// # Returns
    /// Community assignment vector where `result[i]` is the community ID of node `i`
    pub fn detect(&self, graph: &UndirectedGraph) -> Vec<usize> {
        let n = graph.node_count();
        if n == 0 {
            return Vec::new();
        }

        // Build graph representation
        let graph_data = GraphData::from_graph(graph);

        // Initialize each node in its own community
        let mut communities: Vec<usize> = (0..n).collect();
        let mut best_modularity = self.calculate_modularity_internal(&graph_data, &communities);

        for _iteration in 0..self.max_iterations {
            // Local moving phase (parallel)
            let improved = self.local_moving_phase(&graph_data, &mut communities);

            if !improved {
                break;
            }

            // Calculate new modularity
            let new_modularity = self.calculate_modularity_internal(&graph_data, &communities);

            // Check for convergence
            if new_modularity - best_modularity < self.min_improvement {
                break;
            }

            best_modularity = new_modularity;
        }

        // Renumber communities to be contiguous
        self.renumber_communities(&mut communities);

        communities
    }

    /// Perform the local moving phase with parallel processing.
    ///
    /// Returns true if any node was moved.
    fn local_moving_phase(&self, graph_data: &GraphData, communities: &mut [usize]) -> bool {
        let n = communities.len();
        let improved = AtomicBool::new(false);

        // Calculate community data
        let community_data = CommunityData::new(communities, graph_data);

        // Process nodes in parallel batches
        // Each batch computes best moves, then we apply them sequentially
        let best_moves: Vec<Option<(usize, usize)>> = (0..n)
            .into_par_iter()
            .map(|node| self.find_best_move(node, communities[node], graph_data, &community_data))
            .collect();

        // Apply moves sequentially (to avoid race conditions)
        for (node, best_move) in best_moves.into_iter().enumerate() {
            if let Some((_old_comm, new_comm)) = best_move {
                if communities[node] != new_comm {
                    communities[node] = new_comm;
                    improved.store(true, Ordering::Relaxed);
                }
            }
        }

        improved.load(Ordering::Relaxed)
    }

    /// Find the best community move for a single node.
    ///
    /// Returns Some((old_community, new_community)) if a beneficial move exists.
    fn find_best_move(
        &self,
        node: usize,
        current_community: usize,
        graph_data: &GraphData,
        community_data: &CommunityData,
    ) -> Option<(usize, usize)> {
        let node_degree = graph_data.degrees[node];
        let total_weight = graph_data.total_weight;

        if total_weight == 0.0 {
            return None;
        }

        // Calculate current modularity contribution
        let current_gain = self.modularity_gain(
            node,
            current_community,
            graph_data,
            community_data,
            node_degree,
            total_weight,
        );

        // Find best neighbor community
        let mut best_community = current_community;
        let mut best_gain = current_gain;

        // Get neighbor communities
        let neighbor_communities = self.get_neighbor_communities(node, graph_data);

        for &neighbor_comm in &neighbor_communities {
            if neighbor_comm == current_community {
                continue;
            }

            let gain = self.modularity_gain(
                node,
                neighbor_comm,
                graph_data,
                community_data,
                node_degree,
                total_weight,
            );

            if gain > best_gain {
                best_gain = gain;
                best_community = neighbor_comm;
            }
        }

        if best_community != current_community {
            Some((current_community, best_community))
        } else {
            None
        }
    }

    /// Calculate modularity gain for moving a node to a community.
    fn modularity_gain(
        &self,
        node: usize,
        target_community: usize,
        graph_data: &GraphData,
        community_data: &CommunityData,
        node_degree: f64,
        total_weight: f64,
    ) -> f64 {
        // Sum of weights to target community
        let ki_in = graph_data.neighbor_weight_to_community(
            node,
            target_community,
            &community_data.node_to_community,
        );

        // Total degree of target community
        let sigma_tot = community_data
            .community_degrees
            .get(&target_community)
            .copied()
            .unwrap_or(0.0);

        // Modularity gain formula from Blondel et al.
        ki_in - self.resolution * (sigma_tot * node_degree) / (2.0 * total_weight)
    }

    /// Get unique communities of a node's neighbors.
    fn get_neighbor_communities(&self, node: usize, graph_data: &GraphData) -> Vec<usize> {
        graph_data.neighbors[node]
            .iter()
            .map(|(neighbor, _)| graph_data.degrees[*neighbor] as usize % graph_data.n) // Placeholder: actual community from neighbors
            .collect()
    }

    /// Calculate modularity of a community assignment.
    fn calculate_modularity_internal(&self, graph_data: &GraphData, communities: &[usize]) -> f64 {
        if graph_data.total_weight == 0.0 {
            return 0.0;
        }

        let m2 = 2.0 * graph_data.total_weight;
        let mut q = 0.0;

        // Calculate community degrees
        let mut community_degrees: HashMap<usize, f64> = HashMap::new();
        for (node, &community) in communities.iter().enumerate() {
            *community_degrees.entry(community).or_insert(0.0) += graph_data.degrees[node];
        }

        // Sum of internal edges
        for (&(source, target), &weight) in &graph_data.edge_weights {
            if communities[source] == communities[target] {
                q += weight;
            }
        }

        // Subtract expected value
        for degree in community_degrees.values() {
            q -= self.resolution * degree * degree / m2;
        }

        q / m2
    }

    /// Calculate modularity of a community assignment (public API).
    pub fn calculate_modularity(&self, graph: &UndirectedGraph, communities: &[usize]) -> f64 {
        let graph_data = GraphData::from_graph(graph);
        self.calculate_modularity_internal(&graph_data, communities)
    }

    /// Renumber communities to use contiguous IDs starting from 0.
    fn renumber_communities(&self, communities: &mut [usize]) {
        let mut mapping: HashMap<usize, usize> = HashMap::new();
        let mut next_id = 0;

        for community in communities.iter_mut() {
            let new_id = *mapping.entry(*community).or_insert_with(|| {
                let id = next_id;
                next_id += 1;
                id
            });
            *community = new_id;
        }
    }

    /// Get the number of unique communities in an assignment.
    pub fn num_communities(communities: &[usize]) -> usize {
        let unique: std::collections::HashSet<_> = communities.iter().collect();
        unique.len()
    }
}
