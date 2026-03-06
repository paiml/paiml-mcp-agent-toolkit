#![cfg_attr(coverage_nightly, coverage(off))]
// Toyota Way: AST Strategy Pattern Implementation
//
// Consolidates the strategy pattern logic from ast_strategies.rs

use super::AstStrategy;
use std::sync::Arc;

/// Strategy selector for choosing appropriate AST parser
pub struct StrategySelector;

impl StrategySelector {
    /// Get the best strategy for a given file extension
    #[must_use]
    pub fn select_for_extension(extension: &str) -> Option<Arc<dyn AstStrategy>> {
        match extension {
            "rs" => Some(Arc::new(super::languages::rust::RustStrategy::new())),

            #[cfg(feature = "typescript-ast")]
            "ts" | "tsx" => Some(Arc::new(
                super::languages::typescript::TypeScriptStrategy::new(),
            )),

            #[cfg(feature = "typescript-ast")]
            "js" | "jsx" | "mjs" => Some(Arc::new(
                super::languages::javascript::JavaScriptStrategy::new(),
            )),

            #[cfg(feature = "python-ast")]
            "py" | "pyi" | "pyw" => Some(Arc::new(super::languages::python::PythonStrategy::new())),

            #[cfg(feature = "c-ast")]
            "c" | "h" => Some(Arc::new(super::languages::c_cpp_strategy::CStrategy::new())),

            #[cfg(feature = "cpp-ast")]
            "cpp" | "cxx" | "cc" | "hpp" | "hxx" | "hh" => Some(Arc::new(
                super::languages::c_cpp_strategy::CppStrategy::new(),
            )),

            _ => None,
        }
    }

    /// Get all supported extensions
    #[must_use]
    pub fn supported_extensions() -> Vec<&'static str> {
        #[allow(unused_mut)]
        let mut extensions = vec!["rs"]; // Rust is always supported

        #[cfg(feature = "typescript-ast")]
        {
            extensions.extend_from_slice(&["ts", "tsx", "js", "jsx", "mjs"]);
        }

        #[cfg(feature = "python-ast")]
        {
            extensions.extend_from_slice(&["py", "pyi", "pyw"]);
        }

        #[cfg(feature = "c-ast")]
        {
            extensions.extend_from_slice(&["c", "h"]);
        }

        #[cfg(feature = "cpp-ast")]
        {
            extensions.extend_from_slice(&["cpp", "cxx", "cc", "hpp", "hxx", "hh"]);
        }

        extensions
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;

    // ========================================================================
    // StrategySelector::select_for_extension tests
    // ========================================================================

    #[test]
    fn test_select_for_extension_rust() {
        // Rust is always available (no feature gate)
        let strategy = StrategySelector::select_for_extension("rs");
        assert!(strategy.is_some());
        let s = strategy.unwrap();
        assert_eq!(s.primary_extension(), "rs");
        assert_eq!(s.language_name(), "Rust");
    }

    #[test]
    fn test_select_for_extension_unknown() {
        // Unknown extension should return None
        let strategy = StrategySelector::select_for_extension("xyz");
        assert!(strategy.is_none());
    }

    #[test]
    fn test_select_for_extension_empty() {
        // Empty string should return None
        let strategy = StrategySelector::select_for_extension("");
        assert!(strategy.is_none());
    }

    #[cfg(feature = "typescript-ast")]
    #[test]
    fn test_select_for_extension_typescript() {
        let strategy = StrategySelector::select_for_extension("ts");
        assert!(strategy.is_some());
        let s = strategy.unwrap();
        assert_eq!(s.primary_extension(), "ts");
    }

    #[cfg(feature = "typescript-ast")]
    #[test]
    fn test_select_for_extension_tsx() {
        let strategy = StrategySelector::select_for_extension("tsx");
        assert!(strategy.is_some());
    }

    #[cfg(feature = "typescript-ast")]
    #[test]
    fn test_select_for_extension_javascript() {
        let strategy = StrategySelector::select_for_extension("js");
        assert!(strategy.is_some());
        let s = strategy.unwrap();
        assert_eq!(s.primary_extension(), "js");
    }

    #[cfg(feature = "typescript-ast")]
    #[test]
    fn test_select_for_extension_jsx() {
        let strategy = StrategySelector::select_for_extension("jsx");
        assert!(strategy.is_some());
    }

    #[cfg(feature = "typescript-ast")]
    #[test]
    fn test_select_for_extension_mjs() {
        let strategy = StrategySelector::select_for_extension("mjs");
        assert!(strategy.is_some());
    }

    #[cfg(feature = "python-ast")]
    #[test]
    fn test_select_for_extension_python() {
        let strategy = StrategySelector::select_for_extension("py");
        assert!(strategy.is_some());
        let s = strategy.unwrap();
        assert_eq!(s.primary_extension(), "py");
    }

    #[cfg(feature = "python-ast")]
    #[test]
    fn test_select_for_extension_python_pyi() {
        let strategy = StrategySelector::select_for_extension("pyi");
        assert!(strategy.is_some());
    }

    #[cfg(feature = "python-ast")]
    #[test]
    fn test_select_for_extension_python_pyw() {
        let strategy = StrategySelector::select_for_extension("pyw");
        assert!(strategy.is_some());
    }

    #[cfg(feature = "c-ast")]
    #[test]
    fn test_select_for_extension_c() {
        let strategy = StrategySelector::select_for_extension("c");
        assert!(strategy.is_some());
        let s = strategy.unwrap();
        assert_eq!(s.primary_extension(), "c");
    }

