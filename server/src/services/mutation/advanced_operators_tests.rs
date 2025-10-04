//! EXTREME TDD: Advanced mutation operators tests
//!
//! RED PHASE: All tests written BEFORE implementation

#[cfg(test)]
mod advanced_operators_red_tests {
    use crate::services::mutation::{
        MutationOperator, MutationOperatorType, SourceLocation,
        ConditionalReturnOperator, StatementDeletionOperator,
        ReturnValueReplacement, VariableReplacementOperator,
        BoundaryValueOperator, ExceptionHandlerRemoval,
    };

    // ===== CRO: Conditional Return Operator (RED) =====

    #[test]
    fn red_cro_must_have_correct_name() {
        let operator = ConditionalReturnOperator;
        assert_eq!(operator.name(), "CRO");
    }

    #[test]
    fn red_cro_must_have_correct_type() {
        let operator = ConditionalReturnOperator;
        assert_eq!(operator.operator_type(), MutationOperatorType::ConditionalReturn);
    }

    #[test]
    fn red_cro_must_detect_return_statements() {
        let operator = ConditionalReturnOperator;
        let source = "return 42";
        let expr = syn::parse_str::<syn::Expr>(source).unwrap();

        assert!(operator.can_mutate(&expr), "Must detect return statements");
    }

    #[test]
    fn red_cro_must_generate_early_return_mutants() {
        let operator = ConditionalReturnOperator;
        let source = "return 42";
        let expr = syn::parse_str::<syn::Expr>(source).unwrap();
        let location = SourceLocation { line: 1, column: 0, end_line: 1, end_column: 9 };

        let mutants = operator.mutate(&expr, location).unwrap();
        assert!(!mutants.is_empty(), "Must generate early return mutants");
    }

    // ===== SDO: Statement Deletion Operator (RED) =====

    #[test]
    fn red_sdo_must_have_correct_name() {
        let operator = StatementDeletionOperator;
        assert_eq!(operator.name(), "SDL"); // Phase 5: Changed from SDO to SDL per spec
    }

    #[test]
    fn red_sdo_must_have_correct_type() {
        let operator = StatementDeletionOperator;
        assert_eq!(operator.operator_type(), MutationOperatorType::StatementDeletion);
    }

    #[test]
    fn red_sdo_must_detect_deletable_statements() {
        let operator = StatementDeletionOperator;
        // Assignment statement
        let source = "x = 5";
        let expr = syn::parse_str::<syn::Expr>(source).unwrap();

        assert!(operator.can_mutate(&expr), "Must detect deletable statements");
    }

    // ===== RVR: Return Value Replacement (RED) =====

    #[test]
    fn red_rvr_must_have_correct_name() {
        let operator = ReturnValueReplacement;
        assert_eq!(operator.name(), "RVR");
    }

    #[test]
    fn red_rvr_must_have_correct_type() {
        let operator = ReturnValueReplacement;
        assert_eq!(operator.operator_type(), MutationOperatorType::ReturnValueReplacement);
    }

    #[test]
    fn red_rvr_must_detect_return_values() {
        let operator = ReturnValueReplacement;
        let source = "return 42";
        let expr = syn::parse_str::<syn::Expr>(source).unwrap();

        assert!(operator.can_mutate(&expr), "Must detect return values");
    }

    #[test]
    fn red_rvr_must_generate_return_value_mutants() {
        let operator = ReturnValueReplacement;
        let source = "return 42";
        let expr = syn::parse_str::<syn::Expr>(source).unwrap();
        let location = SourceLocation { line: 1, column: 0, end_line: 1, end_column: 9 };

        let mutants = operator.mutate(&expr, location).unwrap();
        assert!(!mutants.is_empty(), "Must generate return value mutants");
    }

    // ===== VRO: Variable Replacement Operator (RED) =====

    #[test]
    fn red_vro_must_have_correct_name() {
        let operator = VariableReplacementOperator;
        assert_eq!(operator.name(), "VRO");
    }

    #[test]
    fn red_vro_must_have_correct_type() {
        let operator = VariableReplacementOperator;
        assert_eq!(operator.operator_type(), MutationOperatorType::VariableReplacement);
    }

    #[test]
    fn red_vro_must_detect_variables() {
        let operator = VariableReplacementOperator;
        let source = "x";
        let expr = syn::parse_str::<syn::Expr>(source).unwrap();

        assert!(operator.can_mutate(&expr), "Must detect variables");
    }

    // ===== BVO: Boundary Value Operator (RED) =====

    #[test]
    fn red_bvo_must_have_correct_name() {
        let operator = BoundaryValueOperator;
        assert_eq!(operator.name(), "BVO");
    }

    #[test]
    fn red_bvo_must_have_correct_type() {
        let operator = BoundaryValueOperator;
        assert_eq!(operator.operator_type(), MutationOperatorType::BoundaryValue);
    }

    #[test]
    fn red_bvo_must_detect_numeric_literals() {
        let operator = BoundaryValueOperator;
        let source = "42";
        let expr = syn::parse_str::<syn::Expr>(source).unwrap();

        assert!(operator.can_mutate(&expr), "Must detect numeric literals");
    }

    #[test]
    fn red_bvo_must_generate_off_by_one_mutants() {
        let operator = BoundaryValueOperator;
        let source = "42";
        let expr = syn::parse_str::<syn::Expr>(source).unwrap();
        let location = SourceLocation { line: 1, column: 0, end_line: 1, end_column: 2 };

        let mutants = operator.mutate(&expr, location).unwrap();
        assert!(mutants.len() >= 2, "Must generate +1 and -1 mutants");
    }

    // ===== EHR: Exception Handler Removal (RED) =====

    #[test]
    fn red_ehr_must_have_correct_name() {
        let operator = ExceptionHandlerRemoval;
        assert_eq!(operator.name(), "EHR");
    }

    #[test]
    fn red_ehr_must_have_correct_type() {
        let operator = ExceptionHandlerRemoval;
        assert_eq!(operator.operator_type(), MutationOperatorType::ExceptionHandlerRemoval);
    }

    #[test]
    fn red_ehr_must_detect_try_expressions() {
        let operator = ExceptionHandlerRemoval;
        let source = "result?";
        let expr = syn::parse_str::<syn::Expr>(source).unwrap();

        assert!(operator.can_mutate(&expr), "Must detect try expressions");
    }

    #[test]
    fn red_ehr_must_generate_try_removal_mutants() {
        let operator = ExceptionHandlerRemoval;
        let source = "result?";
        let expr = syn::parse_str::<syn::Expr>(source).unwrap();
        let location = SourceLocation { line: 1, column: 0, end_line: 1, end_column: 7 };

        let mutants = operator.mutate(&expr, location).unwrap();
        assert!(!mutants.is_empty(), "Must generate try removal mutants");
    }
}
