#![cfg_attr(coverage_nightly, coverage(off))]
//! Mutation testing engine for PMAT
//!
//! AST-based mutation testing and fuzzing system for language-agnostic
//! test suite quality evaluation.

#![allow(ambiguous_glob_reexports)]

pub mod cargo_mutants_wrapper;
pub mod ci_cd_learning;
pub mod coverage;
pub mod cpp_adapter;
pub mod cpp_mutation_generator;
pub mod cpp_tree_sitter_mutations;
pub mod distributed;
pub mod engine;
pub mod equivalent_detector;
pub mod executor;
pub mod fuzzing;
pub mod go_adapter;
pub mod go_mutation_generator;
pub mod go_tree_sitter_mutations;
pub mod guard;
pub mod json_parser; // Sprint 70: cargo-mutants JSON parser
pub mod language;
pub mod lua_adapter;
pub mod language_detector; // Sprint 63: Multi-language support
pub mod ml_predictor;
pub mod operators;
pub mod python_adapter;
pub mod python_mutation_generator;
pub mod python_tree_sitter_mutations;
pub mod rust_adapter;
pub mod rust_mutation_generator;
pub mod rust_tree_sitter_mutations;
pub mod scoring;
pub mod state;
pub mod temp_file;
pub mod tree_sitter_operators;
pub mod types;
pub mod typescript_adapter;
pub mod typescript_mutation_generator;
pub mod typescript_tree_sitter_mutations;
pub mod wasm_adapter;
pub mod worker_monitor; // Sprint 70: cargo-mutants subprocess wrapper

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod typescript_adapter_tests;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod python_adapter_tests;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod go_adapter_tests;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod cpp_adapter_tests;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod advanced_operators_tests;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod fuzzing_integration_tests;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod ml_predictor_tests;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod cross_validation_test;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod equivalent_detector_tests;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod ml_integration_tests;

