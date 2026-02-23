#![cfg_attr(coverage_nightly, coverage(off))]

use super::*;
use syn::parse_quote;

fn create_test_location() -> SourceLocation {
    SourceLocation {
        line: 1,
        column: 1,
        end_line: 1,
        end_column: 10,
    }
}

#[test]
fn test_arithmetic_operator_replacement_subtraction() {
    let operator = ArithmeticOperatorReplacement;
    let expr: syn::Expr = parse_quote!(a - b);
    let location = create_test_location();

    assert!(operator.can_mutate(&expr));
    assert_eq!(operator.name(), "AOR");
    assert_eq!(
        operator.operator_type(),
        MutationOperatorType::ArithmeticReplacement
    );

    let mutants = operator.mutate(&expr, location).unwrap();
    assert!(mutants.len() >= 3); // +, *, /
}

#[test]
fn test_arithmetic_operator_replacement_multiplication() {
    let operator = ArithmeticOperatorReplacement;
    let expr: syn::Expr = parse_quote!(a * b);
    let location = create_test_location();

    assert!(operator.can_mutate(&expr));

    let mutants = operator.mutate(&expr, location).unwrap();
    assert!(mutants.len() >= 3); // +, -, /
}

#[test]
fn test_arithmetic_operator_replacement_division() {
    let operator = ArithmeticOperatorReplacement;
    let expr: syn::Expr = parse_quote!(a / b);
    let location = create_test_location();

    assert!(operator.can_mutate(&expr));

    let mutants = operator.mutate(&expr, location).unwrap();
    assert!(mutants.len() >= 3); // +, -, *
}

#[test]
fn test_arithmetic_operator_replacement_modulo() {
    let operator = ArithmeticOperatorReplacement;
    let expr: syn::Expr = parse_quote!(a % b);
    let location = create_test_location();

    assert!(operator.can_mutate(&expr));

    let mutants = operator.mutate(&expr, location).unwrap();
    assert!(mutants.len() >= 2); // *, /
}

#[test]
fn test_arithmetic_operator_not_applicable_to_comparison() {
    let operator = ArithmeticOperatorReplacement;
    let expr: syn::Expr = parse_quote!(a < b);
    let location = create_test_location();

    assert!(!operator.can_mutate(&expr));

    let mutants = operator.mutate(&expr, location).unwrap();
    assert!(mutants.is_empty());
}

#[test]
fn test_relational_operator_replacement_all_variants() {
    let operator = RelationalOperatorReplacement;
    let location = create_test_location();

    let test_cases = vec![
        parse_quote!(a < b),
        parse_quote!(a <= b),
        parse_quote!(a > b),
        parse_quote!(a >= b),
        parse_quote!(a == b),
        parse_quote!(a != b),
    ];

    for expr in test_cases {
        assert!(operator.can_mutate(&expr));
        let mutants = operator.mutate(&expr, location.clone()).unwrap();
        assert!(mutants.len() >= 5); // Each should produce 5 alternatives
    }
}

#[test]
fn test_relational_operator_not_applicable_to_arithmetic() {
    let operator = RelationalOperatorReplacement;
    let expr: syn::Expr = parse_quote!(a + b);

    assert!(!operator.can_mutate(&expr));
}

#[test]
fn test_conditional_operator_replacement_and_to_or() {
    let operator = ConditionalOperatorReplacement;
    let expr: syn::Expr = parse_quote!(a && b);
    let location = create_test_location();

    assert!(operator.can_mutate(&expr));
    assert_eq!(operator.name(), "COR");

    let mutants = operator.mutate(&expr, location).unwrap();
    assert_eq!(mutants.len(), 1); // && -> ||
}

#[test]
fn test_conditional_operator_replacement_or_to_and() {
    let operator = ConditionalOperatorReplacement;
    let expr: syn::Expr = parse_quote!(a || b);
    let location = create_test_location();

    assert!(operator.can_mutate(&expr));

    let mutants = operator.mutate(&expr, location).unwrap();
    assert_eq!(mutants.len(), 1); // || -> &&
}

#[test]
fn test_unary_operator_replacement_not() {
    let operator = UnaryOperatorReplacement;
    let expr: syn::Expr = parse_quote!(!flag);
    let location = create_test_location();

    assert!(operator.can_mutate(&expr));
    assert_eq!(operator.name(), "UOR");

    let mutants = operator.mutate(&expr, location).unwrap();
    assert_eq!(mutants.len(), 1); // Remove !
}

