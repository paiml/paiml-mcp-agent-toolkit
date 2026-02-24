#![cfg_attr(coverage_nightly, coverage(off))]
use serde::{Deserialize, Serialize};
use serde_json::Value;

include!("mcp_types.rs");
include!("mcp_params.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    include!("mcp_tests.rs");
    include!("mcp_serialization_tests.rs");
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use serde_json::json;

    include!("mcp_property_tests.rs");
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    //! EXTREME TDD coverage tests for models/mcp.rs
    //! These tests ensure comprehensive coverage of all MCP model types.

    use super::*;
    use serde_json::json;

    include!("mcp_coverage_tests.rs");
}
