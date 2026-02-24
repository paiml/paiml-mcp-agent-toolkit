#![cfg_attr(coverage_nightly, coverage(off))]
//! Unified AST node representation for cross-language analysis
//!
//! This module provides a language-agnostic representation of code elements
//! that can be used to represent and analyze code across different programming
//! languages. The `UnifiedNode` struct is the central component, representing
//! a single code element with standardized metadata.

use crate::ast::polyglot::{Language, NodeKind};
use crate::services::context::AstItem;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// Struct and enum definitions (UnifiedNode, SourcePosition, NodeReference, ReferenceKind, TypeInfo)
include!("unified_node_types.rs");

// impl UnifiedNode methods
include!("unified_node_impl.rs");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::context::AstItem;

    fn create_test_ast_item() -> AstItem {
        // Create a simple Struct variant for testing
        AstItem::Struct {
            name: "TestClass".to_string(),
            visibility: "public".to_string(),
            fields_count: 0,
            derives: vec![],
            line: 10,
        }
    }

    #[test]
    fn test_unified_node_from_ast_item() {
        let ast_item = create_test_ast_item();
        let file_path = Path::new("/path/to/TestClass.java");

        let node = UnifiedNode::from_ast_item(&ast_item, Language::Java, file_path, None);

        assert_eq!(node.id, "Java:struct:TestClass"); // Java classes are represented as Struct in AstItem
        assert_eq!(node.kind, NodeKind::Struct);
        assert_eq!(node.name, "TestClass");
        assert_eq!(node.fqn, "TestClass"); // FQN extraction not implemented, defaults to name
        assert_eq!(node.language, Language::Java);
        assert_eq!(node.position.start_line, 10);
        assert_eq!(node.position.end_line, 11); // AstItem::Struct doesn't have end_line, defaults to start_line + 1
        assert_eq!(node.access(), Some("public"));
        // Note: AstItem::Struct doesn't contain modifiers or documentation
        // These would need to be added to AstItem for full fidelity
    }

    #[test]
    fn test_add_reference() {
        let ast_item = create_test_ast_item();
        let file_path = Path::new("/path/to/TestClass.java");

        let mut node = UnifiedNode::from_ast_item(&ast_item, Language::Java, file_path, None);
        node.add_reference(
            ReferenceKind::Inherits,
            "BaseClass".to_string(),
            Some("Java:class:BaseClass".to_string()),
        );

        assert_eq!(node.references.len(), 1);
        assert_eq!(node.references[0].kind, ReferenceKind::Inherits);
        assert_eq!(node.references[0].target_name, "BaseClass");
        assert_eq!(node.references[0].target_id, "Java:class:BaseClass");

        let inherits_refs = node.get_references_by_kind(ReferenceKind::Inherits);
        assert_eq!(inherits_refs.len(), 1);
        assert_eq!(inherits_refs[0].target_name, "BaseClass");
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use crate::services::context::AstItem;

    include!("unified_node_coverage_tests_part1.rs");
    include!("unified_node_coverage_tests_part2.rs");
}
