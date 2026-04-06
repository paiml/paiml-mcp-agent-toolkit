#![cfg_attr(coverage_nightly, coverage(off))]
//! Symbol extraction from AST and deep context

use crate::services::context::AstItem;
use crate::services::deep_context::DeepContext;
use std::path::PathBuf;

use super::filters::{passes_query_filter, passes_type_filter};
use super::types::SymbolInfo;

/// Extract symbol information from an AST item
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::cli::symbol_table_helpers::extract_symbol_from_ast_item;
/// use pmat::services::context::AstItem;
///
/// let item = AstItem::Function {
///     name: "main".to_string(),
///     visibility: "public".to_string(),
///     is_async: false,
///     line: 1,
/// };
///
/// let result = extract_symbol_from_ast_item(&item);
/// assert!(result.is_some());
/// let (name, kind, line, visibility, is_async) = result.unwrap();
/// assert_eq!(name, "main");
/// assert_eq!(kind, "function");
/// assert_eq!(line, 1);
/// ```
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn extract_symbol_from_ast_item(
    item: &AstItem,
) -> Option<(String, &'static str, usize, String, bool)> {
    match item {
        AstItem::Function {
            name,
            visibility,
            is_async,
            line,
        } => Some((
            name.clone(),
            "function",
            *line,
            visibility.clone(),
            *is_async,
        )),
        AstItem::Struct {
            name,
            visibility,
            fields_count: _,
            line,
            ..
        } => Some((name.clone(), "struct", *line, visibility.clone(), false)),
        AstItem::Enum {
            name,
            visibility,
            variants_count: _,
            line,
        } => Some((name.clone(), "enum", *line, visibility.clone(), false)),
        AstItem::Trait {
            name,
            visibility,
            line,
        } => Some((name.clone(), "trait", *line, visibility.clone(), false)),
        AstItem::Module {
            name,
            visibility,
            line,
        } => Some((name.clone(), "module", *line, visibility.clone(), false)),
        AstItem::Use { path, line } => {
            Some((path.clone(), "import", *line, "pub".to_string(), false))
        }
        _ => None,
    }
}

/// Extract all symbols from deep context
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn extract_symbols_from_context(
    deep_context: &DeepContext,
    filter: &Option<super::super::SymbolTypeFilter>,
    query: &Option<String>,
) -> Vec<SymbolInfo> {
    let mut all_symbols = Vec::new();

    for ast_ctx in &deep_context.analyses.ast_contexts {
        for item in &ast_ctx.base.items {
            if let Some((name, kind, line, visibility, is_async)) =
                extract_symbol_from_ast_item(item)
            {
                // Apply filters
                if !passes_type_filter(kind, filter) {
                    continue;
                }

                if !passes_query_filter(&name, query) {
                    continue;
                }

                all_symbols.push(SymbolInfo {
                    name,
                    kind: kind.to_string(),
                    file: PathBuf::from(ast_ctx.base.path.clone()),
                    line,
                    visibility,
                    is_async,
                });
            }
        }
    }

    all_symbols
}
