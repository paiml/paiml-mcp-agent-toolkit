// DependencyGraph: directed graph for module dependency analysis with
// topological sort (DFS-based) and dependency depth calculation.

impl DependencyGraph {
    fn new() -> Self {
        Self {
            edges: HashMap::new(),
        }
    }

    fn add_edge(&mut self, from: String, to: String) {
        self.edges.entry(from).or_insert_with(HashSet::new).insert(to);
    }

    fn topological_sort(&self) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut stack = Vec::new();

        for node in self.edges.keys() {
            if !visited.contains(node) {
                self.dfs(node, &mut visited, &mut stack);
            }
        }

        stack.reverse();
        stack
    }

    fn dfs(&self, node: &str, visited: &mut HashSet<String>, stack: &mut Vec<String>) {
        visited.insert(node.to_string());

        if let Some(neighbors) = self.edges.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    self.dfs(neighbor, visited, stack);
                }
            }
        }

        stack.push(node.to_string());
    }

    fn calculate_depth(&self) -> usize {
        let topo_order = self.topological_sort();
        let mut depths = HashMap::new();
        let mut max_depth = 0;

        for node in topo_order {
            let incoming_depth = self.edges.values()
                .filter_map(|deps| {
                    if deps.contains(&node) {
                        deps.iter()
                            .filter_map(|dep| depths.get(dep))
                            .max()
                            .copied()
                    } else {
                        None
                    }
                })
                .max()
                .unwrap_or(0);

            let node_depth = incoming_depth + 1;
            depths.insert(node.clone(), node_depth);
            max_depth = max_depth.max(node_depth);
        }

        max_depth
    }
}
