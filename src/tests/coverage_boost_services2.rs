#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for services - batch 2
//! Tests for language registry and detection

use crate::services::language_registry::{Language, LanguageRegistry};
use std::path::Path;

#[test]
fn test_language_registry_new() {
    let registry = LanguageRegistry::new();
    assert!(!registry.supported_languages().is_empty());
}

#[test]
fn test_detect_rust() {
    let registry = LanguageRegistry::new();
    let lang = registry.detect_language(Path::new("test.rs"));
    assert_eq!(lang, Language::Rust);
}

#[test]
fn test_detect_python() {
    let registry = LanguageRegistry::new();
    let lang = registry.detect_language(Path::new("test.py"));
    assert_eq!(lang, Language::Python);
}

#[test]
fn test_detect_javascript() {
    let registry = LanguageRegistry::new();
    let lang = registry.detect_language(Path::new("test.js"));
    assert_eq!(lang, Language::JavaScript);
}

#[test]
fn test_detect_typescript() {
    let registry = LanguageRegistry::new();
    let lang = registry.detect_language(Path::new("test.ts"));
    assert_eq!(lang, Language::TypeScript);
}

#[test]
fn test_detect_go() {
    let registry = LanguageRegistry::new();
    let lang = registry.detect_language(Path::new("test.go"));
    assert_eq!(lang, Language::Go);
}

#[test]
fn test_detect_java() {
    let registry = LanguageRegistry::new();
    let lang = registry.detect_language(Path::new("Test.java"));
    assert_eq!(lang, Language::Java);
}

#[test]
fn test_detect_cpp() {
    let registry = LanguageRegistry::new();
    let lang = registry.detect_language(Path::new("test.cpp"));
    assert_eq!(lang, Language::Cpp);
}

#[test]
fn test_detect_c() {
    let registry = LanguageRegistry::new();
    let lang = registry.detect_language(Path::new("test.c"));
    assert_eq!(lang, Language::C);
}

#[test]
fn test_detect_unknown() {
    let registry = LanguageRegistry::new();
    let lang = registry.detect_language(Path::new("test.xyz123"));
    assert_eq!(lang, Language::Unknown);
}

#[test]
fn test_language_debug() {
    // Test Debug trait
    let _ = format!("{:?}", Language::Rust);
    let _ = format!("{:?}", Language::Python);
    let _ = format!("{:?}", Language::JavaScript);
}

#[test]
fn test_language_from_extension() {
    assert_eq!(Language::from_extension("rs"), Language::Rust);
    assert_eq!(Language::from_extension("py"), Language::Python);
    assert_eq!(Language::from_extension("js"), Language::JavaScript);
}

#[test]
fn test_language_from_path() {
    assert_eq!(Language::from_path(Path::new("test.rs")), Language::Rust);
    assert_eq!(Language::from_path(Path::new("test.py")), Language::Python);
}

#[test]
fn test_language_extensions() {
    let exts = Language::Rust.extensions();
    assert!(exts.contains(&"rs"));
}

#[test]
fn test_language_name() {
    assert_eq!(Language::Rust.name(), "Rust");
    assert_eq!(Language::Python.name(), "Python");
}

#[test]
fn test_supported_languages_count() {
    let registry = LanguageRegistry::new();
    let langs = registry.supported_languages();
    // Should have 30+ languages
    assert!(langs.len() >= 30);
}

#[test]
fn test_language_count() {
    let registry = LanguageRegistry::new();
    assert!(registry.language_count() >= 30);
}

#[test]
fn test_ast_supported_languages() {
    let registry = LanguageRegistry::new();
    let ast_langs = registry.ast_supported_languages();
    assert!(!ast_langs.is_empty());
}

#[test]
fn test_complexity_supported_languages() {
    let registry = LanguageRegistry::new();
    let complex_langs = registry.complexity_supported_languages();
    assert!(!complex_langs.is_empty());
}

#[test]
fn test_has_ast_support() {
    assert!(Language::Rust.has_ast_support());
    assert!(Language::Python.has_ast_support());
}

#[test]
fn test_supports_complexity() {
    assert!(Language::Rust.supports_complexity());
    assert!(Language::Python.supports_complexity());
}

#[test]
fn test_language_equality() {
    assert_eq!(Language::Rust, Language::Rust);
    assert_ne!(Language::Rust, Language::Python);
}

#[test]
fn test_language_clone() {
    let lang = Language::TypeScript;
    let cloned = lang.clone();
    assert_eq!(lang, cloned);
}

#[test]
fn test_language_copy() {
    let lang = Language::Go;
    let copied = lang;
    assert_eq!(lang, copied);
}

#[test]
fn test_language_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(Language::Rust);
    set.insert(Language::Python);
    assert!(set.contains(&Language::Rust));
    assert!(set.contains(&Language::Python));
    assert!(!set.contains(&Language::Go));
}
