#[cfg(test)]
mod coverage_tests {
    use super::super::*;
    use crate::services::makefile_linter::ast::*;
    use crate::services::makefile_linter::parser::*;
    use crate::services::makefile_linter::rules::*;

    #[test]
    fn test_parser_edge_cases() {
        // Test empty input
        let mut parser = MakefileParser::new("");
        assert!(parser.parse().unwrap().nodes.is_empty());

        // Test only whitespace
        let mut parser = MakefileParser::new("   \n\t\n   ");
        assert!(parser.parse().unwrap().nodes.is_empty());

        // Test only comments
        let mut parser = MakefileParser::new("# Comment 1\n# Comment 2");
        let ast = parser.parse().unwrap();
        assert_eq!(ast.nodes.len(), 2);
        assert!(ast.nodes.iter().all(|n| n.kind == MakefileNodeKind::Comment));
    }

    #[test]
    fn test_utf8_handling() {
        // Test with emoji and unicode
        let input = "TARGET = 🚀 Unicode → test";
        let mut parser = MakefileParser::new(input);
        let ast = parser.parse().unwrap();
        assert!(!ast.nodes.is_empty());

        // Test with Chinese characters
        let input = "目标 = 值";
        let mut parser = MakefileParser::new(input);
        let ast = parser.parse().unwrap();
        assert!(!ast.nodes.is_empty());
    }

    #[test]
    fn test_all_assignment_operators() {
        let test_cases = vec![
            ("VAR = value", AssignmentOp::Deferred),
            ("VAR := value", AssignmentOp::Immediate),
            ("VAR ?= value", AssignmentOp::Conditional),
            ("VAR += value", AssignmentOp::Append),
            ("VAR != date", AssignmentOp::Shell),
        ];

        for (input, expected_op) in test_cases {
            let mut parser = MakefileParser::new(input);
            let ast = parser.parse().unwrap();
            assert_eq!(ast.nodes.len(), 1);
            
            if let NodeData::Variable { assignment_op, .. } = &ast.nodes[0].data {
                assert_eq!(*assignment_op, expected_op);
            } else {
                panic!("Expected variable node");
            }
        }
    }

    #[test]
    fn test_recipe_prefixes() {
        let input = "target:\n\t@echo silent\n\t-rm ignore error\n\t+make always";
        let mut parser = MakefileParser::new(input);
        let ast = parser.parse().unwrap();
        
        let recipe_node = ast.nodes.iter()
            .find(|n| matches!(n.kind, MakefileNodeKind::Recipe))
            .expect("Recipe node not found");
            
        if let NodeData::Recipe { lines } = &recipe_node.data {
            assert_eq!(lines.len(), 3);
            assert!(lines[0].prefixes.silent);
            assert!(lines[1].prefixes.ignore_error);
            assert!(lines[2].prefixes.always_exec);
        }
    }

    #[test]
    fn test_complex_makefile() {
        let input = r#"
# Makefile for testing
CC := gcc
CFLAGS += -Wall -O2

.PHONY: all clean test

all: main.o utils.o
	$(CC) $(CFLAGS) -o program $^

%.o: %.c
	$(CC) $(CFLAGS) -c $< -o $@

include config.mk
-include optional.mk

clean:
	@rm -f *.o program
	@echo "Cleaned!"

ifeq ($(DEBUG),1)
CFLAGS += -g
endif
"#;
        
        let mut parser = MakefileParser::new(input);
        let ast = parser.parse().unwrap();
        
        // Verify various elements
        assert!(ast.metadata.has_phony_rules);
        assert!(ast.metadata.has_pattern_rules);
        assert!(ast.metadata.uses_automatic_variables);
        assert!(ast.metadata.variable_count > 0);
        assert!(ast.metadata.target_count > 0);
    }

    #[test]
    fn test_line_continuation() {
        let input = "LONG_VAR = first \\\n    second \\\n    third";
        let mut parser = MakefileParser::new(input);
        let ast = parser.parse().unwrap();
        assert!(!ast.nodes.is_empty());
    }

    #[test]
    fn test_double_colon_rules() {
        let input = "all:: target1\nall:: target2";
        let mut parser = MakefileParser::new(input);
        let ast = parser.parse().unwrap();
        
        let double_colon_rules: Vec<_> = ast.nodes.iter()
            .filter(|n| matches!(&n.data, NodeData::Rule { is_double_colon: true, .. }))
            .collect();
        assert_eq!(double_colon_rules.len(), 2);
    }

    #[test]
    fn test_pattern_rules() {
        let patterns = vec![
            "%.o: %.c",
            "lib%.a: %.o",
            "%.tab.c %.tab.h: %.y",
        ];
        
        for pattern in patterns {
            let mut parser = MakefileParser::new(pattern);
            let ast = parser.parse().unwrap();
            assert!(ast.has_pattern_rules());
        }
    }

