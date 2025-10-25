# Sprint 53 AstItem and NodeKind Alignment

## Overview

This task focuses on resolving the mismatches between the `AstItem` enum and `NodeKind` enum that are causing compilation errors in the polyglot AST framework. The goal is to ensure proper alignment between the two types, allowing for seamless conversion between language-specific AST items and unified node representations.

## Current Issues

1. Missing variants in `NodeKind` that are referenced in `from_ast_item` method
2. Referencing non-existent variants of `AstItem` in the `NodeKind` conversion code
3. Incomplete mapping between `AstItem` and `NodeKind` variants

## Goals

1. Update `NodeKind` enum to include all necessary variants
2. Fix the `from_ast_item` method to handle only existing variants
3. Ensure bidirectional conversion between `AstItem` and `NodeKind`
4. Add comprehensive tests for the conversion logic

## Implementation Details

### 1. Update NodeKind Enum

First, we need to update the `NodeKind` enum in `server/src/ast/polyglot/mod.rs` to include all necessary variants:

```rust
/// Common node types across languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeKind {
    // Declarations
    Package,
    Import,
    Module,
    Namespace,
    
    // Types
    Class,
    Interface,
    Trait,
    Enum,
    Struct,
    Union,
    Type,          // New: For type aliases/typedefs
    
    // Type variations
    Record,        // Java record, Kotlin data class, TypeScript interface
    CaseClass,     // Scala case class
    AbstractType,  // Abstract class/interface/trait
    
    // Functions & methods
    Method,
    Function,
    Constructor,
    Lambda,
    Closure,
    
    // Variables
    Field,
    Property,      // Existing but used in conversion
    LocalVariable, // Existing but used in conversion
    Parameter,
    
    // Other elements
    Annotation,
    Decorator,     // Used in conversion
    Comment,       // Used in conversion
    
    // Relationships
    Inherits,
    Implements,
    Uses,
    
    // Implementation-specific
    Impl,          // For Rust impl blocks
    
    // For any language-specific constructs
    LanguageSpecific(u32), // Numeric identifier for language-specific nodes
    Unknown,
}
```

### 2. Fix NodeKind::from_ast_item Method

Update the `from_ast_item` method to handle only existing variants of `AstItem`:

```rust
impl NodeKind {
    /// Convert from AstItem enum
    pub fn from_ast_item(item: &AstItem) -> Self {
        match item {
            AstItem {
                kind: kind_str,
                ..
            } => {
                // First try matching by kind string
                match kind_str.as_str() {
                    "function" => NodeKind::Function,
                    "struct" => NodeKind::Struct,
                    "enum" => NodeKind::Enum,
                    "trait" => NodeKind::Trait,
                    "impl" => NodeKind::Impl,
                    "use" => NodeKind::Import,
                    "module" => NodeKind::Module,
                    "type" => NodeKind::Type,
                    "class" => NodeKind::Class,
                    "interface" => NodeKind::Interface,
                    "method" => NodeKind::Method,
                    "field" | "property" => NodeKind::Property,
                    "package" => NodeKind::Package,
                    "import" => NodeKind::Import,
                    "variable" | "localVariable" => NodeKind::LocalVariable,
                    "comment" => NodeKind::Comment,
                    "constructor" => NodeKind::Constructor,
                    "parameter" => NodeKind::Parameter,
                    "annotation" | "decorator" => NodeKind::Decorator,
                    "namespace" => NodeKind::Namespace,
                    "lambda" | "closure" => NodeKind::Lambda,
                    "record" => NodeKind::Record,
                    "caseClass" => NodeKind::CaseClass,
                    _ => NodeKind::Unknown,
                }
            }
        }
    }

    /// Attempts to determine the NodeKind from a kind string
    pub fn from_kind_str(kind_str: &str) -> Self {
        match kind_str {
            "function" => NodeKind::Function,
            "struct" => NodeKind::Struct,
            "enum" => NodeKind::Enum,
            "trait" => NodeKind::Trait,
            "impl" => NodeKind::Impl,
            "use" | "import" => NodeKind::Import,
            "module" => NodeKind::Module,
            "type" => NodeKind::Type,
            "class" => NodeKind::Class,
            "interface" => NodeKind::Interface,
            "method" => NodeKind::Method,
            "field" | "property" => NodeKind::Property,
            "package" => NodeKind::Package,
            "variable" | "localVariable" => NodeKind::LocalVariable,
            "comment" => NodeKind::Comment,
            "constructor" => NodeKind::Constructor,
            "parameter" => NodeKind::Parameter,
            "annotation" | "decorator" => NodeKind::Decorator,
            "namespace" => NodeKind::Namespace,
            "lambda" | "closure" => NodeKind::Lambda,
            "record" => NodeKind::Record,
            "caseClass" => NodeKind::CaseClass,
            _ => NodeKind::Unknown,
        }
    }
}
```

