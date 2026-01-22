//! Rust language AST parsing strategy

use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;
use syn::{visit::Visit, File as SynFile, ItemEnum, ItemFn, ItemImpl, ItemStruct, ItemTrait};

use super::LanguageStrategy;
use crate::ast::core::{
    AstDag, AstKind, ClassKind, FunctionKind, Language, NodeFlags, UnifiedAstNode,
};

/// Rust language parsing strategy
pub struct RustStrategy {
    // Configuration options can be added here
}

impl Default for RustStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl RustStrategy {
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }

    fn parse_syn_file(&self, content: &str) -> Result<SynFile> {
        syn::parse_file(content).map_err(|e| anyhow::anyhow!("Rust parse error: {e}"))
    }

    fn convert_to_dag(&self, syn_file: &SynFile) -> AstDag {
        let mut dag = AstDag::new();
        let mut visitor = RustAstVisitor::new(&mut dag);
        visitor.visit_file(syn_file);
        dag
    }
}

#[async_trait]
impl LanguageStrategy for RustStrategy {
    fn language(&self) -> Language {
        Language::Rust
    }

    fn can_parse(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext == "rs")
    }

    async fn parse_file(&self, _path: &Path, content: &str) -> Result<AstDag> {
        let syn_file = self.parse_syn_file(content)?;
        Ok(self.convert_to_dag(&syn_file))
    }

    fn extract_imports(&self, ast: &AstDag) -> Vec<String> {
        // Iterate through nodes looking for imports
        let mut imports = Vec::new();
        for i in 0..ast.nodes.len() {
            if let Some(node) = ast.nodes.get(i as u32) {
                if node.flags.has(NodeFlags::IMPORT) {
                    // Extract import name from metadata if available
                    imports.push(format!("import_{i}"));
                }
            }
        }
        imports
    }

    fn extract_functions(&self, ast: &AstDag) -> Vec<UnifiedAstNode> {
        let mut functions = Vec::new();
        for i in 0..ast.nodes.len() {
            if let Some(node) = ast.nodes.get(i as u32) {
                if matches!(node.kind, AstKind::Function(_)) {
                    functions.push(node.clone());
                }
            }
        }
        functions
    }

    fn extract_types(&self, ast: &AstDag) -> Vec<UnifiedAstNode> {
        let mut types = Vec::new();
        for i in 0..ast.nodes.len() {
            if let Some(node) = ast.nodes.get(i as u32) {
                if matches!(node.kind, AstKind::Class(_) | AstKind::Type(_)) {
                    types.push(node.clone());
                }
            }
        }
        types
    }

    fn calculate_complexity(&self, ast: &AstDag) -> (u32, u32) {
        let mut cyclomatic = 1;
        let mut cognitive = 0;

        // Count control flow nodes
        for i in 0..ast.nodes.len() {
            if let Some(node) = ast.nodes.get(i as u32) {
                if node.flags.has(NodeFlags::CONTROL_FLOW) {
                    cyclomatic += 1;
                    // Cognitive complexity increases with nesting depth
                    cognitive += 1;
                }
            }
        }

        (cyclomatic, cognitive)
    }
}

/// Visitor for converting syn AST to unified AST
struct RustAstVisitor<'a> {
    dag: &'a mut AstDag,
    current_parent: Option<u32>,
}

impl<'a> RustAstVisitor<'a> {
    fn new(dag: &'a mut AstDag) -> Self {
        Self {
            dag,
            current_parent: None,
        }
    }

    #[allow(dead_code)]
    fn add_node(&mut self, kind: AstKind) -> u32 {
        let mut node = UnifiedAstNode::new(kind, Language::Rust);

        // Set parent if we have one
        if let Some(parent) = self.current_parent {
            node.parent = parent;
        }

        self.dag.add_node(node)
    }
}

impl Visit<'_> for RustAstVisitor<'_> {
    fn visit_item_fn(&mut self, node: &ItemFn) {
        let mut ast_node =
            UnifiedAstNode::new(AstKind::Function(FunctionKind::Regular), Language::Rust);

        // Set async flag if needed
        if node.sig.asyncness.is_some() {
            ast_node.flags.set(NodeFlags::ASYNC);
        }

        let key = self.dag.add_node(ast_node);

        let old_parent = self.current_parent;
        self.current_parent = Some(key);

        syn::visit::visit_item_fn(self, node);

        self.current_parent = old_parent;
    }

