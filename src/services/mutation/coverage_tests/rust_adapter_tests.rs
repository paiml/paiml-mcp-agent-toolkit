#![cfg_attr(coverage_nightly, coverage(off))]

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
