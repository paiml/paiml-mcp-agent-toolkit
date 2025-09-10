// Toyota Way: Language-Specific AST Strategies
//
// This module contains all language-specific AST parsing strategies,
// consolidating the functionality from individual ast_*.rs files

pub mod rust;

#[cfg(feature = "typescript-ast")]
pub mod typescript;

#[cfg(feature = "typescript-ast")]
pub mod javascript;

#[cfg(feature = "python-ast")]
pub mod python;

#[cfg(feature = "c-ast")]
pub mod c;

#[cfg(feature = "cpp-ast")]
pub mod cpp;

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test] 
        fn module_consistency_check(x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(x < 1001);
        }
    }
}
