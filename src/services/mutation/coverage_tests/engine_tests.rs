#![cfg_attr(coverage_nightly, coverage(off))]

use super::*;
use std::sync::Arc;

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
