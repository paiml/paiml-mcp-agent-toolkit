//! Integration tests for mutation generation
//!
//! These tests verify that PMAT actually generates mutants on real Rust code.
//! Regression test for: 0 mutants generated on pforge validator.rs

use pmat::services::mutation::{MutationEngine, MutationConfig, RustAdapter};
use std::sync::Arc;
use tempfile::NamedTempFile;
use std::io::Write;

/// RED TEST: Verify unary operator mutation (!) is detected
///
/// cargo-mutants found: "delete ! in validate_config"
/// PMAT should find: UOR mutation on ! operator
#[tokio::test]
async fn test_unary_operator_negation_detected() {
    let source = r#"
fn validate(x: bool) -> bool {
    !x
}
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(source.as_bytes()).unwrap();
    let temp_path = temp_file.path().to_path_buf();

    let adapter = Arc::new(RustAdapter::new());
    let config = MutationConfig::default();
    let engine = MutationEngine::new(adapter, config);

    let mutants = engine.generate_mutants_from_file(&temp_path).await.unwrap();

    // Debug: print what we got
    println!("\nUnary test generated {} mutants:", mutants.len());
    for (i, mutant) in mutants.iter().enumerate() {
        println!("  {}. {:?} at line {}", i + 1, mutant.operator, mutant.location.line);
        println!("     Source: {}", &mutant.mutated_source[..mutant.mutated_source.len().min(50)]);
    }

    // Should generate at least 1 mutant for the ! operator
    assert!(
        mutants.len() > 0,
        "Expected at least 1 mutant for unary negation (!), got 0"
    );

    // Check that we have a UOR (unary) mutation
    let has_unary_mutation = mutants.iter().any(|m| {
        format!("{:?}", m.operator).contains("Unary")
    });

    assert!(
        has_unary_mutation,
        "Expected at least one UnaryReplacement mutant, got: {:?}",
        mutants.iter().map(|m| format!("{:?}", m.operator)).collect::<Vec<_>>()
    );
}

/// RED TEST: Verify boolean literal mutations
///
/// Should mutate: true -> false, false -> true
#[tokio::test]
async fn test_boolean_literal_mutation_detected() {
    let source = r#"
fn is_valid() -> bool {
    true
}

fn is_invalid() -> bool {
    false
}
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(source.as_bytes()).unwrap();
    let temp_path = temp_file.path().to_path_buf();

    let adapter = Arc::new(RustAdapter::new());
    let config = MutationConfig::default();
    let engine = MutationEngine::new(adapter, config);

    let mutants = engine.generate_mutants_from_file(&temp_path).await.unwrap();

    // Should generate at least 2 mutants (true->false, false->true)
    assert!(
        mutants.len() >= 2,
        "Expected at least 2 mutants for boolean literals, got {}",
        mutants.len()
    );
}

/// RED TEST: Verify method call mutations
///
/// pforge validator.rs uses: tool_names.insert(name), path.contains("::")
/// These should generate mutants
#[tokio::test]
async fn test_method_call_mutations_detected() {
    let source = r#"
fn validate(s: &str) -> bool {
    !s.is_empty()
}
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(source.as_bytes()).unwrap();
    let temp_path = temp_file.path().to_path_buf();

    let adapter = Arc::new(RustAdapter::new());
    let config = MutationConfig::default();
    let engine = MutationEngine::new(adapter, config);

    let mutants = engine.generate_mutants_from_file(&temp_path).await.unwrap();

    // Should generate at least 1 mutant for !s.is_empty()
    assert!(
        mutants.len() > 0,
        "Expected at least 1 mutant for !s.is_empty(), got 0"
    );
}

/// RED TEST: Verify pforge validator.rs generates mutants
///
/// This is the exact file that failed with 0 mutants
/// cargo-mutants found 4 mutants, PMAT should find at least some
#[tokio::test]
async fn test_pforge_validator_generates_mutants() {
    let source = r#"
use std::collections::HashSet;

pub fn validate_config(tools: &[String]) -> Result<(), String> {
    let mut tool_names = HashSet::new();
    for tool in tools {
        if !tool_names.insert(tool) {
            return Err(format!("Duplicate tool: {}", tool));
        }
    }
    Ok(())
}

fn validate_handler_path(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("empty path".to_string());
    }

    if !path.contains("::") {
        return Err(format!("invalid format: {}", path));
    }

    Ok(())
}
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(source.as_bytes()).unwrap();
    let temp_path = temp_file.path().to_path_buf();

    let adapter = Arc::new(RustAdapter::new());
    let config = MutationConfig::default();
    let engine = MutationEngine::new(adapter, config);

    let mutants = engine.generate_mutants_from_file(&temp_path).await.unwrap();

    // cargo-mutants found 4 mutants on similar code
    // We should find at least 2 (the two ! operators)
    assert!(
        mutants.len() >= 2,
        "Expected at least 2 mutants for pforge-style code (cargo-mutants found 4), got {}",
        mutants.len()
    );

    // Print what we found for debugging
    println!("Generated {} mutants:", mutants.len());
    for (i, mutant) in mutants.iter().enumerate() {
        println!("  {}. {:?} at line {}", i + 1, mutant.operator, mutant.location.line);
    }
}

/// RED TEST: Simple arithmetic that should definitely generate mutants
#[tokio::test]
async fn test_arithmetic_mutations_detected() {
    let source = r#"
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn subtract(a: i32, b: i32) -> i32 {
    a - b
}
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(source.as_bytes()).unwrap();
    let temp_path = temp_file.path().to_path_buf();

    let adapter = Arc::new(RustAdapter::new());
    let config = MutationConfig::default();
    let engine = MutationEngine::new(adapter, config);

    let mutants = engine.generate_mutants_from_file(&temp_path).await.unwrap();

    // Debug: print what we got
    println!("\nArithmetic test generated {} mutants:", mutants.len());
    for (i, mutant) in mutants.iter().enumerate() {
        println!("  {}. {:?} at line {}", i + 1, mutant.operator, mutant.location.line);
        println!("     Source: {}", &mutant.mutated_source[..mutant.mutated_source.len().min(50)]);
    }

    // Should generate mutants for + and -
    assert!(
        mutants.len() >= 2,
        "Expected at least 2 mutants for arithmetic operators, got {}",
        mutants.len()
    );
}
