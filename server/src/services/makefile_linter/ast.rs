use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MakefileAst {
    pub nodes: Vec<MakefileNode>,
    pub source_map: HashMap<usize, SourceSpan>,
    pub metadata: MakefileMetadata,
}

#[derive(Debug, Clone)]
pub struct MakefileNode {
    pub kind: MakefileNodeKind,
    pub span: SourceSpan,
    pub children: Vec<usize>, // Indices into nodes vec
    pub data: NodeData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MakefileNodeKind {
    Rule,
    Variable,
    Recipe,
    Include,
    Conditional,
    Expansion,
    Comment,
    Directive,
    Target,
    Prerequisite,
}

#[derive(Debug, Clone)]
pub enum NodeData {
    Rule {
        targets: Vec<String>,
        prerequisites: Vec<String>,
        is_pattern: bool,
        is_phony: bool,
        is_double_colon: bool,
    },
    Variable {
        name: String,
        assignment_op: AssignmentOp,
        value: String,
    },
    Recipe {
        lines: Vec<RecipeLine>,
    },
    Target {
        name: String,
    },
    Text(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignmentOp {
    Deferred,    // =
    Immediate,   // :=
    Conditional, // ?=
    Append,      // +=
    Shell,       // !=
}

#[derive(Debug, Clone)]
pub struct RecipeLine {
    pub text: String,
    pub prefixes: RecipePrefixes,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RecipePrefixes {
    pub silent: bool,       // @
    pub ignore_error: bool, // -
    pub always_exec: bool,  // +
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct SourceSpan {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl SourceSpan {
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self {
            start,
            end,
            line,
            column,
        }
    }

    pub fn file_level() -> Self {
        Self {
            start: 0,
            end: 0,
            line: 0,
            column: 0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MakefileMetadata {
    pub has_phony_rules: bool,
    pub has_pattern_rules: bool,
    pub uses_automatic_variables: bool,
    pub target_count: usize,
    pub variable_count: usize,
    pub recipe_count: usize,
}

impl Default for MakefileAst {
    fn default() -> Self {
        Self::new()
    }
}

impl MakefileAst {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            source_map: HashMap::new(),
            metadata: MakefileMetadata::default(),
        }
    }

    pub fn add_node(&mut self, node: MakefileNode) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(node);
        idx
    }

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

    pub fn count_targets(&self) -> usize {
        self.nodes
            .iter()
            .filter(|n| n.kind == MakefileNodeKind::Target)
            .count()
    }

    pub fn count_phony_targets(&self) -> usize {
        self.get_phony_targets().len()
    }

    pub fn has_pattern_rules(&self) -> bool {
        self.nodes.iter().any(|n| {
            if let NodeData::Rule { is_pattern, .. } = &n.data {
                *is_pattern
            } else {
                false
            }
        })
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_makefile_ast_creation() {
        let ast = MakefileAst::new();
        assert_eq!(ast.nodes.len(), 0);
        assert_eq!(ast.source_map.len(), 0);
        assert!(!ast.metadata.has_phony_rules);
    }

    #[test]
    fn test_add_node() {
        let mut ast = MakefileAst::new();
        let node = MakefileNode {
            kind: MakefileNodeKind::Rule,
            span: SourceSpan::file_level(),
            children: vec![],
            data: NodeData::Rule {
                targets: vec!["test".to_string()],
                prerequisites: vec![],
                is_pattern: false,
                is_phony: false,
                is_double_colon: false,
            },
        };

        let idx = ast.add_node(node);
        assert_eq!(idx, 0);
        assert_eq!(ast.nodes.len(), 1);
    }

    #[test]
    fn test_find_rules_by_target() {
        let mut ast = MakefileAst::new();

        // Add a rule for "test" target
        let node = MakefileNode {
            kind: MakefileNodeKind::Rule,
            span: SourceSpan::file_level(),
            children: vec![],
            data: NodeData::Rule {
                targets: vec!["test".to_string(), "check".to_string()],
                prerequisites: vec![],
                is_pattern: false,
                is_phony: false,
                is_double_colon: false,
            },
        };
        ast.add_node(node);

        let test_rules = ast.find_rules_by_target("test");
        assert_eq!(test_rules.len(), 1);
        assert_eq!(test_rules[0], 0);

        let check_rules = ast.find_rules_by_target("check");
        assert_eq!(check_rules.len(), 1);

        let missing_rules = ast.find_rules_by_target("missing");
        assert_eq!(missing_rules.len(), 0);
    }

    #[test]
    fn test_get_phony_targets() {
        let mut ast = MakefileAst::new();

        // Add .PHONY rule
        let phony_rule = MakefileNode {
            kind: MakefileNodeKind::Rule,
            span: SourceSpan::file_level(),
            children: vec![],
            data: NodeData::Rule {
                targets: vec![".PHONY".to_string()],
                prerequisites: vec!["test".to_string(), "clean".to_string()],
                is_pattern: false,
                is_phony: true,
                is_double_colon: false,
            },
        };
        ast.add_node(phony_rule);

        let phony_targets = ast.get_phony_targets();
        assert_eq!(phony_targets.len(), 2);
        assert!(phony_targets.contains(&"test".to_string()));
        assert!(phony_targets.contains(&"clean".to_string()));
    }

    #[test]
    fn test_source_span() {
        let span = SourceSpan::new(10, 20, 5, 3);
        assert_eq!(span.start, 10);
        assert_eq!(span.end, 20);
        assert_eq!(span.line, 5);
        assert_eq!(span.column, 3);

        let file_span = SourceSpan::file_level();
        assert_eq!(file_span.start, 0);
        assert_eq!(file_span.end, 0);
        assert_eq!(file_span.line, 0);
        assert_eq!(file_span.column, 0);
    }

    #[test]
    fn test_count_targets() {
        let mut ast = MakefileAst::new();

        // Add some target nodes
        ast.add_node(MakefileNode {
            kind: MakefileNodeKind::Target,
            span: SourceSpan::file_level(),
            children: vec![],
            data: NodeData::Target {
                name: "all".to_string(),
            },
        });

        ast.add_node(MakefileNode {
            kind: MakefileNodeKind::Target,
            span: SourceSpan::file_level(),
            children: vec![],
            data: NodeData::Target {
                name: "clean".to_string(),
            },
        });

        assert_eq!(ast.count_targets(), 2);
    }

    #[test]
    fn test_count_phony_targets() {
        let mut ast = MakefileAst::new();

        // Add .PHONY rule with multiple targets
        ast.add_node(MakefileNode {
            kind: MakefileNodeKind::Rule,
            span: SourceSpan::file_level(),
            children: vec![],
            data: NodeData::Rule {
                targets: vec![".PHONY".to_string()],
                prerequisites: vec!["test".to_string(), "clean".to_string(), "all".to_string()],
                is_pattern: false,
                is_phony: true,
                is_double_colon: false,
            },
        });

        assert_eq!(ast.count_phony_targets(), 3);
    }

    #[test]
    fn test_has_pattern_rules() {
        let mut ast = MakefileAst::new();

        // No pattern rules initially
        assert!(!ast.has_pattern_rules());

        // Add regular rule
        ast.add_node(MakefileNode {
            kind: MakefileNodeKind::Rule,
            span: SourceSpan::file_level(),
            children: vec![],
            data: NodeData::Rule {
                targets: vec!["test".to_string()],
                prerequisites: vec![],
                is_pattern: false,
                is_phony: false,
                is_double_colon: false,
            },
        });

        assert!(!ast.has_pattern_rules());

        // Add pattern rule
        ast.add_node(MakefileNode {
            kind: MakefileNodeKind::Rule,
            span: SourceSpan::file_level(),
            children: vec![],
            data: NodeData::Rule {
                targets: vec!["%.o".to_string()],
                prerequisites: vec!["%.c".to_string()],
                is_pattern: true,
                is_phony: false,
                is_double_colon: false,
            },
        });

        assert!(ast.has_pattern_rules());
    }

    #[test]
    fn test_uses_automatic_variables() {
        let mut ast = MakefileAst::new();

        // No automatic variables initially
        assert!(!ast.uses_automatic_variables());

        // Add recipe with automatic variable
        ast.add_node(MakefileNode {
            kind: MakefileNodeKind::Recipe,
            span: SourceSpan::file_level(),
            children: vec![],
            data: NodeData::Recipe {
                lines: vec![RecipeLine {
                    text: "gcc -o $@ $<".to_string(),
                    prefixes: RecipePrefixes::default(),
                }],
            },
        });

        assert!(ast.uses_automatic_variables());

        // Test variable with automatic variable
        ast.add_node(MakefileNode {
            kind: MakefileNodeKind::Variable,
            span: SourceSpan::file_level(),
            children: vec![],
            data: NodeData::Variable {
                name: "OBJS".to_string(),
                assignment_op: AssignmentOp::Deferred,
                value: "$(patsubst %.c,%.o,$^)".to_string(),
            },
        });

        assert!(ast.uses_automatic_variables());
    }

    #[test]
    fn test_get_variables() {
        let mut ast = MakefileAst::new();

        // Add some variables
        ast.add_node(MakefileNode {
            kind: MakefileNodeKind::Variable,
            span: SourceSpan::file_level(),
            children: vec![],
            data: NodeData::Variable {
                name: "CC".to_string(),
                assignment_op: AssignmentOp::Deferred,
                value: "gcc".to_string(),
            },
        });

        ast.add_node(MakefileNode {
            kind: MakefileNodeKind::Variable,
            span: SourceSpan::file_level(),
            children: vec![],
            data: NodeData::Variable {
                name: "CFLAGS".to_string(),
                assignment_op: AssignmentOp::Immediate,
                value: "-Wall -O2".to_string(),
            },
        });

        let vars = ast.get_variables();
        assert_eq!(vars.len(), 2);
        assert_eq!(vars[0].0, "CC");
        assert_eq!(vars[0].2, "gcc");
        assert_eq!(vars[1].0, "CFLAGS");
        assert_eq!(vars[1].2, "-Wall -O2");
    }

    #[test]
    fn test_metadata_default() {
        let metadata = MakefileMetadata::default();
        assert!(!metadata.has_phony_rules);
        assert!(!metadata.has_pattern_rules);
        assert!(!metadata.uses_automatic_variables);
        assert_eq!(metadata.target_count, 0);
        assert_eq!(metadata.variable_count, 0);
        assert_eq!(metadata.recipe_count, 0);
    }
}