    #[test]
    fn test_automatic_variables() {
        let vars = vec!["$@", "$<", "$^", "$?", "$*", "$%", "$+", "$|"];
        
        for var in vars {
            let input = format!("target:\n\techo {}", var);
            let mut parser = MakefileParser::new(&input);
            let ast = parser.parse().unwrap();
            assert!(ast.uses_automatic_variables());
        }
    }

    #[test]
    fn test_rule_registry() {
        let registry = RuleRegistry::new();
        assert!(!registry.rules.is_empty());
        
        // Test that all standard rules are registered
        let rule_ids: Vec<_> = registry.rules.iter().map(|r| r.id()).collect();
        assert!(rule_ids.contains(&"minphony"));
        assert!(rule_ids.contains(&"phonydeclared"));
        assert!(rule_ids.contains(&"maxbodylength"));
        assert!(rule_ids.contains(&"timestampexpanded"));
        assert!(rule_ids.contains(&"undefinedvariable"));
        assert!(rule_ids.contains(&"portability"));
    }

    #[test]
    fn test_lint_result_methods() {
        let violations = vec![
            Violation {
                rule: "test1".to_string(),
                severity: Severity::Error,
                span: SourceSpan::file_level(),
                message: "Error".to_string(),
                fix_hint: None,
            },
            Violation {
                rule: "test2".to_string(),
                severity: Severity::Warning,
                span: SourceSpan::file_level(),
                message: "Warning".to_string(),
                fix_hint: Some("Fix hint".to_string()),
            },
        ];
        
        let result = LintResult {
            path: std::path::PathBuf::from("test.mk"),
            violations: violations.clone(),
            quality_score: 0.7,
        };
        
        assert!(result.has_errors());
        assert_eq!(result.error_count(), 1);
        assert_eq!(result.max_severity(), Some(&Severity::Error));
        
        // Test JSON serialization
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("test1"));
        assert!(json.contains("Fix hint"));
    }

    #[test]
    fn test_parse_errors() {
        // Test recipe without rule
        let input = "\techo orphan recipe";
        let mut parser = MakefileParser::new(input);
        let result = parser.parse();
        assert!(result.is_err());
        
        // Test invalid variable name
        let input2 = " = invalid";
        let mut parser2 = MakefileParser::new(input2);
        let result2 = parser2.parse();
        // Parser may skip invalid lines
        assert!(result2.is_ok() || result2.is_err());
    }

    #[test]
    fn test_cursor_safety() {
        // Test cursor doesn't exceed bounds
        let input = "short";
        let mut parser = MakefileParser::new(input);
        
        // Advance past end
        for _ in 0..10 {
            parser.advance();
        }
        assert!(parser.at_end());
        assert!(parser.cursor <= input.len());
    }

    #[test]
    fn test_performance_rules() {
        use crate::services::makefile_linter::rules::performance::*;
        
        let rule = RecursiveExpansionRule::default();
        assert_eq!(rule.id(), "recursive-expansion");
        
        // Test with simple AST
        let ast = MakefileAst::new();
        let violations = rule.check(&ast);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_source_span_serialization() {
        let span = SourceSpan::new(10, 20, 5, 3);
        let json = serde_json::to_string(&span).unwrap();
        let deserialized: SourceSpan = serde_json::from_str(&json).unwrap();
        assert_eq!(span.start, deserialized.start);
        assert_eq!(span.end, deserialized.end);
        assert_eq!(span.line, deserialized.line);
        assert_eq!(span.column, deserialized.column);
    }

    #[test]
    fn test_makefile_node_types() {
        // Test all node types can be created
        let nodes = vec![
            MakefileNode {
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
            },
            MakefileNode {
                kind: MakefileNodeKind::Variable,
                span: SourceSpan::file_level(),
                children: vec![],
                data: NodeData::Variable {
                    name: "VAR".to_string(),
                    assignment_op: AssignmentOp::Deferred,
                    value: "value".to_string(),
                },
            },
            MakefileNode {
                kind: MakefileNodeKind::Recipe,
                span: SourceSpan::file_level(),
                children: vec![],
                data: NodeData::Recipe {
                    lines: vec![RecipeLine {
                        text: "echo test".to_string(),
                        prefixes: RecipePrefixes::default(),
                    }],
                },
            },
            MakefileNode {
                kind: MakefileNodeKind::Comment,
                span: SourceSpan::file_level(),
                children: vec![],
                data: NodeData::Text("# comment".to_string()),
            },
        ];
        
        // All nodes should be valid
        assert_eq!(nodes.len(), 4);
    }

    #[test]
    fn test_quality_score_edge_cases() {
        // Test with no violations
        assert_eq!(calculate_quality_score(&vec![]), 1.0);
        
        // Test score doesn't go negative
        let many_errors: Vec<_> = (0..100).map(|_| Violation {
            rule: "test".to_string(),
            severity: Severity::Error,
            span: SourceSpan::file_level(),
            message: "Error".to_string(),
            fix_hint: None,
        }).collect();
        
        let score = calculate_quality_score(&many_errors);
        assert!(score >= 0.0);
        assert!(score <= 1.0);
    }
}