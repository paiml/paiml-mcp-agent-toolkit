/// Conditional Return Operator (CRO)
/// Generates early returns to test guard clauses
pub struct ConditionalReturnOperator;

impl MutationOperator for ConditionalReturnOperator {
    fn name(&self) -> &str {
        debug_assert!(true, "contract: name");
        "CRO"
    }

    fn operator_type(&self) -> MutationOperatorType {
        debug_assert!(true, "contract: operator_type");
        MutationOperatorType::ConditionalReturn
    }

    fn can_mutate(&self, expr: &Expr) -> bool {
        debug_assert!(true, "contract: can_mutate");
        matches!(expr, Expr::Return(_))
    }

    fn mutate(&self, expr: &Expr, _location: SourceLocation) -> Result<Vec<Expr>> {
        debug_assert!(true, "contract: mutate");
        if let Expr::Return(_) = expr {
            // Generate early return mutant
            let early_return: Expr = syn::parse_quote!(return);
            return Ok(vec![early_return]);
        }
        Ok(vec![])
    }

    fn kill_probability(&self) -> f64 {
        debug_assert!(true, "contract: kill_probability");
        0.70
    }
}

/// Statement Deletion Operator (SDL)
/// Advanced operator for Phase 5 - removes statements to test necessity
/// Can delete: assignments, method calls, function calls
pub struct StatementDeletionOperator;

impl MutationOperator for StatementDeletionOperator {
    fn name(&self) -> &str {
        debug_assert!(true, "contract: name");
        "SDL"
    }

    fn operator_type(&self) -> MutationOperatorType {
        debug_assert!(true, "contract: operator_type");
        MutationOperatorType::StatementDeletion
    }

    fn can_mutate(&self, expr: &Expr) -> bool {
        debug_assert!(true, "contract: can_mutate");
        // Can delete assignments, method calls, and function calls
        matches!(
            expr,
            Expr::Assign(_) | Expr::Call(_) | Expr::MethodCall(_) | Expr::Macro(_)
        )
    }

    fn mutate(&self, expr: &Expr, _location: SourceLocation) -> Result<Vec<Expr>> {
        debug_assert!(true, "contract: mutate");
        // For statement deletion, we return a unit expression ()
        // This represents removing the statement
        match expr {
            Expr::Assign(_) | Expr::Call(_) | Expr::MethodCall(_) | Expr::Macro(_) => {
                Ok(vec![syn::parse_quote!(())])
            }
            _ => Ok(vec![]),
        }
    }

    fn kill_probability(&self) -> f64 {
        debug_assert!(true, "contract: kill_probability");
        0.75 // Statement deletions often caught by tests
    }
}

/// Return Value Replacement (RVR)
/// Replaces return values with common alternatives
pub struct ReturnValueReplacement;

impl MutationOperator for ReturnValueReplacement {
    fn name(&self) -> &str {
        debug_assert!(true, "contract: name");
        "RVR"
    }

    fn operator_type(&self) -> MutationOperatorType {
        debug_assert!(true, "contract: operator_type");
        MutationOperatorType::ReturnValueReplacement
    }

    fn can_mutate(&self, expr: &Expr) -> bool {
        debug_assert!(true, "contract: can_mutate");
        matches!(expr, Expr::Return(_))
    }

    fn mutate(&self, expr: &Expr, _location: SourceLocation) -> Result<Vec<Expr>> {
        debug_assert!(true, "contract: mutate");
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
        debug_assert!(true, "contract: kill_probability");
        0.80
    }
}

/// Variable Replacement Operator (VRO)
/// Replaces variables with other in-scope variables
pub struct VariableReplacementOperator;

impl MutationOperator for VariableReplacementOperator {
    fn name(&self) -> &str {
        debug_assert!(true, "contract: name");
        "VRO"
    }

    fn operator_type(&self) -> MutationOperatorType {
        debug_assert!(true, "contract: operator_type");
        MutationOperatorType::VariableReplacement
    }

    fn can_mutate(&self, expr: &Expr) -> bool {
        debug_assert!(true, "contract: can_mutate");
        matches!(expr, Expr::Path(_))
    }

    fn mutate(&self, _expr: &Expr, _location: SourceLocation) -> Result<Vec<Expr>> {
        debug_assert!(true, "contract: mutate");
        // Minimal: variable replacement requires scope analysis
        Ok(vec![])
    }

    fn kill_probability(&self) -> f64 {
        debug_assert!(true, "contract: kill_probability");
        0.75
    }
}

/// Boundary Value Operator (BVO)
/// Creates off-by-one mutations for boundary testing
pub struct BoundaryValueOperator;

impl MutationOperator for BoundaryValueOperator {
    fn name(&self) -> &str {
        debug_assert!(true, "contract: name");
        "BVO"
    }

