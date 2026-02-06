#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for services/mutation module
//!
//! Comprehensive tests covering mutation testing types, scoring, state,
//! language detection, distributed execution, and related functionality.
//!
//! FEATURE GATE: mutation-testing
//!
//! NOTE: The mutation-testing feature currently has compilation issues
//! (syn crate missing `visit-mut` feature). These tests are designed to
//! run when that issue is fixed.
//!
//! To run these tests:
//! ```bash
//! cargo test --lib coverage_boost_mutation --features mutation-testing
//! ```

#![cfg(feature = "mutation-testing")]

use crate::services::mutation::{
    DistributedConfig, Language, LanguageRegistry, Mutant, MutantStatus, MutationConfig,
    MutationEngine, MutationOperatorType, MutationProgress, MutationResult, MutationScore,
    MutationScorer, MutationState, MutationStateConfig, MutationStrategy, RustAdapter,
    SourceLocation, TestRunResult, WeakSpot,
};
use crate::services::mutation::language::LanguageAdapter;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

// =============================================================================
// Helper Functions
// =============================================================================

fn create_test_location(line: usize, column: usize) -> SourceLocation {
    SourceLocation {
        line,
        column,
        end_line: line,
        end_column: column + 10,
    }
}

fn create_test_mutant(id: &str, status: MutantStatus) -> Mutant {
    Mutant {
        id: id.to_string(),
        original_file: PathBuf::from("test.rs"),
        mutated_source: "fn test() {}".to_string(),
        location: create_test_location(1, 1),
        operator: MutationOperatorType::ArithmeticReplacement,
        hash: format!("hash_{}", id),
        status,
    }
}

fn create_test_result(status: MutantStatus, file: &str, line: usize) -> MutationResult {
    MutationResult {
        mutant: Mutant {
            id: format!("{}_{}", file.replace(".rs", ""), line),
            original_file: PathBuf::from(file),
            mutated_source: "mutated".to_string(),
            location: create_test_location(line, 1),
            operator: MutationOperatorType::ArithmeticReplacement,
            hash: format!("hash_{}_{}", file, line),
            status: status.clone(),
        },
        status,
        test_failures: vec![],
        execution_time_ms: 100,
        error_message: None,
    }
}

// =============================================================================
// MutationOperatorType Tests
// =============================================================================

#[test]
fn test_mutation_operator_type_all_variants_debug() {
    let variants = vec![
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
        MutationOperatorType::Custom("custom_op".to_string()),
        MutationOperatorType::None,
    ];

    for variant in variants {
        let debug_str = format!("{:?}", variant);
        assert!(!debug_str.is_empty());
    }
}

