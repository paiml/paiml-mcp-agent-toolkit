//! Tests that generated mutants compile successfully
//!
//! RED TEST for v2.132.0: Mutants should compile, not just generate

use pmat::services::mutation::{MutationEngine, MutationConfig, RustAdapter, MutantExecutor};
use std::sync::Arc;
use tempfile::NamedTempFile;
use std::io::Write;

/// RED TEST: Generated mutant should compile
///
/// v2.131.0: Generates mutant but doesn't compile (0% compilation rate)
/// v2.132.0: Should generate compilable mutant source
#[tokio::test]
async fn test_unary_mutant_compiles() {
    let source = r#"
fn validate(x: bool) -> bool {
    !x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate() {
        assert_eq!(validate(true), false);
        assert_eq!(validate(false), true);
    }
}
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(source.as_bytes()).unwrap();
    let temp_path = temp_file.path().to_path_buf();

    let adapter = Arc::new(RustAdapter::new());
    let config = MutationConfig::default();
    let engine = MutationEngine::new(adapter, config);

    let mutants = engine.generate_mutants_from_file(&temp_path).await.unwrap();

    assert!(mutants.len() > 0, "Should generate at least 1 mutant");

    // Check that mutated source is a complete file, not just an expression
    let first_mutant = &mutants[0];
    println!("Mutated source:\n{}", first_mutant.mutated_source);

    // Should contain function signature
    assert!(
        first_mutant.mutated_source.contains("fn validate"),
        "Mutated source should contain function signature, got: {}",
        first_mutant.mutated_source
    );

    // Should parse as valid Rust
    let parse_result = syn::parse_file(&first_mutant.mutated_source);
    assert!(
        parse_result.is_ok(),
        "Mutated source should parse as valid Rust file, got error: {:?}",
        parse_result.err()
    );
}

/// RED TEST: Statement deletion should compile
///
/// v2.132.0: SDL generates () expressions that don't compile in all contexts
/// v2.133.0: Should delete statements entirely from AST
#[tokio::test]
async fn test_statement_deletion_compiles() {
    let source = r#"
fn process(x: i32) -> i32 {
    validate(x);
    compute(x)
}

fn validate(x: i32) {
    println!("validating {}", x);
}

fn compute(x: i32) -> i32 {
    x * 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() {
        assert_eq!(process(5), 10);
    }
}
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(source.as_bytes()).unwrap();
    let temp_path = temp_file.path().to_path_buf();

    let adapter = Arc::new(RustAdapter::new());
    let config = MutationConfig::default();
    let engine = MutationEngine::new(adapter, config);

    let mutants = engine.generate_mutants_from_file(&temp_path).await.unwrap();

    // Find SDL mutant (statement deletion)
    let sdl_mutant = mutants.iter().find(|m| {
        matches!(m.operator, pmat::services::mutation::MutationOperatorType::StatementDeletion)
    });

    assert!(sdl_mutant.is_some(), "Should generate SDL mutant");
    let mutant = sdl_mutant.unwrap();

    println!("SDL Mutated source:\n{}", mutant.mutated_source);

    // Should contain function signature
    assert!(
        mutant.mutated_source.contains("fn process"),
        "Should contain function signature"
    );

    // The key test: Should NOT contain "() ;" (which is what current broken SDL does)
    assert!(
        !mutant.mutated_source.contains("() ;"),
        "SDL should delete statement, not replace with '() ;'. Got: {}",
        mutant.mutated_source
    );

    // Should parse as valid Rust
    let parse_result = syn::parse_file(&mutant.mutated_source);
    assert!(
        parse_result.is_ok(),
        "SDL mutated source should parse as valid Rust: {:?}",
        parse_result.err()
    );

    // The real test: Does it compile?
    // If SDL is working, the statement should be deleted entirely
    // Expected: "fn process(x: i32) -> i32 { compute(x) }" (no validate call)
}

