// Included from ast_typescript.rs — NO `use` imports, NO `#!` inner attributes
// Coverage and property tests for TypeScript AST types

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;

    // ========================================================================
    // TypeScriptParser tests
    // ========================================================================

    #[test]
    fn test_typescript_parser_new() {
        let parser = TypeScriptParser::new();
        // TypeScriptParser is a unit struct, just verify it creates
        let _ = parser;
    }

    #[test]
    fn test_typescript_parser_default() {
        let parser = TypeScriptParser::default();
        // TypeScriptParser is a unit struct, just verify it creates
        let _ = parser;
    }

    #[test]
    fn test_typescript_parser_new_equals_default() {
        let parser1 = TypeScriptParser::new();
        let parser2 = TypeScriptParser::default();
        // Both should create equivalent instances
        let _ = (parser1, parser2);
    }

    // ========================================================================
    // TypeScriptSymbol tests
    // ========================================================================

    #[test]
    fn test_typescript_symbol_function() {
        let symbol = TypeScriptSymbol {
            name: "myFunction".to_string(),
            kind: SymbolKind::Function,
            line: 10,
            is_exported: true,
            is_async: false,
            variants_count: 0,
            fields_count: 0,
        };

        assert_eq!(symbol.name, "myFunction");
        assert!(matches!(symbol.kind, SymbolKind::Function));
        assert_eq!(symbol.line, 10);
        assert!(symbol.is_exported);
        assert!(!symbol.is_async);
        assert_eq!(symbol.variants_count, 0);
        assert_eq!(symbol.fields_count, 0);
    }

    #[test]
    fn test_typescript_symbol_async_function() {
        let symbol = TypeScriptSymbol {
            name: "asyncHandler".to_string(),
            kind: SymbolKind::Function,
            line: 25,
            is_exported: false,
            is_async: true,
            variants_count: 0,
            fields_count: 0,
        };

        assert!(symbol.is_async);
        assert!(!symbol.is_exported);
    }

    #[test]
    fn test_typescript_symbol_class() {
        let symbol = TypeScriptSymbol {
            name: "MyClass".to_string(),
            kind: SymbolKind::Class,
            line: 1,
            is_exported: true,
            is_async: false,
            variants_count: 0,
            fields_count: 5,
        };

        assert!(matches!(symbol.kind, SymbolKind::Class));
        assert_eq!(symbol.fields_count, 5);
    }

    #[test]
    fn test_typescript_symbol_interface() {
        let symbol = TypeScriptSymbol {
            name: "IUser".to_string(),
            kind: SymbolKind::Interface,
            line: 15,
            is_exported: true,
            is_async: false,
            variants_count: 0,
            fields_count: 3,
        };

        assert!(matches!(symbol.kind, SymbolKind::Interface));
    }

    #[test]
    fn test_typescript_symbol_type_alias() {
        let symbol = TypeScriptSymbol {
            name: "UserId".to_string(),
            kind: SymbolKind::TypeAlias,
            line: 5,
            is_exported: false,
            is_async: false,
            variants_count: 0,
            fields_count: 0,
        };

        assert!(matches!(symbol.kind, SymbolKind::TypeAlias));
    }

    #[test]
    fn test_typescript_symbol_enum() {
        let symbol = TypeScriptSymbol {
            name: "Status".to_string(),
            kind: SymbolKind::Enum,
            line: 20,
            is_exported: true,
            is_async: false,
            variants_count: 4,
            fields_count: 0,
        };

        assert!(matches!(symbol.kind, SymbolKind::Enum));
        assert_eq!(symbol.variants_count, 4);
    }

    #[test]
    fn test_typescript_symbol_variable() {
        let symbol = TypeScriptSymbol {
            name: "API_URL".to_string(),
            kind: SymbolKind::Variable,
            line: 1,
            is_exported: true,
            is_async: false,
            variants_count: 0,
            fields_count: 0,
        };

        assert!(matches!(symbol.kind, SymbolKind::Variable));
    }

    #[test]
    fn test_typescript_symbol_import() {
        let symbol = TypeScriptSymbol {
            name: "React".to_string(),
            kind: SymbolKind::Import,
            line: 1,
            is_exported: false,
            is_async: false,
            variants_count: 0,
            fields_count: 0,
        };

        assert!(matches!(symbol.kind, SymbolKind::Import));
        assert!(!symbol.is_exported); // Imports are typically not re-exported
    }

    #[test]
    fn test_typescript_symbol_export() {
        let symbol = TypeScriptSymbol {
            name: "default".to_string(),
            kind: SymbolKind::Export,
            line: 100,
            is_exported: true,
            is_async: false,
            variants_count: 0,
            fields_count: 0,
        };

        assert!(matches!(symbol.kind, SymbolKind::Export));
    }

    #[test]
    fn test_typescript_symbol_method() {
        let symbol = TypeScriptSymbol {
            name: "getData".to_string(),
            kind: SymbolKind::Method,
            line: 30,
            is_exported: false,
            is_async: true,
            variants_count: 0,
            fields_count: 0,
        };

        assert!(matches!(symbol.kind, SymbolKind::Method));
        assert!(symbol.is_async);
    }

    #[test]
    fn test_typescript_symbol_property() {
        let symbol = TypeScriptSymbol {
            name: "count".to_string(),
            kind: SymbolKind::Property,
            line: 45,
            is_exported: false,
            is_async: false,
            variants_count: 0,
            fields_count: 0,
        };

        assert!(matches!(symbol.kind, SymbolKind::Property));
    }

    // ========================================================================
    // SymbolKind tests
    // ========================================================================

    #[test]
    fn test_symbol_kind_debug() {
        let kind = SymbolKind::Function;
        let debug_str = format!("{kind:?}");
        assert_eq!(debug_str, "Function");
    }

    #[test]
    fn test_symbol_kind_clone() {
        let kind = SymbolKind::Class;
        let cloned = kind.clone();
        assert!(matches!(cloned, SymbolKind::Class));
    }

    #[test]
    fn test_all_symbol_kinds() {
        // Verify all variants are accessible
        let kinds = vec![
            SymbolKind::Function,
            SymbolKind::Class,
            SymbolKind::Interface,
            SymbolKind::TypeAlias,
            SymbolKind::Enum,
            SymbolKind::Variable,
            SymbolKind::Import,
            SymbolKind::Export,
            SymbolKind::Method,
            SymbolKind::Property,
        ];

        assert_eq!(kinds.len(), 10);
    }

    // ========================================================================
    // TypeScriptSymbol Clone and Debug tests
    // ========================================================================

    #[test]
    fn test_typescript_symbol_clone() {
        let symbol = TypeScriptSymbol {
            name: "test".to_string(),
            kind: SymbolKind::Function,
            line: 5,
            is_exported: true,
            is_async: false,
            variants_count: 0,
            fields_count: 0,
        };

        let cloned = symbol.clone();
        assert_eq!(cloned.name, symbol.name);
        assert_eq!(cloned.line, symbol.line);
        assert_eq!(cloned.is_exported, symbol.is_exported);
    }

    #[test]
    fn test_typescript_symbol_debug() {
        let symbol = TypeScriptSymbol {
            name: "debug_test".to_string(),
            kind: SymbolKind::Variable,
            line: 1,
            is_exported: false,
            is_async: false,
            variants_count: 0,
            fields_count: 0,
        };

        let debug_str = format!("{symbol:?}");
        assert!(debug_str.contains("TypeScriptSymbol"));
        assert!(debug_str.contains("debug_test"));
    }
}
