//! Tests for AST strategies

use super::*;

#[test]
fn test_language_strategy_enum() {
    let strategies = vec![
        LanguageStrategy::Rust,
        LanguageStrategy::TypeScript,
        LanguageStrategy::Python,
        LanguageStrategy::C,
        LanguageStrategy::Cpp,
    ];
    
    for strategy in strategies {
        match strategy {
            LanguageStrategy::Rust => assert_eq!(format!("{:?}", strategy), "Rust"),
            LanguageStrategy::TypeScript => assert_eq!(format!("{:?}", strategy), "TypeScript"),
            LanguageStrategy::Python => assert_eq!(format!("{:?}", strategy), "Python"),
            LanguageStrategy::C => assert_eq!(format!("{:?}", strategy), "C"),
            LanguageStrategy::Cpp => assert_eq!(format!("{:?}", strategy), "Cpp"),
        }
    }
}

#[test]
fn test_get_strategy_for_extension() {
    // Rust extensions
    assert!(matches!(get_strategy_for_extension("rs"), Some(LanguageStrategy::Rust)));
    
    // TypeScript extensions
    assert!(matches!(get_strategy_for_extension("ts"), Some(LanguageStrategy::TypeScript)));
    assert!(matches!(get_strategy_for_extension("tsx"), Some(LanguageStrategy::TypeScript)));
    assert!(matches!(get_strategy_for_extension("js"), Some(LanguageStrategy::TypeScript)));
    assert!(matches!(get_strategy_for_extension("jsx"), Some(LanguageStrategy::TypeScript)));
    
    // Python extensions
    assert!(matches!(get_strategy_for_extension("py"), Some(LanguageStrategy::Python)));
    assert!(matches!(get_strategy_for_extension("pyi"), Some(LanguageStrategy::Python)));
    
    // C extensions
    assert!(matches!(get_strategy_for_extension("c"), Some(LanguageStrategy::C)));
    assert!(matches!(get_strategy_for_extension("h"), Some(LanguageStrategy::C)));
    
    // C++ extensions
    assert!(matches!(get_strategy_for_extension("cpp"), Some(LanguageStrategy::Cpp)));
    assert!(matches!(get_strategy_for_extension("cc"), Some(LanguageStrategy::Cpp)));
    assert!(matches!(get_strategy_for_extension("cxx"), Some(LanguageStrategy::Cpp)));
    assert!(matches!(get_strategy_for_extension("hpp"), Some(LanguageStrategy::Cpp)));
    
    // Unknown extension
    assert!(get_strategy_for_extension("xyz").is_none());
}

#[test]
fn test_get_strategy_for_path() {
    use std::path::Path;
    
    assert!(matches!(get_strategy_for_path(Path::new("main.rs")), Some(LanguageStrategy::Rust)));
    assert!(matches!(get_strategy_for_path(Path::new("app.ts")), Some(LanguageStrategy::TypeScript)));
    assert!(matches!(get_strategy_for_path(Path::new("script.py")), Some(LanguageStrategy::Python)));
    assert!(matches!(get_strategy_for_path(Path::new("program.c")), Some(LanguageStrategy::C)));
    assert!(matches!(get_strategy_for_path(Path::new("main.cpp")), Some(LanguageStrategy::Cpp)));
    assert!(get_strategy_for_path(Path::new("README.md")).is_none());
}

#[test]
fn test_ast_strategy_trait() {
    // Just verify the trait exists and can be referenced
    fn accepts_strategy<T: AstStrategy>(_strategy: &T) -> bool {
        true
    }
    
    // This is just a compile-time check
    assert!(true);
}