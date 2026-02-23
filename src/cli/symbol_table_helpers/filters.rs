#![cfg_attr(coverage_nightly, coverage(off))]
//! Filter functions for symbol table queries

/// Check if a symbol passes the type filter
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::cli::symbol_table_helpers::passes_type_filter;
/// use pmat::cli::SymbolTypeFilter;
///
/// assert!(passes_type_filter("function", &Some(SymbolTypeFilter::Functions)));
/// assert!(!passes_type_filter("class", &Some(SymbolTypeFilter::Functions)));
/// assert!(passes_type_filter("anything", &None));
/// ```
/// Checks if a symbol kind passes the type filter
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::cli::symbol_table_helpers::passes_type_filter;
/// use pmat::cli::SymbolTypeFilter;
///
/// // Function passes function filter
/// assert!(passes_type_filter("function", &Some(SymbolTypeFilter::Functions)));
///
/// // Struct passes types filter
/// assert!(passes_type_filter("struct", &Some(SymbolTypeFilter::Types)));
///
/// // Anything passes when no filter
/// assert!(passes_type_filter("enum", &None));
///
/// // Function doesn't pass classes filter
/// assert!(!passes_type_filter("function", &Some(SymbolTypeFilter::Classes)));
/// ```
#[must_use]
pub fn passes_type_filter(kind: &str, filter: &Option<super::super::SymbolTypeFilter>) -> bool {
    match filter {
        Some(super::super::SymbolTypeFilter::Functions) => kind == "function",
        Some(super::super::SymbolTypeFilter::Classes) => kind == "class",
        Some(super::super::SymbolTypeFilter::Types) => matches!(kind, "struct" | "enum" | "trait"),
        Some(super::super::SymbolTypeFilter::Variables) => false, // Not implemented yet
        Some(super::super::SymbolTypeFilter::Modules) => kind == "module",
        Some(super::super::SymbolTypeFilter::All) | None => true,
    }
}

/// Check if a symbol passes the query filter
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::cli::symbol_table_helpers::passes_query_filter;
///
/// assert!(passes_query_filter("hello_world", &Some("hello".to_string())));
/// assert!(!passes_query_filter("goodbye", &Some("hello".to_string())));
/// assert!(passes_query_filter("anything", &None));
/// ```
#[must_use]
pub fn passes_query_filter(name: &str, query: &Option<String>) -> bool {
    match query {
        Some(q) => name.to_lowercase().contains(&q.to_lowercase()),
        None => true,
    }
}
