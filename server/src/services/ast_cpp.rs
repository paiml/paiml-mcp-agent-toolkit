//! C++ AST analysis - MIGRATION IN PROGRESS
//!
//! This module is being migrated to the new unified AST architecture.
//! It now acts as a facade, delegating to the compatibility layer.
//!
//! Migration status: Using compatibility shim
//! Target: server/src/ast/languages/cpp.rs

use std::path::Path;
use anyhow::Result;
use crate::models::unified_ast::AstDag;

// Re-export compatibility functions
pub use super::ast_cpp_compat::{
    analyze_cpp_file,
    analyze_cpp_file_with_classifier,
    analyze_cpp_file_with_complexity,
    analyze_cpp_file_with_complexity_and_classifier,
};

// Dispatch parser removed - functionality moved to new AST module

// Legacy compatibility types (may be referenced by other modules)
pub struct CppAstParser {}

impl Default for CppAstParser {
    fn default() -> Self {
        Self::new()
    }
}

impl CppAstParser {
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse_file(&mut self, _path: &Path, _content: &str) -> Result<AstDag> {
        // Placeholder - use new AST module for C++ parsing
        Err(anyhow::anyhow!(
            "C++ AST parsing has been moved to the new AST module"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "cpp-ast")]
    fn test_parse_simple_cpp_class() {
        let mut parser = CppAstParser::new();
        let content = r#"
class MyClass {
public:
    MyClass() {}
    ~MyClass() {}
    void doSomething() const { }
private:
    int value;
};
"#;
        let result = parser.parse_file(Path::new("test.cpp"), content);
        assert!(result.is_ok());

        let dag = result.unwrap();
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    #[cfg(feature = "cpp-ast")]
    fn test_parse_cpp_templates() {
        let mut parser = CppAstParser::new();
        let content = r#"
template<typename T>
class Vector {
    T* data;
    size_t size;
public:
    Vector() : data(nullptr), size(0) {}
    void push_back(const T& value) { }
};
"#;
        let result = parser.parse_file(Path::new("test.cpp"), content);
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(feature = "cpp-ast")]
    fn test_parse_cpp_lambdas() {
        let mut parser = CppAstParser::new();
        let content = r#"
void example() {
    auto lambda = [](int x) -> int { return x * 2; };
    auto result = lambda(5);
}
"#;
        let result = parser.parse_file(Path::new("test.cpp"), content);
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(not(feature = "cpp-ast"))]
    fn test_cpp_ast_disabled() {
        let mut parser = CppAstParser::new();
        let content = "class A {};";
        let result = parser.parse_file(Path::new("test.cpp"), content);
        assert!(result.is_err());
    }

    #[test]
    fn test_compatibility_layer() {
        // Test that the parser can be created in both feature configurations
        let _parser = CppAstParser::new();

        // Verify that default() works
        let _default_parser = CppAstParser::default();

        // This should compile regardless of feature flags
        // Test passes if compilation succeeds
    }
}
