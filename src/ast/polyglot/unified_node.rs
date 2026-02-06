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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
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
            AstItem::Function {
                name,
                line,
                visibility,
                ..
            } => (name.clone(), *line, visibility.clone(), None::<String>),
            AstItem::Struct {
                name,
                line,
                visibility,
                ..
            } => (name.clone(), *line, visibility.clone(), None::<String>),
            AstItem::Enum {
                name,
                line,
                visibility,
                ..
            } => (name.clone(), *line, visibility.clone(), None::<String>),
            AstItem::Trait {
                name,
                line,
                visibility,
                ..
            } => (name.clone(), *line, visibility.clone(), None::<String>),
            AstItem::Module {
                name,
                line,
                visibility,
                ..
            } => (name.clone(), *line, visibility.clone(), None::<String>),
            AstItem::Use { path, line } => {
                (path.clone(), *line, "public".to_string(), None::<String>)
            }
            AstItem::Impl {
                type_name, line, ..
            } => (
                type_name.clone(),
                *line,
                "public".to_string(),
                None::<String>,
            ),
            AstItem::Import { module, line, .. } => {
                (module.clone(), *line, "public".to_string(), None::<String>)
            }
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
            }
            AstItem::Struct { derives, .. } => {
                for derive in derives {
                    attributes.insert(format!("derive:{}", derive), "true".to_string());
                }
            }
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
                AstItem::Struct { name, .. }
                | AstItem::Enum { name, .. }
                | AstItem::Trait { name, .. } => {
                    // For top-level types, use file path + name
                    let file_name = file_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                    if file_name != name {
                        format!("{}.{}", file_name, name)
                    } else {
                        name.clone()
                    }
                }
                _ => name.clone(),
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
            children: Vec::new(),   // To be populated later
            parent: None,           // To be populated later
            references: Vec::new(), // To be populated later
            type_info: None,        // To be populated later
            signature: None,        // We'll need to extract this separately
            documentation: None,    // We'll need to extract this separately
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
        self.attributes
            .contains_key(&format!("modifier:{}", modifier))
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
    pub fn add_reference(
        &mut self,
        kind: ReferenceKind,
        target_name: String,
        target_id: Option<String>,
    ) {
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
        self.references.iter().filter(|r| r.kind == kind).collect()
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

#[cfg_attr(coverage_nightly, coverage(off))]
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

    // ==========================================================================
    // UnifiedNode::new tests
    // ==========================================================================

    #[test]
    fn test_unified_node_new_basic() {
        let node = UnifiedNode::new(NodeKind::Function, "my_function", Language::Rust);

        assert_eq!(node.id, "Rust:function:my_function");
        assert_eq!(node.kind, NodeKind::Function);
        assert_eq!(node.name, "my_function");
        assert_eq!(node.fqn, "my_function");
        assert_eq!(node.language, Language::Rust);
        assert!(node.file_path.as_os_str().is_empty());
        assert_eq!(node.position.start_line, 0);
        assert_eq!(node.position.start_col, 0);
        assert!(node.attributes.is_empty());
        assert!(node.children.is_empty());
        assert!(node.parent.is_none());
        assert!(node.references.is_empty());
        assert!(node.type_info.is_none());
        assert!(node.signature.is_none());
        assert!(node.documentation.is_none());
        assert!(node.original_item.is_none());
        assert!(node.metadata.is_empty());
    }

    #[test]
    fn test_unified_node_new_all_kinds() {
        let test_cases = vec![
            (NodeKind::Class, "MyClass", Language::Java),
            (NodeKind::Function, "my_func", Language::Python),
            (NodeKind::Method, "do_something", Language::TypeScript),
            (NodeKind::Struct, "DataStruct", Language::Rust),
            (NodeKind::Enum, "Status", Language::Go),
            (NodeKind::Interface, "IService", Language::CSharp),
            (NodeKind::Trait, "MyTrait", Language::Scala),
            (NodeKind::Module, "my_mod", Language::Ruby),
            (NodeKind::LanguageSpecific(42), "custom", Language::Other(1)),
        ];

        for (kind, name, language) in test_cases {
            let node = UnifiedNode::new(kind, name, language);
            assert_eq!(node.kind, kind);
            assert_eq!(node.name, name);
            assert_eq!(node.language, language);
        }
    }

    // ==========================================================================
    // UnifiedNode::from_ast_item tests
    // ==========================================================================

    #[test]
    fn test_from_ast_item_function() {
        let item = AstItem::Function {
            name: "process_data".to_string(),
            visibility: "pub".to_string(),
            is_async: true,
            line: 42,
        };
        let path = Path::new("/src/lib.rs");

        let node = UnifiedNode::from_ast_item(&item, Language::Rust, path, None);

        assert_eq!(node.kind, NodeKind::Function);
        assert_eq!(node.name, "process_data");
        assert_eq!(node.position.start_line, 42);
        assert_eq!(node.access(), Some("pub"));
        assert!(node.has_modifier("async"));
    }

    #[test]
    fn test_from_ast_item_function_sync() {
        let item = AstItem::Function {
            name: "sync_func".to_string(),
            visibility: "private".to_string(),
            is_async: false,
            line: 10,
        };
        let path = Path::new("/src/main.rs");

        let node = UnifiedNode::from_ast_item(&item, Language::Rust, path, None);

        assert!(!node.has_modifier("async"));
    }

    #[test]
    fn test_from_ast_item_struct_with_derives() {
        let item = AstItem::Struct {
            name: "MyStruct".to_string(),
            visibility: "pub".to_string(),
            fields_count: 5,
            derives: vec![
                "Debug".to_string(),
                "Clone".to_string(),
                "Serialize".to_string(),
            ],
            line: 1,
        };
        let path = Path::new("/src/models.rs");

        let node = UnifiedNode::from_ast_item(&item, Language::Rust, path, None);

        assert_eq!(node.kind, NodeKind::Struct);
        assert!(node.attributes.contains_key("derive:Debug"));
        assert!(node.attributes.contains_key("derive:Clone"));
        assert!(node.attributes.contains_key("derive:Serialize"));
    }

    #[test]
    fn test_from_ast_item_struct_same_as_file() {
        let item = AstItem::Struct {
            name: "MyModule".to_string(),
            visibility: "pub".to_string(),
            fields_count: 0,
            derives: vec![],
            line: 1,
        };
        // When struct name matches file name, FQN should just be the name
        let path = Path::new("/src/MyModule.rs");

        let node = UnifiedNode::from_ast_item(&item, Language::Rust, path, None);

        assert_eq!(node.fqn, "MyModule");
    }

    #[test]
    fn test_from_ast_item_struct_different_from_file() {
        let item = AstItem::Struct {
            name: "InnerStruct".to_string(),
            visibility: "pub".to_string(),
            fields_count: 0,
            derives: vec![],
            line: 1,
        };
        let path = Path::new("/src/outer.rs");

        let node = UnifiedNode::from_ast_item(&item, Language::Rust, path, None);

        // FQN should include file name prefix
        assert_eq!(node.fqn, "outer.InnerStruct");
    }

    #[test]
    fn test_from_ast_item_enum() {
        let item = AstItem::Enum {
            name: "Status".to_string(),
            visibility: "pub(crate)".to_string(),
            variants_count: 3,
            line: 20,
        };
        let path = Path::new("/src/types.rs");

        let node = UnifiedNode::from_ast_item(&item, Language::Rust, path, None);

        assert_eq!(node.kind, NodeKind::Enum);
        assert_eq!(node.name, "Status");
        assert_eq!(node.access(), Some("pub(crate)"));
        assert_eq!(node.fqn, "types.Status");
    }

    #[test]
    fn test_from_ast_item_trait() {
        let item = AstItem::Trait {
            name: "Processor".to_string(),
            visibility: "pub".to_string(),
            line: 5,
        };
        let path = Path::new("/src/traits.rs");

        let node = UnifiedNode::from_ast_item(&item, Language::Rust, path, None);

        assert_eq!(node.kind, NodeKind::Trait);
        assert_eq!(node.name, "Processor");
        assert_eq!(node.fqn, "traits.Processor");
    }

    #[test]
    fn test_from_ast_item_impl() {
        let item = AstItem::Impl {
            type_name: "MyStruct".to_string(),
            trait_name: Some("Display".to_string()),
            line: 15,
        };
        let path = Path::new("/src/impls.rs");

        let node = UnifiedNode::from_ast_item(&item, Language::Rust, path, None);

        assert_eq!(node.kind, NodeKind::Implements);
        assert_eq!(node.name, "MyStruct");
        assert_eq!(node.access(), Some("public")); // Impls default to public
    }

    #[test]
    fn test_from_ast_item_impl_no_trait() {
        let item = AstItem::Impl {
            type_name: "MyStruct".to_string(),
            trait_name: None,
            line: 15,
        };
        let path = Path::new("/src/impls.rs");

        let node = UnifiedNode::from_ast_item(&item, Language::Rust, path, None);

        assert_eq!(node.kind, NodeKind::Implements);
        assert_eq!(node.name, "MyStruct");
    }

    #[test]
    fn test_from_ast_item_module() {
        let item = AstItem::Module {
            name: "submodule".to_string(),
            visibility: "pub(super)".to_string(),
            line: 1,
        };
        let path = Path::new("/src/lib.rs");

        let node = UnifiedNode::from_ast_item(&item, Language::Rust, path, None);

        assert_eq!(node.kind, NodeKind::Module);
        assert_eq!(node.name, "submodule");
        assert_eq!(node.access(), Some("pub(super)"));
    }

    #[test]
    fn test_from_ast_item_use() {
        let item = AstItem::Use {
            path: "std::collections::HashMap".to_string(),
            line: 3,
        };
        let path = Path::new("/src/main.rs");

        let node = UnifiedNode::from_ast_item(&item, Language::Rust, path, None);

        assert_eq!(node.kind, NodeKind::Uses);
        assert_eq!(node.name, "std::collections::HashMap");
        assert_eq!(node.access(), Some("public")); // Use defaults to public
    }

    #[test]
    fn test_from_ast_item_import() {
        let item = AstItem::Import {
            module: "external_lib".to_string(),
            items: vec![],
            alias: Some("ext".to_string()),
            line: 1,
        };
        let path = Path::new("/src/main.py");

        let node = UnifiedNode::from_ast_item(&item, Language::Python, path, None);

        assert_eq!(node.kind, NodeKind::Import);
        assert_eq!(node.name, "external_lib");
    }

    #[test]
    fn test_from_ast_item_import_no_alias() {
        let item = AstItem::Import {
            module: "json".to_string(),
            items: vec![],
            alias: None,
            line: 1,
        };
        let path = Path::new("/src/main.py");

        let node = UnifiedNode::from_ast_item(&item, Language::Python, path, None);

        assert_eq!(node.kind, NodeKind::Import);
        assert_eq!(node.name, "json");
    }

    #[test]
    fn test_from_ast_item_with_custom_prefix() {
        let item = AstItem::Function {
            name: "test_fn".to_string(),
            visibility: "pub".to_string(),
            is_async: false,
            line: 1,
        };
        let path = Path::new("/src/lib.rs");

        let node = UnifiedNode::from_ast_item(&item, Language::Rust, path, Some("custom_prefix"));

        assert!(node.id.starts_with("custom_prefix:"));
    }

    // ==========================================================================
    // extract_name_from_item tests
    // ==========================================================================

    #[test]
    fn test_extract_name_from_all_item_types() {
        let test_cases = vec![
            (
                AstItem::Function {
                    name: "func_name".to_string(),
                    visibility: "pub".to_string(),
                    is_async: false,
                    line: 1,
                },
                "func_name",
            ),
            (
                AstItem::Struct {
                    name: "StructName".to_string(),
                    visibility: "pub".to_string(),
                    fields_count: 0,
                    derives: vec![],
                    line: 1,
                },
                "StructName",
            ),
            (
                AstItem::Enum {
                    name: "EnumName".to_string(),
                    visibility: "pub".to_string(),
                    variants_count: 0,
                    line: 1,
                },
                "EnumName",
            ),
            (
                AstItem::Trait {
                    name: "TraitName".to_string(),
                    visibility: "pub".to_string(),
                    line: 1,
                },
                "TraitName",
            ),
            (
                AstItem::Impl {
                    type_name: "TypeName".to_string(),
                    trait_name: None,
                    line: 1,
                },
                "TypeName",
            ),
            (
                AstItem::Use {
                    path: "std::io".to_string(),
                    line: 1,
                },
                "std::io",
            ),
            (
                AstItem::Module {
                    name: "mod_name".to_string(),
                    visibility: "pub".to_string(),
                    line: 1,
                },
                "mod_name",
            ),
            (
                AstItem::Import {
                    module: "import_name".to_string(),
                    items: vec![],
                    alias: None,
                    line: 1,
                },
                "import_name",
            ),
        ];

        for (item, expected_name) in test_cases {
            assert_eq!(
                UnifiedNode::extract_name_from_item(&item),
                expected_name,
                "Failed for item: {:?}",
                item
            );
        }
    }

    // ==========================================================================
    // Modifier tests
    // ==========================================================================

    #[test]
    fn test_has_modifier() {
        let mut node = UnifiedNode::new(NodeKind::Function, "test", Language::Java);
        node.attributes
            .insert("modifier:abstract".to_string(), "true".to_string());
        node.attributes
            .insert("modifier:static".to_string(), "true".to_string());
        node.attributes
            .insert("modifier:final".to_string(), "true".to_string());

        assert!(node.has_modifier("abstract"));
        assert!(node.has_modifier("static"));
        assert!(node.has_modifier("final"));
        assert!(!node.has_modifier("synchronized"));
    }

    #[test]
    fn test_is_abstract() {
        let mut node = UnifiedNode::new(NodeKind::Class, "AbstractClass", Language::Java);
        assert!(!node.is_abstract());

        node.attributes
            .insert("modifier:abstract".to_string(), "true".to_string());
        assert!(node.is_abstract());
    }

    #[test]
    fn test_is_static() {
        let mut node = UnifiedNode::new(NodeKind::Method, "staticMethod", Language::Java);
        assert!(!node.is_static());

        node.attributes
            .insert("modifier:static".to_string(), "true".to_string());
        assert!(node.is_static());
    }

    #[test]
    fn test_is_final() {
        let mut node = UnifiedNode::new(NodeKind::Class, "FinalClass", Language::Java);
        assert!(!node.is_final());

        node.attributes
            .insert("modifier:final".to_string(), "true".to_string());
        assert!(node.is_final());
    }

    #[test]
    fn test_access_none() {
        let node = UnifiedNode::new(NodeKind::Function, "test", Language::Rust);
        assert!(node.access().is_none());
    }

    #[test]
    fn test_access_some() {
        let mut node = UnifiedNode::new(NodeKind::Function, "test", Language::Rust);
        node.attributes
            .insert("access".to_string(), "private".to_string());
        assert_eq!(node.access(), Some("private"));
    }

    // ==========================================================================
    // Child/Parent relationship tests
    // ==========================================================================

    #[test]
    fn test_add_child() {
        let mut node = UnifiedNode::new(NodeKind::Class, "Parent", Language::Java);
        assert!(node.children.is_empty());

        node.add_child("child1".to_string());
        assert_eq!(node.children.len(), 1);
        assert_eq!(node.children[0], "child1");

        node.add_child("child2".to_string());
        node.add_child("child3".to_string());
        assert_eq!(node.children.len(), 3);
    }

    #[test]
    fn test_set_parent() {
        let mut node = UnifiedNode::new(NodeKind::Method, "child", Language::Java);
        assert!(node.parent.is_none());

        node.set_parent("parent_id".to_string());
        assert_eq!(node.parent, Some("parent_id".to_string()));

        // Setting again should overwrite
        node.set_parent("new_parent".to_string());
        assert_eq!(node.parent, Some("new_parent".to_string()));
    }

    // ==========================================================================
    // Reference tests
    // ==========================================================================

    #[test]
    fn test_add_reference_all_kinds() {
        let mut node = UnifiedNode::new(NodeKind::Class, "Test", Language::Java);

        let reference_kinds = vec![
            ReferenceKind::Inherits,
            ReferenceKind::Implements,
            ReferenceKind::Calls,
            ReferenceKind::Uses,
            ReferenceKind::Creates,
            ReferenceKind::Imports,
            ReferenceKind::Annotates,
            ReferenceKind::DependsOn,
        ];

        for (i, kind) in reference_kinds.iter().enumerate() {
            node.add_reference(*kind, format!("target_{}", i), Some(format!("id_{}", i)));
        }

        assert_eq!(node.references.len(), 8);

        // Test get_references_by_kind
        assert_eq!(
            node.get_references_by_kind(ReferenceKind::Inherits).len(),
            1
        );
        assert_eq!(
            node.get_references_by_kind(ReferenceKind::Implements).len(),
            1
        );
        assert_eq!(node.get_references_by_kind(ReferenceKind::Calls).len(), 1);
    }

    #[test]
    fn test_add_reference_without_target_id() {
        let mut node = UnifiedNode::new(NodeKind::Class, "Test", Language::Java);
        node.add_reference(ReferenceKind::Inherits, "BaseClass".to_string(), None);

        assert_eq!(node.references.len(), 1);
        assert_eq!(node.references[0].target_id, "");
        assert_eq!(node.references[0].target_name, "BaseClass");
        assert!(node.references[0].target_language.is_none());
    }

    #[test]
    fn test_get_references_by_kind_empty() {
        let node = UnifiedNode::new(NodeKind::Class, "Test", Language::Java);
        assert!(node
            .get_references_by_kind(ReferenceKind::Inherits)
            .is_empty());
    }

    #[test]
    fn test_get_references_by_kind_multiple() {
        let mut node = UnifiedNode::new(NodeKind::Class, "Test", Language::Java);
        node.add_reference(ReferenceKind::Implements, "Interface1".to_string(), None);
        node.add_reference(ReferenceKind::Implements, "Interface2".to_string(), None);
        node.add_reference(ReferenceKind::Implements, "Interface3".to_string(), None);
        node.add_reference(ReferenceKind::Inherits, "BaseClass".to_string(), None);

        let implements = node.get_references_by_kind(ReferenceKind::Implements);
        assert_eq!(implements.len(), 3);

        let inherits = node.get_references_by_kind(ReferenceKind::Inherits);
        assert_eq!(inherits.len(), 1);
    }

    // ==========================================================================
    // TypeInfo tests
    // ==========================================================================

    #[test]
    fn test_set_type_info() {
        let mut node = UnifiedNode::new(NodeKind::Field, "myField", Language::Java);
        assert!(node.type_info.is_none());

        let type_info = TypeInfo {
            name: "String".to_string(),
            fqn: "java.lang.String".to_string(),
            type_parameters: vec![],
            is_primitive: false,
            is_collection: false,
            is_nullable: true,
            original_type_string: "String?".to_string(),
        };

        node.set_type_info(type_info.clone());

        assert!(node.type_info.is_some());
        let ti = node.type_info.as_ref().unwrap();
        assert_eq!(ti.name, "String");
        assert_eq!(ti.fqn, "java.lang.String");
        assert!(!ti.is_primitive);
        assert!(ti.is_nullable);
    }

    #[test]
    fn test_type_info_with_generics() {
        let inner_type = TypeInfo {
            name: "String".to_string(),
            fqn: "java.lang.String".to_string(),
            type_parameters: vec![],
            is_primitive: false,
            is_collection: false,
            is_nullable: false,
            original_type_string: "String".to_string(),
        };

        let type_info = TypeInfo {
            name: "List".to_string(),
            fqn: "java.util.List".to_string(),
            type_parameters: vec![inner_type],
            is_primitive: false,
            is_collection: true,
            is_nullable: false,
            original_type_string: "List<String>".to_string(),
        };

        assert!(type_info.is_collection);
        assert_eq!(type_info.type_parameters.len(), 1);
        assert_eq!(type_info.type_parameters[0].name, "String");
    }

    // ==========================================================================
    // Metadata tests
    // ==========================================================================

    #[test]
    fn test_add_metadata() {
        let mut node = UnifiedNode::new(NodeKind::Function, "test", Language::Python);
        assert!(node.metadata.is_empty());

        node.add_metadata("decorator", "@pytest.fixture");
        node.add_metadata("docstring", "Test function");

        assert_eq!(node.metadata.len(), 2);
        assert_eq!(
            node.metadata.get("decorator"),
            Some(&"@pytest.fixture".to_string())
        );
        assert_eq!(
            node.metadata.get("docstring"),
            Some(&"Test function".to_string())
        );
    }

    #[test]
    fn test_add_metadata_overwrite() {
        let mut node = UnifiedNode::new(NodeKind::Function, "test", Language::Python);

        node.add_metadata("key", "value1");
        assert_eq!(node.metadata.get("key"), Some(&"value1".to_string()));

        node.add_metadata("key", "value2");
        assert_eq!(node.metadata.get("key"), Some(&"value2".to_string()));
    }

    // ==========================================================================
    // SourcePosition tests
    // ==========================================================================

    #[test]
    fn test_source_position_default() {
        let pos = SourcePosition::default();
        assert_eq!(pos.start_line, 0);
        assert_eq!(pos.start_col, 0);
        assert_eq!(pos.end_line, 0);
        assert_eq!(pos.end_col, 0);
    }

    #[test]
    fn test_source_position_clone() {
        let pos = SourcePosition {
            start_line: 10,
            start_col: 5,
            end_line: 15,
            end_col: 20,
        };
        let cloned = pos.clone();
        assert_eq!(pos.start_line, cloned.start_line);
        assert_eq!(pos.start_col, cloned.start_col);
        assert_eq!(pos.end_line, cloned.end_line);
        assert_eq!(pos.end_col, cloned.end_col);
    }

    // ==========================================================================
    // ReferenceKind tests
    // ==========================================================================

    #[test]
    fn test_reference_kind_ordering() {
        // ReferenceKind derives PartialOrd and Ord
        assert!(ReferenceKind::Inherits < ReferenceKind::Implements);
        assert!(ReferenceKind::Implements < ReferenceKind::Calls);

        let mut kinds = vec![
            ReferenceKind::DependsOn,
            ReferenceKind::Inherits,
            ReferenceKind::Calls,
        ];
        kinds.sort();
        assert_eq!(kinds[0], ReferenceKind::Inherits);
    }

    #[test]
    fn test_reference_kind_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(ReferenceKind::Inherits);
        set.insert(ReferenceKind::Implements);
        set.insert(ReferenceKind::Inherits); // Duplicate

        assert_eq!(set.len(), 2);
    }

    // ==========================================================================
    // Serialization tests
    // ==========================================================================

    #[test]
    fn test_unified_node_serialize() {
        let node = UnifiedNode::new(NodeKind::Function, "test_func", Language::Rust);
        let json = serde_json::to_string(&node).unwrap();

        assert!(json.contains("test_func"));
        assert!(json.contains("function"));
        assert!(json.contains("Rust"));
    }

    #[test]
    fn test_unified_node_deserialize() {
        let node = UnifiedNode::new(NodeKind::Function, "test_func", Language::Rust);
        let json = serde_json::to_string(&node).unwrap();
        let deserialized: UnifiedNode = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "test_func");
        assert_eq!(deserialized.kind, NodeKind::Function);
        assert_eq!(deserialized.language, Language::Rust);
    }

    #[test]
    fn test_node_reference_serialize() {
        let reference = NodeReference {
            kind: ReferenceKind::Inherits,
            target_id: "id_123".to_string(),
            target_name: "BaseClass".to_string(),
            target_language: Some(Language::Java),
        };

        let json = serde_json::to_string(&reference).unwrap();
        let deserialized: NodeReference = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.kind, ReferenceKind::Inherits);
        assert_eq!(deserialized.target_id, "id_123");
        assert_eq!(deserialized.target_language, Some(Language::Java));
    }

    #[test]
    fn test_type_info_serialize() {
        let type_info = TypeInfo {
            name: "HashMap".to_string(),
            fqn: "std::collections::HashMap".to_string(),
            type_parameters: vec![],
            is_primitive: false,
            is_collection: true,
            is_nullable: false,
            original_type_string: "HashMap<K, V>".to_string(),
        };

        let json = serde_json::to_string(&type_info).unwrap();
        let deserialized: TypeInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "HashMap");
        assert!(deserialized.is_collection);
    }

    // ==========================================================================
    // Clone tests
    // ==========================================================================

    #[test]
    fn test_unified_node_clone() {
        let mut node = UnifiedNode::new(NodeKind::Class, "MyClass", Language::Java);
        node.add_child("child1".to_string());
        node.add_reference(ReferenceKind::Inherits, "Parent".to_string(), None);
        node.add_metadata("key", "value");

        let cloned = node.clone();

        assert_eq!(node.id, cloned.id);
        assert_eq!(node.name, cloned.name);
        assert_eq!(node.children.len(), cloned.children.len());
        assert_eq!(node.references.len(), cloned.references.len());
        assert_eq!(node.metadata.len(), cloned.metadata.len());
    }

    #[test]
    fn test_source_position_debug() {
        let pos = SourcePosition {
            start_line: 10,
            start_col: 5,
            end_line: 15,
            end_col: 20,
        };
        let debug_str = format!("{:?}", pos);
        assert!(debug_str.contains("SourcePosition"));
        assert!(debug_str.contains("10"));
    }

    #[test]
    fn test_reference_kind_debug() {
        let kind = ReferenceKind::Inherits;
        let debug_str = format!("{:?}", kind);
        assert_eq!(debug_str, "Inherits");
    }
}
