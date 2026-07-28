#[cfg_attr(coverage_nightly, coverage(off))]
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

