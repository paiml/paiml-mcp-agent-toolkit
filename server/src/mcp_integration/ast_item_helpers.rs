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

#[cfg(test)]
mod tests {
    use super::*;

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
}
