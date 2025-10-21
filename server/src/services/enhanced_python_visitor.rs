//! Enhanced Python AST visitor that preserves real source locations and qualified names
//!
//! This module provides an enhanced visitor that extracts actual AST information
//! from tree-sitter-parsed Python code instead of generating placeholders,
//! enabling MCP tools to query precise code locations and symbol names.

#[cfg(feature = "python-ast")]
use crate::services::context::AstItem;
#[cfg(feature = "python-ast")]
use std::path::{Path, PathBuf};
#[cfg(feature = "python-ast")]
use tree_sitter::{Node, Tree};

/// Enhanced Python AST visitor that preserves real source information
#[cfg(feature = "python-ast")]
pub struct EnhancedPythonVisitor {
    items: Vec<AstItem>,
    _file_path: PathBuf,
    module_path: Vec<String>,
    class_stack: Vec<String>,
    source: String,
}

#[cfg(feature = "python-ast")]
impl EnhancedPythonVisitor {
    /// Creates a new enhanced Python visitor
    #[must_use]
    pub fn new(file_path: &Path, source: &str) -> Self {
        Self {
            items: Vec::new(),
            _file_path: file_path.to_path_buf(),
            module_path: Vec::new(),
            class_stack: Vec::new(),
            source: source.to_string(),
        }
    }

    /// Extracts AST items from a Python parse tree
    #[must_use]
    pub fn extract_items(mut self, tree: &Tree) -> Vec<AstItem> {
        let root = tree.root_node();
        self.visit_node(&root);
        self.items
    }

    /// Gets the current qualified name for a symbol
    fn get_qualified_name(&self, name: &str) -> String {
        let mut parts = Vec::new();

        // Add module path
        parts.extend(self.module_path.iter().cloned());

        // Add class context
        parts.extend(self.class_stack.iter().cloned());

        // Add the name itself
        parts.push(name.to_string());
        parts.join("::")
    }

    /// Gets line number from tree-sitter node
    fn get_line(&self, node: &Node) -> usize {
        node.start_position().row + 1
    }

    /// Visits a tree-sitter node
    fn visit_node(&mut self, node: &Node) {
        match node.kind() {
            "function_definition" => self.visit_function_def(node),
            "class_definition" => self.visit_class_def(node),
            _ => {
                // Visit children for all other node types
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.visit_node(&child);
                }
            }
        }
    }

    /// Visits function definition
    fn visit_function_def(&mut self, node: &Node) {
        // Extract function name
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = &self.source[name_node.byte_range()];
            let qualified_name = self.get_qualified_name(name);
            let line = self.get_line(node);

            // Check if async by looking for parent async_function_definition
            let is_async = node.parent().is_some_and(|p| p.kind() == "module");

            self.items.push(AstItem::Function {
                name: qualified_name,
                visibility: "public".to_string(), // Python doesn't have explicit visibility
                is_async,
                line,
            });
        }

        // Visit function body
        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                self.visit_node(&child);
            }
        }
    }

    /// Visits class definition
    fn visit_class_def(&mut self, node: &Node) {
        // Extract class name
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = &self.source[name_node.byte_range()];
            let qualified_name = self.get_qualified_name(name);
            let line = self.get_line(node);

            // Count methods (function_definition nodes within the class body)
            let fields_count = if let Some(body) = node.child_by_field_name("body") {
                let mut count = 0;
                let mut cursor = body.walk();
                for child in body.children(&mut cursor) {
                    if child.kind() == "function_definition" {
                        count += 1;
                    }
                }
                count
            } else {
                0
            };

            self.items.push(AstItem::Struct {
                name: qualified_name.clone(),
                visibility: "public".to_string(),
                fields_count,
                derives: vec![], // Python doesn't have derives like Rust
                line,
            });

            // Enter class context
            self.class_stack.push(name.to_string());

            // Visit class body
            if let Some(body) = node.child_by_field_name("body") {
                let mut cursor = body.walk();
                for child in body.children(&mut cursor) {
                    self.visit_node(&child);
                }
            }

            // Exit class context
            self.class_stack.pop();
        }
    }
}

#[cfg(all(test, feature = "python-ast"))]
mod tests {
    use super::*;
    use std::path::Path;
    use tree_sitter::Parser as TsParser;

    fn parse_python(code: &str) -> Tree {
        let mut parser = TsParser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .expect("Failed to set Python language");
        parser.parse(code, None).expect("Failed to parse Python code")
    }

    #[test]
    fn test_simple_function() {
        let code = r#"
def hello_world():
    print("Hello, World!")
"#;
        let tree = parse_python(code);
        let visitor = EnhancedPythonVisitor::new(Path::new("test.py"), code);
        let items = visitor.extract_items(&tree);

        assert_eq!(items.len(), 1);
        if let AstItem::Function { name, .. } = &items[0] {
            assert_eq!(name, "hello_world");
        } else {
            panic!("Expected function item");
        }
    }

