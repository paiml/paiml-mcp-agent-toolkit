//! Mutation operators for AST transformation

use super::types::*;
use anyhow::Result;
use syn::{BinOp, Expr, UnOp};

/// Trait for mutation operators
pub trait MutationOperator: Send + Sync {
    /// Name of this operator
    fn name(&self) -> &str;

    /// Operator type
    fn operator_type(&self) -> MutationOperatorType;

    /// Can this operator mutate the given AST node?
    fn can_mutate(&self, expr: &Expr) -> bool;

    /// Generate mutants for the given AST node
    fn mutate(&self, expr: &Expr, location: SourceLocation) -> Result<Vec<Expr>>;

    /// Estimated kill probability (0.0 - 1.0)
    fn kill_probability(&self) -> f64 {
        0.5 // Default 50%
    }
}

/// Arithmetic Operator Replacement (AOR)
/// Replaces: + → -, * → /, % → *, etc.
pub struct ArithmeticOperatorReplacement;

impl MutationOperator for ArithmeticOperatorReplacement {
    fn name(&self) -> &str {
        "AOR"
    }

    fn operator_type(&self) -> MutationOperatorType {
        MutationOperatorType::ArithmeticReplacement
    }

    fn can_mutate(&self, expr: &Expr) -> bool {
        matches!(expr, Expr::Binary(bin) if is_arithmetic_op(&bin.op))
    }

    fn mutate(&self, expr: &Expr, _location: SourceLocation) -> Result<Vec<Expr>> {
        if let Expr::Binary(bin) = expr {
            let mut mutants = Vec::new();

            match bin.op {
                BinOp::Add(_) => {
                    for new_op in [BinOp::Sub(Default::default()), BinOp::Mul(Default::default()), BinOp::Div(Default::default())] {
                        let mut mutated = bin.clone();
                        mutated.op = new_op;
                        mutants.push(Expr::Binary(mutated));
                    }
                }
                BinOp::Sub(_) => {
                    for new_op in [BinOp::Add(Default::default()), BinOp::Mul(Default::default()), BinOp::Div(Default::default())] {
                        let mut mutated = bin.clone();
                        mutated.op = new_op;
                        mutants.push(Expr::Binary(mutated));
                    }
                }
                BinOp::Mul(_) => {
                    for new_op in [BinOp::Add(Default::default()), BinOp::Sub(Default::default()), BinOp::Div(Default::default())] {
                        let mut mutated = bin.clone();
                        mutated.op = new_op;
                        mutants.push(Expr::Binary(mutated));
                    }
                }
                BinOp::Div(_) => {
                    for new_op in [BinOp::Add(Default::default()), BinOp::Sub(Default::default()), BinOp::Mul(Default::default())] {
                        let mut mutated = bin.clone();
                        mutated.op = new_op;
                        mutants.push(Expr::Binary(mutated));
                    }
                }
                BinOp::Rem(_) => {
                    for new_op in [BinOp::Mul(Default::default()), BinOp::Div(Default::default())] {
                        let mut mutated = bin.clone();
                        mutated.op = new_op;
                        mutants.push(Expr::Binary(mutated));
                    }
                }
                _ => {}
            }

            return Ok(mutants);
        }

        Ok(vec![])
    }

    fn kill_probability(&self) -> f64 {
        0.85 // Arithmetic changes are usually caught
    }
}

/// Relational Operator Replacement (ROR)
/// Replaces: < → <=, == → !=, > → >=, etc.
pub struct RelationalOperatorReplacement;

impl MutationOperator for RelationalOperatorReplacement {
    fn name(&self) -> &str {
        "ROR"
    }

    fn operator_type(&self) -> MutationOperatorType {
        MutationOperatorType::RelationalReplacement
    }

    fn can_mutate(&self, expr: &Expr) -> bool {
        matches!(expr, Expr::Binary(bin) if is_relational_op(&bin.op))
    }