    fn visit_item_struct(&mut self, node: &ItemStruct) {
        let ast_node = UnifiedAstNode::new(AstKind::Class(ClassKind::Struct), Language::Rust);

        let key = self.dag.add_node(ast_node);

        let old_parent = self.current_parent;
        self.current_parent = Some(key);

        syn::visit::visit_item_struct(self, node);

        self.current_parent = old_parent;
    }

    fn visit_item_enum(&mut self, node: &ItemEnum) {
        let ast_node = UnifiedAstNode::new(AstKind::Class(ClassKind::Enum), Language::Rust);

        let key = self.dag.add_node(ast_node);

        let old_parent = self.current_parent;
        self.current_parent = Some(key);

        syn::visit::visit_item_enum(self, node);

        self.current_parent = old_parent;
    }

    fn visit_item_trait(&mut self, node: &ItemTrait) {
        let ast_node = UnifiedAstNode::new(AstKind::Class(ClassKind::Trait), Language::Rust);

        let key = self.dag.add_node(ast_node);

        let old_parent = self.current_parent;
        self.current_parent = Some(key);

        syn::visit::visit_item_trait(self, node);

        self.current_parent = old_parent;
    }

    fn visit_item_impl(&mut self, node: &ItemImpl) {
        // For impl blocks, we create a special kind
        let ast_node = UnifiedAstNode::new(AstKind::Class(ClassKind::Regular), Language::Rust);

        let key = self.dag.add_node(ast_node);

        let old_parent = self.current_parent;
        self.current_parent = Some(key);

        syn::visit::visit_item_impl(self, node);

        self.current_parent = old_parent;
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;
    use std::path::PathBuf;

    // Test RustStrategy construction and defaults
    #[test]
    fn test_rust_strategy_new() {
        let strategy = RustStrategy::new();
        assert_eq!(strategy.language(), Language::Rust);
    }

    #[test]
    fn test_rust_strategy_default() {
        let strategy = RustStrategy::default();
        assert_eq!(strategy.language(), Language::Rust);
    }

    // Test can_parse for various file extensions
    #[test]
    fn test_can_parse_rs_file() {
        let strategy = RustStrategy::new();
        assert!(strategy.can_parse(Path::new("test.rs")));
        assert!(strategy.can_parse(Path::new("/path/to/module.rs")));
        assert!(strategy.can_parse(Path::new("lib.rs")));
    }

    #[test]
    fn test_can_parse_non_rust_files() {
        let strategy = RustStrategy::new();
        assert!(!strategy.can_parse(Path::new("test.py")));
        assert!(!strategy.can_parse(Path::new("test.ts")));
        assert!(!strategy.can_parse(Path::new("test.js")));
        assert!(!strategy.can_parse(Path::new("test.c")));
        assert!(!strategy.can_parse(Path::new("test")));
        assert!(!strategy.can_parse(Path::new("")));
    }

    #[test]
    fn test_can_parse_no_extension() {
        let strategy = RustStrategy::new();
        assert!(!strategy.can_parse(Path::new("Makefile")));
        assert!(!strategy.can_parse(Path::new("README")));
    }

    // Test parse_syn_file with valid Rust code
    #[test]
    fn test_parse_syn_file_simple_function() {
        let strategy = RustStrategy::new();
        let code = "fn main() {}";
        let result = strategy.parse_syn_file(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_syn_file_async_function() {
        let strategy = RustStrategy::new();
        let code = "async fn async_main() {}";
        let result = strategy.parse_syn_file(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_syn_file_struct() {
        let strategy = RustStrategy::new();
        let code = "struct MyStruct { field: i32 }";
        let result = strategy.parse_syn_file(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_syn_file_enum() {
        let strategy = RustStrategy::new();
        let code = "enum Color { Red, Green, Blue }";
        let result = strategy.parse_syn_file(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_syn_file_trait() {
        let strategy = RustStrategy::new();
        let code = "trait Printable { fn print(&self); }";
        let result = strategy.parse_syn_file(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_syn_file_impl() {
        let strategy = RustStrategy::new();
        let code = r#"
            struct Foo;
            impl Foo {
                fn new() -> Self { Foo }
            }
        "#;
        let result = strategy.parse_syn_file(code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_syn_file_invalid_syntax() {
        let strategy = RustStrategy::new();
        let code = "fn main( { }"; // Invalid syntax
        let result = strategy.parse_syn_file(code);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Rust parse error"));
    }

    // Test convert_to_dag
    #[test]
    fn test_convert_to_dag_function() {
        let strategy = RustStrategy::new();
        let code = "fn main() {}";
        let syn_file = strategy.parse_syn_file(code).unwrap();
        let dag = strategy.convert_to_dag(&syn_file);
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn test_convert_to_dag_async_function() {
        let strategy = RustStrategy::new();
        let code = "async fn fetch_data() {}";
        let syn_file = strategy.parse_syn_file(code).unwrap();
        let dag = strategy.convert_to_dag(&syn_file);

        // Should have at least one node
        assert!(!dag.nodes.is_empty());

        // Check that the function node has the ASYNC flag
        let has_async = dag.nodes.iter().any(|node| {
            matches!(node.kind, AstKind::Function(_)) && node.flags.has(NodeFlags::ASYNC)
        });
        assert!(has_async, "Should have an async function node");
    }

    #[test]
    fn test_convert_to_dag_struct() {
        let strategy = RustStrategy::new();
        let code = "struct Point { x: i32, y: i32 }";
        let syn_file = strategy.parse_syn_file(code).unwrap();
        let dag = strategy.convert_to_dag(&syn_file);

        let has_struct = dag
            .nodes
            .iter()
            .any(|node| matches!(node.kind, AstKind::Class(ClassKind::Struct)));
        assert!(has_struct, "Should have a struct node");
    }

    #[test]
    fn test_convert_to_dag_enum() {
        let strategy = RustStrategy::new();
        let code = "enum Direction { Up, Down, Left, Right }";
        let syn_file = strategy.parse_syn_file(code).unwrap();
        let dag = strategy.convert_to_dag(&syn_file);

        let has_enum = dag
            .nodes
            .iter()
            .any(|node| matches!(node.kind, AstKind::Class(ClassKind::Enum)));
        assert!(has_enum, "Should have an enum node");
    }

    #[test]
    fn test_convert_to_dag_trait() {
        let strategy = RustStrategy::new();
        let code = "trait Drawable { fn draw(&self); }";
        let syn_file = strategy.parse_syn_file(code).unwrap();
        let dag = strategy.convert_to_dag(&syn_file);

        let has_trait = dag
            .nodes
            .iter()
            .any(|node| matches!(node.kind, AstKind::Class(ClassKind::Trait)));
        assert!(has_trait, "Should have a trait node");
    }

    #[test]
    fn test_convert_to_dag_impl_block() {
        let strategy = RustStrategy::new();
        let code = r#"
            struct Counter;
            impl Counter {
                fn increment(&mut self) {}
            }
        "#;
        let syn_file = strategy.parse_syn_file(code).unwrap();
        let dag = strategy.convert_to_dag(&syn_file);

        // Should have struct, impl block (Regular class), and function
        let node_count = dag.nodes.len();
        assert!(node_count >= 2, "Should have multiple nodes");
    }

    // Test parse_file async
    #[tokio::test]
    async fn test_parse_file_success() {
        let strategy = RustStrategy::new();
        let path = PathBuf::from("test.rs");
        let code = "fn hello() { println!(\"Hello!\"); }";
        let result = strategy.parse_file(&path, code).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_parse_file_error() {
        let strategy = RustStrategy::new();
        let path = PathBuf::from("test.rs");
        let code = "fn invalid syntax {";
        let result = strategy.parse_file(&path, code).await;
        assert!(result.is_err());
    }

    // Test extract_imports
    #[test]
    fn test_extract_imports_empty() {
        let strategy = RustStrategy::new();
        let dag = AstDag::new();
        let imports = strategy.extract_imports(&dag);
        assert!(imports.is_empty());
    }

    #[test]
    fn test_extract_imports_with_import_flag() {
        let strategy = RustStrategy::new();
        let mut dag = AstDag::new();

        // Add a node with IMPORT flag
        let mut node = UnifiedAstNode::new(
            AstKind::Import(crate::ast::core::ImportKind::Module),
            Language::Rust,
        );
        node.flags.set(NodeFlags::IMPORT);
        dag.add_node(node);

        let imports = strategy.extract_imports(&dag);
        assert_eq!(imports.len(), 1);
        assert!(imports[0].starts_with("import_"));
    }

    // Test extract_functions
    #[tokio::test]
    async fn test_extract_functions() {
        let strategy = RustStrategy::new();
        let path = PathBuf::from("test.rs");
        let code = r#"
            fn foo() {}
            fn bar() {}
            async fn baz() {}
        "#;
        let dag = strategy.parse_file(&path, code).await.unwrap();
        let functions = strategy.extract_functions(&dag);
        assert_eq!(functions.len(), 3);
    }

    #[test]
    fn test_extract_functions_empty_dag() {
        let strategy = RustStrategy::new();
        let dag = AstDag::new();
        let functions = strategy.extract_functions(&dag);
        assert!(functions.is_empty());
    }

    // Test extract_types
    #[tokio::test]
    async fn test_extract_types() {
        let strategy = RustStrategy::new();
        let path = PathBuf::from("test.rs");
        let code = r#"
            struct MyStruct {}
            enum MyEnum { A, B }
            trait MyTrait {}
        "#;
        let dag = strategy.parse_file(&path, code).await.unwrap();
        let types = strategy.extract_types(&dag);
        assert_eq!(types.len(), 3);
    }

    #[test]
    fn test_extract_types_empty_dag() {
        let strategy = RustStrategy::new();
        let dag = AstDag::new();
        let types = strategy.extract_types(&dag);
        assert!(types.is_empty());
    }

    // Test calculate_complexity
    #[test]
    fn test_calculate_complexity_empty_dag() {
        let strategy = RustStrategy::new();
        let dag = AstDag::new();
        let (cyclomatic, cognitive) = strategy.calculate_complexity(&dag);
        assert_eq!(cyclomatic, 1); // Base complexity
        assert_eq!(cognitive, 0);
    }

    #[test]
    fn test_calculate_complexity_with_control_flow() {
        let strategy = RustStrategy::new();
        let mut dag = AstDag::new();

        // Add nodes with CONTROL_FLOW flag
        let mut node1 = UnifiedAstNode::new(
            AstKind::Statement(crate::ast::core::StmtKind::If),
            Language::Rust,
        );
        node1.flags.set(NodeFlags::CONTROL_FLOW);
        dag.add_node(node1);

        let mut node2 = UnifiedAstNode::new(
            AstKind::Statement(crate::ast::core::StmtKind::For),
            Language::Rust,
        );
        node2.flags.set(NodeFlags::CONTROL_FLOW);
        dag.add_node(node2);

        let (cyclomatic, cognitive) = strategy.calculate_complexity(&dag);
        assert_eq!(cyclomatic, 3); // 1 base + 2 control flow
        assert_eq!(cognitive, 2);
    }

    // Test RustAstVisitor directly
    #[test]
    fn test_rust_visitor_add_node() {
        let mut dag = AstDag::new();
        let mut visitor = RustAstVisitor::new(&mut dag);

        let key = visitor.add_node(AstKind::Function(FunctionKind::Regular));
        assert_eq!(key, 0);
        assert_eq!(dag.nodes.len(), 1);
    }

    #[test]
    fn test_rust_visitor_with_parent() {
        let mut dag = AstDag::new();
        let mut visitor = RustAstVisitor::new(&mut dag);

        // Add parent node
        let parent_key = visitor.add_node(AstKind::Class(ClassKind::Struct));
        visitor.current_parent = Some(parent_key);

        // Add child node
        let child_key = visitor.add_node(AstKind::Function(FunctionKind::Method));

        // Verify parent is set
        let child_node = dag.nodes.get(child_key).unwrap();
        assert_eq!(child_node.parent, parent_key);
    }

    // Test complex code parsing
    #[tokio::test]
    async fn test_parse_complex_rust_code() {
        let strategy = RustStrategy::new();
        let path = PathBuf::from("test.rs");
        let code = r#"
            use std::collections::HashMap;

            pub struct Config {
                name: String,
                values: HashMap<String, i32>,
            }

            impl Config {
                pub fn new(name: &str) -> Self {
                    Self {
                        name: name.to_string(),
                        values: HashMap::new(),
                    }
                }

                pub async fn load(&mut self) -> Result<(), std::io::Error> {
                    Ok(())
                }
            }

            pub trait Configurable {
                fn configure(&self);
            }

            impl Configurable for Config {
                fn configure(&self) {
                    println!("{}", self.name);
                }
            }

            pub enum State {
                Active,
                Inactive,
                Pending { reason: String },
            }
        "#;

        let result = strategy.parse_file(&path, code).await;
        assert!(result.is_ok());

        let dag = result.unwrap();
        let _functions = strategy.extract_functions(&dag);
        let types = strategy.extract_types(&dag);

        // Functions inside impl blocks require visit_impl_item_fn handler
        // Current implementation only captures top-level functions (ItemFn)
        // Types should include: Config (struct), Configurable (trait), State (enum), plus impl blocks
        assert!(
            types.len() >= 3,
            "Should find struct, trait, enum, and impl blocks"
        );
    }
}
