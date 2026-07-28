// RustVisitor coverage tests for context service.
// Included via include!() in context_tests.rs.

// RustVisitor tests

#[test]
fn test_rust_visitor_new() {
    let visitor = RustVisitor::new("fn main() {}".to_string());
    assert!(visitor.items.is_empty());
}

#[test]
fn test_rust_visitor_get_visibility_public() {
    let visitor = RustVisitor::new(String::new());
    let vis = syn::parse_quote!(pub);
    assert_eq!(visitor.get_visibility(&vis), "pub");
}

#[test]
fn test_rust_visitor_get_visibility_private() {
    let visitor = RustVisitor::new(String::new());
    let vis = syn::Visibility::Inherited;
    assert_eq!(visitor.get_visibility(&vis), "private");
}

#[test]
fn test_rust_visitor_get_visibility_restricted_crate() {
    let visitor = RustVisitor::new(String::new());
    let vis: syn::Visibility = syn::parse_quote!(pub(crate));
    let result = visitor.get_visibility(&vis);
    assert!(result.starts_with("pub("));
}

#[test]
fn test_rust_visitor_use_statement_path() {
    use syn::parse_str;

    let code = "use std::io;";
    let syntax = parse_str::<syn::File>(code).unwrap();
    let mut visitor = RustVisitor::new(code.to_string());
    visitor.visit_file(&syntax);

    assert_eq!(visitor.items.len(), 1);
    if let AstItem::Use { path, .. } = &visitor.items[0] {
        assert_eq!(path, "std");
    } else {
        panic!("Expected Use item");
    }
}

#[test]
fn test_rust_visitor_use_statement_name() {
    use syn::parse_str;

    let code = "use io;";
    let syntax = parse_str::<syn::File>(code).unwrap();
    let mut visitor = RustVisitor::new(code.to_string());
    visitor.visit_file(&syntax);

    assert_eq!(visitor.items.len(), 1);
    if let AstItem::Use { path, .. } = &visitor.items[0] {
        assert_eq!(path, "io");
    }
}

#[test]
fn test_rust_visitor_use_statement_glob() {
    use syn::parse_str;

    let code = "use std::prelude::*;";
    let syntax = parse_str::<syn::File>(code).unwrap();
    let mut visitor = RustVisitor::new(code.to_string());
    visitor.visit_file(&syntax);

    assert_eq!(visitor.items.len(), 1);
}

#[test]
fn test_rust_visitor_use_statement_rename() {
    use syn::parse_str;

    let code = "use std::io as stdio;";
    let syntax = parse_str::<syn::File>(code).unwrap();
    let mut visitor = RustVisitor::new(code.to_string());
    visitor.visit_file(&syntax);

    assert_eq!(visitor.items.len(), 1);
}

#[test]
fn test_rust_visitor_use_statement_group() {
    use syn::parse_str;

    let code = "use std::{io, fs};";
    let syntax = parse_str::<syn::File>(code).unwrap();
    let mut visitor = RustVisitor::new(code.to_string());
    visitor.visit_file(&syntax);

    assert_eq!(visitor.items.len(), 1);
}

#[test]
fn test_rust_visitor_impl_inherent() {
    use syn::parse_str;

    let code = r#"
        impl MyStruct {
            fn new() -> Self { Self {} }
        }
    "#;

    let syntax = parse_str::<syn::File>(code).unwrap();
    let mut visitor = RustVisitor::new(code.to_string());
    visitor.visit_file(&syntax);

    assert_eq!(visitor.items.len(), 1);
    if let AstItem::Impl {
        type_name,
        trait_name,
        ..
    } = &visitor.items[0]
    {
        assert_eq!(type_name, "MyStruct");
        assert!(trait_name.is_none());
    }
}

#[test]
fn test_rust_visitor_struct_unit() {
    use syn::parse_str;

    let code = "pub struct UnitStruct;";
    let syntax = parse_str::<syn::File>(code).unwrap();
    let mut visitor = RustVisitor::new(code.to_string());
    visitor.visit_file(&syntax);

    assert_eq!(visitor.items.len(), 1);
    if let AstItem::Struct { fields_count, .. } = &visitor.items[0] {
        assert_eq!(*fields_count, 0);
    }
}

#[test]
fn test_rust_visitor_struct_tuple() {
    use syn::parse_str;

    let code = "pub struct TupleStruct(i32, String);";
    let syntax = parse_str::<syn::File>(code).unwrap();
    let mut visitor = RustVisitor::new(code.to_string());
    visitor.visit_file(&syntax);

    assert_eq!(visitor.items.len(), 1);
    if let AstItem::Struct { fields_count, .. } = &visitor.items[0] {
        assert_eq!(*fields_count, 2);
    }
}

// AstItem display name tests

#[test]
fn test_ast_item_display_name_function() {
    let item = AstItem::Function {
        name: "my_func".to_string(),
        visibility: "pub".to_string(),
        is_async: false,
        line: 1,
    };
    assert_eq!(item.display_name(), "my_func");
}

#[test]
fn test_ast_item_display_name_struct() {
    let item = AstItem::Struct {
        name: "MyStruct".to_string(),
        visibility: "pub".to_string(),
        fields_count: 3,
        derives: vec![],
        line: 1,
    };
    assert_eq!(item.display_name(), "MyStruct");
}

#[test]
fn test_ast_item_display_name_enum() {
    let item = AstItem::Enum {
        name: "MyEnum".to_string(),
        visibility: "pub".to_string(),
        variants_count: 5,
        line: 1,
    };
    assert_eq!(item.display_name(), "MyEnum");
}

