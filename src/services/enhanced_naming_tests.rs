//! Comprehensive TDD Tests for Enhanced Naming in JavaScript/TypeScript/WASM
//!
//! Following extreme TDD approach - these tests define the enhanced naming behaviors
//! we want to implement for better deep context generation.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod enhanced_javascript_naming_tests {
    use crate::services::context::AstItem;
    use crate::services::enhanced_typescript_visitor::EnhancedTypeScriptVisitor;
    use std::path::Path;

    #[cfg(feature = "typescript-ast")]
    mod javascript_real_world_tests {
        use super::*;
        use std::sync::Arc;
        use swc_common::{FileName, SourceMap};
        use swc_ecma_ast::Module;
        use swc_ecma_parser::{lexer::Lexer, EsSyntax, Parser, StringInput, Syntax};

        // React components, async patterns, higher-order functions
        include!("enhanced_naming_tests_js_components.rs");

        // Module exports/imports, ES6+ features
        include!("enhanced_naming_tests_js_patterns.rs");
    }

    // JSDoc extraction tests
    include!("enhanced_naming_tests_js_jsdoc.rs");
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod enhanced_typescript_naming_tests {
    use crate::services::context::AstItem;
    use crate::services::enhanced_typescript_visitor::EnhancedTypeScriptVisitor;
    use std::path::Path;

    #[cfg(feature = "typescript-ast")]
    mod typescript_real_world_tests {
        use super::*;
        use std::sync::Arc;
        use swc_common::{FileName, SourceMap};
        use swc_ecma_ast::Module;
        use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax};

        // Type information, React TS components, decorators/metadata
        include!("enhanced_naming_tests_typescript.rs");
    }
}

#[cfg(test)]
#[cfg(feature = "wasm-ast")]
mod enhanced_wasm_naming_tests {
    use crate::services::context::AstItem;
    use crate::services::languages::wasm::WasmModuleAnalyzer;
    use std::path::Path;

    // WASM export names, import tracking, complex modules, binary extraction, validation
    include!("enhanced_naming_tests_wasm.rs");
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod enhanced_naming_integration_tests {
    // Multi-language integration, deep context markdown tests
    include!("enhanced_naming_tests_integration.rs");
}

/// Test utility functions for enhanced naming tests
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod test_utilities {
    use crate::services::context::AstItem;

    // Mock creation helpers and validation utilities
    include!("enhanced_naming_tests_utilities.rs");
}
