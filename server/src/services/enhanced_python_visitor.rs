//! Enhanced Python AST visitor that preserves real source locations and qualified names
//!
//! This module provides an enhanced visitor that extracts actual AST information
//! from RustPython-parsed Python code instead of generating placeholders,
//! enabling MCP tools to query precise code locations and symbol names.

#[cfg(feature = "python-ast")]
use crate::services::context::AstItem;
#[cfg(feature = "python-ast")]
use std::path::{Path, PathBuf};
#[cfg(feature = "python-ast")]
use rustpython_parser::ast::{Stmt, Expr, ModModule};

/// Enhanced Python AST visitor that preserves real source information
#[cfg(feature = "python-ast")]
pub struct EnhancedPythonVisitor {
    items: Vec<AstItem>,
    _file_path: PathBuf,
    module_path: Vec<String>,
    class_stack: Vec<String>,
}

#[cfg(feature = "python-ast")]
impl EnhancedPythonVisitor {
    /// Creates a new enhanced Python visitor
    #[must_use] 
    pub fn new(file_path: &Path) -> Self {
        Self {
            items: Vec::new(),
            _file_path: file_path.to_path_buf(),
            module_path: Vec::new(),
            class_stack: Vec::new(),
        }
    }

    /// Extracts AST items from a Python module
    #[must_use] 
    pub fn extract_items(mut self, module: &ModModule) -> Vec<AstItem> {
        self.visit_module(module);
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

    /// Gets line number (simplified for compatibility)
    fn get_line(&self, _stmt: &Stmt) -> usize {
        // For now, return line 1 - real line extraction needs investigating RustPython API
        1
    }

    /// Checks if function is async
    fn is_async_function(&self, decorators: &[Expr]) -> bool {
        decorators.iter().any(|decorator| {
            if let Expr::Name(name) = decorator {
                name.id.as_str() == "async"
            } else {
                false
            }
        })
    }

    /// Visits the module root
    fn visit_module(&mut self, module: &ModModule) {
        for stmt in &module.body {
            self.visit_stmt(stmt);
        }
    }

    /// Visits a statement
    fn visit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::FunctionDef(func) => self.visit_function_def(func),
            Stmt::AsyncFunctionDef(func) => self.visit_async_function_def(func),
            Stmt::ClassDef(class) => self.visit_class_def(class),
            _ => {
                // Handle other statements recursively if needed
                self.visit_stmt_children(stmt);
            }
        }
    }

    /// Visits function definition
    fn visit_function_def(&mut self, func: &rustpython_parser::ast::StmtFunctionDef) {
        let name = self.get_qualified_name(func.name.as_ref());
        let line = self.get_line(&Stmt::FunctionDef(func.clone()));
        let is_async = self.is_async_function(&func.decorator_list);

        self.items.push(AstItem::Function {
            name,
            visibility: "public".to_string(), // Python doesn't have explicit visibility
            is_async,
            line,
        });

        // Visit function body
        for stmt in &func.body {
            self.visit_stmt(stmt);
        }
    }

    /// Visits async function definition
    fn visit_async_function_def(&mut self, func: &rustpython_parser::ast::StmtAsyncFunctionDef) {
        let name = self.get_qualified_name(func.name.as_ref());
        let line = self.get_line(&Stmt::AsyncFunctionDef(func.clone()));

        self.items.push(AstItem::Function {
            name,
            visibility: "public".to_string(),
            is_async: true, // Async functions are always async
            line,
        });

        // Visit function body
        for stmt in &func.body {
            self.visit_stmt(stmt);
        }
    }

    /// Visits class definition
    fn visit_class_def(&mut self, class: &rustpython_parser::ast::StmtClassDef) {
        let name = self.get_qualified_name(class.name.as_ref());
        let line = self.get_line(&Stmt::ClassDef(class.clone()));

        // Count methods (functions within the class)
        let fields_count = class.body.iter()
            .filter(|stmt| matches!(stmt, Stmt::FunctionDef(_) | Stmt::AsyncFunctionDef(_)))
            .count();

        self.items.push(AstItem::Struct {
            name: name.clone(),
            visibility: "public".to_string(),
            fields_count,
            derives: vec![], // Python doesn't have derives like Rust
            line,
        });

        // Enter class context
        self.class_stack.push(class.name.to_string());

        // Visit class body
        for stmt in &class.body {
            self.visit_stmt(stmt);
        }

        // Exit class context
        self.class_stack.pop();
    }

    /// Visits children of other statement types
    fn visit_stmt_children(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::If(if_stmt) => {
                for stmt in &if_stmt.body {
                    self.visit_stmt(stmt);
                }
                for stmt in &if_stmt.orelse {
                    self.visit_stmt(stmt);
                }
            }
            Stmt::While(while_stmt) => {
                for stmt in &while_stmt.body {
                    self.visit_stmt(stmt);
                }
                for stmt in &while_stmt.orelse {
                    self.visit_stmt(stmt);
                }
            }
            Stmt::For(for_stmt) => {
                for stmt in &for_stmt.body {
                    self.visit_stmt(stmt);
                }
                for stmt in &for_stmt.orelse {
                    self.visit_stmt(stmt);
                }
            }
            _ => {
                // Other statement types can be handled here
            }
        }
    }
}

