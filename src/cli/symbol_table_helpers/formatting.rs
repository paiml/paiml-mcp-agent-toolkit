#![cfg_attr(coverage_nightly, coverage(off))]
//! Output formatting functions for symbol tables

use crate::services::deep_context::DeepContext;
use std::path::PathBuf;

use super::stats::{count_by_type, count_by_visibility};
use super::types::SymbolInfo;

/// Format symbol table summary
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
        output.push_str(&format!("  {kind}: {count}\n"));
    }

    output.push_str("\nSymbols by visibility:\n");
    let vis_counts = count_by_visibility(symbols);
    for (vis, count) in vis_counts {
        output.push_str(&format!("  {vis}: {count}\n"));
    }

    output.push_str("\nTop 10 most referenced files:\n");
    let mut file_counts: std::collections::HashMap<PathBuf, usize> =
        std::collections::HashMap::with_capacity(64);
    for symbol in symbols {
        *file_counts.entry(symbol.file.clone()).or_insert(0) += 1;
    }
    let mut file_vec: Vec<_> = file_counts.into_iter().collect();
    file_vec.sort_by_key(|b| std::cmp::Reverse(b.1));

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
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
