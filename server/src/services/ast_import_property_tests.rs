//! Property-based tests for AST Import variant
//! 
//! Tests the Import variant handling across different language contexts

use proptest::prelude::*;
use pmat::services::context::AstItem;

proptest! {
    #[test]
    fn test_import_display_name_consistency(
        module in "[a-zA-Z][a-zA-Z0-9_]*(\\.?[a-zA-Z][a-zA-Z0-9_]*){0,5}",
        items in prop::collection::vec("[a-zA-Z][a-zA-Z0-9_]*", 0..10),
        alias in prop::option::of("[a-zA-Z][a-zA-Z0-9_]*"),
        line in 1usize..10000,
    ) {
        let import = AstItem::Import {
            module: module.clone(),
            items,
            alias,
            line,
        };
        
        // Display name should always be the module name
        prop_assert_eq!(import.display_name(), module);
    }

    #[test]
    fn test_import_python_patterns(
        module in prop::oneof![
            Just("os"),
            Just("sys"),
            Just("numpy"),
            Just("pandas"),
            Just("matplotlib.pyplot"),
            Just("typing"),
            Just("collections.abc"),
            "[a-z]+(\\.?[a-z]+){0,3}",
        ],
        line in 1usize..1000,
    ) {
        // Test common Python import patterns
        let import = AstItem::Import {
            module: module.clone(),
            items: vec![],
            alias: None,
            line,
        };
        
        prop_assert_eq!(import.display_name(), module);
        
        // Test with alias
        let import_with_alias = AstItem::Import {
            module: module.clone(),
            items: vec![],
            alias: Some("alias".to_string()),
            line,
        };
        
        prop_assert_eq!(import_with_alias.display_name(), module);
    }

    #[test]
    fn test_import_javascript_patterns(
        module in prop::oneof![
            Just("react"),
            Just("vue"),
            Just("express"),
            Just("lodash"),
            Just("@mui/material"),
            Just("@testing-library/react"),
            Just("./utils"),
            Just("../components/Button"),
            "(@[a-z]+/)?[a-z]+(-[a-z]+)*",
        ],
        items in prop::collection::vec("[A-Z][a-zA-Z0-9]*", 0..5),
        line in 1usize..1000,
    ) {
        // Test common JavaScript/TypeScript import patterns
        let import = AstItem::Import {
            module: module.clone(),
            items,
            alias: None,
            line,
        };
        
        prop_assert_eq!(import.display_name(), module);
    }

    #[test]
    fn test_import_from_patterns(
        module in "[a-z]+(\\.?[a-z]+){0,3}",
        items in prop::collection::vec("[A-Z][a-zA-Z0-9]*", 1..10),
        line in 1usize..1000,
    ) {
        // Test "from X import Y, Z" patterns
        let import = AstItem::Import {
            module: module.clone(),
            items: items.clone(),
            alias: None,
            line,
        };
        
        prop_assert_eq!(import.display_name(), module);
        
        // Items should be preserved
        if let AstItem::Import { items: stored_items, .. } = import {
            prop_assert_eq!(stored_items.len(), items.len());
        }
    }

    #[test]
    fn test_import_relative_patterns(
        dots in 1usize..4,
        path in prop::option::of("[a-z]+(/[a-z]+){0,3}"),
        line in 1usize..1000,
    ) {
        // Test relative import patterns like ".", "..", "../foo"
        let module = format!("{}{}", 
            ".".repeat(dots),
            path.as_ref().map(|p| format!("/{}", p)).unwrap_or_default()
        );
        
        let import = AstItem::Import {
            module: module.clone(),
            items: vec![],
            alias: None,
            line,
        };
        
        prop_assert_eq!(import.display_name(), module);
    }

    #[test]
    fn test_import_wildcard_patterns(
        module in "[a-z]+(\\.?[a-z]+){0,3}",
        line in 1usize..1000,
    ) {
        // Test wildcard imports like "from X import *"
        let import = AstItem::Import {
            module: module.clone(),
            items: vec!["*".to_string()],
            alias: None,
            line,
        };
        
        prop_assert_eq!(import.display_name(), module);
    }

    #[test]
    fn test_import_alias_preservation(
        module in "[a-z]+(\\.?[a-z]+){0,3}",
        alias in "[a-z][a-z0-9_]*",
        line in 1usize..1000,
    ) {
        // Test that aliases are preserved but don't affect display_name
        let import = AstItem::Import {
            module: module.clone(),
            items: vec![],
            alias: Some(alias.clone()),
            line,
        };
        
        prop_assert_eq!(import.display_name(), module);
        
        // Verify alias is stored
        if let AstItem::Import { alias: stored_alias, .. } = import {
            prop_assert_eq!(stored_alias, Some(alias));
        }
    }

    #[test]
    fn test_import_line_boundaries(
        module in "[a-z]+",
        line in prop::oneof![
            Just(0usize),
            Just(1usize),
            Just(usize::MAX),
            1usize..100000,
        ],
    ) {
        // Test edge cases for line numbers
        let import = AstItem::Import {
            module: module.clone(),
            items: vec![],
            alias: None,
            line,
        };
        
        prop_assert_eq!(import.display_name(), module);
    }

    #[test]
    fn test_import_empty_items(
        module in "[a-z]+(\\.?[a-z]+){0,3}",
        line in 1usize..1000,
    ) {
        // Test that empty items list works correctly
        let import = AstItem::Import {
            module: module.clone(),
            items: vec![],
            alias: None,
            line,
        };
        
        prop_assert_eq!(import.display_name(), module);
        
        if let AstItem::Import { items, .. } = import {
            prop_assert!(items.is_empty());
        }
    }

    #[test]
    fn test_import_special_characters(
        prefix in prop::option::of("@[a-z]+"),
        name in "[a-z]+(-[a-z]+)*",
        suffix in prop::option::of("/[a-z]+(-[a-z]+)*"),
        line in 1usize..1000,
    ) {
        // Test npm scoped packages and special characters
        let module = format!("{}{}{}",
            prefix.as_ref().map(|p| format!("{}/", p)).unwrap_or_default(),
            name,
            suffix.unwrap_or_default()
        );
        
        let import = AstItem::Import {
            module: module.clone(),
            items: vec![],
            alias: None,
            line,
        };
        
        prop_assert_eq!(import.display_name(), module);
    }
}