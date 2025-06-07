//! C++ language AST parser implementation
//!
//! Refactored with dispatch table architecture for reduced complexity.
//! Uses modular design patterns to improve maintainability.
//!
//! The complex implementation has been moved to ast_cpp_dispatch.rs
//! This file now serves as a compatibility wrapper.

use crate::models::unified_ast::AstDag;
use anyhow::Result;
use std::path::Path;

// Re-export the improved dispatch parser
pub use crate::services::ast_cpp_dispatch::CppAstDispatchParser;

/// C++ language AST parser implementation (Legacy compatibility wrapper)
///
/// This is a compatibility wrapper around the new CppAstDispatchParser.
/// The actual implementation with improved dispatch table architecture
/// is located in ast_cpp_dispatch.rs.
///
/// This reduces the complexity of this file from ~840 lines to ~60 lines
/// while maintaining backward compatibility.
pub struct CppAstParser {
    #[cfg(feature = "cpp-ast")]
    inner: CppAstDispatchParser,
}

impl Default for CppAstParser {
    fn default() -> Self {
        Self::new()
    }
}

impl CppAstParser {
    pub fn new() -> Self {
        #[cfg(feature = "cpp-ast")]
        {
            Self {
                inner: CppAstDispatchParser::new(),
            }
        }

        #[cfg(not(feature = "cpp-ast"))]
        {
            Self {}
        }
    }

    pub fn parse_file(&mut self, path: &Path, content: &str) -> Result<AstDag> {
        #[cfg(feature = "cpp-ast")]
        {
            self.inner.parse_file(path, content)
        }

        #[cfg(not(feature = "cpp-ast"))]
        {
            let _ = (path, content);
            Err(anyhow::anyhow!(
                "C++ AST parsing requires the 'cpp-ast' feature"
            ))
        }
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
