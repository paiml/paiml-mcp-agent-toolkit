//! Unified AST node representation for cross-language analysis
//!
//! This module provides a language-agnostic representation of code elements
//! that can be used to represent and analyze code across different programming
//! languages. The `UnifiedNode` struct is the central component, representing
//! a single code element with standardized metadata.

use crate::services::context::AstItem;
use crate::ast::polyglot::{Language, NodeKind};
use std::collections::HashMap;
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
#[derive(Default)]
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
    /// Create a new basic unified node with minimal information
    pub fn new(kind: NodeKind, name: &str, language: Language) -> Self {
        let id = format!("{}:{}:{}", language.name(), kind.as_str(), name);
        Self {
            id,
            kind,
            name: name.to_string(),
            fqn: name.to_string(),
            language,
            file_path: std::path::PathBuf::new(),
            position: SourcePosition::default(),
            attributes: HashMap::new(),
            children: Vec::new(),
            parent: None,
            references: Vec::new(),
            type_info: None,
            signature: None,
            documentation: None,
            original_item: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Create a new unified node from an AST item
    pub fn from_ast_item(
        item: &AstItem,
        language: Language,
        file_path: &Path,
        id_prefix: Option<&str>,
    ) -> Self {
        // Generate a unique ID
        let prefix = id_prefix.unwrap_or(language.name());
        
        // Extract name based on AstItem type
        let (name, line, visibility, namespace) = match item {
            AstItem::Function { name, line, visibility, .. } =>
                (name.clone(), *line, visibility.clone(), None::<String>),
            AstItem::Struct { name, line, visibility, .. } =>
                (name.clone(), *line, visibility.clone(), None::<String>),
            AstItem::Enum { name, line, visibility, .. } =>
                (name.clone(), *line, visibility.clone(), None::<String>),
            AstItem::Trait { name, line, visibility, .. } =>
                (name.clone(), *line, visibility.clone(), None::<String>),
            AstItem::Module { name, line, visibility, .. } =>
                (name.clone(), *line, visibility.clone(), None::<String>),
            AstItem::Use { path, line } =>
                (path.clone(), *line, "public".to_string(), None::<String>),
            AstItem::Impl { type_name, line, .. } =>
                (type_name.clone(), *line, "public".to_string(), None::<String>),
            AstItem::Import { module, line, .. } =>
                (module.clone(), *line, "public".to_string(), None::<String>),
        };
        
        let kind = NodeKind::from_ast_item(item);
        let id = format!("{}:{}:{}", prefix, kind.as_str(), name);
        
        // Create position info
        let position = SourcePosition {
            start_line: line,
            start_col: 0,
            end_line: line + 1, // Approximate
            end_col: 0,
        };
        
        // Extract attributes
        let mut attributes = HashMap::new();
        attributes.insert("access".to_string(), visibility);
        
        // Get special attributes from item type
        match item {
            AstItem::Function { is_async, .. } => {
                if *is_async {
                    attributes.insert("modifier:async".to_string(), "true".to_string());
                }
            },
            AstItem::Struct { derives, .. } => {
                for derive in derives {
                    attributes.insert(format!("derive:{}", derive), "true".to_string());
                }
            },
            _ => {}
        }
        
        // Create the FQN with namespace if available
        let fqn = if let Some(ns) = namespace {
            if !ns.is_empty() {
                format!("{}.{}", ns, name)
            } else {
                name.clone()
            }
        } else {
            match item {
                AstItem::Struct { name, .. } | AstItem::Enum { name, .. } |
                AstItem::Trait { name, .. } => {
                    // For top-level types, use file path + name
                    let file_name = file_path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("");
                    if file_name != name {
                        format!("{}.{}", file_name, name)
                    } else {
                        name.clone()
                    }
                },
                _ => name.clone()
            }
        };
        
        UnifiedNode {
            id,
            kind,
            name,
            fqn,
            language,
            file_path: file_path.to_path_buf(),
            position,
            attributes,
            children: Vec::new(), // To be populated later
            parent: None,         // To be populated later
            references: Vec::new(), // To be populated later
            type_info: None,      // To be populated later
            signature: None,      // We'll need to extract this separately
            documentation: None,  // We'll need to extract this separately
            original_item: Some(item.clone()),
            metadata: HashMap::new(),
        }
    }
    
    // Helper to extract name from any AstItem
    pub fn extract_name_from_item(item: &AstItem) -> String {
        match item {
            AstItem::Function { name, .. } => name.clone(),
            AstItem::Struct { name, .. } => name.clone(),
            AstItem::Enum { name, .. } => name.clone(),
            AstItem::Trait { name, .. } => name.clone(),
            AstItem::Impl { type_name, .. } => type_name.clone(),
            AstItem::Use { path, .. } => path.clone(),
            AstItem::Module { name, .. } => name.clone(),
            AstItem::Import { module, .. } => module.clone(),
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
            target_id: target_id.unwrap_or_default(),
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