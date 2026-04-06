#![cfg_attr(coverage_nightly, coverage(off))]
//! Statistics and aggregation functions for symbol tables

use super::types::SymbolInfo;

/// Count symbols by type
/// Count symbols by their type/kind
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::cli::symbol_table_helpers::{count_by_type, SymbolInfo};
/// use std::path::PathBuf;
///
/// let symbols = vec![
///     SymbolInfo {
///         name: "main".to_string(),
///         kind: "function".to_string(),
///         file: PathBuf::from("src/main.rs"),
///         line: 1,
///         visibility: "public".to_string(),
///         is_async: false,
///     },
///     SymbolInfo {
///         name: "Config".to_string(),
///         kind: "struct".to_string(),
///         file: PathBuf::from("src/lib.rs"),
///         line: 10,
///         visibility: "public".to_string(),
///         is_async: false,
///     },
/// ];
///
/// let counts = count_by_type(&symbols);
/// assert_eq!(counts.get("function"), Some(&1));
/// assert_eq!(counts.get("struct"), Some(&1));
/// ```
#[must_use]
pub fn count_by_type(symbols: &[SymbolInfo]) -> std::collections::HashMap<String, usize> {
    debug_assert!(!symbols.is_empty(), "symbols must not be empty");
    let mut counts = std::collections::HashMap::with_capacity(64);
    for symbol in symbols {
        *counts.entry(symbol.kind.clone()).or_insert(0) += 1;
    }
    counts
}

/// Count symbols by visibility
#[must_use]
pub fn count_by_visibility(symbols: &[SymbolInfo]) -> std::collections::HashMap<String, usize> {
    debug_assert!(!symbols.is_empty(), "symbols must not be empty");
    let mut counts = std::collections::HashMap::with_capacity(64);
    for symbol in symbols {
        *counts.entry(symbol.visibility.clone()).or_insert(0) += 1;
    }
    counts
}