#[test]
fn test_mutation_operator_type_clone() {
    let original = MutationOperatorType::Custom("test".to_string());
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_mutation_operator_type_ordering() {
    let ops: Vec<MutationOperatorType> = vec![
        MutationOperatorType::ArithmeticReplacement,
        MutationOperatorType::RelationalReplacement,
        MutationOperatorType::ConditionalReplacement,
    ];

    let mut sorted = ops.clone();
    sorted.sort();

    // Just verify sorting works without panic
    assert_eq!(sorted.len(), 3);
}

#[test]
fn test_mutation_operator_type_hash() {
    let mut set: HashSet<MutationOperatorType> = HashSet::new();
    set.insert(MutationOperatorType::ArithmeticReplacement);
    set.insert(MutationOperatorType::RelationalReplacement);
    set.insert(MutationOperatorType::ArithmeticReplacement); // duplicate

    assert_eq!(set.len(), 2);
}

#[test]
fn test_mutation_operator_type_custom_variants() {
    let custom1 = MutationOperatorType::Custom("op1".to_string());
    let custom2 = MutationOperatorType::Custom("op2".to_string());
    let custom1_dup = MutationOperatorType::Custom("op1".to_string());

    assert_ne!(custom1, custom2);
    assert_eq!(custom1, custom1_dup);
}

// =============================================================================
// MutantStatus Tests
// =============================================================================

#[test]
fn test_mutant_status_all_variants_debug() {
    let variants = vec![
        MutantStatus::Pending,
        MutantStatus::Killed,
        MutantStatus::Survived,
        MutantStatus::CompileError,
        MutantStatus::Timeout,
        MutantStatus::Equivalent,
    ];

    for status in variants {
        let debug_str = format!("{:?}", status);
        assert!(!debug_str.is_empty());
    }
}

#[test]
fn test_mutant_status_equality() {
    assert_eq!(MutantStatus::Pending, MutantStatus::Pending);
    assert_eq!(MutantStatus::Killed, MutantStatus::Killed);
    assert_ne!(MutantStatus::Killed, MutantStatus::Survived);
    assert_ne!(MutantStatus::CompileError, MutantStatus::Timeout);
}

#[test]
fn test_mutant_status_hash() {
    let mut set: HashSet<MutantStatus> = HashSet::new();
    set.insert(MutantStatus::Pending);
    set.insert(MutantStatus::Killed);
    set.insert(MutantStatus::Pending); // duplicate

    assert_eq!(set.len(), 2);
}

#[test]
fn test_mutant_status_clone() {
    let status = MutantStatus::Killed;
    let cloned = status.clone();
    assert_eq!(status, cloned);
}

// =============================================================================
// SourceLocation Tests
// =============================================================================

#[test]
fn test_source_location_creation() {
    let loc = SourceLocation {
        line: 10,
        column: 5,
        end_line: 15,
        end_column: 20,
    };

    assert_eq!(loc.line, 10);
    assert_eq!(loc.column, 5);
    assert_eq!(loc.end_line, 15);
    assert_eq!(loc.end_column, 20);
}

#[test]
fn test_source_location_zero_values() {
    let loc = SourceLocation {
        line: 0,
        column: 0,
        end_line: 0,
        end_column: 0,
    };

    assert_eq!(loc.line, 0);
    assert_eq!(loc.column, 0);
}

#[test]
fn test_source_location_max_values() {
    let loc = SourceLocation {
        line: usize::MAX,
        column: usize::MAX,
        end_line: usize::MAX,
        end_column: usize::MAX,
    };

    assert_eq!(loc.line, usize::MAX);
    assert_eq!(loc.column, usize::MAX);
}

#[test]
fn test_source_location_clone() {
    let loc = create_test_location(42, 10);
    let cloned = loc.clone();
    assert_eq!(loc, cloned);
}

#[test]
fn test_source_location_hash() {
    let mut set: HashSet<SourceLocation> = HashSet::new();
    set.insert(create_test_location(1, 1));
    set.insert(create_test_location(2, 2));
    set.insert(create_test_location(1, 1)); // duplicate

    assert_eq!(set.len(), 2);
}

// =============================================================================
// Mutant Tests
// =============================================================================

#[test]
fn test_mutant_creation_complete() {
    let mutant = Mutant {
        id: "test_123".to_string(),
        original_file: PathBuf::from("src/lib.rs"),
        mutated_source: "fn mutated() { }".to_string(),
        location: create_test_location(10, 5),
        operator: MutationOperatorType::ArithmeticReplacement,
        hash: "abc123def456".to_string(),
        status: MutantStatus::Pending,
    };

    assert_eq!(mutant.id, "test_123");
    assert_eq!(mutant.original_file, PathBuf::from("src/lib.rs"));
    assert_eq!(mutant.mutated_source, "fn mutated() { }");
    assert_eq!(mutant.location.line, 10);
    assert_eq!(mutant.hash, "abc123def456");
    assert_eq!(mutant.status, MutantStatus::Pending);
}

#[test]
fn test_mutant_clone_and_equality() {
    let mutant = create_test_mutant("m1", MutantStatus::Pending);
    let cloned = mutant.clone();

    assert_eq!(mutant, cloned);
    assert_eq!(mutant.id, cloned.id);
    assert_eq!(mutant.hash, cloned.hash);
}

#[test]
fn test_mutant_hash_in_collection() {
    let mut set: HashSet<Mutant> = HashSet::new();
    set.insert(create_test_mutant("m1", MutantStatus::Pending));
    set.insert(create_test_mutant("m2", MutantStatus::Killed));
    set.insert(create_test_mutant("m1", MutantStatus::Pending)); // duplicate

    assert_eq!(set.len(), 2);
}

#[test]
fn test_mutant_debug_format() {
    let mutant = create_test_mutant("debug_test", MutantStatus::Survived);
    let debug_str = format!("{:?}", mutant);

    assert!(debug_str.contains("debug_test"));
    assert!(debug_str.contains("Survived"));
}

// =============================================================================
// MutationResult Tests
// =============================================================================

#[test]
fn test_mutation_result_creation_killed() {
    let result = MutationResult {
        mutant: create_test_mutant("killed_test", MutantStatus::Killed),
        status: MutantStatus::Killed,
        test_failures: vec!["test_add".to_string(), "test_sub".to_string()],
        execution_time_ms: 250,
        error_message: None,
    };

    assert_eq!(result.status, MutantStatus::Killed);
    assert_eq!(result.test_failures.len(), 2);
    assert_eq!(result.execution_time_ms, 250);
    assert!(result.error_message.is_none());
}

#[test]
fn test_mutation_result_with_error() {
    let result = MutationResult {
        mutant: create_test_mutant("error_test", MutantStatus::CompileError),
        status: MutantStatus::CompileError,
        test_failures: vec![],
        execution_time_ms: 0,
        error_message: Some("error[E0308]: mismatched types".to_string()),
    };

    assert!(result.error_message.is_some());
    assert!(result.error_message.unwrap().contains("E0308"));
}

#[test]
fn test_mutation_result_timeout() {
    let result = MutationResult {
        mutant: create_test_mutant("timeout_test", MutantStatus::Timeout),
        status: MutantStatus::Timeout,
        test_failures: vec![],
        execution_time_ms: 60000,
        error_message: Some("Test exceeded timeout of 60s".to_string()),
    };

    assert_eq!(result.status, MutantStatus::Timeout);
    assert_eq!(result.execution_time_ms, 60000);
}

#[test]
fn test_mutation_result_clone() {
    let result = create_test_result(MutantStatus::Survived, "test.rs", 10);
    let cloned = result.clone();

    assert_eq!(result.status, cloned.status);
    assert_eq!(result.execution_time_ms, cloned.execution_time_ms);
}

// =============================================================================
// MutationScore Tests
// =============================================================================

#[test]
fn test_mutation_score_from_empty_results() {
    let results: Vec<MutationResult> = vec![];
    let score = MutationScore::from_results(&results);

    assert_eq!(score.total, 0);
    assert_eq!(score.killed, 0);
    assert_eq!(score.survived, 0);
    assert_eq!(score.score, 0.0);
}

#[test]
fn test_mutation_score_all_killed() {
    let results = vec![
        create_test_result(MutantStatus::Killed, "a.rs", 1),
        create_test_result(MutantStatus::Killed, "a.rs", 2),
        create_test_result(MutantStatus::Killed, "a.rs", 3),
    ];
    let score = MutationScore::from_results(&results);

    assert_eq!(score.total, 3);
    assert_eq!(score.killed, 3);
    assert_eq!(score.survived, 0);
    assert_eq!(score.score, 1.0);
}

#[test]
fn test_mutation_score_all_survived() {
    let results = vec![
        create_test_result(MutantStatus::Survived, "a.rs", 1),
        create_test_result(MutantStatus::Survived, "a.rs", 2),
    ];
    let score = MutationScore::from_results(&results);

    assert_eq!(score.killed, 0);
    assert_eq!(score.survived, 2);
    assert_eq!(score.score, 0.0);
}

#[test]
fn test_mutation_score_mixed_results() {
    let results = vec![
        create_test_result(MutantStatus::Killed, "a.rs", 1),
        create_test_result(MutantStatus::Killed, "a.rs", 2),
        create_test_result(MutantStatus::Survived, "a.rs", 3),
        create_test_result(MutantStatus::Equivalent, "a.rs", 4),
        create_test_result(MutantStatus::CompileError, "a.rs", 5),
    ];
    let score = MutationScore::from_results(&results);

    assert_eq!(score.total, 5);
    assert_eq!(score.killed, 2);
    assert_eq!(score.survived, 1);
    assert_eq!(score.equivalent, 1);
    assert_eq!(score.compile_errors, 1);
}

#[test]
fn test_mutation_score_all_equivalent() {
    let results = vec![
        create_test_result(MutantStatus::Equivalent, "a.rs", 1),
        create_test_result(MutantStatus::Equivalent, "a.rs", 2),
    ];
    let score = MutationScore::from_results(&results);

    assert_eq!(score.equivalent, 2);
    assert_eq!(score.score, 0.0); // No valid mutants
}

#[test]
fn test_mutation_score_all_compile_errors() {
    let results = vec![
        create_test_result(MutantStatus::CompileError, "a.rs", 1),
        create_test_result(MutantStatus::CompileError, "a.rs", 2),
    ];
    let score = MutationScore::from_results(&results);

    assert_eq!(score.compile_errors, 2);
    assert_eq!(score.score, 0.0); // No valid mutants
}

#[test]
fn test_mutation_score_clone() {
    let results = vec![create_test_result(MutantStatus::Killed, "a.rs", 1)];
    let score = MutationScore::from_results(&results);
    let cloned = score.clone();

    assert_eq!(score.total, cloned.total);
    assert_eq!(score.score, cloned.score);
}

// =============================================================================
// WeakSpot Tests
// =============================================================================

#[test]
fn test_weak_spot_creation() {
    let weak_spot = WeakSpot {
        file: PathBuf::from("src/vulnerable.rs"),
        line_range: (10, 50),
        survived_mutants: 5,
        suggestions: vec![
            "Add edge case tests".to_string(),
            "Test error handling".to_string(),
        ],
    };

    assert_eq!(weak_spot.file, PathBuf::from("src/vulnerable.rs"));
    assert_eq!(weak_spot.line_range, (10, 50));
    assert_eq!(weak_spot.survived_mutants, 5);
    assert_eq!(weak_spot.suggestions.len(), 2);
}

#[test]
fn test_weak_spot_clone() {
    let weak_spot = WeakSpot {
        file: PathBuf::from("test.rs"),
        line_range: (1, 10),
        survived_mutants: 3,
        suggestions: vec!["suggestion".to_string()],
    };
    let cloned = weak_spot.clone();

    assert_eq!(weak_spot.file, cloned.file);
    assert_eq!(weak_spot.line_range, cloned.line_range);
}

// =============================================================================
// MutationScorer Tests
// =============================================================================

#[test]
fn test_mutation_scorer_new() {
    let results = vec![create_test_result(MutantStatus::Killed, "a.rs", 1)];
    let scorer = MutationScorer::new(results);
    let score = scorer.calculate_score();

    assert_eq!(score.total, 1);
}

#[test]
fn test_mutation_scorer_weak_spots_empty() {
    let results: Vec<MutationResult> = vec![];
    let scorer = MutationScorer::new(results);
    let weak_spots = scorer.weak_spots();

    assert!(weak_spots.is_empty());
}

#[test]
fn test_mutation_scorer_weak_spots_no_survivors() {
    let results = vec![
        create_test_result(MutantStatus::Killed, "a.rs", 1),
        create_test_result(MutantStatus::Killed, "b.rs", 1),
    ];
    let scorer = MutationScorer::new(results);
    let weak_spots = scorer.weak_spots();

    assert!(weak_spots.is_empty());
}

#[test]
fn test_mutation_scorer_weak_spots_sorted() {
    let results = vec![
        create_test_result(MutantStatus::Survived, "a.rs", 1),
        create_test_result(MutantStatus::Survived, "b.rs", 1),
        create_test_result(MutantStatus::Survived, "b.rs", 2),
        create_test_result(MutantStatus::Survived, "b.rs", 3),
    ];
    let scorer = MutationScorer::new(results);
    let weak_spots = scorer.weak_spots();

    // b.rs should be first (3 survivors) vs a.rs (1 survivor)
    assert_eq!(weak_spots.len(), 2);
    assert_eq!(weak_spots[0].survived_mutants, 3);
    assert_eq!(weak_spots[1].survived_mutants, 1);
}

#[test]
fn test_mutation_scorer_summary() {
    let results = vec![
        create_test_result(MutantStatus::Killed, "a.rs", 1),
        create_test_result(MutantStatus::Survived, "a.rs", 2),
        create_test_result(MutantStatus::CompileError, "a.rs", 3),
    ];
    let scorer = MutationScorer::new(results);
    let summary = scorer.summary();

    assert_eq!(summary.total_mutants, 3);
    assert_eq!(summary.killed, 1);
    assert_eq!(summary.survived, 1);
    assert_eq!(summary.compile_errors, 1);
}

// =============================================================================
// MutationProgress Tests
// =============================================================================

#[test]
fn test_mutation_progress_percentage_zero_total() {
    let progress = MutationProgress {
        total: 0,
        completed: 0,
        in_progress: 0,
        killed: 0,
        survived: 0,
        failed: 0,
    };

    assert_eq!(progress.percentage(), 100.0);
}

#[test]
fn test_mutation_progress_percentage_partial() {
    let progress = MutationProgress {
        total: 100,
        completed: 25,
        in_progress: 5,
        killed: 20,
        survived: 5,
        failed: 0,
    };

    assert_eq!(progress.percentage(), 25.0);
}

#[test]
fn test_mutation_progress_mutation_score() {
    let progress = MutationProgress {
        total: 100,
        completed: 100,
        in_progress: 0,
        killed: 80,
        survived: 20,
        failed: 0,
    };

    assert_eq!(progress.mutation_score(), 80.0);
}

#[test]
fn test_mutation_progress_mutation_score_no_tested() {
    let progress = MutationProgress {
        total: 100,
        completed: 0,
        in_progress: 0,
        killed: 0,
        survived: 0,
        failed: 0,
    };

    assert_eq!(progress.mutation_score(), 0.0);
}

#[test]
fn test_mutation_progress_clone() {
    let progress = MutationProgress {
        total: 50,
        completed: 25,
        in_progress: 5,
        killed: 20,
        survived: 5,
        failed: 0,
    };
    let cloned = progress.clone();

    assert_eq!(progress.total, cloned.total);
    assert_eq!(progress.completed, cloned.completed);
}

// =============================================================================
// MutationState Tests
// =============================================================================

#[test]
fn test_mutation_state_new() {
    let mutants = vec![
        create_test_mutant("m1", MutantStatus::Pending),
        create_test_mutant("m2", MutantStatus::Pending),
    ];
    let state = MutationState::new(std::path::Path::new("/project"), mutants, 60, true, Some(4));

    assert_eq!(state.pending_mutants.len(), 2);
    assert!(state.completed_mutants.is_empty());
    assert!(!state.is_complete());
    assert_eq!(state.total_mutants(), 2);
}

#[test]
fn test_mutation_state_add_result() {
    let mutants = vec![create_test_mutant("m1", MutantStatus::Pending)];
    let mut state =
        MutationState::new(std::path::Path::new("/project"), mutants, 60, false, None);

    let result = MutationResult {
        mutant: create_test_mutant("m1", MutantStatus::Killed),
        status: MutantStatus::Killed,
        test_failures: vec!["test".to_string()],
        execution_time_ms: 100,
        error_message: None,
    };

    state.add_result(result);

    assert!(state.pending_mutants.is_empty());
    assert_eq!(state.completed_mutants.len(), 1);
    assert!(state.is_complete());
}

#[test]
fn test_mutation_state_completion_percentage() {
    let mutants = vec![
        create_test_mutant("m1", MutantStatus::Pending),
        create_test_mutant("m2", MutantStatus::Pending),
    ];
    let mut state =
        MutationState::new(std::path::Path::new("/project"), mutants, 60, false, None);

    assert_eq!(state.completion_percentage(), 0.0);

    let result = MutationResult {
        mutant: create_test_mutant("m1", MutantStatus::Killed),
        status: MutantStatus::Killed,
        test_failures: vec![],
        execution_time_ms: 100,
        error_message: None,
    };
    state.add_result(result);

    assert_eq!(state.completion_percentage(), 50.0);
}

#[test]
fn test_mutation_state_empty() {
    let state = MutationState::new(std::path::Path::new("/project"), vec![], 60, false, None);

    assert!(state.is_complete());
    assert_eq!(state.completion_percentage(), 100.0);
}

#[test]
fn test_mutation_state_default_path() {
    let path = MutationState::default_state_path(std::path::Path::new("/my/project"));

    assert!(path.to_string_lossy().contains(".pmat"));
    assert!(path.to_string_lossy().contains("mutation_state.json"));
}

#[test]
fn test_mutation_state_config() {
    let config = MutationStateConfig {
        timeout_secs: 120,
        worker_count: Some(8),
        parallel: true,
    };

    assert_eq!(config.timeout_secs, 120);
    assert_eq!(config.worker_count, Some(8));
    assert!(config.parallel);
}

// =============================================================================
// MutationConfig Tests
// =============================================================================

#[test]
fn test_mutation_config_default() {
    let config = MutationConfig::default();

    assert!(matches!(config.strategy, MutationStrategy::Selective));
    assert_eq!(config.max_mutants, 0);
    assert!(config.parallel_threads > 0);
}

#[test]
fn test_mutation_config_custom() {
    let config = MutationConfig {
        strategy: MutationStrategy::Random,
        max_mutants: 100,
        parallel_threads: 4,
    };

    assert!(matches!(config.strategy, MutationStrategy::Random));
    assert_eq!(config.max_mutants, 100);
    assert_eq!(config.parallel_threads, 4);
}

#[test]
fn test_mutation_strategy_hybrid() {
    let strategy = MutationStrategy::Hybrid {
        selective: 0.7,
        random: 0.3,
    };

    if let MutationStrategy::Hybrid { selective, random } = strategy {
        assert!((selective - 0.7).abs() < 0.001);
        assert!((random - 0.3).abs() < 0.001);
    } else {
        panic!("Expected Hybrid strategy");
    }
}

// =============================================================================
// DistributedConfig Tests
// =============================================================================

#[test]
fn test_distributed_config_default() {
    let config = DistributedConfig::default();

    assert!(config.worker_count > 0);
    assert!(config.max_concurrent >= config.worker_count);
    assert_eq!(config.queue_size, 1000);
    assert!(config.track_progress);
}

#[test]
fn test_distributed_config_custom() {
    let config = DistributedConfig {
        worker_count: 8,
        max_concurrent: 16,
        queue_size: 500,
        track_progress: false,
    };

    assert_eq!(config.worker_count, 8);
    assert_eq!(config.max_concurrent, 16);
    assert_eq!(config.queue_size, 500);
    assert!(!config.track_progress);
}

#[test]
fn test_distributed_config_clone() {
    let config = DistributedConfig::default();
    let cloned = config.clone();

    assert_eq!(config.worker_count, cloned.worker_count);
    assert_eq!(config.track_progress, cloned.track_progress);
}

// =============================================================================
// Language Tests
// =============================================================================

#[test]
fn test_language_from_extension_rust() {
    let lang = Language::from_extension(std::path::Path::new("test.rs"));
    assert_eq!(lang, Language::Rust);
    assert_eq!(lang.name(), "Rust");
    assert!(lang.is_supported());
}

#[test]
fn test_language_from_extension_python() {
    let lang = Language::from_extension(std::path::Path::new("test.py"));
    assert_eq!(lang, Language::Python);
    assert_eq!(lang.name(), "Python");
    assert!(lang.is_supported());
}

#[test]
fn test_language_from_extension_typescript() {
    let lang = Language::from_extension(std::path::Path::new("test.ts"));
    assert_eq!(lang, Language::TypeScript);

    let tsx_lang = Language::from_extension(std::path::Path::new("test.tsx"));
    assert_eq!(tsx_lang, Language::TypeScript);
}

#[test]
fn test_language_from_extension_javascript() {
    let lang = Language::from_extension(std::path::Path::new("test.js"));
    assert_eq!(lang, Language::JavaScript);

    let jsx_lang = Language::from_extension(std::path::Path::new("test.jsx"));
    assert_eq!(jsx_lang, Language::JavaScript);
}

#[test]
fn test_language_from_extension_go() {
    let lang = Language::from_extension(std::path::Path::new("test.go"));
    assert_eq!(lang, Language::Go);
    assert_eq!(lang.name(), "Go");
}

#[test]
fn test_language_from_extension_cpp() {
    let extensions = vec!["cpp", "cc", "cxx", "hpp", "hxx", "h"];
    for ext in extensions {
        let path = format!("test.{}", ext);
        let lang = Language::from_extension(std::path::Path::new(&path));
        assert_eq!(lang, Language::Cpp, "Failed for extension: {}", ext);
    }
}

#[test]
fn test_language_from_extension_unsupported() {
    let lang = Language::from_extension(std::path::Path::new("test.xyz"));
    assert_eq!(lang, Language::Unsupported);
    assert!(!lang.is_supported());
    assert_eq!(lang.name(), "Unsupported");
}

#[test]
fn test_language_extensions() {
    assert_eq!(Language::Rust.extensions(), vec!["rs"]);
    assert_eq!(Language::Python.extensions(), vec!["py"]);
    assert_eq!(Language::Go.extensions(), vec!["go"]);
    assert!(Language::Unsupported.extensions().is_empty());
}

#[test]
fn test_language_case_sensitive() {
    let lang = Language::from_extension(std::path::Path::new("test.RS"));
    assert_eq!(lang, Language::Unsupported);
}

// =============================================================================
// LanguageRegistry Tests
// =============================================================================

#[test]
fn test_language_registry_new() {
    let registry = LanguageRegistry::new();
    assert!(registry.languages().is_empty());
}

#[test]
fn test_language_registry_default() {
    let registry = LanguageRegistry::default();
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
fn test_language_registry_detect_rust() {
    let mut registry = LanguageRegistry::new();
    registry.register(Arc::new(RustAdapter::new()));

    let detected = registry.detect_language(std::path::Path::new("test.rs"));
    assert!(detected.is_some());
    assert_eq!(detected.unwrap().name(), "rust");
}

#[test]
fn test_language_registry_detect_unknown() {
    let registry = LanguageRegistry::new();
    let detected = registry.detect_language(std::path::Path::new("test.xyz"));
    assert!(detected.is_none());
}

#[test]
fn test_language_registry_get_adapter_unknown() {
    let registry = LanguageRegistry::new();
    assert!(registry.get_adapter("unknown").is_none());
}

// =============================================================================
// TestRunResult Tests
// =============================================================================

#[test]
fn test_test_run_result_passed() {
    let result = TestRunResult {
        passed: true,
        failures: vec![],
        execution_time_ms: 100,
        stdout: "All tests passed".to_string(),
        stderr: String::new(),
    };

    assert!(result.passed);
    assert!(result.failures.is_empty());
}

#[test]
fn test_test_run_result_failed() {
    let result = TestRunResult {
        passed: false,
        failures: vec!["test_add".to_string(), "test_sub".to_string()],
        execution_time_ms: 500,
        stdout: "running 2 tests".to_string(),
        stderr: "thread panicked".to_string(),
    };

    assert!(!result.passed);
    assert_eq!(result.failures.len(), 2);
    assert!(result.stderr.contains("panicked"));
}

#[test]
fn test_test_run_result_clone() {
    let result = TestRunResult {
        passed: true,
        failures: vec![],
        execution_time_ms: 100,
        stdout: "output".to_string(),
        stderr: String::new(),
    };
    let cloned = result.clone();

    assert_eq!(result.passed, cloned.passed);
    assert_eq!(result.execution_time_ms, cloned.execution_time_ms);
}

// =============================================================================
// MutationEngine Tests (basic instantiation)
// =============================================================================

#[test]
fn test_mutation_engine_default_rust() {
    let engine = MutationEngine::default_rust();
    // Just verify it can be created
    assert!(std::mem::size_of_val(&engine) > 0);
}

#[test]
fn test_mutation_engine_new() {
    let adapter = Arc::new(RustAdapter::new());
    let config = MutationConfig::default();
    let engine = MutationEngine::new(adapter, config);
    assert!(std::mem::size_of_val(&engine) > 0);
}

#[test]
fn test_mutation_engine_default() {
    let engine = MutationEngine::default();
    assert!(std::mem::size_of_val(&engine) > 0);
}

// =============================================================================
// RustAdapter Tests
// =============================================================================

#[test]
fn test_rust_adapter_new() {
    let adapter = RustAdapter::new();
    assert_eq!(adapter.name(), "rust");
}

#[test]
fn test_rust_adapter_extensions() {
    let adapter = RustAdapter::new();
    assert_eq!(adapter.extensions(), &["rs"]);
}

#[test]
fn test_rust_adapter_mutation_operators() {
    let adapter = RustAdapter::new();
    let operators = adapter.mutation_operators();
    assert!(!operators.is_empty());

    let names: Vec<&str> = operators.iter().map(|op| op.name()).collect();
    assert!(names.contains(&"AOR"));
    assert!(names.contains(&"ROR"));
}
