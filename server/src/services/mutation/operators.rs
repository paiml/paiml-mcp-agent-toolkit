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
        MutationOperatorType::ConditionalReplacement
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

/// Statement Deletion Operator (SDO)
/// Removes non-critical statements to test necessity
pub struct StatementDeletionOperator;

impl MutationOperator for StatementDeletionOperator {
    fn name(&self) -> &str {
        "SDO"
    }

    fn operator_type(&self) -> MutationOperatorType {
        MutationOperatorType::StatementDeletion
    }

    fn can_mutate(&self, expr: &Expr) -> bool {
        // Can delete assignments and method calls
        matches!(expr, Expr::Assign(_) | Expr::Call(_) | Expr::MethodCall(_))
    }

    fn mutate(&self, _expr: &Expr, _location: SourceLocation) -> Result<Vec<Expr>> {
        // Minimal: return empty vec (deletion simulated elsewhere)
        Ok(vec![])
    }

    fn kill_probability(&self) -> f64 {
        0.65
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
}