    fn mutate(&self, expr: &Expr, _location: SourceLocation) -> Result<Vec<Expr>> {
        if let Expr::Binary(bin) = expr {
            let mut mutants = Vec::new();

            match bin.op {
                BinOp::Lt(_) => {
                    for new_op in [BinOp::Le(Default::default()), BinOp::Gt(Default::default()), BinOp::Ge(Default::default()), BinOp::Eq(Default::default()), BinOp::Ne(Default::default())] {
                        let mut mutated = bin.clone();
                        mutated.op = new_op;
                        mutants.push(Expr::Binary(mutated));
                    }
                }
                BinOp::Le(_) => {
                    for new_op in [BinOp::Lt(Default::default()), BinOp::Gt(Default::default()), BinOp::Ge(Default::default()), BinOp::Eq(Default::default()), BinOp::Ne(Default::default())] {
                        let mut mutated = bin.clone();
                        mutated.op = new_op;
                        mutants.push(Expr::Binary(mutated));
                    }
                }
                BinOp::Gt(_) => {
                    for new_op in [BinOp::Lt(Default::default()), BinOp::Le(Default::default()), BinOp::Ge(Default::default()), BinOp::Eq(Default::default()), BinOp::Ne(Default::default())] {
                        let mut mutated = bin.clone();
                        mutated.op = new_op;
                        mutants.push(Expr::Binary(mutated));
                    }
                }
                BinOp::Ge(_) => {
                    for new_op in [BinOp::Lt(Default::default()), BinOp::Le(Default::default()), BinOp::Gt(Default::default()), BinOp::Eq(Default::default()), BinOp::Ne(Default::default())] {
                        let mut mutated = bin.clone();
                        mutated.op = new_op;
                        mutants.push(Expr::Binary(mutated));
                    }
                }
                BinOp::Eq(_) => {
                    for new_op in [BinOp::Ne(Default::default()), BinOp::Lt(Default::default()), BinOp::Le(Default::default()), BinOp::Gt(Default::default()), BinOp::Ge(Default::default())] {
                        let mut mutated = bin.clone();
                        mutated.op = new_op;
                        mutants.push(Expr::Binary(mutated));
                    }
                }
                BinOp::Ne(_) => {
                    for new_op in [BinOp::Eq(Default::default()), BinOp::Lt(Default::default()), BinOp::Le(Default::default()), BinOp::Gt(Default::default()), BinOp::Ge(Default::default())] {
                        let mut mutated = bin.clone();
                        mutated.op = new_op;
                        mutants.push(Expr::Binary(mutated));
                    }
                }
                _ => {}
            }

            return Ok(mutants);
        }

        Ok(vec![])
    }

    fn kill_probability(&self) -> f64 {
        0.90 // Relational changes often caught by tests
    }
}

/// Conditional Operator Replacement (COR)
/// Replaces: && → ||, || → &&
pub struct ConditionalOperatorReplacement;

impl MutationOperator for ConditionalOperatorReplacement {
    fn name(&self) -> &str {
        "COR"
    }

    fn operator_type(&self) -> MutationOperatorType {
        MutationOperatorType::ConditionalReplacement
    }

    fn can_mutate(&self, expr: &Expr) -> bool {
        matches!(expr, Expr::Binary(bin) if is_logical_op(&bin.op))
    }

    fn mutate(&self, expr: &Expr, _location: SourceLocation) -> Result<Vec<Expr>> {
        if let Expr::Binary(bin) = expr {
            let replacement = match bin.op {
                BinOp::And(_) => Some(BinOp::Or(Default::default())),
                BinOp::Or(_) => Some(BinOp::And(Default::default())),
                _ => None,
            };

            if let Some(new_op) = replacement {
                let mut mutated = bin.clone();
                mutated.op = new_op;
                return Ok(vec![Expr::Binary(mutated)]);
            }
        }

        Ok(vec![])
    }

    fn kill_probability(&self) -> f64 {
        0.80 // Logic changes often detected
    }
}

/// Unary Operator Replacement
/// Replaces: ! → identity, - → +
pub struct UnaryOperatorReplacement;

impl MutationOperator for UnaryOperatorReplacement {
    fn name(&self) -> &str {
        "UOR"
    }

    fn operator_type(&self) -> MutationOperatorType {
        MutationOperatorType::UnaryReplacement
    }

    fn can_mutate(&self, expr: &Expr) -> bool {
        matches!(expr, Expr::Unary(_))
    }

    fn mutate(&self, expr: &Expr, _location: SourceLocation) -> Result<Vec<Expr>> {
        if let Expr::Unary(unary) = expr {
            match unary.op {
                UnOp::Not(_) => {
                    // Remove negation
                    Ok(vec![(*unary.expr).clone()])
                }
                UnOp::Neg(_) => {
                    // Remove negation or add positive sign
                    Ok(vec![(*unary.expr).clone()])
                }
                _ => Ok(vec![]),
            }
        } else {
            Ok(vec![])
        }
    }

    fn kill_probability(&self) -> f64 {
        0.75
    }
}

// Helper functions
fn is_arithmetic_op(op: &BinOp) -> bool {
    matches!(
        op,
        BinOp::Add(_) | BinOp::Sub(_) | BinOp::Mul(_) | BinOp::Div(_) | BinOp::Rem(_)
    )
}

fn is_relational_op(op: &BinOp) -> bool {
    matches!(
        op,
        BinOp::Lt(_) | BinOp::Le(_) | BinOp::Gt(_) | BinOp::Ge(_) | BinOp::Eq(_) | BinOp::Ne(_)
    )
}

