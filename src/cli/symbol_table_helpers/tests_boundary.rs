#![cfg_attr(coverage_nightly, coverage(off))]
//! Edge case and boundary tests for symbol table helpers

#[cfg(test)]
mod tests_boundary {
    use crate::cli::symbol_table_helpers::{
        extract_symbol_from_ast_item, passes_query_filter,
    };
    use crate::services::context::AstItem;

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