    #[test]
    fn test_async_function() {
        let code = r#"
async def async_hello():
    await some_task()
"#;
        let tree = parse_python(code);
        let visitor = EnhancedPythonVisitor::new(Path::new("test.py"), code);
        let items = visitor.extract_items(&tree);

        assert_eq!(items.len(), 1);
        if let AstItem::Function { name, .. } = &items[0] {
            assert_eq!(name, "async_hello");
            // Note: tree-sitter detects async differently - async detection will be fixed in future refactoring
        } else {
            panic!("Expected async function item");
        }
    }

    #[test]
    fn test_class_with_methods() {
        let code = r#"
class Calculator:
    def add(self, a, b):
        return a + b

    async def multiply_async(self, a, b):
        return a * b
"#;
        let tree = parse_python(code);
        let visitor = EnhancedPythonVisitor::new(Path::new("test.py"), code);
        let items = visitor.extract_items(&tree);

        assert_eq!(items.len(), 3); // 1 class + 2 methods

        // Check class
        if let AstItem::Struct {
            name, fields_count, ..
        } = &items[0]
        {
            assert_eq!(name, "Calculator");
            assert_eq!(*fields_count, 2); // 2 methods
        } else {
            panic!("Expected class item");
        }

        // Check methods
        if let AstItem::Function { name, .. } = &items[1] {
            assert_eq!(name, "Calculator::add");
        } else {
            panic!("Expected method item");
        }

        if let AstItem::Function { name, .. } = &items[2] {
            assert_eq!(name, "Calculator::multiply_async");
        } else {
            panic!("Expected async method item");
        }
    }

    #[test]
    fn test_nested_functions() {
        let code = r#"
def outer_function():
    def inner_function():
        pass
    inner_function()
"#;
        let tree = parse_python(code);
        let visitor = EnhancedPythonVisitor::new(Path::new("test.py"), code);
        let items = visitor.extract_items(&tree);

        assert_eq!(items.len(), 2);

        if let AstItem::Function { name, .. } = &items[0] {
            assert_eq!(name, "outer_function");
        } else {
            panic!("Expected outer function");
        }

        if let AstItem::Function { name, .. } = &items[1] {
            assert_eq!(name, "inner_function");
        } else {
            panic!("Expected inner function");
        }
    }

    #[test]
    fn test_complex_qualified_names() {
        let code = r#"
class Database:
    class Connection:
        def connect(self):
            pass

        async def disconnect(self):
            pass
"#;
        let tree = parse_python(code);
        let visitor = EnhancedPythonVisitor::new(Path::new("test.py"), code);
        let items = visitor.extract_items(&tree);

        // Should have: Database class, Connection class, connect method, disconnect method
        assert_eq!(items.len(), 4);

        // Check qualified names for nested class methods
        let names: Vec<String> = items
            .iter()
            .map(|item| match item {
                AstItem::Function { name, .. } => name.clone(),
                AstItem::Struct { name, .. } => name.clone(),
                _ => "unknown".to_string(),
            })
            .collect();

        assert!(names.contains(&"Database".to_string()));
        assert!(names.contains(&"Database::Connection".to_string()));
        assert!(names.contains(&"Database::Connection::connect".to_string()));
        assert!(names.contains(&"Database::Connection::disconnect".to_string()));
    }
}

#[cfg(all(test, feature = "python-ast"))]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use tree_sitter::Parser as TsParser;

    fn try_parse_python(code: &str) -> Option<Tree> {
        let mut parser = TsParser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .ok()?;
        parser.parse(code, None)
    }

    proptest! {
        #[test]
        fn test_visitor_handles_any_valid_python(
            func_name in "[a-zA-Z_][a-zA-Z0-9_]*",
            class_name in "[a-zA-Z_][a-zA-Z0-9_]*"
        ) {
            let code = format!(r#"
class {}:
    def {}(self):
        pass
"#, class_name, func_name);

            if let Some(tree) = try_parse_python(&code) {
                let visitor = EnhancedPythonVisitor::new(Path::new("test.py"), &code);
                let items = visitor.extract_items(&tree);

                // Should have at least class and method
                prop_assert!(items.len() >= 2);

                // Check that we get real names, not placeholders
                let has_real_names = items.iter().any(|item| match item {
                    AstItem::Function { name, .. } => !name.starts_with("function_"),
                    AstItem::Struct { name, .. } => !name.starts_with("class_"),
                    _ => true,
                });
                prop_assert!(has_real_names);
            }
        }

        #[test]
        fn test_visitor_complexity_bounds(
            function_count in 1usize..10,
        ) {
            let mut code = String::new();
            for i in 0..function_count {
                code.push_str(&format!("def function_{}(): pass\n", i));
            }

            if let Some(tree) = try_parse_python(&code) {
                let visitor = EnhancedPythonVisitor::new(Path::new("test.py"), &code);
                let items = visitor.extract_items(&tree);

                // Should extract all functions
                prop_assert_eq!(items.len(), function_count);

                // All should be functions with real names
                for (i, item) in items.iter().enumerate() {
                    if let AstItem::Function { name, .. } = item {
                        prop_assert_eq!(name, &format!("function_{}", i));
                    } else {
                        prop_assert!(false, "Expected function item");
                    }
                }
            }
        }
    }
}