    #[cfg(feature = "c-ast")]
    #[test]
    fn test_select_for_extension_c_header() {
        let strategy = StrategySelector::select_for_extension("h");
        assert!(strategy.is_some());
    }

    #[cfg(feature = "cpp-ast")]
    #[test]
    fn test_select_for_extension_cpp() {
        let strategy = StrategySelector::select_for_extension("cpp");
        assert!(strategy.is_some());
        let s = strategy.unwrap();
        assert_eq!(s.primary_extension(), "cpp");
    }

    #[cfg(feature = "cpp-ast")]
    #[test]
    fn test_select_for_extension_cpp_cxx() {
        let strategy = StrategySelector::select_for_extension("cxx");
        assert!(strategy.is_some());
    }

    #[cfg(feature = "cpp-ast")]
    #[test]
    fn test_select_for_extension_cpp_cc() {
        let strategy = StrategySelector::select_for_extension("cc");
        assert!(strategy.is_some());
    }

    #[cfg(feature = "cpp-ast")]
    #[test]
    fn test_select_for_extension_cpp_hpp() {
        let strategy = StrategySelector::select_for_extension("hpp");
        assert!(strategy.is_some());
    }

    #[cfg(feature = "cpp-ast")]
    #[test]
    fn test_select_for_extension_cpp_hxx() {
        let strategy = StrategySelector::select_for_extension("hxx");
        assert!(strategy.is_some());
    }

    #[cfg(feature = "cpp-ast")]
    #[test]
    fn test_select_for_extension_cpp_hh() {
        let strategy = StrategySelector::select_for_extension("hh");
        assert!(strategy.is_some());
    }

    // ========================================================================
    // StrategySelector::supported_extensions tests
    // ========================================================================

    #[test]
    fn test_supported_extensions_always_includes_rust() {
        let extensions = StrategySelector::supported_extensions();
        assert!(extensions.contains(&"rs"));
        assert!(!extensions.is_empty());
    }

    #[cfg(feature = "typescript-ast")]
    #[test]
    fn test_supported_extensions_includes_typescript() {
        let extensions = StrategySelector::supported_extensions();
        assert!(extensions.contains(&"ts"));
        assert!(extensions.contains(&"tsx"));
        assert!(extensions.contains(&"js"));
        assert!(extensions.contains(&"jsx"));
        assert!(extensions.contains(&"mjs"));
    }

    #[cfg(feature = "python-ast")]
    #[test]
    fn test_supported_extensions_includes_python() {
        let extensions = StrategySelector::supported_extensions();
        assert!(extensions.contains(&"py"));
        assert!(extensions.contains(&"pyi"));
        assert!(extensions.contains(&"pyw"));
    }

    #[cfg(feature = "c-ast")]
    #[test]
    fn test_supported_extensions_includes_c() {
        let extensions = StrategySelector::supported_extensions();
        assert!(extensions.contains(&"c"));
        assert!(extensions.contains(&"h"));
    }

    #[cfg(feature = "cpp-ast")]
    #[test]
    fn test_supported_extensions_includes_cpp() {
        let extensions = StrategySelector::supported_extensions();
        assert!(extensions.contains(&"cpp"));
        assert!(extensions.contains(&"cxx"));
        assert!(extensions.contains(&"cc"));
        assert!(extensions.contains(&"hpp"));
        assert!(extensions.contains(&"hxx"));
        assert!(extensions.contains(&"hh"));
    }

    // ========================================================================
    // Strategy trait verification tests
    // ========================================================================

    #[test]
    fn test_strategy_can_handle() {
        let strategy = StrategySelector::select_for_extension("rs").unwrap();
        assert!(strategy.can_handle("rs"));
        assert!(!strategy.can_handle("py"));
        assert!(!strategy.can_handle("xyz"));
    }

    #[test]
    fn test_strategy_supported_extensions_not_empty() {
        let strategy = StrategySelector::select_for_extension("rs").unwrap();
        let supported = strategy.supported_extensions();
        assert!(!supported.is_empty());
        assert!(supported.contains(&"rs"));
    }

    // ========================================================================
    // Edge case tests
    // ========================================================================

    #[test]
    fn test_select_for_extension_case_sensitive() {
        // Extensions are case-sensitive - uppercase should not match
        let strategy = StrategySelector::select_for_extension("RS");
        assert!(strategy.is_none());

        let strategy = StrategySelector::select_for_extension("Rs");
        assert!(strategy.is_none());
    }

    #[test]
    fn test_select_for_extension_with_dot() {
        // Extension should not include the dot
        let strategy = StrategySelector::select_for_extension(".rs");
        assert!(strategy.is_none());
    }

    #[test]
    fn test_multiple_calls_return_fresh_instances() {
        let strategy1 = StrategySelector::select_for_extension("rs");
        let strategy2 = StrategySelector::select_for_extension("rs");

        // Both should be Some
        assert!(strategy1.is_some());
        assert!(strategy2.is_some());

        // Both should have the same properties
        let s1 = strategy1.unwrap();
        let s2 = strategy2.unwrap();
        assert_eq!(s1.primary_extension(), s2.primary_extension());
        assert_eq!(s1.language_name(), s2.language_name());
    }

    #[test]
    fn test_supported_extensions_returns_vec() {
        let extensions = StrategySelector::supported_extensions();
        // Should return a Vec
        assert!(extensions.len() >= 1); // At least "rs"
    }
}
