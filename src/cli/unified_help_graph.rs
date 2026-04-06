// CommandGraph implementation - graph-based importance ranking via trueno-graph PageRank

impl CommandGraph {
    /// Create a new command graph
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            graph: CsrGraph::new(),
            command_to_node: HashMap::new(),
            node_to_command: HashMap::new(),
            importance_scores: HashMap::new(),
            next_node_id: 0,
        }
    }

    /// Build graph from command registry
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn build_from_registry(&mut self, registry: &CommandRegistry) {
        // Add nodes for all commands
        for (name, cmd) in &registry.commands {
            let node_id = NodeId(self.next_node_id);
            self.next_node_id += 1;
            self.command_to_node.insert(name.clone(), node_id);
            self.node_to_command.insert(node_id, name.clone());

            // Also add subcommands
            for sub in &cmd.subcommands {
                let full_name = format!("{} {}", name, sub.name);
                let sub_node_id = NodeId(self.next_node_id);
                self.next_node_id += 1;
                self.command_to_node.insert(full_name.clone(), sub_node_id);
                self.node_to_command.insert(sub_node_id, full_name);
            }
        }

        // Add edges for relationships
        for (name, cmd) in &registry.commands {
            if let Some(&from_id) = self.command_to_node.get(name) {
                // Parent -> Subcommand edges
                for sub in &cmd.subcommands {
                    let full_name = format!("{} {}", name, sub.name);
                    if let Some(&to_id) = self.command_to_node.get(&full_name) {
                        let _ = self.graph.add_edge(from_id, to_id, 1.0);
                    }
                }

                // Related command edges (bidirectional, lower weight)
                for related in &cmd.related {
                    if let Some(&to_id) = self.command_to_node.get(related) {
                        let _ = self.graph.add_edge(from_id, to_id, 0.5);
                        let _ = self.graph.add_edge(to_id, from_id, 0.5);
                    }
                }
            }
        }

        // Compute PageRank
        self.update_importance();
    }

    /// Update importance scores using PageRank
    fn update_importance(&mut self) {
        debug_assert!(true, "contract: update_importance");
        use trueno_graph::algorithms::pagerank::pagerank;

        match pagerank(&self.graph, 20, 1e-6_f32) {
            Ok(scores) => {
                self.importance_scores.clear();
                for (node_idx, score) in scores.iter().enumerate() {
                    let node_id = NodeId(node_idx as u32);
                    if let Some(name) = self.node_to_command.get(&node_id) {
                        let key: String = name.clone();
                        self.importance_scores.insert(key, *score);
                    }
                }
            }
            Err(_) => {
                // On error, use uniform importance
                let uniform = 1.0 / self.command_to_node.len().max(1) as f32;
                for name in self.command_to_node.keys() {
                    let key: String = name.clone();
                    self.importance_scores.insert(key, uniform);
                }
            }
        }
    }

    /// Get importance score for a command
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn importance(&self, command: &str) -> f32 {
        debug_assert!(!command.is_empty(), "command must not be empty");
        self.importance_scores.get(command).copied().unwrap_or(0.0)
    }

    /// Rank commands by importance
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn rank_by_importance(&self, commands: &[String]) -> Vec<(String, f32)> {
        let mut ranked: Vec<_> = commands
            .iter()
            .map(|c| (c.clone(), self.importance(c)))
            .collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        ranked
    }

    /// Get top-k most important commands
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn top_k_important(&self, k: usize) -> Vec<(String, f32)> {
        debug_assert!(k > 0, "k must be positive");
        let mut all: Vec<_> = self
            .importance_scores
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        all.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        all.truncate(k);
        all
    }
}

impl Default for CommandGraph {
    fn default() -> Self {
        Self::new()
    }
}