#[test]
fn test_unary_operator_replacement_negation() {
    let operator = UnaryOperatorReplacement;
    let expr: syn::Expr = parse_quote!(-value);
    let location = create_test_location();

    assert!(operator.can_mutate(&expr));

    let mutants = operator.mutate(&expr, location).unwrap();
    assert_eq!(mutants.len(), 1); // Remove -
}

#[test]
fn test_constant_replacement_negative_one() {
    let operator = ConstantReplacementOperator;
    let expr: syn::Expr = syn::parse_str("-1").unwrap();
    let location = create_test_location();

    // The expression -1 is parsed as UnaryNeg(1), not a literal -1
    // so it may not be mutable by constant replacement
    let can_mutate = operator.can_mutate(&expr);
    // This tests the boundary case for constant replacement
    if can_mutate {
        let mutants = operator.mutate(&expr, location).unwrap();
        assert!(!mutants.is_empty());
    }
}

#[test]
fn test_boundary_value_operator() {
    let operator = BoundaryValueOperator;
    let expr: syn::Expr = parse_quote!(10);
    let location = create_test_location();

    assert!(operator.can_mutate(&expr));
    assert_eq!(operator.name(), "BVO");

    let mutants = operator.mutate(&expr, location).unwrap();
    assert_eq!(mutants.len(), 2); // +1 and -1
}

#[test]
fn test_statement_deletion_operator_assignment() {
    let operator = StatementDeletionOperator;
    let expr: syn::Expr = parse_quote!(x = 5);
    let location = create_test_location();

    assert!(operator.can_mutate(&expr));
    assert_eq!(operator.name(), "SDL");

    let mutants = operator.mutate(&expr, location).unwrap();
    assert_eq!(mutants.len(), 1); // Replaced with ()
}

#[test]
fn test_return_value_replacement() {
    let operator = ReturnValueReplacement;
    let expr: syn::Expr = parse_quote!(return x);
    let location = create_test_location();

    assert!(operator.can_mutate(&expr));
    assert_eq!(operator.name(), "RVR");

    let mutants = operator.mutate(&expr, location).unwrap();
    assert_eq!(mutants.len(), 3); // 0, 1, -1
}

#[test]
fn test_exception_handler_removal() {
    let operator = ExceptionHandlerRemoval;
    let expr: syn::Expr = parse_quote!(foo()?);
    let location = create_test_location();

    assert!(operator.can_mutate(&expr));
    assert_eq!(operator.name(), "EHR");

    let mutants = operator.mutate(&expr, location).unwrap();
    assert_eq!(mutants.len(), 1); // Removes ?
}

#[test]
fn test_variable_replacement_operator() {
    let operator = VariableReplacementOperator;
    let expr: syn::Expr = parse_quote!(variable_name);
    let location = create_test_location();

    assert!(operator.can_mutate(&expr));
    assert_eq!(operator.name(), "VRO");

    // VRO requires scope analysis and returns empty for now
    let mutants = operator.mutate(&expr, location).unwrap();
    assert!(mutants.is_empty());
}

#[test]
fn test_conditional_return_operator() {
    let operator = ConditionalReturnOperator;
    let expr: syn::Expr = parse_quote!(return Some(value));
    let location = create_test_location();

    assert!(operator.can_mutate(&expr));
    assert_eq!(operator.name(), "CRO");

    let mutants = operator.mutate(&expr, location).unwrap();
    assert_eq!(mutants.len(), 1); // Early return
}

#[test]
fn test_kill_probability_values() {
    let aor = ArithmeticOperatorReplacement;
    let ror = RelationalOperatorReplacement;
    let cor = ConditionalOperatorReplacement;
    let uor = UnaryOperatorReplacement;
    let ehr = ExceptionHandlerRemoval;

    // All kill probabilities should be between 0.0 and 1.0
    assert!(aor.kill_probability() > 0.0 && aor.kill_probability() <= 1.0);
    assert!(ror.kill_probability() > 0.0 && ror.kill_probability() <= 1.0);
    assert!(cor.kill_probability() > 0.0 && cor.kill_probability() <= 1.0);
    assert!(uor.kill_probability() > 0.0 && uor.kill_probability() <= 1.0);
    assert!(ehr.kill_probability() > 0.0 && ehr.kill_probability() <= 1.0);
}

#[test]
fn test_non_mutatable_expression() {
    let operator = ArithmeticOperatorReplacement;
    let expr: syn::Expr = parse_quote!(foo.bar());
    let location = create_test_location();

    assert!(!operator.can_mutate(&expr));

    let mutants = operator.mutate(&expr, location).unwrap();
    assert!(mutants.is_empty());
}
