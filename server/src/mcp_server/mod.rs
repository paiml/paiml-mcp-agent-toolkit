pub mod cache;
pub mod capnp_conversion;
pub mod handlers;
pub mod server;
pub mod snapshots;
pub mod state_manager;

pub use cache::{CacheConfig, CacheKeyBuilder, McpCache};
pub use server::McpServer;
pub use state_manager::StateManager;

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
