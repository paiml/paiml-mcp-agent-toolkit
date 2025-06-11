//! C language AST parser implementation
//!
//! Refactored with dispatch table architecture for reduced complexity.
//! Uses modular design patterns to improve maintainability.
//!
//! The complex implementation has been moved to ast_c_dispatch.rs
//! This file now serves as a compatibility wrapper.

use crate::models::unified_ast::AstDag;
use anyhow::Result;
use std::path::Path;

// Re-export the improved dispatch parser
pub use crate::services::ast_c_dispatch::CAstDispatchParser;

/// C language AST parser implementation (Legacy compatibility wrapper)
///
/// This is a compatibility wrapper around the new CAstDispatchParser.
/// The actual implementation with improved dispatch table architecture
/// is located in ast_c_dispatch.rs.
///
/// This reduces the complexity of this file from ~670 lines to ~60 lines
/// while maintaining backward compatibility and adding C-specific features:
/// - Better goto statement handling
/// - C-specific keywords (restrict)
/// - Enhanced preprocessor support
pub struct CAstParser {
    #[cfg(feature = "c-ast")]
    inner: CAstDispatchParser,
}

impl Default for CAstParser {
    fn default() -> Self {
        Self::new()
    }
}

impl CAstParser {
    pub fn new() -> Self {
        #[cfg(feature = "c-ast")]
        {
            Self {
                inner: CAstDispatchParser::new(),
            }
        }

        #[cfg(not(feature = "c-ast"))]
        {
            Self {}
        }
    }

    pub fn parse_file(&mut self, path: &Path, content: &str) -> Result<AstDag> {
        #[cfg(feature = "c-ast")]
        {
            self.inner.parse_file(path, content)
        }

        #[cfg(not(feature = "c-ast"))]
        {
            let _ = (path, content);
            Err(anyhow::anyhow!(
                "C AST parsing requires the 'c-ast' feature"
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::unified_ast::AstKind;

    #[test]
    #[cfg(feature = "c-ast")]
    fn test_parse_simple_c_function() {
        let mut parser = CAstParser::new();
        let content = r#"
int add(int a, int b) {
    return a + b;
}
"#;
        let result = parser.parse_file(Path::new("test.c"), content);
        assert!(result.is_ok());

        let dag = result.unwrap();
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    #[cfg(feature = "c-ast")]
    fn test_parse_c_with_pointers() {
        let mut parser = CAstParser::new();
        let content = r#"
void process(int *ptr, char **argv) {
    *ptr = 42;
}
"#;
        let result = parser.parse_file(Path::new("test.c"), content);
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(feature = "c-ast")]
    fn test_parse_c_with_goto() {
        let mut parser = CAstParser::new();
        let content = r#"
void example() {
    int i = 0;
    start:
    if (i < 10) {
        i++;
        goto start;
    }
}
"#;
        let result = parser.parse_file(Path::new("test.c"), content);
        assert!(result.is_ok());

        // Verify goto complexity
        let dag = result.unwrap();
        let mut found_goto = false;
        for node in dag.nodes.iter() {
            if matches!(
                node.kind,
                AstKind::Statement(crate::models::unified_ast::StmtKind::Goto)
            ) {
                found_goto = true;
                break;
            }
        }
        assert!(found_goto, "Should have found a goto statement");
    }

    #[test]
    #[cfg(feature = "c-ast")]
    fn test_parse_c_with_restrict() {
        let mut parser = CAstParser::new();
        let content = r#"
void process_array(int * restrict a, int * restrict b, int n) {
    for (int i = 0; i < n; i++) {
        a[i] = b[i] * 2;
    }
}
"#;
        let result = parser.parse_file(Path::new("test.c"), content);
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(not(feature = "c-ast"))]
    fn test_c_ast_disabled() {
        let mut parser = CAstParser::new();
        let content = "int main() { return 0; }";
        let result = parser.parse_file(Path::new("test.c"), content);
        assert!(result.is_err());
    }

    #[test]
    fn test_compatibility_layer() {
        // Test that the parser can be created in both feature configurations
        let _parser = CAstParser::new();

        // Verify that default() works
        let _default_parser = CAstParser::default();

        // This should compile regardless of feature flags
        // Test passes if compilation succeeds
    }
}
