// ==================== FuzzMutationStrategy Tests ====================

#[test]
fn test_fuzz_mutation_strategy_new() {
    let engine = MutationEngine::default_rust();
    let config = FuzzConfig::default();
    let strategy = FuzzMutationStrategy::new(engine, config);
    assert_eq!(strategy.config().iterations, 1000);
}

#[test]
fn test_fuzz_mutation_strategy_config() {
    let engine = MutationEngine::default_rust();
    let config = FuzzConfig {
        iterations: 500,
        ..Default::default()
    };
    let strategy = FuzzMutationStrategy::new(engine, config);
    assert_eq!(strategy.config().iterations, 500);
}

#[test]
fn test_fuzz_mutation_strategy_engine() {
    let engine = MutationEngine::default_rust();
    let config = FuzzConfig::default();
    let strategy = FuzzMutationStrategy::new(engine, config);
    // Just verify we can access the engine
    let _ = strategy.engine();
}

#[test]
fn test_fuzz_mutation_strategy_generate_inputs() {
    let engine = MutationEngine::default_rust();
    let config = FuzzConfig::default();
    let strategy = FuzzMutationStrategy::new(engine, config);

    let inputs = strategy.generate_inputs(10);
    assert_eq!(inputs.len(), 10);
    for input in &inputs {
        assert!(!input.is_empty());
        assert!(input.len() <= 256);
    }
}

#[test]
fn test_fuzz_mutation_strategy_generate_inputs_zero() {
    let engine = MutationEngine::default_rust();
    let config = FuzzConfig::default();
    let strategy = FuzzMutationStrategy::new(engine, config);

    let inputs = strategy.generate_inputs(0);
    assert!(inputs.is_empty());
}

#[test]
fn test_fuzz_mutation_strategy_generate_grammar_based_inputs() {
    let engine = MutationEngine::default_rust();
    let config = FuzzConfig::default();
    let strategy = FuzzMutationStrategy::new(engine, config);

    let inputs = strategy.generate_grammar_based_inputs(6, "json");
    assert_eq!(inputs.len(), 6);

    // Check pattern: 0, 3 -> {}, 1, 4 -> [], 2, 5 -> empty
    assert_eq!(inputs[0], b"{}".to_vec());
    assert_eq!(inputs[1], b"[]".to_vec());
    assert!(inputs[2].is_empty());
    assert_eq!(inputs[3], b"{}".to_vec());
}

#[test]
fn test_fuzz_mutation_strategy_generate_grammar_based_inputs_empty() {
    let engine = MutationEngine::default_rust();
    let config = FuzzConfig::default();
    let strategy = FuzzMutationStrategy::new(engine, config);

    let inputs = strategy.generate_grammar_based_inputs(0, "any");
    assert!(inputs.is_empty());
}

// ==================== execute_mutant_with_input Tests ====================

#[test]
fn test_execute_mutant_with_input_success() {
    let result = execute_mutant_with_input("fn test() {}", &[1, 2, 3]);
    assert!(result.is_ok());
}

#[test]
fn test_execute_mutant_with_input_empty_input() {
    let result = execute_mutant_with_input("fn test() {}", &[]);
    assert!(result.is_ok());
}

#[test]
fn test_execute_mutant_with_input_small_input() {
    let result = execute_mutant_with_input("fn test() {}", &[0; 50]);
    assert!(result.is_ok());
}

#[test]
fn test_execute_mutant_with_input_boundary() {
    let result = execute_mutant_with_input("fn test() {}", &[0; 100]);
    assert!(result.is_ok());
}

#[test]
fn test_execute_mutant_with_input_crash() {
    let result = execute_mutant_with_input("fn test() {}", &[0; 101]);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("buffer overflow"));
}

#[test]
fn test_execute_mutant_with_input_large_crash() {
    let result = execute_mutant_with_input("fn test() {}", &[0; 500]);
    assert!(result.is_err());
}

// ==================== mutate_input Tests ====================

#[test]
fn test_mutate_input_empty() {
    let result = mutate_input(&[]);
    assert_eq!(result.len(), 1);
}

#[test]
fn test_mutate_input_single_byte() {
    let original = vec![42u8];
    let mutated = mutate_input(&original);
    // Mutated should be different from original or have different length
    assert!(!mutated.is_empty());
}

