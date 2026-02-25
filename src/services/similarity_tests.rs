// Tests for similarity service
// Extracted for file health compliance (CB-040)
// Split into submodules via include!() for line-count compliance (PMAT-503)

use super::*;

mod tests {
    use super::*;

    // SimilarityConfig, CloneType, SimilarityDetector core method tests
    include!("similarity_tests_config_types.rs");

    // Winnowing, TokenAnalyzer, EntropyCalculator, and type clone tests
    include!("similarity_tests_detector_internals.rs");

    // Internal helpers, serialization, and integration tests
    include!("similarity_tests_helpers_integration.rs");
}

mod property_tests {
    // Property-based and SIMD equivalence tests
    include!("similarity_tests_property_simd.rs");
}