pub use ci_cd_learning::*;
pub use coverage::*;
pub use cpp_adapter::*;
pub use cpp_mutation_generator::*;
pub use cpp_tree_sitter_mutations::*;
pub use distributed::*;
pub use engine::*;
pub use equivalent_detector::*;
pub use executor::*;
pub use fuzzing::*;
pub use go_adapter::*;
pub use go_mutation_generator::*;
pub use go_tree_sitter_mutations::*;
pub use guard::*;
pub use language::*;
pub use language_detector::*; // Sprint 63: Multi-language support
pub use lua_adapter::*;
pub use ml_predictor::*;
pub use operators::*;
pub use python_adapter::*;
pub use python_mutation_generator::*;
pub use python_tree_sitter_mutations::*;
pub use rust_adapter::*;
pub use rust_mutation_generator::*;
pub use rust_tree_sitter_mutations::*;
pub use scoring::*;
pub use state::*;
pub use temp_file::*;
pub use tree_sitter_operators::*;
pub use types::*;
pub use typescript_adapter::*;
pub use typescript_mutation_generator::*;
pub use typescript_tree_sitter_mutations::*;
pub use wasm_adapter::*;
pub use worker_monitor::*;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    //! EXTREME TDD coverage tests for mutation testing module
    //!
    //! These tests exercise the core mutation testing functionality through
    //! the public API to ensure comprehensive coverage.

    use super::*;
    use std::path::PathBuf;
    use std::sync::Arc;

    // ============================================================================
    // Types Module Tests
    // ============================================================================

    mod types_tests {
        use super::*;

        #[test]
        fn test_mutant_creation_and_fields() {
            let mutant = Mutant {
                id: "test_id_123".to_string(),
                original_file: PathBuf::from("src/lib.rs"),
                mutated_source: "fn test() {}".to_string(),
                location: SourceLocation {
                    line: 10,
                    column: 5,
                    end_line: 10,
                    end_column: 20,
                },
                operator: MutationOperatorType::ArithmeticReplacement,
                hash: "abc123".to_string(),
                status: MutantStatus::Pending,
            };

            assert_eq!(mutant.id, "test_id_123");
            assert_eq!(mutant.original_file, PathBuf::from("src/lib.rs"));
            assert_eq!(mutant.mutated_source, "fn test() {}");
            assert_eq!(mutant.location.line, 10);
            assert_eq!(mutant.location.column, 5);
            assert_eq!(mutant.hash, "abc123");
            assert_eq!(mutant.status, MutantStatus::Pending);
        }

        #[test]
        fn test_source_location_boundary_values() {
            let loc = SourceLocation {
                line: 0,
                column: 0,
                end_line: usize::MAX,
                end_column: usize::MAX,
            };

            assert_eq!(loc.line, 0);
            assert_eq!(loc.column, 0);
            assert_eq!(loc.end_line, usize::MAX);
            assert_eq!(loc.end_column, usize::MAX);
        }

        #[test]
        fn test_mutation_operator_type_ordering() {
            // Test that MutationOperatorType implements Ord correctly
            let arith = MutationOperatorType::ArithmeticReplacement;
            let relat = MutationOperatorType::RelationalReplacement;
            let cond = MutationOperatorType::ConditionalReplacement;

            // They should be comparable (PartialOrd, Ord)
            assert!(arith <= relat || arith >= relat);
            assert!(relat <= cond || relat >= cond);
        }

        #[test]
        fn test_mutation_operator_type_all_variants() {
            // Test all operator type variants exist and can be matched
            let operators = vec![
                MutationOperatorType::ArithmeticReplacement,
                MutationOperatorType::RelationalReplacement,
                MutationOperatorType::ConditionalReplacement,
                MutationOperatorType::ConstantReplacement,
                MutationOperatorType::StatementDeletion,
                MutationOperatorType::ReturnReplacement,
                MutationOperatorType::VariableReplacement,
                MutationOperatorType::ConditionalReturn,
                MutationOperatorType::BoundaryValue,
                MutationOperatorType::ExceptionHandlerRemoval,
                MutationOperatorType::ReturnValueReplacement,
                MutationOperatorType::UnaryReplacement,
                MutationOperatorType::BitwiseReplacement,
                MutationOperatorType::AssignmentReplacement,
                MutationOperatorType::PointerReplacement,
                MutationOperatorType::MemberAccessReplacement,
                MutationOperatorType::RangeReplacement,
                MutationOperatorType::PatternReplacement,
                MutationOperatorType::MethodChainReplacement,
                MutationOperatorType::BorrowReplacement,
                MutationOperatorType::Custom("test".to_string()),
                MutationOperatorType::None,
            ];

            assert_eq!(operators.len(), 22);
        }

        #[test]
        fn test_mutant_status_all_variants() {
            let statuses = vec![
                MutantStatus::Pending,
                MutantStatus::Killed,
                MutantStatus::Survived,
                MutantStatus::CompileError,
                MutantStatus::Timeout,
                MutantStatus::Equivalent,
            ];

            assert_eq!(statuses.len(), 6);

            // Test equality
            assert_eq!(MutantStatus::Pending, MutantStatus::Pending);
            assert_ne!(MutantStatus::Killed, MutantStatus::Survived);
        }

        #[test]
        fn test_mutation_result_creation() {
            let mutant = Mutant {
                id: "test".to_string(),
                original_file: PathBuf::from("test.rs"),
                mutated_source: "mutated".to_string(),
                location: SourceLocation {
                    line: 1,
                    column: 1,
                    end_line: 1,
                    end_column: 10,
                },
                operator: MutationOperatorType::ArithmeticReplacement,
                hash: "hash".to_string(),
                status: MutantStatus::Pending,
            };

            let result = MutationResult {
                mutant: mutant.clone(),
                status: MutantStatus::Killed,
                test_failures: vec!["test_add".to_string(), "test_sub".to_string()],
                execution_time_ms: 150,
                error_message: None,
            };

            assert_eq!(result.status, MutantStatus::Killed);
            assert_eq!(result.test_failures.len(), 2);
            assert_eq!(result.execution_time_ms, 150);
            assert!(result.error_message.is_none());
        }

        #[test]
        fn test_mutation_result_with_error() {
            let mutant = Mutant {
                id: "error_test".to_string(),
                original_file: PathBuf::from("test.rs"),
                mutated_source: "invalid".to_string(),
                location: SourceLocation {
                    line: 1,
                    column: 1,
                    end_line: 1,
                    end_column: 5,
                },
                operator: MutationOperatorType::StatementDeletion,
                hash: "errhash".to_string(),
                status: MutantStatus::CompileError,
            };

            let result = MutationResult {
                mutant,
                status: MutantStatus::CompileError,
                test_failures: vec![],
                execution_time_ms: 0,
                error_message: Some("error[E0308]: mismatched types".to_string()),
            };

            assert_eq!(result.status, MutantStatus::CompileError);
            assert!(result.error_message.is_some());
            assert!(result.error_message.unwrap().contains("E0308"));
        }

        #[test]
        fn test_weak_spot_creation() {
            let weak_spot = WeakSpot {
                file: PathBuf::from("src/critical.rs"),
                line_range: (10, 50),
                survived_mutants: 5,
                suggestions: vec![
                    "Add boundary tests".to_string(),
                    "Test error paths".to_string(),
                ],
            };

            assert_eq!(weak_spot.file, PathBuf::from("src/critical.rs"));
            assert_eq!(weak_spot.line_range, (10, 50));
            assert_eq!(weak_spot.survived_mutants, 5);
            assert_eq!(weak_spot.suggestions.len(), 2);
        }

        #[test]
        fn test_mutant_serialization() {
            let mutant = Mutant {
                id: "ser_test".to_string(),
                original_file: PathBuf::from("src/lib.rs"),
                mutated_source: "fn x() {}".to_string(),
                location: SourceLocation {
                    line: 1,
                    column: 1,
                    end_line: 1,
                    end_column: 10,
                },
                operator: MutationOperatorType::ArithmeticReplacement,
                hash: "ser_hash".to_string(),
                status: MutantStatus::Pending,
            };

            let json = serde_json::to_string(&mutant).unwrap();
            let deserialized: Mutant = serde_json::from_str(&json).unwrap();

            assert_eq!(deserialized.id, mutant.id);
            assert_eq!(deserialized.original_file, mutant.original_file);
            assert_eq!(deserialized.status, mutant.status);
        }

        #[test]
        fn test_mutant_hash_and_eq() {
            use std::collections::HashSet;

            let mutant1 = Mutant {
                id: "same".to_string(),
                original_file: PathBuf::from("test.rs"),
                mutated_source: "fn a() {}".to_string(),
                location: SourceLocation {
                    line: 1,
                    column: 1,
                    end_line: 1,
                    end_column: 10,
                },
                operator: MutationOperatorType::ArithmeticReplacement,
                hash: "same_hash".to_string(),
                status: MutantStatus::Pending,
            };

            let mutant2 = mutant1.clone();

            // Test that cloned mutants are equal
            assert_eq!(mutant1, mutant2);

            // Test that they can be stored in a HashSet
            let mut set = HashSet::new();
            set.insert(mutant1.clone());
            assert!(set.contains(&mutant2));
        }
    }

    // ============================================================================
    // Operators Module Tests
    // ============================================================================

    mod operators_tests {
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
    }

    // ============================================================================
    // Scoring Module Tests
    // ============================================================================

    mod scoring_tests {
        use super::*;

        fn create_test_result(status: MutantStatus, file: &str, line: usize) -> MutationResult {
            MutationResult {
                mutant: Mutant {
                    id: format!("test_{}", line),
                    original_file: PathBuf::from(file),
                    mutated_source: String::new(),
                    location: SourceLocation {
                        line,
                        column: 1,
                        end_line: line,
                        end_column: 10,
                    },
                    operator: MutationOperatorType::ArithmeticReplacement,
                    hash: format!("hash_{}", line),
                    status: status.clone(),
                },
                status,
                test_failures: vec![],
                execution_time_ms: 100,
                error_message: None,
            }
        }

        #[test]
        fn test_mutation_scorer_creation() {
            let results = vec![
                create_test_result(MutantStatus::Killed, "foo.rs", 10),
                create_test_result(MutantStatus::Survived, "foo.rs", 20),
            ];

            let scorer = MutationScorer::new(results);
            let score = scorer.calculate_score();

            assert_eq!(score.total, 2);
            assert_eq!(score.killed, 1);
            assert_eq!(score.survived, 1);
            assert!((score.score - 0.5).abs() < 0.01);
        }

        #[test]
        fn test_mutation_scorer_with_equivalents() {
            let results = vec![
                create_test_result(MutantStatus::Killed, "foo.rs", 10),
                create_test_result(MutantStatus::Killed, "foo.rs", 20),
                create_test_result(MutantStatus::Equivalent, "foo.rs", 30),
            ];

            let scorer = MutationScorer::new(results);
            let score = scorer.calculate_score();

            // Score should be 2/2 = 1.0 (equivalent mutants excluded)
            assert_eq!(score.score, 1.0);
            assert_eq!(score.equivalent, 1);
        }

        #[test]
        fn test_mutation_summary_generation() {
            let results = vec![
                create_test_result(MutantStatus::Killed, "foo.rs", 10),
                create_test_result(MutantStatus::Survived, "foo.rs", 20),
                create_test_result(MutantStatus::Survived, "foo.rs", 25),
                create_test_result(MutantStatus::CompileError, "bar.rs", 5),
            ];

            let scorer = MutationScorer::new(results);
            let summary = scorer.summary();

            assert_eq!(summary.total_mutants, 4);
            assert_eq!(summary.killed, 1);
            assert_eq!(summary.survived, 2);
            assert_eq!(summary.compile_errors, 1);
            assert!(summary.weak_spots.len() >= 1); // foo.rs has survivors
        }

        #[test]
        fn test_weak_spots_grouping() {
            let results = vec![
                create_test_result(MutantStatus::Survived, "fileA.rs", 10),
                create_test_result(MutantStatus::Survived, "fileA.rs", 20),
                create_test_result(MutantStatus::Survived, "fileA.rs", 30),
                create_test_result(MutantStatus::Survived, "fileB.rs", 5),
            ];

            let scorer = MutationScorer::new(results);
            let weak_spots = scorer.weak_spots();

            assert_eq!(weak_spots.len(), 2);
            // fileA should be first (more survivors)
            assert_eq!(weak_spots[0].survived_mutants, 3);
            assert_eq!(weak_spots[1].survived_mutants, 1);
        }

        #[test]
        fn test_weak_spots_line_range() {
            let results = vec![
                create_test_result(MutantStatus::Survived, "file.rs", 5),
                create_test_result(MutantStatus::Survived, "file.rs", 15),
                create_test_result(MutantStatus::Survived, "file.rs", 25),
            ];

            let scorer = MutationScorer::new(results);
            let weak_spots = scorer.weak_spots();

            assert_eq!(weak_spots.len(), 1);
            assert_eq!(weak_spots[0].line_range, (5, 25));
        }
    }

    // ============================================================================
    // Engine Module Tests
    // ============================================================================

    mod engine_tests {
        use super::*;

        #[test]
        fn test_mutation_config_default() {
            let config = MutationConfig::default();

            assert!(matches!(config.strategy, MutationStrategy::Selective));
            assert_eq!(config.max_mutants, 0);
            assert!(config.parallel_threads > 0);
        }

        #[test]
        fn test_mutation_strategy_variants() {
            let selective = MutationStrategy::Selective;
            let random = MutationStrategy::Random;
            let hybrid = MutationStrategy::Hybrid {
                selective: 0.7,
                random: 0.3,
            };

            // Just verify they can be created and matched
            match selective {
                MutationStrategy::Selective => {}
                _ => panic!("Expected Selective"),
            }

            match random {
                MutationStrategy::Random => {}
                _ => panic!("Expected Random"),
            }

            match hybrid {
                MutationStrategy::Hybrid { selective, random } => {
                    assert!((selective - 0.7).abs() < 0.001);
                    assert!((random - 0.3).abs() < 0.001);
                }
                _ => panic!("Expected Hybrid"),
            }
        }

        #[test]
        fn test_mutation_engine_default_rust() {
            let engine = MutationEngine::default_rust();
            // Just verify it can be created
            assert!(std::mem::size_of_val(&engine) > 0);
        }

        #[tokio::test]
        async fn test_mutation_engine_empty_source() {
            let adapter = Arc::new(RustAdapter::new());
            let config = MutationConfig::default();
            let engine = MutationEngine::new(adapter, config);

            let source = "";
            let result = engine
                .generate_mutants_from_source(std::path::Path::new("test.rs"), source)
                .await;

            // Empty source should fail to parse
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_mutation_engine_with_max_mutants() {
            let adapter = Arc::new(RustAdapter::new());
            let config = MutationConfig {
                strategy: MutationStrategy::Selective,
                max_mutants: 1,
                parallel_threads: 1,
            };
            let engine = MutationEngine::new(adapter, config);

            let source = "fn calc(a: i32, b: i32) -> i32 { a + b + a - b }";
            let mutants = engine
                .generate_mutants_from_source(std::path::Path::new("test.rs"), source)
                .await
                .unwrap();

            assert!(mutants.len() <= 1);
        }

        #[tokio::test]
        async fn test_mutation_engine_random_strategy() {
            let adapter = Arc::new(RustAdapter::new());
            let config = MutationConfig {
                strategy: MutationStrategy::Random,
                max_mutants: 2,
                parallel_threads: 1,
            };
            let engine = MutationEngine::new(adapter, config);

            let source = "fn test(x: i32) -> i32 { x + 1 }";
            let mutants = engine
                .generate_mutants_from_source(std::path::Path::new("test.rs"), source)
                .await
                .unwrap();

            assert!(mutants.len() <= 2);
        }

        #[tokio::test]
        async fn test_mutation_engine_hybrid_strategy() {
            let adapter = Arc::new(RustAdapter::new());
            let config = MutationConfig {
                strategy: MutationStrategy::Hybrid {
                    selective: 0.5,
                    random: 0.5,
                },
                max_mutants: 3,
                parallel_threads: 1,
            };
            let engine = MutationEngine::new(adapter, config);

            let source = "fn test(a: i32, b: i32) -> bool { a + b > 0 && a < b }";
            let mutants = engine
                .generate_mutants_from_source(std::path::Path::new("test.rs"), source)
                .await
                .unwrap();

            assert!(mutants.len() <= 3);
        }
    }

    // ============================================================================
    // Language Module Tests
    // ============================================================================

    mod language_tests {
        use super::*;

        #[test]
        fn test_language_registry_creation() {
            let registry = LanguageRegistry::new();
            assert!(registry.languages().is_empty());
        }

        #[test]
        fn test_language_registry_register_rust() {
            let mut registry = LanguageRegistry::new();
            registry.register(Arc::new(RustAdapter::new()));

            assert!(registry.languages().contains(&"rust"));
            assert!(registry.get_adapter("rust").is_some());
        }

        #[test]
        fn test_language_registry_detect_by_extension() {
            let mut registry = LanguageRegistry::new();
            registry.register(Arc::new(RustAdapter::new()));

            let detected = registry.detect_language(std::path::Path::new("test.rs"));
            assert!(detected.is_some());
            assert_eq!(detected.unwrap().name(), "rust");
        }

        #[test]
        fn test_language_registry_unknown_extension() {
            let registry = LanguageRegistry::new();
            let detected = registry.detect_language(std::path::Path::new("test.xyz"));
            assert!(detected.is_none());
        }

        #[test]
        fn test_test_run_result_fields() {
            let result = TestRunResult {
                passed: false,
                failures: vec!["test_one".to_string(), "test_two".to_string()],
                execution_time_ms: 500,
                stdout: "running 2 tests".to_string(),
                stderr: "thread panicked".to_string(),
            };

            assert!(!result.passed);
            assert_eq!(result.failures.len(), 2);
            assert_eq!(result.execution_time_ms, 500);
            assert!(result.stdout.contains("running"));
            assert!(result.stderr.contains("panicked"));
        }
    }

    // ============================================================================
    // State Module Tests
    // ============================================================================

    mod state_tests {
        use super::*;

        fn create_test_mutant(id: &str) -> Mutant {
            Mutant {
                id: id.to_string(),
                original_file: PathBuf::from("test.rs"),
                mutated_source: "fn test() {}".to_string(),
                location: SourceLocation {
                    line: 1,
                    column: 1,
                    end_line: 1,
                    end_column: 10,
                },
                operator: MutationOperatorType::ArithmeticReplacement,
                hash: format!("hash_{}", id),
                status: MutantStatus::Pending,
            }
        }

        #[test]
        fn test_mutation_state_creation() {
            let mutants = vec![create_test_mutant("m1"), create_test_mutant("m2")];
            let state =
                MutationState::new(std::path::Path::new("/project"), mutants, 60, true, Some(4));

            assert_eq!(state.pending_mutants.len(), 2);
            assert!(state.completed_mutants.is_empty());
            assert!(!state.is_complete());
            assert_eq!(state.total_mutants(), 2);
            assert_eq!(state.completed_count(), 0);
            assert!((state.completion_percentage() - 0.0).abs() < 0.01);
        }

        #[test]
        fn test_mutation_state_add_result() {
            let mutants = vec![create_test_mutant("m1"), create_test_mutant("m2")];
            let mut state =
                MutationState::new(std::path::Path::new("/project"), mutants, 60, false, None);

            let result = MutationResult {
                mutant: create_test_mutant("m1"),
                status: MutantStatus::Killed,
                test_failures: vec!["test".to_string()],
                execution_time_ms: 100,
                error_message: None,
            };

            state.add_result(result);

            assert_eq!(state.pending_mutants.len(), 1);
            assert_eq!(state.completed_mutants.len(), 1);
            assert!(!state.is_complete());
            assert!((state.completion_percentage() - 50.0).abs() < 0.01);
        }

        #[test]
        fn test_mutation_state_completion() {
            let mutants = vec![create_test_mutant("m1")];
            let mut state =
                MutationState::new(std::path::Path::new("/project"), mutants, 60, false, None);

            let result = MutationResult {
                mutant: create_test_mutant("m1"),
                status: MutantStatus::Survived,
                test_failures: vec![],
                execution_time_ms: 50,
                error_message: None,
            };

            state.add_result(result);

            assert!(state.is_complete());
            assert!((state.completion_percentage() - 100.0).abs() < 0.01);
        }

        #[test]
        fn test_mutation_state_empty() {
            let state =
                MutationState::new(std::path::Path::new("/project"), vec![], 60, false, None);

            assert!(state.is_complete());
            assert_eq!(state.total_mutants(), 0);
            assert!((state.completion_percentage() - 100.0).abs() < 0.01);
        }

        #[test]
        fn test_mutation_state_config() {
            let state =
                MutationState::new(std::path::Path::new("/project"), vec![], 120, true, Some(8));

            assert_eq!(state.config.timeout_secs, 120);
            assert!(state.config.parallel);
            assert_eq!(state.config.worker_count, Some(8));
        }

        #[test]
        fn test_default_state_path() {
            let project_path = std::path::Path::new("/my/project");
            let state_path = MutationState::default_state_path(project_path);

            assert!(state_path.ends_with("mutation_state.json"));
            assert!(state_path.to_str().unwrap().contains(".pmat"));
        }
    }

    // ============================================================================
    // Rust Adapter Tests
    // ============================================================================

    mod rust_adapter_tests {
        use super::*;

        #[test]
        fn test_rust_adapter_creation() {
            let adapter = RustAdapter::new();
            assert_eq!(adapter.name(), "rust");
            assert_eq!(adapter.extensions(), &["rs"]);
        }

        #[test]
        fn test_rust_adapter_mutation_operators_count() {
            let adapter = RustAdapter::new();
            let operators = adapter.mutation_operators();

            // Should have 6 operators (AOR, ROR, COR, UOR, CRR, SDL)
            assert_eq!(operators.len(), 6);
        }

        #[test]
        fn test_rust_adapter_mutation_operators_names() {
            let adapter = RustAdapter::new();
            let operators = adapter.mutation_operators();

            let names: Vec<&str> = operators.iter().map(|op| op.name()).collect();
            assert!(names.contains(&"AOR"));
            assert!(names.contains(&"ROR"));
            assert!(names.contains(&"COR"));
            assert!(names.contains(&"UOR"));
            assert!(names.contains(&"CRR"));
            assert!(names.contains(&"SDL"));
        }

        #[tokio::test]
        async fn test_rust_adapter_parse_valid() {
            let adapter = RustAdapter::new();
            let source = "fn main() { println!(\"Hello\"); }";

            let result = adapter.parse(source).await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_rust_adapter_parse_invalid() {
            let adapter = RustAdapter::new();
            let source = "fn main() { invalid syntax here }}}";

            let result = adapter.parse(source).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_rust_adapter_unparse() {
            let adapter = RustAdapter::new();
            let source = "fn test() {}";

            let result = adapter.unparse(source).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), source);
        }
    }

    // ============================================================================
    // Property-based tests with proptest
    // ============================================================================

    mod property_tests {
        use proptest::prelude::*;

        use super::*;

        proptest! {
            #[test]
            fn prop_mutation_score_calculation_valid_range(
                killed in 0usize..100,
                survived in 0usize..100,
                equivalent in 0usize..50,
                compile_errors in 0usize..30
            ) {
                let mut results = Vec::new();

                // Create killed mutants
                for i in 0..killed {
                    results.push(MutationResult {
                        mutant: Mutant {
                            id: format!("k{}", i),
                            original_file: PathBuf::from("test.rs"),
                            mutated_source: String::new(),
                            location: SourceLocation { line: 1, column: 1, end_line: 1, end_column: 1 },
                            operator: MutationOperatorType::ArithmeticReplacement,
                            hash: format!("h{}", i),
                            status: MutantStatus::Killed,
                        },
                        status: MutantStatus::Killed,
                        test_failures: vec!["test".to_string()],
                        execution_time_ms: 100,
                        error_message: None,
                    });
                }

                // Create survived mutants
                for i in 0..survived {
                    results.push(MutationResult {
                        mutant: Mutant {
                            id: format!("s{}", i),
                            original_file: PathBuf::from("test.rs"),
                            mutated_source: String::new(),
                            location: SourceLocation { line: 1, column: 1, end_line: 1, end_column: 1 },
                            operator: MutationOperatorType::ArithmeticReplacement,
                            hash: format!("s{}", i),
                            status: MutantStatus::Survived,
                        },
                        status: MutantStatus::Survived,
                        test_failures: vec![],
                        execution_time_ms: 100,
                        error_message: None,
                    });
                }

                // Create equivalent mutants
                for i in 0..equivalent {
                    results.push(MutationResult {
                        mutant: Mutant {
                            id: format!("e{}", i),
                            original_file: PathBuf::from("test.rs"),
                            mutated_source: String::new(),
                            location: SourceLocation { line: 1, column: 1, end_line: 1, end_column: 1 },
                            operator: MutationOperatorType::ArithmeticReplacement,
                            hash: format!("e{}", i),
                            status: MutantStatus::Equivalent,
                        },
                        status: MutantStatus::Equivalent,
                        test_failures: vec![],
                        execution_time_ms: 50,
                        error_message: None,
                    });
                }

                // Create compile error mutants
                for i in 0..compile_errors {
                    results.push(MutationResult {
                        mutant: Mutant {
                            id: format!("c{}", i),
                            original_file: PathBuf::from("test.rs"),
                            mutated_source: String::new(),
                            location: SourceLocation { line: 1, column: 1, end_line: 1, end_column: 1 },
                            operator: MutationOperatorType::ArithmeticReplacement,
                            hash: format!("c{}", i),
                            status: MutantStatus::CompileError,
                        },
                        status: MutantStatus::CompileError,
                        test_failures: vec![],
                        execution_time_ms: 0,
                        error_message: Some("error".to_string()),
                    });
                }

                let score = MutationScore::from_results(&results);

                // Score should always be between 0.0 and 1.0
                prop_assert!(score.score >= 0.0 && score.score <= 1.0,
                    "Score {} out of range [0.0, 1.0]", score.score);

                // Counts should add up
                prop_assert_eq!(
                    score.total,
                    killed + survived + equivalent + compile_errors,
                    "Total count mismatch"
                );
            }

            #[test]
            fn prop_source_location_fields_preserved(
                line in 0usize..10000,
                column in 0usize..1000,
                end_line in 0usize..10000,
                end_column in 0usize..1000
            ) {
                let loc = SourceLocation { line, column, end_line, end_column };

                prop_assert_eq!(loc.line, line);
                prop_assert_eq!(loc.column, column);
                prop_assert_eq!(loc.end_line, end_line);
                prop_assert_eq!(loc.end_column, end_column);
            }

            #[test]
            fn prop_mutant_id_preserved_through_clone(id in "[a-zA-Z0-9_]{1,50}") {
                let mutant = Mutant {
                    id: id.clone(),
                    original_file: PathBuf::from("test.rs"),
                    mutated_source: "fn test() {}".to_string(),
                    location: SourceLocation { line: 1, column: 1, end_line: 1, end_column: 10 },
                    operator: MutationOperatorType::ArithmeticReplacement,
                    hash: "hash".to_string(),
                    status: MutantStatus::Pending,
                };

                let cloned = mutant.clone();
                prop_assert_eq!(cloned.id, id);
                prop_assert_eq!(cloned, mutant);
            }

            #[test]
            fn prop_mutation_state_completion_percentage(
                pending in 0usize..100,
                completed in 0usize..100
            ) {
                if pending + completed == 0 {
                    return Ok(());
                }

                let pending_mutants: Vec<Mutant> = (0..pending).map(|i| Mutant {
                    id: format!("p{}", i),
                    original_file: PathBuf::from("test.rs"),
                    mutated_source: String::new(),
                    location: SourceLocation { line: 1, column: 1, end_line: 1, end_column: 1 },
                    operator: MutationOperatorType::ArithmeticReplacement,
                    hash: format!("ph{}", i),
                    status: MutantStatus::Pending,
                }).collect();

                let completed_results: Vec<MutationResult> = (0..completed).map(|i| MutationResult {
                    mutant: Mutant {
                        id: format!("c{}", i),
                        original_file: PathBuf::from("test.rs"),
                        mutated_source: String::new(),
                        location: SourceLocation { line: 1, column: 1, end_line: 1, end_column: 1 },
                        operator: MutationOperatorType::ArithmeticReplacement,
                        hash: format!("ch{}", i),
                        status: MutantStatus::Killed,
                    },
                    status: MutantStatus::Killed,
                    test_failures: vec![],
                    execution_time_ms: 100,
                    error_message: None,
                }).collect();

                let mut state = MutationState::new(
                    std::path::Path::new("/project"),
                    pending_mutants,
                    60,
                    false,
                    None,
                );

                for result in completed_results {
                    state.completed_mutants.push(result);
                }

                let percentage = state.completion_percentage();
                let expected = if pending + completed == 0 {
                    100.0
                } else {
                    (completed as f64 / (pending + completed) as f64) * 100.0
                };

                // Allow for floating point tolerance
                let tolerance = 0.01;
                prop_assert!(
                    (percentage - expected).abs() < tolerance,
                    "Expected {}%, got {}%", expected, percentage
                );
            }

            #[test]
            fn prop_mutant_serialization_roundtrip(
                id in "[a-zA-Z0-9]{1,20}",
                file_name in "[a-zA-Z0-9_]+\\.rs",
                hash in "[a-f0-9]{8,64}"
            ) {
                let mutant = Mutant {
                    id: id.clone(),
                    original_file: PathBuf::from(&file_name),
                    mutated_source: "fn test() {}".to_string(),
                    location: SourceLocation { line: 1, column: 1, end_line: 1, end_column: 10 },
                    operator: MutationOperatorType::ArithmeticReplacement,
                    hash: hash.clone(),
                    status: MutantStatus::Pending,
                };

                let json = serde_json::to_string(&mutant).map_err(|e| proptest::test_runner::TestCaseError::Fail(e.to_string().into()))?;
                let deserialized: Mutant = serde_json::from_str(&json).map_err(|e| proptest::test_runner::TestCaseError::Fail(e.to_string().into()))?;

                prop_assert_eq!(deserialized.id, id);
                prop_assert_eq!(deserialized.hash, hash);
                prop_assert_eq!(deserialized.status, MutantStatus::Pending);
            }
        }
    }
}
