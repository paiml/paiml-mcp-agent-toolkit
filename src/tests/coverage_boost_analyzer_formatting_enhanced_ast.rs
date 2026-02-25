fn test_enhanced_ast_section_empty() {
    let analyzer = make_analyzer();
    let mut output = String::new();
    analyzer
        .format_enhanced_ast_section(&mut output, &[])
        .unwrap();
    assert!(output.contains("## Enhanced AST Analysis"));
}

#[test]
fn test_enhanced_ast_section_with_functions() {
    let analyzer = make_analyzer();
    let efc = EnhancedFileContext {
        base: FileContext {
            path: "src/lib.rs".to_string(),
            language: "Rust".to_string(),
            items: vec![
                AstItem::Function {
                    name: "my_func".to_string(),
                    visibility: "pub".to_string(),
                    is_async: true,
                    line: 10,
                },
                AstItem::Function {
                    name: "private_func".to_string(),
                    visibility: "priv".to_string(),
                    is_async: false,
                    line: 20,
                },
            ],
            complexity_metrics: None,
        },
        complexity_metrics: None,
        churn_metrics: None,
        defects: DefectAnnotations {
            dead_code: None,
            technical_debt: Vec::new(),
            complexity_violations: Vec::new(),
            tdg_score: None,
        },
        symbol_id: "sym_lib".to_string(),
    };
    let mut output = String::new();
    analyzer
        .format_enhanced_ast_section(&mut output, &[efc])
        .unwrap();
    assert!(output.contains("### src/lib.rs"));
    assert!(output.contains("**Language:** Rust"));
    assert!(output.contains("Total Symbols:** 2"));
    assert!(output.contains("`my_func (async)`"));
    assert!(output.contains("`private_func`"));
}

#[test]
fn test_enhanced_ast_section_with_structs_and_enums() {
    let analyzer = make_analyzer();
    let efc = EnhancedFileContext {
        base: FileContext {
            path: "src/types.rs".to_string(),
            language: "Rust".to_string(),
            items: vec![
                AstItem::Struct {
                    name: "MyStruct".to_string(),
                    visibility: "pub".to_string(),
                    fields_count: 3,
                    derives: vec!["Debug".to_string(), "Clone".to_string()],
                    line: 5,
                },
                AstItem::Enum {
                    name: "MyEnum".to_string(),
                    visibility: "pub".to_string(),
                    variants_count: 4,
                    line: 15,
                },
            ],
            complexity_metrics: None,
        },
        complexity_metrics: None,
        churn_metrics: None,
        defects: DefectAnnotations {
            dead_code: None,
            technical_debt: Vec::new(),
            complexity_violations: Vec::new(),
            tdg_score: None,
        },
        symbol_id: "sym_types".to_string(),
    };
    let mut output = String::new();
    analyzer
        .format_enhanced_ast_section(&mut output, &[efc])
        .unwrap();
    assert!(output.contains("MyStruct"));
    assert!(output.contains("3 fields"));
    assert!(output.contains("Debug, Clone"));
    assert!(output.contains("MyEnum"));
    assert!(output.contains("4 variants"));
}

#[test]
fn test_enhanced_ast_section_with_traits_and_impls() {
    let analyzer = make_analyzer();
    let efc = EnhancedFileContext {
        base: FileContext {
            path: "src/trait.rs".to_string(),
            language: "Rust".to_string(),
            items: vec![
                AstItem::Trait {
                    name: "Processor".to_string(),
                    visibility: "pub".to_string(),
                    line: 1,
                },
                AstItem::Impl {
                    type_name: "MyStruct".to_string(),
                    trait_name: Some("Processor".to_string()),
                    line: 10,
                },
                AstItem::Impl {
                    type_name: "MyStruct".to_string(),
                    trait_name: None,
                    line: 20,
                },
            ],
            complexity_metrics: None,
        },
        complexity_metrics: None,
        churn_metrics: None,
        defects: DefectAnnotations {
            dead_code: None,
            technical_debt: Vec::new(),
            complexity_violations: Vec::new(),
            tdg_score: None,
        },
        symbol_id: "sym_trait".to_string(),
    };
    let mut output = String::new();
    analyzer
        .format_enhanced_ast_section(&mut output, &[efc])
        .unwrap();
    assert!(output.contains("Processor"));
    assert!(output.contains("Processor for MyStruct"));
    assert!(output.contains("impl MyStruct"));
}

#[test]
fn test_enhanced_ast_section_with_modules_and_imports() {
    let analyzer = make_analyzer();
    let efc = EnhancedFileContext {
        base: FileContext {
            path: "src/mod.rs".to_string(),
            language: "Rust".to_string(),
            items: vec![
                AstItem::Module {
                    name: "submod".to_string(),
                    visibility: "pub".to_string(),
                    line: 1,
                },
                AstItem::Use {
                    path: "std::collections::HashMap".to_string(),
                    line: 3,
                },
                AstItem::Import {
                    module: "os".to_string(),
                    items: vec!["path".to_string(), "env".to_string()],
                    alias: None,
                    line: 5,
                },
                AstItem::Import {
                    module: "numpy".to_string(),
                    items: Vec::new(),
                    alias: Some("np".to_string()),
                    line: 6,
                },
            ],
            complexity_metrics: None,
        },
        complexity_metrics: None,
        churn_metrics: None,
        defects: DefectAnnotations {
            dead_code: None,
            technical_debt: Vec::new(),
            complexity_violations: Vec::new(),
            tdg_score: None,
        },
        symbol_id: "sym_mod".to_string(),
    };
    let mut output = String::new();
    analyzer
        .format_enhanced_ast_section(&mut output, &[efc])
        .unwrap();
    assert!(output.contains("submod"));
    assert!(output.contains("std::collections::HashMap"));
    // Import with items: "os (path, env)"
    assert!(output.contains("os (path, env)"));
    // Import with alias: "numpy as np"
    assert!(output.contains("numpy as np"));
}

