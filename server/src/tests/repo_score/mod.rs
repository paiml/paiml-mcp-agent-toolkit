// TDD Tests for pmat repo-score
// Written BEFORE implementation (RED phase)
//
// Test Coverage Target: 85%+
// All tests should FAIL initially, then pass after implementation

pub mod models_tests;
pub mod readme_scorer_tests;
pub mod precommit_scorer_tests;
pub mod hygiene_scorer_tests;
pub mod makefile_scorer_tests;
pub mod ci_scorer_tests;
pub mod pmat_scorer_tests;
pub mod bonus_tests;
pub mod aggregator_tests;
pub mod integration_tests;

// Shared test utilities
pub mod test_utils;
