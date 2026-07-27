// Tests for AST core
// Extracted for file health compliance (CB-040)
// Split into include files for maintainability (PMAT-503)

use super::*;

mod tests {
    use super::*;

    #[test]
    fn test_node_size() {
        // Ensure our node structure is within expected bounds
        // With proof annotations, the size is larger than the original 64 bytes
        let size = std::mem::size_of::<UnifiedAstNode>();
        assert!(
            size <= 128,
            "Node size {size} exceeds maximum expected size of 128 bytes"
        );
        // Structure should be at least 64 bytes for the core data
        assert!(
            size >= 64,
            "Node size {size} is smaller than minimum expected size of 64 bytes"
        );
    }

    #[test]
    fn test_node_alignment() {
        // Ensure proper alignment for SIMD operations
        assert_eq!(std::mem::align_of::<UnifiedAstNode>(), 32);
    }

    #[test]
    fn test_node_flags() {
        let mut flags = NodeFlags::new();

        flags.set(NodeFlags::ASYNC);
        flags.set(NodeFlags::EXPORTED);

        assert!(flags.has(NodeFlags::ASYNC));
        assert!(flags.has(NodeFlags::EXPORTED));
        assert!(!flags.has(NodeFlags::PRIVATE));

        flags.unset(NodeFlags::ASYNC);
        assert!(!flags.has(NodeFlags::ASYNC));
    }

    #[test]
    fn test_ast_dag() {
        let mut dag = AstDag::new();

        let node = UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);

        let key = dag.add_node(node);

        assert_eq!(dag.nodes.len(), 1);
        assert!(dag.dirty_nodes.contains(key));

        dag.mark_clean(key);
        assert!(!dag.dirty_nodes.contains(key));
    }
}

/// Comprehensive EXTREME TDD tests for ast/core module
/// Tests all public structs, enums, and functions with edge cases
mod coverage_tests {
    use super::*;
    use chrono::Utc;
    use proptest::prelude::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::path::PathBuf;

    // Language, NodeFlags, AstKind, FunctionKind, VarKind, ImportKind,
    // ExprKind, StmtKind, TypeKind, ModuleKind, MacroKind tests
    include!("core_tests_enums_flags.rs");

    // ConfidenceLevel, PropertyType, VerificationMethod, EvidenceType tests
    include!("core_tests_type_system.rs");

    // BytePos, Span, Location, QualifiedName tests
    include!("core_tests_spatial.rs");

    // RelativeLocation, NodeMetadata, ProofAnnotation, UnifiedAstNode,
    // ColumnStore, AstDag, INVALID_NODE_KEY, ClassKind tests
    include!("core_tests_nodes_dag.rs");

    // Property-based proptest! tests and edge case tests
    include!("core_tests_properties.rs");
}
