//! Helper functions for working with AstItem enum in MCP integration
//!
//! This module provides utility functions to extract information from AstItem
//! enum variants, avoiding direct field access which doesn't work on enums.
//!
//! ## Module Organization
//! - Main file: imports, extraction functions, type predicates
//! - `ast_item_helpers_tests.rs`: comprehensive unit tests

use crate::services::context::AstItem;

/// Extract the name from an AstItem
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn extract_visibility(item: &AstItem) -> String {
    match item {
        AstItem::Function { visibility, .. } => visibility.clone(),
        AstItem::Struct { visibility, .. } => visibility.clone(),
        AstItem::Enum { visibility, .. } => visibility.clone(),
        AstItem::Trait { visibility, .. } => visibility.clone(),
        AstItem::Module { visibility, .. } => visibility.clone(),
        AstItem::Impl { .. } => "public".to_string(),
        AstItem::Use { .. } => "public".to_string(),
        AstItem::Import { .. } => "public".to_string(),
    }
}

/// Extract the line number from an AstItem
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn extract_complexity(item: &AstItem) -> u32 {
    match item {
        AstItem::Function { .. } => 5,
        AstItem::Impl { .. } => 3,
        AstItem::Struct { .. } => 2,
        AstItem::Enum { .. } => 2,
        AstItem::Trait { .. } => 2,
        AstItem::Module { .. } => 1,
        AstItem::Use { .. } => 1,
        AstItem::Import { .. } => 1,
    }
}

/// Extract all common information from an AstItem as a tuple
/// Returns: (name, kind, visibility, line, complexity)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn is_function(item: &AstItem) -> bool {
    matches!(item, AstItem::Function { .. })
}

/// Check if an AstItem is a struct
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn is_struct(item: &AstItem) -> bool {
    matches!(item, AstItem::Struct { .. })
}

/// Check if an AstItem is an enum
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn is_enum(item: &AstItem) -> bool {
    matches!(item, AstItem::Enum { .. })
}

/// Check if an AstItem is a trait
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn is_trait(item: &AstItem) -> bool {
    matches!(item, AstItem::Trait { .. })
}

/// Check if an AstItem is an impl block
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn is_impl(item: &AstItem) -> bool {
    matches!(item, AstItem::Impl { .. })
}

/// Check if an AstItem is a module
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn is_module(item: &AstItem) -> bool {
    matches!(item, AstItem::Module { .. })
}

/// Check if an AstItem is async (only applicable to functions)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn is_async(item: &AstItem) -> bool {
    match item {
        AstItem::Function { is_async, .. } => *is_async,
        _ => false,
    }
}

// --- Tests (included from separate file) ---
include!("ast_item_helpers_tests.rs");
