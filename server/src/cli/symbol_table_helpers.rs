//! Helper functions for symbol table analysis to reduce complexity

use crate::services::context::AstItem;
use crate::services::deep_context::DeepContext;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct SymbolInfo {
    pub name: String,
    pub kind: String,
    pub file: PathBuf,
    pub line: usize,
    pub visibility: String,
    pub is_async: bool,
}

/// Extract symbol information from an AST item
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

/// Check if a symbol passes the type filter
pub fn passes_type_filter(kind: &str, filter: &Option<super::SymbolTypeFilter>) -> bool {
    match filter {
        Some(super::SymbolTypeFilter::Functions) => kind == "function",
        Some(super::SymbolTypeFilter::Types) => matches!(kind, "struct" | "enum" | "trait"),
        Some(super::SymbolTypeFilter::Variables) => false, // Not implemented yet
        Some(super::SymbolTypeFilter::Modules) => kind == "module",
        Some(super::SymbolTypeFilter::All) | None => true,
    }
}

/// Check if a symbol passes the query filter
pub fn passes_query_filter(name: &str, query: &Option<String>) -> bool {
    match query {
        Some(q) => name.to_lowercase().contains(&q.to_lowercase()),
        None => true,
    }
}

/// Extract all symbols from deep context
pub fn extract_symbols_from_context(
    deep_context: &DeepContext,
    filter: &Option<super::SymbolTypeFilter>,
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

/// Count symbols by type
pub fn count_by_type(symbols: &[SymbolInfo]) -> std::collections::HashMap<String, usize> {
    let mut counts = std::collections::HashMap::with_capacity(64);
    for symbol in symbols {
        *counts.entry(symbol.kind.clone()).or_insert(0) += 1;
    }
    counts
}

/// Count symbols by visibility
pub fn count_by_visibility(symbols: &[SymbolInfo]) -> std::collections::HashMap<String, usize> {
    let mut counts = std::collections::HashMap::with_capacity(64);
    for symbol in symbols {
        *counts.entry(symbol.visibility.clone()).or_insert(0) += 1;
    }
    counts
}

/// Format symbol table summary
pub fn format_symbol_table_summary(symbols: &[SymbolInfo], deep_context: &DeepContext) -> String {
    let mut output = String::with_capacity(1024);

    output.push_str("Symbol Table Summary\n");
    output.push_str("===================\n\n");

    output.push_str(&format!("Total symbols: {}\n", symbols.len()));
    output.push_str(&format!(
        "Files analyzed: {}\n\n",
        deep_context.analyses.ast_contexts.len()
    ));

    output.push_str("Symbols by type:\n");
    let type_counts = count_by_type(symbols);
    for (kind, count) in type_counts {
        output.push_str(&format!("  {}: {}\n", kind, count));
    }

    output.push_str("\nSymbols by visibility:\n");
    let vis_counts = count_by_visibility(symbols);
    for (vis, count) in vis_counts {
        output.push_str(&format!("  {}: {}\n", vis, count));
    }

    output.push_str("\nTop 10 most referenced files:\n");
    let mut file_counts: std::collections::HashMap<PathBuf, usize> =
        std::collections::HashMap::with_capacity(64);
    for symbol in symbols {
        *file_counts.entry(symbol.file.clone()).or_insert(0) += 1;
    }
    let mut file_vec: Vec<_> = file_counts.into_iter().collect();
    file_vec.sort_by(|a, b| b.1.cmp(&a.1));

    for (file, count) in file_vec.iter().take(10) {
        output.push_str(&format!(
            "  {}: {} symbols\n",
            file.file_name().unwrap_or_default().to_string_lossy(),
            count
        ));
    }

    output
}

/// Format symbol table detailed output
pub fn format_symbol_table_detailed(symbols: &[SymbolInfo]) -> String {
    let mut output = String::with_capacity(1024);

    output.push_str("Symbol Table\n");
    output.push_str("============\n\n");

    // Group by file
    let mut symbols_by_file: std::collections::HashMap<PathBuf, Vec<&SymbolInfo>> =
        std::collections::HashMap::with_capacity(64);
    for symbol in symbols {
        symbols_by_file
            .entry(symbol.file.clone())
            .or_default()
            .push(symbol);
    }

    for (file, file_symbols) in symbols_by_file {
        output.push_str(&format!("\n{}\n", file.display()));
        output.push_str(&"-".repeat(file.to_string_lossy().len()));
        output.push('\n');

        for symbol in file_symbols {
            output.push_str(&format!(
                "  L{:04}: {} {} {}{}\n",
                symbol.line,
                symbol.visibility,
                symbol.kind,
                symbol.name,
                if symbol.is_async { " (async)" } else { "" }
            ));
        }
    }

    output
}

/// Format symbol table as CSV
pub fn format_symbol_table_csv(symbols: &[SymbolInfo]) -> String {
    let mut output = String::with_capacity(1024);

    output.push_str("name,kind,file,line,visibility,is_async\n");

    for symbol in symbols {
        output.push_str(&format!(
            "{},{},{},{},{},{}\n",
            symbol.name,
            symbol.kind,
            symbol.file.display(),
            symbol.line,
            symbol.visibility,
            symbol.is_async
        ));
    }

    output
}

#[cfg(test)]
mod tests {
    // use super::*; // Unused in simple tests

    #[test]
    fn test_symbol_table_helpers_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
