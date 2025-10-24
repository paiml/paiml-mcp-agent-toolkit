//! Unified AST node representation for cross-language analysis
//!
//! This module provides a language-agnostic representation of code elements
//! that can be used to represent and analyze code across different programming
//! languages. The `UnifiedNode` struct is the central component, representing
//! a single code element with standardized metadata.

use crate::ast::core::AstItem;
use crate::ast::polyglot::{Language, NodeKind};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};

/// A unified representation of a node in the polyglot AST
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedNode {
    /// Unique identifier for this node
    pub id: String,
    
    /// The kind of node (class, method, etc.)
    pub kind: NodeKind,
    
    /// The name of the node
    pub name: String,
    
    /// The fully qualified name (including package/namespace)
    pub fqn: String,
    
    /// The source language of this node
    pub language: Language,
    
    /// The file path where this node is defined
    pub file_path: PathBuf,
    
    /// Line and column position in source
    pub position: SourcePosition,
    
    /// Node attributes (modifiers, visibility, etc.)
    pub attributes: HashMap<String, String>,
    
    /// For container nodes, child nodes
    pub children: Vec<String>, // IDs of child nodes
    
    /// For class members, the parent class/struct
    pub parent: Option<String>, // ID of parent node
    
    /// References to other nodes (inheritance, implementation, etc.)
    pub references: Vec<NodeReference>,
    
    /// Type information
    pub type_info: Option<TypeInfo>,
    
    /// Signature for methods/functions
    pub signature: Option<String>,
    
    /// Documentation/comments
    pub documentation: Option<String>,
    
    /// Original AST item this was created from (optional)
    #[serde(skip_serializing, skip_deserializing)]
    pub original_item: Option<AstItem>,
    
    /// Language-specific metadata
    pub metadata: HashMap<String, String>,
}

/// Position in source code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourcePosition {
    /// Starting line (1-based)
    pub start_line: usize,
    /// Starting column (1-based)
    pub start_col: usize,
    /// Ending line (1-based)
    pub end_line: usize,
    /// Ending column (1-based)
    pub end_col: usize,
}

impl Default for SourcePosition {
    fn default() -> Self {
        Self {
            start_line: 0,
            start_col: 0,
            end_line: 0,
            end_col: 0,
        }
    }
}

/// A reference to another node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeReference {
    /// Type of reference (inherits, implements, calls, etc.)
    pub kind: ReferenceKind,
    
    /// Target node ID
    pub target_id: String,
    
    /// Target name (may be used before resolving to ID)
    pub target_name: String,
    
    /// Target language (may be different than source node)
    pub target_language: Option<Language>,
}

/// Type of reference between nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReferenceKind {
    /// Inheritance relationship (extends)
    Inherits,
    
    /// Implementation relationship (implements)
    Implements,
    
    /// Calls a method or function
    Calls,
    
    /// Uses a field, property or variable
    Uses,
    
    /// Creates an instance of a class
    Creates,
    
    /// Imports or requires
    Imports,
    
    /// Annotates or decorates
    Annotates,
    
    /// Generic dependency
    DependsOn,
}

/// Type information for a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeInfo {
    /// Base type name
    pub name: String,
    
    /// Fully qualified type name
    pub fqn: String,
    
    /// Type parameters (for generics)
    pub type_parameters: Vec<TypeInfo>,
    
    /// Is it a primitive type?
    pub is_primitive: bool,
    
    /// Is it a collection type?
    pub is_collection: bool,
    
    /// Is it nullable?
    pub is_nullable: bool,
    
    /// Original type string from source language
    pub original_type_string: String,
}

impl UnifiedNode {
    /// Create a new unified node from an AST item
    pub fn from_ast_item(
        item: &AstItem,
        language: Language,
        file_path: &Path,
        id_prefix: Option<&str>,
    ) -> Self {
        // Generate a unique ID
        let prefix = id_prefix.unwrap_or(language.name());
        let id = format!("{}:{}:{}", prefix, item.kind, item.name);
        
        // Extract position information
        let position = SourcePosition {
            start_line: item.span.start.line,
            start_col: item.span.start.column,
            end_line: item.span.end.line,
            end_col: item.span.end.column,
        };
        
        // Extract attributes from metadata
        let mut attributes = HashMap::new();
        if let Some(access) = &item.access {
            attributes.insert("access".to_string(), access.clone());
        }
        if let Some(modifiers) = &item.modifiers {
            for modifier in modifiers {
                attributes.insert(format!("modifier:{}", modifier), "true".to_string());
            }
        }
        
        // Create the FQN
        let fqn = if item.namespace.is_empty() {
            item.name.clone()
        } else {
            format!("{}.{}", item.namespace, item.name)
        };
        
        // Extract documentation
        let documentation = item.documentation.clone();
        
        UnifiedNode {
            id,
            kind: NodeKind::from_ast_item_kind(&item.kind),
            name: item.name.clone(),
            fqn,
            language,
            file_path: file_path.to_path_buf(),
            position,
            attributes,
            children: Vec::new(), // To be populated later
            parent: None,         // To be populated later
            references: Vec::new(), // To be populated later
            type_info: None,      // To be populated later
            signature: item.signature.clone(),
            documentation,
            original_item: Some(item.clone()),
            metadata: HashMap::new(),
        }
    }
    