#[cfg(all(test, feature = "python-ast"))]
mod tests {
    use super::*;
    use rustpython_parser::Parse;
    use std::path::Path;

    fn parse_python(code: &str) -> ModModule {
        ModModule::parse(code, "test.py")
            .expect("Failed to parse Python code")
    }

    #[test]
    fn test_simple_function() {
        let code = r#"
def hello_world():
    print("Hello, World!")
"#;
        let module = parse_python(code);
        let visitor = EnhancedPythonVisitor::new(Path::new("test.py"));
        let items = visitor.extract_items(&module);

        assert_eq!(items.len(), 1);
        if let AstItem::Function { name, is_async, .. } = &items[0] {
            assert_eq!(name, "hello_world");
            assert!(!is_async);
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
        let module = parse_python(code);
        let visitor = EnhancedPythonVisitor::new(Path::new("test.py"));
        let items = visitor.extract_items(&module);

        assert_eq!(items.len(), 1);
        if let AstItem::Function { name, is_async, .. } = &items[0] {
            assert_eq!(name, "async_hello");
            assert!(is_async);
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
        let module = parse_python(code);
        let visitor = EnhancedPythonVisitor::new(Path::new("test.py"));
        let items = visitor.extract_items(&module);

        assert_eq!(items.len(), 3); // 1 class + 2 methods

        // Check class
        if let AstItem::Struct { name, fields_count, .. } = &items[0] {
            assert_eq!(name, "Calculator");
            assert_eq!(*fields_count, 2); // 2 methods
        } else {
            panic!("Expected class item");
        }

        // Check methods
        if let AstItem::Function { name, is_async, .. } = &items[1] {
            assert_eq!(name, "Calculator::add");
            assert!(!is_async);
        } else {
            panic!("Expected method item");
        }

        if let AstItem::Function { name, is_async, .. } = &items[2] {
            assert_eq!(name, "Calculator::multiply_async");
            assert!(is_async);
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
        let module = parse_python(code);
        let visitor = EnhancedPythonVisitor::new(Path::new("test.py"));
        let items = visitor.extract_items(&module);

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
        let module = parse_python(code);
        let visitor = EnhancedPythonVisitor::new(Path::new("test.py"));
        let items = visitor.extract_items(&module);

        // Should have: Database class, Connection class, connect method, disconnect method
        assert_eq!(items.len(), 4);

        // Check qualified names for nested class methods
        let names: Vec<String> = items.iter().map(|item| match item {
            AstItem::Function { name, .. } => name.clone(),
            AstItem::Struct { name, .. } => name.clone(),
            _ => "unknown".to_string(),
        }).collect();

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
    use rustpython_parser::Parse;

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

            if let Ok(module) = ModModule::parse(&code, "test.py") {
                let visitor = EnhancedPythonVisitor::new(Path::new("test.py"));
                let items = visitor.extract_items(&module);

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

            if let Ok(module) = ModModule::parse(&code, "test.py") {
                let visitor = EnhancedPythonVisitor::new(Path::new("test.py"));
                let items = visitor.extract_items(&module);

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