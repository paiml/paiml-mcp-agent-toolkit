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
pub fn passes_type_filter(kind: &str, filter: &Option<super::SymbolTypeFilter>) -> bool {
    match filter {
        Some(super::SymbolTypeFilter::Functions) => kind == "function",
        Some(super::SymbolTypeFilter::Classes) => kind == "class",
        Some(super::SymbolTypeFilter::Types) => matches!(kind, "struct" | "enum" | "trait"),
        Some(super::SymbolTypeFilter::Variables) => false, // Not implemented yet
        Some(super::SymbolTypeFilter::Modules) => kind == "module",
        Some(super::SymbolTypeFilter::All) | None => true,
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

/// Extract all symbols from deep context
#[must_use]
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
    let mut counts = std::collections::HashMap::with_capacity(64);
    for symbol in symbols {
        *counts.entry(symbol.kind.clone()).or_insert(0) += 1;
    }
    counts
}

/// Count symbols by visibility
#[must_use]
pub fn count_by_visibility(symbols: &[SymbolInfo]) -> std::collections::HashMap<String, usize> {
    let mut counts = std::collections::HashMap::with_capacity(64);
    for symbol in symbols {
        *counts.entry(symbol.visibility.clone()).or_insert(0) += 1;
    }
    counts
}

/// Format symbol table summary
#[must_use]
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
#[must_use]
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
    use super::*;
    use crate::cli::SymbolTypeFilter;
    use crate::services::context::AstItem;
    use crate::services::deep_context::{
        AnalysisResults, AnnotatedFileTree, ContextMetadata, DeepContext, DefectAnnotations,
        DefectSummary, EnhancedFileContext, QualityScorecard,
    };
    use std::path::PathBuf;

    // ============================================================
    // Tests for extract_symbol_from_ast_item
    // ============================================================

    #[test]
    fn test_extract_function_symbol() {
        let item = AstItem::Function {
            name: "my_function".to_string(),
            visibility: "pub".to_string(),
            is_async: false,
            line: 42,
        };

        let result = extract_symbol_from_ast_item(&item);
        assert!(result.is_some());

        let (name, kind, line, visibility, is_async) = result.unwrap();
        assert_eq!(name, "my_function");
        assert_eq!(kind, "function");
        assert_eq!(line, 42);
        assert_eq!(visibility, "pub");
        assert!(!is_async);
    }

    #[test]
    fn test_extract_async_function_symbol() {
        let item = AstItem::Function {
            name: "async_handler".to_string(),
            visibility: "pub(crate)".to_string(),
            is_async: true,
            line: 100,
        };

        let result = extract_symbol_from_ast_item(&item);
        assert!(result.is_some());

        let (name, kind, line, visibility, is_async) = result.unwrap();
        assert_eq!(name, "async_handler");
        assert_eq!(kind, "function");
        assert_eq!(line, 100);
        assert_eq!(visibility, "pub(crate)");
        assert!(is_async);
    }

    #[test]
    fn test_extract_struct_symbol() {
        let item = AstItem::Struct {
            name: "MyStruct".to_string(),
            visibility: "pub".to_string(),
            fields_count: 5,
            derives: vec!["Debug".to_string(), "Clone".to_string()],
            line: 10,
        };

        let result = extract_symbol_from_ast_item(&item);
        assert!(result.is_some());

        let (name, kind, line, visibility, is_async) = result.unwrap();
        assert_eq!(name, "MyStruct");
        assert_eq!(kind, "struct");
        assert_eq!(line, 10);
        assert_eq!(visibility, "pub");
        assert!(!is_async); // Structs are never async
    }

    #[test]
    fn test_extract_enum_symbol() {
        let item = AstItem::Enum {
            name: "Status".to_string(),
            visibility: "pub".to_string(),
            variants_count: 3,
            line: 25,
        };

        let result = extract_symbol_from_ast_item(&item);
        assert!(result.is_some());

        let (name, kind, line, visibility, is_async) = result.unwrap();
        assert_eq!(name, "Status");
        assert_eq!(kind, "enum");
        assert_eq!(line, 25);
        assert_eq!(visibility, "pub");
        assert!(!is_async);
    }

    #[test]
    fn test_extract_trait_symbol() {
        let item = AstItem::Trait {
            name: "Processor".to_string(),
            visibility: "pub".to_string(),
            line: 50,
        };

        let result = extract_symbol_from_ast_item(&item);
        assert!(result.is_some());

        let (name, kind, line, visibility, is_async) = result.unwrap();
        assert_eq!(name, "Processor");
        assert_eq!(kind, "trait");
        assert_eq!(line, 50);
        assert_eq!(visibility, "pub");
        assert!(!is_async);
    }

    #[test]
    fn test_extract_module_symbol() {
        let item = AstItem::Module {
            name: "utils".to_string(),
            visibility: "pub(crate)".to_string(),
            line: 1,
        };

        let result = extract_symbol_from_ast_item(&item);
        assert!(result.is_some());

        let (name, kind, line, visibility, is_async) = result.unwrap();
        assert_eq!(name, "utils");
        assert_eq!(kind, "module");
        assert_eq!(line, 1);
        assert_eq!(visibility, "pub(crate)");
        assert!(!is_async);
    }

    #[test]
    fn test_extract_use_symbol() {
        let item = AstItem::Use {
            path: "std::collections::HashMap".to_string(),
            line: 3,
        };

        let result = extract_symbol_from_ast_item(&item);
        assert!(result.is_some());

        let (name, kind, line, visibility, is_async) = result.unwrap();
        assert_eq!(name, "std::collections::HashMap");
        assert_eq!(kind, "import");
        assert_eq!(line, 3);
        assert_eq!(visibility, "pub");
        assert!(!is_async);
    }

    #[test]
    fn test_extract_impl_returns_none() {
        let item = AstItem::Impl {
            type_name: "MyStruct".to_string(),
            trait_name: Some("Display".to_string()),
            line: 100,
        };

        let result = extract_symbol_from_ast_item(&item);
        assert!(result.is_none());
    }

    // ============================================================
    // Tests for passes_type_filter
    // ============================================================

    #[test]
    fn test_passes_type_filter_functions() {
        assert!(passes_type_filter(
            "function",
            &Some(SymbolTypeFilter::Functions)
        ));
        assert!(!passes_type_filter(
            "struct",
            &Some(SymbolTypeFilter::Functions)
        ));
        assert!(!passes_type_filter(
            "enum",
            &Some(SymbolTypeFilter::Functions)
        ));
        assert!(!passes_type_filter(
            "module",
            &Some(SymbolTypeFilter::Functions)
        ));
        assert!(!passes_type_filter(
            "class",
            &Some(SymbolTypeFilter::Functions)
        ));
    }

    #[test]
    fn test_passes_type_filter_classes() {
        assert!(passes_type_filter(
            "class",
            &Some(SymbolTypeFilter::Classes)
        ));
        assert!(!passes_type_filter(
            "function",
            &Some(SymbolTypeFilter::Classes)
        ));
        assert!(!passes_type_filter(
            "struct",
            &Some(SymbolTypeFilter::Classes)
        ));
    }

    #[test]
    fn test_passes_type_filter_types() {
        assert!(passes_type_filter("struct", &Some(SymbolTypeFilter::Types)));
        assert!(passes_type_filter("enum", &Some(SymbolTypeFilter::Types)));
        assert!(passes_type_filter("trait", &Some(SymbolTypeFilter::Types)));
        assert!(!passes_type_filter(
            "function",
            &Some(SymbolTypeFilter::Types)
        ));
        assert!(!passes_type_filter(
            "module",
            &Some(SymbolTypeFilter::Types)
        ));
    }

    #[test]
    fn test_passes_type_filter_modules() {
        assert!(passes_type_filter(
            "module",
            &Some(SymbolTypeFilter::Modules)
        ));
        assert!(!passes_type_filter(
            "function",
            &Some(SymbolTypeFilter::Modules)
        ));
        assert!(!passes_type_filter(
            "struct",
            &Some(SymbolTypeFilter::Modules)
        ));
    }

    #[test]
    fn test_passes_type_filter_variables_always_false() {
        // Variables filter is not implemented yet, always returns false
        assert!(!passes_type_filter(
            "variable",
            &Some(SymbolTypeFilter::Variables)
        ));
        assert!(!passes_type_filter(
            "const",
            &Some(SymbolTypeFilter::Variables)
        ));
        assert!(!passes_type_filter(
            "function",
            &Some(SymbolTypeFilter::Variables)
        ));
    }

    #[test]
    fn test_passes_type_filter_all() {
        assert!(passes_type_filter("function", &Some(SymbolTypeFilter::All)));
        assert!(passes_type_filter("struct", &Some(SymbolTypeFilter::All)));
        assert!(passes_type_filter("enum", &Some(SymbolTypeFilter::All)));
        assert!(passes_type_filter("module", &Some(SymbolTypeFilter::All)));
        assert!(passes_type_filter("anything", &Some(SymbolTypeFilter::All)));
    }

    #[test]
    fn test_passes_type_filter_none() {
        assert!(passes_type_filter("function", &None));
        assert!(passes_type_filter("struct", &None));
        assert!(passes_type_filter("anything", &None));
    }

    // ============================================================
    // Tests for passes_query_filter
    // ============================================================

    #[test]
    fn test_passes_query_filter_exact_match() {
        assert!(passes_query_filter(
            "my_function",
            &Some("my_function".to_string())
        ));
    }

    #[test]
    fn test_passes_query_filter_partial_match() {
        assert!(passes_query_filter(
            "my_function",
            &Some("function".to_string())
        ));
        assert!(passes_query_filter("my_function", &Some("my".to_string())));
        assert!(passes_query_filter(
            "hello_world",
            &Some("hello".to_string())
        ));
    }

    #[test]
    fn test_passes_query_filter_case_insensitive() {
        assert!(passes_query_filter(
            "MyFunction",
            &Some("myfunction".to_string())
        ));
        assert!(passes_query_filter(
            "myfunction",
            &Some("MYFUNCTION".to_string())
        ));
        assert!(passes_query_filter(
            "HelloWorld",
            &Some("helloworld".to_string())
        ));
        assert!(passes_query_filter(
            "UPPERCASE",
            &Some("uppercase".to_string())
        ));
    }

    #[test]
    fn test_passes_query_filter_no_match() {
        assert!(!passes_query_filter("hello", &Some("goodbye".to_string())));
        assert!(!passes_query_filter("foo", &Some("bar".to_string())));
    }

    #[test]
    fn test_passes_query_filter_none() {
        assert!(passes_query_filter("anything", &None));
        assert!(passes_query_filter("", &None));
    }

    #[test]
    fn test_passes_query_filter_empty_query() {
        assert!(passes_query_filter("anything", &Some("".to_string())));
    }

    // ============================================================
    // Tests for SymbolInfo
    // ============================================================

    #[test]
    fn test_symbol_info_creation() {
        let symbol = SymbolInfo {
            name: "test_fn".to_string(),
            kind: "function".to_string(),
            file: PathBuf::from("src/lib.rs"),
            line: 10,
            visibility: "pub".to_string(),
            is_async: true,
        };

        assert_eq!(symbol.name, "test_fn");
        assert_eq!(symbol.kind, "function");
        assert_eq!(symbol.file, PathBuf::from("src/lib.rs"));
        assert_eq!(symbol.line, 10);
        assert_eq!(symbol.visibility, "pub");
        assert!(symbol.is_async);
    }

    #[test]
    fn test_symbol_info_clone() {
        let symbol = SymbolInfo {
            name: "test".to_string(),
            kind: "struct".to_string(),
            file: PathBuf::from("src/mod.rs"),
            line: 5,
            visibility: "pub(crate)".to_string(),
            is_async: false,
        };

        let cloned = symbol.clone();
        assert_eq!(symbol.name, cloned.name);
        assert_eq!(symbol.kind, cloned.kind);
        assert_eq!(symbol.file, cloned.file);
    }

    // ============================================================
    // Tests for count_by_type
    // ============================================================

    #[test]
    fn test_count_by_type_empty() {
        let symbols: Vec<SymbolInfo> = vec![];
        let counts = count_by_type(&symbols);
        assert!(counts.is_empty());
    }

    #[test]
    fn test_count_by_type_single() {
        let symbols = vec![SymbolInfo {
            name: "main".to_string(),
            kind: "function".to_string(),
            file: PathBuf::from("src/main.rs"),
            line: 1,
            visibility: "pub".to_string(),
            is_async: false,
        }];

        let counts = count_by_type(&symbols);
        assert_eq!(counts.get("function"), Some(&1));
        assert_eq!(counts.len(), 1);
    }

    #[test]
    fn test_count_by_type_multiple() {
        let symbols = vec![
            SymbolInfo {
                name: "fn1".to_string(),
                kind: "function".to_string(),
                file: PathBuf::from("src/lib.rs"),
                line: 1,
                visibility: "pub".to_string(),
                is_async: false,
            },
            SymbolInfo {
                name: "fn2".to_string(),
                kind: "function".to_string(),
                file: PathBuf::from("src/lib.rs"),
                line: 10,
                visibility: "pub".to_string(),
                is_async: false,
            },
            SymbolInfo {
                name: "MyStruct".to_string(),
                kind: "struct".to_string(),
                file: PathBuf::from("src/lib.rs"),
                line: 20,
                visibility: "pub".to_string(),
                is_async: false,
            },
            SymbolInfo {
                name: "Status".to_string(),
                kind: "enum".to_string(),
                file: PathBuf::from("src/lib.rs"),
                line: 30,
                visibility: "pub".to_string(),
                is_async: false,
            },
        ];

        let counts = count_by_type(&symbols);
        assert_eq!(counts.get("function"), Some(&2));
        assert_eq!(counts.get("struct"), Some(&1));
        assert_eq!(counts.get("enum"), Some(&1));
        assert_eq!(counts.len(), 3);
    }

    // ============================================================
    // Tests for count_by_visibility
    // ============================================================

    #[test]
    fn test_count_by_visibility_empty() {
        let symbols: Vec<SymbolInfo> = vec![];
        let counts = count_by_visibility(&symbols);
        assert!(counts.is_empty());
    }

    #[test]
    fn test_count_by_visibility_single() {
        let symbols = vec![SymbolInfo {
            name: "main".to_string(),
            kind: "function".to_string(),
            file: PathBuf::from("src/main.rs"),
            line: 1,
            visibility: "pub".to_string(),
            is_async: false,
        }];

        let counts = count_by_visibility(&symbols);
        assert_eq!(counts.get("pub"), Some(&1));
    }

    #[test]
    fn test_count_by_visibility_multiple() {
        let symbols = vec![
            SymbolInfo {
                name: "fn1".to_string(),
                kind: "function".to_string(),
                file: PathBuf::from("src/lib.rs"),
                line: 1,
                visibility: "pub".to_string(),
                is_async: false,
            },
            SymbolInfo {
                name: "fn2".to_string(),
                kind: "function".to_string(),
                file: PathBuf::from("src/lib.rs"),
                line: 10,
                visibility: "pub".to_string(),
                is_async: false,
            },
            SymbolInfo {
                name: "private_fn".to_string(),
                kind: "function".to_string(),
                file: PathBuf::from("src/lib.rs"),
                line: 20,
                visibility: "private".to_string(),
                is_async: false,
            },
            SymbolInfo {
                name: "crate_fn".to_string(),
                kind: "function".to_string(),
                file: PathBuf::from("src/lib.rs"),
                line: 30,
                visibility: "pub(crate)".to_string(),
                is_async: false,
            },
        ];

        let counts = count_by_visibility(&symbols);
        assert_eq!(counts.get("pub"), Some(&2));
        assert_eq!(counts.get("private"), Some(&1));
        assert_eq!(counts.get("pub(crate)"), Some(&1));
        assert_eq!(counts.len(), 3);
    }

    // ============================================================
    // Helper function to create mock DeepContext
    // ============================================================

    fn create_mock_deep_context(ast_contexts: Vec<EnhancedFileContext>) -> DeepContext {
        use crate::services::deep_context::{AnnotatedNode, CacheStats, NodeAnnotations, NodeType};
        use chrono::Utc;

        DeepContext {
            metadata: ContextMetadata {
                generated_at: Utc::now(),
                tool_version: "test".to_string(),
                project_root: PathBuf::from("/test/project"),
                cache_stats: CacheStats::default(),
                analysis_duration: std::time::Duration::from_millis(100),
            },
            file_tree: AnnotatedFileTree {
                root: AnnotatedNode {
                    name: "project".to_string(),
                    path: PathBuf::from("/test/project"),
                    node_type: NodeType::Directory,
                    children: vec![],
                    annotations: NodeAnnotations::default(),
                },
                total_files: ast_contexts.len(),
                total_size_bytes: 0,
            },
            analyses: AnalysisResults {
                ast_contexts,
                complexity_report: None,
                churn_analysis: None,
                dependency_graph: None,
                dead_code_results: None,
                duplicate_code_results: None,
                satd_results: None,
                provability_results: None,
                cross_language_refs: vec![],
                big_o_analysis: None,
            },
            quality_scorecard: QualityScorecard {
                overall_health: 80.0,
                complexity_score: 85.0,
                maintainability_index: 75.0,
                modularity_score: 90.0,
                test_coverage: Some(60.0),
                technical_debt_hours: 10.0,
            },
            template_provenance: None,
            defect_summary: DefectSummary::default(),
            hotspots: vec![],
            recommendations: vec![],
            qa_verification: None,
            build_info: None,
            project_overview: None,
        }
    }

    fn create_file_context(path: &str, items: Vec<AstItem>) -> EnhancedFileContext {
        use crate::services::context::FileContext;

        EnhancedFileContext {
            base: FileContext {
                path: path.to_string(),
                language: "rust".to_string(),
                items,
                complexity_metrics: None,
            },
            complexity_metrics: None,
            churn_metrics: None,
            defects: DefectAnnotations {
                dead_code: None,
                technical_debt: vec![],
                complexity_violations: vec![],
                tdg_score: None,
            },
            symbol_id: format!("sym_{}", path.replace('/', "_")),
        }
    }

    // ============================================================
    // Tests for extract_symbols_from_context
    // ============================================================

    #[test]
    fn test_extract_symbols_from_context_empty() {
        let deep_context = create_mock_deep_context(vec![]);
        let symbols = extract_symbols_from_context(&deep_context, &None, &None);
        assert!(symbols.is_empty());
    }

    #[test]
    fn test_extract_symbols_from_context_single_file() {
        let items = vec![
            AstItem::Function {
                name: "main".to_string(),
                visibility: "pub".to_string(),
                is_async: false,
                line: 1,
            },
            AstItem::Struct {
                name: "Config".to_string(),
                visibility: "pub".to_string(),
                fields_count: 3,
                derives: vec![],
                line: 10,
            },
        ];

        let file_context = create_file_context("src/main.rs", items);
        let deep_context = create_mock_deep_context(vec![file_context]);

        let symbols = extract_symbols_from_context(&deep_context, &None, &None);
        assert_eq!(symbols.len(), 2);
        assert_eq!(symbols[0].name, "main");
        assert_eq!(symbols[1].name, "Config");
    }

    #[test]
    fn test_extract_symbols_from_context_with_type_filter() {
        let items = vec![
            AstItem::Function {
                name: "main".to_string(),
                visibility: "pub".to_string(),
                is_async: false,
                line: 1,
            },
            AstItem::Struct {
                name: "Config".to_string(),
                visibility: "pub".to_string(),
                fields_count: 3,
                derives: vec![],
                line: 10,
            },
            AstItem::Enum {
                name: "Status".to_string(),
                visibility: "pub".to_string(),
                variants_count: 2,
                line: 20,
            },
        ];

        let file_context = create_file_context("src/main.rs", items);
        let deep_context = create_mock_deep_context(vec![file_context]);

        // Filter for functions only
        let symbols =
            extract_symbols_from_context(&deep_context, &Some(SymbolTypeFilter::Functions), &None);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "main");

        // Filter for types only
        let symbols =
            extract_symbols_from_context(&deep_context, &Some(SymbolTypeFilter::Types), &None);
        assert_eq!(symbols.len(), 2);
        assert!(symbols.iter().any(|s| s.name == "Config"));
        assert!(symbols.iter().any(|s| s.name == "Status"));
    }

    #[test]
    fn test_extract_symbols_from_context_with_query_filter() {
        let items = vec![
            AstItem::Function {
                name: "process_data".to_string(),
                visibility: "pub".to_string(),
                is_async: false,
                line: 1,
            },
            AstItem::Function {
                name: "handle_error".to_string(),
                visibility: "pub".to_string(),
                is_async: false,
                line: 10,
            },
            AstItem::Function {
                name: "validate".to_string(),
                visibility: "pub".to_string(),
                is_async: false,
                line: 20,
            },
        ];

        let file_context = create_file_context("src/handlers.rs", items);
        let deep_context = create_mock_deep_context(vec![file_context]);

        // Filter for functions containing "handle"
        let symbols =
            extract_symbols_from_context(&deep_context, &None, &Some("handle".to_string()));
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "handle_error");

        // Case insensitive search
        let symbols =
            extract_symbols_from_context(&deep_context, &None, &Some("PROCESS".to_string()));
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "process_data");
    }

    #[test]
    fn test_extract_symbols_from_context_multiple_files() {
        let file1_items = vec![AstItem::Function {
            name: "main".to_string(),
            visibility: "pub".to_string(),
            is_async: false,
            line: 1,
        }];

        let file2_items = vec![
            AstItem::Struct {
                name: "Config".to_string(),
                visibility: "pub".to_string(),
                fields_count: 2,
                derives: vec![],
                line: 1,
            },
            AstItem::Function {
                name: "new".to_string(),
                visibility: "pub".to_string(),
                is_async: false,
                line: 10,
            },
        ];

        let file1 = create_file_context("src/main.rs", file1_items);
        let file2 = create_file_context("src/config.rs", file2_items);
        let deep_context = create_mock_deep_context(vec![file1, file2]);

        let symbols = extract_symbols_from_context(&deep_context, &None, &None);
        assert_eq!(symbols.len(), 3);

        // Verify files are correctly associated
        let main_symbols: Vec<_> = symbols
            .iter()
            .filter(|s| s.file == PathBuf::from("src/main.rs"))
            .collect();
        assert_eq!(main_symbols.len(), 1);

        let config_symbols: Vec<_> = symbols
            .iter()
            .filter(|s| s.file == PathBuf::from("src/config.rs"))
            .collect();
        assert_eq!(config_symbols.len(), 2);
    }

    #[test]
    fn test_extract_symbols_from_context_skips_impl() {
        let items = vec![
            AstItem::Function {
                name: "main".to_string(),
                visibility: "pub".to_string(),
                is_async: false,
                line: 1,
            },
            AstItem::Impl {
                type_name: "MyStruct".to_string(),
                trait_name: None,
                line: 10,
            },
        ];

        let file_context = create_file_context("src/main.rs", items);
        let deep_context = create_mock_deep_context(vec![file_context]);

        let symbols = extract_symbols_from_context(&deep_context, &None, &None);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "main");
    }

    // ============================================================
    // Tests for format_symbol_table_summary
    // ============================================================

    #[test]
    fn test_format_symbol_table_summary_empty() {
        let deep_context = create_mock_deep_context(vec![]);
        let symbols: Vec<SymbolInfo> = vec![];

        let output = format_symbol_table_summary(&symbols, &deep_context);

        assert!(output.contains("Symbol Table Summary"));
        assert!(output.contains("Total symbols: 0"));
        assert!(output.contains("Files analyzed: 0"));
    }

    #[test]
    fn test_format_symbol_table_summary_with_symbols() {
        let items = vec![
            AstItem::Function {
                name: "main".to_string(),
                visibility: "pub".to_string(),
                is_async: false,
                line: 1,
            },
            AstItem::Struct {
                name: "Config".to_string(),
                visibility: "pub".to_string(),
                fields_count: 2,
                derives: vec![],
                line: 10,
            },
        ];

        let file_context = create_file_context("src/main.rs", items);
        let deep_context = create_mock_deep_context(vec![file_context]);
        let symbols = extract_symbols_from_context(&deep_context, &None, &None);

        let output = format_symbol_table_summary(&symbols, &deep_context);

        assert!(output.contains("Symbol Table Summary"));
        assert!(output.contains("Total symbols: 2"));
        assert!(output.contains("Files analyzed: 1"));
        assert!(output.contains("Symbols by type:"));
        assert!(output.contains("function: 1"));
        assert!(output.contains("struct: 1"));
        assert!(output.contains("Symbols by visibility:"));
        assert!(output.contains("pub: 2"));
        assert!(output.contains("Top 10 most referenced files:"));
        assert!(output.contains("main.rs: 2 symbols"));
    }

    // ============================================================
    // Tests for format_symbol_table_detailed
    // ============================================================

    #[test]
    fn test_format_symbol_table_detailed_empty() {
        let symbols: Vec<SymbolInfo> = vec![];

        let output = format_symbol_table_detailed(&symbols);

        assert!(output.contains("Symbol Table"));
        assert!(output.contains("============"));
    }

    #[test]
    fn test_format_symbol_table_detailed_with_symbols() {
        let symbols = vec![
            SymbolInfo {
                name: "main".to_string(),
                kind: "function".to_string(),
                file: PathBuf::from("src/main.rs"),
                line: 1,
                visibility: "pub".to_string(),
                is_async: false,
            },
            SymbolInfo {
                name: "async_handler".to_string(),
                kind: "function".to_string(),
                file: PathBuf::from("src/main.rs"),
                line: 10,
                visibility: "pub".to_string(),
                is_async: true,
            },
        ];

        let output = format_symbol_table_detailed(&symbols);

        assert!(output.contains("Symbol Table"));
        assert!(output.contains("src/main.rs"));
        assert!(output.contains("L0001: pub function main"));
        assert!(output.contains("L0010: pub function async_handler (async)"));
    }

    #[test]
    fn test_format_symbol_table_detailed_multiple_files() {
        let symbols = vec![
            SymbolInfo {
                name: "main".to_string(),
                kind: "function".to_string(),
                file: PathBuf::from("src/main.rs"),
                line: 1,
                visibility: "pub".to_string(),
                is_async: false,
            },
            SymbolInfo {
                name: "Config".to_string(),
                kind: "struct".to_string(),
                file: PathBuf::from("src/config.rs"),
                line: 5,
                visibility: "pub".to_string(),
                is_async: false,
            },
        ];

        let output = format_symbol_table_detailed(&symbols);

        assert!(output.contains("src/main.rs"));
        assert!(output.contains("src/config.rs"));
        assert!(output.contains("L0001: pub function main"));
        assert!(output.contains("L0005: pub struct Config"));
    }

    // ============================================================
    // Tests for format_symbol_table_csv
    // ============================================================

    #[test]
    fn test_format_symbol_table_csv_empty() {
        let symbols: Vec<SymbolInfo> = vec![];

        let output = format_symbol_table_csv(&symbols);

        assert_eq!(output, "name,kind,file,line,visibility,is_async\n");
    }

    #[test]
    fn test_format_symbol_table_csv_with_symbols() {
        let symbols = vec![
            SymbolInfo {
                name: "main".to_string(),
                kind: "function".to_string(),
                file: PathBuf::from("src/main.rs"),
                line: 1,
                visibility: "pub".to_string(),
                is_async: false,
            },
            SymbolInfo {
                name: "async_handler".to_string(),
                kind: "function".to_string(),
                file: PathBuf::from("src/handlers.rs"),
                line: 10,
                visibility: "pub(crate)".to_string(),
                is_async: true,
            },
        ];

        let output = format_symbol_table_csv(&symbols);
        let lines: Vec<&str> = output.lines().collect();

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "name,kind,file,line,visibility,is_async");
        assert_eq!(lines[1], "main,function,src/main.rs,1,pub,false");
        assert_eq!(
            lines[2],
            "async_handler,function,src/handlers.rs,10,pub(crate),true"
        );
    }

    // ============================================================
    // Edge cases and boundary tests
    // ============================================================

    #[test]
    fn test_symbol_with_empty_name() {
        let item = AstItem::Function {
            name: "".to_string(),
            visibility: "pub".to_string(),
            is_async: false,
            line: 1,
        };

        let result = extract_symbol_from_ast_item(&item);
        assert!(result.is_some());

        let (name, _, _, _, _) = result.unwrap();
        assert_eq!(name, "");
    }

    #[test]
    fn test_symbol_with_line_zero() {
        let item = AstItem::Function {
            name: "test".to_string(),
            visibility: "pub".to_string(),
            is_async: false,
            line: 0,
        };

        let result = extract_symbol_from_ast_item(&item);
        assert!(result.is_some());

        let (_, _, line, _, _) = result.unwrap();
        assert_eq!(line, 0);
    }

    #[test]
    fn test_symbol_with_large_line_number() {
        let item = AstItem::Function {
            name: "test".to_string(),
            visibility: "pub".to_string(),
            is_async: false,
            line: usize::MAX,
        };

        let result = extract_symbol_from_ast_item(&item);
        assert!(result.is_some());

        let (_, _, line, _, _) = result.unwrap();
        assert_eq!(line, usize::MAX);
    }

    #[test]
    fn test_special_characters_in_query() {
        assert!(passes_query_filter(
            "fn_with_underscore",
            &Some("_with_".to_string())
        ));
        assert!(passes_query_filter(
            "CamelCase123",
            &Some("123".to_string())
        ));
    }

    #[test]
    fn test_unicode_in_names() {
        let item = AstItem::Function {
            name: "calculer_somme".to_string(),
            visibility: "pub".to_string(),
            is_async: false,
            line: 1,
        };

        let result = extract_symbol_from_ast_item(&item);
        assert!(result.is_some());

        let (name, _, _, _, _) = result.unwrap();
        assert_eq!(name, "calculer_somme");

        // Query with unicode
        assert!(passes_query_filter(
            "calculer_somme",
            &Some("somme".to_string())
        ));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::cli::SymbolTypeFilter;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_passes_query_filter_case_insensitivity(
            name in "[a-zA-Z][a-zA-Z0-9_]{0,20}",
            query in "[a-zA-Z][a-zA-Z0-9_]{0,10}"
        ) {
            // If the name contains the query (case insensitive), the filter should pass
            let contains_case_insensitive = name.to_lowercase().contains(&query.to_lowercase());
            let result = passes_query_filter(&name, &Some(query));
            prop_assert_eq!(result, contains_case_insensitive);
        }

        #[test]
        fn test_passes_query_filter_none_always_passes(name in ".*") {
            prop_assert!(passes_query_filter(&name, &None));
        }

        #[test]
        fn test_count_by_type_sum_equals_total(
            count in 1usize..20,
        ) {
            let mut symbols = Vec::with_capacity(count);
            let kinds = ["function", "struct", "enum", "trait", "module"];

            for i in 0..count {
                symbols.push(SymbolInfo {
                    name: format!("symbol_{}", i),
                    kind: kinds[i % kinds.len()].to_string(),
                    file: std::path::PathBuf::from("test.rs"),
                    line: i,
                    visibility: "pub".to_string(),
                    is_async: false,
                });
            }

            let counts = count_by_type(&symbols);
            let total: usize = counts.values().sum();
            prop_assert_eq!(total, count);
        }

        #[test]
        fn test_count_by_visibility_sum_equals_total(
            count in 1usize..20,
        ) {
            let mut symbols = Vec::with_capacity(count);
            let visibilities = ["pub", "pub(crate)", "private", "pub(super)"];

            for i in 0..count {
                symbols.push(SymbolInfo {
                    name: format!("symbol_{}", i),
                    kind: "function".to_string(),
                    file: std::path::PathBuf::from("test.rs"),
                    line: i,
                    visibility: visibilities[i % visibilities.len()].to_string(),
                    is_async: false,
                });
            }

            let counts = count_by_visibility(&symbols);
            let total: usize = counts.values().sum();
            prop_assert_eq!(total, count);
        }

        #[test]
        fn test_passes_type_filter_all_or_none_always_passes(kind in "[a-z]+") {
            prop_assert!(passes_type_filter(&kind, &None));
            prop_assert!(passes_type_filter(&kind, &Some(SymbolTypeFilter::All)));
        }

        #[test]
        fn test_csv_output_has_correct_line_count(count in 0usize..50) {
            let mut symbols = Vec::with_capacity(count);

            for i in 0..count {
                symbols.push(SymbolInfo {
                    name: format!("sym_{}", i),
                    kind: "function".to_string(),
                    file: std::path::PathBuf::from("test.rs"),
                    line: i,
                    visibility: "pub".to_string(),
                    is_async: false,
                });
            }

            let csv = format_symbol_table_csv(&symbols);
            let line_count = csv.lines().count();
            // Header + one line per symbol
            prop_assert_eq!(line_count, count + 1);
        }
    }
}
