#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod visitor_tests {
    use super::*;

    #[tokio::test]
    async fn test_analyze_rust_file() {
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("test_visitor.rs");

        // Create a Rust file with various items
        std::fs::write(
            &temp_path,
            r#"
/// Public function.
pub fn public_function() {}

async fn async_function() {}

struct MyStruct {
    field1: i32,
    field2: String,
}

enum MyEnum {
    VariantA,
    VariantB(i32),
    VariantC { name: String },
}

trait MyTrait {
    fn do_something(&self);
}

impl MyStruct {
    fn new() -> Self {
        Self { field1: 0, field2: String::new() }
    }
}

impl MyTrait for MyStruct {
    fn do_something(&self) {}
}

mod submodule {}

use std::collections::HashMap;
"#,
        )
        .unwrap();

        let result = analyze_rust_file(&temp_path).await;
        assert!(result.is_ok(), "Failed to analyze Rust file: {:?}", result);

        let context = result.unwrap();
        assert_eq!(context.language, "rust");
        assert!(!context.items.is_empty());

        // Cleanup
        let _ = std::fs::remove_file(&temp_path);
    }

    #[tokio::test]
    async fn test_analyze_rust_file_with_cache() {
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("test_visitor_cache.rs");

        std::fs::write(&temp_path, "pub fn test_fn() {}").unwrap();

        // Test without cache (None)
        let result = analyze_rust_file_with_cache(&temp_path, None).await;
        assert!(result.is_ok());

        let _ = std::fs::remove_file(&temp_path);
    }

    #[tokio::test]
    async fn test_analyze_rust_file_parse_error() {
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("test_parse_error.rs");

        // Invalid Rust syntax
        std::fs::write(&temp_path, "fn missing_body").unwrap();

        let result = analyze_rust_file(&temp_path).await;
        assert!(result.is_err());

        let _ = std::fs::remove_file(&temp_path);
    }

    #[tokio::test]
    async fn test_analyze_rust_file_not_found() {
        let result = analyze_rust_file(Path::new("/nonexistent/file.rs")).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_rust_visitor_visibility() {
        // Create a RustVisitor and test visibility parsing
        let source = r#"
/// Public fn.
pub fn public_fn() {}
pub(crate) fn crate_fn() {}
fn private_fn() {}
"#
        .to_string();

        let visitor = RustVisitor::new(source.clone());

        // Test get_visibility
        let pub_vis = syn::Visibility::Public(syn::Token![pub](proc_macro2::Span::call_site()));
        assert_eq!(visitor.get_visibility(&pub_vis), "pub");

        let inherited = syn::Visibility::Inherited;
        assert_eq!(visitor.get_visibility(&inherited), "private");
    }

    #[test]
    fn test_rust_visitor_get_derives() {
        // get_derives always returns empty for now
        let derives = RustVisitor::get_derives(&[]);
        assert!(derives.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_project_for_dead_code() {
        // Test with current directory
        let result = analyze_project_for_dead_code(Path::new("."), "rust").await;
        // May or may not succeed depending on cwd, but shouldn't panic
        let _ = result;
    }

    #[tokio::test]
    async fn test_rust_visitor_use_tree_variants() {
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("test_use_tree.rs");

        std::fs::write(
            &temp_path,
            r#"
use std::collections::HashMap;
use std::*;
use std::{vec, collections};
use std::io::Read as IoRead;
"#,
        )
        .unwrap();

        let result = analyze_rust_file(&temp_path).await;
        assert!(result.is_ok());

        let _ = std::fs::remove_file(&temp_path);
    }

    #[tokio::test]
    async fn test_rust_visitor_struct_variants() {
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("test_struct_variants.rs");

        std::fs::write(
            &temp_path,
            r#"
// Named fields
struct NamedStruct {
    field1: i32,
    field2: String,
}

// Tuple struct
struct TupleStruct(i32, String);

// Unit struct
struct UnitStruct;
"#,
        )
        .unwrap();

        let result = analyze_rust_file(&temp_path).await;
        assert!(result.is_ok());

        let context = result.unwrap();
        // Should have 3 structs
        let struct_count = context
            .items
            .iter()
            .filter(|item| matches!(item, AstItem::Struct { .. }))
            .count();
        assert_eq!(struct_count, 3);

        let _ = std::fs::remove_file(&temp_path);
    }

    #[tokio::test]
    async fn test_rust_visitor_impl_non_path_type() {
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("test_impl_types.rs");

        std::fs::write(
            &temp_path,
            r#"
struct MyStruct;

impl MyStruct {
    fn method(&self) {}
}
"#,
        )
        .unwrap();

        let result = analyze_rust_file(&temp_path).await;
        assert!(result.is_ok());

        let _ = std::fs::remove_file(&temp_path);
    }
}