#[test]
fn test_enhanced_ast_section_single_field_struct() {
    let analyzer = make_analyzer();
    let efc = EnhancedFileContext {
        base: FileContext {
            path: "src/single.rs".to_string(),
            language: "Rust".to_string(),
            items: vec![AstItem::Struct {
                name: "Wrapper".to_string(),
                visibility: "pub".to_string(),
                fields_count: 1,
                derives: Vec::new(),
                line: 1,
            }],
            complexity_metrics: None,
        },
        complexity_metrics: None,
        churn_metrics: None,
        defects: DefectAnnotations {
            dead_code: None,
            technical_debt: Vec::new(),
            complexity_violations: Vec::new(),
            tdg_score: None,
        },
        symbol_id: "sym_single".to_string(),
    };
    let mut output = String::new();
    analyzer
        .format_enhanced_ast_section(&mut output, &[efc])
        .unwrap();
    // Single field should not have an 's' suffix
    assert!(output.contains("1 field "));
    assert!(!output.contains("1 fields"));
}

#[test]
fn test_enhanced_ast_section_single_variant_enum() {
    let analyzer = make_analyzer();
    let efc = EnhancedFileContext {
        base: FileContext {
            path: "src/single_enum.rs".to_string(),
            language: "Rust".to_string(),
            items: vec![AstItem::Enum {
                name: "Single".to_string(),
                visibility: "pub".to_string(),
                variants_count: 1,
                line: 1,
            }],
            complexity_metrics: None,
        },
        complexity_metrics: None,
        churn_metrics: None,
        defects: DefectAnnotations {
            dead_code: None,
            technical_debt: Vec::new(),
            complexity_violations: Vec::new(),
            tdg_score: None,
        },
        symbol_id: "sym_single_enum".to_string(),
    };
    let mut output = String::new();
    analyzer
        .format_enhanced_ast_section(&mut output, &[efc])
        .unwrap();
    assert!(output.contains("1 variant "));
    assert!(!output.contains("1 variants"));
}

// ===========================================================================
// Edge case: more than 10 functions triggers truncation message
// ===========================================================================

#[test]
fn test_enhanced_ast_section_truncated_functions() {
    let analyzer = make_analyzer();
    let items: Vec<AstItem> = (0..15)
        .map(|i| AstItem::Function {
            name: format!("func_{i}"),
            visibility: "pub".to_string(),
            is_async: false,
            line: i * 10,
        })
        .collect();
    let efc = EnhancedFileContext {
        base: FileContext {
            path: "src/many.rs".to_string(),
            language: "Rust".to_string(),
            items,
            complexity_metrics: None,
        },
        complexity_metrics: None,
        churn_metrics: None,
        defects: DefectAnnotations {
            dead_code: None,
            technical_debt: Vec::new(),
            complexity_violations: Vec::new(),
            tdg_score: None,
        },
        symbol_id: "sym_many".to_string(),
    };
    let mut output = String::new();
    analyzer
        .format_enhanced_ast_section(&mut output, &[efc])
        .unwrap();
    assert!(output.contains("... and 5 more functions"));
}

// ===========================================================================
// Edge case: more than 8 imports triggers compact mode
// ===========================================================================

#[test]
fn test_enhanced_ast_section_many_imports_compact() {
    let analyzer = make_analyzer();
    let items: Vec<AstItem> = (0..12)
        .map(|i| AstItem::Use {
            path: format!("crate::module_{i}"),
            line: i,
        })
        .collect();
    let efc = EnhancedFileContext {
        base: FileContext {
            path: "src/imports.rs".to_string(),
            language: "Rust".to_string(),
            items,
            complexity_metrics: None,
        },
        complexity_metrics: None,
        churn_metrics: None,
        defects: DefectAnnotations {
            dead_code: None,
            technical_debt: Vec::new(),
            complexity_violations: Vec::new(),
            tdg_score: None,
        },
        symbol_id: "sym_imports".to_string(),
    };
    let mut output = String::new();
    analyzer
        .format_enhanced_ast_section(&mut output, &[efc])
        .unwrap();
    // With > 8 imports, should show compact form
    assert!(output.contains("**Imports:** 12 import statements"));
}

// ===========================================================================
// Edge case: more than 5 structs triggers truncation
// ===========================================================================

#[test]
fn test_enhanced_ast_section_truncated_structs() {
    let analyzer = make_analyzer();
    let items: Vec<AstItem> = (0..8)
        .map(|i| AstItem::Struct {
            name: format!("Struct_{i}"),
            visibility: "pub".to_string(),
            fields_count: i + 1,
            derives: Vec::new(),
            line: i * 5,
        })
        .collect();
    let efc = EnhancedFileContext {
        base: FileContext {
            path: "src/structs.rs".to_string(),
            language: "Rust".to_string(),
            items,
            complexity_metrics: None,
        },
        complexity_metrics: None,
        churn_metrics: None,
        defects: DefectAnnotations {
            dead_code: None,
            technical_debt: Vec::new(),
            complexity_violations: Vec::new(),
            tdg_score: None,
        },
        symbol_id: "sym_structs".to_string(),
    };
    let mut output = String::new();
    analyzer
        .format_enhanced_ast_section(&mut output, &[efc])
        .unwrap();
    assert!(output.contains("... and 3 more structs"));
}

// ===========================================================================
// Legacy markdown: file metrics (complexity, churn, TDG)
// ===========================================================================