/// `mutate_input` must never hand back an empty input for a non-empty seed.
///
/// This was a real flake, not a theoretical one: the delete-byte strategy is
/// one of five picked uniformly at random, so a 1-byte seed came back empty
/// 20% of the time and `test_mutate_input_single_byte` above failed roughly
/// one run in five. It survived because `src/services/mutation/` is behind
/// `mutation-testing` -> `advanced-analysis` -> `full`, and until the
/// `feature-tests / full` leg in .github/workflows/feature-matrix.yml, nothing
/// executed it. The first two runs of that leg disagreed with each other.
///
/// The invariant is the function's own: the empty-seed branch deliberately
/// returns a fresh byte rather than nothing, and the coverage-guided loop in
/// `fuzzing_strategy.rs` feeds this straight back into the corpus, where an
/// empty entry is a wasted iteration.
///
/// Single sample is 20% to fail; 200 draws of a 1-byte seed makes a surviving
/// bug a 1-in-10^19 event, so this fails deterministically in practice.
#[test]
fn mutate_input_never_empties_a_non_empty_seed() {
    for seed_len in 1..=3usize {
        let seed: Vec<u8> = (0..seed_len as u8).collect();
        for i in 0..200 {
            let mutated = mutate_input(&seed);
            assert!(
                !mutated.is_empty(),
                "draw {i} on a {seed_len}-byte seed returned an empty input; \
                 mutate_input must not delete a seed out of existence"
            );
        }
    }
}

/// COUNTER-TEST: passes before and after the fix. The delete strategy must
/// still be able to SHRINK a seed -- the fix must not turn deletion into a
/// no-op, which would silently remove one of the five strategies.
#[test]
fn mutate_input_can_still_shrink_a_multi_byte_seed() {
    let seed = vec![1u8, 2, 3, 4, 5];
    let shrunk = (0..500).any(|_| mutate_input(&seed).len() < seed.len());
    assert!(
        shrunk,
        "no draw in 500 shrank a 5-byte seed; the delete-byte strategy is dead"
    );
}

#[test]
fn test_mutate_input_multiple_bytes() {
    let original = vec![1, 2, 3, 4, 5];
    let mutated = mutate_input(&original);
    // Mutated should exist (can be same or different)
    assert!(!mutated.is_empty() || mutated == original.clone());
}

#[test]
fn test_mutate_input_preserves_some_structure() {
    let original = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mutated = mutate_input(&original);
    // Size should be similar (within +/- 4 bytes typically)
    let size_diff = (mutated.len() as i32 - original.len() as i32).abs();
    assert!(size_diff <= 5);
}

#[test]
fn test_mutate_input_randomness() {
    let original = vec![100, 100, 100, 100, 100];
    let mut results = std::collections::HashSet::new();

    // Run multiple times to verify randomness
    for _ in 0..100 {
        let mutated = mutate_input(&original);
        results.insert(mutated);
    }

    // Should produce multiple different results
    assert!(results.len() > 1);
}

#[test]
fn test_mutate_input_all_zeros() {
    let original = vec![0, 0, 0, 0, 0];
    let mutated = mutate_input(&original);
    assert!(!mutated.is_empty());
}

#[test]
fn test_mutate_input_all_ones() {
    let original = vec![255, 255, 255, 255];
    let mutated = mutate_input(&original);
    assert!(!mutated.is_empty());
}

// ==================== Edge Cases and Integration Tests ====================

#[test]
fn test_fuzz_config_min_valid_values() {
    let config = FuzzConfig {
        iterations: 1,
        input_generator: InputGeneratorType::Random,
        crash_detection: false,
        // 1 MILLISECOND, not 1 nanosecond. `validate()` requires
        // `iteration_timeout.as_millis() > 0`, and `from_nanos(1)` has
        // as_millis() == 0 — so the old value was not the minimum VALID
        // one, it was just below the floor. The test's intent (minimum
        // valid values validate) is preserved; its arithmetic was wrong.
        iteration_timeout: Duration::from_millis(1),
    };
    assert!(config.validate().is_ok());
}

#[test]
fn test_fuzz_config_max_values() {
    let config = FuzzConfig {
        iterations: usize::MAX,
        input_generator: InputGeneratorType::CoverageGuided,
        crash_detection: true,
        iteration_timeout: Duration::from_secs(3600),
    };
    assert!(config.validate().is_ok());
}

