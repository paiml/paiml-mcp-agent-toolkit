// terminal_rendering.rs - Visualizable impl for VisGraph (force-directed rendering)
// Included by terminal.rs - NO use imports, NO #! inner attributes

impl Visualizable for VisGraph {
    fn render_terminal(&self, config: &RenderConfig) -> Result<String> {
        if self.nodes.is_empty() {
            return Ok("(empty graph)\n".to_string());
        }

        // Semantic zooming: filter to top N nodes by criticality
        let mut indexed_criticality: Vec<(usize, f32)> = self
            .criticality
            .iter()
            .enumerate()
            .map(|(i, &c)| (i, c))
            .collect();
        indexed_criticality
            .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let visible_indices: std::collections::HashSet<usize> = indexed_criticality
            .iter()
            .take(config.max_nodes)
            .map(|(i, _)| *i)
            .collect();

        // Build force graph.
        // aprender-viz 0.50 removed ForceGraph's `dimensions()` setter; the
        // layout now uses the default 600x400 framebuffer, which the
        // TerminalEncoder below downsamples to the terminal grid
        // (config.width x config.height), so the rendered output is unchanged.
        let mut fg = ForceGraph::new()
            .iterations(config.iterations)
            .background(config.theme.background_color());

        // Add nodes
        for (idx, name) in self.nodes.iter().enumerate() {
            if !visible_indices.contains(&idx) {
                continue;
            }

            let criticality = self.criticality[idx];
            let is_critical = criticality >= config.critical_threshold;

            let color = if is_critical {
                config.theme.critical_color()
            } else {
                config.theme.normal_color()
            };

            // Dual encoding: critical nodes are larger
            let radius = if is_critical { 12.0 } else { 8.0 };

            let node = GraphNode::new(idx).label(name).color(color).radius(radius);

            fg = fg.add_node(node);
        }

        // Add edges (only between visible nodes)
        for &(from, to) in &self.edges {
            if visible_indices.contains(&from) && visible_indices.contains(&to) {
                let edge = GraphEdge::new(from, to).color(config.theme.edge_color());
                fg = fg.add_edge(edge);
            }
        }

        // Build and render
        let built = fg.build().context("Failed to build force graph")?;
        let fb = built
            .to_framebuffer()
            .context("Failed to create framebuffer")?;

        let encoder = TerminalEncoder::new()
            .mode(config.mode)
            .width(config.width)
            .height(config.height);

        Ok(encoder.render(&fb))
    }

    fn node_count(&self) -> usize {
        self.nodes.len()
    }
}