#[test]
fn test_ast_item_display_name_trait() {
    let item = AstItem::Trait {
        name: "MyTrait".to_string(),
        visibility: "pub".to_string(),
        line: 1,
    };
    assert_eq!(item.display_name(), "MyTrait");
}

#[test]
fn test_ast_item_display_name_impl() {
    let item = AstItem::Impl {
        type_name: "MyType".to_string(),
        trait_name: Some("Display".to_string()),
        line: 1,
    };
    assert_eq!(item.display_name(), "MyType");
}

#[test]
fn test_ast_item_display_name_module() {
    let item = AstItem::Module {
        name: "my_mod".to_string(),
        visibility: "pub".to_string(),
        line: 1,
    };
    assert_eq!(item.display_name(), "my_mod");
}

#[test]
fn test_ast_item_display_name_use() {
    let item = AstItem::Use {
        path: "std::io".to_string(),
        line: 1,
    };
    assert_eq!(item.display_name(), "std::io");
}

#[test]
fn test_ast_item_display_name_import() {
    let item = AstItem::Import {
        module: "numpy".to_string(),
        items: vec!["array".to_string()],
        alias: Some("np".to_string()),
        line: 1,
    };
    assert_eq!(item.display_name(), "numpy");
}

#[test]
fn test_ast_item_equality() {
    let item1 = AstItem::Function {
        name: "test".to_string(),
        visibility: "pub".to_string(),
        is_async: false,
        line: 1,
    };
    let item2 = AstItem::Function {
        name: "test".to_string(),
        visibility: "pub".to_string(),
        is_async: false,
        line: 1,
    };
    assert_eq!(item1, item2);
}

#[test]
fn test_ast_item_inequality_different_type() {
    let func = AstItem::Function {
        name: "test".to_string(),
        visibility: "pub".to_string(),
        is_async: false,
        line: 1,
    };
    let struct_item = AstItem::Struct {
        name: "test".to_string(),
        visibility: "pub".to_string(),
        fields_count: 0,
        derives: vec![],
        line: 1,
    };
    assert_ne!(func, struct_item);
}

// Format functions tests

#[test]
fn test_format_module_item() {
    let item = AstItem::Module {
        name: "utils".to_string(),
        visibility: "pub".to_string(),
        line: 10,
    };
    let result = format_module_item(&item);
    assert!(result.contains("pub mod utils"));
    assert!(result.contains("line 10"));
}

#[test]
fn test_format_module_item_wrong_type() {
    let item = AstItem::Function {
        name: "func".to_string(),
        visibility: "pub".to_string(),
        is_async: false,
        line: 1,
    };
    let result = format_module_item(&item);
    assert!(result.is_empty());
}

#[test]
fn test_format_struct_item_with_derives() {
    let item = AstItem::Struct {
        name: "MyStruct".to_string(),
        visibility: "pub".to_string(),
        fields_count: 5,
        derives: vec!["Debug".to_string(), "Clone".to_string()],
        line: 20,
    };
    let result = format_struct_item(&item);
    assert!(result.contains("pub struct MyStruct"));
    assert!(result.contains("5 fields"));
    assert!(result.contains("Debug"));
    assert!(result.contains("Clone"));
}

#[test]
fn test_format_struct_item_no_derives() {
    let item = AstItem::Struct {
        name: "Simple".to_string(),
        visibility: "pub".to_string(),
        fields_count: 1,
        derives: vec![],
        line: 5,
    };
    let result = format_struct_item(&item);
    assert!(result.contains("Simple"));
    assert!(!result.contains("derives"));
}

#[test]
fn test_format_enum_item() {
    let item = AstItem::Enum {
        name: "Status".to_string(),
        visibility: "pub".to_string(),
        variants_count: 3,
        line: 15,
    };
    let result = format_enum_item(&item);
    assert!(result.contains("pub enum Status"));
    assert!(result.contains("3 variants"));
}

#[test]
fn test_format_trait_item() {
    let item = AstItem::Trait {
        name: "Printable".to_string(),
        visibility: "pub".to_string(),
        line: 25,
    };
    let result = format_trait_item(&item);
    assert!(result.contains("pub trait Printable"));
}

#[test]
fn test_format_function_item_async() {
    let item = AstItem::Function {
        name: "fetch_data".to_string(),
        visibility: "pub".to_string(),
        is_async: true,
        line: 30,
    };
    let result = format_function_item(&item);
    assert!(result.contains("async"));
    assert!(result.contains("fetch_data"));
}

#[test]
fn test_format_function_item_sync() {
    let item = AstItem::Function {
        name: "process".to_string(),
        visibility: "private".to_string(),
        is_async: false,
        line: 35,
    };
    let result = format_function_item(&item);
    assert!(!result.contains("async"));
    assert!(result.contains("private"));
}

#[test]
fn test_format_impl_item_with_trait() {
    let item = AstItem::Impl {
        type_name: "MyStruct".to_string(),
        trait_name: Some("Display".to_string()),
        line: 40,
    };
    let result = format_impl_item(&item);
    assert!(result.contains("impl Display for MyStruct"));
}

#[test]
fn test_format_impl_item_inherent() {
    let item = AstItem::Impl {
        type_name: "MyStruct".to_string(),
        trait_name: None,
        line: 45,
    };
    let result = format_impl_item(&item);
    assert!(result.contains("impl MyStruct"));
    assert!(!result.contains("for"));
}