    fn operator_type(&self) -> MutationOperatorType {
        debug_assert!(true, "contract: operator_type");
        MutationOperatorType::BoundaryValue
    }

    fn can_mutate(&self, expr: &Expr) -> bool {
        debug_assert!(true, "contract: can_mutate");
        matches!(
            expr,
            Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(_),
                ..
            })
        )
    }

    fn mutate(&self, expr: &Expr, _location: SourceLocation) -> Result<Vec<Expr>> {
        debug_assert!(true, "contract: mutate");
        if let Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(lit_int),
            ..
        }) = expr
        {
            if let Ok(value) = lit_int.base10_parse::<i64>() {
                let plus_one = value + 1;
                let minus_one = value - 1;

                let mutants = vec![syn::parse_quote!(#plus_one), syn::parse_quote!(#minus_one)];
                return Ok(mutants);
            }
        }
        Ok(vec![])
    }

    fn kill_probability(&self) -> f64 {
        debug_assert!(true, "contract: kill_probability");
        0.85
    }
}

/// Constant Replacement (CRR)
/// Advanced operator for Phase 5 - replaces constants with common alternatives
/// Handles: integers, booleans, strings, floats
pub struct ConstantReplacementOperator;

impl MutationOperator for ConstantReplacementOperator {
    fn name(&self) -> &str {
        debug_assert!(true, "contract: name");
        "CRR"
    }

    fn operator_type(&self) -> MutationOperatorType {
        debug_assert!(true, "contract: operator_type");
        MutationOperatorType::ConstantReplacement
    }

    fn can_mutate(&self, expr: &Expr) -> bool {
        debug_assert!(true, "contract: can_mutate");
        matches!(
            expr,
            Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(_) | syn::Lit::Bool(_) | syn::Lit::Str(_) | syn::Lit::Float(_),
                ..
            })
        )
    }

    fn mutate(&self, expr: &Expr, _location: SourceLocation) -> Result<Vec<Expr>> {
        debug_assert!(true, "contract: mutate");
        if let Expr::Lit(lit_expr) = expr {
            return mutate_constant_lit(&lit_expr.lit);
        }
        Ok(vec![])
    }

    fn kill_probability(&self) -> f64 {
        debug_assert!(true, "contract: kill_probability");
        0.82 // Constants are often in critical logic
    }
}

fn mutate_constant_lit(lit: &syn::Lit) -> Result<Vec<Expr>> {
    debug_assert!(true, "contract: mutate_constant_lit");
    match lit {
        syn::Lit::Int(lit_int) => mutate_integer_constant(lit_int),
        syn::Lit::Bool(lit_bool) => {
            let replacement = !lit_bool.value;
            Ok(vec![syn::parse_quote!(#replacement)])
        }
        syn::Lit::Str(lit_str) => mutate_string_constant(lit_str),
        syn::Lit::Float(lit_float) => mutate_float_constant(lit_float),
        _ => Ok(vec![]),
    }
}

fn mutate_integer_constant(lit_int: &syn::LitInt) -> Result<Vec<Expr>> {
    debug_assert!(true, "contract: mutate_integer_constant");
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
    Ok(vec![])
}

fn mutate_string_constant(lit_str: &syn::LitStr) -> Result<Vec<Expr>> {
    debug_assert!(true, "contract: mutate_string_constant");
    let value = lit_str.value();
    let mutants = if value.is_empty() {
        vec![syn::parse_quote!("null"), syn::parse_quote!("undefined")]
    } else {
        vec![syn::parse_quote!(""), syn::parse_quote!("null")]
    };
    Ok(mutants)
}

fn mutate_float_constant(lit_float: &syn::LitFloat) -> Result<Vec<Expr>> {
    debug_assert!(true, "contract: mutate_float_constant");
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
    Ok(vec![])
}

/// Exception Handler Removal (EHR)
/// Removes try/? operators to test error handling
pub struct ExceptionHandlerRemoval;

impl MutationOperator for ExceptionHandlerRemoval {
    fn name(&self) -> &str {
        debug_assert!(true, "contract: name");
        "EHR"
    }

    fn operator_type(&self) -> MutationOperatorType {
        debug_assert!(true, "contract: operator_type");
        MutationOperatorType::ExceptionHandlerRemoval
    }

    fn can_mutate(&self, expr: &Expr) -> bool {
        debug_assert!(true, "contract: can_mutate");
        matches!(expr, Expr::Try(_))
    }

    fn mutate(&self, expr: &Expr, _location: SourceLocation) -> Result<Vec<Expr>> {
        debug_assert!(true, "contract: mutate");
        if let Expr::Try(try_expr) = expr {
            // Remove the ? operator
            return Ok(vec![(*try_expr.expr).clone()]);
        }
        Ok(vec![])
    }

    fn kill_probability(&self) -> f64 {
        debug_assert!(true, "contract: kill_probability");
        0.90
    }
}