fn is_logical_op(op: &BinOp) -> bool {
    matches!(op, BinOp::And(_) | BinOp::Or(_))
}

/// Conditional Return Operator (CRO)
/// Generates early returns to test guard clauses
pub struct ConditionalReturnOperator;

impl MutationOperator for ConditionalReturnOperator {
    fn name(&self) -> &str {
        "CRO"
    }

    fn operator_type(&self) -> MutationOperatorType {
        MutationOperatorType::ConditionalReturn
    }

    fn can_mutate(&self, expr: &Expr) -> bool {
        matches!(expr, Expr::Return(_))
    }

    fn mutate(&self, expr: &Expr, _location: SourceLocation) -> Result<Vec<Expr>> {
        if let Expr::Return(_) = expr {
            // Generate early return mutant
            let early_return: Expr = syn::parse_quote!(return);
            return Ok(vec![early_return]);
        }
        Ok(vec![])
    }

    fn kill_probability(&self) -> f64 {
        0.70
    }
}

/// Statement Deletion Operator (SDL)
/// Advanced operator for Phase 5 - removes statements to test necessity
/// Can delete: assignments, method calls, function calls
pub struct StatementDeletionOperator;

impl MutationOperator for StatementDeletionOperator {
    fn name(&self) -> &str {
        "SDL"
    }

    fn operator_type(&self) -> MutationOperatorType {
        MutationOperatorType::StatementDeletion
    }

    fn can_mutate(&self, expr: &Expr) -> bool {
        // Can delete assignments, method calls, and function calls
        matches!(
            expr,
            Expr::Assign(_)
                | Expr::Call(_)
                | Expr::MethodCall(_)
                | Expr::Macro(_)
        )
    }

    fn mutate(&self, expr: &Expr, _location: SourceLocation) -> Result<Vec<Expr>> {
        // For statement deletion, we return a unit expression ()
        // This represents removing the statement
        match expr {
            Expr::Assign(_) => {
                // Delete assignment - replace with ()
                Ok(vec![syn::parse_quote!(())])
            }
            Expr::Call(_) | Expr::MethodCall(_) => {
                // Delete function/method call - replace with ()
                Ok(vec![syn::parse_quote!(())])
            }
            Expr::Macro(_) => {
                // Delete macro call - replace with ()
                Ok(vec![syn::parse_quote!(())])
            }
            _ => Ok(vec![]),
        }
    }

    fn kill_probability(&self) -> f64 {
        0.75 // Statement deletions often caught by tests
    }
}

/// Return Value Replacement (RVR)
/// Replaces return values with common alternatives
pub struct ReturnValueReplacement;

impl MutationOperator for ReturnValueReplacement {
    fn name(&self) -> &str {
        "RVR"
    }

    fn operator_type(&self) -> MutationOperatorType {
        MutationOperatorType::ReturnValueReplacement
    }

    fn can_mutate(&self, expr: &Expr) -> bool {
        matches!(expr, Expr::Return(_))
    }

    fn mutate(&self, expr: &Expr, _location: SourceLocation) -> Result<Vec<Expr>> {
        if let Expr::Return(_) = expr {
            // Generate alternative return values
            let mutants = vec![
                syn::parse_quote!(return 0),
                syn::parse_quote!(return 1),
                syn::parse_quote!(return -1),
            ];
            return Ok(mutants);
        }
        Ok(vec![])
    }

    fn kill_probability(&self) -> f64 {
        0.80
    }
}

/// Variable Replacement Operator (VRO)
/// Replaces variables with other in-scope variables
pub struct VariableReplacementOperator;

impl MutationOperator for VariableReplacementOperator {
    fn name(&self) -> &str {
        "VRO"
    }

    fn operator_type(&self) -> MutationOperatorType {
        MutationOperatorType::VariableReplacement
    }

    fn can_mutate(&self, expr: &Expr) -> bool {
        matches!(expr, Expr::Path(_))
    }

    fn mutate(&self, _expr: &Expr, _location: SourceLocation) -> Result<Vec<Expr>> {
        // Minimal: variable replacement requires scope analysis
        Ok(vec![])
    }

    fn kill_probability(&self) -> f64 {
        0.75
    }
}

/// Boundary Value Operator (BVO)
/// Creates off-by-one mutations for boundary testing
pub struct BoundaryValueOperator;

impl MutationOperator for BoundaryValueOperator {
    fn name(&self) -> &str {
        "BVO"
    }

    fn operator_type(&self) -> MutationOperatorType {
        MutationOperatorType::BoundaryValue
    }

