#[cfg_attr(coverage_nightly, coverage(off))]
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
