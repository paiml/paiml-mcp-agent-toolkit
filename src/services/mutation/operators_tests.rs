#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_arithmetic_operator_replacement() {
        let operator = ArithmeticOperatorReplacement;
        let expr: Expr = parse_quote!(a + b);
        let location = SourceLocation {
            line: 1,
            column: 1,
            end_line: 1,
            end_column: 5,
        };

        assert!(operator.can_mutate(&expr));

        let mutants = operator.mutate(&expr, location).unwrap();
        assert!(mutants.len() >= 3); // -, *, /
    }

    #[test]
    fn test_relational_operator_replacement() {
        let operator = RelationalOperatorReplacement;
        let expr: Expr = parse_quote!(x < y);
        let location = SourceLocation {
            line: 1,
            column: 1,
            end_line: 1,
            end_column: 5,
        };

        assert!(operator.can_mutate(&expr));

        let mutants = operator.mutate(&expr, location).unwrap();
        assert!(mutants.len() >= 5); // <=, >, >=, ==, !=
    }

    #[test]
    fn test_conditional_operator_replacement() {
        let operator = ConditionalOperatorReplacement;
        let expr: Expr = parse_quote!(a && b);
        let location = SourceLocation {
            line: 1,
            column: 1,
            end_line: 1,
            end_column: 6,
        };

        assert!(operator.can_mutate(&expr));

        let mutants = operator.mutate(&expr, location).unwrap();
        assert_eq!(mutants.len(), 1); // ||
    }

    #[test]
    fn test_unary_operator_replacement() {
        let operator = UnaryOperatorReplacement;
        let expr: Expr = parse_quote!(!flag);
        let location = SourceLocation {
            line: 1,
            column: 1,
            end_line: 1,
            end_column: 5,
        };

        assert!(operator.can_mutate(&expr));

        let mutants = operator.mutate(&expr, location).unwrap();
        assert_eq!(mutants.len(), 1); // Remove !
    }

    // Phase 5 Advanced Operator Tests

    #[test]
    fn test_constant_replacement_integer_zero() {
        let operator = ConstantReplacementOperator;
        let expr: Expr = parse_quote!(0);
        let location = SourceLocation {
            line: 1,
            column: 1,
            end_line: 1,
            end_column: 1,
        };

        assert!(operator.can_mutate(&expr));

        let mutants = operator.mutate(&expr, location).unwrap();
        assert_eq!(mutants.len(), 2); // 1, -1
    }

    #[test]
    fn test_constant_replacement_integer_one() {
        let operator = ConstantReplacementOperator;
        let expr: Expr = parse_quote!(1);
        let location = SourceLocation {
            line: 1,
            column: 1,
            end_line: 1,
            end_column: 1,
        };

        assert!(operator.can_mutate(&expr));

        let mutants = operator.mutate(&expr, location).unwrap();
        assert_eq!(mutants.len(), 2); // 0, 2
    }

    #[test]
    fn test_constant_replacement_integer_arbitrary() {
        let operator = ConstantReplacementOperator;
        let expr: Expr = parse_quote!(42);
        let location = SourceLocation {
            line: 1,
            column: 1,
            end_line: 1,
            end_column: 2,
        };

        assert!(operator.can_mutate(&expr));

        let mutants = operator.mutate(&expr, location).unwrap();
        assert_eq!(mutants.len(), 4); // 0, 1, 43, 41
    }

    #[test]
    fn test_constant_replacement_boolean_true() {
        let operator = ConstantReplacementOperator;
        let expr: Expr = parse_quote!(true);
        let location = SourceLocation {
            line: 1,
            column: 1,
            end_line: 1,
            end_column: 4,
        };

        assert!(operator.can_mutate(&expr));

        let mutants = operator.mutate(&expr, location).unwrap();
        assert_eq!(mutants.len(), 1); // false
    }

    #[test]
    fn test_constant_replacement_boolean_false() {
        let operator = ConstantReplacementOperator;
        let expr: Expr = parse_quote!(false);
        let location = SourceLocation {
            line: 1,
            column: 1,
            end_line: 1,
            end_column: 5,
        };

        assert!(operator.can_mutate(&expr));

        let mutants = operator.mutate(&expr, location).unwrap();
        assert_eq!(mutants.len(), 1); // true
    }

    #[test]
    fn test_constant_replacement_string_empty() {
        let operator = ConstantReplacementOperator;
        let expr: Expr = parse_quote!("");
        let location = SourceLocation {
            line: 1,
            column: 1,
            end_line: 1,
            end_column: 2,
        };

        assert!(operator.can_mutate(&expr));

        let mutants = operator.mutate(&expr, location).unwrap();
        assert_eq!(mutants.len(), 2); // "null", "undefined"
    }

    #[test]
    fn test_constant_replacement_string_nonempty() {
        let operator = ConstantReplacementOperator;
        let expr: Expr = parse_quote!("hello");
        let location = SourceLocation {
            line: 1,
            column: 1,
            end_line: 1,
            end_column: 7,
        };

        assert!(operator.can_mutate(&expr));

        let mutants = operator.mutate(&expr, location).unwrap();
        assert_eq!(mutants.len(), 2); // "", "null"
    }

    #[test]
    fn test_constant_replacement_float() {
        let operator = ConstantReplacementOperator;
        let expr: Expr = parse_quote!(3.14);
        let location = SourceLocation {
            line: 1,
            column: 1,
            end_line: 1,
            end_column: 4,
        };

        assert!(operator.can_mutate(&expr));

        let mutants = operator.mutate(&expr, location).unwrap();
        assert_eq!(mutants.len(), 4); // 0.0, 1.0, 4.14, 2.14
    }

    #[test]
    fn test_statement_deletion_assignment() {
        let operator = StatementDeletionOperator;
        let expr: Expr = parse_quote!(x = 5);
        let location = SourceLocation {
            line: 1,
            column: 1,
            end_line: 1,
            end_column: 5,
        };

        assert!(operator.can_mutate(&expr));

        let mutants = operator.mutate(&expr, location).unwrap();
        assert_eq!(mutants.len(), 1); // ()
    }

    #[test]
    fn test_statement_deletion_method_call() {
        let operator = StatementDeletionOperator;
        let expr: Expr = parse_quote!(obj.method());
        let location = SourceLocation {
            line: 1,
            column: 1,
            end_line: 1,
            end_column: 12,
        };

        assert!(operator.can_mutate(&expr));

        let mutants = operator.mutate(&expr, location).unwrap();
        assert_eq!(mutants.len(), 1); // ()
    }

    #[test]
    fn test_statement_deletion_function_call() {
        let operator = StatementDeletionOperator;
        let expr: Expr = parse_quote!(func());
        let location = SourceLocation {
            line: 1,
            column: 1,
            end_line: 1,
            end_column: 6,
        };

        assert!(operator.can_mutate(&expr));

        let mutants = operator.mutate(&expr, location).unwrap();
        assert_eq!(mutants.len(), 1); // ()
    }

    #[test]
    fn test_statement_deletion_macro_call() {
        let operator = StatementDeletionOperator;
        let expr: Expr = parse_quote!(println!("test"));
        let location = SourceLocation {
            line: 1,
            column: 1,
            end_line: 1,
            end_column: 18,
        };

        assert!(operator.can_mutate(&expr));

        let mutants = operator.mutate(&expr, location).unwrap();
        assert_eq!(mutants.len(), 1); // ()
    }
}
