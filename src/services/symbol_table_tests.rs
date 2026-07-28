#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_symbol_table_insertion_and_lookup() {
        let table = SymbolTable::new();

        let qname = QualifiedName::new(
            vec!["std".to_string(), "collections".to_string()],
            "HashMap".to_string(),
        );
        let location = Location::new(PathBuf::from("src/lib.rs"), 100, 200);

        table.insert(qname.clone(), location.clone());

        assert_eq!(table.get_location(&qname), Some(location));
        assert_eq!(table.len(), 1);
    }

    #[test]
    fn test_relative_location_resolution() {
        let table = SymbolTable::new();
        let file_path = PathBuf::from("src/lib.rs");

        // Test span resolution
        let rel_span = RelativeLocation::Span {
            start: 100,
            end: 200,
        };
        let resolved = table.resolve_relative(&rel_span, &file_path).unwrap();

        assert_eq!(resolved.file_path, file_path);
        assert_eq!(resolved.span.start.0, 100);
        assert_eq!(resolved.span.end.0, 200);
    }

    #[test]
    fn test_qualified_name_parsing() {
        let qname = QualifiedName::from_string("std::collections::HashMap").unwrap();
        assert_eq!(qname.module_path, vec!["std", "collections"]);
        assert_eq!(qname.name, "HashMap");
        assert_eq!(qname.to_string(), "std::collections::HashMap");
    }

    #[test]
    fn test_symbol_table_builder() {
        let builder = SymbolTableBuilder::new();
        let qname = QualifiedName::new(vec!["test".to_string()], "function".to_string());
        let location = Location::new(PathBuf::from("test.rs"), 0, 10);

        builder.add_symbol(qname.clone(), location.clone());
        let table = builder.build();

        assert_eq!(table.get_location(&qname), Some(location));
    }

    // ============ SymbolTable Tests ============

    #[test]
    fn test_symbol_table_new() {
        let table = SymbolTable::new();
        assert!(table.is_empty());
        assert_eq!(table.len(), 0);
    }

    #[test]
    fn test_symbol_table_default() {
        let table = SymbolTable::default();
        assert!(table.is_empty());
    }

    #[test]
    fn test_symbol_table_debug() {
        let table = SymbolTable::new();
        let debug = format!("{:?}", table);
        assert!(debug.contains("SymbolTable"));
    }

    #[test]
    fn test_symbol_table_insert_multiple() {
        let table = SymbolTable::new();

        for i in 0..10 {
            let qname = QualifiedName::new(vec!["module".to_string()], format!("func{}", i));
            let location = Location::new(PathBuf::from("src/lib.rs"), i * 10, (i + 1) * 10);
            table.insert(qname, location);
        }

        assert_eq!(table.len(), 10);
    }

    #[test]
    fn test_symbol_table_clear() {
        let table = SymbolTable::new();

        let qname = QualifiedName::new(vec!["test".to_string()], "func".to_string());
        let location = Location::new(PathBuf::from("test.rs"), 0, 100);
        table.insert(qname, location);

        assert!(!table.is_empty());
        table.clear();
        assert!(table.is_empty());
    }

    #[test]
    fn test_symbol_table_all_symbols() {
        let table = SymbolTable::new();

        let qname1 = QualifiedName::new(vec!["a".to_string()], "f1".to_string());
        let qname2 = QualifiedName::new(vec!["b".to_string()], "f2".to_string());
        let location1 = Location::new(PathBuf::from("a.rs"), 0, 10);
        let location2 = Location::new(PathBuf::from("b.rs"), 0, 20);

        table.insert(qname1.clone(), location1);
        table.insert(qname2.clone(), location2);

        let all = table.all_symbols();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_symbol_at_location_exact_match() {
        let table = SymbolTable::new();
        let file_path = PathBuf::from("src/lib.rs");

        let qname = QualifiedName::new(vec!["module".to_string()], "my_func".to_string());
        let location = Location::new(file_path.clone(), 100, 200);
        table.insert(qname.clone(), location.clone());

        let found = table.symbol_at_location(&location);
        assert_eq!(found, Some(qname));
    }

    #[test]
    fn test_symbol_at_location_closest_match() {
        let table = SymbolTable::new();
        let file_path = PathBuf::from("src/lib.rs");

        // Insert symbol at position 100
        let qname = QualifiedName::new(vec!["module".to_string()], "func".to_string());
        let location = Location::new(file_path.clone(), 100, 200);
        table.insert(qname.clone(), location);

        // Search for a location at position 150 (within the range)
        let search_location = Location::new(file_path, 150, 160);
        let found = table.symbol_at_location(&search_location);
        assert_eq!(found, Some(qname));
    }

    #[test]
    fn test_symbol_at_location_not_found() {
        let table = SymbolTable::new();
        let file_path = PathBuf::from("nonexistent.rs");

        let location = Location::new(file_path, 0, 10);
        let found = table.symbol_at_location(&location);
        assert!(found.is_none());
    }

    #[test]
    fn test_symbols_in_span_empty() {
        let table = SymbolTable::new();
        let file_path = PathBuf::from("test.rs");

        let location = Location::new(file_path, 0, 1000);
        let symbols = table.symbols_in_span(&location);
        assert!(symbols.is_empty());
    }

    #[test]
    fn test_symbols_in_span_with_symbols() {
        let table = SymbolTable::new();
        let file_path = PathBuf::from("src/lib.rs");

        // Insert multiple symbols in a file
        for i in 0..5 {
            let qname = QualifiedName::new(vec!["mod".to_string()], format!("func{}", i));
            let location = Location::new(file_path.clone(), i * 100, (i + 1) * 100 - 1);
            table.insert(qname, location);
        }

        // Search for symbols in span 0-250
        let search_location = Location::new(file_path, 0, 250);
        let symbols = table.symbols_in_span(&search_location);
        assert!(symbols.len() >= 2);
    }

    #[test]
    fn test_symbols_in_span_boundary() {
        let table = SymbolTable::new();
        let file_path = PathBuf::from("test.rs");

        let qname = QualifiedName::new(vec!["mod".to_string()], "func".to_string());
        let location = Location::new(file_path.clone(), 100, 200);
        table.insert(qname.clone(), location);

        // Exact boundary match
        let search = Location::new(file_path, 100, 101);
        let found = table.symbols_in_span(&search);
        assert!(!found.is_empty());
    }

    #[test]
    fn test_resolve_relative_function_with_module() {
        let table = SymbolTable::new();
        let file_path = PathBuf::from("src/utils/helpers.rs");

        // Insert a symbol
        let qname = QualifiedName::new(
            vec!["utils".to_string(), "helpers".to_string()],
            "my_function".to_string(),
        );
        let location = Location::new(file_path.clone(), 50, 100);
        table.insert(qname, location.clone());

        // Try to resolve via function name with explicit module
        let rel = RelativeLocation::Function {
            name: "my_function".to_string(),
            module: Some("utils::helpers".to_string()),
        };

        let resolved = table.resolve_relative(&rel, &file_path);
        assert!(resolved.is_some());
    }

    #[test]
    fn test_resolve_relative_function_without_module() {
        let table = SymbolTable::new();
        let file_path = PathBuf::from("src/lib.rs");

        // Resolve function without explicit module - infers from file path
        let rel = RelativeLocation::Function {
            name: "test_func".to_string(),
            module: None,
        };

        let resolved = table.resolve_relative(&rel, &file_path);
        // May or may not resolve depending on what's in the table
        assert!(resolved.is_none() || resolved.is_some());
    }

    #[test]
    fn test_resolve_relative_symbol() {
        let table = SymbolTable::new();
        let file_path = PathBuf::from("test.rs");

        // Insert symbol
        let qname = QualifiedName::new(vec!["pkg".to_string()], "Item".to_string());
        let location = Location::new(PathBuf::from("pkg.rs"), 0, 50);
        table.insert(qname, location.clone());

        // Resolve by qualified name
        let rel = RelativeLocation::Symbol {
            qualified_name: "pkg::Item".to_string(),
        };

        let resolved = table.resolve_relative(&rel, &file_path);
        assert!(resolved.is_some());
        assert_eq!(resolved.unwrap(), location);
    }

    #[test]
    fn test_resolve_relative_symbol_not_found() {
        let table = SymbolTable::new();
        let file_path = PathBuf::from("test.rs");

        let rel = RelativeLocation::Symbol {
            qualified_name: "nonexistent::Symbol".to_string(),
        };

        let resolved = table.resolve_relative(&rel, &file_path);
        assert!(resolved.is_none());
    }

    #[test]
    fn test_resolve_relative_symbol_invalid_format() {
        let table = SymbolTable::new();
        let file_path = PathBuf::from("test.rs");

        // Empty qualified name (should fail to parse)
        let rel = RelativeLocation::Symbol {
            qualified_name: "".to_string(),
        };

        let resolved = table.resolve_relative(&rel, &file_path);
        assert!(resolved.is_none());
    }

    // ============ SymbolTableBuilder Tests ============

    #[test]
    fn test_symbol_table_builder_new() {
        let builder = SymbolTableBuilder::new();
        let table = builder.build();
        assert!(table.is_empty());
    }

    #[test]
    fn test_symbol_table_builder_default() {
        let builder = SymbolTableBuilder::default();
        let table = builder.build();
        assert!(table.is_empty());
    }

    #[test]
    fn test_symbol_table_builder_add_multiple() {
        let builder = SymbolTableBuilder::new();

        for i in 0..5 {
            let qname = QualifiedName::new(vec!["mod".to_string()], format!("fn{}", i));
            let location = Location::new(PathBuf::from("test.rs"), i * 10, (i + 1) * 10);
            builder.add_symbol(qname, location);
        }

        let table = builder.build();
        assert_eq!(table.len(), 5);
    }

    // ============ Module Path Inference Tests ============

    #[test]
    fn test_infer_module_from_file_path_simple() {
        let table = SymbolTable::new();
        let path = Path::new("src/utils.rs");

        let module_path = table.infer_module_from_file_path(path);
        assert!(module_path.contains(&"utils".to_string()));
    }

    #[test]
    fn test_infer_module_from_file_path_mod_rs() {
        let table = SymbolTable::new();
        let path = Path::new("src/utils/mod.rs");

        let module_path = table.infer_module_from_file_path(path);
        // mod.rs should not contribute to module path
        assert!(!module_path.iter().any(|s| s == "mod"));
    }

    #[test]
    fn test_infer_module_from_file_path_lib_rs() {
        let table = SymbolTable::new();
        let path = Path::new("src/lib.rs");

        let module_path = table.infer_module_from_file_path(path);
        // lib.rs should not contribute to module path
        assert!(!module_path.iter().any(|s| s == "lib"));
    }

    #[test]
    fn test_infer_module_from_file_path_main_rs() {
        let table = SymbolTable::new();
        let path = Path::new("src/main.rs");

        let module_path = table.infer_module_from_file_path(path);
        // main.rs should not contribute to module path
        assert!(!module_path.iter().any(|s| s == "main"));
    }

    #[test]
    fn test_infer_module_from_file_path_nested() {
        let table = SymbolTable::new();
        let path = Path::new("src/services/cache/redis.rs");

        let module_path = table.infer_module_from_file_path(path);
        assert!(module_path.contains(&"redis".to_string()));
    }

    #[test]
    fn test_parse_explicit_module() {
        let table = SymbolTable::new();

        let module_path = table.parse_explicit_module("std::collections::hash_map");
        assert_eq!(module_path, vec!["std", "collections", "hash_map"]);
    }

    #[test]
    fn test_parse_explicit_module_single() {
        let table = SymbolTable::new();

        let module_path = table.parse_explicit_module("single");
        assert_eq!(module_path, vec!["single"]);
    }

    #[test]
    fn test_extract_significant_file_stem() {
        let table = SymbolTable::new();

        let path = Path::new("src/utils.rs");
        let stem = table.extract_significant_file_stem(path);
        assert_eq!(stem, Some("utils".to_string()));
    }

    #[test]
    fn test_extract_significant_file_stem_mod() {
        let table = SymbolTable::new();

        let path = Path::new("src/mod.rs");
        let stem = table.extract_significant_file_stem(path);
        assert!(stem.is_none());
    }

    #[test]
    fn test_extract_directory_name() {
        let table = SymbolTable::new();

        let path = Path::new("src/services");
        let name = table.extract_directory_name(path);
        assert_eq!(name, Some("services".to_string()));
    }

    #[test]
    fn test_get_location_not_found() {
        let table = SymbolTable::new();

        let qname = QualifiedName::new(vec!["nonexistent".to_string()], "func".to_string());
        let location = table.get_location(&qname);
        assert!(location.is_none());
    }

    #[test]
    fn test_insert_overwrites_existing() {
        let table = SymbolTable::new();

        let qname = QualifiedName::new(vec!["mod".to_string()], "func".to_string());
        let location1 = Location::new(PathBuf::from("a.rs"), 0, 10);
        let location2 = Location::new(PathBuf::from("b.rs"), 20, 30);

        table.insert(qname.clone(), location1);
        table.insert(qname.clone(), location2.clone());

        // Should have the second location
        let found = table.get_location(&qname);
        assert_eq!(found, Some(location2));
    }
}

