// Tests for unified protocol service
// Extracted for file health compliance (CB-040)
// Split into submodules via include!() for PMAT file size compliance

use super::*;

mod tests {
    use super::*;

    include!("service_tests_core.rs");
    include!("service_tests_requests.rs");
}

/// EXTREME TDD Coverage Tests for Unified Service
/// Sprint 46 Phase 6: Comprehensive coverage for uncovered lines
/// NOTE: Temporarily disabled due to private function access issues
#[cfg(all(test, pmat_broken_tests))]
mod coverage_tests {
    use super::*;
    use axum::body::Body;
    use axum::http::StatusCode;

    include!("service_tests_handlers.rs");
    include!("service_tests_metrics.rs");
    include!("service_tests_deep_context.rs");
    include!("service_tests_data_structures.rs");
    include!("service_tests_service_impl.rs");
}
