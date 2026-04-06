impl DeadCodeAnalyzer {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn classify_dead_code(&self, dag: &AstDag) -> DeadCodeReport {
        let reachable = self.reachability.read();
        let mut dead_functions = Vec::new();
        let mut dead_classes = Vec::new();
        let mut dead_variables = Vec::new();
        let unreachable_code = Vec::new();

        let total_nodes = dag.nodes.len();
        let reachable_count = reachable.count_set();
        let dead_count = total_nodes.saturating_sub(reachable_count);

        // TRACKED: Iterate through DAG nodes and classify dead code
        for (idx, node) in dag.nodes.iter().enumerate() {
            if !reachable.is_set(idx as u32) {
                // Classify based on node type
                match &node.kind {
                    crate::models::unified_ast::AstKind::Function(_) => {
                        dead_functions.push(DeadCodeItem {
                            node_key: idx as NodeKey,
                            name: String::new(),      // TRACKED: Extract name
                            file_path: String::new(), // TRACKED: Extract path
                            line_number: node.source_range.start,
                            dead_type: DeadCodeType::UnusedFunction,
                            confidence: 0.95,
                            reason: "Not reachable from any entry point".to_string(),
                        });
                    }
                    crate::models::unified_ast::AstKind::Class(_) => {
                        dead_classes.push(DeadCodeItem {
                            node_key: idx as NodeKey,
                            name: String::new(),
                            file_path: String::new(),
                            line_number: node.source_range.start,
                            dead_type: DeadCodeType::UnusedClass,
                            confidence: 0.95,
                            reason: "Class never instantiated or referenced".to_string(),
                        });
                    }
                    crate::models::unified_ast::AstKind::Variable(_) => {
                        dead_variables.push(DeadCodeItem {
                            node_key: idx as NodeKey,
                            name: String::new(),
                            file_path: String::new(),
                            line_number: node.source_range.start,
                            dead_type: DeadCodeType::UnusedVariable,
                            confidence: 0.90,
                            reason: "Variable never accessed".to_string(),
                        });
                    }
                    _ => {}
                }
            }
        }

        let percentage_dead = if total_nodes > 0 {
            (dead_count as f32 / total_nodes as f32) * 100.0
        } else {
            0.0
        };

        DeadCodeReport {
            dead_functions,
            dead_classes,
            dead_variables,
            unreachable_code,
            summary: DeadCodeSummary {
                total_dead_code_lines: dead_count * 10, // Rough estimate
                percentage_dead,
                dead_by_type: HashMap::new(), // TRACKED: Populate
                confidence_level: 0.85,
            },
        }
    }

    /// Classify dead code from dependency graph
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn classify_dead_code_from_dep_graph(&self, dag: &DependencyGraph) -> DeadCodeReport {
        let reachable = self.reachability.read();
        let mut dead_functions = Vec::new();
        let mut dead_classes = Vec::new();
        let dead_variables = Vec::new();
        let unreachable_code = Vec::new();

        let total_nodes = dag.nodes.len();
        let reachable_count = reachable.count_set();
        let dead_count = total_nodes.saturating_sub(reachable_count);

        // Process nodes from dependency graph
        for (node_id, node_info) in &dag.nodes {
            let key = node_id.parse::<u32>().unwrap_or(0);
            if !reachable.is_set(key) {
                // Classify based on node type
                match node_info.node_type {
                    crate::models::dag::NodeType::Function => {
                        dead_functions.push(DeadCodeItem {
                            node_key: key,
                            name: node_info.label.clone(),
                            file_path: node_info.file_path.clone(),
                            line_number: node_info.line_number as u32,
                            dead_type: DeadCodeType::UnusedFunction,
                            confidence: 0.95,
                            reason: "Not reachable from any entry point".to_string(),
                        });
                    }
                    crate::models::dag::NodeType::Class => {
                        dead_classes.push(DeadCodeItem {
                            node_key: key,
                            name: node_info.label.clone(),
                            file_path: node_info.file_path.clone(),
                            line_number: node_info.line_number as u32,
                            dead_type: DeadCodeType::UnusedClass,
                            confidence: 0.95,
                            reason: "Class never instantiated or referenced".to_string(),
                        });
                    }
                    _ => {}
                }
            }
        }

        let percentage_dead = if total_nodes > 0 {
            (dead_count as f32 / total_nodes as f32) * 100.0
        } else {
            0.0
        };

        DeadCodeReport {
            dead_functions,
            dead_classes,
            dead_variables,
            unreachable_code,
            summary: DeadCodeSummary {
                total_dead_code_lines: dead_count * 10, // Rough estimate
                percentage_dead,
                dead_by_type: HashMap::new(), // TRACKED: Populate
                confidence_level: 0.85,
            },
        }
    }
}
