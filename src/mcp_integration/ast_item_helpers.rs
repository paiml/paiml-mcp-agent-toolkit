//! Helper functions for working with AstItem enum in MCP integration
//!
//! This module provides utility functions to extract information from AstItem
//! enum variants, avoiding direct field access which doesn't work on enums.

use crate::services::context::AstItem;

/// Extract the name from an AstItem
pub fn extract_name(item: &AstItem) -> String {
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

/// Extract the kind/type as a string from an AstItem
pub fn extract_kind(item: &AstItem) -> String {
    match item {
        AstItem::Function { .. } => "function".to_string(),
        AstItem::Struct { .. } => "struct".to_string(),
        AstItem::Enum { .. } => "enum".to_string(),
        AstItem::Trait { .. } => "trait".to_string(),
        AstItem::Impl { .. } => "impl".to_string(),
        AstItem::Use { .. } => "use".to_string(),
        AstItem::Module { .. } => "module".to_string(),
        AstItem::Import { .. } => "import".to_string(),
    }
}

/// Extract the visibility from an AstItem
pub fn extract_visibility(item: &AstItem) -> String {
    match item {
        AstItem::Function { visibility, .. } => visibility.clone(),
        AstItem::Struct { visibility, .. } => visibility.clone(),
        AstItem::Enum { visibility, .. } => visibility.clone(),
        AstItem::Trait { visibility, .. } => visibility.clone(),
        AstItem::Module { visibility, .. } => visibility.clone(),
        AstItem::Impl { .. } => "public".to_string(), // impl blocks don't have visibility
        AstItem::Use { .. } => "public".to_string(),  // use statements are typically public
        AstItem::Import { .. } => "public".to_string(), // imports are typically public
    }
}

/// Extract the line number from an AstItem
pub fn extract_line(item: &AstItem) -> usize {
    match item {
        AstItem::Function { line, .. } => *line,
        AstItem::Struct { line, .. } => *line,
        AstItem::Enum { line, .. } => *line,
        AstItem::Trait { line, .. } => *line,
        AstItem::Impl { line, .. } => *line,
        AstItem::Use { line, .. } => *line,
        AstItem::Module { line, .. } => *line,
        AstItem::Import { line, .. } => *line,
    }
}

/// Calculate a simple complexity score for an AstItem
/// This is a basic heuristic; real complexity should come from analysis
pub fn extract_complexity(item: &AstItem) -> u32 {
    match item {
        // Functions can be complex
        AstItem::Function { .. } => 5,
        // Impl blocks can be complex
        AstItem::Impl { .. } => 3,
        // Structs and enums have moderate complexity
        AstItem::Struct { .. } => 2,
        AstItem::Enum { .. } => 2,
        // Traits have moderate complexity
        AstItem::Trait { .. } => 2,
        // Modules, uses, and imports are simple
        AstItem::Module { .. } => 1,
        AstItem::Use { .. } => 1,
        AstItem::Import { .. } => 1,
    }
}

/// Extract all common information from an AstItem as a tuple
/// Returns: (name, kind, visibility, line, complexity)
pub fn extract_all_info(item: &AstItem) -> (String, String, String, usize, u32) {
    (
        extract_name(item),
        extract_kind(item),
        extract_visibility(item),
        extract_line(item),
        extract_complexity(item),
    )
}

/// Check if an AstItem is a function
pub fn is_function(item: &AstItem) -> bool {
    matches!(item, AstItem::Function { .. })
}

/// Check if an AstItem is a struct
pub fn is_struct(item: &AstItem) -> bool {
    matches!(item, AstItem::Struct { .. })
}

/// Check if an AstItem is an enum
pub fn is_enum(item: &AstItem) -> bool {
    matches!(item, AstItem::Enum { .. })
}

/// Check if an AstItem is a trait
pub fn is_trait(item: &AstItem) -> bool {
    matches!(item, AstItem::Trait { .. })
}

/// Check if an AstItem is an impl block
pub fn is_impl(item: &AstItem) -> bool {
    matches!(item, AstItem::Impl { .. })
}

/// Check if an AstItem is a module
pub fn is_module(item: &AstItem) -> bool {
    matches!(item, AstItem::Module { .. })
}

/// Check if an AstItem is async (only applicable to functions)
pub fn is_async(item: &AstItem) -> bool {
    match item {
        AstItem::Function { is_async, .. } => *is_async,
        _ => false,
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create test items
    fn create_function(name: &str, visibility: &str, is_async: bool, line: usize) -> AstItem {
        AstItem::Function {
            name: name.to_string(),
            visibility: visibility.to_string(),
            is_async,
            line,
        }
    }

    fn create_struct(name: &str, visibility: &str, fields_count: usize, line: usize) -> AstItem {
        AstItem::Struct {
            name: name.to_string(),
            visibility: visibility.to_string(),
            fields_count,
            derives: vec![],
            line,
        }
    }

    fn create_enum(name: &str, visibility: &str, line: usize) -> AstItem {
        AstItem::Enum {
            name: name.to_string(),
            visibility: visibility.to_string(),
            variants_count: 3,
            line,
        }
    }

    fn create_trait(name: &str, visibility: &str, line: usize) -> AstItem {
        AstItem::Trait {
            name: name.to_string(),
            visibility: visibility.to_string(),
            line,
        }
    }

    fn create_impl(type_name: &str, trait_name: Option<&str>, line: usize) -> AstItem {
        AstItem::Impl {
            type_name: type_name.to_string(),
            trait_name: trait_name.map(|s| s.to_string()),
            line,
        }
    }

    fn create_use(path: &str, line: usize) -> AstItem {
        AstItem::Use {
            path: path.to_string(),
            line,
        }
    }

    fn create_module(name: &str, visibility: &str, line: usize) -> AstItem {
        AstItem::Module {
            name: name.to_string(),
            visibility: visibility.to_string(),
            line,
        }
    }

    fn create_import(module: &str, line: usize) -> AstItem {
        AstItem::Import {
            module: module.to_string(),
            items: vec![],
            alias: None,
            line,
        }
    }

    // === extract_name Tests ===

    #[test]
    fn test_extract_name() {
        let item = AstItem::Function {
            name: "test_func".to_string(),
            visibility: "pub".to_string(),
            is_async: false,
            line: 10,
        };
        assert_eq!(extract_name(&item), "test_func");
    }

    #[test]
    fn test_extract_name_struct() {
        let item = create_struct("MyStruct", "pub", 3, 5);
        assert_eq!(extract_name(&item), "MyStruct");
    }

    #[test]
    fn test_extract_name_enum() {
        let item = create_enum("MyEnum", "pub(crate)", 10);
        assert_eq!(extract_name(&item), "MyEnum");
    }

    #[test]
    fn test_extract_name_trait() {
        let item = create_trait("MyTrait", "pub", 15);
        assert_eq!(extract_name(&item), "MyTrait");
    }

    #[test]
    fn test_extract_name_impl() {
        let item = create_impl("MyType", None, 20);
        assert_eq!(extract_name(&item), "MyType");
    }

    #[test]
    fn test_extract_name_use() {
        let item = create_use("std::collections::HashMap", 25);
        assert_eq!(extract_name(&item), "std::collections::HashMap");
    }

    #[test]
    fn test_extract_name_module() {
        let item = create_module("my_module", "pub", 30);
        assert_eq!(extract_name(&item), "my_module");
    }

    #[test]
    fn test_extract_name_import() {
        let item = create_import("external_crate", 35);
        assert_eq!(extract_name(&item), "external_crate");
    }

    // === extract_kind Tests ===

    #[test]
    fn test_extract_kind() {
        let item = AstItem::Struct {
            name: "TestStruct".to_string(),
            visibility: "pub".to_string(),
            fields_count: 3,
            derives: vec![],
            line: 5,
        };
        assert_eq!(extract_kind(&item), "struct");
    }

    #[test]
    fn test_extract_kind_function() {
        let item = create_function("test", "pub", false, 1);
        assert_eq!(extract_kind(&item), "function");
    }

    #[test]
    fn test_extract_kind_enum() {
        let item = create_enum("MyEnum", "pub", 1);
        assert_eq!(extract_kind(&item), "enum");
    }

    #[test]
    fn test_extract_kind_trait() {
        let item = create_trait("MyTrait", "pub", 1);
        assert_eq!(extract_kind(&item), "trait");
    }

    #[test]
    fn test_extract_kind_impl() {
        let item = create_impl("MyType", None, 1);
        assert_eq!(extract_kind(&item), "impl");
    }

    #[test]
    fn test_extract_kind_use() {
        let item = create_use("std::io", 1);
        assert_eq!(extract_kind(&item), "use");
    }

    #[test]
    fn test_extract_kind_module() {
        let item = create_module("mod1", "pub", 1);
        assert_eq!(extract_kind(&item), "module");
    }

    #[test]
    fn test_extract_kind_import() {
        let item = create_import("crate1", 1);
        assert_eq!(extract_kind(&item), "import");
    }

    // === extract_visibility Tests ===

    #[test]
    fn test_extract_visibility_function() {
        let item = create_function("test", "pub(crate)", false, 1);
        assert_eq!(extract_visibility(&item), "pub(crate)");
    }

    #[test]
    fn test_extract_visibility_struct() {
        let item = create_struct("Test", "private", 0, 1);
        assert_eq!(extract_visibility(&item), "private");
    }

    #[test]
    fn test_extract_visibility_enum() {
        let item = create_enum("Test", "pub", 1);
        assert_eq!(extract_visibility(&item), "pub");
    }

    #[test]
    fn test_extract_visibility_trait() {
        let item = create_trait("Test", "pub", 1);
        assert_eq!(extract_visibility(&item), "pub");
    }

    #[test]
    fn test_extract_visibility_module() {
        let item = create_module("test", "pub(super)", 1);
        assert_eq!(extract_visibility(&item), "pub(super)");
    }

    #[test]
    fn test_extract_visibility_impl() {
        let item = create_impl("Test", None, 1);
        assert_eq!(extract_visibility(&item), "public");
    }

    #[test]
    fn test_extract_visibility_use() {
        let item = create_use("std::io", 1);
        assert_eq!(extract_visibility(&item), "public");
    }

    #[test]
    fn test_extract_visibility_import() {
        let item = create_import("crate1", 1);
        assert_eq!(extract_visibility(&item), "public");
    }

    // === extract_line Tests ===

    #[test]
    fn test_extract_line_function() {
        let item = create_function("test", "pub", false, 42);
        assert_eq!(extract_line(&item), 42);
    }

    #[test]
    fn test_extract_line_struct() {
        let item = create_struct("Test", "pub", 0, 100);
        assert_eq!(extract_line(&item), 100);
    }

    #[test]
    fn test_extract_line_enum() {
        let item = create_enum("Test", "pub", 200);
        assert_eq!(extract_line(&item), 200);
    }

    #[test]
    fn test_extract_line_trait() {
        let item = create_trait("Test", "pub", 300);
        assert_eq!(extract_line(&item), 300);
    }

    #[test]
    fn test_extract_line_impl() {
        let item = create_impl("Test", None, 400);
        assert_eq!(extract_line(&item), 400);
    }

    #[test]
    fn test_extract_line_use() {
        let item = create_use("std::io", 500);
        assert_eq!(extract_line(&item), 500);
    }

    #[test]
    fn test_extract_line_module() {
        let item = create_module("test", "pub", 600);
        assert_eq!(extract_line(&item), 600);
    }

    #[test]
    fn test_extract_line_import() {
        let item = create_import("crate1", 700);
        assert_eq!(extract_line(&item), 700);
    }

    // === extract_complexity Tests ===

    #[test]
    fn test_extract_complexity_function() {
        let item = create_function("test", "pub", false, 1);
        assert_eq!(extract_complexity(&item), 5);
    }

    #[test]
    fn test_extract_complexity_impl() {
        let item = create_impl("Test", None, 1);
        assert_eq!(extract_complexity(&item), 3);
    }

    #[test]
    fn test_extract_complexity_struct() {
        let item = create_struct("Test", "pub", 0, 1);
        assert_eq!(extract_complexity(&item), 2);
    }

    #[test]
    fn test_extract_complexity_enum() {
        let item = create_enum("Test", "pub", 1);
        assert_eq!(extract_complexity(&item), 2);
    }

    #[test]
    fn test_extract_complexity_trait() {
        let item = create_trait("Test", "pub", 1);
        assert_eq!(extract_complexity(&item), 2);
    }

    #[test]
    fn test_extract_complexity_module() {
        let item = create_module("test", "pub", 1);
        assert_eq!(extract_complexity(&item), 1);
    }

    #[test]
    fn test_extract_complexity_use() {
        let item = create_use("std::io", 1);
        assert_eq!(extract_complexity(&item), 1);
    }

    #[test]
    fn test_extract_complexity_import() {
        let item = create_import("crate1", 1);
        assert_eq!(extract_complexity(&item), 1);
    }

    // === extract_all_info Tests ===

    #[test]
    fn test_extract_all_info_function() {
        let item = create_function("my_func", "pub", false, 42);
        let (name, kind, vis, line, complexity) = extract_all_info(&item);
        assert_eq!(name, "my_func");
        assert_eq!(kind, "function");
        assert_eq!(vis, "pub");
        assert_eq!(line, 42);
        assert_eq!(complexity, 5);
    }

    #[test]
    fn test_extract_all_info_struct() {
        let item = create_struct("MyStruct", "private", 3, 100);
        let (name, kind, vis, line, complexity) = extract_all_info(&item);
        assert_eq!(name, "MyStruct");
        assert_eq!(kind, "struct");
        assert_eq!(vis, "private");
        assert_eq!(line, 100);
        assert_eq!(complexity, 2);
    }

    // === is_function Tests ===

    #[test]
    fn test_is_function() {
        let func = AstItem::Function {
            name: "test".to_string(),
            visibility: "pub".to_string(),
            is_async: false,
            line: 1,
        };
        assert!(is_function(&func));

        let struct_item = AstItem::Struct {
            name: "Test".to_string(),
            visibility: "pub".to_string(),
            fields_count: 0,
            derives: vec![],
            line: 1,
        };
        assert!(!is_function(&struct_item));
    }

    // === is_struct Tests ===

    #[test]
    fn test_is_struct() {
        let struct_item = create_struct("Test", "pub", 0, 1);
        assert!(is_struct(&struct_item));

        let func = create_function("test", "pub", false, 1);
        assert!(!is_struct(&func));
    }

    // === is_enum Tests ===

    #[test]
    fn test_is_enum() {
        let enum_item = create_enum("Test", "pub", 1);
        assert!(is_enum(&enum_item));

        let func = create_function("test", "pub", false, 1);
        assert!(!is_enum(&func));
    }

    // === is_trait Tests ===

    #[test]
    fn test_is_trait() {
        let trait_item = create_trait("Test", "pub", 1);
        assert!(is_trait(&trait_item));

        let func = create_function("test", "pub", false, 1);
        assert!(!is_trait(&func));
    }

    // === is_impl Tests ===

    #[test]
    fn test_is_impl() {
        let impl_item = create_impl("Test", None, 1);
        assert!(is_impl(&impl_item));

        let func = create_function("test", "pub", false, 1);
        assert!(!is_impl(&func));
    }

    // === is_module Tests ===

    #[test]
    fn test_is_module() {
        let module_item = create_module("test", "pub", 1);
        assert!(is_module(&module_item));

        let func = create_function("test", "pub", false, 1);
        assert!(!is_module(&func));
    }

    // === is_async Tests ===

    #[test]
    fn test_is_async_true() {
        let async_func = create_function("test", "pub", true, 1);
        assert!(is_async(&async_func));
    }

    #[test]
    fn test_is_async_false() {
        let sync_func = create_function("test", "pub", false, 1);
        assert!(!is_async(&sync_func));
    }

    #[test]
    fn test_is_async_non_function() {
        let struct_item = create_struct("Test", "pub", 0, 1);
        assert!(!is_async(&struct_item));

        let enum_item = create_enum("Test", "pub", 1);
        assert!(!is_async(&enum_item));

        let trait_item = create_trait("Test", "pub", 1);
        assert!(!is_async(&trait_item));
    }
}
