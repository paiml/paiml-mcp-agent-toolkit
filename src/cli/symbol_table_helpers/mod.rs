#![cfg_attr(coverage_nightly, coverage(off))]
//! Helper functions for symbol table analysis to reduce complexity

mod extraction;
mod filters;
mod formatting;
mod property_tests;
mod stats;
mod tests;
mod tests_boundary;
mod tests_context;
mod types;

pub use extraction::{extract_symbol_from_ast_item, extract_symbols_from_context};
pub use filters::{passes_query_filter, passes_type_filter};
pub use formatting::{
    format_symbol_table_csv, format_symbol_table_detailed, format_symbol_table_summary,
};
pub use stats::{count_by_type, count_by_visibility};
pub use types::SymbolInfo;
