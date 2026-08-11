// Tests for the line-type scanner's colon handling
// Included by parser.rs - shares parent module scope (no `use` imports here)

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod colon_operator_tests {
    //! `X ::= hello` (POSIX immediate assignment) was scanned as a
    //! double-colon *rule*, so `pmat analyze makefile` reported a target named
    //! `X` and emitted "Target 'X' should probably be declared .PHONY" for what
    //! is a variable definition.
    use super::*;

    fn variables(src: &str) -> Vec<(String, AssignmentOp, String)> {
        let mut parser = MakefileParser::new(src);
        let ast = parser.parse().expect("parse must succeed");
        ast.get_variables()
            .into_iter()
            .map(|(n, op, v)| (n.clone(), *op, v.clone()))
            .collect()
    }

    fn targets(src: &str) -> Vec<String> {
        let mut parser = MakefileParser::new(src);
        let ast = parser.parse().expect("parse must succeed");
        ast.nodes
            .iter()
            .filter_map(|n| match &n.data {
                NodeData::Rule { targets, .. } => Some(targets.clone()),
                _ => None,
            })
            .flatten()
            .collect()
    }

    #[test]
    fn posix_immediate_assignment_is_a_variable_not_a_rule() {
        assert_eq!(
            variables("X ::= hello\n"),
            vec![(
                "X".to_string(),
                AssignmentOp::Immediate,
                "hello".to_string()
            )]
        );
        assert!(
            targets("X ::= hello\n").is_empty(),
            "`::=` must not produce a rule target"
        );
    }

    #[test]
    fn gnu_44_immediate_with_expansion_is_a_variable_not_a_rule() {
        assert_eq!(
            variables("X :::= hello\n"),
            vec![(
                "X".to_string(),
                AssignmentOp::Immediate,
                "hello".to_string()
            )]
        );
        assert!(targets("X :::= hello\n").is_empty());
    }

    #[test]
    fn plain_immediate_assignment_still_parses() {
        assert_eq!(
            variables("X := hello\n"),
            vec![(
                "X".to_string(),
                AssignmentOp::Immediate,
                "hello".to_string()
            )]
        );
    }

    #[test]
    fn a_real_double_colon_rule_is_still_a_rule() {
        assert_eq!(targets("build:: dep\n\techo hi\n"), vec!["build".to_string()]);
        assert!(variables("build:: dep\n\techo hi\n").is_empty());
    }
}
