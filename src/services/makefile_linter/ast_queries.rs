impl MakefileAst {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            source_map: HashMap::new(),
            metadata: MakefileMetadata::default(),
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_node(&mut self, node: MakefileNode) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(node);
        idx
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn find_rules_by_target(&self, target: &str) -> Vec<usize> {
        self.nodes
            .iter()
            .enumerate()
            .filter_map(|(idx, node)| {
                if node.kind == MakefileNodeKind::Rule {
                    if let NodeData::Rule { targets, .. } = &node.data {
                        if targets.contains(&target.to_string()) {
                            return Some(idx);
                        }
                    }
                }
                None
            })
            .collect()
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_phony_targets(&self) -> Vec<String> {
        let phony_rules = self.find_rules_by_target(".PHONY");
        let mut targets = Vec::new();

        for rule_idx in phony_rules {
            if let Some(rule) = self.nodes.get(rule_idx) {
                if let NodeData::Rule { prerequisites, .. } = &rule.data {
                    targets.extend(prerequisites.clone());
                }
            }
        }

        targets
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn count_targets(&self) -> usize {
        self.nodes
            .iter()
            .filter(|n| n.kind == MakefileNodeKind::Target)
            .count()
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn count_phony_targets(&self) -> usize {
        self.get_phony_targets().len()
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn has_pattern_rules(&self) -> bool {
        self.nodes.iter().any(|n| {
            if let NodeData::Rule { is_pattern, .. } = &n.data {
                *is_pattern
            } else {
                false
            }
        })
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn uses_automatic_variables(&self) -> bool {
        self.nodes.iter().any(|n| match &n.data {
            NodeData::Recipe { lines } => lines.iter().any(|line| {
                line.text.contains("$@")
                    || line.text.contains("$<")
                    || line.text.contains("$^")
                    || line.text.contains("$?")
                    || line.text.contains("$*")
            }),
            NodeData::Variable { value, .. } => {
                value.contains("$@")
                    || value.contains("$<")
                    || value.contains("$^")
                    || value.contains("$?")
                    || value.contains("$*")
            }
            _ => false,
        })
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_variables(&self) -> Vec<(&String, &AssignmentOp, &String)> {
        self.nodes
            .iter()
            .filter_map(|n| {
                if n.kind == MakefileNodeKind::Variable {
                    if let NodeData::Variable {
                        name,
                        assignment_op,
                        value,
                    } = &n.data
                    {
                        Some((name, assignment_op, value))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }
}
