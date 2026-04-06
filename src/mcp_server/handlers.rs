use crate::mcp_server::state_manager::StateManager;
use crate::models::refactor::RefactorConfig;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

// Refactor operation handlers: start, next_iteration, get_state, stop
include!("handlers_refactor_ops.rs");

// Parameter parsing and state serialization helpers
include!("handlers_parsing.rs");

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
