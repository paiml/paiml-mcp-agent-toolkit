#![cfg_attr(coverage_nightly, coverage(off))]
//! Tests for context extraction and symbol table formatting

#[cfg(test)]
mod tests_context {
    use crate::cli::symbol_table_helpers::{
        extract_symbols_from_context, format_symbol_table_csv, format_symbol_table_detailed,
        format_symbol_table_summary, SymbolInfo,
    };
    use crate::cli::SymbolTypeFilter;
    use crate::services::context::AstItem;
    use crate::services::deep_context::{
        AnalysisResults, AnnotatedFileTree, ContextMetadata, DeepContext, DefectAnnotations,
        DefectSummary, EnhancedFileContext, QualityScorecard,
    };
    use std::path::PathBuf;

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
                overall_health: Some(80.0),
                complexity_score: Some(85.0),
                maintainability_index: Some(75.0),
                modularity_score: Some(90.0),
                test_coverage: Some(60.0),
                technical_debt_hours: Some(10.0),
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
}
