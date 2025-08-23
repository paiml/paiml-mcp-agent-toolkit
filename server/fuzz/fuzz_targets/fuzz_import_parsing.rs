//! Fuzz testing for Import statement parsing
//! 
//! This fuzz test ensures that import statement parsing across different
//! languages handles malformed and edge-case inputs gracefully.

#![no_main]

use libfuzzer_sys::fuzz_target;
use pmat::services::context::AstItem;

fuzz_target!(|data: &[u8]| {
    // Try to interpret the data as UTF-8
    if let Ok(content) = std::str::from_utf8(data) {
        // Test various import patterns
        test_python_import_patterns(content);
        test_javascript_import_patterns(content);
        test_edge_cases(content);
    }
    
    // Test with binary data to ensure no panics
    test_binary_import_data(data);
});

fn test_python_import_patterns(module: &str) {
    // Test simple import
    let _ = AstItem::Import {
        module: module.to_string(),
        items: vec![],
        alias: None,
        line: 1,
    }.display_name();
    
    // Test import with alias
    let _ = AstItem::Import {
        module: module.to_string(),
        items: vec![],
        alias: Some(module.to_string()),
        line: 1,
    }.display_name();
    
    // Test from...import with items
    let items: Vec<String> = module.split_whitespace()
        .take(10)
        .map(|s| s.to_string())
        .collect();
    let _ = AstItem::Import {
        module: module.to_string(),
        items,
        alias: None,
        line: 1,
    }.display_name();
    
    // Test wildcard import
    let _ = AstItem::Import {
        module: module.to_string(),
        items: vec!["*".to_string()],
        alias: None,
        line: 1,
    }.display_name();
}

fn test_javascript_import_patterns(module: &str) {
    // Test ES6 default import
    let _ = AstItem::Import {
        module: module.to_string(),
        items: vec![],
        alias: None,
        line: 1,
    }.display_name();
    
    // Test named imports
    let items: Vec<String> = module.chars()
        .filter(|c| c.is_alphabetic())
        .take(5)
        .map(|c| c.to_string())
        .collect();
    let _ = AstItem::Import {
        module: module.to_string(),
        items,
        alias: None,
        line: 1,
    }.display_name();
    
    // Test relative imports
    let relative_module = format!("./{}", module);
    let _ = AstItem::Import {
        module: relative_module,
        items: vec![],
        alias: None,
        line: 1,
    }.display_name();
    
    // Test scoped packages
    let scoped_module = format!("@fuzzer/{}", module);
    let _ = AstItem::Import {
        module: scoped_module,
        items: vec![],
        alias: None,
        line: 1,
    }.display_name();
}

fn test_edge_cases(content: &str) {
    // Empty module
    let _ = AstItem::Import {
        module: String::new(),
        items: vec![],
        alias: None,
        line: 0,
    }.display_name();
    
    // Very long module name
    let long_module = content.chars().take(10000).collect::<String>();
    let _ = AstItem::Import {
        module: long_module,
        items: vec![],
        alias: None,
        line: usize::MAX,
    }.display_name();
    
    // Many items
    let many_items: Vec<String> = (0..1000)
        .map(|i| format!("item_{}", i))
        .collect();
    let _ = AstItem::Import {
        module: content.to_string(),
        items: many_items,
        alias: None,
        line: 1,
    }.display_name();
    
    // Special characters in module name
    let _ = AstItem::Import {
        module: content.to_string(),
        items: vec![],
        alias: Some(content.to_string()),
        line: 1,
    }.display_name();
}

fn test_binary_import_data(data: &[u8]) {
    // Convert binary data to string (may be invalid UTF-8)
    let module = data.iter()
        .map(|b| *b as char)
        .take(100)
        .collect::<String>();
    
    // Should not panic even with invalid characters
    let _ = AstItem::Import {
        module,
        items: vec![],
        alias: None,
        line: 1,
    }.display_name();
}