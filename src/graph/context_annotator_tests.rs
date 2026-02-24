#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use std::path::PathBuf;

    // --- Test fixtures and unit tests ---
    include!("context_annotator_tests_unit.rs");

    // --- Graph, property-based, and edge case tests ---
    include!("context_annotator_tests_graph.rs");
}