### 3. Create a Helper Method to Extract Kind from AstItem

Add a helper method to safely extract the kind from any AstItem:

```rust
/// Extract the kind from an AstItem
pub fn extract_kind_from_ast_item(item: &AstItem) -> &str {
    &item.kind
}
```

### 4. Update UnifiedNode::from_ast_item Method

Update the `UnifiedNode::from_ast_item` method in `server/src/ast/polyglot/unified_node.rs` to use the new approach:

```rust
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
        
        // Extract basic properties from AstItem
        let name = item.name.clone();
        let line = item.span.start.line;
        let visibility = item.access.clone().unwrap_or_default();
        
        // Determine the node kind from the AstItem kind
        let kind = NodeKind::from_ast_item(item);
        let id = format!("{}:{}:{}", prefix, kind.as_str(), name);
        
        // Create position info
        let position = SourcePosition {
            start_line: line,
            start_col: item.span.start.column,
            end_line: item.span.end.line,
            end_col: item.span.end.column,
        };
        
        // Extract attributes
        let mut attributes = HashMap::new();
        attributes.insert("access".to_string(), visibility);
        
        // Add any modifiers as attributes
        if let Some(modifiers) = &item.modifiers {
            for modifier in modifiers {
                attributes.insert(format!("modifier:{}", modifier), "true".to_string());
            }
        }
        
        // Create the FQN (namespace + name)
        let fqn = if item.namespace.is_empty() {
            name.clone()
        } else {
            format!("{}.{}", item.namespace, name)
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
            signature: item.signature.clone(),
            documentation: item.documentation.clone(),
            original_item: Some(item.clone()),
            metadata: HashMap::new(),
        }
    }
}
```

### 5. Add Tests for Conversions

Create comprehensive tests for the AstItem to NodeKind conversions:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::context::{AstItem, Span, Position};
    
    fn create_test_ast_item(kind_str: &str, name: &str) -> AstItem {
        AstItem {
            id: 1,
            kind: kind_str.to_string(),
            name: name.to_string(),
            namespace: "com.example".to_string(),
            signature: Some(format!("test {}", name)),
            content: format!("{} {{}}", name),
            complexity: 1,
            span: Span {
                start: Position { line: 1, column: 1 },
                end: Position { line: 5, column: 1 },
            },
            access: Some("public".to_string()),
            modifiers: Some(vec!["static".to_string()]),
            documentation: Some(format!("Test {} documentation", name)),
            children: Vec::new(),
            references: Vec::new(),
        }
    }
    
    #[test]
    fn test_node_kind_from_ast_item() {
        // Test all supported kinds
        let item_kinds = vec![
            ("function", NodeKind::Function),
            ("struct", NodeKind::Struct),
            ("enum", NodeKind::Enum),
            ("trait", NodeKind::Trait),
            ("impl", NodeKind::Impl),
            ("use", NodeKind::Import),
            ("import", NodeKind::Import),
            ("module", NodeKind::Module),
            ("type", NodeKind::Type),
            ("class", NodeKind::Class),
            ("interface", NodeKind::Interface),
            ("method", NodeKind::Method),
            ("field", NodeKind::Property),
            ("property", NodeKind::Property),
            ("package", NodeKind::Package),
            ("variable", NodeKind::LocalVariable),
            ("localVariable", NodeKind::LocalVariable),
            ("comment", NodeKind::Comment),
            ("constructor", NodeKind::Constructor),
            ("parameter", NodeKind::Parameter),
            ("annotation", NodeKind::Decorator),
            ("decorator", NodeKind::Decorator),
            ("namespace", NodeKind::Namespace),
            ("lambda", NodeKind::Lambda),
            ("closure", NodeKind::Lambda),
            ("record", NodeKind::Record),
            ("caseClass", NodeKind::CaseClass),
            ("unknown", NodeKind::Unknown),
        ];
        
        for (kind_str, expected_kind) in item_kinds {
            let ast_item = create_test_ast_item(kind_str, "TestItem");
            let node_kind = NodeKind::from_ast_item(&ast_item);
            assert_eq!(node_kind, expected_kind, "Failed for kind: {}", kind_str);
            
            // Also test from_kind_str
            let kind_from_str = NodeKind::from_kind_str(kind_str);
            assert_eq!(kind_from_str, expected_kind, "from_kind_str failed for: {}", kind_str);
        }
    }
    
    #[test]
    fn test_unified_node_from_ast_item() {
        let ast_item = create_test_ast_item("class", "TestClass");
        let file_path = Path::new("/test/TestClass.java");
        
        let node = UnifiedNode::from_ast_item(&ast_item, Language::Java, file_path, None);
        
        assert_eq!(node.id, "Java:class:TestClass");
        assert_eq!(node.kind, NodeKind::Class);
        assert_eq!(node.name, "TestClass");
        assert_eq!(node.fqn, "com.example.TestClass");
        assert_eq!(node.language, Language::Java);
        assert_eq!(node.position.start_line, 1);
        assert_eq!(node.position.end_line, 5);
        assert_eq!(node.access(), Some("public"));
        assert!(node.has_modifier("static"));
        assert_eq!(node.documentation.as_deref(), Some("Test TestClass documentation"));
    }
    
    #[test]
    fn test_node_kind_as_str() {
        // Test all variants
        assert_eq!(NodeKind::Class.as_str(), "class");
        assert_eq!(NodeKind::Function.as_str(), "function");
        assert_eq!(NodeKind::Method.as_str(), "method");
        assert_eq!(NodeKind::Struct.as_str(), "struct");
        assert_eq!(NodeKind::Enum.as_str(), "enum");
        assert_eq!(NodeKind::Trait.as_str(), "trait");
        assert_eq!(NodeKind::Interface.as_str(), "interface");
        assert_eq!(NodeKind::Record.as_str(), "record");
        assert_eq!(NodeKind::CaseClass.as_str(), "caseClass");
        assert_eq!(NodeKind::Module.as_str(), "module");
        assert_eq!(NodeKind::Package.as_str(), "package");
        assert_eq!(NodeKind::Import.as_str(), "import");
        assert_eq!(NodeKind::Property.as_str(), "property");
        assert_eq!(NodeKind::LocalVariable.as_str(), "localVariable");
        assert_eq!(NodeKind::Unknown.as_str(), "unknown");
    }
}
```

### 6. Add Proper as_str Implementation for NodeKind

Update the `as_str` method for NodeKind to handle all variants:

```rust
impl NodeKind {
    /// Returns a string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeKind::Package => "package",
            NodeKind::Import => "import",
            NodeKind::Module => "module",
            NodeKind::Namespace => "namespace",
            