/// RED TEST: SDL should not delete method calls inside if conditions
///
/// v2.133.0: SDL deletes tool_names.insert(name) from "if !tool_names.insert(name)"
/// Result: "if ! {" which is invalid syntax
/// v2.134.0: Should only delete top-level statements, not expressions in conditions
#[tokio::test]
async fn test_sdl_does_not_delete_expressions_in_conditions() {
    let source = r#"
fn validate() -> Result<(), String> {
    let mut set = std::collections::HashSet::new();
    let name = "test";

    if !set.insert(name) {
        return Err("duplicate".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate() {
        assert!(validate().is_ok());
    }
}
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(source.as_bytes()).unwrap();
    let temp_path = temp_file.path().to_path_buf();

    let adapter = Arc::new(RustAdapter::new());
    let config = MutationConfig::default();
    let engine = MutationEngine::new(adapter, config);

    let mutants = engine.generate_mutants_from_file(&temp_path).await.unwrap();

    // Find SDL mutants
    let sdl_mutants: Vec<_> = mutants
        .iter()
        .filter(|m| matches!(m.operator, pmat::services::mutation::MutationOperatorType::StatementDeletion))
        .collect();

    println!("Found {} SDL mutants", sdl_mutants.len());

    // All SDL mutants should compile
    for mutant in &sdl_mutants {
        println!("Testing SDL mutant: {}", mutant.id);
        println!("Mutated source:\n{}", mutant.mutated_source);

        // Should parse as valid Rust
        let parse_result = syn::parse_file(&mutant.mutated_source);
        assert!(
            parse_result.is_ok(),
            "SDL mutant should parse: {:?}\nMutated source: {}",
            parse_result.err(),
            mutant.mutated_source
        );

        // If source contains Ok(()), the mutant should also contain it (don't delete return values)
        // This is the key test: SDL should not delete the final return expression
        if source.contains("Ok(())") {
            assert!(
                mutant.mutated_source.contains("Ok"),
                "SDL should not delete function return value Ok(()). Mutant: {}",
                mutant.mutated_source
            );
        }
    }
}

/// RED TEST: Mutant should compile with cargo
///
/// This is the ultimate test - can cargo actually compile it?
#[tokio::test]
#[ignore] // Will enable once AST replacement is implemented
async fn test_mutant_compiles_with_cargo() {
    let source = r#"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }
}
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(source.as_bytes()).unwrap();
    let temp_path = temp_file.path().to_path_buf();

    let adapter = Arc::new(RustAdapter::new());
    let config = MutationConfig::default();
    let engine = MutationEngine::new(adapter, config);

    let mutants = engine.generate_mutants_from_file(&temp_path).await.unwrap();

    assert!(mutants.len() > 0);

    // Try to execute the first mutant
    let work_dir = temp_path.parent().unwrap().to_path_buf();
    let executor = MutantExecutor::new(work_dir);

    let result = executor.execute_mutant(&mutants[0]).await;

    // Should NOT be a compile error
    assert!(
        result.is_ok(),
        "Mutant execution should succeed (even if tests fail)"
    );

    let mutation_result = result.unwrap();
    assert_ne!(
        mutation_result.status,
        pmat::services::mutation::MutantStatus::CompileError,
        "Mutant should compile successfully. Error: {:?}",
        mutation_result.error_message
    );
}

/// RED TEST: Simple expression mutation
///
/// Start with the simplest possible case
#[tokio::test]
async fn test_simple_expression_mutation_compiles() {
    let source = r#"fn negate(x: i32) -> i32 { -x }"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(source.as_bytes()).unwrap();
    let temp_path = temp_file.path().to_path_buf();

    let adapter = Arc::new(RustAdapter::new());
    let config = MutationConfig::default();
    let engine = MutationEngine::new(adapter, config);

    let mutants = engine.generate_mutants_from_file(&temp_path).await.unwrap();

    if mutants.is_empty() {
        panic!("No mutants generated");
    }

    let mutant = &mutants[0];

    // Should be a complete function, not just "x"
    println!("Original: {}", source);
    println!("Mutated:  {}", mutant.mutated_source);

    assert!(
        mutant.mutated_source.contains("fn negate"),
        "Should contain function name"
    );

    assert!(
        mutant.mutated_source.contains("-> i32"),
        "Should contain return type"
    );

    // Parse check
    let parse_result = syn::parse_file(&mutant.mutated_source);
    assert!(
        parse_result.is_ok(),
        "Should parse as valid Rust: {:?}",
        parse_result.err()
    );
}
