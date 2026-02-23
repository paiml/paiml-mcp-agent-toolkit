#![cfg_attr(coverage_nightly, coverage(off))]
//! Unit tests for symbol extraction, filtering, and statistics

#[cfg(test)]
mod tests {
    use crate::cli::symbol_table_helpers::{
        count_by_type, count_by_visibility, extract_symbol_from_ast_item, passes_query_filter,
        passes_type_filter, SymbolInfo,
    };
    use crate::cli::SymbolTypeFilter;
    use crate::services::context::AstItem;
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

}
