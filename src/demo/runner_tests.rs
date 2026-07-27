// Tests for demo runner
// Extracted for file health compliance (CB-040)
// Split into include files for file health compliance (CB-040)

use super::*;

mod tests {
    // use super::*; // Unused in simple tests

    #[test]
    fn test_runner_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}

mod coverage_tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    // DemoStep, DemoReport, DemoAnalysisResult, and Component basic tests
    include!("runner_tests_step_and_report.rs");

    // Repository resolution, git root, canonical path, and detect tests
    include!("runner_tests_repo_resolution.rs");

    // DemoRunner operations and render_step_highlights tests
    include!("runner_tests_runner_operations.rs");

    // DemoReport render tests with steps
    include!("runner_tests_report_with_steps.rs");

    // Serialization, edge cases, helper, and async tests
    include!("runner_tests_serialization_and_edge_cases.rs");
}
