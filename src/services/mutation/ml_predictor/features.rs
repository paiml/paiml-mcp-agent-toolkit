#![cfg_attr(coverage_nightly, coverage(off))]
//! Feature extraction implementation for MutantFeatures.

use super::helpers::{count_parameters, count_unique_variables, estimate_nesting_depth};
use super::types::MutantFeatures;
use crate::services::mutation::{Mutant, MutationOperatorType};

impl MutantFeatures {
    /// Extract features from a mutant
    /// Enhanced extraction (v2) - extracts 18 features
    #[allow(clippy::cast_possible_truncation)]
    pub fn from_mutant(mutant: &Mutant) -> Self {
        let source_line = mutant.location.line;
        let source = &mutant.mutated_source;

        // Original 10 features
        let has_loops =
            source.contains("for") || source.contains("while") || source.contains("loop");
        let has_conditionals = source.contains("if") || source.contains("match");

        let control_flow_count = source.matches("if").count() as u32
            + source.matches("for").count() as u32
            + source.matches("while").count() as u32
            + source.matches("match").count() as u32;

        let nesting_depth = estimate_nesting_depth(source);
        let cyclomatic_complexity = 1 + control_flow_count;
        let cognitive_complexity = cyclomatic_complexity + nesting_depth;
        let function_size = source.lines().count() as u32;
        let parameter_count = count_parameters(source);

        // NEW ENHANCED FEATURES (v2) - 8 additional features
        let has_error_handling = source.contains("Result<")
            || source.contains("Option<")
            || source.contains("unwrap")
            || source.contains("expect")
            || source.contains("?")
            || source.contains("try")
            || source.contains("catch");

        let has_assertions = source.contains("assert")
            || source.contains("debug_assert")
            || source.contains("#[test]");

        // Token count (split by whitespace and common delimiters)
        let token_count = source.split_whitespace().count() as u32;

        // Unique variables (simple heuristic: lowercase words)
        let unique_variables = count_unique_variables(source);

        let has_arithmetic = source.contains('+')
            || source.contains('-')
            || source.contains('*')
            || source.contains('/');

        let has_comparisons = source.contains("==")
            || source.contains("!=")
            || source.contains("<=")
            || source.contains(">=")
            || source.contains('<')
            || source.contains('>');

        let has_logical_ops =
            source.contains("&&") || source.contains("||") || source.contains('!');

        // Mutation depth = nesting at mutation point
        let mutation_depth = nesting_depth;

        Self {
            operator_type: mutant.operator.clone(),
            cyclomatic_complexity,
            cognitive_complexity,
            source_line: source_line as u32,
            nesting_depth,
            control_flow_count,
            has_loops,
            has_conditionals,
            function_size,
            parameter_count,
            // New features
            has_error_handling,
            has_assertions,
            token_count,
            unique_variables,
            has_arithmetic,
            has_comparisons,
            has_logical_ops,
            mutation_depth,
        }
    }

    /// Convert features to vector for ML model
    /// Enhanced vector (v2) - 18 features total
    pub fn to_feature_vector(&self) -> Vec<f64> {
        vec![
            // Original 10 features
            self.operator_type_as_numeric(),
            self.cyclomatic_complexity as f64,
            self.cognitive_complexity as f64,
            self.source_line as f64,
            self.nesting_depth as f64,
            self.control_flow_count as f64,
            if self.has_loops { 1.0 } else { 0.0 },
            if self.has_conditionals { 1.0 } else { 0.0 },
            self.function_size as f64,
            self.parameter_count as f64,
            // New 8 features (v2)
            if self.has_error_handling { 1.0 } else { 0.0 },
            if self.has_assertions { 1.0 } else { 0.0 },
            self.token_count as f64,
            self.unique_variables as f64,
            if self.has_arithmetic { 1.0 } else { 0.0 },
            if self.has_comparisons { 1.0 } else { 0.0 },
            if self.has_logical_ops { 1.0 } else { 0.0 },
            self.mutation_depth as f64,
        ]
    }

    fn operator_type_as_numeric(&self) -> f64 {
        match self.operator_type {
            MutationOperatorType::ArithmeticReplacement => 1.0,
            MutationOperatorType::RelationalReplacement => 2.0,
            MutationOperatorType::ConditionalReplacement => 3.0,
            MutationOperatorType::ConstantReplacement => 4.0,
            MutationOperatorType::StatementDeletion => 5.0,
            MutationOperatorType::ReturnReplacement => 6.0,
            MutationOperatorType::VariableReplacement => 7.0,
            MutationOperatorType::ConditionalReturn => 8.0,
            MutationOperatorType::BoundaryValue => 9.0,
            MutationOperatorType::ExceptionHandlerRemoval => 10.0,
            MutationOperatorType::ReturnValueReplacement => 11.0,
            MutationOperatorType::UnaryReplacement => 12.0,
            MutationOperatorType::BitwiseReplacement => 13.0,
            MutationOperatorType::AssignmentReplacement => 14.0,
            MutationOperatorType::PointerReplacement => 15.0,
            MutationOperatorType::MemberAccessReplacement => 16.0,
            MutationOperatorType::RangeReplacement => 17.0,
            MutationOperatorType::PatternReplacement => 18.0,
            MutationOperatorType::MethodChainReplacement => 19.0,
            MutationOperatorType::BorrowReplacement => 20.0,
            MutationOperatorType::Custom(_) => 21.0,
            MutationOperatorType::None => 0.0,
        }
    }
}
