// Tests for language mapper
// Extracted to separate file for file health compliance (CB-040)
// Split into submodule include files for line-count compliance (PMAT-503)

use super::*;

mod tests {
    use super::*;
    use crate::services::context::AstItem;

    include!("language_mapper_tests_basic.rs");
}

/// Comprehensive coverage tests for language_mapper.rs
mod coverage_tests {
    use super::*;
    use crate::services::context::AstItem;
    use std::fs;
    use tempfile::TempDir;

    include!("language_mapper_tests_helpers_base.rs");
    include!("language_mapper_tests_java_kotlin.rs");
    include!("language_mapper_tests_scala_typescript.rs");
    include!("language_mapper_tests_js_csharp_ruby.rs");
    include!("language_mapper_tests_integration_edge.rs");
}

/// Property-based tests for language mapper
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    include!("language_mapper_tests_property.rs");
}