    fn can_mutate(&self, expr: &Expr) -> bool {
        matches!(expr, Expr::Lit(syn::ExprLit { lit: syn::Lit::Int(_), .. }))
    }

    fn mutate(&self, expr: &Expr, _location: SourceLocation) -> Result<Vec<Expr>> {
        if let Expr::Lit(syn::ExprLit { lit: syn::Lit::Int(lit_int), .. }) = expr {
            if let Ok(value) = lit_int.base10_parse::<i64>() {
                let plus_one = value + 1;
                let minus_one = value - 1;

                let mutants = vec![
                    syn::parse_quote!(#plus_one),
                    syn::parse_quote!(#minus_one),
                ];
                return Ok(mutants);
            }
        }
        Ok(vec![])
    }

    fn kill_probability(&self) -> f64 {
        0.85
    }
}

/// Constant Replacement (CRR)
/// Advanced operator for Phase 5 - replaces constants with common alternatives
/// Handles: integers, booleans, strings, floats
pub struct ConstantReplacementOperator;

impl MutationOperator for ConstantReplacementOperator {
    fn name(&self) -> &str {
        "CRR"
    }

    fn operator_type(&self) -> MutationOperatorType {
        MutationOperatorType::ConstantReplacement
    }

    fn can_mutate(&self, expr: &Expr) -> bool {
        matches!(
            expr,
            Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(_) | syn::Lit::Bool(_) | syn::Lit::Str(_) | syn::Lit::Float(_),
                ..
            })
        )
    }

    fn mutate(&self, expr: &Expr, _location: SourceLocation) -> Result<Vec<Expr>> {
        if let Expr::Lit(lit_expr) = expr {
            match &lit_expr.lit {
                // Integer replacements: 0→1, 1→0, n→n+1, n→n-1
                syn::Lit::Int(lit_int) => {
                    if let Ok(value) = lit_int.base10_parse::<i64>() {
                        let mutants = match value {
                            0 => vec![syn::parse_quote!(1), syn::parse_quote!(-1)],
                            1 => vec![syn::parse_quote!(0), syn::parse_quote!(2)],
                            -1 => vec![syn::parse_quote!(0), syn::parse_quote!(1)],
                            n => {
                                let plus = n + 1;
                                let minus = n - 1;
                                vec![
                                    syn::parse_quote!(0),
                                    syn::parse_quote!(1),
                                    syn::parse_quote!(#plus),
                                    syn::parse_quote!(#minus),
                                ]
                            }
                        };
                        return Ok(mutants);
                    }
                }
                // Boolean replacements: true→false, false→true
                syn::Lit::Bool(lit_bool) => {
                    let replacement = !lit_bool.value;
                    return Ok(vec![syn::parse_quote!(#replacement)]);
                }
                // String replacements: ""→"null", "x"→""
                syn::Lit::Str(lit_str) => {
                    let value = lit_str.value();
                    let mutants = if value.is_empty() {
                        vec![syn::parse_quote!("null"), syn::parse_quote!("undefined")]
                    } else {
                        vec![syn::parse_quote!(""), syn::parse_quote!("null")]
                    };
                    return Ok(mutants);
                }
                // Float replacements: 0.0→1.0, n→n+1.0, n→n-1.0
                syn::Lit::Float(lit_float) => {
                    if let Ok(value) = lit_float.base10_parse::<f64>() {
                        let plus = value + 1.0;
                        let minus = value - 1.0;
                        let mutants = vec![
                            syn::parse_quote!(0.0),
                            syn::parse_quote!(1.0),
                            syn::parse_quote!(#plus),
                            syn::parse_quote!(#minus),
                        ];
                        return Ok(mutants);
                    }
                }
                _ => {}
            }
        }
        Ok(vec![])
    }

    fn kill_probability(&self) -> f64 {
        0.82 // Constants are often in critical logic
    }
}

/// Exception Handler Removal (EHR)
/// Removes try/? operators to test error handling
pub struct ExceptionHandlerRemoval;

impl MutationOperator for ExceptionHandlerRemoval {
    fn name(&self) -> &str {
        "EHR"
    }

    fn operator_type(&self) -> MutationOperatorType {
        MutationOperatorType::ExceptionHandlerRemoval
    }

    fn can_mutate(&self, expr: &Expr) -> bool {
        matches!(expr, Expr::Try(_))
    }

    fn mutate(&self, expr: &Expr, _location: SourceLocation) -> Result<Vec<Expr>> {
        if let Expr::Try(try_expr) = expr {
            // Remove the ? operator
            return Ok(vec![(*try_expr.expr).clone()]);
        }
        Ok(vec![])
    }

    fn kill_probability(&self) -> f64 {
        0.90
    }
}

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