    /// Get the access/visibility modifier of this node
    pub fn access(&self) -> Option<&str> {
        self.attributes.get("access").map(AsRef::as_ref)
    }
    
    /// Check if this node has a specific modifier
    pub fn has_modifier(&self, modifier: &str) -> bool {
        self.attributes.contains_key(&format!("modifier:{}", modifier))
    }
    
    /// Check if this node is abstract
    pub fn is_abstract(&self) -> bool {
        self.has_modifier("abstract")
    }
    
    /// Check if this node is static
    pub fn is_static(&self) -> bool {
        self.has_modifier("static")
    }
    
    /// Check if this node is final
    pub fn is_final(&self) -> bool {
        self.has_modifier("final")
    }
    
    /// Add a child node
    pub fn add_child(&mut self, child_id: String) {
        self.children.push(child_id);
    }
    
    /// Set the parent node
    pub fn set_parent(&mut self, parent_id: String) {
        self.parent = Some(parent_id);
    }
    
    /// Add a reference to another node
    pub fn add_reference(&mut self, kind: ReferenceKind, target_name: String, target_id: Option<String>) {
        let reference = NodeReference {
            kind,
            target_id: target_id.unwrap_or_else(|| "".to_string()),
            target_name,
            target_language: None, // To be resolved later
        };
        self.references.push(reference);
    }
    
    /// Get all references of a specific kind
    pub fn get_references_by_kind(&self, kind: ReferenceKind) -> Vec<&NodeReference> {
        self.references
            .iter()
            .filter(|r| r.kind == kind)
            .collect()
    }
    
    /// Set type information
    pub fn set_type_info(&mut self, type_info: TypeInfo) {
        self.type_info = Some(type_info);
    }
    
    /// Add language-specific metadata
    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::core::{AstItem, ItemKind, Span, Position};
    
    fn create_test_ast_item() -> AstItem {
        AstItem {
            id: 1,
            kind: "class".into(),
            name: "TestClass".to_string(),
            namespace: "com.example".to_string(),
            signature: Some("public class TestClass".to_string()),
            content: "class TestClass {}".to_string(),
            complexity: 1,
            span: Span {
                start: Position { line: 10, column: 1 },
                end: Position { line: 20, column: 1 },
            },
            access: Some("public".to_string()),
            modifiers: Some(vec!["final".to_string()]),
            documentation: Some("Test class documentation".to_string()),
            children: Vec::new(),
            references: Vec::new(),
        }
    }
    
    #[test]
    fn test_unified_node_from_ast_item() {
        let ast_item = create_test_ast_item();
        let file_path = Path::new("/path/to/TestClass.java");
        
        let node = UnifiedNode::from_ast_item(&ast_item, Language::Java, file_path, None);
        
        assert_eq!(node.id, "Java:class:TestClass");
        assert_eq!(node.kind, NodeKind::Class);
        assert_eq!(node.name, "TestClass");
        assert_eq!(node.fqn, "com.example.TestClass");
        assert_eq!(node.language, Language::Java);
        assert_eq!(node.position.start_line, 10);
        assert_eq!(node.position.end_line, 20);
        assert_eq!(node.access(), Some("public"));
        assert!(node.has_modifier("final"));
        assert!(!node.has_modifier("abstract"));
        assert_eq!(node.documentation.as_deref(), Some("Test class documentation"));
    }
    
    #[test]
    fn test_add_reference() {
        let ast_item = create_test_ast_item();
        let file_path = Path::new("/path/to/TestClass.java");
        
        let mut node = UnifiedNode::from_ast_item(&ast_item, Language::Java, file_path, None);
        node.add_reference(ReferenceKind::Inherits, "BaseClass".to_string(), Some("Java:class:BaseClass".to_string()));
        
        assert_eq!(node.references.len(), 1);
        assert_eq!(node.references[0].kind, ReferenceKind::Inherits);
        assert_eq!(node.references[0].target_name, "BaseClass");
        assert_eq!(node.references[0].target_id, "Java:class:BaseClass");
        
        let inherits_refs = node.get_references_by_kind(ReferenceKind::Inherits);
        assert_eq!(inherits_refs.len(), 1);
        assert_eq!(inherits_refs[0].target_name, "BaseClass");
    }
}