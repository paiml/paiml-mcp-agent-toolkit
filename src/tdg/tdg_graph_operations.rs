impl TdgGraph {
    /// Create new TDG graph
    #[must_use]
    pub fn new() -> Self {
        Self {
            graph: CsrGraph::new(),
            node_map: HashMap::new(),
            reverse_node_map: HashMap::new(),
            criticality_scores: HashMap::new(),
            next_node_id: 0,
        }
    }

    /// Add function to graph (O(1))
    ///
    /// # Arguments
    ///
    /// * `name` - Function name (unique identifier)
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    ///
    /// # Errors
    ///
    /// Returns error if function already exists (duplicate name)
    pub fn add_function(&mut self, name: String) -> Result<()> {
        // Check for duplicates
        if self.node_map.contains_key(&name) {
            anyhow::bail!("Duplicate function name: {}", name);
        }

        // Create node
        let node_id = NodeId(self.next_node_id);
        self.next_node_id += 1;

        // Store mappings
        self.node_map.insert(name.clone(), node_id);
        self.reverse_node_map.insert(node_id, name.clone());

        // Set node name in graph (trueno-graph pattern)
        self.graph.set_node_name(node_id, name);

        Ok(())
    }

    /// Add edge between functions (e.g., function calls function)
    ///
    /// # Arguments
    ///
    /// * `from` - Caller function name
    /// * `to` - Callee function name
    ///
    /// # Returns
    ///
    /// Ok(()) on success, silently ignores if either function doesn't exist
    ///
    /// # Errors
    ///
    /// Returns error if CSR graph operation fails
    pub fn add_edge(&mut self, from: &str, to: &str) -> Result<()> {
        debug_assert!(!from.is_empty(), "from must not be empty");
        debug_assert!(!to.is_empty(), "to must not be empty");
        if let (Some(&from_id), Some(&to_id)) = (self.node_map.get(from), self.node_map.get(to)) {
            self.graph
                .add_edge(from_id, to_id, 1.0)
                .context("Failed to add edge to CSR graph")?;
        }
        Ok(())
    }

    /// Check if function exists in graph (O(1))
    ///
    /// # Arguments
    ///
    /// * `name` - Function name to check
    ///
    /// # Returns
    ///
    /// true if function exists, false otherwise
    #[must_use]
    pub fn has_function(&self, name: &str) -> bool {
        debug_assert!(!name.is_empty(), "name must not be empty");
        self.node_map.contains_key(name)
    }

    /// Update PageRank criticality scores
    ///
    /// Runs PageRank algorithm on the CSR graph to identify critical functions
    /// (functions with many incoming edges = called by many others).
    ///
    /// # Errors
    ///
    /// Returns error if PageRank computation fails
    pub fn update_criticality(&mut self) -> Result<()> {
        if self.graph.num_nodes() == 0 {
            return Ok(());
        }

        // Run PageRank (20 iterations, tolerance 1e-6)
        let scores = pagerank(&self.graph, 20, 1e-6).context("PageRank computation failed")?;

        // Aggregate scores by function name
        self.criticality_scores.clear();
        for (node_id, score) in scores.iter().enumerate() {
            let node_id = NodeId(node_id as u32);
            if let Some(name) = self.reverse_node_map.get(&node_id) {
                self.criticality_scores.insert(name.clone(), *score);
            }
        }

        Ok(())
    }

    /// Get critical functions (sorted by PageRank score)
    ///
    /// Returns functions ranked by criticality (PageRank score).
    ///
    /// # Returns
    ///
    /// Vec<(function_name, criticality_score)> sorted by score (highest first)
    #[must_use]
    pub fn critical_functions(&self) -> Vec<(String, f32)> {
        let mut functions: Vec<_> = self
            .criticality_scores
            .iter()
            .map(|(name, score)| (name.clone(), *score))
            .collect();
        functions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        functions
    }

    /// Get number of nodes in graph
    ///
    /// Returns the count of functions we've added to the graph.
    #[must_use]
    pub fn num_nodes(&self) -> usize {
        self.node_map.len()
    }

    /// Get number of edges in graph
    #[must_use]
    pub fn num_edges(&self) -> usize {
        self.graph.num_edges()
    }
}
