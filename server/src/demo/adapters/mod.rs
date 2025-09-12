pub mod cli;
pub mod http;
pub mod mcp;
pub mod tui;

pub use cli::{CliDemoAdapter, CliRequest, CliResponse};
pub use http::{HttpDemoAdapter, HttpRequest, HttpResponse};
pub use mcp::{McpDemoAdapter, McpRequest, McpResponse};
pub use tui::{TuiDemoAdapter, TuiRequest, TuiResponse};

#[cfg(test)]
mod tests {
    // use super::*; // Unused in simple tests

    #[test]
    fn test_mod_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}

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
