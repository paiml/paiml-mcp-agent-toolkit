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
            let mutants = mutate_arithmetic_op(bin);
            return Ok(mutants);
        }
        Ok(vec![])
    }

    fn kill_probability(&self) -> f64 {
        0.85 // Arithmetic changes are usually caught
    }
}

fn mutate_arithmetic_op(bin: &syn::ExprBinary) -> Vec<Expr> {
    let replacements: &[BinOp] = match bin.op {
        BinOp::Add(_) => &[
            BinOp::Sub(Default::default()),
            BinOp::Mul(Default::default()),
            BinOp::Div(Default::default()),
        ],
        BinOp::Sub(_) => &[
            BinOp::Add(Default::default()),
            BinOp::Mul(Default::default()),
            BinOp::Div(Default::default()),
        ],
        BinOp::Mul(_) => &[
            BinOp::Add(Default::default()),
            BinOp::Sub(Default::default()),
            BinOp::Div(Default::default()),
        ],
        BinOp::Div(_) => &[
            BinOp::Add(Default::default()),
            BinOp::Sub(Default::default()),
            BinOp::Mul(Default::default()),
        ],
        BinOp::Rem(_) => &[
            BinOp::Mul(Default::default()),
            BinOp::Div(Default::default()),
        ],
        _ => &[],
    };
    apply_binary_replacements(bin, replacements)
}

fn apply_binary_replacements(bin: &syn::ExprBinary, ops: &[BinOp]) -> Vec<Expr> {
    ops.iter()
        .map(|&new_op| {
            let mut mutated = bin.clone();
            mutated.op = new_op;
            Expr::Binary(mutated)
        })
        .collect()
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
            let mutants = mutate_relational_op(bin);
            return Ok(mutants);
        }
        Ok(vec![])
    }

    fn kill_probability(&self) -> f64 {
        0.90 // Relational changes often caught by tests
    }
}

fn mutate_relational_op(bin: &syn::ExprBinary) -> Vec<Expr> {
    let replacements: &[BinOp] = match bin.op {
        BinOp::Lt(_) => &[
            BinOp::Le(Default::default()),
            BinOp::Gt(Default::default()),
            BinOp::Ge(Default::default()),
            BinOp::Eq(Default::default()),
            BinOp::Ne(Default::default()),
        ],
        BinOp::Le(_) => &[
            BinOp::Lt(Default::default()),
            BinOp::Gt(Default::default()),
            BinOp::Ge(Default::default()),
            BinOp::Eq(Default::default()),
            BinOp::Ne(Default::default()),
        ],
        BinOp::Gt(_) => &[
            BinOp::Lt(Default::default()),
            BinOp::Le(Default::default()),
            BinOp::Ge(Default::default()),
            BinOp::Eq(Default::default()),
            BinOp::Ne(Default::default()),
        ],
        BinOp::Ge(_) => &[
            BinOp::Lt(Default::default()),
            BinOp::Le(Default::default()),
            BinOp::Gt(Default::default()),
            BinOp::Eq(Default::default()),
            BinOp::Ne(Default::default()),
        ],
        BinOp::Eq(_) => &[
            BinOp::Ne(Default::default()),
            BinOp::Lt(Default::default()),
            BinOp::Le(Default::default()),
            BinOp::Gt(Default::default()),
            BinOp::Ge(Default::default()),
        ],
        BinOp::Ne(_) => &[
            BinOp::Eq(Default::default()),
            BinOp::Lt(Default::default()),
            BinOp::Le(Default::default()),
            BinOp::Gt(Default::default()),
            BinOp::Ge(Default::default()),
        ],
        _ => &[],
    };
    apply_binary_replacements(bin, replacements)
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

// Helper functions for operator classification
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn is_arithmetic_op(op: &BinOp) -> bool {
    matches!(
        op,
        BinOp::Add(_) | BinOp::Sub(_) | BinOp::Mul(_) | BinOp::Div(_) | BinOp::Rem(_)
    )
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn is_relational_op(op: &BinOp) -> bool {
    matches!(
        op,
        BinOp::Lt(_) | BinOp::Le(_) | BinOp::Gt(_) | BinOp::Ge(_) | BinOp::Eq(_) | BinOp::Ne(_)
    )
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn is_logical_op(op: &BinOp) -> bool {
    matches!(op, BinOp::And(_) | BinOp::Or(_))
}