            NodeKind::Class => "class",
            NodeKind::Interface => "interface",
            NodeKind::Trait => "trait",
            NodeKind::Enum => "enum",
            NodeKind::Struct => "struct",
            NodeKind::Union => "union",
            NodeKind::Type => "type",
            
            NodeKind::Record => "record",
            NodeKind::CaseClass => "caseClass",
            NodeKind::AbstractType => "abstractType",
            
            NodeKind::Method => "method",
            NodeKind::Function => "function",
            NodeKind::Constructor => "constructor",
            NodeKind::Lambda => "lambda",
            NodeKind::Closure => "closure",
            
            NodeKind::Field => "field",
            NodeKind::Property => "property",
            NodeKind::LocalVariable => "localVariable",
            NodeKind::Parameter => "parameter",
            
            NodeKind::Annotation => "annotation",
            NodeKind::Decorator => "decorator",
            NodeKind::Comment => "comment",
            
            NodeKind::Inherits => "inherits",
            NodeKind::Implements => "implements",
            NodeKind::Uses => "uses",
            
            NodeKind::Impl => "impl",
            
            NodeKind::LanguageSpecific(_) => "languageSpecific",
            NodeKind::Unknown => "unknown",
        }
    }
}
```

### 7. Add from_ast_item_kind Function

Create a utility function to convert from AstItem kind string to NodeKind:

```rust
/// Convert an AstItem kind string to NodeKind
pub fn from_ast_item_kind(kind_str: &str) -> NodeKind {
    NodeKind::from_kind_str(kind_str)
}

#[test]
fn test_from_ast_item_kind() {
    assert_eq!(from_ast_item_kind("class"), NodeKind::Class);
    assert_eq!(from_ast_item_kind("function"), NodeKind::Function);
    assert_eq!(from_ast_item_kind("method"), NodeKind::Method);
    assert_eq!(from_ast_item_kind("unknown"), NodeKind::Unknown);
}
```

## Integration with Language Mappers

When implementing language mappers, ensure that AstItem kinds are correctly set to values that will map to the appropriate NodeKind:

```rust
// Example in JavaMapper implementation
let ast_items = visitor.analyze_java_source(source)?;
for ast_item in &ast_items {
    // Ensure the kind string is set to a value that maps to the correct NodeKind
    let node = UnifiedNode::from_ast_item(ast_item, Language::Java, path, None);
    nodes.push(node);
}
```

## Success Criteria

1. All compilation errors related to AstItem and NodeKind mismatches are resolved
2. The `from_ast_item` method correctly maps AstItem variants to NodeKind variants
3. Tests pass for all conversions
4. Language mappers can successfully convert language-specific AST items to UnifiedNodes

## Estimated Effort

- Implementation: 0.5 day
- Testing: 0.5 day
- Integration: 0.5 day

Total: 1.5 days

## Dependencies

- Should be implemented alongside or after the feature flag implementation
- Required for StubMapper and language mappers to work correctly

## Next Steps After Completion

1. Complete the StubMapper implementation
2. Implement language mappers for Java, Kotlin, and Scala
3. Implement cross-language dependency detection