#[test]
fn test_fuzz_result_max_crashes() {
    let crashes: Vec<String> = (0..1000).map(|i| format!("crash_{}", i)).collect();
    let result = FuzzResult {
        crashes,
        hangs: vec![],
        coverage_increase: 0.0,
    };
    assert!(result.has_crashes());
    assert_eq!(result.crashes.len(), 1000);
}

#[test]
fn test_fuzz_result_max_hangs() {
    let hangs: Vec<Vec<u8>> = (0..1000).map(|i| vec![i as u8]).collect();
    let result = FuzzResult {
        crashes: vec![],
        hangs,
        coverage_increase: 0.0,
    };
    assert!(result.has_hangs());
    assert_eq!(result.hangs.len(), 1000);
}

#[test]
fn test_fuzz_result_coverage_boundaries() {
    let result_zero = FuzzResult {
        crashes: vec![],
        hangs: vec![],
        coverage_increase: 0.0,
    };
    assert!((result_zero.coverage_increase - 0.0).abs() < f64::EPSILON);

    let result_one = FuzzResult {
        crashes: vec![],
        hangs: vec![],
        coverage_increase: 1.0,
    };
    assert!((result_one.coverage_increase - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_strategy_with_different_generator_types() {
    let engine = MutationEngine::default_rust();

    for gen_type in [
        InputGeneratorType::Random,
        InputGeneratorType::GrammarBased,
        InputGeneratorType::MutationBased,
        InputGeneratorType::CoverageGuided,
    ] {
        let config = FuzzConfig {
            input_generator: gen_type,
            ..Default::default()
        };
        let strategy = FuzzMutationStrategy::new(engine.clone(), config);
        assert_eq!(strategy.config().input_generator, gen_type);
    }
}

#[test]
fn test_input_generator_type_all_variants_serialization_roundtrip() {
    let variants = [
        InputGeneratorType::Random,
        InputGeneratorType::GrammarBased,
        InputGeneratorType::MutationBased,
        InputGeneratorType::CoverageGuided,
    ];

    for variant in variants {
        let json = serde_json::to_string(&variant).unwrap();
        let parsed: InputGeneratorType = serde_json::from_str(&json).unwrap();
        assert_eq!(variant, parsed);
    }
}

#[test]
fn test_fuzz_config_serialization_roundtrip() {
    let config = FuzzConfig {
        iterations: 777,
        input_generator: InputGeneratorType::MutationBased,
        crash_detection: false,
        iteration_timeout: Duration::from_millis(333),
    };

    let json = serde_json::to_string(&config).unwrap();
    let parsed: FuzzConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.iterations, parsed.iterations);
    assert_eq!(config.input_generator, parsed.input_generator);
    assert_eq!(config.crash_detection, parsed.crash_detection);
}

#[test]
fn test_fuzz_result_serialization_roundtrip() {
    let result = FuzzResult {
        crashes: vec!["a".to_string(), "b".to_string()],
        hangs: vec![vec![1, 2], vec![3, 4, 5]],
        coverage_increase: 0.42,
    };

    let json = serde_json::to_string(&result).unwrap();
    let parsed: FuzzResult = serde_json::from_str(&json).unwrap();

    assert_eq!(result.crashes, parsed.crashes);
    assert_eq!(result.hangs, parsed.hangs);
    assert!((result.coverage_increase - parsed.coverage_increase).abs() < f64::EPSILON);
}

#[test]
fn test_fuzz_mutation_report_execution_time() {
    let report = FuzzMutationReport {
        total_mutants: 1,
        mutants_with_crashes: 0,
        mutants_with_hangs: 0,
        execution_time: Duration::from_millis(12345),
        results: vec![],
    };

    assert_eq!(report.execution_time.as_millis(), 12345);
    assert_eq!(report.execution_time.as_secs(), 12);
}

#[test]
fn test_multiple_input_generation_consistency() {
    let engine = MutationEngine::default_rust();
    let config = FuzzConfig::default();
    let strategy = FuzzMutationStrategy::new(engine, config);

    // Generate multiple batches
    let batch1 = strategy.generate_inputs(5);
    let batch2 = strategy.generate_inputs(5);

    // Both should have correct count
    assert_eq!(batch1.len(), 5);
    assert_eq!(batch2.len(), 5);

    // Due to randomness, batches should be different
    // (extremely unlikely to be the same)
    assert_ne!(batch1, batch2);
}
