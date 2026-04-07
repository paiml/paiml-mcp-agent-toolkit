// SimpleDiGraph implementation: node/edge management and Kosaraju's SCC algorithm

impl<N, E> SimpleDiGraph<N, E> {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    fn add_node(&mut self, node: N) -> CfgNodeIndex {
        let idx = CfgNodeIndex(self.nodes.len());
        self.nodes.push(node);
        idx
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    fn add_edge(&mut self, from: CfgNodeIndex, to: CfgNodeIndex, edge: E) {
        self.edges.push((from, to, edge));
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    fn node_count(&self) -> usize {
        self.nodes.len()
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    fn edge_count(&self) -> usize {
        self.edges.len()
    }

    // Simple SCC using Kosaraju's algorithm
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    fn kosaraju_scc(&self) -> Vec<Vec<CfgNodeIndex>> {
        let n = self.nodes.len();
        if n == 0 {
            return Vec::new();
        }

        // Build adjacency list
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
        let mut adj_rev: Vec<Vec<usize>> = vec![Vec::new(); n];

        for (from, to, _) in &self.edges {
            adj[from.0].push(to.0);
            adj_rev[to.0].push(from.0);
        }

        // First DFS to get finish order
        let mut visited = vec![false; n];
        let mut finish_order = Vec::with_capacity(n);

        for i in 0..n {
            if !visited[i] {
                self.dfs_finish(&adj, i, &mut visited, &mut finish_order);
            }
        }

        // Second DFS on transpose in reverse finish order
        let mut visited = vec![false; n];
        let mut sccs = Vec::new();

        for &node in finish_order.iter().rev() {
            if !visited[node] {
                let mut scc = Vec::new();
                self.dfs_collect(&adj_rev, node, &mut visited, &mut scc);
                sccs.push(scc.into_iter().map(CfgNodeIndex).collect());
            }
        }

        sccs
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    fn dfs_finish(
        &self,
        adj: &[Vec<usize>],
        node: usize,
        visited: &mut [bool],
        finish: &mut Vec<usize>,
    ) {
        visited[node] = true;
        for &next in &adj[node] {
            if !visited[next] {
                self.dfs_finish(adj, next, visited, finish);
            }
        }
        finish.push(node);
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    fn dfs_collect(
        &self,
        adj: &[Vec<usize>],
        node: usize,
        visited: &mut [bool],
        scc: &mut Vec<usize>,
    ) {
        visited[node] = true;
        scc.push(node);
        for &next in &adj[node] {
            if !visited[next] {
                self.dfs_collect(adj, next, visited, scc);
            }
        }
    }
}

// ControlFlowGraph: construction from AST and complexity metrics

impl ControlFlowGraph {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn from_ast(ast: &File) -> Self {
        let mut graph = SimpleDiGraph::new();
        let entry = graph.add_node(CfgNode::Entry);
        let exit = graph.add_node(CfgNode::Exit);

        let mut builder = CfgBuilder {
            graph,
            current: entry,
            _exit: exit,
            break_targets: Vec::new(),
            continue_targets: Vec::new(),
        };

        builder.visit_file(ast);
        builder
            .graph
            .add_edge(builder.current, exit, CfgEdge::Sequential);

        ControlFlowGraph {
            graph: builder.graph,
            _entry: entry,
            _exit: exit,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn cyclomatic_complexity(&self) -> u32 {
        let edges = self.graph.edge_count() as u32;
        let nodes = self.graph.node_count() as u32;
        let components = self.graph.kosaraju_scc().len() as u32;

        edges - nodes + 2 * components
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn essential_complexity(&self) -> u32 {
        // Count strongly connected components (loops)
        let sccs = self.graph.kosaraju_scc();
        sccs.iter().filter(|scc| scc.len() > 1).count() as u32
    }
}

// CfgBuilder: Visit trait implementation for building CFG from syn AST

impl<'ast> Visit<'ast> for CfgBuilder {
    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
        let condition = self.graph.add_node(CfgNode::Condition("if".to_string()));
        self.graph
            .add_edge(self.current, condition, CfgEdge::Sequential);

        let then_branch = self.graph.add_node(CfgNode::Branch("then".to_string()));
        self.graph.add_edge(condition, then_branch, CfgEdge::True);

        let _old_current = self.current;
        self.current = then_branch;
        syn::visit::visit_block(self, &node.then_branch);
        let then_exit = self.current;

        let merge = self.graph.add_node(CfgNode::Statement("merge".to_string()));

        if let Some((_, else_branch)) = &node.else_branch {
            let else_node = self.graph.add_node(CfgNode::Branch("else".to_string()));
            self.graph.add_edge(condition, else_node, CfgEdge::False);

            self.current = else_node;
            syn::visit::visit_expr(self, else_branch);
            self.graph
                .add_edge(self.current, merge, CfgEdge::Sequential);
        } else {
            self.graph.add_edge(condition, merge, CfgEdge::False);
        }

        self.graph.add_edge(then_exit, merge, CfgEdge::Sequential);
        self.current = merge;
    }

    fn visit_expr_loop(&mut self, node: &'ast syn::ExprLoop) {
        let loop_entry = self.graph.add_node(CfgNode::Statement("loop".to_string()));
        self.graph
            .add_edge(self.current, loop_entry, CfgEdge::Sequential);

        let loop_exit = self
            .graph
            .add_node(CfgNode::Statement("loop_exit".to_string()));
        self.break_targets.push(loop_exit);
        self.continue_targets.push(loop_entry);

        self.current = loop_entry;
        syn::visit::visit_block(self, &node.body);
        self.graph.add_edge(self.current, loop_entry, CfgEdge::Jump);

        self.break_targets.pop();
        self.continue_targets.pop();
        self.current = loop_exit;
    }
}
